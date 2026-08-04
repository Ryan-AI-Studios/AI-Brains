//! T207 AC11 / F32 — `get_project_by_id` returns name/alias for a known id.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    Actor, AggregateType, Payload,
    constructors::EventBuilder,
    payload::{ProjectAliasAddedPayload, ProjectRegisteredPayload},
};
use ai_brains_store::QueryStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;

#[test]
fn get_project_by_id__known_id_with_alias__returns_name_and_alias() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);

    let project_id = ProjectId::new();
    let actor = Actor::System;

    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "T207 Test Project".to_string(),
        tx_id: None,
    }))
    .unwrap();
    store.append_event(&reg).expect("register project");

    let alias_ev = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
        project_id,
        alias: "t207-alias".to_string(),
    }))
    .unwrap();
    store.append_event(&alias_ev).expect("add alias");

    let found = store
        .connection()
        .get_project_by_id(&project_id)
        .expect("get_project_by_id");
    assert_eq!(
        found,
        Some(("T207 Test Project".to_string(), "t207-alias".to_string())),
        "known id must return (name, alias)"
    );
}

#[test]
fn get_project_by_id__known_id_no_alias__returns_name_empty_alias() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);

    let project_id = ProjectId::new();
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "NoAlias".to_string(),
        tx_id: None,
    }))
    .unwrap();
    store.append_event(&reg).expect("register");

    let found = store
        .connection()
        .get_project_by_id(&project_id)
        .expect("lookup");
    assert_eq!(
        found,
        Some(("NoAlias".to_string(), String::new())),
        "missing alias must be empty string"
    );
}

#[test]
fn get_project_by_id__unknown_id__returns_none() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();

    let missing = ProjectId::new();
    let found = conn.get_project_by_id(&missing).expect("lookup");
    assert_eq!(found, None, "unknown id must return None");
}
