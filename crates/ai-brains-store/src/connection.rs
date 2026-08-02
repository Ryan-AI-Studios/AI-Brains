use crate::errors::{Result, StoreError};
use crate::header::{is_plain_sqlite_header, legacy_plaintext_migrate_hint};
use crate::migrations::apply_migrations;
use crate::pragmas::{apply_key_pragmas, apply_pragmas};
use ai_brains_crypto::SqlCipherKey;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

use std::sync::{Arc, Mutex};

/// Env escape hatch for all-zero SQLCipher keys (tests / legacy dogfood only).
pub const ALLOW_ZERO_KEY_ENV: &str = "AI_BRAINS_ALLOW_ZERO_KEY";

#[derive(Clone)]
pub struct VaultConnection {
    inner: Arc<Mutex<Connection>>,
}

impl VaultConnection {
    pub fn open<P: AsRef<Path>>(path: P, key: &SqlCipherKey) -> Result<Self> {
        enforce_key_policy(key)?;
        let path = path.as_ref();
        refuse_legacy_plaintext_if_present(path)?;

        let conn = Connection::open(path).map_err(map_open_db_err)?;

        // Apply pragmas (including key). Key/codec failures → VaultLocked.
        apply_pragmas(&conn, key).map_err(map_key_err)?;
        verify_key(&conn)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open a vault **read-only** without mutating journal mode or WAL/SHM.
    ///
    /// Always opens with `OpenFlags::SQLITE_OPEN_READ_ONLY`, then applies
    /// [`apply_key_pragmas`] (key + cipher_compat + busy_timeout — **no**
    /// `journal_mode` / `synchronous`) and verifies the key. Never falls back
    /// to read/write open (avoids touching existing WAL/SHM state). Never runs
    /// migrations and never creates a missing vault file.
    ///
    /// If the RO open fails (missing path, wrong key, or platform/SQLCipher
    /// rejection of RO), returns an error — callers must not open R/W as a
    /// substitute.
    ///
    /// Use for migrate source opens and dry-run destination peeks (M5).
    pub fn open_read_intent<P: AsRef<Path>>(path: P, key: &SqlCipherKey) -> Result<Self> {
        enforce_key_policy(key)?;
        let path = path.as_ref();
        if !path.exists() {
            return Err(StoreError::ConnectionFailed(format!(
                "vault does not exist: {}",
                path.display()
            )));
        }
        refuse_legacy_plaintext_if_present(path)?;

        let conn =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
                StoreError::ConnectionFailed(format!(
                    "read-only open failed for {} (no R/W fallback): {e}",
                    path.display()
                ))
            })?;
        apply_key_pragmas(&conn, key).map_err(map_key_err)?;
        verify_key(&conn)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.inner
            .lock()
            .map_err(|e| StoreError::ConnectionFailed(e.to_string()))
    }

    pub fn migrate(&self) -> Result<()> {
        let mut conn = self.lock()?;
        apply_migrations(&mut conn)
    }

    pub fn wal_checkpoint(&self) -> Result<()> {
        let conn = self.lock()?;
        let _ = conn.query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |_| Ok(()));
        Ok(())
    }

    /// Post-wipe WAL checkpoint (E16): `PRAGMA wal_checkpoint(TRUNCATE)`.
    ///
    /// On busy result, retries once. Still busy →
    /// [`crate::projections::content_envelope::WalCheckpointOutcome::PendingPassive`]
    /// (caller warns; wipe still success). Does **not** claim NIST Purge.
    pub fn wal_checkpoint_truncate(
        &self,
    ) -> Result<crate::projections::content_envelope::WalCheckpointOutcome> {
        let conn = self.lock()?;
        crate::projections::content_envelope::wal_checkpoint_truncate(&conn)
    }
}

fn enforce_key_policy(key: &SqlCipherKey) -> Result<()> {
    if key.is_blank() {
        return Err(StoreError::VaultLocked(
            "blank SQLCipher key refused".into(),
        ));
    }
    if let Err(e) = key.validate() {
        return Err(StoreError::VaultLocked(format!("invalid key format: {e}")));
    }
    if key.is_zero() && !zero_key_allowed() {
        return Err(StoreError::VaultLocked(
            "zero key refused; set a non-zero --key / AI_BRAINS_KEY, or set \
             AI_BRAINS_ALLOW_ZERO_KEY=1 for tests/legacy only"
                .into(),
        ));
    }
    Ok(())
}

fn zero_key_allowed() -> bool {
    match std::env::var(ALLOW_ZERO_KEY_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn refuse_legacy_plaintext_if_present(path: &Path) -> Result<()> {
    if path.exists()
        && path.metadata().map(|m| m.len() > 0).unwrap_or(false)
        && is_plain_sqlite_header(path)
    {
        return Err(StoreError::LegacyPlaintextVault {
            migrate_hint: legacy_plaintext_migrate_hint(),
        });
    }
    Ok(())
}

fn verify_key(conn: &Connection) -> Result<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| StoreError::VaultLocked(format!("Key verification failed: {}", e)))
}

fn map_open_db_err(e: rusqlite::Error) -> StoreError {
    let s = e.to_string();
    let lower = s.to_ascii_lowercase();
    if lower.contains("not a database")
        || lower.contains("file is encrypted")
        || lower.contains("hmac")
    {
        StoreError::VaultLocked(format!("Key verification failed: {s}"))
    } else {
        StoreError::DatabaseError(e)
    }
}

fn map_key_err(e: StoreError) -> StoreError {
    match e {
        StoreError::DatabaseError(inner) => {
            let s = inner.to_string();
            let lower = s.to_ascii_lowercase();
            if lower.contains("not a database")
                || lower.contains("file is encrypted")
                || lower.contains("hmac")
                || lower.contains("disk image")
            {
                StoreError::VaultLocked(format!("Key verification failed: {s}"))
            } else {
                StoreError::DatabaseError(inner)
            }
        }
        other => other,
    }
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::DataKey;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn random_key() -> SqlCipherKey {
        SqlCipherKey::from_data_key(&DataKey::generate())
    }

    fn zero_key() -> SqlCipherKey {
        SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".into(),
        )
    }

    #[test]
    fn open_read_intent__validates_key_and_reads_sqlite_master() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("vault.db");
        let key = random_key();

        {
            let conn = VaultConnection::open(&path, &key).expect("create open");
            conn.migrate().expect("migrate");
        }

        let ro = VaultConnection::open_read_intent(&path, &key).expect("read intent");
        let guard = ro.lock().expect("lock");
        let count: i64 = guard
            .query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))
            .expect("sqlite_master");
        assert!(count > 0, "expected schema tables, got {count}");
    }

    #[test]
    fn open_read_intent__missing_path__errors() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("does-not-exist.db");
        let key = random_key();
        assert!(
            VaultConnection::open_read_intent(&path, &key).is_err(),
            "missing vault must error"
        );
    }

    #[test]
    fn open_read_intent__does_not_force_journal_mode_wal() {
        use crate::pragmas::apply_key_pragmas;

        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("delete-mode.db");
        let key = random_key();

        // Seed without WAL pragma so default journal_mode stays non-WAL.
        {
            let conn = Connection::open(&path).expect("raw open");
            apply_key_pragmas(&conn, &key).expect("key");
            conn.execute_batch("CREATE TABLE t(x INTEGER);")
                .expect("create");
            let mode: String = conn
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .expect("mode");
            assert_ne!(
                mode.to_lowercase(),
                "wal",
                "precondition: seed must not already be WAL, got {mode}"
            );
        }

        {
            let ro = VaultConnection::open_read_intent(&path, &key).expect("read intent");
            let mode: String = ro
                .lock()
                .expect("lock")
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .expect("mode");
            assert_ne!(
                mode.to_lowercase(),
                "wal",
                "open_read_intent must not apply PRAGMA journal_mode=WAL; got {mode}"
            );
        }

        // Contrast: normal open does set WAL.
        {
            let rw = VaultConnection::open(dir.path().join("wal-mode.db"), &key).expect("open");
            let mode: String = rw
                .lock()
                .expect("lock")
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .expect("mode");
            assert_eq!(mode.to_lowercase(), "wal");
        }
    }

    #[test]
    fn open_read_intent__does_not_require_wal_sidecar_when_absent() {
        use crate::pragmas::apply_key_pragmas;

        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("vault.db");
        let key = random_key();

        // Non-WAL seed so -wal/-shm need not exist.
        {
            let conn = Connection::open(&path).expect("raw open");
            apply_key_pragmas(&conn, &key).expect("key");
            conn.execute_batch("CREATE TABLE t(x INTEGER);")
                .expect("create");
        }

        let wal = PathBuf::from(format!("{}-wal", path.display()));
        let shm = PathBuf::from(format!("{}-shm", path.display()));
        let _ = fs::remove_file(&wal);
        let _ = fs::remove_file(&shm);
        assert!(!wal.exists(), "precondition: no -wal");
        assert!(!shm.exists(), "precondition: no -shm");

        {
            let ro = VaultConnection::open_read_intent(&path, &key).expect("read intent");
            let _: i64 = ro
                .lock()
                .expect("lock")
                .query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))
                .expect("read");
        }

        assert!(
            !wal.exists(),
            "open_read_intent must not create source -wal when absent"
        );
        assert!(
            !shm.exists(),
            "open_read_intent must not create source -shm when absent"
        );
    }

    #[test]
    fn open__new_vault_header_not_plain_after_write() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("enc.db");
        let key = random_key();
        {
            let conn = VaultConnection::open(&path, &key).expect("open");
            conn.migrate().expect("migrate");
            let g = conn.lock().expect("lock");
            g.execute_batch("CREATE TABLE t187(x INTEGER); INSERT INTO t187 VALUES (1);")
                .expect("write");
        }
        assert!(
            !is_plain_sqlite_header(&path),
            "T187-H-01: new vault must not have plain SQLite header"
        );
    }

    #[test]
    fn open__plain_legacy_vault__legacy_plaintext_error() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("plain.db");
        {
            // Unkeyed plain SQLite
            let conn = Connection::open(&path).expect("plain open");
            conn.execute_batch("CREATE TABLE t(x INTEGER);")
                .expect("create");
        }
        assert!(is_plain_sqlite_header(&path));
        let err = match VaultConnection::open(&path, &random_key()) {
            Ok(_) => panic!("must refuse plain"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(
            msg.to_ascii_lowercase().contains("plaintext")
                || msg.contains("Legacy plaintext")
                || matches!(err, StoreError::LegacyPlaintextVault { .. }),
            "T187-P-01 expected LegacyPlaintextVault class, got: {msg}"
        );
        assert!(
            msg.contains("vault encrypt") || msg.contains("sqlcipher_export"),
            "migrate hint must mention vault encrypt; got: {msg}"
        );
    }

    #[test]
    fn open__zero_key_refused_without_escape_hatch() {
        let _clear = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("z.db");
        let err = match VaultConnection::open(&path, &zero_key()) {
            Ok(_) => panic!("zero refused"),
            Err(e) => e,
        };
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("zero key") || msg.contains("ai_brains_allow_zero_key"),
            "T187-Z-01: {msg}"
        );
    }

    #[test]
    fn open__zero_key_allowed_with_escape_hatch() {
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("z.db");
        let conn = VaultConnection::open(&path, &zero_key()).expect("zero allowed");
        conn.migrate().expect("migrate");
    }

    #[test]
    fn cipher_version__non_empty_when_sqlcipher_linked() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("v.db");
        let key = random_key();
        let conn = VaultConnection::open(&path, &key).expect("open");
        let g = conn.lock().expect("lock");
        let ver = crate::pragmas::cipher_version(&g).expect("cipher_version");
        assert!(
            !ver.trim().is_empty(),
            "T187-V-01: PRAGMA cipher_version must be non-empty (SQLCipher linked); got {ver:?}"
        );
        // Observed under MSVC + bundled-sqlcipher-vendored-openssl (2026-08-02): "4.10.0 community"
        assert!(
            ver.contains("4."),
            "unexpected cipher_version shape (expected 4.x community): {ver}"
        );
    }

    #[test]
    fn open__wrong_key__vault_locked() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("vault.db");
        let key = random_key();
        {
            let conn = VaultConnection::open(&path, &key).expect("create");
            conn.migrate().expect("migrate");
            let g = conn.lock().expect("lock");
            g.execute_batch("CREATE TABLE t(x INTEGER); INSERT INTO t VALUES (1);")
                .expect("write");
        }
        let wrong = random_key();
        assert_ne!(key.expose_secret(), wrong.expose_secret());
        let err = match VaultConnection::open(&path, &wrong) {
            Ok(_) => panic!("wrong key must fail"),
            Err(e) => e,
        };
        let msg = err.to_string();
        let lower = msg.to_ascii_lowercase();
        assert!(
            matches!(err, StoreError::VaultLocked(_))
                || lower.contains("key")
                || lower.contains("not a database")
                || lower.contains("locked"),
            "wrong key must be VaultLocked class: {err}"
        );
    }
}
