//! T214 AC13 / F5 / F7 / F8 — QueryStore preflight rollup count helpers.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    Actor, AggregateType, Payload,
    constructors::EventBuilder,
    payload::{MemoryPinnedPayload, ProjectRegisteredPayload, SessionStartedPayload},
};
use ai_brains_store::QueryStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;

fn open_store() -> SqliteEventStore {
    let temp_file = NamedTempFile::new().unwrap();
    // Keep temp path alive by leaking into process (test isolation via unique file).
    let db_path = temp_file.path().to_str().unwrap().to_string();
    // Prevent drop of NamedTempFile from deleting while store open: keep handle.
    std::mem::forget(temp_file);

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    SqliteEventStore::new(conn)
}

fn register_project(store: &SqliteEventStore, project_id: ProjectId, name: &str) {
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: name.to_string(),
        tx_id: None,
    }))
    .unwrap();
    store.append_event(&reg).expect("register project");
}

fn pin_memory(store: &SqliteEventStore, project_id: ProjectId, content: &str) {
    let memory_id = MemoryId::new();
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.to_string(),
        session_id: None,
        project_id: Some(project_id),
        tx_id: None,
        rank: None,
        source_tag: None,
        query_text: None,
    }))
    .unwrap();
    store.append_event(&envelope).expect("pin memory");
}

fn start_session(store: &SqliteEventStore, project_id: ProjectId) -> SessionId {
    let session_id = SessionId::new();
    let envelope = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::SessionStarted(SessionStartedPayload {
        session_id,
        project_id,
        tx_id: None,
    }))
    .unwrap();
    store.append_event(&envelope).expect("session started");
    session_id
}

#[test]
fn count_preflight_rollup__empty_vault__all_zeros() {
    let store = open_store();
    let conn = store.connection();

    assert_eq!(conn.count_projects_with_pinned().unwrap(), 0);
    assert_eq!(conn.count_pinned_memories(None).unwrap(), 0);
    assert_eq!(conn.count_active_sessions(None).unwrap(), 0);
}

#[test]
fn count_preflight_rollup__multi_project_pins__projects_and_totals() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "Project A");
    register_project(&store, b, "Project B");
    pin_memory(&store, a, "DECISION: use SQLCipher");
    pin_memory(&store, a, "CONSTRAINT: no AGPL");
    pin_memory(&store, b, "DECISION: dual count model");

    let conn = store.connection();
    let projects = conn.count_projects_with_pinned().unwrap();
    assert!(
        projects >= 2,
        "two projects with pins must count >= 2; got {projects}"
    );
    let total = conn.count_pinned_memories(None).unwrap();
    assert_eq!(total, 3, "vault-wide pinned must equal 3");

    let scoped_a = conn.count_pinned_memories(Some(&a)).unwrap();
    assert_eq!(scoped_a, 2, "project A must have 2 pins");
    let scoped_b = conn.count_pinned_memories(Some(&b)).unwrap();
    assert_eq!(scoped_b, 1, "project B must have 1 pin");
}

#[test]
fn count_preflight_rollup__scoped_pinned_filters_one_project() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    pin_memory(&store, a, "pin A only");
    pin_memory(&store, b, "pin B only");
    pin_memory(&store, b, "pin B two");

    let conn = store.connection();
    assert_eq!(conn.count_pinned_memories(Some(&a)).unwrap(), 1);
    assert_eq!(conn.count_pinned_memories(Some(&b)).unwrap(), 2);
    assert_eq!(conn.count_pinned_memories(None).unwrap(), 3);
    // Unknown project → 0
    let missing = ProjectId::new();
    assert_eq!(conn.count_pinned_memories(Some(&missing)).unwrap(), 0);
}

#[test]
fn count_preflight_rollup__active_sessions_from_session_started() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    let _s1 = start_session(&store, a);
    let _s2 = start_session(&store, a);
    let _s3 = start_session(&store, b);

    let conn = store.connection();
    assert_eq!(conn.count_active_sessions(None).unwrap(), 3);
    assert_eq!(conn.count_active_sessions(Some(&a)).unwrap(), 2);
    assert_eq!(conn.count_active_sessions(Some(&b)).unwrap(), 1);
    let missing = ProjectId::new();
    assert_eq!(conn.count_active_sessions(Some(&missing)).unwrap(), 0);
}

#[test]
fn count_preflight_rollup__projects_with_pinned_ignores_unpinned_only() {
    // Projects with only SessionStarted and no pins must not inflate F7.
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    let _s = start_session(&store, a);
    pin_memory(&store, b, "only B pinned");

    let conn = store.connection();
    assert_eq!(
        conn.count_projects_with_pinned().unwrap(),
        1,
        "only projects with pinned memories count"
    );
    assert_eq!(conn.count_pinned_memories(None).unwrap(), 1);
    // Active session still counts independently
    assert_eq!(conn.count_active_sessions(None).unwrap(), 1);
}
