#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::type_complexity)]

//! Cloud-policy gate on the embedding path (Codex P1-01 / P2-02).

use ai_brains_brain::EmbeddingService;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::MemoryPinnedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_models::MockProvider;
use ai_brains_models::registry::ALLOW_CLOUD_EXTRACTION_ENV;
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

/// Serialize env mutation within this binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn seed_memory(
    event_store: &SqliteEventStore,
    project_id: ProjectId,
    privacy: Privacy,
    content: &str,
) -> Result<MemoryId, Box<dyn std::error::Error>> {
    let memory_id = MemoryId::new();
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
        source_tag: Some("test".into()),
        query_text: None,
    }))?;
    event_store.append_event(&env)?;
    Ok(memory_id)
}

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
    let key = ai_brains_crypto::DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let vault = VaultConnection::open(db_path, &sql_key)?;
    vault.migrate()?;
    let vault = Arc::new(vault);
    let event_store = Arc::new(SqliteEventStore::new((*vault).clone()));
    let query_store: Arc<dyn QueryStore> = vault.clone();
    Ok((dir, vault, event_store, query_store))
}

#[tokio::test]
async fn generate_and_store__sealed_plus_remote__no_embed_no_store()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");

    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    let memory_id = seed_memory(
        &event_store,
        project_id,
        Privacy::Sealed,
        "sealed secret content",
    )?;

    let mut mock = MockProvider::new(vec![]).failing_if_called();
    mock.is_local = false;
    let mock = Arc::new(mock);
    let service = EmbeddingService::new(query_store.clone(), mock.clone());

    let stored = service
        .generate_and_store(&memory_id.to_string(), "sealed secret content")
        .await?;
    assert!(!stored, "policy denial must not store embedding");
    assert_eq!(
        mock.embed_call_count(),
        0,
        "Sealed + non-local must not call embed()"
    );

    let still_missing = query_store.get_memories_without_embeddings(10, None)?;
    assert!(
        still_missing
            .iter()
            .any(|(id, _)| id == &memory_id.to_string()),
        "memory must remain without embedding after policy skip"
    );
    Ok(())
}

#[tokio::test]
async fn generate_and_store__local_only_plus_remote__no_embed()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");

    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    let memory_id = seed_memory(
        &event_store,
        project_id,
        Privacy::LocalOnly,
        "local only content",
    )?;

    let mut mock = MockProvider::new(vec![]).failing_if_called();
    mock.is_local = false;
    let mock = Arc::new(mock);
    let service = EmbeddingService::new(query_store, mock.clone());

    let stored = service
        .generate_and_store(&memory_id.to_string(), "local only content")
        .await?;
    assert!(!stored);
    assert_eq!(mock.embed_call_count(), 0);
    Ok(())
}

#[tokio::test]
async fn generate_and_store__sealed_plus_local__embed_allowed()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);

    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    let memory_id = seed_memory(
        &event_store,
        project_id,
        Privacy::Sealed,
        "sealed but local embed ok",
    )?;

    let mock = Arc::new(MockProvider::new(vec![])); // is_local defaults true
    let service = EmbeddingService::new(query_store.clone(), mock.clone());

    let stored = service
        .generate_and_store(&memory_id.to_string(), "sealed but local embed ok")
        .await?;
    assert!(stored, "local provider may embed Sealed memory");
    assert_eq!(mock.embed_call_count(), 1);

    let missing = query_store.get_memories_without_embeddings(10, None)?;
    assert!(
        missing.iter().all(|(id, _)| id != &memory_id.to_string()),
        "embedding must be stored for local provider"
    );
    Ok(())
}

#[tokio::test]
async fn generate_and_store__cloud_ok_remote_flag_off__denied()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);

    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    let memory_id = seed_memory(
        &event_store,
        project_id,
        Privacy::CloudOk,
        "cloud ok content",
    )?;

    let mut mock = MockProvider::new(vec![]).failing_if_called();
    mock.is_local = false;
    let mock = Arc::new(mock);
    let service = EmbeddingService::new(query_store, mock.clone());

    let stored = service
        .generate_and_store(&memory_id.to_string(), "cloud ok content")
        .await?;
    assert!(!stored, "CloudOk + remote + flag off must deny");
    assert_eq!(mock.embed_call_count(), 0);
    Ok(())
}

#[tokio::test]
async fn generate_and_store__unknown_memory_id__fail_closed_sealed()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");

    let (_dir, _vault, _event_store, query_store) = open_vault()?;

    // Unparseable id → fail closed as Sealed → remote denied without embed.
    let mut mock = MockProvider::new(vec![]).failing_if_called();
    mock.is_local = false;
    let mock = Arc::new(mock);
    let service = EmbeddingService::new(query_store, mock.clone());

    let stored = service
        .generate_and_store("not-a-uuid", "orphan content")
        .await?;
    assert!(!stored);
    assert_eq!(mock.embed_call_count(), 0);
    Ok(())
}

#[tokio::test]
async fn backfill_recent__policy_denial__counts_failed_not_success()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");

    let (_dir, _vault, event_store, query_store) = open_vault()?;
    let project_id = ProjectId::new();
    let _m1 = seed_memory(&event_store, project_id, Privacy::Sealed, "s1")?;
    let _m2 = seed_memory(&event_store, project_id, Privacy::LocalOnly, "s2")?;

    let mut mock = MockProvider::new(vec![]).failing_if_called();
    mock.is_local = false;
    let mock = Arc::new(mock);
    let service = EmbeddingService::new(query_store, mock.clone());

    let (success, failed, _truncated) = service.backfill_recent(10, None).await?;
    assert_eq!(success, 0, "policy denials must not count as success");
    assert!(failed >= 2, "policy denials count as failed, got {failed}");
    assert_eq!(mock.embed_call_count(), 0);
    Ok(())
}
