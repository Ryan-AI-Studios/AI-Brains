//! `ai-brains doctor` — read-only operator health surface (T192).
//!
//! Never migrates, never creates vault/backups dirs, never prints secrets.
//! Opens vault via [`VaultConnection::open_read_intent`] only (no AppContext).

use crate::commands::backup::probe_restore_daemon_busy;
use crate::commands::device::data_key_from_sqlcipher;
use crate::commands::governed_common::{
    DISCOVERY_CAP_LABELS, POLICY_BOOTSTRAP_SOOT_LONG, discovery_active_count, resolve_principal,
    resolve_scope_key_for_cli,
};
use crate::commands::recovery::acquire_passphrase;
use crate::daemon_client::DaemonClient;
use crate::graph_density::{
    DensityVerdict, GatherResult, assess_graph_density, gather_density_snapshot,
};
use crate::key_resolve::{KeyResolveError, resolve_operator_sqlcipher_key, vault_locked_message};
use ai_brains_brain::{BackupService, ListMode, has_core_tables, is_usable_class, parse_duration};
use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport, DoctorStatus, HealthCheck};
use ai_brains_control_plane::StorePorts;
use ai_brains_crypto::{RecoveryKit, SqlCipherKey};
use ai_brains_store::ALLOW_ZERO_KEY_ENV;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::StoreError;
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
    pub summary: bool,
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
    emit_report(&report, &opts.format, opts.json, opts.summary)?;
    Ok(exit_code_for(&report, opts.fail_on_degraded))
}

/// Build a full doctor report (pure-ish; may open vault read-only).
///
/// T197 F9: missing key → still emit report; `vault_open` = **skipped**; overall
/// status forced to Fail (exit 1). Format/Zero fail early with F8 messages.
/// Wrong key → `vault_open` = **fail** with Vault locked hint; no hmac spam.
pub fn build_report(
    opts: &DoctorOptions,
    daemon_up: bool,
) -> Result<DoctorReport, Box<dyn std::error::Error>> {
    // Format / Zero: early clear F8 (prefer not conflate with vault_open).
    // Missing: continue so vault_exists still runs and report emits (F9).
    let key_result = resolve_operator_sqlcipher_key(opts.key.clone());
    let (key_opt, key_missing) = match key_result {
        Ok(k) => (Some(k), false),
        Err(KeyResolveError::Missing) => (None, true),
        Err(e) => return Err(e.into()),
    };

    let vault_path = &opts.vault_path;
    let mut checks: Vec<HealthCheck> = Vec::with_capacity(15);

    // 1. vault_exists
    let exists_check = check_vault_exists(vault_path);
    let vault_exists_ok = exists_check.severity == CheckSeverity::Ok;
    checks.push(exists_check);

    // 2. vault_open (open_read_intent only) — F9 missing vs wrong
    let open_result = match key_opt.as_ref() {
        None => None,
        Some(key) if vault_exists_ok => Some(VaultConnection::open_read_intent(vault_path, key)),
        Some(_) => Some(Err(StoreError::ConnectionFailed(
            "vault missing; open skipped".into(),
        ))),
    };

    checks.push(match (&open_result, key_missing) {
        (_, true) => HealthCheck::skip(
            "vault_open",
            "skipped: vault key missing — set --key or AI_BRAINS_KEY (see INSTALL)",
        ),
        (Some(Ok(_)), false) => HealthCheck::ok_msg("vault_open", "opened read-only"),
        (Some(Err(_)), false) if !vault_exists_ok => HealthCheck::fail(
            "vault_open",
            "vault not openable (missing, not a regular file, or reparse refused)",
            Some(
                "create vault with `ai-brains init` or pass a valid regular-file --vault-path"
                    .into(),
            ),
        ),
        (Some(Err(e)), false) => {
            let detail = e.to_string();
            let is_locked = matches!(e, StoreError::VaultLocked(_))
                || detail.to_ascii_lowercase().contains("vault is locked")
                || detail.to_ascii_lowercase().contains("key verification");
            let msg = if is_locked {
                vault_locked_message("wrong key or cannot decrypt")
            } else {
                format!("open_read_intent failed: {detail}")
            };
            HealthCheck::fail(
                "vault_open",
                msg,
                Some("verify --key / AI_BRAINS_KEY matches the vault (see INSTALL)".into()),
            )
        }
        (None, false) => HealthCheck::fail("vault_open", "vault open skipped unexpectedly", None),
    });

    // Hold VaultConnection for subsequent open-dependent checks (lock per use).
    let vault_conn = open_result.and_then(|r| r.ok());
    let open_failed = vault_conn.is_none();
    let skip_reason = if key_missing {
        "skipped: vault key missing"
    } else {
        "skipped: vault open failed"
    };

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
        None if open_failed => HealthCheck::skip("schema_readable", skip_reason),
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
        None if open_failed => HealthCheck::skip("cipher_page", skip_reason),
        None => HealthCheck::fail("cipher_page", "failed to lock vault connection", None),
    });

    // 5. daemon_reachable (info: never hard-fail alone)
    checks.push(if daemon_up {
        HealthCheck::ok_msg("daemon_reachable", "up")
    } else {
        HealthCheck::ok_msg("daemon_reachable", "down")
    });

    // 6. backup_recent (soft) — needs key
    checks.push(match key_opt.as_ref() {
        Some(key) => check_backup_recent(vault_path, key, &opts.backup_max_age),
        None => HealthCheck::skip("backup_recent", skip_reason),
    });

    // 7. recovery_kit_event (soft) — event_type stored WITHOUT JSON quotes
    //    (event_store.rs trim_matches('"'); live fact vs early F16 draft).
    checks.push(match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
        Some(conn) => check_recovery_kit_event(&conn),
        None if open_failed => HealthCheck::warn(
            "recovery_kit_event",
            if key_missing {
                "cannot query events (vault key missing)"
            } else {
                "cannot query events (vault open failed)"
            },
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
        None => HealthCheck::warn(
            "recovery_kit_event",
            "failed to lock vault connection for event query",
            Some("ai-brains recovery export --output <offline-path>".into()),
        ),
    });

    // 8. recovery_kit_file
    checks.push(match key_opt.as_ref() {
        Some(key) => check_recovery_kit_file(
            opts.kit_path.as_deref(),
            opts.passphrase_file.as_deref(),
            key,
        ),
        None => HealthCheck::skip("recovery_kit_file", skip_reason),
    });

    // 9. zero_key_escape (soft)
    checks.push(match key_opt.as_ref() {
        Some(key) => check_zero_key_escape(key),
        None => HealthCheck::skip("zero_key_escape", skip_reason),
    });

    // 10. graph_feature (soft info — compile-time; never alone fail/degraded; T222)
    checks.push(check_graph_feature());

    // 11. graph_density (soft — never alone forces fail; SQL-only, capture-independent)
    checks.push(check_graph_density(
        vault_conn.as_ref(),
        open_failed,
        skip_reason,
    ));

    // 12. harness_wiring (soft info — never fail/degraded solely for missing hooks; T235 F17)
    checks.push(check_harness_wiring());

    // 13. project_identity (soft — env vs path alias; never alone forces fail; T240 F12)
    checks.push(check_project_identity(
        vault_conn.as_ref(),
        open_failed,
        skip_reason,
    ));

    // 14. policy_grants (soft — discovery Read* probe; warn incomplete; never alone Fail; T241)
    checks.push(check_policy_grants(
        vault_conn.as_ref(),
        open_failed,
        skip_reason,
    ));

    // 15. integrity (optional --full)
    checks.push(if opts.full {
        match vault_conn.as_ref().and_then(|vc| vc.lock().ok()) {
            Some(conn) => check_integrity(&conn),
            None if open_failed => HealthCheck::skip("integrity", skip_reason),
            None => HealthCheck::fail("integrity", "failed to lock vault connection", None),
        }
    } else {
        HealthCheck::skip("integrity", "pass --full to run PRAGMA integrity_check")
    });

    let mut status = DoctorReport::roll_up(&checks);
    // F9: missing key must exit 1 (fail status) even when vault_open is only skip.
    if key_missing {
        status = DoctorStatus::Fail;
    }
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
    let backups = match service.list_backups(ListMode::Quiet) {
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

    // T225 F9 / T244 F4: usable = is_usable_class (Readable | PreT109; cores
    // required). Age newest usable only; ignore Incomplete / plain / key / corrupt
    // timestamps (even if more recent).
    let usable: Vec<_> = backups
        .iter()
        .filter(|b| is_usable_class(b.class))
        .collect();

    if usable.is_empty() {
        return HealthCheck::warn(
            "backup_recent",
            "no usable encrypted backup under current key",
            Some("ai-brains backup create".into()),
        );
    }

    // list_backups is sorted Reverse(timestamp); first usable with a timestamp wins.
    let newest = usable.iter().find_map(|b| b.timestamp);
    let Some(ts) = newest else {
        return HealthCheck::warn(
            "backup_recent",
            "usable backups present but timestamps unparseable",
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
                "newest usable backup within {max_age} (timestamp {})",
                ts.format("%Y-%m-%dT%H:%M:%S")
            ),
        )
    } else {
        HealthCheck::warn(
            "backup_recent",
            format!(
                "newest usable backup older than {max_age} (timestamp {})",
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

/// Soft discovery-grants probe (T241 F1/F1b/F31): when vault open + authoritative
/// project scope, count ReadEvidence/ReadConclusions/ReadDecisions. Incomplete
/// (`active_count < 3`) → **warn** + long SOOT; never alone forces **Fail**.
/// Skip when vault closed / open-failed / no authoritative scope / list error.
/// No AppContext — StorePorts from VaultConnection clone only.
fn check_policy_grants(
    vault_conn: Option<&VaultConnection>,
    open_failed: bool,
    skip_reason: &str,
) -> HealthCheck {
    if open_failed {
        return HealthCheck::skip("policy_grants", skip_reason);
    }
    let Some(vc) = vault_conn else {
        return HealthCheck::skip("policy_grants", "vault connection unavailable");
    };

    let ports = StorePorts::from_store(SqliteEventStore::new(vc.clone()));
    let scope_key = match resolve_scope_key_for_cli(None, &ports.identity_store()) {
        Ok(k) => k,
        Err(_) => {
            return HealthCheck::skip(
                "policy_grants",
                "no authoritative project scope resolved in current context",
            );
        }
    };

    let principal = resolve_principal(None);
    let grant_store = ports.grant_store();
    let grants = match grant_store.list_applied_grants(
        principal.id,
        &scope_key,
        Some(&DISCOVERY_CAP_LABELS),
    ) {
        Ok(g) => g,
        Err(_) => {
            return HealthCheck::skip("policy_grants", "could not list applied grants");
        }
    };

    let active_count = discovery_active_count(grants.iter().map(|g| g.capability.as_str()));
    if active_count < 3 {
        let message = if active_count == 0 {
            "discovery grants empty (0 of 3)".to_string()
        } else {
            format!("discovery grants incomplete ({active_count} of 3)")
        };
        return HealthCheck::warn(
            "policy_grants",
            message,
            Some(POLICY_BOOTSTRAP_SOOT_LONG.to_string()),
        );
    }

    HealthCheck::ok_msg("policy_grants", "discovery grants active (3 of 3)")
}

/// Soft project identity check (T240 F12): env PROJECT_ID ≠ path-alias owner of
/// cwd/toplevel when both present. Uses vault_conn read-only (no AppContext).
/// Warn severity may degrade overall status but never alone forces **fail**.
fn check_project_identity(
    vault_conn: Option<&VaultConnection>,
    open_failed: bool,
    skip_reason: &str,
) -> HealthCheck {
    if open_failed {
        return HealthCheck::skip("project_identity", skip_reason);
    }
    let Some(vc) = vault_conn else {
        return HealthCheck::skip("project_identity", "vault connection unavailable");
    };

    let env_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let Some(env_id) = env_id else {
        return HealthCheck::ok_msg(
            "project_identity",
            "no AI_BRAINS_PROJECT_ID in env; path/detect not compared",
        );
    };

    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => {
            return HealthCheck::skip("project_identity", format!("cannot resolve cwd: {e}"));
        }
    };

    let git = crate::commands::project::collect_git_identity(&cwd).unwrap_or_default();
    let path_owner = match crate::commands::project::resolve_path_alias_for_location(vc, &cwd, &git)
    {
        Ok(p) => p,
        Err(e) => {
            return HealthCheck::skip("project_identity", format!("path alias lookup failed: {e}"));
        }
    };

    let Some(path_id) = path_owner else {
        return HealthCheck::ok_msg(
            "project_identity",
            "no path alias for cwd/toplevel; env Scope not compared to path",
        );
    };

    if env_id == path_id {
        return HealthCheck::ok_msg(
            "project_identity",
            format!("env Scope matches path alias owner ({env_id})"),
        );
    }

    HealthCheck::warn(
        "project_identity",
        format!(
            "daily Scope env PROJECT_ID={env_id} differs from path alias owner={path_id}"
        ),
        Some("Run `ai-brains project whoami`; rebind .env PROJECT_ID if path owner is intended (no auto-switch).".into()),
    )
}

/// Soft harness wiring check (T235). Always Ok severity so missing hooks never
/// roll up to Degraded/Fail alone (AC9). Message carries info.
fn check_harness_wiring() -> HealthCheck {
    let home = crate::harness::resolve_home();
    let report = crate::harness::collect_status_report(home.as_deref());
    let present: Vec<&crate::harness::HarnessStatus> =
        report.harnesses.iter().filter(|h| h.present).collect();
    if present.is_empty() {
        return HealthCheck::ok_msg(
            "harness_wiring",
            "no coding harnesses detected on this machine",
        );
    }
    HealthCheck::ok_msg(
        "harness_wiring",
        doctor_harness_wiring_message(&report.harnesses),
    )
}

/// F6 / AC8 / AC9: ready vs pending doctor copy. Soft message only.
///
/// Uses literal `T253` (do not call `HarnessId::pending_track()`, which still says T239+).
pub(crate) fn doctor_harness_wiring_message(statuses: &[crate::harness::HarnessStatus]) -> String {
    let present: Vec<&crate::harness::HarnessStatus> =
        statuses.iter().filter(|h| h.present).collect();
    if present.is_empty() {
        return "no coding harnesses detected on this machine".to_string();
    }

    let ready_missing: Vec<&str> = present
        .iter()
        .filter(|h| h.install_ready && h.wiring != crate::harness::WiringStatus::Ok)
        .map(|h| h.id.as_str())
        .collect();
    let pending_present: Vec<&str> = present
        .iter()
        .filter(|h| !h.install_ready)
        .map(|h| h.id.as_str())
        .collect();
    let ok_ready = present
        .iter()
        .filter(|h| h.install_ready && h.wiring == crate::harness::WiringStatus::Ok)
        .count();
    let ready_present = present.iter().filter(|h| h.install_ready).count();

    if !ready_missing.is_empty() {
        let mut msg = format!(
            "{ok_ready}/{ready_present} ready wired, {} ready missing ({}); next: ai-brains harness install --harness all-ready --dry-run",
            ready_missing.len(),
            ready_missing.join(", ")
        );
        if !pending_present.is_empty() {
            msg.push_str(&format!(
                " {} backend pending (T253): {}",
                pending_present.len(),
                pending_present.join(", ")
            ));
        }
        return msg;
    }

    if ready_present == 0 {
        return format!(
            "{} backend pending (T253): {}",
            pending_present.len(),
            pending_present.join(", ")
        );
    }

    if !pending_present.is_empty() {
        return format!(
            "{ok_ready}/{ready_present} ready wired ({} pending backend support: {})",
            pending_present.len(),
            pending_present.join(", ")
        );
    }

    format!("{ok_ready}/{ready_present} ready wired; message-only capture")
}

/// Soft compile-time graph capability signal (T222 F9). Always Ok severity —
/// never alone fail/degraded. Message `available` | `unavailable`.
fn check_graph_feature() -> HealthCheck {
    if cfg!(feature = "graph") {
        HealthCheck::ok_msg("graph_feature", "available")
    } else {
        HealthCheck::new(
            "graph_feature",
            CheckSeverity::Ok,
            Some("unavailable".into()),
            Some(crate::commands::governed_common::GRAPH_REINSTALL_SOOT.into()),
        )
    }
}

/// Soft density check (T213): SQL counts only; never alone forces `fail`.
fn check_graph_density(
    vault_conn: Option<&VaultConnection>,
    open_failed: bool,
    skip_reason: &str,
) -> HealthCheck {
    if open_failed {
        return HealthCheck::skip("graph_density", skip_reason);
    }
    let Some(vc) = vault_conn else {
        return HealthCheck::skip("graph_density", "vault connection unavailable");
    };
    let conn = match vc.lock() {
        Ok(c) => c,
        Err(_) => {
            return HealthCheck::skip("graph_density", "failed to lock vault connection");
        }
    };

    let gather = match gather_density_snapshot(&conn) {
        Ok(g) => g,
        Err(e) => {
            return HealthCheck::warn(
                "graph_density",
                format!("graph count query failed: {e}"),
                Some(crate::graph_density::density_remediation(cfg!(feature = "graph")).into()),
            );
        }
    };

    match gather {
        GatherResult::TablesMissing => {
            HealthCheck::skip("graph_density", "tables absent (graph_node/graph_edge)")
        }
        GatherResult::PinnedCountFailed { .. } => HealthCheck::skip(
            "graph_density",
            "pinned memory count failed (cannot assess empty_lag without pins)",
        ),
        GatherResult::Ok(snap) => {
            let assessment = assess_graph_density(&snap);
            match assessment.verdict {
                DensityVerdict::Ok => HealthCheck::ok_msg("graph_density", assessment.message),
                DensityVerdict::Skip => HealthCheck::skip("graph_density", assessment.message),
                DensityVerdict::EmptyLag
                | DensityVerdict::OrphanNodes
                | DensityVerdict::Sparse
                | DensityVerdict::ProjectionLag => {
                    HealthCheck::warn("graph_density", assessment.message, assessment.remediation)
                }
            }
        }
    }
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

/// Compact human summary of the same `DoctorReport` (T249). Warn+fail only.
pub(crate) fn format_doctor_summary(report: &DoctorReport) -> String {
    let mut ok = 0usize;
    let mut warn = 0usize;
    let mut fail = 0usize;
    let mut skip = 0usize;
    for c in &report.checks {
        match c.severity {
            CheckSeverity::Ok => ok += 1,
            CheckSeverity::Warn => warn += 1,
            CheckSeverity::Fail => fail += 1,
            CheckSeverity::Skip => skip += 1,
        }
    }
    let mut lines = Vec::new();
    lines.push(format!(
        "doctor: status={}  vault={}  ok={} warn={} fail={} skip={}",
        status_label(report.status),
        report.vault_path,
        ok,
        warn,
        fail,
        skip
    ));
    if warn + fail == 0 {
        lines.push("No issues.".into());
    } else {
        lines.push("attention:".into());
        for c in &report.checks {
            if !matches!(c.severity, CheckSeverity::Warn | CheckSeverity::Fail) {
                continue;
            }
            let sev = severity_label(c.severity);
            let msg = c.message.as_deref().unwrap_or("");
            lines.push(format!("  [{sev}] {} — {msg}", c.name));
            if let Some(rem) = &c.remediation {
                lines.push(format!("         remediation: {rem}"));
            }
        }
    }
    format!("{}\n", lines.join("\n"))
}

fn emit_report(
    report: &DoctorReport,
    format: &str,
    force_json: bool,
    summary: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let use_json = force_json || format.eq_ignore_ascii_case("json");
    if use_json {
        let json = serde_json::to_string_pretty(report)?;
        println!("{json}");
        return Ok(());
    }
    if summary {
        print!("{}", format_doctor_summary(report));
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
        // Document expected fixed order for determinism (F16/F30; T213 graph_density;
        // T222 graph_feature before graph_density; T235 harness_wiring; T240 project_identity;
        // T241 policy_grants between project_identity and integrity).
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
            "graph_feature",
            "graph_density",
            "harness_wiring",
            "project_identity",
            "policy_grants",
            "integrity",
        ];
        assert_eq!(expected.len(), 15);
        assert_eq!(expected[9], "graph_feature");
        assert_eq!(expected[10], "graph_density");
        assert_eq!(expected[11], "harness_wiring");
        assert_eq!(expected[12], "project_identity");
        assert_eq!(expected[13], "policy_grants");
        assert_eq!(expected[14], "integrity");
        // Ensure HealthCheck helpers set ok flag correctly.
        assert!(HealthCheck::skip("integrity", "x").ok);
        assert_eq!(
            HealthCheck::skip("integrity", "x").severity,
            CheckSeverity::Skip
        );
    }

    fn fifteen_names() -> [&'static str; 15] {
        [
            "vault_exists",
            "vault_open",
            "schema_readable",
            "cipher_page",
            "daemon_reachable",
            "backup_recent",
            "recovery_kit_event",
            "recovery_kit_file",
            "zero_key_escape",
            "graph_feature",
            "graph_density",
            "harness_wiring",
            "project_identity",
            "policy_grants",
            "integrity",
        ]
    }

    fn attention_block(out: &str) -> &str {
        out.split_once("attention:")
            .map(|(_, rest)| rest)
            .unwrap_or("")
    }

    #[test]
    fn format_doctor_summary__degraded_15__header_counts_attention_warn_fail() {
        let checks = vec![
            HealthCheck::ok_msg("vault_exists", "present"),
            HealthCheck::ok_msg("vault_open", "read-only"),
            HealthCheck::ok_msg("schema_readable", "ok"),
            HealthCheck::ok_msg("cipher_page", "ok"),
            HealthCheck::ok_msg("daemon_reachable", "down"),
            HealthCheck::warn(
                "backup_recent",
                "no backups",
                Some("ai-brains backup create".into()),
            ),
            HealthCheck::warn(
                "recovery_kit_event",
                "no kit event",
                Some("export a recovery kit".into()),
            ),
            HealthCheck::skip("recovery_kit_file", "no --kit-path"),
            HealthCheck::ok_msg("zero_key_escape", "ok"),
            HealthCheck::ok_msg("graph_feature", "available"),
            HealthCheck::warn(
                "graph_density",
                "sparse",
                Some("ai-brains graph rebuild".into()),
            ),
            HealthCheck::ok_msg("harness_wiring", "ok"),
            HealthCheck::ok_msg("project_identity", "ok"),
            HealthCheck::fail(
                "policy_grants",
                "empty",
                Some("ai-brains policy bootstrap".into()),
            ),
            HealthCheck::skip("integrity", "not requested"),
        ];
        assert_eq!(
            checks.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            fifteen_names()
        );
        let report = DoctorReport {
            schema_version: 1,
            status: DoctorReport::roll_up(&checks),
            checks,
            vault_path: "C:\\vault.db".into(),
            generated_at: "2026-08-14T00:00:00Z".into(),
        };
        let out = format_doctor_summary(&report);
        assert!(
            out.starts_with("doctor: status=fail  vault=C:\\vault.db  ok=9 warn=3 fail=1 skip=2"),
            "header mismatch:\n{out}"
        );
        assert!(out.contains("ok="), "got:\n{out}");
        assert!(out.contains("warn="), "got:\n{out}");
        assert!(out.contains("fail="), "got:\n{out}");
        assert!(out.contains("skip="), "got:\n{out}");
        assert!(out.contains("attention:"), "got:\n{out}");
        let attention = attention_block(&out);
        assert!(attention.contains("backup_recent"), "got:\n{out}");
        assert!(attention.contains("recovery_kit_event"), "got:\n{out}");
        assert!(attention.contains("graph_density"), "got:\n{out}");
        assert!(attention.contains("policy_grants"), "got:\n{out}");
        assert!(
            !attention.contains("recovery_kit_file"),
            "skip listed in attention:\n{out}"
        );
        assert!(
            !attention.contains("integrity"),
            "skip listed in attention:\n{out}"
        );
        assert!(
            !attention.contains("vault_exists"),
            "ok listed in attention:\n{out}"
        );
        assert!(
            out.contains("         remediation: ai-brains backup create"),
            "remediation indent missing:\n{out}"
        );
        assert!(!out.contains("No issues."), "got:\n{out}");
    }

    #[test]
    fn format_doctor_summary__all_ok__no_issues() {
        let checks: Vec<HealthCheck> = fifteen_names()
            .into_iter()
            .map(|n| HealthCheck::ok_msg(n, "ok"))
            .collect();
        let report = DoctorReport {
            schema_version: 1,
            status: DoctorStatus::Ok,
            checks,
            vault_path: "C:\\vault.db".into(),
            generated_at: "2026-08-14T00:00:00Z".into(),
        };
        let out = format_doctor_summary(&report);
        assert!(
            out.contains("doctor: status=ok  vault=C:\\vault.db  ok=15 warn=0 fail=0 skip=0"),
            "header mismatch:\n{out}"
        );
        assert!(out.contains("No issues."), "got:\n{out}");
        assert!(!out.contains("attention:"), "got:\n{out}");
    }

    /// T241 AC2: no authoritative scope → policy_grants skip.
    #[test]
    fn doctor__policy_grants__no_authoritative_scope__skip() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let _clear_proj = TempEnv::remove("AI_BRAINS_PROJECT_ID");
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        {
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
        }
        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report");
        let pg = report
            .checks
            .iter()
            .find(|c| c.name == "policy_grants")
            .expect("policy_grants");
        assert_eq!(pg.severity, CheckSeverity::Skip);
        let msg = pg.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("no authoritative project scope") || msg.contains("authoritative"),
            "skip message must note scope; got {msg}"
        );
        // Warn alone must not force Fail — skip is fine.
        assert_ne!(report.status, DoctorStatus::Fail);
    }

    /// T241 AC1: authoritative + empty discovery grants → policy_grants warn + long SOOT.
    #[test]
    fn doctor__policy_grants__authoritative_empty__warn_bootstrap() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let _proj = TempEnv::set(
            "AI_BRAINS_PROJECT_ID",
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        );
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        {
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
        }
        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report");
        let pg = report
            .checks
            .iter()
            .find(|c| c.name == "policy_grants")
            .expect("policy_grants");
        assert_eq!(pg.severity, CheckSeverity::Warn, "msg={:?}", pg.message);
        let msg = pg.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("empty") || msg.contains("0 of 3"),
            "empty grants message; got {msg}"
        );
        let rem = pg.remediation.as_deref().unwrap_or("");
        assert!(
            rem.contains("policy bootstrap"),
            "rem must contain policy bootstrap; got {rem}"
        );
        assert!(
            rem.contains("omit --scope") || rem.contains("authoritative"),
            "long SOOT expected; got {rem}"
        );
        // Warn → Degraded, never Fail alone.
        assert_ne!(report.status, DoctorStatus::Fail);
        assert_eq!(report.status, DoctorStatus::Degraded);
    }

    /// T241 AC1/F31: partial discovery (1 of 3) still warns incomplete.
    #[test]
    fn doctor__policy_grants__partial_one_of_three__warn_incomplete() {
        use ai_brains_control_plane::{StorePorts, SystemClock, issue_grant, register_principal};
        use ai_brains_core::ids::ProjectId;
        use ai_brains_core::privacy::Privacy;
        use ai_brains_core::scope::{GrantCapability, ScopeRef};
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;
        use uuid::Uuid;

        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let project = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let _proj = TempEnv::set("AI_BRAINS_PROJECT_ID", project);
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        {
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
            let ports = StorePorts::from_store(SqliteEventStore::new(conn));
            let clock = SystemClock;
            // Doctor uses resolve_principal(None) → default System principal.
            let principal = resolve_principal(None);
            let _ = register_principal(&ports.writer, &clock, &principal);
            let scope = ScopeRef::Repository(ProjectId::from_uuid(
                Uuid::parse_str(project).expect("uuid"),
            ));
            issue_grant(
                &ports.writer,
                &clock,
                principal.id,
                scope,
                GrantCapability::ReadEvidence,
                Privacy::LocalOnly,
            )
            .expect("issue one discovery grant");
        }
        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report");
        let pg = report
            .checks
            .iter()
            .find(|c| c.name == "policy_grants")
            .expect("policy_grants");
        assert_eq!(pg.severity, CheckSeverity::Warn, "msg={:?}", pg.message);
        let msg = pg.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("incomplete") && msg.contains("1 of 3"),
            "partial message; got {msg}"
        );
        assert!(
            pg.remediation
                .as_deref()
                .unwrap_or("")
                .contains("policy bootstrap"),
            "rem must name bootstrap"
        );
    }

    /// T213 AC10/AC11: graph_density present; open-failed path is skip (not fail).
    #[test]
    fn doctor__graph_density_present__open_failed_is_skip() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _clear = TempEnv::remove("AI_BRAINS_KEY");
        let _clear_allow = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let dir = tempdir().expect("tempdir");
        // Missing vault → open fails; graph_density must skip.
        let vault = dir.path().join("missing-vault.db");
        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let report = build_report(&opts, false).expect("report");
        assert_eq!(report.checks.len(), 15, "15-check matrix");
        let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "vault_exists",
                "vault_open",
                "schema_readable",
                "cipher_page",
                "daemon_reachable",
                "backup_recent",
                "recovery_kit_event",
                "recovery_kit_file",
                "zero_key_escape",
                "graph_feature",
                "graph_density",
                "harness_wiring",
                "project_identity",
                "policy_grants",
                "integrity",
            ]
        );
        let density = report
            .checks
            .iter()
            .find(|c| c.name == "graph_density")
            .expect("graph_density");
        assert_eq!(density.severity, CheckSeverity::Skip);
        assert_ne!(density.severity, CheckSeverity::Fail);
    }

    /// T222 AC4–AC6: graph_feature message available|unavailable via compile-time cfg.
    #[test]
    fn doctor__graph_feature__message_available_or_unavailable() {
        let check = check_graph_feature();
        assert_eq!(check.name, "graph_feature");
        assert_eq!(check.severity, CheckSeverity::Ok);
        assert!(check.ok);
        if cfg!(feature = "graph") {
            assert_eq!(check.message.as_deref(), Some("available"));
            assert!(check.remediation.is_none());
        } else {
            assert_eq!(check.message.as_deref(), Some("unavailable"));
            assert_eq!(
                check.remediation.as_deref(),
                Some(crate::commands::governed_common::GRAPH_REINSTALL_SOOT)
            );
        }
    }

    /// T213: migrated vault with empty small graph → graph_density skip or ok (not fail).
    #[test]
    fn doctor__graph_density_on_migrated_vault__not_fail() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
        {
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
        }
        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ZERO_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report");
        let density = report
            .checks
            .iter()
            .find(|c| c.name == "graph_density")
            .expect("graph_density");
        assert!(
            matches!(
                density.severity,
                CheckSeverity::Ok | CheckSeverity::Skip | CheckSeverity::Warn
            ),
            "density must not hard-fail alone; got {:?}",
            density.severity
        );
        // Message must not look like secrets.
        let msg = density.message.as_deref().unwrap_or("");
        assert!(!msg.contains("x'"), "no key material: {msg}");
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
            summary: false,
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
    const ALT_KEY_LITERAL: &str =
        "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'";

    /// F9/AC11: missing key → vault_open skipped; overall Fail; report emits.
    #[test]
    fn doctor__missing_key__vault_open_skipped_status_fail() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _clear = TempEnv::remove("AI_BRAINS_KEY");
        let _clear_allow = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        // Create vault with zero key (fixture), then doctor without key.
        {
            let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
            let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
        }

        let opts = DoctorOptions {
            vault_path: vault,
            key: None,
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report must emit");
        assert_eq!(report.status, DoctorStatus::Fail);
        let open = report
            .checks
            .iter()
            .find(|c| c.name == "vault_open")
            .expect("vault_open");
        assert_eq!(open.severity, CheckSeverity::Skip);
        let msg = open.message.as_deref().unwrap_or("");
        assert!(
            msg.contains("key missing") || msg.contains("AI_BRAINS_KEY"),
            "expected missing-key skip message, got {msg}"
        );
        assert_eq!(exit_code_for(&report, false), 1);
    }

    /// F9/AC11: wrong key → vault_open fail with Vault locked hint.
    #[test]
    fn doctor__wrong_key__vault_open_fail() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        ai_brains_store::sqlcipher_log_policy::install();
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        {
            let key = SqlCipherKey::from_raw(ZERO_KEY_LITERAL.to_string());
            let conn = VaultConnection::open(&vault, &key).expect("open");
            conn.migrate().expect("migrate");
        }

        let opts = DoctorOptions {
            vault_path: vault,
            key: Some(ALT_KEY_LITERAL.to_string()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let report = build_report(&opts, false).expect("report");
        assert_eq!(report.status, DoctorStatus::Fail);
        let open = report
            .checks
            .iter()
            .find(|c| c.name == "vault_open")
            .expect("vault_open");
        assert_eq!(open.severity, CheckSeverity::Fail);
        let msg = open.message.as_deref().unwrap_or("");
        assert!(
            msg.starts_with("Vault locked:") || msg.contains("Vault locked"),
            "expected Vault locked prefix, got {msg}"
        );
    }

    /// Format error fails early (F8) before report.
    #[test]
    fn doctor__invalid_format__early_error() {
        use ai_brains_core::temp_env::TempEnv;
        use tempfile::tempdir;

        let _clear = TempEnv::remove("AI_BRAINS_KEY");
        let dir = tempdir().expect("tempdir");
        let opts = DoctorOptions {
            vault_path: dir.path().join("vault.db"),
            key: Some("not-a-key".into()),
            format: "json".into(),
            json: true,
            fail_on_degraded: false,
            kit_path: None,
            passphrase_file: None,
            backup_max_age: "7d".into(),
            full: false,
            summary: false,
        };
        let err = build_report(&opts, false).expect_err("format");
        let msg = err.to_string();
        assert!(msg.starts_with("Vault key invalid format:"), "got {msg}");
    }

    fn dummy_harness_status(
        id: &str,
        present: bool,
        install_ready: bool,
        wiring: crate::harness::WiringStatus,
    ) -> crate::harness::HarnessStatus {
        crate::harness::HarnessStatus {
            id: id.to_string(),
            display_name: id.to_string(),
            present,
            binary: None,
            home_path: None,
            wiring,
            install_ready,
            targets: vec!["dummy".into()],
            next_action: "dummy".into(),
        }
    }

    #[test]
    fn doctor_harness_wiring_message__separates_ready_from_pending() {
        use crate::harness::WiringStatus;
        let statuses = vec![
            dummy_harness_status("grok", true, true, WiringStatus::Missing),
            dummy_harness_status("agy", true, true, WiringStatus::Missing),
            dummy_harness_status("opencode", true, true, WiringStatus::Partial),
            dummy_harness_status("claude", true, false, WiringStatus::Missing),
            dummy_harness_status("codex", true, false, WiringStatus::Unknown),
        ];
        let msg = doctor_harness_wiring_message(&statuses);
        assert!(
            msg.contains("0/3 ready wired, 3 ready missing (grok, agy, opencode)"),
            "ready-missing listed separately; got {msg}"
        );
        assert!(
            msg.contains("ai-brains harness install --harness all-ready --dry-run"),
            "next SOOT must be exact all-ready dry-run; got {msg}"
        );
        assert!(
            msg.contains("2 backend pending (T253): claude, codex"),
            "pending must be T253, not lumped as installable; got {msg}"
        );
        assert!(
            !msg.contains("5 missing"),
            "must not treat Claude/Codex as installable missing; got {msg}"
        );
    }

    #[test]
    fn doctor_harness_wiring_message__ready_ok_pending__not_missing_wiring() {
        use crate::harness::WiringStatus;
        let statuses = vec![
            dummy_harness_status("grok", true, true, WiringStatus::Ok),
            dummy_harness_status("agy", true, true, WiringStatus::Ok),
            dummy_harness_status("opencode", true, true, WiringStatus::Ok),
            dummy_harness_status("claude", true, false, WiringStatus::Missing),
            dummy_harness_status("codex", true, false, WiringStatus::Unknown),
        ];
        let msg = doctor_harness_wiring_message(&statuses);
        assert!(
            msg.contains("3/3 ready wired"),
            "ready backends are wired; got {msg}"
        );
        assert!(
            msg.contains("2 pending backend support: claude, codex"),
            "F6 all-ok+pending branch (no T253 token on this arm); got {msg}"
        );
        assert!(
            !msg.contains("missing wiring")
                && !msg.contains("ready missing")
                && !msg.contains("5 missing"),
            "must not say missing wiring for grok/agy/opencode; got {msg}"
        );
    }
}
