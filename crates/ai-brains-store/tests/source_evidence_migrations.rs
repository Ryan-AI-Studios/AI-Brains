#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_crypto::DataKey;
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use tempfile::NamedTempFile;

fn open_migrated_vault() -> (NamedTempFile, VaultConnection) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    (temp_file, conn)
}

fn table_exists(conn: &VaultConnection, name: &str) -> bool {
    let locked = conn.lock().unwrap();
    locked
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type IN ('table', 'view') AND name = ?
            )",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .map(|v| v == 1)
        .unwrap()
}

#[test]
fn migrate_full_vault__source_evidence_tables_exist() {
    let (_temp, conn) = open_migrated_vault();

    for name in [
        "source_projection",
        "source_alias_projection",
        "source_version_projection",
        "evidence_projection",
        "evidence_fts",
        "knowledge_dependency_projection",
        "invalidation_queue_projection",
    ] {
        assert!(
            table_exists(&conn, name),
            "expected table {name} after migrations through 0021"
        );
    }

    // Legacy tables still present.
    assert!(table_exists(&conn, "events"));
    assert!(table_exists(&conn, "memory_projection"));
}

#[test]
fn source_version_unique_source_id_fingerprint__second_insert_fails() {
    let (_temp, conn) = open_migrated_vault();
    let locked = conn.lock().unwrap();

    locked
        .execute(
            "INSERT INTO source_projection (
                source_id, scope, kind, display_name, locator, status, recorded_at, updated_at
             ) VALUES (?, '', 'File', 'readme', '/tmp/a.md', 'Active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            ["src-1"],
        )
        .unwrap();

    locked
        .execute(
            "INSERT INTO source_version_projection (
                version_id, source_id, fingerprint, normalizer_version, recorded_at
             ) VALUES (?, ?, ?, 1, '2026-01-01T00:00:00Z')",
            ["ver-1", "src-1", "v1:abc"],
        )
        .unwrap();

    let err = locked
        .execute(
            "INSERT INTO source_version_projection (
                version_id, source_id, fingerprint, normalizer_version, recorded_at
             ) VALUES (?, ?, ?, 1, '2026-01-01T00:00:01Z')",
            ["ver-2", "src-1", "v1:abc"],
        )
        .expect_err("duplicate (source_id, fingerprint) must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("UNIQUE") || msg.contains("unique"),
        "expected UNIQUE constraint error, got: {msg}"
    );
}

#[test]
fn migrate_twice__idempotent() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);

    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    conn.migrate().unwrap();

    let applied: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE name IN (
                '0020_source_evidence', '0021_knowledge_dependencies'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 2);

    // Tables still usable after second migrate.
    assert!(table_exists(&conn, "source_projection"));
    assert!(table_exists(&conn, "knowledge_dependency_projection"));
}

/// R1: open a vault stopped at 0019, insert a legacy row, migrate forward,
/// preserve counts, and create 0020/0021 tables + UNIQUE.
#[test]
fn migrate_from_0019__preserves_legacy_rows_and_creates_source_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    // Stop at pre-T149 watermark.
    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0019_embedding_timestamp")).unwrap();
    }

    assert!(
        table_exists(&conn, "memory_projection"),
        "0019 vault must have legacy memory_projection"
    );
    assert!(
        !table_exists(&conn, "source_projection"),
        "0019-only vault must not yet have source_projection"
    );

    // Insert a legacy-style memory row at the 0019 watermark.
    {
        let locked = conn.lock().unwrap();
        locked
            .execute(
                "INSERT INTO memory_projection (
                    memory_id, content, privacy, status, created_at, updated_at
                 ) VALUES (?, 'legacy body', 'LocalOnly', 'pinned',
                           '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                ["mem-legacy-0019"],
            )
            .unwrap();
        let count: i64 = locked
            .query_row("SELECT COUNT(*) FROM memory_projection", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    // Forward migrate remaining (0020+).
    conn.migrate().unwrap();

    assert!(table_exists(&conn, "source_projection"));
    assert!(table_exists(&conn, "source_version_projection"));
    assert!(table_exists(&conn, "evidence_projection"));
    assert!(table_exists(&conn, "knowledge_dependency_projection"));
    assert!(table_exists(&conn, "invalidation_queue_projection"));

    let locked = conn.lock().unwrap();
    let memory_count: i64 = locked
        .query_row("SELECT COUNT(*) FROM memory_projection", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        memory_count, 1,
        "legacy memory row must survive forward migration"
    );
    let body: String = locked
        .query_row(
            "SELECT content FROM memory_projection WHERE memory_id = ?",
            ["mem-legacy-0019"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(body, "legacy body");

    // UNIQUE (source_id, fingerprint) enforced after 0020.
    locked
        .execute(
            "INSERT INTO source_projection (
                source_id, scope, kind, display_name, locator, status, recorded_at, updated_at
             ) VALUES (?, '', 'File', 'r1', '/r1', 'Active', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            ["src-r1"],
        )
        .unwrap();
    locked
        .execute(
            "INSERT INTO source_version_projection (
                version_id, source_id, fingerprint, normalizer_version, recorded_at
             ) VALUES (?, ?, ?, 1, '2026-01-01T00:00:00Z')",
            ["ver-r1-a", "src-r1", "v1:r1"],
        )
        .unwrap();
    let err = locked
        .execute(
            "INSERT INTO source_version_projection (
                version_id, source_id, fingerprint, normalizer_version, recorded_at
             ) VALUES (?, ?, ?, 1, '2026-01-01T00:00:01Z')",
            ["ver-r1-b", "src-r1", "v1:r1"],
        )
        .expect_err("UNIQUE (source_id, fingerprint) after forward migrate");
    let msg = err.to_string();
    assert!(
        msg.contains("UNIQUE") || msg.contains("unique"),
        "expected UNIQUE constraint error, got: {msg}"
    );
}
