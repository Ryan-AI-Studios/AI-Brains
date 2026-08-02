use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_brain::BackupService;
use ai_brains_store::pragmas::apply_key_pragmas;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const RETENTION_SENTINEL: &str = ".retention-acknowledged";

fn retention_sentinel_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        dirs::home_dir().ok_or("Could not determine home directory")?
    };
    let mut path = home;
    path.push(".ai-brains");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    path.push(RETENTION_SENTINEL);
    Ok(path)
}

fn maybe_emit_retention_warning() -> Result<(), Box<dyn std::error::Error>> {
    let sentinel = retention_sentinel_path()?;
    if !sentinel.exists() {
        tracing::warn!(
            "Default retention changed: keeping 10 most recent backups. Use --no-prune to keep all. This notice won't appear again."
        );
        fs::write(&sentinel, b"")?;
    }
    Ok(())
}

pub fn run_create(
    ctx: &AppContext,
    output_dir: Option<PathBuf>,
    keep: Option<usize>,
    dry_run: bool,
    is_default_retention: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(0) = keep {
        return Err("--keep 0 is invalid; use --no-prune to disable pruning".into());
    }

    let mut service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    if let Some(dir) = output_dir {
        service = service.with_output_dir(dir);
    }

    if dry_run {
        let backup_path = service.preview_backup_path()?;
        let size = fs::metadata(&ctx.vault_path).map(|m| m.len()).unwrap_or(0);
        println!(
            "[dry-run] Would create backup at {}, source vault {}, estimated size {} bytes.",
            backup_path.display(),
            ctx.vault_path.display(),
            size
        );
        if let Some(n) = keep {
            let result = service.prune_backups(n, None, true)?;
            println!(
                "[dry-run] Would prune {} backup(s), {} remaining. Would free {:.2} MB.",
                result.pruned_count,
                result.remaining_count,
                result.freed_bytes as f64 / (1024.0 * 1024.0)
            );
        }
        return Ok(());
    }

    if is_default_retention {
        maybe_emit_retention_warning()?;
    }

    tracing::info!("Creating vault backup...");
    let conn = ctx.conn.lock()?;
    let backup_path = service.run_backup_from_conn(&conn)?;
    println!("Backup created and verified: {}", backup_path.display());

    if let Some(n) = keep {
        // Build a fresh service so prune_backups sees the newly created backup.
        let service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
        let result = service.prune_backups(n, None, false)?;
        if result.pruned_count > 0 {
            tracing::info!(
                "Pruned {} old backups (kept {}).",
                result.pruned_count,
                result.remaining_count
            );
        }
    }

    Ok(())
}

pub fn run_prune(
    ctx: &AppContext,
    keep: usize,
    older_than: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if keep == 0 {
        return Err("--keep 0 is invalid; use --no-prune to disable pruning".into());
    }

    let service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    let result = service.prune_backups(keep, older_than.as_deref(), dry_run)?;
    let freed_mib = result.freed_bytes as f64 / (1024.0 * 1024.0);

    if dry_run {
        println!(
            "Would prune {} backup(s), {} remaining. Would free {:.2} MB.",
            result.pruned_count, result.remaining_count, freed_mib
        );
    } else {
        println!(
            "Pruned {} backup(s), {} remaining. Freed {:.2} MB.",
            result.pruned_count, result.remaining_count, freed_mib
        );
    }
    Ok(())
}

pub fn run_list(ctx: &AppContext, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    let service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    let backups = service.list_backups(quiet)?;
    if backups.is_empty() {
        println!("No backups found.");
        return Ok(());
    }

    println!(
        "{:<35} {:<22} {:<40} {:<14} {:<20}",
        "Filename", "Timestamp", "Source Vault", "Version", "Size (bytes)"
    );
    for info in backups {
        let ts = info
            .timestamp
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "(unparseable)".to_string());
        let source = info
            .metadata
            .get("source_vault_path")
            .cloned()
            .unwrap_or_else(|| "(no metadata)".to_string());
        let version = info
            .metadata
            .get("ai_brains_version")
            .cloned()
            .unwrap_or_else(|| "(no metadata)".to_string());
        let size = info
            .metadata
            .get("backup_file_size_bytes")
            .cloned()
            .unwrap_or_else(|| "(no metadata)".to_string());
        let filename = info
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        println!(
            "{:<35} {:<22} {:<40} {:<14} {:<20}",
            filename,
            ts,
            truncate_right(&source, 40),
            truncate(&version, 14),
            size
        );
    }
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len - 3).collect::<String>() + "..."
    }
}

fn truncate_right(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else {
        let keep = max_len.saturating_sub(3);
        "...".to_string() + &s.chars().skip(char_count - keep).collect::<String>()
    }
}

#[derive(Debug, serde::Serialize)]
struct VerifyResult {
    path: String,
    status: String,
    check: String,
    tables: Vec<String>,
    size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct VerifyOutput {
    results: Vec<VerifyResult>,
}

pub fn run_verify(
    ctx: &AppContext,
    path: Option<PathBuf>,
    full: bool,
    format: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    let check_name = if full {
        "integrity_check"
    } else {
        "quick_check"
    };

    let paths: Vec<PathBuf> = match path {
        Some(p) => vec![p],
        None => service.find_backup_files()?,
    };

    tracing::info!("Verifying {} backup file(s)...", paths.len());
    let mut results = Vec::new();
    let mut any_failed = false;

    for path in &paths {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        tracing::info!("Verifying {} ({})...", filename, check_name);

        let size_bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let mut tables: Vec<String> = Vec::new();
        let status: String;
        let mut error_msg: Option<String> = None;

        match verify_single_backup(path, &ctx._key, full, &mut tables) {
            Ok(()) => {
                status = "ok".to_string();
                tracing::info!("{}: OK", filename);
            }
            Err(err) => {
                status = "fail".to_string();
                any_failed = true;
                error_msg = Some(err.to_string());
                tracing::info!("{}: FAIL — {}", filename, err);
            }
        }

        results.push(VerifyResult {
            path: path.to_string_lossy().to_string(),
            status,
            check: check_name.to_string(),
            tables,
            size_bytes,
            error: error_msg,
        });
    }

    if format.as_deref() == Some("json") {
        let output = VerifyOutput { results };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        for result in &results {
            let filename = PathBuf::from(&result.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| result.path.clone());
            if let Some(ref err) = result.error {
                println!("{}: FAIL — {}", filename, err);
            } else if result.status == "ok" {
                println!("{}: OK", filename);
            } else {
                println!("{}: FAIL", filename);
            }
        }
    }

    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

fn verify_single_backup(
    path: &PathBuf,
    key: &ai_brains_crypto::SqlCipherKey,
    full: bool,
    tables_out: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // T187: refuse plain legacy backup files with migrate hint class
    if ai_brains_store::is_plain_sqlite_header(path) {
        return Err(
            "Legacy plaintext backup (SQLite format 3 header): not SQLCipher-encrypted. \
             Key verification cannot succeed; re-create backup under live SQLCipher or use vault encrypt on a vault first."
                .into(),
        );
    }

    let conn = rusqlite::Connection::open(path)?;
    apply_key_pragmas(&conn, key).map_err(|e| format!("Key verification failed: {e}"))?;
    // Mandatory post-key schema read (F3)
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| format!("Key verification failed: {e}"))?;

    let check_sql = if full {
        "PRAGMA integrity_check"
    } else {
        "PRAGMA quick_check"
    };
    let check_res: String = match conn.query_row(check_sql, [], |row| row.get(0)) {
        Ok(v) => v,
        Err(err) => return Err(format!("{} query failed: {}", check_sql, err).into()),
    };
    if check_res.to_lowercase() != "ok" {
        return Err(format!("{} failed: {}", check_sql, check_res).into());
    }

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('events', 'memory_projection')",
    )?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?;
    for row in rows {
        tables_out.push(row?);
    }
    tables_out.sort();

    if tables_out.is_empty() {
        return Err("backup is missing core tables".into());
    }
    Ok(())
}

/// Robust daemon probe for destructive restore (T188 F1b).
///
/// Per-attempt timeout ≥1000ms; ≥2 retries (3 total attempts) with short
/// backoff. Returns true if any Ping/Pong succeeds.
pub async fn probe_restore_daemon_busy(client: &DaemonClient) -> bool {
    const ATTEMPTS: u32 = 3;
    const PER_ATTEMPT: std::time::Duration = std::time::Duration::from_millis(1000);
    const BACKOFF: std::time::Duration = std::time::Duration::from_millis(50);

    for attempt in 0..ATTEMPTS {
        if client.probe(PER_ATTEMPT).await {
            return true;
        }
        if attempt + 1 < ATTEMPTS {
            tokio::time::sleep(BACKOFF).await;
        }
    }
    false
}

/// Error message when mutating restore is blocked because the daemon is up.
///
/// Substring classes: `daemon is running`, `ai-brains daemon stop`, and
/// service stop guidance (`sc stop` + service name).
fn restore_daemon_busy_message() -> String {
    "Cannot restore: daemon is running and holds the vault open. \
     Stop it first with `ai-brains daemon stop`, or if installed as a Windows \
     service: `sc stop AI-Brains-Daemon` (service hosts `ai-brainsd`). \
     `--force` does not override this safety check."
        .to_string()
}

/// Prominent dry-run notice text when a live restore would fail (F3 / AC3).
const RESTORE_DRY_RUN_DAEMON_NOTICE: &str = "NOTICE: live restore will fail while the daemon is running. \
     Stop with `ai-brains daemon stop` or `sc stop AI-Brains-Daemon` before a real restore.";

fn restore_dry_run_daemon_notice() {
    println!("{RESTORE_DRY_RUN_DAEMON_NOTICE}");
}

pub async fn run_restore(
    ctx: &AppContext,
    backup_path: PathBuf,
    force: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let daemon_up = probe_restore_daemon_busy(&client).await;
    run_restore_with_daemon_state(ctx, backup_path, force, dry_run, daemon_up).await
}

/// Core restore path with injectable daemon-up state (T188 unit tests).
///
/// Production callers use [`run_restore`], which probes via
/// [`probe_restore_daemon_busy`]. Tests inject `daemon_up` without live IPC.
pub async fn run_restore_with_daemon_state(
    ctx: &AppContext,
    backup_path: PathBuf,
    force: bool,
    dry_run: bool,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !backup_path.exists() {
        return Err(format!("Backup file not found: {}", backup_path.display()).into());
    }

    // Verify integrity of the backup before doing anything destructive.
    // Apply key pragmas to read the encrypted backup.
    let bak_conn = rusqlite::Connection::open(&backup_path)?;
    apply_key_pragmas(&bak_conn, &ctx._key)?;
    let res: String = bak_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if res != "ok" {
        return Err(format!("Integrity check failed: {}", res).into());
    }

    // --dry-run: report and exit. No prompt, no overwrite. Allowed while daemon up (F3).
    if dry_run {
        if daemon_up {
            restore_dry_run_daemon_notice();
        }
        println!(
            "dry-run: backup {} verified ok; would overwrite vault at {} (no changes made).",
            backup_path.display(),
            ctx.vault_path.display()
        );
        print_backup_metadata(&bak_conn)?;
        return Ok(());
    }

    // T188 F1/F2: hard-fail when daemon is reachable. `--force` never overrides.
    if daemon_up {
        return Err(restore_daemon_busy_message().into());
    }

    // Interactive confirm unless --force was passed (e.g. in CI/automation).
    if !force {
        tracing::warn!(
            "This will overwrite the current vault at {}",
            ctx.vault_path.display()
        );
        eprint!("Type 'yes' to continue: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "yes" {
            return Err("Restore cancelled.".into());
        }
    }

    // Restore via SQLite backup API (overwrites current vault).
    // Apply key pragmas so the destination is encrypted.
    // Use apply_key_pragmas for the vault (may already be open by AppContext)
    // and apply_pragmas for the backup source connection.
    let mut vault_conn = rusqlite::Connection::open(&ctx.vault_path)?;
    apply_key_pragmas(&vault_conn, &ctx._key)?;
    {
        let backup = rusqlite::backup::Backup::new(&bak_conn, &mut vault_conn)?;
        backup.run_to_completion(10, std::time::Duration::from_millis(250), None)?;
    }

    // T109: Remove the backup metadata table from the live vault.
    vault_conn.execute_batch("DROP TABLE IF EXISTS _aibrains_backup_meta;")?;

    println!("Vault restored from: {}", backup_path.display());
    Ok(())
}

fn print_backup_metadata(conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    let table_exists: bool = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_aibrains_backup_meta'",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);

    if !table_exists {
        println!("Backup metadata: (no metadata)");
        return Ok(());
    }

    let mut stmt = conn.prepare("SELECT key, value FROM _aibrains_backup_meta")?;
    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((key, value))
    })?;

    let mut meta = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        meta.insert(k, v);
    }

    if meta.is_empty() {
        println!("Backup metadata: (no metadata)");
        return Ok(());
    }

    println!("Backup metadata:");
    let mut keys: Vec<&String> = meta.keys().collect();
    keys.sort();
    for key in keys {
        println!("  {}: {}", key, meta[key]);
    }
    Ok(())
}

#[cfg(test)]
mod restore_daemon_tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::SqlCipherKey;
    use ai_brains_store::connection::VaultConnection;
    use std::sync::Arc;
    use std::time::SystemTime;

    const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

    fn zero_key() -> SqlCipherKey {
        SqlCipherKey::from_raw(ZERO_KEY.to_string())
    }

    /// Build an AppContext over a temp vault. Holds `_allow` for the test duration.
    fn make_ctx(vault: PathBuf) -> (AppContext, TempEnv) {
        let allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let key = zero_key();
        let conn = VaultConnection::open(&vault, &key).expect("open vault");
        conn.migrate().expect("migrate");
        (
            AppContext {
                vault_path: vault,
                _key: key,
                conn: Arc::new(conn),
            },
            allow,
        )
    }

    fn seed_vault_and_backup(dir: &std::path::Path) -> (AppContext, PathBuf, PathBuf, TempEnv) {
        let vault = dir.join("vault.db");
        let (ctx, allow) = make_ctx(vault.clone());
        // Create a real Online Backup API backup of the empty migrated vault.
        let service = BackupService::new(vault.clone(), ctx._key.clone());
        let backup_path = {
            let conn = ctx.conn.lock().expect("lock");
            service.run_backup_from_conn(&conn).expect("create backup")
        };
        // Marker so overwrite detection is meaningful if restore incorrectly proceeds.
        {
            let conn = ctx.conn.lock().expect("lock");
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS _t188_marker (id INTEGER PRIMARY KEY, note TEXT); \
                 INSERT INTO _t188_marker (note) VALUES ('pre-restore-marker');",
            )
            .expect("marker");
        }
        (ctx, vault, backup_path, allow)
    }

    fn vault_bytes(path: &std::path::Path) -> Vec<u8> {
        fs::read(path).expect("read vault")
    }

    fn vault_mtime(path: &std::path::Path) -> SystemTime {
        fs::metadata(path).expect("meta").modified().expect("mtime")
    }

    #[tokio::test]
    async fn backup_restore__daemon_running__fails_no_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, vault, backup_path, _allow) = seed_vault_and_backup(dir.path());
        let before = vault_bytes(&vault);
        let before_mtime = vault_mtime(&vault);

        let result = run_restore_with_daemon_state(
            &ctx,
            backup_path,
            true,  // force must not override probe
            false, // mutating
            true,  // daemon_up simulated
        )
        .await;

        assert!(result.is_err(), "mutating restore must fail when daemon up");
        let err = result.unwrap_err().to_string();
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("daemon is running"),
            "must include 'daemon is running'; got: {err}"
        );
        assert!(
            err.contains("ai-brains daemon stop"),
            "must include daemon stop guidance; got: {err}"
        );
        assert!(
            err.contains("sc stop")
                && (err.contains("AI-Brains-Daemon") || err.contains("ai-brainsd")),
            "must include service stop guidance; got: {err}"
        );
        assert!(
            err.contains("--force") || lower.contains("force"),
            "should note force does not override; got: {err}"
        );

        let after = vault_bytes(&vault);
        assert_eq!(before, after, "vault bytes must be unchanged on hard-fail");
        assert_eq!(
            before_mtime,
            vault_mtime(&vault),
            "vault mtime must be unchanged on hard-fail"
        );
        // Marker must still be present (restore did not overwrite).
        let conn = ctx.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM _t188_marker", [], |r| r.get(0))
            .expect("marker still present");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn backup_restore__daemon_running_dry_run__ok_with_notice() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, vault, backup_path, _allow) = seed_vault_and_backup(dir.path());
        let before = vault_bytes(&vault);

        // F3 notice class must be present in the constant printed by dry-run.
        assert!(
            RESTORE_DRY_RUN_DAEMON_NOTICE.contains("live restore will fail"),
            "notice must include live-restore-will-fail class"
        );
        assert!(
            RESTORE_DRY_RUN_DAEMON_NOTICE
                .to_ascii_lowercase()
                .contains("daemon"),
            "notice must mention daemon"
        );

        let result = run_restore_with_daemon_state(
            &ctx,
            backup_path,
            false,
            true, // dry_run
            true, // daemon_up
        )
        .await;

        assert!(
            result.is_ok(),
            "dry-run + daemon up must succeed: {:?}",
            result.err().map(|e| e.to_string())
        );
        assert_eq!(before, vault_bytes(&vault), "dry-run must not mutate vault");
    }

    #[tokio::test]
    async fn backup_restore__daemon_down_force__succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let (ctx, vault, backup_path, _allow) = seed_vault_and_backup(dir.path());

        let result = run_restore_with_daemon_state(
            &ctx,
            backup_path.clone(),
            true,  // force skips confirm
            false, // mutating
            false, // daemon_down
        )
        .await;

        assert!(
            result.is_ok(),
            "daemon-down force restore must succeed: {:?}",
            result.err().map(|e| e.to_string())
        );
        // Marker table is only on live vault; restore overwrites from backup
        // (which was taken before marker) so marker is gone — proves overwrite.
        let vault_conn = rusqlite::Connection::open(&vault).unwrap();
        apply_key_pragmas(&vault_conn, &ctx._key).unwrap();
        let has_marker: bool = vault_conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_t188_marker'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(
            !has_marker,
            "successful restore from pre-marker backup must drop live marker table"
        );
        // Backup meta must be dropped from live vault (T109).
        let has_meta: bool = vault_conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='_aibrains_backup_meta'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(
            !has_meta,
            "backup meta must be absent on live after restore"
        );
        let _ = backup_path;
    }

    #[test]
    fn restore_daemon_busy_message__has_required_classes() {
        let msg = restore_daemon_busy_message();
        let lower = msg.to_ascii_lowercase();
        assert!(lower.contains("daemon is running"));
        assert!(msg.contains("ai-brains daemon stop"));
        assert!(msg.contains("sc stop"));
        assert!(msg.contains("AI-Brains-Daemon") || msg.contains("ai-brainsd"));
    }
}
