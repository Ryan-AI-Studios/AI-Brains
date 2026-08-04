use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::is_plain_sqlite_header;
use ai_brains_store::pragmas::apply_key_pragmas;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Minimum size for a plausible multi-page SQLite/SQLCipher backup (F31).
pub const MIN_PLAUSIBLE_BACKUP_BYTES: u64 = 512;

/// How a backup file classifies under the current vault key (T209).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub enum BackupReadClass {
    /// Meta SELECT succeeded under the current key.
    #[default]
    Readable,
    /// Key opens; core tables present (or meta table absent); no usable meta rows.
    PreT109,
    /// Plain SQLite header (pre-encrypt residual); no key probe.
    LegacyPlain,
    /// Not plain; size ≥ [`MIN_PLAUSIBLE_BACKUP_BYTES`]; key/schema verification failed.
    KeyMismatch,
    /// Not plain; open I/O failure, unreadable size, or size &lt; 512 with key fail.
    Corrupt,
}

/// Noise / detail mode for [`BackupService::list_backups`] (T209 F14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListMode {
    #[default]
    Default,
    Quiet,
    Verbose,
}

impl ListMode {
    /// Quiet wins over verbose when both true (F8/M4). No clap `conflicts_with`.
    pub fn from_flags(quiet: bool, verbose: bool) -> Self {
        if quiet {
            Self::Quiet
        } else if verbose {
            Self::Verbose
        } else {
            Self::Default
        }
    }
}

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
    /// Classification under the current vault key (T209).
    pub class: BackupReadClass,
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

    /// Resolve the backup directory, creating it if absent (write paths only).
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

    /// Resolve the backup directory without creating it (T192 F17b).
    ///
    /// Returns `Ok(None)` when the directory is absent. Never calls
    /// `create_dir_all`. Used by read-only callers (`list_backups`,
    /// `find_backup_files`, `preview_backup_path`).
    fn backup_dir_read_only(&self) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let dir = self
            .custom_output
            .clone()
            .unwrap_or_else(|| parent.join("backups"));
        if !dir.exists() {
            return Ok(None);
        }
        Ok(Some(dir))
    }

    /// Compute the backup path that the next backup would be written to
    /// without actually writing it or creating the backups directory (F17b).
    pub fn preview_backup_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let parent = self.vault_path.parent().ok_or("Invalid vault path")?;
        let backup_dir = self
            .custom_output
            .clone()
            .unwrap_or_else(|| parent.join("backups"));
        // Do not create the directory — preview is read-only.
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
    ///
    /// T187 F6: source must apply key + verify (never unkeyed open of a vault).
    pub fn run_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let src = rusqlite::Connection::open(&self.vault_path)?;
        apply_key_pragmas(&src, &self.key)?;
        src.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| format!("Key verification failed on backup source: {e}"))?;
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
        let Some(backup_dir) = self.backup_dir_read_only()? else {
            return Ok(Vec::new());
        };
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

    /// List all backups in the backup directory, classifying each file and
    /// reading metadata when possible (T209).
    ///
    /// When the backups directory is absent, returns an empty list without
    /// creating the directory (T192 F17b).
    ///
    /// Noise (F5–F8): Corrupt → `warn!` (unless Quiet → `debug!`);
    /// LegacyPlain/KeyMismatch → `debug!` (Verbose → `warn!`); PreT109 → `debug!`.
    pub fn list_backups(
        &self,
        mode: ListMode,
    ) -> Result<Vec<BackupInfo>, Box<dyn std::error::Error>> {
        let Some(backup_dir) = self.backup_dir_read_only()? else {
            return Ok(Vec::new());
        };
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

            let (class, metadata) = classify_backup_read(&path, &self.key);
            emit_list_noise(&path, class, mode);

            infos.push(BackupInfo {
                path,
                timestamp,
                metadata,
                class,
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

/// Classify a backup file under the current key (header-first + F31 size gate).
///
/// Public for unit tests (T209 B1–B4). Reuses
/// [`ai_brains_store::is_plain_sqlite_header`] — no duplicate magic.
pub fn classify_backup_read(
    path: &Path,
    key: &SqlCipherKey,
) -> (BackupReadClass, HashMap<String, String>) {
    // F3: plain header → LegacyPlain immediately; no key probe.
    if is_plain_sqlite_header(path) {
        return (BackupReadClass::LegacyPlain, HashMap::new());
    }

    // F31: size for Corrupt vs KeyMismatch after not-plain.
    let file_len = match fs::metadata(path) {
        Ok(m) => m.len(),
        Err(_) => {
            return (BackupReadClass::Corrupt, HashMap::new());
        }
    };

    let conn = match rusqlite::Connection::open(path) {
        Ok(c) => c,
        Err(_) => {
            return (BackupReadClass::Corrupt, HashMap::new());
        }
    };

    let key_ok = match apply_key_pragmas(&conn, key) {
        Ok(()) => conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .is_ok(),
        Err(_) => false,
    };

    if !key_ok {
        if file_len < MIN_PLAUSIBLE_BACKUP_BYTES {
            return (BackupReadClass::Corrupt, HashMap::new());
        }
        return (BackupReadClass::KeyMismatch, HashMap::new());
    }

    // Key opens — read meta if present. Missing meta table / row errors → PreT109
    // (T120 / F4: openable keyed backup without usable `_aibrains_backup_meta`).
    match conn.prepare("SELECT key, value FROM _aibrains_backup_meta") {
        Ok(mut stmt) => {
            let rows = stmt.query_map([], |row| {
                let k: String = row.get(0)?;
                let v: String = row.get(1)?;
                Ok((k, v))
            });
            match rows {
                Ok(iter) => {
                    let mut map = HashMap::new();
                    let mut row_err = false;
                    for row in iter {
                        match row {
                            Ok((k, v)) => {
                                map.insert(k, v);
                            }
                            Err(_) => {
                                row_err = true;
                                break;
                            }
                        }
                    }
                    if row_err {
                        return (BackupReadClass::PreT109, HashMap::new());
                    }
                    (BackupReadClass::Readable, map)
                }
                Err(_) => (BackupReadClass::PreT109, HashMap::new()),
            }
        }
        // no such table or other prepare failure with key already verified
        Err(_) => (BackupReadClass::PreT109, HashMap::new()),
    }
}

fn emit_list_noise(path: &Path, class: BackupReadClass, mode: ListMode) {
    match class {
        BackupReadClass::Readable => {}
        BackupReadClass::PreT109 => {
            tracing::debug!(
                path = %path.display(),
                "Backup predates metadata table; core tables present"
            );
        }
        BackupReadClass::LegacyPlain => match mode {
            ListMode::Verbose => {
                tracing::warn!(
                    path = %path.display(),
                    "Backup is legacy plaintext (not SQLCipher-encrypted with current key)"
                );
            }
            ListMode::Default | ListMode::Quiet => {
                tracing::debug!(
                    path = %path.display(),
                    "Backup is legacy plaintext (list detail suppressed; use --verbose)"
                );
            }
        },
        BackupReadClass::KeyMismatch => match mode {
            ListMode::Verbose => {
                tracing::warn!(
                    path = %path.display(),
                    "Backup not readable with current key (key mismatch or unreadable cipher)"
                );
            }
            ListMode::Default | ListMode::Quiet => {
                tracing::debug!(
                    path = %path.display(),
                    "Backup not readable with current key (list detail suppressed; use --verbose)"
                );
            }
        },
        BackupReadClass::Corrupt => match mode {
            ListMode::Quiet => {
                tracing::debug!(
                    path = %path.display(),
                    "Backup file appears corrupt or unreadable (quiet)"
                );
            }
            ListMode::Default | ListMode::Verbose => {
                tracing::warn!(
                    path = %path.display(),
                    "Backup file appears corrupt or unreadable"
                );
            }
        },
    }
}

/// True when product core tables (`events` and `memory_projection`) are present.
///
/// Used by backup metadata probes and doctor `schema_readable` (T192 F19).
pub fn has_core_tables(conn: &rusqlite::Connection) -> bool {
    let has_events = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name = 'events' LIMIT 1",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    let has_mem = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name = 'memory_projection' LIMIT 1",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    has_events && has_mem
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
    if normalized != s
        && let Ok(ts) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H-%M-%S%.f%:z")
    {
        return Some(ts);
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

/// Parse prune/doctor age thresholds: `Nd` / `Nh` / `Nw` only (no humantime).
///
/// Shared by backup prune and doctor `backup_recent` (T192 F17 / F23).
/// Uses checked multiplication — oversized values return Err (never panic).
pub fn parse_duration(s: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration".into());
    }
    if s.len() < 2 {
        return Err(format!("Invalid duration: {}", s).into());
    }
    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse()?;
    let secs = match unit {
        "d" => num
            .checked_mul(86400)
            .ok_or_else(|| format!("duration overflow for '{s}' (days)"))?,
        "h" => num
            .checked_mul(3600)
            .ok_or_else(|| format!("duration overflow for '{s}' (hours)"))?,
        "w" => num
            .checked_mul(86400)
            .and_then(|d| d.checked_mul(7))
            .ok_or_else(|| format!("duration overflow for '{s}' (weeks)"))?,
        _ => return Err(format!("Unknown duration unit: {}. Use d, h, or w", unit).into()),
    };
    Ok(Duration::from_secs(secs))
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_duration__overflow_days__errors_not_panic() {
        let err = parse_duration("18446744073709551615d").expect_err("must overflow");
        assert!(
            err.to_string().contains("overflow"),
            "expected overflow error, got {err}"
        );
    }

    #[test]
    fn parse_duration__valid_units() {
        assert_eq!(parse_duration("7d").expect("7d").as_secs(), 7 * 86400);
        assert_eq!(parse_duration("2h").expect("2h").as_secs(), 2 * 3600);
        assert_eq!(parse_duration("1w").expect("1w").as_secs(), 7 * 86400);
    }

    #[test]
    #[allow(non_snake_case)]
    fn backup_create__encrypted_vault__produces_valid_backup()
    -> Result<(), Box<dyn std::error::Error>> {
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

        // T187: wrong key must fail on backup file
        let wrong = SqlCipherKey::from_raw(
            "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'".to_string(),
        );
        let wrong_conn = rusqlite::Connection::open(&backup_path)?;
        apply_key_pragmas(&wrong_conn, &wrong)?;
        let wrong_open =
            wrong_conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get::<_, i32>(0));
        assert!(
            wrong_open.is_err(),
            "T187: wrong key must not read backup contents: {wrong_open:?}"
        );

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
        // Compare after canonicalize so macOS /var → /private/var is honest (T179 soft macOS).
        let recorded = std::fs::canonicalize(std::path::Path::new(&source_path))
            .unwrap_or_else(|_| std::path::PathBuf::from(&source_path));
        let expected = std::fs::canonicalize(&vault_path).unwrap_or(vault_path.clone());
        assert_eq!(
            recorded, expected,
            "source_vault_path must match the original vault path (after canonicalize)"
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
    fn list_backups__missing_dir__no_create() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        fs::write(&vault_path, b"placeholder")?;
        let backups = dir.path().join("backups");
        assert!(!backups.exists());

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let service = BackupService::new(vault_path, key);
        let list = service.list_backups(ListMode::Quiet)?;
        assert!(list.is_empty());
        assert!(
            !backups.exists(),
            "list_backups must not create backups/ when absent (T192 F17b)"
        );
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn preview_backup_path__missing_dir__no_create() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        fs::write(&vault_path, b"placeholder")?;
        let backups = dir.path().join("backups");
        assert!(!backups.exists());

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let service = BackupService::new(vault_path, key);
        let preview = service.preview_backup_path()?;
        assert!(
            preview.starts_with(&backups),
            "preview path should be under backups/: {}",
            preview.display()
        );
        assert!(
            !backups.exists(),
            "preview_backup_path must not create backups/ when absent (T192 F17b)"
        );
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn run_backup__creates_dir() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        let backups = dir.path().join("backups");
        assert!(!backups.exists());

        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = rusqlite::Connection::open(&vault_path)?;
        apply_key_pragmas(&conn, &key)?;
        conn.execute_batch(
            "CREATE TABLE test (id INTEGER PRIMARY KEY); INSERT INTO test VALUES (1);",
        )?;
        drop(conn);

        let service = BackupService::new(vault_path, key);
        let backup_path = service.run_backup()?;
        assert!(backup_path.exists());
        assert!(
            backups.exists() && backups.is_dir(),
            "run_backup must still create backups/ for writes"
        );
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn list_backups__quiet__classifies_short_garbage_as_corrupt()
    -> Result<(), Box<dyn std::error::Error>> {
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
        let backups = service.list_backups(ListMode::Quiet)?;
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].class, BackupReadClass::Corrupt);
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn list_backups__default__classifies_short_garbage_as_corrupt()
    -> Result<(), Box<dyn std::error::Error>> {
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
        let backups = service.list_backups(ListMode::Default)?;
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].class, BackupReadClass::Corrupt);
        Ok(())
    }

    // --- T209 classify units (B1–B4) ---

    fn zero_key() -> SqlCipherKey {
        SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        )
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_backup_read__plain_header__legacy_plain() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir()?;
        let path = dir.path().join("vault-2026-01-01T00-00-00.db.bak");
        // F33: valid plain SQLite magic + padding (header-based; no key success required).
        let mut bytes = b"SQLite format 3\0".to_vec();
        bytes.extend_from_slice(&[0u8; 100]);
        fs::write(&path, &bytes)?;

        let (class, meta) = classify_backup_read(&path, &zero_key());
        assert_eq!(class, BackupReadClass::LegacyPlain);
        assert!(meta.is_empty());
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_backup_read__short_garbage__corrupt() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("vault-2026-01-01T00-00-00.db.bak");
        fs::write(&path, b"not a valid sqlite database")?;
        assert!(fs::metadata(&path)?.len() < MIN_PLAUSIBLE_BACKUP_BYTES);

        let (class, meta) = classify_backup_read(&path, &zero_key());
        assert_eq!(class, BackupReadClass::Corrupt);
        assert!(meta.is_empty());
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_backup_read__large_non_plain_garbage__key_mismatch()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("vault-2026-01-01T00-00-00.db.bak");
        // ≥512 non-plain random bytes → key/schema fails → KeyMismatch (F31).
        let bytes = vec![0xABu8; 600];
        fs::write(&path, &bytes)?;
        assert!(fs::metadata(&path)?.len() >= MIN_PLAUSIBLE_BACKUP_BYTES);
        assert!(!is_plain_sqlite_header(&path));

        let (class, meta) = classify_backup_read(&path, &zero_key());
        assert_eq!(class, BackupReadClass::KeyMismatch);
        assert!(meta.is_empty());
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_backup_read__real_backup__readable_with_meta()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let vault_path = dir.path().join("vault.db");
        let key = zero_key();
        let conn = rusqlite::Connection::open(&vault_path)?;
        apply_key_pragmas(&conn, &key)?;
        conn.execute_batch(
            "CREATE TABLE test (id INTEGER PRIMARY KEY); INSERT INTO test VALUES (1);",
        )?;
        drop(conn);

        let service = BackupService::new(vault_path, key.clone());
        let backup_path = service.run_backup()?;

        let (class, meta) = classify_backup_read(&backup_path, &key);
        assert_eq!(class, BackupReadClass::Readable);
        assert!(
            meta.contains_key("backup_timestamp"),
            "readable backup must expose meta; keys={:?}",
            meta.keys().collect::<Vec<_>>()
        );
        assert!(meta.contains_key("source_vault_path"));
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn list_mode__from_flags__quiet_wins_over_verbose() {
        assert_eq!(ListMode::from_flags(false, false), ListMode::Default);
        assert_eq!(ListMode::from_flags(true, false), ListMode::Quiet);
        assert_eq!(ListMode::from_flags(false, true), ListMode::Verbose);
        assert_eq!(ListMode::from_flags(true, true), ListMode::Quiet);
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
