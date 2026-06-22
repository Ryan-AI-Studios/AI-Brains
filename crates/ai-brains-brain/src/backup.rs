use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::pragmas::apply_pragmas;
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

    pub fn run_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

        // Apply SQLCipher key pragmas (key + cipher_compatibility + journal_mode
        // + synchronous + busy_timeout = 5000) to both connections.
        let src = rusqlite::Connection::open(&self.vault_path)?;
        apply_pragmas(&src, &self.key)?;
        let mut dst = rusqlite::Connection::open(&backup_path)?;
        apply_pragmas(&dst, &self.key)?;

        // Use SQLite backup API for consistent, safe backups
        {
            let backup = rusqlite::backup::Backup::new(&src, &mut dst)?;
            backup.run_to_completion(10, std::time::Duration::from_millis(250), None)?;
        }

        // Verify integrity of the backup
        let res: String = dst.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if res != "ok" {
            return Err(format!("Integrity check failed: {}", res).into());
        }

        Ok(backup_path)
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
        apply_pragmas(&conn, &key)?;
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
        apply_pragmas(&backup_conn, &key)?;
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
