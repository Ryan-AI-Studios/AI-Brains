#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::type_complexity)]

use ai_brains_brain::memory_synthesis::{
    GOVERNED_SYNTHESIS_ENV, HIERARCHICAL_SYNTHESIS_WORKFLOW_VERSION, MemorySynthesizer,
    SYSTEM_SYNTHESIS_PRINCIPAL_UUID,
};
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::MemoryPinnedPayload;
use ai_brains_events::{Actor, AggregateType, EventKind, Payload};
use ai_brains_models::{CompletionResponse, MockProvider};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use uuid::Uuid;

/// Serialize env mutation within this binary (cargo test threads; nextest isolates processes).
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn seed_level0_memories(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    seed_level0_memories_with_privacy(event_store, project_id, Privacy::LocalOnly)
}

fn seed_level0_memories_with_privacy(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    privacy: Privacy,
) -> Result<(), Box<dyn std::error::Error>> {
    let actor = Actor::System;
    for content in ["alpha memory one", "alpha memory two", "alpha memory three"] {
        let memory_id = MemoryId::new();
        let env = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            actor.clone(),
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
    }
    Ok(())
}

#[tokio::test]
async fn run_synthesis__flag_off__emits_memory_synthesized()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::remove(GOVERNED_SYNTHESIS_ENV);

    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_level0_memories(&event_store, project_id)?;

    let mock = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text:
                r#"{"title":"t","aggregated_context":"c","invariants":[],"cumulative_progress":[]}"#
                    .into(),
            model: "mock".into(),
        },
        CompletionResponse {
            text: "SUPPORTED".into(),
            model: "mock".into(),
        },
    ]));
    let synth = MemorySynthesizer::new(query_store, event_store.clone(), mock);
    let n = synth.run_synthesis(1, project_id).await?;
    assert!(n >= 1, "expected at least one synthesis, got {n}");

    let events = event_store.read_all_events()?;
    let synth_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventKind::MemorySynthesized)
        .collect();
    assert!(
        !synth_events.is_empty(),
        "legacy path must emit MemorySynthesized"
    );
    assert!(
        events
            .iter()
            .all(|e| e.event_type != EventKind::ConclusionProposed),
        "flag off must not emit ConclusionProposed"
    );
    Ok(())
}

#[tokio::test]
async fn run_synthesis__flag_on__emits_conclusion_proposed_candidate()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(GOVERNED_SYNTHESIS_ENV, "1");

    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_level0_memories(&event_store, project_id)?;

    let mock = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: r#"{"title":"gov","aggregated_context":"c","invariants":[],"cumulative_progress":[]}"#.into(),
            model: "mock".into(),
        },
        CompletionResponse {
            text: "SUPPORTED".into(),
            model: "mock".into(),
        },
    ]));
    let synth = MemorySynthesizer::new(query_store, event_store.clone(), mock);
    let n = synth.run_synthesis(1, project_id).await?;
    assert!(n >= 1);

    let events = event_store.read_all_events()?;
    let proposed: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventKind::ConclusionProposed)
        .collect();
    assert!(
        !proposed.is_empty(),
        "governed path must emit ConclusionProposed"
    );
    match &proposed[0].payload {
        Payload::ConclusionProposed(p) => {
            assert!(p.unsupported, "synthesis without evidence is unsupported");
            assert!(!p.statement.is_empty());
            assert!(p.scope.starts_with("Repository:"));
            assert!(p.evidence_ids.is_empty());
            let prov = p
                .model_provenance
                .as_ref()
                .expect("governed path must attach model_provenance");
            assert!(
                !prov.provider.is_empty() && !prov.model.is_empty(),
                "provenance must have non-empty provider AND model: {prov:?}"
            );
            assert_eq!(
                prov.model_version.as_deref(),
                Some("unknown"),
                "model_version must be concrete, not None"
            );
            assert_eq!(
                prov.workflow_version.as_deref(),
                Some(HIERARCHICAL_SYNTHESIS_WORKFLOW_VERSION),
                "workflow_version must be hierarchical-synthesis/v1"
            );
            assert!(
                prov.output_hash.as_ref().is_some_and(|h| !h.is_empty()),
                "output_hash must be present"
            );
            assert_ne!(
                p.proposer.as_uuid(),
                Uuid::nil(),
                "proposer must not be nil UUID"
            );
            assert_eq!(
                p.proposer.as_uuid(),
                SYSTEM_SYNTHESIS_PRINCIPAL_UUID,
                "proposer must be well-known system synthesis principal"
            );
        }
        other => panic!("expected ConclusionProposed, got {other:?}"),
    }
    let mem_synth = events
        .iter()
        .filter(|e| e.event_type == EventKind::MemorySynthesized)
        .count();
    assert_eq!(
        mem_synth, 0,
        "governed path must not emit MemorySynthesized"
    );
    Ok(())
}

#[tokio::test]
async fn run_synthesis__flag_on__inherits_strictest_eligible_source_privacy()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(GOVERNED_SYNTHESIS_ENV, "1");

    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    // CloudOk + LocalOnly mix: LocalOnly is stricter among eligible (non-Sealed) sources.
    seed_level0_memories_with_privacy(&event_store, project_id, Privacy::CloudOk)?;
    // Extra LocalOnly memory so cluster strictest becomes LocalOnly.
    let actor = Actor::System;
    let memory_id = MemoryId::new();
    let env = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "local only stricter".into(),
        session_id: None,
        project_id: Some(project_id),
        tx_id: None,
        rank: Some(0),
        source_tag: Some("test".into()),
        query_text: None,
    }))?;
    event_store.append_event(&env)?;

    let mock = Arc::new(MockProvider::new(vec![
        CompletionResponse {
            text: r#"{"title":"gov","aggregated_context":"c","invariants":[],"cumulative_progress":[]}"#.into(),
            model: "mock".into(),
        },
        CompletionResponse {
            text: "SUPPORTED".into(),
            model: "mock".into(),
        },
    ]));
    let synth = MemorySynthesizer::new(query_store, event_store.clone(), mock);
    let n = synth.run_synthesis(1, project_id).await?;
    assert!(n >= 1);

    let events = event_store.read_all_events()?;
    let proposed: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventKind::ConclusionProposed)
        .collect();
    assert!(!proposed.is_empty());
    assert_eq!(
        proposed[0].privacy,
        Privacy::LocalOnly,
        "governed synthesis must inherit strictest eligible source privacy, got {:?}",
        proposed[0].privacy
    );
    Ok(())
}

#[tokio::test]
async fn run_synthesis__sealed_sources__excluded_before_model_call()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(GOVERNED_SYNTHESIS_ENV, "1");

    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_level0_memories_with_privacy(&event_store, project_id, Privacy::Sealed)?;

    // Empty mock queue: any model call would fail. Exclusion must yield 0 without calling.
    let mock = Arc::new(MockProvider::new(vec![]));
    let synth = MemorySynthesizer::new(query_store, event_store.clone(), mock);
    let n = synth.run_synthesis(1, project_id).await?;
    assert_eq!(n, 0, "Sealed sources must not be auto-synthesized");

    let events = event_store.read_all_events()?;
    assert!(
        events
            .iter()
            .all(|e| e.event_type != EventKind::ConclusionProposed
                && e.event_type != EventKind::MemorySynthesized),
        "no synthesis events for Sealed-only sources"
    );
    Ok(())
}

#[tokio::test]
async fn run_synthesis__local_only__refuses_cloud_provider()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(GOVERNED_SYNTHESIS_ENV, "1");

    let dir = tempdir()?;
    let db_path = dir.path().join("vault.db");
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    let project_id = ProjectId::new();
    seed_level0_memories_with_privacy(&event_store, project_id, Privacy::LocalOnly)?;

    let mut mock = MockProvider::new(vec![]);
    mock.is_local = false;
    let mock = Arc::new(mock);
    let synth = MemorySynthesizer::new(query_store, event_store.clone(), mock);
    let n = synth.run_synthesis(1, project_id).await?;
    assert_eq!(
        n, 0,
        "LocalOnly clusters must refuse non-local providers before model call"
    );
    Ok(())
}
