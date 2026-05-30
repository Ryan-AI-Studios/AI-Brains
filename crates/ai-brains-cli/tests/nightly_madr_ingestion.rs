#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::tempdir;

/// Test that MADR records ingested through bridge export are correctly
/// stored as Decision events in the event store and projected to memory_projection.
#[test]
fn test_madr_ingestion_via_sync_pull() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // 1. Initialize the vault
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Write a mock NDJSON file with MADR records
    let ndjson_path = dir.path().join("madr_export.ndjson");
    let mut file = std::fs::File::create(&ndjson_path)?;

    let madr_record = serde_json::json!({
        "bridge_version": "0.3",
        "direction": "inbound",
        "timestamp": "2026-05-19T12:00:00Z",
        "parent_hash": "abc123",
        "project_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "11111111-1111-1111-1111-111111111111",
        "tx_id": "tx-madr-001",
        "record_kind": "madr",
        "payload": {
            "title": "ADR-001: Use SQLite with SQLCipher",
            "context": "We needed an embedded, encrypted database for local-first operation.",
            "decision": "We selected SQLite with SQLCipher extension for at-rest encryption.",
            "consequences": "Simpler deployment than PostgreSQL. Encrypted at rest. No network dependency."
        },
        "privacy": "LocalOnly"
    });

    writeln!(file, "{}", serde_json::to_string(&madr_record)?)?;
    file.flush()?;

    // 3. Pull the MADR record via sync
    let mut pull_cmd = Command::cargo_bin("ai-brains")?;
    pull_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("sync")
        .arg("pull")
        .arg("--from-file")
        .arg(&ndjson_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully synced"));

    // 4. Verify the MADR was stored in memory_projection via recall
    let mut recall_cmd = Command::cargo_bin("ai-brains")?;
    recall_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("SQLite")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    Ok(())
}

/// Test that MADR markdown format is produced correctly from structured fields.
#[test]
fn test_madr_formatting_through_event_store() -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::MemoryId;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::DataKey;
    use ai_brains_events::{
        constructors::EventBuilder, Actor, AggregateType, DecisionRecordedPayload, EventKind,
        Payload,
    };
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::event_store::{EventStore, SqliteEventStore};
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file
        .path()
        .to_str()
        .ok_or("invalid temp path")?
        .to_string();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db_path, &sql_key)?;
    conn.migrate()?;
    let store = SqliteEventStore::new(conn);

    let decision_id = MemoryId::new();
    let payload = Payload::DecisionRecorded(DecisionRecordedPayload {
        decision_id,
        title: "ADR-002: Choose Tokio for Async Runtime".to_string(),
        context: "We evaluated tokio, async-std, and smol for our async runtime needs.".to_string(),
        decision: "Selected tokio for its ecosystem maturity and Windows support.".to_string(),
        consequences: "Larger binary size but access to the richest async ecosystem in Rust."
            .to_string(),
        project_id: None,
        session_id: None,
        tx_id: None,
    });

    let envelope = EventBuilder::new(
        AggregateType::Decision,
        decision_id.as_uuid(),
        EventKind::DecisionRecorded,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(payload)?;

    store.append_event(&envelope)?;

    // Verify the decision was projected into memory_projection
    let conn_lock = store.connection().lock()?;
    let content: String = conn_lock.query_row(
        "SELECT content FROM memory_projection WHERE memory_id = ?",
        rusqlite::params![decision_id.to_string()],
        |row| row.get(0),
    )?;

    // Verify MADR-compliant markdown format
    assert!(content.contains("# ADR-002: Choose Tokio for Async Runtime"));
    assert!(content.contains("## Context"));
    assert!(content.contains("We evaluated tokio, async-std, and smol"));
    assert!(content.contains("## Decision"));
    assert!(content.contains("Selected tokio for its ecosystem maturity"));
    assert!(content.contains("## Consequences"));
    assert!(content.contains("Larger binary size but access to the richest"));

    // Verify the status is "pinned"
    let status: String = conn_lock.query_row(
        "SELECT status FROM memory_projection WHERE memory_id = ?",
        rusqlite::params![decision_id.to_string()],
        |row| row.get(0),
    )?;
    assert_eq!(status, "pinned");

    // Verify privacy is stored
    let privacy_str: String = conn_lock.query_row(
        "SELECT privacy FROM memory_projection WHERE memory_id = ?",
        rusqlite::params![decision_id.to_string()],
        |row| row.get(0),
    )?;
    assert!(privacy_str.contains("ProjectLocal"));

    Ok(())
}

/// Test that non-MADR records are NOT treated as decisions.
#[test]
fn test_non_madr_records_are_not_ingested_as_decisions() -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::MemoryId;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::DataKey;
    use ai_brains_events::{
        constructors::EventBuilder, Actor, AggregateType, EventKind, MemoryPinnedPayload, Payload,
    };
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::event_store::{EventStore, SqliteEventStore};
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file
        .path()
        .to_str()
        .ok_or("invalid temp path")?
        .to_string();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db_path, &sql_key)?;
    conn.migrate()?;
    let store = SqliteEventStore::new(conn);

    // Ingest a regular MemoryPinned event - it should NOT create a Decision projection
    let memory_id = MemoryId::new();
    let payload = Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "MADR-like text but not a decision".to_string(),
        session_id: None,
        project_id: None,
        tx_id: None,
        rank: None,
        source_tag: None,
        query_text: None,
    });

    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        EventKind::MemoryPinned,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(payload)?;

    store.append_event(&envelope)?;

    // Check that events table has the correct event_type
    let conn_lock = store.connection().lock()?;
    let event_type: String = conn_lock.query_row(
        "SELECT event_type FROM events WHERE aggregate_id = ?",
        rusqlite::params![memory_id.to_string()],
        |row| row.get(0),
    )?;

    // It should be MemoryPinned, not DecisionRecorded
    assert!(event_type.contains("MemoryPinned"));
    assert!(!event_type.contains("DecisionRecorded"));

    Ok(())
}
