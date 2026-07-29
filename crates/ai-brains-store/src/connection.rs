use crate::errors::{Result, StoreError};
use crate::migrations::apply_migrations;
use crate::pragmas::{apply_key_pragmas, apply_pragmas};
use ai_brains_crypto::SqlCipherKey;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VaultConnection {
    inner: Arc<Mutex<Connection>>,
}

impl VaultConnection {
    pub fn open<P: AsRef<Path>>(path: P, key: &SqlCipherKey) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Apply pragmas (including key)
        apply_pragmas(&conn, key)?;

        // Validate the key by trying to read from sqlite_master
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| StoreError::VaultLocked(format!("Key verification failed: {}", e)))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open a vault for read-intent use without mutating journal mode.
    ///
    /// Prefer `SQLITE_OPEN_READ_ONLY` so SQLite cannot create journal sidecars
    /// or rewrite `journal_mode`. SQLCipher may reject RO open for some
    /// encrypted vaults; in that case fall back to a non-creating open with
    /// [`apply_key_pragmas`] only (key + cipher_compat + busy_timeout — **no**
    /// `journal_mode` / `synchronous`). Never runs migrations and never
    /// creates a missing vault file.
    ///
    /// Use for migrate source opens and dry-run destination peeks (M5).
    pub fn open_read_intent<P: AsRef<Path>>(path: P, key: &SqlCipherKey) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(StoreError::ConnectionFailed(format!(
                "vault does not exist: {}",
                path.display()
            )));
        }

        let conn = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(conn) => {
                apply_key_pragmas(&conn, key)?;
                verify_key(&conn)?;
                conn
            }
            Err(_ro_err) => {
                // SQLCipher / platform may reject READ_ONLY for encrypted files.
                // Fall back: open read/write without CREATE so we never materialize
                // a new vault, and still skip WAL/synchronous pragmas.
                let flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
                let conn = Connection::open_with_flags(path, flags).map_err(|e| {
                    StoreError::ConnectionFailed(format!(
                        "read-intent open failed for {}: {e}",
                        path.display()
                    ))
                })?;
                apply_key_pragmas(&conn, key)?;
                verify_key(&conn)?;
                conn
            }
        };

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

fn verify_key(conn: &Connection) -> Result<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| StoreError::VaultLocked(format!("Key verification failed: {}", e)))
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_crypto::DataKey;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn zero_key() -> SqlCipherKey {
        let data = DataKey::generate();
        SqlCipherKey::from_data_key(&data)
    }

    #[test]
    fn open_read_intent__validates_key_and_reads_sqlite_master() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("vault.db");
        let key = zero_key();

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
        let key = zero_key();
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
        let key = zero_key();

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
        let key = zero_key();

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
}
