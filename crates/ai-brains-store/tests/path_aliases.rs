//! T233 — path alias query surface (`list_path_aliases`, `find_path_alias_owner`).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    Actor, AggregateType, Payload,
    constructors::EventBuilder,
    payload::{ProjectRegisteredPayload, RepositoryPathAliasAddedPayload},
};
use ai_brains_store::QueryStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;

fn open_store() -> SqliteEventStore {
    let temp_file = NamedTempFile::new().unwrap();
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

fn add_path_alias(store: &SqliteEventStore, project_id: ProjectId, normalized_path: &str) {
    let ev = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::RepositoryPathAliasAdded(
        RepositoryPathAliasAddedPayload {
            project_id,
            normalized_path: normalized_path.to_string(),
        },
    ))
    .unwrap();
    store.append_event(&ev).expect("path alias");
}

#[test]
fn list_path_aliases__multi_alias_same_project__all_returned_sorted_asc() {
    let store = open_store();
    let project = ProjectId::new();
    register_project(&store, project, "MultiRoot");

    // Insert out of order; list must return ASC by normalized_path.
    add_path_alias(&store, project, "c:/dev/z-last");
    add_path_alias(&store, project, "c:/dev/a-first");
    add_path_alias(&store, project, "c:/dev/m-mid");

    let rows = store.connection().list_path_aliases().expect("list");
    assert_eq!(rows.len(), 3, "three aliases for one project");
    assert_eq!(rows[0].0, project);
    assert_eq!(rows[0].1, "c:/dev/a-first");
    assert_eq!(rows[1].0, project);
    assert_eq!(rows[1].1, "c:/dev/m-mid");
    assert_eq!(rows[2].0, project);
    assert_eq!(rows[2].1, "c:/dev/z-last");
}

#[test]
fn list_path_aliases__empty_vault__returns_empty_vec() {
    let store = open_store();
    let rows = store.connection().list_path_aliases().expect("list");
    assert!(rows.is_empty());
}

#[test]
fn find_path_alias_owner__known_path__returns_owner() {
    let store = open_store();
    let project = ProjectId::new();
    register_project(&store, project, "Owner");
    add_path_alias(&store, project, "c:/dev/owned");

    let found = store
        .connection()
        .find_path_alias_owner("c:/dev/owned")
        .expect("find");
    assert_eq!(found, Some(project));
}

#[test]
fn find_path_alias_owner__unknown_path__returns_none() {
    let store = open_store();
    let found = store
        .connection()
        .find_path_alias_owner("c:/dev/missing")
        .expect("find");
    assert_eq!(found, None);
}

#[test]
fn find_path_alias_owner__two_projects__lookup_is_exclusive() {
    let store = open_store();
    let a = ProjectId::new();
    let b = ProjectId::new();
    register_project(&store, a, "A");
    register_project(&store, b, "B");
    add_path_alias(&store, a, "c:/dev/a");
    add_path_alias(&store, b, "c:/dev/b");

    assert_eq!(
        store
            .connection()
            .find_path_alias_owner("c:/dev/a")
            .unwrap(),
        Some(a)
    );
    assert_eq!(
        store
            .connection()
            .find_path_alias_owner("c:/dev/b")
            .unwrap(),
        Some(b)
    );

    let listed = store.connection().list_path_aliases().unwrap();
    assert_eq!(listed.len(), 2);
    // ASC sort across projects
    assert_eq!(listed[0].1, "c:/dev/a");
    assert_eq!(listed[1].1, "c:/dev/b");
}
