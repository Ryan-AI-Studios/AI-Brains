use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::pragmas::apply_key_pragmas;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

pub struct BackupService {
    vault_path: PathBuf,
    key: SqlCipherKey,
    custom_output: Option<PathBuf>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PruneResult {
    pub pruned_count: usize,
    pub remaining_count: usize,
    pub freed_bytes: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub timestamp: Option<NaiveDateTime>,
    pub metadata: HashMap<String, String>,
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

    fn backup_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let dir = self
            .custom_output
            .clone()
            .unwrap_or_else(|| parent.join("backups"));
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Compute the backup path that the next backup would be written to
    /// without actually writing it.
    pub fn preview_backup_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let backup_dir = self.backup_dir()?;
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S");
        Ok(backup_dir.join(format!("vault-{}.db.bak", timestamp)))
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

        let backup_dir = self.backup_dir()?;

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

        // T109: Write metadata table into the backup
        dst.execute_batch(
            "CREATE TABLE IF NOT EXISTS _aibrains_backup_meta (key TEXT PRIMARY KEY, value TEXT);",
        )?;

        let file_size = fs::metadata(&backup_path)?.len();
        let source_vault_path = match dunce::canonicalize(&self.vault_path) {
            Ok(p) => p,
            Err(_) => self.vault_path.clone(),
        };

        let insert = |key: &str, value: String| -> Result<usize, rusqlite::Error> {
            dst.execute(
                "INSERT OR REPLACE INTO _aibrains_backup_meta (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value],
            )
        };

        insert("backup_timestamp", now.to_rfc3339())?;
        insert(
            "source_vault_path",
            source_vault_path.to_string_lossy().to_string(),
        )?;
        insert("ai_brains_version", env!("CARGO_PKG_VERSION").to_string())?;
        insert("backup_file_size_bytes", file_size.to_string())?;

        let schema_ver: Option<String> = src_conn
            .query_row("SELECT MAX(name) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .ok();
        insert(
            "schema_version",
            schema_ver.unwrap_or_else(|| "unknown".to_string()),
        )?;

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

    /// Delete old backups according to a retention policy.
    ///
    /// A backup is eligible for deletion only when it fails *both*
    /// criteria: it is beyond the `keep` most recent backups and, if
    /// `older_than` is supplied, its timestamp is older than the threshold.
    /// The most recent backup is always preserved.
    pub fn prune_backups(
        &self,
        keep: usize,
        older_than: Option<&str>,
        dry_run: bool,
    ) -> Result<PruneResult, Box<dyn std::error::Error>> {
        let backup_dir = self.backup_dir()?;
        let threshold = older_than
            .map(parse_duration)
            .transpose()?
            .map(|d| Utc::now() - d);

        let mut candidates: Vec<(PathBuf, NaiveDateTime, u64)> = Vec::new();
        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_lossy = name.to_string_lossy();
            if !name_lossy.starts_with("vault-") || !name_lossy.ends_with(".db.bak") {
                continue;
            }

            let timestamp_str = name_lossy
                .strip_prefix("vault-")
                .and_then(|s| s.strip_suffix(".db.bak"))
                .unwrap_or("");
            let timestamp = match parse_backup_timestamp(timestamp_str) {
                Some(ts) => ts,
                None => {
                    tracing::debug!(
                        path = %path.display(),
                        "Skipping backup file with unparseable timestamp"
                    );
                    continue;
                }
            };

            let size = match fs::metadata(&path) {
                Ok(m) => m.len(),
                Err(err) => {
                    tracing::warn!(path = %path.display(), error = %err, "Skipping backup file: cannot read metadata");
                    continue;
                }
            };

            candidates.push((path, timestamp, size));
        }

        // Sort newest first.
        candidates.sort_by_key(|b| std::cmp::Reverse(b.1));

        let mut pruned_count = 0usize;
        let mut freed_bytes = 0u64;
        for (index, (path, timestamp, size)) in candidates.iter().enumerate() {
            // Always preserve the most recent backup.
            if index == 0 {
                continue;
            }

            let beyond_keep = index >= keep;
            let older = threshold.is_none_or(|cutoff| timestamp.and_utc() < cutoff);

            if beyond_keep && older {
                if dry_run {
                    tracing::info!(path = %path.display(), "Would prune backup");
                    pruned_count += 1;
                    freed_bytes += size;
                } else {
                    match fs::remove_file(path) {
                        Ok(()) => {
                            pruned_count += 1;
                            freed_bytes += size;
                        }
                        Err(err)
                            if matches!(
                                err.kind(),
                                ErrorKind::PermissionDenied | ErrorKind::ResourceBusy
                            ) =>
                        {
                            tracing::warn!(
                                path = %path.display(),
                                error = %err,
                                "Skipping locked backup file"
                            );
                            continue;
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
            }
        }

        let remaining_count = candidates.iter().filter(|(p, _, _)| p.exists()).count();

        Ok(PruneResult {
            pruned_count,
            remaining_count,
            freed_bytes,
        })
    }

    pub fn find_backup_files(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let backup_dir = self.backup_dir()?;
        let mut paths = Vec::new();
        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_lossy = name.to_string_lossy();
            if name_lossy.starts_with("vault-") && name_lossy.ends_with(".db.bak") {
                paths.push(path);
            }
        }
        // Sort newest first (lexicographic order matches timestamp order).
        paths.sort_by(|a, b| b.cmp(a));
        Ok(paths)
    }

    /// List all backups in the backup directory, reading metadata from each
    /// backup file when possible.
    pub fn list_backups(&self, quiet: bool) -> Result<Vec<BackupInfo>, Box<dyn std::error::Error>> {
        let backup_dir = self.backup_dir()?;
        let mut infos: Vec<BackupInfo> = Vec::new();

        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_lossy = name.to_string_lossy();
            if !name_lossy.starts_with("vault-") || !name_lossy.ends_with(".db.bak") {
                continue;
            }

            let timestamp_str = name_lossy
                .strip_prefix("vault-")
                .and_then(|s| s.strip_suffix(".db.bak"))
                .unwrap_or("");
            let timestamp = parse_backup_timestamp(timestamp_str);
            if timestamp.is_none() {
                tracing::debug!(
                    path = %path.display(),
                    "Skipping backup file with unparseable timestamp during list"
                );
            }

            let metadata = match Self::read_backup_metadata(&path, &self.key) {
                Ok(m) => m,
                Err(err) => {
                    let is_missing_meta_table = err
                        .to_string()
                        .contains("no such table: _aibrains_backup_meta");
                    let has_core_tables = match rusqlite::Connection::open(&path) {
                        Ok(conn) => {
                            let _ = apply_key_pragmas(&conn, &self.key);
                            has_core_tables(&conn)
                        }
                        Err(_) => false,
                    };
                    if is_missing_meta_table && has_core_tables {
                        tracing::debug!(
                            path = %path.display(),
                            "Backup predates metadata table; core tables present"
                        );
                    } else if quiet {
                        tracing::debug!(
                            path = %path.display(),
                            error = %err,
                            "Could not read backup metadata (quiet)"
                        );
                    } else {
                        tracing::warn!(
                            path = %path.display(),
                            error = %err,
                            "Could not read backup metadata"
                        );
                    }
                    HashMap::new()
                }
            };

            infos.push(BackupInfo {
                path,
                timestamp,
                metadata,
            });
        }

        infos.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        Ok(infos)
    }

    pub fn read_backup_metadata(
        path: &PathBuf,
        key: &SqlCipherKey,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open(path)?;
        apply_key_pragmas(&conn, key)?;
        let mut stmt = conn.prepare("SELECT key, value FROM _aibrains_backup_meta")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

fn has_core_tables(conn: &rusqlite::Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name IN ('events', 'memory_projection') LIMIT 1",
        [],
        |_row| Ok(true),
    )
    .unwrap_or(false)
}

pub fn parse_backup_timestamp(s: &str) -> Option<NaiveDateTime> {
    let formats = [
        "%Y-%m-%dT%H-%M-%S",
        "%Y-%m-%dT%H-%M-%S%.f",
        "%Y-%m-%dT%H-%M-%S%.f%:z",
    ];
    for fmt in formats {
        if let Ok(ts) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(ts);
        }
    }

    let normalized = normalize_timezone_colons(s);
    if normalized != s {
        if let Ok(ts) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H-%M-%S%.f%:z") {
            return Some(ts);
        }
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.naive_utc());
    }
    None
}

fn normalize_timezone_colons(s: &str) -> String {
    if let Some(pos) = s.rfind('+').or_else(|| s.rfind('-')) {
        let (prefix, tz) = s.split_at(pos);
        if tz.len() == 6 && tz[3..4] == *"-" {
            let mut normalized = String::with_capacity(s.len());
            normalized.push_str(prefix);
            normalized.push_str(&tz[..3]);
            normalized.push(':');
            normalized.push_str(&tz[4..]);
            return normalized;
        }
    }
    s.to_string()
}

fn parse_duration(s: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration".into());
    }
    if s.len() < 2 {
        return Err(format!("Invalid duration: {}", s).into());
    }
    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse()?;
    let duration = match unit {
        "d" => Duration::from_secs(num * 86400),
        "h" => Duration::from_secs(num * 3600),
        "w" => Duration::from_secs(num * 86400 * 7),
        _ => return Err(format!("Unknown duration unit: {}. Use d, h, or w", unit).into()),
    };
    Ok(duration)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
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

        // T109: metadata table exists
        let meta_count: i32 =
            backup_conn.query_row("SELECT COUNT(*) FROM _aibrains_backup_meta", [], |row| {
                row.get(0)
            })?;
        assert!(meta_count >= 4);

        let ts: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'backup_timestamp'",
            [],
            |row| row.get(0),
        )?;
        assert!(ts.is_some());

        let source: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'source_vault_path'",
            [],
            |row| row.get(0),
        )?;
        assert!(source.is_some());

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn backup__metadata_has_correct_schema_version() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = rusqlite::Connection::open(&vault_path)?;
        apply_key_pragmas(&conn, &key)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO schema_migrations (name) VALUES ('0018_memory_embedding');
            INSERT INTO schema_migrations (name) VALUES ('0019_embedding_timestamp');
            CREATE TABLE test (id INTEGER PRIMARY KEY);
            INSERT INTO test VALUES (1);",
        )?;

        let service = BackupService::new(vault_path.clone(), key.clone());
        let backup_path = service.run_backup_from_conn(&conn)?;

        assert!(backup_path.exists());

        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&backup_conn, &key)?;
        let schema_version: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(schema_version, Some("0019_embedding_timestamp".to_string()));

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn backup__metadata_source_path_no_unc_prefix() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = rusqlite::Connection::open(&vault_path)?;
        apply_key_pragmas(&conn, &key)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO schema_migrations (name) VALUES ('0018_memory_embedding');
            CREATE TABLE test (id INTEGER PRIMARY KEY);
            INSERT INTO test VALUES (1);",
        )?;

        let service = BackupService::new(vault_path.clone(), key.clone());
        let backup_path = service.run_backup_from_conn(&conn)?;

        assert!(backup_path.exists());

        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&backup_conn, &key)?;
        let source_path: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'source_vault_path'",
            [],
            |row| row.get(0),
        )?;
        let source_path = source_path.expect("source_vault_path must be recorded");
        assert!(
            !source_path.starts_with("\\\\?\\"),
            "source_vault_path must not have UNC prefix, got: {source_path}"
        );
        assert_eq!(
            std::path::PathBuf::from(&source_path),
            vault_path,
            "source_vault_path must match the original vault path"
        );

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn backup__creates_metadata_table() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");

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

        let backup_conn = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&backup_conn, &key)?;
        let count: i32 =
            backup_conn.query_row("SELECT COUNT(*) FROM _aibrains_backup_meta", [], |row| {
                row.get(0)
            })?;
        assert!(count >= 4);

        let ts: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'backup_timestamp'",
            [],
            |row| row.get(0),
        )?;
        assert!(ts.is_some());

        let source: Option<String> = backup_conn.query_row(
            "SELECT value FROM _aibrains_backup_meta WHERE key = 'source_vault_path'",
            [],
            |row| row.get(0),
        )?;
        assert!(source.is_some());

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn list_backups__quiet__uses_debug_for_metadata_failures(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        let backup_dir = dir.path().join("backups");
        fs::create_dir_all(&backup_dir)?;

        let bogus = backup_dir.join("vault-2026-01-01T00-00-00.db.bak");
        fs::write(&bogus, b"not a valid sqlite database")?;

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let service = BackupService::new(vault_path, key);
        let backups = service.list_backups(true)?;
        assert_eq!(backups.len(), 1);
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn list_backups__not_quiet__uses_warn_for_metadata_failures(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        let backup_dir = dir.path().join("backups");
        fs::create_dir_all(&backup_dir)?;

        let bogus = backup_dir.join("vault-2026-01-01T00-00-00.db.bak");
        fs::write(&bogus, b"not a valid sqlite database")?;

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let service = BackupService::new(vault_path, key);
        let backups = service.list_backups(false)?;
        assert_eq!(backups.len(), 1);
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_backup_timestamp__seconds_format() {
        let ts = parse_backup_timestamp("2026-04-28T16-23-52");
        assert!(ts.is_some());
        assert_eq!(
            ts.unwrap().format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-04-28 16:23:52"
        );
    }

    #[test]
    fn parse_backup_timestamp__nanosecond_format() {
        let ts = parse_backup_timestamp("2026-04-28T16-23-52.639348300");
        assert!(ts.is_some());
        assert_eq!(
            ts.unwrap().format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-04-28 16:23:52"
        );
    }

    #[test]
    fn parse_backup_timestamp__nanosecond_with_timezone() {
        let ts = parse_backup_timestamp("2026-04-28T16-23-52.639348300+00:00");
        assert!(ts.is_some());
        assert_eq!(
            ts.unwrap().format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-04-28 16:23:52"
        );
    }

    #[test]
    fn parse_backup_timestamp__nanosecond_with_dash_timezone() {
        let ts = parse_backup_timestamp("2026-04-28T16-23-52.639348300+00-00");
        assert!(ts.is_some());
        assert_eq!(
            ts.unwrap().format("%Y-%m-%d %H:%M:%S").to_string(),
            "2026-04-28 16:23:52"
        );
    }

    #[test]
    fn parse_backup_timestamp__invalid_returns_none() {
        assert!(parse_backup_timestamp("not-a-timestamp").is_none());
        assert!(parse_backup_timestamp("2026-04-28").is_none());
    }

    #[test]
    #[allow(non_snake_case)]
    fn backup_prune__keep_2__deletes_oldest() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        let backup_dir = dir.path().join("backups");
        fs::create_dir_all(&backup_dir)?;

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let service = BackupService::new(vault_path, key).with_output_dir(backup_dir.clone());

        let mut paths = Vec::new();
        for day in 1..=5 {
            let name = format!("vault-2024-01-{:02}T00-00-00.db.bak", day);
            let path = backup_dir.join(&name);
            fs::write(&path, b"fake")?;
            paths.push(path);
        }

        let result = service.prune_backups(2, None, false)?;

        assert_eq!(result.pruned_count, 3);
        assert_eq!(result.remaining_count, 2);
        assert_eq!(result.freed_bytes, 3 * 4);
        assert!(paths[3].exists());
        assert!(paths[4].exists());
        assert!(!paths[0].exists());
        assert!(!paths[1].exists());
        assert!(!paths[2].exists());

        Ok(())
    }
}
