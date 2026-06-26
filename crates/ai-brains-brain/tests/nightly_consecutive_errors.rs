#![allow(clippy::disallowed_methods)]

use ai_brains_brain::NightlyService;
use ai_brains_core::ids::{ProjectId, SessionId};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::{
    Payload, SessionCompletedPayload, SessionStartedPayload, UserPromptRecordedPayload,
};
use ai_brains_models::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result as ModelResult, TokenizeRequest, TokenizeResponse,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use async_trait::async_trait;
use std::sync::Arc;
use tempfile::tempdir;

struct FailingCompletionProvider;

#[async_trait]
impl ModelProvider for FailingCompletionProvider {
    async fn complete(&self, _request: CompletionRequest) -> ModelResult<CompletionResponse> {
        Err(ModelError::Timeout)
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
        "failing"
    }

    fn is_local(&self) -> bool {
        true
    }
}

fn append_completed_session(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    session_id: SessionId,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = ai_brains_core::ids::UserId::new();

    let events = vec![
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionStarted,
            ai_brains_events::Actor::User(user),
            Default::default(),
        )
        .build(Payload::SessionStarted(SessionStartedPayload {
            session_id,
            project_id,
            tx_id: None,
        }))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::UserPromptRecorded,
            ai_brains_events::Actor::User(user),
            Default::default(),
        )
        .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
            session_id,
            content: content.to_string(),
            tx_id: None,
        }))?,
        ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Session,
            session_id.as_uuid(),
            ai_brains_events::EventKind::SessionCompleted,
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

#[tokio::test]
#[allow(non_snake_case)]
async fn nightly__three_consecutive_summary_errors__aborts_remaining_sessions(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );

    let vault = Arc::new(VaultConnection::open(db_path, &key)?);
    vault.migrate()?;
    let event_store = Arc::new(SqliteEventStore::new(vault.as_ref().clone()));
    let project_id = ProjectId::new();

    let project_event = ai_brains_events::constructors::EventBuilder::new(
        ai_brains_events::AggregateType::Project,
        project_id.as_uuid(),
        ai_brains_events::EventKind::ProjectRegistered,
        ai_brains_events::Actor::User(ai_brains_core::ids::UserId::new()),
        Default::default(),
    )
    .build(Payload::ProjectRegistered(
        ai_brains_events::ProjectRegisteredPayload {
            project_id,
            name: "Nightly failure threshold test".to_string(),
            tx_id: None,
        },
    ))?;
    event_store.append_event(&project_event)?;

    for i in 0..5 {
        append_completed_session(
            event_store.as_ref(),
            project_id,
            SessionId::new(),
            &format!("session {i} should fail summarization"),
        )?;
    }

    let mut store_for_replay = SqliteEventStore::new(vault.as_ref().clone());
    store_for_replay.rebuild_projections()?;

    let provider = Arc::new(FailingCompletionProvider);
    let nightly = NightlyService::new(
        vault.clone(),
        event_store.clone(),
        provider.clone(),
        provider,
    );
    let summarized_count = nightly.run_nightly(project_id, Some(5)).await?;

    assert_eq!(summarized_count, 0);

    let conn = vault.lock()?;
    let serialized_errors: String = conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'last_nightly_errors'",
        [],
        |row| row.get(0),
    )?;
    let errors: Vec<String> = serde_json::from_str(&serialized_errors)?;
    let summary_errors = errors
        .iter()
        .filter(|error| error.starts_with("summarize_session "))
        .count();

    assert_eq!(
        summary_errors, 3,
        "nightly should stop summarizing after three consecutive errors; errors={errors:?}"
    );

    Ok(())
}
