//! `ai-brains doctor` — read-only operator health surface (T192).
//!
//! Never migrates, never creates vault/backups dirs, never prints secrets.
//! Opens vault via [`VaultConnection::open_read_intent`] only (no AppContext).

use crate::commands::backup::probe_restore_daemon_busy;
use crate::commands::device::data_key_from_sqlcipher;
use crate::commands::recovery::acquire_passphrase;
use crate::daemon_client::DaemonClient;
use ai_brains_brain::{BackupService, has_core_tables, parse_duration};
use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport, DoctorStatus, HealthCheck};
use ai_brains_crypto::{RecoveryKit, SqlCipherKey};
use ai_brains_store::ALLOW_ZERO_KEY_ENV;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::pragmas::cipher_version;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// CLI options for `doctor`.
pub struct DoctorOptions {
    pub vault_path: PathBuf,
    pub key: Option<String>,
    pub format: String,
    pub json: bool,
    pub fail_on_degraded: bool,
    pub kit_path: Option<PathBuf>,
    pub passphrase_file: Option<PathBuf>,
    pub backup_max_age: String,
    pub full: bool,
}

/// Production entry: probe daemon, run checks, emit report, map exit code.
pub async fn run(opts: DoctorOptions) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let daemon_up = probe_restore_daemon_busy(&client).await;
    let code = run_with_daemon_state(opts, daemon_up)?;
    if code != 0 {
        process::exit(code);
    }
    Ok(())
}

/// Core runner with injectable daemon-up (unit / hermetic injection).
///
/// Returns the process exit code (0 or 1) without calling [`process::exit`] so
/// unit tests can invoke this path in-process. Production [`run`] maps non-zero
/// to `process::exit`.
pub fn run_with_daemon_state(
    opts: DoctorOptions,
    daemon_up: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let report = build_report(&opts, daemon_up)?;
    emit_report(&report, &opts.format, opts.json)?;
    Ok(exit_code_for(&report, opts.fail_on_degraded))
}

/// Build a full doctor report (pure-ish; may open vault read-only).
pub fn build_report(
    opts: &DoctorOptions,
    daemon_up: bool,
) -> Result<DoctorReport, Box<dyn std::error::Error>> {
    let key = resolve_sqlcipher_key(opts.key.clone())?;
    let vault_path = &opts.vault_path;
    let mut checks: Vec<HealthCheck> = Vec::with_capacity(10);

    // 1. vault_exists
    let exists_check = check_vault_exists(vault_path);
    let vault_exists_ok = exists_check.severity == CheckSeverity::Ok;
    checks.push(exists_check);

    // 2. vault_open (open_read_intent only)
    let open_result = if vault_exists_ok {
        VaultConnection::open_read_intent(vault_path, &key)
    } else {
        Err(ai_brains_store::StoreError::ConnectionFailed(
            "vault missing; open skipped".into(),
        ))
    };

    checks.push(match &open_result {
        Ok(_) => HealthCheck::ok_msg("vault_open", "opened read-only"),
        Err(_) if !vault_exists_ok => HealthCheck::fail(
            "vault_open",
            "vault not openable (missing, not a regular file, or reparse refused)",
            Some(
                "create vault with `ai-brains init` or pass a valid regular-file --vault-path"
                    .into(),
            ),
        ),
        Err(e) => HealthCheck::fail(
            "vault_open",
            format!("open_read_intent failed: {e}"),
            Some("verify --key / AI_BRAINS_KEY matches the vault".into()),
        ),
    });

    // Hold VaultConnection for subsequent open-dependent checks (lock per use).
    let vault_conn = open_result.ok();

    // 3. schema_readable
    checks.push(match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
        Some(conn) => {
            if has_core_tables(&conn) {
                HealthCheck::ok_msg("schema_readable", "core tables present")
            } else {
                HealthCheck::fail(
                    "schema_readable",
                    "missing core tables (events and/or memory_projection)",
                    Some("re-init or restore from a known-good backup".into()),
                )
            }
        }
        None if vault_conn.is_none() => {
            HealthCheck::skip("schema_readable", "skipped: vault open failed")
        }
        None => HealthCheck::fail("schema_readable", "failed to lock vault connection", None),
    });

    // 4. cipher_page
    checks.push(match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
        Some(conn) => match cipher_version(&conn) {
            Ok(ver) if !ver.trim().is_empty() => {
                HealthCheck::ok_msg("cipher_page", format!("cipher_version={ver}"))
            }
            Ok(_) => HealthCheck::fail(
                "cipher_page",
                "PRAGMA cipher_version empty (SQLCipher not linked?)",
                Some("rebuild with bundled-sqlcipher; see COMPATIBILITY F8 / T187".into()),
            ),
            Err(e) => HealthCheck::fail(
                "cipher_page",
                format!("cipher_version probe failed: {e}"),
                None,
            ),
        },
        None if vault_conn.is_none() => {
            HealthCheck::skip("cipher_page", "skipped: vault open failed")
        }
        None => HealthCheck::fail("cipher_page", "failed to lock vault connection", None),
    });

    // 5. daemon_reachable (info: never hard-fail alone)
    checks.push(if daemon_up {
        HealthCheck::ok_msg("daemon_reachable", "up")
    } else {
        HealthCheck::ok_msg("daemon_reachable", "down")
    });

    // 6. backup_recent (soft)
    checks.push(check_backup_recent(vault_path, &key, &opts.backup_max_age));

    // 7. recovery_kit_event (soft) — event_type stored WITHOUT JSON quotes
    //    (event_store.rs trim_matches('"'); live fact vs early F16 draft).
    checks.push(match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
        Some(conn) => check_recovery_kit_event(&conn),
        None if vault_conn.is_none() => HealthCheck::warn(
            "recovery_kit_event",
            "cannot query events (vault open failed)",
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
        None => HealthCheck::warn(
            "recovery_kit_event",
            "failed to lock vault connection for event query",
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
    });

    // 8. recovery_kit_file
    checks.push(check_recovery_kit_file(
        opts.kit_path.as_deref(),
        opts.passphrase_file.as_deref(),
        &key,
    ));

    // 9. zero_key_escape (soft)
    checks.push(check_zero_key_escape(&key));

    // 10. integrity (optional --full)
    checks.push(if opts.full {
        match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
            Some(conn) => check_integrity(&conn),
            None if vault_conn.is_none() => {
                HealthCheck::skip("integrity", "skipped: vault open failed")
            }
            None => HealthCheck::fail("integrity", "failed to lock vault connection", None),
        }
    } else {
        HealthCheck::skip("integrity", "pass --full to run PRAGMA integrity_check")
    });

    let status = DoctorReport::roll_up(&checks);
    let generated_at = Utc::now().to_rfc3339();

    Ok(DoctorReport {
        schema_version: DoctorReport::SCHEMA_VERSION,
        status,
        checks,
        vault_path: vault_path.display().to_string(),
        generated_at,
    })
}

fn check_vault_exists(vault_path: &Path) -> HealthCheck {
    // Prefer fail on reparse vault.
    match ai_brains_path::is_reparse_or_symlink(vault_path) {
        Ok(true) => {
            return HealthCheck::fail(
                "vault_exists",
                format!(
                    "vault path is a reparse point/symlink/junction (refused): {}",
                    vault_path.display()
                ),
                Some("point --vault-path at a regular vault file".into()),
            );
        }
        Ok(false) => {}
        Err(e) => {
            // Missing path often errors on some platforms; fall through to exists check.
            tracing::debug!(error = %e, "vault reparse probe failed; continuing with exists check");
        }
    }

    if !vault_path.exists() {
        return HealthCheck::fail(
            "vault_exists",
            format!("vault not found: {}", vault_path.display()),
            Some(
                "create with `ai-brains init` or correct --vault-path / AI_BRAINS_VAULT_PATH"
                    .into(),
            ),
        );
    }
    if !vault_path.is_file() {
        return HealthCheck::fail(
            "vault_exists",
            format!("vault path is not a regular file: {}", vault_path.display()),
            None,
        );
    }
    HealthCheck::ok_msg("vault_exists", "present")
}

fn check_backup_recent(vault_path: &Path, key: &SqlCipherKey, max_age: &str) -> HealthCheck {
    let threshold = match parse_duration(max_age) {
        Ok(d) => d,
        Err(e) => {
            return HealthCheck::warn(
                "backup_recent",
                format!("invalid --backup-max-age '{max_age}': {e}"),
                Some("use Nd / Nh / Nw (same as prune --older-than)".into()),
            );
        }
    };

    let service = BackupService::new(vault_path.to_path_buf(), key.clone());
    let backups = match service.list_backups(true) {
        Ok(b) => b,
        Err(e) => {
            return HealthCheck::warn(
                "backup_recent",
                format!("list_backups failed: {e}"),
                Some("ai-brains backup create".into()),
            );
        }
    };

    if backups.is_empty() {
        return HealthCheck::warn(
            "backup_recent",
            "no backups found (directory may be absent)",
            Some("ai-brains backup create".into()),
        );
    }

    let newest = backups.iter().find_map(|b| b.timestamp);
    let Some(ts) = newest else {
        return HealthCheck::warn(
            "backup_recent",
            "backups present but timestamps unparseable",
            Some("ai-brains backup create".into()),
        );
    };

    let age = Utc::now().signed_duration_since(ts.and_utc());
    let max = match chrono::Duration::from_std(threshold) {
        Ok(d) => d,
        Err(e) => {
            return HealthCheck::warn(
                "backup_recent",
                format!("--backup-max-age '{max_age}' is out of chrono range: {e}"),
                Some("use a smaller Nd / Nh / Nw value".into()),
            );
        }
    };
    if age <= max {
        HealthCheck::ok_msg(
            "backup_recent",
            format!(
                "newest backup within {max_age} (timestamp {})",
                ts.format("%Y-%m-%dT%H:%M:%S")
            ),
        )
    } else {
        HealthCheck::warn(
            "backup_recent",
            format!(
                "newest backup older than {max_age} (timestamp {})",
                ts.format("%Y-%m-%dT%H:%M:%S")
            ),
            Some("ai-brains backup create".into()),
        )
    }
}

/// Count RecoveryKitCreated events.
///
/// **Storage truth (T192 implementer note):** `event_store` serializes kind
/// then `trim_matches('"')`, so the column stores `RecoveryKitCreated`
/// (no surrounding JSON quotes). Spec F16 draft text claiming
/// `'"RecoveryKitCreated"'` is incorrect against live code.
fn check_recovery_kit_event(conn: &rusqlite::Connection) -> HealthCheck {
    const KIND: &str = "RecoveryKitCreated";
    let count: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM events WHERE event_type = ?1",
        [KIND],
        |row| row.get(0),
    );
    match count {
        Ok(n) if n >= 1 => HealthCheck::ok_msg(
            "recovery_kit_event",
            format!(
                "{n} RecoveryKitCreated event(s) in log (does not prove offline kit file still exists)"
            ),
        ),
        Ok(_) => HealthCheck::warn(
            "recovery_kit_event",
            "no RecoveryKitCreated event in vault log",
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
        Err(e) => HealthCheck::warn(
            "recovery_kit_event",
            format!("event query failed: {e}"),
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
    }
}

fn check_recovery_kit_file(
    kit_path: Option<&Path>,
    passphrase_file: Option<&Path>,
    vault_key: &SqlCipherKey,
) -> HealthCheck {
    let Some(path) = kit_path else {
        return HealthCheck::skip("recovery_kit_file", "pass --kit-path to verify offline kit");
    };

    // F15b: refuse reparse/symlink/junction before read.
    match ai_brains_path::is_reparse_or_symlink(path) {
        Ok(is_reparse) => {
            if let Some(fail) = recovery_kit_file_reparse_fail(path, is_reparse) {
                return fail;
            }
        }
        Err(e) => {
            return HealthCheck::fail(
                "recovery_kit_file",
                format!("kit path check failed ({}): {e}", path.display()),
                None,
            );
        }
    }

    let kit_bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return HealthCheck::fail(
                "recovery_kit_file",
                format!("failed to read kit ({}): {e}", path.display()),
                Some("check --kit-path exists and is readable".into()),
            );
        }
    };
    let kit_str = match std::str::from_utf8(&kit_bytes) {
        Ok(s) => s,
        Err(_) => {
            return HealthCheck::fail(
                "recovery_kit_file",
                "kit file is not valid UTF-8 JSON",
                None,
            );
        }
    };
    let kit = match RecoveryKit::from_json(kit_str) {
        Ok(k) => k,
        Err(_) => {
            // F22: do not echo serde/path snippets that could include kit field material.
            return HealthCheck::fail(
                "recovery_kit_file",
                "failed to parse RecoveryKit JSON (invalid schema or corrupted file)",
                Some("re-export with `ai-brains recovery export --output <path>`".into()),
            );
        }
    };

    let unlocked = match unlock_kit(&kit, passphrase_file) {
        Ok(k) => k,
        Err(e) => {
            return HealthCheck::fail(
                "recovery_kit_file",
                format!("kit unlock failed: {e}"),
                Some(
                    "provide --passphrase-file (or TTY) matching the kit, or use a DPAPI kit on the same machine"
                        .into(),
                ),
            );
        }
    };

    let vault_data_key = match data_key_from_sqlcipher(vault_key) {
        Ok(k) => k,
        Err(e) => {
            return HealthCheck::fail(
                "recovery_kit_file",
                format!("cannot derive vault DataKey for compare: {e}"),
                None,
            );
        }
    };

    if unlocked.expose_secret() == vault_data_key.expose_secret() {
        HealthCheck::ok_msg(
            "recovery_kit_file",
            "offline kit unlocks and matches vault key",
        )
    } else {
        HealthCheck::fail(
            "recovery_kit_file",
            "kit unlocked but DataKey does not match vault key",
            Some(
                "export a new kit for this vault: ai-brains recovery export --output <path>".into(),
            ),
        )
    }
}

/// F15b helper: when `is_reparse` is true, return a fail check (unit-testable).
fn recovery_kit_file_reparse_fail(path: &Path, is_reparse: bool) -> Option<HealthCheck> {
    if !is_reparse {
        return None;
    }
    if let Err(msg) = ai_brains_path::refuse_if_reparse(path, true) {
        return Some(HealthCheck::fail(
            "recovery_kit_file",
            msg,
            Some("point --kit-path at a regular kit file (no symlink/junction)".into()),
        ));
    }
    Some(HealthCheck::fail(
        "recovery_kit_file",
        format!("kit path is reparse/symlink/junction: {}", path.display()),
        Some("point --kit-path at a regular kit file (no symlink/junction)".into()),
    ))
}

/// Unlock kit: passphrase-file / TTY when available; else DPAPI-only if present.
fn unlock_kit(
    kit: &RecoveryKit,
    passphrase_file: Option<&Path>,
) -> Result<ai_brains_crypto::DataKey, Box<dyn std::error::Error>> {
    // Prefer passphrase when a source is available (file or we will try TTY).
    // If no passphrase-file and stdin is not a TTY, fall back to DPAPI-only.
    let try_passphrase = passphrase_file.is_some() || is_terminal::is_terminal(std::io::stdin());

    if try_passphrase {
        match acquire_passphrase(passphrase_file) {
            Ok(pass) => {
                let result = kit
                    .unlock_with_passphrase(pass.as_slice())
                    .map_err(|e| format!("passphrase unlock failed: {e}").into());
                drop(pass);
                return result;
            }
            Err(e) if passphrase_file.is_some() => {
                return Err(e);
            }
            Err(_) => {
                // TTY path failed unexpectedly; try DPAPI fallback below.
            }
        }
    }

    if kit.dpapi.is_some() {
        return kit
            .unlock_with_dpapi()
            .map_err(|e| format!("DPAPI unlock failed: {e}").into());
    }

    Err(
        "no passphrase source (pass --passphrase-file or use a TTY) and kit has no DPAPI wrap"
            .into(),
    )
}

fn check_zero_key_escape(key: &SqlCipherKey) -> HealthCheck {
    // Match store `zero_key_allowed` truthy semantics (1/true/yes), not mere presence.
    let escape_enabled = match std::env::var(ALLOW_ZERO_KEY_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    };
    if key.is_zero() || escape_enabled {
        let mut parts = Vec::new();
        if key.is_zero() {
            parts.push("vault key is all-zero material");
        }
        if escape_enabled {
            parts.push("AI_BRAINS_ALLOW_ZERO_KEY is enabled");
        }
        HealthCheck::warn(
            "zero_key_escape",
            format!("{} (R-ZERO-KEY honesty)", parts.join("; ")),
            Some(
                "use a non-zero production key; unset AI_BRAINS_ALLOW_ZERO_KEY outside tests"
                    .into(),
            ),
        )
    } else {
        HealthCheck::ok_msg("zero_key_escape", "non-zero key; escape hatch unset")
    }
}

fn check_integrity(conn: &rusqlite::Connection) -> HealthCheck {
    match conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)) {
        Ok(res) if res.eq_ignore_ascii_case("ok") => {
            HealthCheck::ok_msg("integrity", "integrity_check ok")
        }
        Ok(res) => HealthCheck::fail(
            "integrity",
            format!("integrity_check failed: {res}"),
            Some("restore from a known-good backup".into()),
        ),
        Err(e) => HealthCheck::fail("integrity", format!("integrity_check error: {e}"), None),
    }
}

fn resolve_sqlcipher_key(key: Option<String>) -> Result<SqlCipherKey, Box<dyn std::error::Error>> {
    // Same default zero-key path as recovery export / AppContext (tests use ALLOW_ZERO_KEY).
    let key_str = key.unwrap_or_else(|| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });
    let sql = SqlCipherKey::from_raw(key_str);
    if let Err(e) = sql.validate() {
        return Err(format!("invalid vault key: {e}").into());
    }
    Ok(sql)
}

/// Exit code policy (F9): 0 for ok|degraded; 1 for fail; --fail-on-degraded → 1.
pub fn exit_code_for(report: &DoctorReport, fail_on_degraded: bool) -> i32 {
    match report.status {
        DoctorStatus::Ok => 0,
        DoctorStatus::Degraded if fail_on_degraded => 1,
        DoctorStatus::Degraded => 0,
        DoctorStatus::Fail => 1,
    }
}

fn emit_report(
    report: &DoctorReport,
    format: &str,
    force_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let use_json = force_json || format.eq_ignore_ascii_case("json");
    if use_json {
        let json = serde_json::to_string_pretty(report)?;
        println!("{json}");
        return Ok(());
    }
    // Human
    println!(
        "doctor: status={}  vault={}  checks={}",
        status_label(report.status),
        report.vault_path,
        report.checks.len()
    );
    for c in &report.checks {
        let sev = severity_label(c.severity);
        let msg = c.message.as_deref().unwrap_or("");
        println!("  [{sev}] {} — {msg}", c.name);
        if let Some(rem) = &c.remediation {
            println!("         remediation: {rem}");
        }
    }
    Ok(())
}

fn status_label(s: DoctorStatus) -> &'static str {
    match s {
        DoctorStatus::Ok => "ok",
        DoctorStatus::Degraded => "degraded",
        DoctorStatus::Fail => "fail",
    }
}

fn severity_label(s: CheckSeverity) -> &'static str {
    match s {
        CheckSeverity::Ok => "ok",
        CheckSeverity::Warn => "warn",
        CheckSeverity::Fail => "fail",
        CheckSeverity::Skip => "skip",
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport, HealthCheck};

    #[test]
    fn roll_up__fail_beats_warn() {
        let checks = vec![
            HealthCheck::warn("a", "w", None),
            HealthCheck::fail("b", "f", None),
        ];
        assert_eq!(DoctorReport::roll_up(&checks), DoctorStatus::Fail);
    }

    #[test]
    fn exit_code_for__ok_and_degraded_default_0() {
        let ok = DoctorReport {
            schema_version: 1,
            status: DoctorStatus::Ok,
            checks: vec![],
            vault_path: "v".into(),
            generated_at: "t".into(),
        };
        let deg = DoctorReport {
            status: DoctorStatus::Degraded,
            ..ok.clone()
        };
        let fail = DoctorReport {
            status: DoctorStatus::Fail,
            ..ok.clone()
        };
        assert_eq!(exit_code_for(&ok, false), 0);
        assert_eq!(exit_code_for(&deg, false), 0);
        assert_eq!(exit_code_for(&deg, true), 1);
        assert_eq!(exit_code_for(&fail, false), 1);
        assert_eq!(exit_code_for(&fail, true), 1);
    }

    #[test]
    fn health_check_order_names__fixed_matrix() {
        // Document expected fixed order for determinism (F30).
        let expected = [
            "vault_exists",
            "vault_open",
            "schema_readable",
            "cipher_page",
            "daemon_reachable",
            "backup_recent",
            "recovery_kit_event",
            "recovery_kit_file",
            "zero_key_escape",
            "integrity",
        ];
        assert_eq!(expected.len(), 10);
        // Ensure HealthCheck helpers set ok flag correctly.
        assert!(HealthCheck::skip("integrity", "x").ok);
        assert_eq!(
            HealthCheck::skip("integrity", "x").severity,
            CheckSeverity::Skip
        );
    }

    /// AC8: build_report with daemon_up=true still uses open_read_intent only
    /// (never AppContext/migrate). Injectable daemon flag must not force fail.
    #[test]
    fn doctor__no_migrate_while_daemon_up__build_report_ok() {
        use ai_brains_core::temp_env::TempEnv;
        use ai_brains_store::connection::VaultConnection;
        use tempfile::tempdir;

        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        {
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate once for fixture");
        }

        let opts = DoctorOptions {
            vault_path: vault.clone(),
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
        };
        // daemon_up=true (simulates busy probe) — must not hard-fail or migrate.
        let report = build_report(&opts, true).expect("build_report");
        assert_eq!(report.schema_version, 1);
        assert!(
            matches!(report.status, DoctorStatus::Ok | DoctorStatus::Degraded),
            "daemon-up doctor must be ok|degraded, got {:?}",
            report.status
        );
        let daemon = report
            .checks
            .iter()
            .find(|c| c.name == "daemon_reachable")
            .expect("daemon check");
        assert_eq!(daemon.severity, CheckSeverity::Ok);
        assert_eq!(daemon.message.as_deref(), Some("up"));
        let open = report
            .checks
            .iter()
            .find(|c| c.name == "vault_open")
            .expect("vault_open");
        assert_eq!(open.severity, CheckSeverity::Ok);

        // Still readable via open_read_intent (no exclusive writer side effects).
        VaultConnection::open_read_intent(&vault, &key).expect("still open_read_intent");
    }

    /// F22: kit parse failure messages must not echo raw serde / field snippets.
    #[test]
    fn doctor__kit_parse_fail__message_has_no_secretish_payload() {
        use tempfile::tempdir;

        let dir = tempdir().expect("tempdir");
        let kit_path = dir.path().join("bad-kit.json");
        // Looks like kit shape fragments that must not be re-echoed from serde.
        let toxic = r#"{"passphrase":{"ciphertext":"DEADBEEFSECRET","salt":"aa","nonce":"bb"}}"#;
        std::fs::write(&kit_path, toxic).expect("write");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        let check = check_recovery_kit_file(Some(&kit_path), None, &key);
        assert_eq!(check.severity, CheckSeverity::Fail);
        let msg = check.message.as_deref().unwrap_or("");
        assert!(
            !msg.contains("DEADBEEFSECRET") && !msg.contains("ciphertext"),
            "parse fail must not echo kit field material: {msg}"
        );
        assert!(
            msg.contains("failed to parse RecoveryKit"),
            "expected generic parse fail message, got {msg}"
        );
    }

    /// AC6 / F15b: injected reparse flag refuses without reading kit (privilege-free).
    #[test]
    fn doctor__kit_path_reparse__unit_refused() {
        use std::path::PathBuf;
        let path = PathBuf::from("C:\\fake\\kit-reparse.json");
        let check = recovery_kit_file_reparse_fail(&path, true).expect("must fail when reparse");
        assert_eq!(check.severity, CheckSeverity::Fail);
        let msg = check.message.as_deref().unwrap_or("").to_ascii_lowercase();
        assert!(
            msg.contains("reparse")
                || msg.contains("symlink")
                || msg.contains("junction")
                || msg.contains("kit"),
            "expected reparse refuse message, got: {:?}",
            check.message
        );
        assert!(recovery_kit_file_reparse_fail(&path, false).is_none());
    }

    /// Oversized --backup-max-age must warn (parse_duration checked mul), never panic.
    #[test]
    fn doctor__backup_max_age_overflow__warns_not_panic() {
        use tempfile::tempdir;

        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        // No vault needed for age parse path when list is empty, but check_backup_recent
        // parses age first — exercise overflow before list.
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        let check = check_backup_recent(&vault, &key, "18446744073709551615d");
        assert_eq!(check.severity, CheckSeverity::Warn);
        let msg = check.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("invalid") || msg.contains("overflow"),
            "expected overflow/invalid age message, got {msg}"
        );
    }

    const ZERO_KEY_LITERAL: &str =
        "x'0000000000000000000000000000000000000000000000000000000000000000'";
}
