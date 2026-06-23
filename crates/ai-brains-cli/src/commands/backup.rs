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

pub fn run_list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let service = BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    let backups = service.list_backups()?;
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

        match verify_single_backup(path, &ctx._key, full, &mut tables) {
            Ok(()) => {
                status = "ok".to_string();
                tracing::info!("{}: OK", filename);
            }
            Err(err) => {
                status = "fail".to_string();
                any_failed = true;
                tracing::info!("{}: FAIL — {}", filename, err);
            }
        }

        results.push(VerifyResult {
            path: path.to_string_lossy().to_string(),
            status,
            check: check_name.to_string(),
            tables,
            size_bytes,
        });
    }

    if format.as_deref() == Some("json") {
        let output = VerifyOutput { results };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        for result in &results {
            let label = if result.status == "ok" { "OK" } else { "FAIL" };
            println!(
                "{}: {}",
                PathBuf::from(&result.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| result.path.clone()),
                label
            );
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
    let conn = rusqlite::Connection::open(path)?;
    apply_key_pragmas(&conn, key)?;

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

pub async fn run_restore(
    ctx: &AppContext,
    backup_path: PathBuf,
    force: bool,
    dry_run: bool,
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

    // --dry-run: report and exit. No prompt, no overwrite.
    if dry_run {
        println!(
            "dry-run: backup {} verified ok; would overwrite vault at {} (no changes made).",
            backup_path.display(),
            ctx.vault_path.display()
        );
        print_backup_metadata(&bak_conn)?;
        return Ok(());
    }

    // AC7: Warn if the daemon is running, as restoring may corrupt the
    // daemon's open connection. The busy_timeout pragma handles lock
    // contention, but overwriting a file the daemon has open is risky.
    let client = DaemonClient::new();
    if client.probe(std::time::Duration::from_millis(200)).await {
        tracing::warn!(
            "Daemon is running. Restoring while the daemon has the vault open \
             may cause corruption. Consider running `ai-brains daemon stop` first."
        );
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
