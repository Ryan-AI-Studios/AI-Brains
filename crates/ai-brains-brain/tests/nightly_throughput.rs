#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]
#![allow(clippy::await_holding_lock)]

//! T338 nightly throughput: deadline, probe skip, error budget, embed catch-up.

use ai_brains_brain::{
    EmbeddingService, GraduationMode, NightlyDeadline, NightlyRunOpts, NightlyService,
    parse_deadline_minutes,
};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::{
    Actor, AggregateType, Payload, SessionCompletedPayload, SessionStartedPayload,
    UserPromptRecordedPayload, constructors::EventBuilder, payload::MemoryPinnedPayload,
};
use ai_brains_models::MockProvider;
use ai_brains_models::llama_cpp::ProbeStatus;
use ai_brains_models::registry::ALLOW_CLOUD_EXTRACTION_ENV;
use ai_brains_models::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result as ModelResult, TokenizeRequest, TokenizeResponse,
};
use ai_brains_store::QueryStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[allow(clippy::type_complexity)]
fn open_vault() -> Result<
    (
        tempfile::TempDir,
        Arc<VaultConnection>,
        Arc<SqliteEventStore>,
        Arc<dyn QueryStore>,
    ),
    Box<dyn std::error::Error>,
> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let vault = Arc::new(VaultConnection::open(db_path, &sql_key)?);
    vault.migrate()?;
    let event_store = Arc::new(SqliteEventStore::new(vault.as_ref().clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    Ok((dir, vault, event_store, query_store))
}

fn register_project(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
        Default::default(),
    )
    .build(Payload::ProjectRegistered(
        ai_brains_events::ProjectRegisteredPayload {
            project_id,
            name: "T338 throughput".to_string(),
            tx_id: None,
        },
    ))?;
    event_store.append_event(&event)?;
    Ok(())
}

fn append_completed_session(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    session_id: SessionId,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = ai_brains_core::ids::UserId::new();
    let events = vec![
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::Actor::User(user),
            Default::default(),
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
            tx_id: None,
        }))?,
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::Actor::User(user),
            Default::default(),
        )
        .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
            session_id,
            content: content.to_string(),
            tx_id: None,
            turn_id: None,
        }))?,
        EventBuilder::new(
            AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::Actor::User(user),
            Default::default(),
        )
        .build(Payload::SessionCompleted(SessionCompletedPayload {
            session_id,
        }))?,
    ];
    for event in events {
        event_store.append_event(&event)?;
    }
    Ok(())
}

fn seed_memory(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    privacy: Privacy,
    content: &str,
) -> Result<MemoryId, Box<dyn std::error::Error>> {
    let memory_id = MemoryId::new();
    let env = EventBuilder::new(
        ai_brains_events::AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        privacy,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.into(),
        session_id: None,
        project_id: Some(project_id),
        tx_id: None,
        rank: Some(0),
        source_tag: Some("test".into()),
        query_text: None,
    }))?;
    event_store.append_event(&env)?;
    Ok(memory_id)
}

fn read_abort(
    vault: &VaultConnection,
) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
    let conn = vault.lock()?;
    let raw: String = conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'last_nightly_aborted'",
        [],
        |row| row.get(0),
    )?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let early = v.get("early").and_then(|x| x.as_bool()).unwrap_or(false);
    let reason = v.get("reason").and_then(|x| x.as_str()).map(str::to_string);
    Ok((early, reason))
}

fn succeeding_mock() -> Arc<MockProvider> {
    Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: "Summary of the session.".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "mock".to_string(),
        },
        CompletionResponse {
            text: "NO RECIPE".to_string(),
            model: "mock".to_string(),
        },
    ]))
}

struct ErrorBudgetProvider {
    summary_attempts: AtomicUsize,
}

#[async_trait]
impl ModelProvider for ErrorBudgetProvider {
    async fn complete(&self, request: CompletionRequest) -> ModelResult<CompletionResponse> {
        if request
            .prompt
            .contains("Analyze the following developer session")
        {
            let n = self.summary_attempts.fetch_add(1, Ordering::SeqCst);
            if n.is_multiple_of(2) {
                return Err(ModelError::Timeout);
            }
            return Ok(CompletionResponse {
                text: "Summary of the session.".to_string(),
                model: "budget".to_string(),
            });
        }
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "budget".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            vector: vec![0.0; 1536],
        })
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "error-budget"
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// Records `complete` vs `embed` call order across the two nightly providers (CX3).
struct PhaseOrderProvider {
    phases: Arc<Mutex<Vec<&'static str>>>,
    kind: &'static str,
}

#[async_trait]
impl ModelProvider for PhaseOrderProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        self.phases
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push("complete");
        Ok(CompletionResponse {
            text:
                r#"{"title":"t","aggregated_context":"c","invariants":[],"cumulative_progress":[]}"#
                    .to_string(),
            model: self.kind.to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        self.phases
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push("embed");
        Ok(EmbeddingResponse {
            vector: vec![0.0; 1536],
        })
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        self.kind
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// T343 AC1: count `embed_batch` invocations (never `embed_calls`).
struct CountingBatchProvider {
    embed_batch_calls: AtomicUsize,
    embed_calls: AtomicUsize,
    captured_texts: Mutex<Vec<String>>,
    batch_lens: Mutex<Vec<usize>>,
}

impl CountingBatchProvider {
    fn new() -> Self {
        Self {
            embed_batch_calls: AtomicUsize::new(0),
            embed_calls: AtomicUsize::new(0),
            captured_texts: Mutex::new(Vec::new()),
            batch_lens: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl ModelProvider for CountingBatchProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "count-batch".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        Ok(EmbeddingResponse {
            vector: vec![0.1; 8],
        })
    }

    async fn embed_batch(&self, texts: Vec<String>) -> ModelResult<Vec<EmbeddingResponse>> {
        self.embed_batch_calls.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut lens) = self.batch_lens.lock() {
            lens.push(texts.len());
        }
        if let Ok(mut cap) = self.captured_texts.lock() {
            cap.extend(texts.iter().cloned());
        }
        Ok(texts
            .iter()
            .map(|_| EmbeddingResponse {
                vector: vec![0.25; 8],
            })
            .collect())
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "count-batch"
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// T351 AC1: orthogonal unit vectors per chunk so tail influences the mean-pooled BLOB.
struct OrthogonalChunkProvider {
    embed_calls: AtomicUsize,
    batch_calls: AtomicUsize,
}

impl OrthogonalChunkProvider {
    fn new() -> Self {
        Self {
            embed_calls: AtomicUsize::new(0),
            batch_calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl ModelProvider for OrthogonalChunkProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "ortho".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        Ok(EmbeddingResponse {
            vector: vec![1.0, 0.0],
        })
    }

    async fn embed_batch(&self, texts: Vec<String>) -> ModelResult<Vec<EmbeddingResponse>> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        Ok(texts
            .iter()
            .enumerate()
            .map(|(i, _)| EmbeddingResponse {
                vector: if i == 0 {
                    vec![1.0, 0.0]
                } else {
                    vec![0.0, 1.0]
                },
            })
            .collect())
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "ortho-chunk"
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// AC9: unequal chunk dims must skip store (not helper-only).
struct DimMismatchBatchProvider;

#[async_trait]
impl ModelProvider for DimMismatchBatchProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "dim-mismatch".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            vector: vec![1.0, 0.0],
        })
    }

    async fn embed_batch(&self, texts: Vec<String>) -> ModelResult<Vec<EmbeddingResponse>> {
        Ok(texts
            .iter()
            .enumerate()
            .map(|(i, _)| EmbeddingResponse {
                vector: if i == 0 { vec![1.0, 0.0] } else { vec![0.0] },
            })
            .collect())
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "dim-mismatch"
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// Isolated/one-shot long path must reject embed_batch length ≠ chunk count.
struct ShortCountBatchProvider;

#[async_trait]
impl ModelProvider for ShortCountBatchProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "short-count".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            vector: vec![1.0, 0.0],
        })
    }

    async fn embed_batch(&self, _texts: Vec<String>) -> ModelResult<Vec<EmbeddingResponse>> {
        Ok(vec![EmbeddingResponse {
            vector: vec![1.0, 0.0],
        }])
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "short-count"
    }

    fn is_local(&self) -> bool {
        true
    }
}

/// AC9: non-finite chunk vectors skip store.
struct NanBatchProvider;

#[async_trait]
impl ModelProvider for NanBatchProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Ok(CompletionResponse {
            text: "NO CONFLICT".to_string(),
            model: "nan-batch".to_string(),
        })
    }

    async fn embed(&self, _request: EmbeddingRequest) -> ModelResult<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            vector: vec![1.0, 0.0],
        })
    }

    async fn embed_batch(&self, texts: Vec<String>) -> ModelResult<Vec<EmbeddingResponse>> {
        Ok(texts
            .iter()
            .map(|_| EmbeddingResponse {
                vector: vec![f32::NAN, 0.0],
            })
            .collect())
    }

    async fn tokenize(&self, request: TokenizeRequest) -> ModelResult<TokenizeResponse> {
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "nan-batch"
    }

    fn is_local(&self) -> bool {
        true
    }
}

#[test]
fn nightly__deadline_unparseable__defaults_150() {
    assert_eq!(parse_deadline_minutes(Some("abc")), 150);
    assert_eq!(parse_deadline_minutes(Some("-1")), 150);
    assert_eq!(parse_deadline_minutes(Some("0")), 0);
}

#[tokio::test]
async fn nightly__deadline_already_expired__skips_summarize()
-> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..3 {
        append_completed_session(
            event_store.as_ref(),
            project_id,
            SessionId::new(),
            &format!("session {i}"),
        )?;
    }
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let mock = succeeding_mock();
    let nightly = NightlyService::new(query_store, event_store, mock.clone(), mock);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                batch_size: Some(3),
                deadline: NightlyDeadline::already_expired(),
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(count, 0);
    let (early, reason) = read_abort(vault.as_ref())?;
    assert!(early);
    assert_eq!(reason.as_deref(), Some("deadline"));
    Ok(())
}

#[tokio::test]
async fn nightly__deadline_env_zero__skips_summarize() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set("AI_BRAINS_NIGHTLY_DEADLINE_MINUTES", "0");
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    append_completed_session(
        event_store.as_ref(),
        project_id,
        SessionId::new(),
        "env zero session",
    )?;
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let mock = succeeding_mock();
    let nightly = NightlyService::new(query_store, event_store, mock.clone(), mock);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                deadline: NightlyDeadline::from_env(),
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(count, 0);
    let (early, reason) = read_abort(vault.as_ref())?;
    assert!(early);
    assert_eq!(reason.as_deref(), Some("deadline"));
    Ok(())
}

#[tokio::test]
async fn nightly__completion_probe_not_ok__skips_summarize()
-> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    append_completed_session(
        event_store.as_ref(),
        project_id,
        SessionId::new(),
        "probe skip session",
    )?;
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let mock = succeeding_mock();
    let nightly = NightlyService::new(query_store, event_store, mock.clone(), mock);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                completion_probe: ProbeStatus::Down,
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(count, 0);
    let (early, reason) = read_abort(vault.as_ref())?;
    assert!(early);
    assert_eq!(reason.as_deref(), Some("completion_probe"));
    Ok(())
}

#[tokio::test]
async fn nightly__env_batch_1__caps_summarize() -> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..3 {
        append_completed_session(
            event_store.as_ref(),
            project_id,
            SessionId::new(),
            &format!("batch session {i}"),
        )?;
    }
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let mock = succeeding_mock();
    let nightly = NightlyService::new(query_store, event_store, mock.clone(), mock);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                batch_size: Some(1),
                deadline: NightlyDeadline::from_minutes(150),
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(count, 1);
    Ok(())
}

#[tokio::test]
async fn nightly__total_error_budget_20__aborts() -> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..45 {
        append_completed_session(
            event_store.as_ref(),
            project_id,
            SessionId::new(),
            &format!("budget session {i}"),
        )?;
    }
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let provider = Arc::new(ErrorBudgetProvider {
        summary_attempts: AtomicUsize::new(0),
    });
    let nightly = NightlyService::new(query_store, event_store, provider.clone(), provider);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                batch_size: None,
                deadline: NightlyDeadline::from_minutes(150),
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert!(count > 0, "interleaved successes should summarize some");
    let conn = vault.lock()?;
    let serialized: String = conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'last_nightly_errors'",
        [],
        |row| row.get(0),
    )?;
    let errors: Vec<String> = serde_json::from_str(&serialized)?;
    let summary_errors = errors
        .iter()
        .filter(|e| e.starts_with("summarize_session "))
        .count();
    assert_eq!(
        summary_errors, 20,
        "error budget must stop at 20 summarize errors; errors={errors:?}"
    );
    drop(conn);
    let (early, reason) = read_abort(vault.as_ref())?;
    assert!(early);
    assert_eq!(reason.as_deref(), Some("error_budget"));
    Ok(())
}

#[tokio::test]
async fn backfill_recent__old_pinned_null_embedding__included_when_since_none()
-> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "old pinned content",
    )?;
    {
        let conn = vault.lock()?;
        conn.execute(
            "UPDATE memory_projection SET updated_at = datetime('now', '-30 days') WHERE memory_id = ?",
            [id.to_string()],
        )?;
    }
    let mock = Arc::new(MockProvider::new(vec![]));
    let service = EmbeddingService::new(query_store.clone(), mock);
    let missed = query_store.get_memories_without_embeddings(10, Some(7))?;
    assert!(
        missed.iter().all(|(mid, _)| mid != &id.to_string()),
        "Some(7) must miss 30-day-old row"
    );
    let (success, failed, _truncated) = service.backfill_recent(10, None).await?;
    assert_eq!(failed, 0);
    assert!(success >= 1, "since_days=None must embed the old pin");
    Ok(())
}

#[tokio::test]
async fn nightly_embed__deadline_mid_chunks__stops() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _chunk = TempEnv::set("AI_BRAINS_EMBED_CHUNK", "2");
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..4 {
        let id = seed_memory(
            event_store.as_ref(),
            project_id,
            Privacy::LocalOnly,
            &format!("chunk mem {i}"),
        )?;
        let conn = vault.lock()?;
        conn.execute(
            "UPDATE memory_projection SET updated_at = datetime('now', ?) WHERE memory_id = ?",
            rusqlite::params![format!("-{} seconds", 10 - i), id.to_string()],
        )?;
        drop(conn);
        let _ = id;
    }
    let mock = Arc::new(MockProvider::new(vec![]));
    let nightly = NightlyService::new(query_store.clone(), event_store, mock.clone(), mock);
    nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                deadline: NightlyDeadline::expire_after_checks(1),
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    let remaining = query_store.count_pinned_without_embeddings()?;
    assert!(
        remaining > 0 && remaining < 4,
        "first chunk of 2 should run, not all 4; remaining={remaining}"
    );
    Ok(())
}

#[tokio::test]
async fn nightly_embed__poisoned_head_policy_denied__progresses_past()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _allow = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let older_a = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::CloudOk,
        "older a",
    )?;
    let older_b = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::CloudOk,
        "older b",
    )?;
    let newest = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::Sealed,
        "poisoned newest",
    )?;
    {
        let conn = vault.lock()?;
        conn.execute(
            "UPDATE memory_projection SET updated_at = datetime('now', '-30 seconds') WHERE memory_id = ?",
            [older_a.to_string()],
        )?;
        conn.execute(
            "UPDATE memory_projection SET updated_at = datetime('now', '-20 seconds') WHERE memory_id = ?",
            [older_b.to_string()],
        )?;
        conn.execute(
            "UPDATE memory_projection SET updated_at = datetime('now', '-10 seconds') WHERE memory_id = ?",
            [newest.to_string()],
        )?;
    }
    let mut mock = MockProvider::new(vec![]);
    mock.is_local = false;
    let mock = Arc::new(mock);
    let nightly = NightlyService::new(query_store.clone(), event_store.clone(), mock.clone(), mock);
    nightly
        .run_nightly_with(project_id, NightlyRunOpts::default())
        .await?;
    let conn = vault.lock()?;
    let success: String = conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'last_embedding_backfill_count'",
        [],
        |row| row.get(0),
    )?;
    let failed: String = conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'last_embedding_backfill_failed'",
        [],
        |row| row.get(0),
    )?;
    let success_n: usize = success.parse()?;
    let failed_n: usize = failed.parse()?;
    assert!(
        success_n >= 2,
        "two CloudOk rows must embed; success={success_n}"
    );
    assert!(
        failed_n >= 1,
        "poisoned Sealed head must count as failed; failed={failed_n}"
    );
    drop(conn);
    let still_null = query_store.get_memories_without_embeddings(10, None)?;
    assert!(
        still_null.iter().any(|(id, _)| id == &newest.to_string()),
        "policy-denied newest must remain NULL"
    );
    Ok(())
}

#[tokio::test]
async fn nightly_embed__embedding_probe_not_ok__skips_catchup_and_stale()
-> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let null_id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "null embed for catch-up",
    )?;
    let stale_id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "stale embed for refresh",
    )?;
    query_store.store_embedding(&stale_id.to_string(), &[0u8; 16])?;
    {
        let conn = vault.lock()?;
        conn.execute(
            "UPDATE memory_projection SET embedding_generated_at = datetime('now', '-40 days') WHERE memory_id = ?",
            [stale_id.to_string()],
        )?;
    }
    let mock = Arc::new(MockProvider::new(vec![]));
    let nightly = NightlyService::new(query_store, event_store, mock.clone(), mock.clone());
    nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                embedding_probe: ProbeStatus::Down,
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(
        mock.embed_call_count(),
        0,
        "embedding probe Down must skip catch-up and stale refresh"
    );
    let still = vault.get_memories_without_embeddings(10, None)?;
    assert!(
        still.iter().any(|(id, _)| id == &null_id.to_string()),
        "NULL pin must remain unembedded when probe skips catch-up"
    );
    let (early, reason) = read_abort(vault.as_ref())?;
    assert!(early);
    assert_eq!(reason.as_deref(), Some("embedding_probe"));
    Ok(())
}

#[tokio::test]
async fn nightly_embed__catchup__runs_before_synthesis() -> Result<(), Box<dyn std::error::Error>> {
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "catch-up before synthesis alpha",
    )?;
    seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "catch-up before synthesis beta",
    )?;
    let mut replay = SqliteEventStore::new(vault.as_ref().clone());
    replay.rebuild_projections()?;

    let phases = Arc::new(Mutex::new(Vec::new()));
    let completion = Arc::new(PhaseOrderProvider {
        phases: phases.clone(),
        kind: "completion",
    });
    let embedding = Arc::new(PhaseOrderProvider {
        phases: phases.clone(),
        kind: "embedding",
    });
    let nightly = NightlyService::new(query_store, event_store, completion, embedding);
    nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                graduation: GraduationMode::Skip,
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    let order = phases.lock().unwrap_or_else(|e| e.into_inner());
    let embed_at = order.iter().position(|p| *p == "embed");
    let complete_at = order.iter().position(|p| *p == "complete");
    assert!(
        embed_at.is_some(),
        "catch-up must call embed; order={order:?}"
    );
    assert!(
        complete_at.is_some(),
        "two pinned memories must reach synthesis complete; order={order:?}"
    );
    assert!(
        embed_at < complete_at,
        "embedding catch-up must run before synthesis (spec summarize then embed); order={order:?}"
    );
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__sixteen_memories_batch_eight__two_http()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _batch = TempEnv::set("AI_BRAINS_EMBED_HTTP_BATCH", "8");
    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..16 {
        seed_memory(
            event_store.as_ref(),
            project_id,
            Privacy::LocalOnly,
            &format!("batch mem {i}"),
        )?;
    }
    let provider = Arc::new(CountingBatchProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let (success, failed, truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(failed, 0);
    assert_eq!(truncated, 0);
    assert_eq!(success, 16);
    assert_eq!(
        provider.embed_batch_calls.load(Ordering::SeqCst),
        2,
        "16 memories at batch 8 must be two HTTP embed_batch calls"
    );
    assert_eq!(
        provider.embed_calls.load(Ordering::SeqCst),
        0,
        "packer must call embed_batch, not embed"
    );
    let lens = provider
        .batch_lens
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(&*lens, &[8, 8]);
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__ten_memories_batch_eight__flushes_trailing_two()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _batch = TempEnv::set("AI_BRAINS_EMBED_HTTP_BATCH", "8");
    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    for i in 0..10 {
        seed_memory(
            event_store.as_ref(),
            project_id,
            Privacy::LocalOnly,
            &format!("trail mem {i}"),
        )?;
    }
    let provider = Arc::new(CountingBatchProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let (success, failed, _truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(failed, 0);
    assert_eq!(success, 10);
    assert_eq!(
        provider.embed_batch_calls.load(Ordering::SeqCst),
        2,
        "page of 10 at batch 8 must flush 8 then trailing 2"
    );
    let lens = provider
        .batch_lens
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(&*lens, &[8, 2]);
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__over_2048_scalar_memory__truncated_counted_and_stored()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _batch = TempEnv::set("AI_BRAINS_EMBED_HTTP_BATCH", "8");
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let long = "a".repeat(3000);
    let id = seed_memory(event_store.as_ref(), project_id, Privacy::LocalOnly, &long)?;
    let provider = Arc::new(CountingBatchProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let (success, failed, truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(failed, 0);
    assert_eq!(success, 1);
    assert_eq!(
        truncated, 0,
        "3000 ASCII scalars fit in 2 chunks; truncated must be 0, got {truncated}"
    );
    {
        let conn = vault.lock()?;
        let blob: Option<Vec<u8>> = conn.query_row(
            "SELECT embedding FROM memory_projection WHERE memory_id = ?",
            [id.to_string()],
            |row| row.get(0),
        )?;
        assert!(
            blob.as_ref().is_some_and(|b| !b.is_empty()),
            "chunked memory must still store a vector"
        );
    }
    let captured = provider
        .captured_texts
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(captured.len(), 2, "3000 scalars → 2 chunks");
    for t in captured.iter() {
        assert!(
            t.chars().count() <= 2048,
            "POST body must be <=2048 scalars, got {}",
            t.chars().count()
        );
        assert!(std::str::from_utf8(t.as_bytes()).is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__over_8192_scalar_memory__truncated_counted_and_stored()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _batch = TempEnv::set("AI_BRAINS_EMBED_HTTP_BATCH", "8");
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let long = "a".repeat(9000);
    let id = seed_memory(event_store.as_ref(), project_id, Privacy::LocalOnly, &long)?;
    let provider = Arc::new(CountingBatchProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let (success, failed, truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(failed, 0);
    assert_eq!(success, 1);
    assert!(
        truncated >= 1,
        "ASCII >8192 scalars must increment truncated, got {truncated}"
    );
    {
        let conn = vault.lock()?;
        let blob: Option<Vec<u8>> = conn.query_row(
            "SELECT embedding FROM memory_projection WHERE memory_id = ?",
            [id.to_string()],
            |row| row.get(0),
        )?;
        assert!(blob.as_ref().is_some_and(|b| !b.is_empty()));
    }
    let captured = provider
        .captured_texts
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(captured.len(), 4);
    Ok(())
}

#[tokio::test]
async fn generate_and_store__short_memory__one_embed_not_batch()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "short pin",
    )?;
    let provider = Arc::new(OrthogonalChunkProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let stored = service
        .generate_and_store(&id.to_string(), "short pin")
        .await?;
    assert!(stored);
    assert_eq!(provider.embed_calls.load(Ordering::SeqCst), 1);
    assert_eq!(provider.batch_calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn generate_and_store__chunk_meanpool__tail_influences_blob()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let content = format!("{}{}", "H".repeat(2048), "T".repeat(2048));
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        &content,
    )?;
    let provider = Arc::new(OrthogonalChunkProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let stored = service
        .generate_and_store(&id.to_string(), &content)
        .await?;
    assert!(stored);
    let blob: Vec<u8> = {
        let conn = vault.lock()?;
        conn.query_row(
            "SELECT embedding FROM memory_projection WHERE memory_id = ?",
            [id.to_string()],
            |row| row.get(0),
        )?
    };
    let stored_vec = bytes_to_f32(&blob);
    let tail = [0.0_f32, 1.0];
    let prefix = [1.0_f32, 0.0];
    let cos_tail = cosine(&stored_vec, &tail);
    let cos_prefix_to_tail = cosine(&prefix, &tail);
    assert!(
        cos_tail > cos_prefix_to_tail,
        "mean-pool must beat prefix-only vs tail; stored·tail={cos_tail} prefix·tail={cos_prefix_to_tail}"
    );
    let n = stored_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (n - 1.0).abs() < 1e-4,
        "stored mean must be unit-norm, got {n}"
    );
    assert_eq!(provider.embed_calls.load(Ordering::SeqCst), 0);
    assert_eq!(provider.batch_calls.load(Ordering::SeqCst), 1);
    Ok(())
}

fn two_chunk_content() -> String {
    format!("{}{}", "H".repeat(2048), "T".repeat(2048))
}

fn embedding_blob(
    vault: &VaultConnection,
    id: &MemoryId,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let conn = vault.lock()?;
    Ok(conn.query_row(
        "SELECT embedding FROM memory_projection WHERE memory_id = ?",
        [id.to_string()],
        |row| row.get(0),
    )?)
}

#[tokio::test]
async fn generate_and_store__chunk_meanpool__dim_mismatch__skips_store()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let content = two_chunk_content();
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        &content,
    )?;
    let service = EmbeddingService::new(query_store, Arc::new(DimMismatchBatchProvider));
    let stored = service
        .generate_and_store(&id.to_string(), &content)
        .await?;
    assert!(!stored);
    assert!(
        embedding_blob(&vault, &id)?.is_none(),
        "dim mismatch must skip store"
    );
    Ok(())
}

#[tokio::test]
async fn generate_and_store__chunk_meanpool__batch_len_mismatch__skips_store()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let content = two_chunk_content();
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        &content,
    )?;
    let service = EmbeddingService::new(query_store, Arc::new(ShortCountBatchProvider));
    let stored = service
        .generate_and_store(&id.to_string(), &content)
        .await?;
    assert!(!stored);
    assert!(
        embedding_blob(&vault, &id)?.is_none(),
        "embed_batch length mismatch must skip store"
    );
    Ok(())
}

#[tokio::test]
async fn generate_and_store__chunk_meanpool__nan_vector__skips_store()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let content = two_chunk_content();
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        &content,
    )?;
    let service = EmbeddingService::new(query_store, Arc::new(NanBatchProvider));
    let stored = service
        .generate_and_store(&id.to_string(), &content)
        .await?;
    assert!(!stored);
    assert!(
        embedding_blob(&vault, &id)?.is_none(),
        "non-finite mean must skip store"
    );
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__long_memory__batch_len_mismatch__counts_failed()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    let content = two_chunk_content();
    let id = seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        &content,
    )?;
    let service = EmbeddingService::new(query_store, Arc::new(ShortCountBatchProvider));
    let (success, failed, _truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(success, 0);
    assert_eq!(failed, 1);
    assert!(
        embedding_blob(&vault, &id)?.is_none(),
        "isolated long path must not store on batch length mismatch"
    );
    Ok(())
}

#[tokio::test]
async fn backfill_catchup__long_memory__isolated_chunk_batch_meanpool()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _batch = TempEnv::set("AI_BRAINS_EMBED_HTTP_BATCH", "8");
    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    register_project(event_store.as_ref(), project_id)?;
    seed_memory(
        event_store.as_ref(),
        project_id,
        Privacy::LocalOnly,
        "short",
    )?;
    let long = format!("{}{}", "H".repeat(2048), "T".repeat(500));
    seed_memory(event_store.as_ref(), project_id, Privacy::LocalOnly, &long)?;
    let provider = Arc::new(CountingBatchProvider::new());
    let service = EmbeddingService::new(query_store, provider.clone());
    let (success, failed, truncated) = service
        .backfill_catchup(&NightlyDeadline::from_minutes(150), 200)
        .await?;
    assert_eq!(failed, 0);
    assert_eq!(success, 2);
    assert_eq!(truncated, 0);
    let lens = provider
        .batch_lens
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert!(
        lens.contains(&2),
        "long memory must be an isolated chunk batch of 2, got {lens:?}"
    );
    assert!(
        !lens.iter().any(|&n| n > 2),
        "must not mix long-memory chunks into the 8-slot page; lens={lens:?}"
    );
    Ok(())
}

fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na <= 1e-12 || nb <= 1e-12 {
        0.0
    } else {
        dot / (na * nb)
    }
}
