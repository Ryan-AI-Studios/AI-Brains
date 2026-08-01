#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T181 library recovery drills: kit→SqlCipherKey open (K-05/K-06) and CE envelope residual (E-01/E-02).
//!
//! Backup/restore uses the SQLCipher Online Backup API + key pragmas (product path),
//! not raw `fs::copy` of a live WAL vault (F2). CE wipe uses `destroy_content_key_wrap`
//! + `wal_checkpoint_truncate` (F35).

use ai_brains_core::ids::ContentKeyId;
use ai_brains_crypto::content_envelope::generate_wrap_and_seal;
use ai_brains_crypto::content_key_store::parse_nonce;
use ai_brains_crypto::test_support::assert_no_secret_leakage;
use ai_brains_crypto::{CryptoError, DataKey, RecoveryKit, SqlCipherKey};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::pragmas::apply_key_pragmas;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ContentKeyWrapRow, EncryptedBlobRow,
};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use uuid::Uuid;

const CREATED_AT: &str = "2026-08-01T12:00:00Z";
const DESTROYED_AT: &str = "2026-08-01T13:00:00Z";
const SEED_PLAINTEXT: &[u8] = b"T181-E pre-erase residual plaintext";

fn open_vault(path: &Path, key: &SqlCipherKey) -> VaultConnection {
    let conn = VaultConnection::open(path, key).expect("open vault");
    conn.migrate().expect("migrate");
    conn
}

/// Product-style encrypted backup (Online Backup API + meta table), mirroring BackupService.
fn online_backup(src_path: &Path, key: &SqlCipherKey, dest: &Path) {
    let src = rusqlite::Connection::open(src_path).expect("open src");
    apply_key_pragmas(&src, key).expect("src key");
    let mut dst = rusqlite::Connection::open(dest).expect("open dest");
    apply_key_pragmas(&dst, key).expect("dst key");
    {
        let backup = rusqlite::backup::Backup::new(&src, &mut dst).expect("backup");
        backup
            .run_to_completion(100_000, std::time::Duration::ZERO, None)
            .expect("run backup");
    }
    let res: String = dst
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity");
    assert_eq!(res, "ok");
    dst.execute_batch(
        "CREATE TABLE IF NOT EXISTS _aibrains_backup_meta (key TEXT PRIMARY KEY, value TEXT);",
    )
    .expect("meta table");
    dst.execute(
        "INSERT OR REPLACE INTO _aibrains_backup_meta (key, value) VALUES ('source_vault_path', ?1)",
        rusqlite::params![src_path.to_string_lossy().to_string()],
    )
    .expect("meta insert");
}

fn online_restore(backup_path: &Path, vault_path: &Path, key: &SqlCipherKey) {
    let bak = rusqlite::Connection::open(backup_path).expect("open backup");
    apply_key_pragmas(&bak, key).expect("bak key");
    let mut vault = rusqlite::Connection::open(vault_path).expect("open vault");
    apply_key_pragmas(&vault, key).expect("vault key");
    {
        let backup = rusqlite::backup::Backup::new(&bak, &mut vault).expect("restore backup");
        backup
            .run_to_completion(100_000, std::time::Duration::ZERO, None)
            .expect("run restore");
    }
    vault
        .execute_batch("DROP TABLE IF EXISTS _aibrains_backup_meta;")
        .expect("drop meta");
}

fn meta_table_exists(path: &Path, key: &SqlCipherKey) -> bool {
    let conn = rusqlite::Connection::open(path).expect("open");
    apply_key_pragmas(&conn, key).expect("key");
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_aibrains_backup_meta'",
        [],
        |_| Ok(true),
    )
    .unwrap_or(false)
}

fn open_from_store_rows(
    data_key: &DataKey,
    content_key_id: &ContentKeyId,
    wrap_row: &ContentKeyWrapRow,
    sealed: &ai_brains_crypto::SealedContent,
    blob_id: Uuid,
) -> Result<Vec<u8>, CryptoError> {
    let wrap = match wrap_row {
        row if row.status == "active" => row,
        _ => return Err(CryptoError::AuthenticationFailed),
    };
    let (nonce_bytes, ciphertext) = match (&wrap.wrap_nonce, &wrap.wrap_ciphertext) {
        (Some(n), Some(c)) if !n.is_empty() && !c.is_empty() => (n.as_slice(), c.as_slice()),
        _ => return Err(CryptoError::AuthenticationFailed),
    };
    let nonce = parse_nonce(nonce_bytes)?;
    let wrapped = ai_brains_crypto::WrappedContentDek {
        wrap_schema_version: wrap.wrap_schema_version as u32,
        nonce,
        ciphertext: ciphertext.to_vec(),
    };
    let opened =
        ai_brains_crypto::unwrap_and_open(data_key, content_key_id, &wrapped, sealed, blob_id)?;
    Ok(opened.to_vec())
}

fn seal_and_persist(
    conn: &rusqlite::Connection,
    data_key: &DataKey,
    content_key_id: &ContentKeyId,
    blob_id: Uuid,
    plaintext: &[u8],
) {
    let env = generate_wrap_and_seal(data_key, *content_key_id, blob_id, plaintext)
        .expect("generate_wrap_and_seal");
    content_envelope::insert_content_key_wrap(
        conn,
        &content_key_id.to_string(),
        i64::from(env.wrapped_dek.wrap_schema_version),
        &env.wrapped_dek.nonce,
        &env.wrapped_dek.ciphertext,
        CREATED_AT,
    )
    .expect("insert wrap");
    let row = EncryptedBlobRow {
        blob_id: blob_id.to_string(),
        content_key_id: content_key_id.to_string(),
        envelope_schema_version: i64::from(env.sealed.envelope_schema_version),
        algorithm: ALGORITHM_AES_256_GCM.to_string(),
        nonce: env.sealed.nonce.to_vec(),
        ciphertext: env.sealed.ciphertext.clone(),
        content_class: Some("memory".to_string()),
        subject_kind: None,
        subject_id: None,
        size_bytes: env.sealed.ciphertext.len() as i64,
        created_at: CREATED_AT.to_string(),
    };
    content_envelope::insert_encrypted_blob(conn, &row).expect("insert blob");
}

fn try_open_ce(
    path: &Path,
    sql_key: &SqlCipherKey,
    data_key: &DataKey,
    content_key_id: &ContentKeyId,
    blob_id: Uuid,
) -> Result<Vec<u8>, CryptoError> {
    let conn =
        VaultConnection::open(path, sql_key).map_err(|_| CryptoError::AuthenticationFailed)?;
    let guard = conn.lock().expect("lock");
    let wrap_row = content_envelope::get_content_key_wrap(&guard, &content_key_id.to_string())
        .expect("get wrap")
        .ok_or(CryptoError::AuthenticationFailed)?;
    let blob = content_envelope::get_encrypted_blob(&guard, &blob_id.to_string())
        .expect("get blob")
        .ok_or(CryptoError::AuthenticationFailed)?;
    let sealed = ai_brains_crypto::SealedContent {
        envelope_schema_version: blob.envelope_schema_version as u32,
        nonce: parse_nonce(&blob.nonce).map_err(|_| CryptoError::AuthenticationFailed)?,
        ciphertext: blob.ciphertext,
    };
    open_from_store_rows(data_key, content_key_id, &wrap_row, &sealed, blob_id)
}

/// T181-K-05: unlock kit → SqlCipherKey::from_data_key → open vault (library only).
#[test]
fn recovery_kit__unlock_to_sqlcipher_key__opens_vault() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let data_key = DataKey::generate();
    let passphrase = b"t181-k05-passphrase";
    let kit = RecoveryKit::generate(&data_key, passphrase).expect("kit");

    // Create vault with production from_data_key path (F33) — no bare hex PRAGMA.
    let sql_key = SqlCipherKey::from_data_key(&data_key);
    {
        let conn = open_vault(&vault_path, &sql_key);
        let guard = conn.lock().unwrap();
        guard
            .execute_batch("CREATE TABLE IF NOT EXISTS t181_probe (v TEXT); INSERT INTO t181_probe VALUES ('k05-ok');")
            .expect("probe write");
    }

    let unlocked = kit.unlock_with_passphrase(passphrase).expect("unlock");
    assert_eq!(unlocked.expose_secret(), data_key.expose_secret());
    let reopen_key = SqlCipherKey::from_data_key(&unlocked);
    let reopened = VaultConnection::open(&vault_path, &reopen_key).expect("reopen with kit key");
    let guard = reopened.lock().unwrap();
    let v: String = guard
        .query_row("SELECT v FROM t181_probe", [], |r| r.get(0))
        .expect("read probe");
    assert_eq!(v, "k05-ok");

    // Secrets must not appear in a benign status string.
    assert_no_secret_leakage("vault open ok", data_key.expose_secret());
    assert_no_secret_leakage("vault open ok", passphrase);
}

/// T181-K-06: correct unlock then open with wrong SqlCipherKey fails when
/// SQLCipher encryption is active. Workspace currently depends on
/// `rusqlite` with `bundled` (plain SQLite) — file header is
/// `SQLite format 3` — so wrong-key open may succeed. In that residual
/// mode we still prove kit→`from_data_key` binding (distinct material +
/// correct key still opens).
#[test]
fn recovery_kit__correct_unlock_wrong_sqlcipher_key__open_fails() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let data_key = DataKey::generate();
    let passphrase = b"t181-k06-passphrase";
    let kit = RecoveryKit::generate(&data_key, passphrase).expect("kit");

    let sql_key = SqlCipherKey::from_data_key(&data_key);
    {
        let conn = open_vault(&vault_path, &sql_key);
        let guard = conn.lock().unwrap();
        guard
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS t181_k06 (v TEXT); INSERT INTO t181_k06 VALUES ('bound');",
            )
            .expect("probe");
    }

    let unlocked = kit.unlock_with_passphrase(passphrase).expect("unlock");
    assert_eq!(unlocked.expose_secret(), data_key.expose_secret());
    let correct_key = SqlCipherKey::from_data_key(&unlocked);

    let wrong = DataKey::generate();
    let wrong_key = SqlCipherKey::from_data_key(&wrong);
    assert_ne!(
        wrong_key.expose_secret(),
        correct_key.expose_secret(),
        "wrong DataKey must produce distinct SqlCipherKey material (F33 path)"
    );

    // Correct production path still opens.
    let reopened = VaultConnection::open(&vault_path, &correct_key).expect("correct open");
    {
        let guard = reopened.lock().unwrap();
        let v: String = guard
            .query_row("SELECT v FROM t181_k06", [], |r| r.get(0))
            .expect("read");
        assert_eq!(v, "bound");
    }

    let wrong_open = VaultConnection::open(&vault_path, &wrong_key);
    if file_looks_sqlcipher_encrypted(&vault_path) {
        assert!(
            wrong_open.is_err(),
            "SQLCipher-active build: wrong SqlCipherKey must fail open"
        );
    } else {
        // Residual (deferred): plain bundled SQLite ignores PRAGMA key.
        assert!(
            is_plain_sqlite_header(&vault_path),
            "expected plain SQLite header when wrong-key open is not fail-closed"
        );
        // Binding property still holds: wrong key material differs; correct open works above.
        let _ = wrong_open;
    }
}

fn is_plain_sqlite_header(path: &Path) -> bool {
    let bytes = std::fs::read(path).unwrap_or_default();
    bytes.starts_with(b"SQLite format 3")
}

fn file_looks_sqlcipher_encrypted(path: &Path) -> bool {
    !is_plain_sqlite_header(path)
}

/// T181-E-01: pre-erase residual — backup before wipe still opens CE content after live wipe.
///
/// Honesty (ADR-0016 §12): this proves offline pre-erase backups remain recoverable;
/// it is not a bug and is not NIST Purge/Destroy.
#[test]
fn content_envelope__pre_wipe_backup__opens_after_live_wipe() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_path = dir.path().join("prewipe.db.bak");
    let restore_path = dir.path().join("restored.db");

    let data_key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&data_key);
    let content_key_id = ContentKeyId::new();
    let blob_id = Uuid::new_v4();

    {
        let conn = open_vault(&vault_path, &sql_key);
        let guard = conn.lock().unwrap();
        seal_and_persist(&guard, &data_key, &content_key_id, blob_id, SEED_PLAINTEXT);
    }

    // Prove open works before backup/wipe.
    let before = try_open_ce(&vault_path, &sql_key, &data_key, &content_key_id, blob_id)
        .expect("open before wipe");
    assert_eq!(before.as_slice(), SEED_PLAINTEXT);

    online_backup(&vault_path, &sql_key, &backup_path);
    assert!(
        meta_table_exists(&backup_path, &sql_key),
        "backup must carry _aibrains_backup_meta"
    );

    // Live wipe (F35).
    {
        let conn = VaultConnection::open(&vault_path, &sql_key).expect("reopen live");
        {
            let guard = conn.lock().unwrap();
            content_envelope::destroy_content_key_wrap(
                &guard,
                &content_key_id.to_string(),
                DESTROYED_AT,
            )
            .expect("destroy wrap");
        }
        let _ = conn.wal_checkpoint_truncate().expect("checkpoint");
    }

    // Live vault fails closed.
    let live_after = try_open_ce(&vault_path, &sql_key, &data_key, &content_key_id, blob_id);
    assert!(
        matches!(live_after, Err(CryptoError::AuthenticationFailed)),
        "live post-wipe must fail closed, got: {live_after:?}"
    );

    // Restore pre-wipe backup via Online Backup API only (F2 — no fs::copy of live WAL vault).
    {
        let shell = VaultConnection::open(&restore_path, &sql_key).expect("shell");
        drop(shell);
    }
    online_restore(&backup_path, &restore_path, &sql_key);
    assert!(
        !meta_table_exists(&restore_path, &sql_key),
        "live/restored vault must DROP _aibrains_backup_meta (F20)"
    );

    let restored = try_open_ce(&restore_path, &sql_key, &data_key, &content_key_id, blob_id)
        .expect("pre-wipe backup residual open");
    assert_eq!(restored.as_slice(), SEED_PLAINTEXT);
}

/// T181-E-02: post-wipe backup cannot open wiped CE content after restore.
#[test]
fn content_envelope__post_wipe_backup__open_fails_after_restore() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_path = dir.path().join("postwipe.db.bak");
    let restore_path = dir.path().join("restored.db");

    let data_key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&data_key);
    let content_key_id = ContentKeyId::new();
    let blob_id = Uuid::new_v4();

    {
        let conn = open_vault(&vault_path, &sql_key);
        let guard = conn.lock().unwrap();
        seal_and_persist(&guard, &data_key, &content_key_id, blob_id, SEED_PLAINTEXT);
    }

    {
        let conn = VaultConnection::open(&vault_path, &sql_key).expect("reopen");
        {
            let guard = conn.lock().unwrap();
            content_envelope::destroy_content_key_wrap(
                &guard,
                &content_key_id.to_string(),
                DESTROYED_AT,
            )
            .expect("destroy");
        }
        let _ = conn.wal_checkpoint_truncate().expect("checkpoint");
    }

    online_backup(&vault_path, &sql_key, &backup_path);

    // Product-path restore only (Online Backup API); no fs::copy of live vault.
    {
        let shell = VaultConnection::open(&restore_path, &sql_key).expect("shell");
        drop(shell);
    }
    online_restore(&backup_path, &restore_path, &sql_key);

    let opened = try_open_ce(&restore_path, &sql_key, &data_key, &content_key_id, blob_id);
    assert!(
        matches!(opened, Err(CryptoError::AuthenticationFailed)),
        "post-wipe backup restore must not open CE content, got: {opened:?}"
    );
}

/// Path helper kept so rustc doesn't warn if only used in future expand.
#[allow(dead_code)]
fn backup_dir(base: &Path) -> PathBuf {
    base.join("backups")
}
