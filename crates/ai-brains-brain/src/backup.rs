use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::pragmas::apply_key_pragmas;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct BackupService {
    vault_path: PathBuf,
    key: SqlCipherKey,
    custom_output: Option<PathBuf>,
}

impl BackupService {
    pub fn new(vault_path: PathBuf, key: SqlCipherKey) -> Self {
        Self {
            vault_path,
            key,
            custom_output: None,
        }
    }

    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.custom_output = Some(dir);
        self
    }

    /// Run a backup using the SQLite backup API.
    /// The source connection is borrowed from the caller to avoid opening
    /// a second connection to the same WAL file (which deadlocks).
    pub fn run_backup_from_conn(
        &self,
        src_conn: &rusqlite::Connection,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.vault_path.exists() {
            return Err("Vault file does not exist".into());
        }

        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let backup_dir = self
            .custom_output
            .clone()
            .unwrap_or_else(|| parent.join("backups"));
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H-%M-%S");
        let backup_path = backup_dir.join(format!("vault-{}.db.bak", timestamp));

        // AC8: Delete any existing backup file at the same timestamp path
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }

        let mut dst = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&dst, &self.key)?;

        // Use SQLite backup API with the borrowed source connection.
        // Use -1 (all remaining pages) per step with no sleep for fast
        // completion on small-to-medium vaults.
        {
            let backup = rusqlite::backup::Backup::new(src_conn, &mut dst)?;
            backup.run_to_completion(100000, std::time::Duration::ZERO, None)?;
        }

        // Verify integrity of the backup
        let res: String = dst.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if res != "ok" {
            return Err(format!("Integrity check failed: {}", res).into());
        }

        Ok(backup_path)
    }

    /// Run a backup by opening a fresh connection to the vault.
    /// WARNING: This will deadlock if another connection to the same vault
    /// is already open in WAL mode. Prefer `run_backup_from_conn` with the
    /// existing AppContext connection.
    pub fn run_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let src = rusqlite::Connection::open(&self.vault_path)?;
        src.execute_batch("PRAGMA busy_timeout = 5000;")?;
        self.run_backup_from_conn(&src)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[allow(non_snake_case)]
    fn backup_create__encrypted_vault__produces_valid_backup(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");

        // Create a SQLCipher-encrypted vault with data
        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = rusqlite::Connection::open(&vault_path)?;
        apply_key_pragmas(&conn, &key)?;
        conn.execute_batch(
            "CREATE TABLE test (id INTEGER PRIMARY KEY); INSERT INTO test VALUES (1);",
        )?;
        drop(conn);

        let service = BackupService::new(vault_path.clone(), key.clone());
        let backup_path = service.run_backup()?;

        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backups"));

        // Verify the backup has our table (open with key)
        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&backup_conn, &key)?;
        let count: i32 =
            backup_conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
        assert_eq!(count, 1);

        // Verify integrity
        let integrity: String =
            backup_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        assert_eq!(integrity, "ok");

        Ok(())
    }
}
