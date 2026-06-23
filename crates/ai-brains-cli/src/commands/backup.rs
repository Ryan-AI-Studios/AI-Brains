use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_store::pragmas::apply_key_pragmas;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn run_create(
    ctx: &AppContext,
    output_dir: Option<PathBuf>,
    keep: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut service = ai_brains_brain::BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    if let Some(dir) = output_dir {
        service = service.with_output_dir(dir);
    }
    eprintln!("Creating vault backup...");
    // Use the existing AppContext connection to avoid opening a second
    // connection to the same WAL file (which deadlocks).
    let conn = ctx.conn.lock()?;
    let backup_path = service.run_backup_from_conn(&conn)?;
    println!("Backup created and verified: {}", backup_path.display());

    if let Some(n) = keep {
        let result = service.prune_backups(n, None, false)?;
        println!(
            "Pruned {} backup(s), {} remaining. Freed {:.2} MB.",
            result.pruned_count,
            result.remaining_count,
            result.freed_bytes as f64 / (1024.0 * 1024.0)
        );
    }

    Ok(())
}

pub fn run_prune(
    ctx: &AppContext,
    keep: usize,
    older_than: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = ai_brains_brain::BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
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
    let service = ai_brains_brain::BackupService::new(ctx.vault_path.clone(), ctx._key.clone());
    let backups = service.list_backups()?;
    if backups.is_empty() {
        println!("No backups found.");
        return Ok(());
    }

    println!(
        "{:<40} {:<22} {:<26} {:<14} {:<20}",
        "Path", "Timestamp", "Source Vault", "Version", "Size (bytes)"
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
        println!(
            "{:<40} {:<22} {:<26} {:<14} {:<20}",
            truncate(&info.path.to_string_lossy(), 40),
            ts,
            truncate(&source, 26),
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
        eprintln!(
            "WARNING: Daemon is running. Restoring while the daemon has the vault open \
             may cause corruption. Consider running `ai-brains daemon stop` first."
        );
    }

    // Interactive confirm unless --force was passed (e.g. in CI/automation).
    if !force {
        eprintln!(
            "WARNING: This will overwrite the current vault at {}",
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
