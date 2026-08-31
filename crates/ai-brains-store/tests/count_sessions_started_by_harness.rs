//! T337 AC5 / AC12 — per-harness SessionStarted counts via actor_json.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectRegisteredPayload, SessionStartedPayload,
};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::str::FromStr;
use tempfile::tempdir;

const AGY_IMPORT: &str = "00000000-0000-0000-0000-000000000001";
const AGY_HOOK: &str = "00000000-0000-0000-0000-000000000002";
const CURSOR: &str = "00000000-0000-0000-0000-000000000007";
const GROK: &str = "00000000-0000-0000-0000-000000000003";

fn open_store() -> (tempfile::TempDir, SqliteEventStore) {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("v.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db, &sql_key).expect("open");
    conn.migrate().expect("migrate");
    (dir, SqliteEventStore::new(conn))
}

fn register_project(store: &SqliteEventStore, project_id: ProjectId) {
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Project,
                project_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                project_id,
                name: "t337".into(),
                tx_id: None,
            }))
            .expect("project"),
        )
        .expect("append project");
}

fn start_session(store: &SqliteEventStore, project_id: ProjectId, harness_uuid: &str) -> SessionId {
    let session_id = SessionId::new();
    let harness = HarnessId::from_str(harness_uuid).expect("harness uuid");
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                Actor::Harness(harness),
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
    session_id
}

#[test]
fn capture_coverage__session_started_cursor_actor__vault_sessions() {
    let (_dir, store) = open_store();
    let project_id = ProjectId::new();
    register_project(&store, project_id);
    start_session(&store, project_id, CURSOR);
    start_session(&store, project_id, GROK);

    let cursor = store
        .connection()
        .count_sessions_started_by_harness(&[CURSOR], Some(&project_id))
        .expect("count cursor");
    assert_eq!(cursor, 1, "AC5: Cursor SessionStarted counts as 1");

    let grok = store
        .connection()
        .count_sessions_started_by_harness(&[GROK], Some(&project_id))
        .expect("count grok");
    assert_eq!(grok, 1);

    let empty = store
        .connection()
        .count_sessions_started_by_harness(&[], Some(&project_id))
        .expect("empty ids");
    assert_eq!(empty, 0);
}

#[test]
fn capture_coverage__agy_both_harness_uuids__sum() {
    let (_dir, store) = open_store();
    let project_id = ProjectId::new();
    let other = ProjectId::new();
    register_project(&store, project_id);
    register_project(&store, other);
    start_session(&store, project_id, AGY_IMPORT);
    start_session(&store, project_id, AGY_HOOK);
    start_session(&store, other, AGY_HOOK);

    let agy = store
        .connection()
        .count_sessions_started_by_harness(&[AGY_IMPORT, AGY_HOOK], Some(&project_id))
        .expect("count agy");
    assert_eq!(agy, 2, "AC12: combined agy row sums …0001 and …0002");

    let global = store
        .connection()
        .count_sessions_started_by_harness(&[AGY_IMPORT, AGY_HOOK], None)
        .expect("count agy global");
    assert_eq!(global, 3);
}
