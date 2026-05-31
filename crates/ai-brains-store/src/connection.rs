use crate::errors::{Result, StoreError};
use crate::migrations::apply_migrations;
use crate::pragmas::apply_pragmas;
use ai_brains_crypto::SqlCipherKey;
use rusqlite::Connection;
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
        conn.execute("PRAGMA wal_checkpoint(PASSIVE)", [])?;
        Ok(())
    }
}
