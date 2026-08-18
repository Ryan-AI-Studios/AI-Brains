//! T262 AC4/AC5 — TurnProjection uses payload turn_id when present.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::ids::{ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectRegisteredPayload, SessionStartedPayload,
    UserPromptRecordedPayload,
};
use ai_brains_store::{EventStore, SqliteEventStore, VaultConnection};
use tempfile::tempdir;

fn open_store() -> (tempfile::TempDir, SqliteEventStore) {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("v.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db, &sql_key).expect("open");
    conn.migrate().expect("migrate");
    (dir, SqliteEventStore::new(conn))
}

fn seed_session(store: &SqliteEventStore) -> (ProjectId, SessionId) {
    let project_id = ProjectId::new();
    let session_id = SessionId::new();
    let actor = Actor::System;
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Project,
                project_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                project_id,
                name: "t262".into(),
                tx_id: None,
            }))
            .expect("project"),
        )
        .expect("append project");
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::SessionStarted(SessionStartedPayload {
                session_id,
                project_id,
                tx_id: None,
            }))
            .expect("session"),
        )
        .expect("append session");
    (project_id, session_id)
}

fn memory_ids(store: &SqliteEventStore) -> Vec<String> {
    let conn = store.connection().lock().expect("lock");
    let mut stmt = conn
        .prepare("SELECT memory_id FROM memory_projection ORDER BY memory_id")
        .expect("prepare");
    stmt.query_map([], |row| row.get(0))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect()
}

#[test]
fn turn_projection__with_turn_id__memory_id_matches__ac4() {
    let (_dir, store) = open_store();
    let (_project_id, session_id) = seed_session(&store);
    let turn_id = TurnId::new();
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
                session_id,
                content: "DECISION: t262 ac4".into(),
                tx_id: None,
                turn_id: Some(turn_id),
            }))
            .expect("prompt"),
        )
        .expect("append prompt");
    let ids = memory_ids(&store);
    assert_eq!(ids, vec![turn_id.to_string()]);
}

#[test]
fn turn_projection__legacy_none__still_inserts_memory__ac5() {
    let (_dir, store) = open_store();
    let (_project_id, session_id) = seed_session(&store);
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
                session_id,
                content: "legacy none".into(),
                tx_id: None,
                turn_id: None,
            }))
            .expect("prompt"),
        )
        .expect("append prompt");
    let ids = memory_ids(&store);
    assert_eq!(
        ids.len(),
        1,
        "legacy None still inserts a memory row: {ids:?}"
    );
    assert_ne!(ids[0], session_id.to_string());
}
