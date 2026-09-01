//! T338 F5 / AC2 / AC10 — pinned NULL embedding COUNT + keyset page.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::MemoryPinnedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use tempfile::tempdir;

fn open_store() -> (tempfile::TempDir, VaultConnection, SqliteEventStore) {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("v.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db, &sql_key).expect("open");
    conn.migrate().expect("migrate");
    let store = SqliteEventStore::new(conn.clone());
    (dir, conn, store)
}

fn pin(store: &SqliteEventStore, project_id: ProjectId, content: &str) -> MemoryId {
    let memory_id = MemoryId::new();
    let env = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
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
    }))
    .expect("pin");
    store.append_event(&env).expect("append");
    memory_id
}

#[test]
fn count_pinned_without_embeddings__three_null__count_3() {
    let (_dir, conn, store) = open_store();
    let project_id = ProjectId::new();
    pin(&store, project_id, "one");
    pin(&store, project_id, "two");
    pin(&store, project_id, "three");
    let n = conn.count_pinned_without_embeddings().expect("count");
    assert_eq!(n, 3);
}

#[test]
fn page_pinned_without_embeddings__keyset_binds_cursor__second_page_excludes_first() {
    let (_dir, conn, store) = open_store();
    let project_id = ProjectId::new();
    let a = pin(&store, project_id, "a");
    let b = pin(&store, project_id, "b");
    let c = pin(&store, project_id, "c");
    {
        let locked = conn.lock().expect("lock");
        locked
            .execute(
                "UPDATE memory_projection SET updated_at = datetime('now', '-1 seconds') WHERE memory_id = ?",
                [a.to_string()],
            )
            .expect("age a");
        locked
            .execute(
                "UPDATE memory_projection SET updated_at = datetime('now', '-2 seconds') WHERE memory_id = ?",
                [b.to_string()],
            )
            .expect("age b");
        locked
            .execute(
                "UPDATE memory_projection SET updated_at = datetime('now', '-3 seconds') WHERE memory_id = ?",
                [c.to_string()],
            )
            .expect("age c");
    }
    let page1 = conn.page_pinned_without_embeddings(2, None).expect("page1");
    assert_eq!(page1.len(), 2);
    let last = page1.last().expect("last");
    let page2 = conn
        .page_pinned_without_embeddings(2, Some((&last.updated_at, &last.memory_id)))
        .expect("page2");
    assert_eq!(page2.len(), 1);
    assert_ne!(page2[0].memory_id, page1[0].memory_id);
    assert_ne!(page2[0].memory_id, page1[1].memory_id);
}

#[test]
fn get_memories_without_embeddings__old_pinned__included_when_since_none() {
    let (_dir, conn, store) = open_store();
    let project_id = ProjectId::new();
    let id = pin(&store, project_id, "old pinned");
    {
        let locked = conn.lock().expect("lock");
        locked
            .execute(
                "UPDATE memory_projection SET updated_at = datetime('now', '-30 days') WHERE memory_id = ?",
                [id.to_string()],
            )
            .expect("age");
    }
    let none = conn
        .get_memories_without_embeddings(10, None)
        .expect("none");
    let week = conn
        .get_memories_without_embeddings(10, Some(7))
        .expect("week");
    assert!(
        none.iter().any(|(mid, _)| mid == &id.to_string()),
        "since_days=None must include 30-day-old pinned NULL; got {none:?}"
    );
    assert!(
        week.iter().all(|(mid, _)| mid != &id.to_string()),
        "since_days=Some(7) must miss 30-day-old pinned NULL; got {week:?}"
    );
}
