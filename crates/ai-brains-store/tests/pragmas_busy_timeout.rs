#![allow(clippy::disallowed_methods)]

use ai_brains_crypto::DataKey;
use ai_brains_store::connection::VaultConnection;
use tempfile::tempdir;

#[test]
#[allow(non_snake_case)]
fn pragma_busy_timeout_is_set__new_connection__returns_5000() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("vault.db");

    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path.to_str().unwrap(), &sql_key).unwrap();

    let busy_timeout_ms: i32 = conn
        .lock()
        .unwrap()
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .unwrap();

    assert_eq!(busy_timeout_ms, 5000);
}
