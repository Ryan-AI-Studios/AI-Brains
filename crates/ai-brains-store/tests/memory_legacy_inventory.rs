//! T270 AC1 — `memory_legacy_inventory` COUNT + SQL LIMIT 5.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_crypto::DataKey;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brains_store::projections::retention::memory_legacy_inventory;
use tempfile::NamedTempFile;

fn open_store() -> SqliteEventStore {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();
    std::mem::forget(temp_file);

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(&db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    SqliteEventStore::new(conn)
}

fn insert_memory(conn: &rusqlite::Connection, memory_id: &str, status: &str) {
    conn.execute(
        "INSERT INTO memory_projection (
            memory_id, content, privacy, status, level, created_at, updated_at
         ) VALUES (?, ?, '\"LocalOnly\"', ?, 0, '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z')",
        rusqlite::params![memory_id, format!("body-{memory_id}"), status],
    )
    .unwrap();
}

#[test]
fn memory_legacy_inventory__pinned_and_other__counts_and_limit_5() {
    let store = open_store();
    let conn = store.connection().lock().unwrap();
    insert_memory(&conn, "aaa-pinned-1", "pinned");
    insert_memory(&conn, "bbb-pinned-2", "pinned");
    insert_memory(&conn, "ccc-pinned-3", "pinned");
    insert_memory(&conn, "ddd-active-1", "active");
    insert_memory(&conn, "eee-active-2", "active");
    insert_memory(&conn, "fff-forgotten-1", "forgotten");

    let inv = memory_legacy_inventory(&conn).unwrap();
    assert_eq!(inv.pinned, 3);
    assert_eq!(inv.other, 3);
    assert_eq!(inv.total(), 6);
    assert_eq!(
        inv.sample_ids,
        vec![
            "aaa-pinned-1".to_string(),
            "bbb-pinned-2".to_string(),
            "ccc-pinned-3".to_string()
        ]
    );
}

#[test]
fn memory_legacy_inventory__pinned_zero_other_nonzero__samples_from_non_pinned() {
    let store = open_store();
    let conn = store.connection().lock().unwrap();
    insert_memory(&conn, "fff-active-1", "active");
    insert_memory(&conn, "ggg-forgotten-1", "forgotten");
    insert_memory(&conn, "hhh-active-2", "active");

    let inv = memory_legacy_inventory(&conn).unwrap();
    assert_eq!(inv.pinned, 0);
    assert_eq!(inv.other, 3);
    assert_eq!(inv.sample_ids.len(), 3);
    assert_eq!(
        inv.sample_ids,
        vec![
            "fff-active-1".to_string(),
            "ggg-forgotten-1".to_string(),
            "hhh-active-2".to_string()
        ]
    );
}

#[test]
fn memory_legacy_inventory__empty_table__zeros_and_empty_samples() {
    let store = open_store();
    let conn = store.connection().lock().unwrap();
    let inv = memory_legacy_inventory(&conn).unwrap();
    assert_eq!(inv.pinned, 0);
    assert_eq!(inv.other, 0);
    assert!(inv.sample_ids.is_empty());
}

#[test]
fn memory_legacy_inventory__more_than_five_other__sql_limit_5() {
    let store = open_store();
    let conn = store.connection().lock().unwrap();
    for i in 0..7 {
        insert_memory(&conn, &format!("other-{i:02}"), "active");
    }
    let inv = memory_legacy_inventory(&conn).unwrap();
    assert_eq!(inv.pinned, 0);
    assert_eq!(inv.other, 7);
    assert_eq!(inv.sample_ids.len(), 5);
    assert_eq!(inv.sample_ids[0], "other-00");
    assert_eq!(inv.sample_ids[4], "other-04");
}
