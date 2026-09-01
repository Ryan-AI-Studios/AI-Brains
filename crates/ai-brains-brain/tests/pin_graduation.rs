#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]
#![allow(clippy::await_holding_lock)]

use ai_brains_brain::memory_synthesis::system_synthesis_principal;
use ai_brains_brain::{
    GRADUATION_CAP_ENV, GraduationMode, NightlyRunOpts, NightlyService, PIN_GRADUATION_NAMESPACE,
    graduate_pins, graduation_cap_from_env,
};
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::MemoryPinnedPayload;
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind, Payload};
use ai_brains_models::{CompletionResponse, MockProvider};
use ai_brains_store::errors::StoreError;
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use uuid::Uuid;

/// Serialize env mutation within this binary (`AI_BRAINS_GRADUATION_CAP`).
static ENV_LOCK: Mutex<()> = Mutex::new(());

type VaultFixture = (
    tempfile::TempDir,
    Arc<VaultConnection>,
    Arc<SqliteEventStore>,
);

fn open_vault() -> Result<VaultFixture, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    Ok((dir, vault, event_store))
}

fn seed_pin(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    memory_id: MemoryId,
    content: &str,
    privacy: Privacy,
) -> Result<(), Box<dyn std::error::Error>> {
    let env = EventBuilder::new(
        AggregateType::Memory,
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
        source_tag: Some("t336".into()),
        query_text: None,
    }))?;
    event_store.append_event(&env)?;
    Ok(())
}

fn count_kind(events: &[Envelope], kind: EventKind) -> usize {
    events.iter().filter(|e| e.event_type == kind).count()
}

struct FailAppendStore {
    inner: Arc<SqliteEventStore>,
}

impl EventStore for FailAppendStore {
    fn append_event(&self, envelope: &Envelope) -> ai_brains_store::errors::Result<()> {
        let _ = envelope;
        Err(StoreError::EventAppendFailed(
            "injected graduation failure".into(),
        ))
    }

    fn append_events(&self, envelopes: &[Envelope]) -> ai_brains_store::errors::Result<()> {
        let _ = envelopes;
        Err(StoreError::EventAppendFailed(
            "injected graduation failure".into(),
        ))
    }

    fn read_events(&self, aggregate_id: Uuid) -> ai_brains_store::errors::Result<Vec<Envelope>> {
        self.inner.read_events(aggregate_id)
    }

    fn read_all_events(&self) -> ai_brains_store::errors::Result<Vec<Envelope>> {
        self.inner.read_all_events()
    }

    fn get_sync_state(&self, key: &str) -> ai_brains_store::errors::Result<Option<String>> {
        self.inner.get_sync_state(key)
    }

    fn set_sync_state(&self, key: &str, value: &str) -> ai_brains_store::errors::Result<()> {
        self.inner.set_sync_state(key, value)
    }

    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> ai_brains_store::errors::Result<Option<Privacy>> {
        self.inner.get_session_privacy(session_id)
    }
}

#[tokio::test]
async fn pin_graduation__decision_prefix__decision_proposed_and_review_opened()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    let memory_id = MemoryId::from_uuid(Uuid::from_u128(0x3361));
    seed_pin(
        &event_store,
        project_id,
        memory_id,
        "DECISION: graduate pins as Proposed only",
        Privacy::LocalOnly,
    )?;

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.proposed, 1, "expected one proposal, got {report:?}");

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 1);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 1);
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 0);

    let open_reviews: i64 = vault.lock()?.query_row(
        "SELECT COUNT(*) FROM review_item_projection WHERE status = 'Open'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(
        open_reviews, 1,
        "review_item_projection must have one Open row (review list source)"
    );

    let proposed = events
        .iter()
        .find(|e| e.event_type == EventKind::DecisionProposed)
        .expect("DecisionProposed");
    assert_eq!(proposed.privacy, Privacy::LocalOnly);
    match &proposed.payload {
        Payload::DecisionProposed(p) => {
            assert_eq!(p.proposer, system_synthesis_principal());
            assert!(p.conclusion_ids.is_none());
            assert!(p.evidence_ids.is_none());
            assert!(p.title.contains("DECISION:"));
        }
        other => panic!("expected DecisionProposed, got {other:?}"),
    }
    let opened = events
        .iter()
        .find(|e| e.event_type == EventKind::ReviewItemOpened)
        .expect("ReviewItemOpened");
    match &opened.payload {
        Payload::ReviewItemOpened(p) => {
            assert!(p.related_decision_id.is_some());
            assert!(p.related_conclusion_id.is_none());
        }
        other => panic!("expected ReviewItemOpened, got {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn pin_graduation__constraint_prefix__conclusion_proposed_and_review_opened()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3362)),
        "CONSTRAINT: skip Sealed pins",
        Privacy::LocalOnly,
    )?;

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.proposed, 1);

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 1);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 1);
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);

    let proposed = events
        .iter()
        .find(|e| e.event_type == EventKind::ConclusionProposed)
        .expect("ConclusionProposed");
    match &proposed.payload {
        Payload::ConclusionProposed(p) => {
            assert!(!p.unsupported);
            assert!(p.evidence_ids.is_empty());
            assert!(p.model_provenance.is_none());
            assert_eq!(p.proposer, system_synthesis_principal());
        }
        other => panic!("expected ConclusionProposed, got {other:?}"),
    }
    let opened = events
        .iter()
        .find(|e| e.event_type == EventKind::ReviewItemOpened)
        .expect("ReviewItemOpened");
    match &opened.payload {
        Payload::ReviewItemOpened(p) => {
            assert!(p.related_conclusion_id.is_some());
            assert!(p.related_decision_id.is_none());
        }
        other => panic!("expected ReviewItemOpened, got {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn pin_graduation__invariant_prefix__conclusion_not_decision()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3363)),
        "INVARIANT: never auto-approve from pins",
        Privacy::LocalOnly,
    )?;

    graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 1);
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__hotspot_and_other__skipped() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3364)),
        "HOTSPOT: crates/ai-brains-cli/src/commands/project.rs",
        Privacy::LocalOnly,
    )?;
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3365)),
        "just a note without a pin prefix",
        Privacy::LocalOnly,
    )?;

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.proposed, 0);
    assert!(report.skipped_kind >= 2);
    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 0);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 0);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__sealed_or_never_inject__skipped() -> Result<(), Box<dyn std::error::Error>>
{
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3366)),
        "DECISION: sealed must not graduate",
        Privacy::Sealed,
    )?;
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3367)),
        "CONSTRAINT: never-inject must not graduate",
        Privacy::NeverInject,
    )?;

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.proposed, 0);
    assert!(report.skipped_privacy >= 2);
    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 0);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__no_approved_or_activated_events() -> Result<(), Box<dyn std::error::Error>>
{
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3368)),
        "DECISION: propose only",
        Privacy::LocalOnly,
    )?;
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3369)),
        "CONSTRAINT: propose only",
        Privacy::LocalOnly,
    )?;

    graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionApproved), 0);
    assert_eq!(count_kind(&events, EventKind::ConclusionActivated), 0);
    assert_eq!(count_kind(&events, EventKind::ReviewItemResolved), 0);
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 1);
    assert_eq!(count_kind(&events, EventKind::ConclusionProposed), 1);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__same_memory_second_run__idempotent()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x336A)),
        "DECISION: idempotent graduation",
        Privacy::LocalOnly,
    )?;

    let first = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    let second = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(first.proposed, 1);
    assert_eq!(second.proposed, 0);
    assert!(second.skipped_existing >= 1);

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 1);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 1);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__twelve_eligible__cap_ten_sorted() -> Result<(), Box<dyn std::error::Error>>
{
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    for i in 1u128..=12 {
        seed_pin(
            &event_store,
            project_id,
            MemoryId::from_uuid(Uuid::from_u128(i)),
            &format!("DECISION: cap candidate {i}"),
            Privacy::LocalOnly,
        )?;
    }

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.cap, 10);
    assert_eq!(report.eligible_before_cap, 12);
    assert_eq!(report.proposed, 10);

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 10);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 10);

    for i in 11u128..=12 {
        let memory_id = MemoryId::from_uuid(Uuid::from_u128(i));
        let name = format!("{project_id}:{memory_id}:decision");
        let aggregate = Uuid::new_v5(&PIN_GRADUATION_NAMESPACE, name.as_bytes());
        let existing = event_store.read_events(aggregate)?;
        assert!(
            existing.is_empty(),
            "pin {i} is beyond cap; expected no proposal events"
        );
    }
    Ok(())
}

#[tokio::test]
async fn pin_graduation__env_cap_two__emits_two() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(GRADUATION_CAP_ENV, "2");
    assert_eq!(graduation_cap_from_env(), 2);

    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    for i in 1u128..=5 {
        seed_pin(
            &event_store,
            project_id,
            MemoryId::from_uuid(Uuid::from_u128(0x3400 + i)),
            &format!("DECISION: env cap {i}"),
            Privacy::LocalOnly,
        )?;
    }

    let report = graduate_pins(
        query_store.as_ref(),
        event_store.as_ref(),
        project_id,
        false,
    )?;
    assert_eq!(report.cap, 2);
    assert_eq!(report.proposed, 2);
    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 2);
    Ok(())
}

#[tokio::test]
async fn nightly_graduation__skip_flag__no_events() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3370)),
        "DECISION: skip flag must not graduate",
        Privacy::LocalOnly,
    )?;

    let mock = Arc::new(MockProvider::new(vec![CompletionResponse {
        text: "unused".into(),
        model: "mock".into(),
    }]));
    let nightly = NightlyService::new(query_store, event_store.clone(), mock.clone(), mock);
    let count = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                graduation: GraduationMode::Skip,
                ..NightlyRunOpts::default()
            },
        )
        .await?;
    assert_eq!(count, 0);

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 0);
    Ok(())
}

#[tokio::test]
async fn nightly_graduation__dry_run__prints_count_no_append()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3371)),
        "DECISION: dry-run must not append",
        Privacy::LocalOnly,
    )?;

    let report = graduate_pins(query_store.as_ref(), event_store.as_ref(), project_id, true)?;
    assert_eq!(report.proposed, 0);
    assert_eq!(report.would_propose, 1);

    let mock = Arc::new(MockProvider::new(vec![CompletionResponse {
        text: "unused".into(),
        model: "mock".into(),
    }]));
    let nightly = NightlyService::new(query_store, event_store.clone(), mock.clone(), mock);
    nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                graduation: GraduationMode::DryRun,
                ..NightlyRunOpts::default()
            },
        )
        .await?;

    let events = event_store.read_all_events()?;
    assert_eq!(count_kind(&events, EventKind::DecisionProposed), 0);
    assert_eq!(count_kind(&events, EventKind::ReviewItemOpened), 0);
    Ok(())
}

#[tokio::test]
async fn pin_graduation__failure__nightly_still_ok() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let (_dir, vault, event_store) = open_vault()?;
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_pin(
        &event_store,
        project_id,
        MemoryId::from_uuid(Uuid::from_u128(0x3372)),
        "DECISION: fail-open nightly",
        Privacy::LocalOnly,
    )?;

    let failing: Arc<dyn EventStore> = Arc::new(FailAppendStore {
        inner: event_store.clone(),
    });
    let mock = Arc::new(MockProvider::new(vec![CompletionResponse {
        text: "unused".into(),
        model: "mock".into(),
    }]));
    let nightly = NightlyService::new(query_store, failing, mock.clone(), mock);
    let result = nightly
        .run_nightly_with(
            project_id,
            NightlyRunOpts {
                graduation: GraduationMode::Run,
                ..NightlyRunOpts::default()
            },
        )
        .await;
    assert!(result.is_ok(), "nightly must fail-open, got {result:?}");
    Ok(())
}
