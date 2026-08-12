#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T192 doctor CLI hermetic integration tests (AC1–AC16 subset).

mod common;

use ai_brains_contracts::doctor::{DoctorReport, DoctorStatus};
use ai_brains_crypto::test_support::{assert_no_kit_dump, assert_no_secret_leakage};
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
const ALT_KEY: &str = "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'";
const ZERO_KEY_BYTES: [u8; 32] = [0u8; 32];
const ALT_KEY_BYTES: [u8; 32] = [0xffu8; 32];

fn hermetic_with_key(vault_path: &Path, key: &str) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .env("AI_BRAINS_KEY", key)
        .arg("--key")
        .arg(key);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic_with_key(vault_path, ZERO_KEY)
        .arg("init")
        .assert()
        .success();
}

fn combined_output(output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

fn export_kit(vault: &Path, kit_path: &Path, pw_path: &Path, passphrase: &[u8]) {
    fs::write(pw_path, passphrase).expect("write passphrase");
    let out = hermetic_with_key(vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(kit_path)
        .arg("--passphrase-file")
        .arg(pw_path)
        .output()
        .expect("recovery export");
    assert!(
        out.status.success(),
        "export must succeed; out={}",
        combined_output(&out)
    );
    assert!(kit_path.exists(), "kit file must exist");
}

/// AC8 process-level: doctor never migrates (AppContext path not used).
/// Asserts read-only open language, no backups/ dir creation, and success
/// while ambient daemon is down (injectable daemon-up covered in unit test).
/// Note: SQLCipher may materialize -wal/-shm sidecars when opening a WAL-mode
/// vault even under SQLITE_OPEN_READ_ONLY — that is not migrate/AppContext.
#[test]
fn doctor__no_migrate_while_daemon_up__process_read_only() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backups = dir.path().join("backups");
    assert!(!backups.exists(), "precondition: no backups/");

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "doctor must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("json");
    let open = report
        .checks
        .iter()
        .find(|c| c.name == "vault_open")
        .expect("vault_open");
    assert_eq!(
        open.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok
    );
    assert!(
        open.message.as_deref().unwrap_or("").contains("read-only"),
        "vault_open must claim read-only path: {:?}",
        open.message
    );
    assert!(
        !backups.exists(),
        "doctor must not create backups/ (F17b / F33)"
    );
    // Must not create a second vault or migrate-only sidecar named oddly.
    assert!(
        !dir.path().join("vault.db.bak").exists(),
        "doctor must not write backup of vault"
    );
}

/// AC2: happy path temp vault → ok or degraded, exit 0.
#[test]
fn doctor__happy_temp_vault__ok_or_degraded_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "happy doctor must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    assert_eq!(report.schema_version, 1);
    assert!(
        matches!(report.status, DoctorStatus::Ok | DoctorStatus::Degraded),
        "expected ok|degraded, got {:?}",
        report.status
    );
    // No kit path → recovery_kit_file skip
    let kit_file = report
        .checks
        .iter()
        .find(|c| c.name == "recovery_kit_file")
        .expect("recovery_kit_file check");
    assert_eq!(
        kit_file.severity,
        ai_brains_contracts::doctor::CheckSeverity::Skip
    );
}

/// AC3: wrong key → vault_open fail → overall fail → exit 1.
#[test]
fn doctor__wrong_key__fail_exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ALT_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor wrong key");
    assert_eq!(
        out.status.code(),
        Some(1),
        "wrong key must exit 1; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    assert_eq!(report.status, DoctorStatus::Fail);
    let open = report
        .checks
        .iter()
        .find(|c| c.name == "vault_open")
        .expect("vault_open");
    assert_eq!(
        open.severity,
        ai_brains_contracts::doctor::CheckSeverity::Fail
    );
    assert_no_secret_leakage(&combined_output(&out), &ALT_KEY_BYTES);
    assert_no_secret_leakage(&combined_output(&out), &ZERO_KEY_BYTES);
}

/// AC4: missing vault → fail exit 1; no create.
#[test]
fn doctor__missing_vault__fail_no_create() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("missing-vault.db");
    assert!(!vault.exists());

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor missing");
    assert_eq!(
        out.status.code(),
        Some(1),
        "missing vault must exit 1; out={}",
        combined_output(&out)
    );
    assert!(!vault.exists(), "doctor must not create a missing vault");
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    assert_eq!(report.status, DoctorStatus::Fail);
    let exists = report
        .checks
        .iter()
        .find(|c| c.name == "vault_exists")
        .expect("vault_exists");
    assert_eq!(
        exists.severity,
        ai_brains_contracts::doctor::CheckSeverity::Fail
    );
}

/// AC5: daemon down produces daemon_reachable without forcing fail.
#[test]
fn doctor__daemon_up_or_down__not_fail() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Hermetic: no daemon → probe returns false → message "down", severity ok.
    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "daemon-down doctor must not fail overall from daemon alone; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let daemon = report
        .checks
        .iter()
        .find(|c| c.name == "daemon_reachable")
        .expect("daemon_reachable");
    assert_eq!(
        daemon.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok
    );
    let msg = daemon.message.as_deref().unwrap_or("");
    assert!(
        msg == "up" || msg == "down",
        "daemon message must be up|down, got {msg}"
    );
    assert_ne!(report.status, DoctorStatus::Fail); // soft warns only possible
}

/// AC6 good kit.
#[test]
fn doctor__kit_path_good__recovery_kit_file_ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    let passphrase = b"doctor-kit-passphrase";
    export_kit(&vault, &kit_path, &pw_path, passphrase);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--kit-path")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("doctor kit");
    assert!(
        out.status.success(),
        "good kit must not force fail; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let kit = report
        .checks
        .iter()
        .find(|c| c.name == "recovery_kit_file")
        .expect("recovery_kit_file");
    assert_eq!(
        kit.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok,
        "msg={:?}",
        kit.message
    );
}

/// AC6 bad passphrase.
#[test]
fn doctor__kit_path_bad_pass__fail() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let good_pw = dir.path().join("good.txt");
    let bad_pw = dir.path().join("bad.txt");
    export_kit(&vault, &kit_path, &good_pw, b"correct-passphrase!!");
    fs::write(&bad_pw, b"wrong-passphrase!!!!").unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--kit-path")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&bad_pw)
        .output()
        .expect("doctor bad pass");
    assert_eq!(
        out.status.code(),
        Some(1),
        "bad pass must exit 1; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    assert_eq!(report.status, DoctorStatus::Fail);
    let kit = report
        .checks
        .iter()
        .find(|c| c.name == "recovery_kit_file")
        .expect("recovery_kit_file");
    assert_eq!(
        kit.severity,
        ai_brains_contracts::doctor::CheckSeverity::Fail
    );
}

/// AC6 reparse refuse (Windows junction when feasible).
#[test]
fn doctor__kit_path_reparse__refused() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let real_kit = dir.path().join("real-kit.json");
    let pw_path = dir.path().join("pw.txt");
    export_kit(&vault, &real_kit, &pw_path, b"reparse-passphrase!!");

    #[cfg(windows)]
    {
        let link = dir.path().join("kit-link.json");
        // Prefer symlink; fall back to skip if privilege missing.
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                &link.to_string_lossy(),
                &real_kit.to_string_lossy(),
            ])
            .status();
        match status {
            Ok(s) if s.success() && link.exists() => {
                let out = hermetic_with_key(&vault, ZERO_KEY)
                    .arg("doctor")
                    .arg("--json")
                    .arg("--kit-path")
                    .arg(&link)
                    .arg("--passphrase-file")
                    .arg(&pw_path)
                    .output()
                    .expect("doctor reparse");
                assert_eq!(
                    out.status.code(),
                    Some(1),
                    "reparse kit must fail; out={}",
                    combined_output(&out)
                );
                let report: DoctorReport =
                    serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
                let kit = report
                    .checks
                    .iter()
                    .find(|c| c.name == "recovery_kit_file")
                    .expect("recovery_kit_file");
                assert_eq!(
                    kit.severity,
                    ai_brains_contracts::doctor::CheckSeverity::Fail
                );
            }
            _ => {
                eprintln!(
                    "skip doctor__kit_path_reparse__refused: mklink not available \
                     (need admin or Developer Mode)"
                );
            }
        }
    }

    #[cfg(unix)]
    {
        let link = dir.path().join("kit-link.json");
        std::os::unix::fs::symlink(&real_kit, &link).expect("symlink");
        let out = hermetic_with_key(&vault, ZERO_KEY)
            .arg("doctor")
            .arg("--json")
            .arg("--kit-path")
            .arg(&link)
            .arg("--passphrase-file")
            .arg(&pw_path)
            .output()
            .expect("doctor reparse");
        assert_eq!(
            out.status.code(),
            Some(1),
            "reparse kit must fail; out={}",
            combined_output(&out)
        );
        let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
        let kit = report
            .checks
            .iter()
            .find(|c| c.name == "recovery_kit_file")
            .expect("recovery_kit_file");
        assert_eq!(
            kit.severity,
            ai_brains_contracts::doctor::CheckSeverity::Fail
        );
    }
}

/// No kit path → skip.
#[test]
fn doctor__no_kit_path__recovery_kit_file_skip() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(out.status.success());
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let kit = report
        .checks
        .iter()
        .find(|c| c.name == "recovery_kit_file")
        .expect("recovery_kit_file");
    assert_eq!(
        kit.severity,
        ai_brains_contracts::doctor::CheckSeverity::Skip
    );
    let msg = kit.message.as_deref().unwrap_or("");
    assert!(
        msg.contains("--kit-path"),
        "skip message should mention --kit-path: {msg}"
    );
}

/// AC9: JSON schema_version=1 deserializes.
#[test]
fn doctor__json__schema_v1_deserializes() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(out.status.success());
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    assert_eq!(report.schema_version, 1);
    assert!(!report.checks.is_empty());
    assert!(!report.vault_path.is_empty());
    assert!(!report.generated_at.is_empty());
}

/// AC9: default format human without --json.
#[test]
fn doctor__default_format_human__without_json_flag() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .output()
        .expect("doctor human");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("doctor: status="),
        "human summary missing: {stdout}"
    );
    assert!(
        stdout.contains("[ok]") || stdout.contains("[warn]") || stdout.contains("[skip]"),
        "per-check lines missing: {stdout}"
    );
    // Not a raw JSON object
    assert!(
        !stdout.trim_start().starts_with('{'),
        "default must not be JSON: {stdout}"
    );
}

/// AC10: --fail-on-degraded turns degraded into exit 1.
#[test]
fn doctor__fail_on_degraded__exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Fresh vault: zero key + ALLOW_ZERO_KEY + no backups + no kit event → degraded.
    let baseline = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("baseline");
    assert!(baseline.status.success());
    let report: DoctorReport = serde_json::from_slice(&baseline.stdout).expect("DoctorReport JSON");
    assert_eq!(
        report.status,
        DoctorStatus::Degraded,
        "expected degraded for zero-key hermetic vault without backups; got {:?}",
        report.status
    );

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--fail-on-degraded")
        .output()
        .expect("fail-on-degraded");
    assert_eq!(
        out.status.code(),
        Some(1),
        "fail-on-degraded must exit 1; out={}",
        combined_output(&out)
    );
}

/// AC7: no secrets in stdout/stderr.
#[test]
fn doctor__stdout__no_secrets() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    let passphrase = b"secret-doctor-pass!!";
    export_kit(&vault, &kit_path, &pw_path, passphrase);
    let kit_json = fs::read_to_string(&kit_path).unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--kit-path")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("doctor");
    let combined = combined_output(&out);
    assert_no_secret_leakage(&combined, &ZERO_KEY_BYTES);
    assert_no_secret_leakage(&combined, passphrase);
    assert_no_kit_dump(&combined, &kit_json);
    assert!(!combined.contains(&kit_json));
    assert!(
        !combined.contains("\"ciphertext\""),
        "must not dump kit ciphertext fields"
    );
}

/// AC15: doctor without backups/ does not create that directory.
#[test]
fn doctor__no_backups_dir__does_not_create() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let backups = dir.path().join("backups");
    assert!(!backups.exists(), "precondition: no backups/ after init");

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(out.status.success(), "out={}", combined_output(&out));
    assert!(
        !backups.exists(),
        "doctor must not create backups/ (AC15 / F17b / F33)"
    );

    // Also list parent for any new dirs under vault parent.
    let entries: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    for e in &entries {
        assert!(
            e.file_name().and_then(|n| n.to_str()) != Some("backups"),
            "unexpected backups dir: {}",
            e.display()
        );
    }
}

/// AC16: after recovery export, recovery_kit_event is ok (not false warn).
#[test]
fn doctor__recovery_kit_created_event__ok_not_false_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    export_kit(&vault, &kit_path, &pw_path, b"event-check-passphrase");

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let evt = report
        .checks
        .iter()
        .find(|c| c.name == "recovery_kit_event")
        .expect("recovery_kit_event");
    assert_eq!(
        evt.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok,
        "AC16: RecoveryKitCreated must be found with live event_type storage; msg={:?}",
        evt.message
    );
}

/// AC1: doctor appears in --help.
#[test]
fn doctor__help__lists_command() {
    common::hermetic_bin()
        .arg("doctor")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Read-only"))
        .stdout(predicate::str::contains("kit-path"))
        .stdout(predicate::str::contains("fail-on-degraded"));
}

fn backup_recent_check(report: &DoctorReport) -> &ai_brains_contracts::doctor::HealthCheck {
    report
        .checks
        .iter()
        .find(|c| c.name == "backup_recent")
        .expect("backup_recent check present")
}

/// T225 AC7/M3: all LegacyPlain backups → backup_recent warn + create remediation
/// even when filenames have recent timestamps.
#[test]
fn doctor__backup_recent__all_legacy_plain__warn_create() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let backups = dir.path().join("backups");
    fs::create_dir_all(&backups).unwrap();
    // Recent-looking plain SQLite magic files (LegacyPlain under classify).
    for name in [
        "vault-2026-08-01T12-00-00.db.bak",
        "vault-2026-08-02T12-00-00.db.bak",
    ] {
        let mut bytes = b"SQLite format 3\0".to_vec();
        bytes.resize(64, 0);
        fs::write(backups.join(name), &bytes).unwrap();
    }

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "doctor soft warn must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Warn,
        "all-plain must warn; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("no usable") || msg.contains("usable encrypted"),
        "message must indicate no usable encrypted backup; got {:?}",
        br.message
    );
    let rem = br.remediation.as_deref().unwrap_or("");
    assert!(
        rem.contains("ai-brains backup create"),
        "remediation must cite create; got {rem}"
    );
}

/// T225 AC8/M3: Readable backup within age → backup_recent ok.
#[test]
fn doctor__backup_recent__readable_within_age__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok,
        "fresh readable must be ok; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("usable") || msg.contains("within"),
        "ok message should mention usable/within; got {:?}",
        br.message
    );
}

/// T225 AC8/M3: Readable backup with stale filename timestamp → warn + create.
#[test]
fn doctor__backup_recent__readable_stale__warn_create() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    let backups = dir.path().join("backups");
    let entries: Vec<PathBuf> = fs::read_dir(&backups)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    assert_eq!(entries.len(), 1, "expected one backup file");
    let stale_name = backups.join("vault-2020-01-01T00-00-00.db.bak");
    fs::rename(&entries[0], &stale_name).expect("rename backup to stale timestamp name");

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "soft stale warn exits 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Warn,
        "stale usable must warn; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("older") || msg.contains("stale"),
        "stale message expected; got {:?}",
        br.message
    );
    let rem = br.remediation.as_deref().unwrap_or("");
    assert!(
        rem.contains("ai-brains backup create"),
        "remediation must cite create; got {rem}"
    );
}

/// T225 AC8/M3: fresh Readable + plain residuals → ok.
///
/// Proves doctor does **not** warn when the newest *usable* backup is within
/// age. Plain residuals (older or fresher-named) must not force a warn when a
/// recent usable backup exists. This case is **not** discriminating against
/// age-only doctors that pick the newest filename overall — see
/// `doctor__backup_recent__stale_usable_plus_fresher_plain__warns`.
#[test]
fn doctor__backup_recent__readable_recent_plus_older_plain__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    // Older-named plain residual must not poison usable age.
    let backups = dir.path().join("backups");
    let mut plain = b"SQLite format 3\0".to_vec();
    plain.resize(64, 0);
    fs::write(backups.join("vault-2019-01-01T00-00-00.db.bak"), &plain).unwrap();
    // Fresher-named plain must also not become the age source when usable is fresh.
    fs::write(backups.join("vault-2099-01-01T00-00-00.db.bak"), &plain).unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok,
        "must age newest usable (readable), not freshest plain; msg={:?}",
        br.message
    );
}

/// T225 P2-1: stale usable + fresher plain residual → warn (ages usable only).
///
/// Discriminating mixed-age case: an age-only doctor that picks the newest
/// filename overall would treat the fresher plain residual as within age and
/// return Ok. Usable-only aging must warn on the stale Readable/PreT109.
#[test]
fn doctor__backup_recent__stale_usable_plus_fresher_plain__warns() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    let backups = dir.path().join("backups");
    let entries: Vec<PathBuf> = fs::read_dir(&backups)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    assert_eq!(entries.len(), 1, "expected one backup file");
    // Stale usable Readable (key opens; meta present).
    let stale_name = backups.join("vault-2020-01-01T00-00-00.db.bak");
    fs::rename(&entries[0], &stale_name).expect("rename backup to stale timestamp name");

    // Fresher plain residual — newest overall timestamp and *within age*
    // (future date keeps it forever in-window under --backup-max-age 7d so this
    // test stays discriminating on any calendar day). Age-only doctor would Ok
    // on this plain; usable-only must still Warn on the stale Readable.
    let mut plain = b"SQLite format 3\0".to_vec();
    plain.resize(64, 0);
    fs::write(backups.join("vault-2099-12-31T23-59-59.db.bak"), &plain).unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "soft warn must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Warn,
        "must age stale usable, not fresher plain; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("older") || msg.contains("stale"),
        "stale-usable message expected; got {:?}",
        br.message
    );
    let rem = br.remediation.as_deref().unwrap_or("");
    assert!(
        rem.contains("ai-brains backup create"),
        "remediation must cite create; got {rem}"
    );
}

/// T244 AC3: Incomplete-only fleet → no usable + create (not stale-on-Incomplete).
#[test]
fn doctor__backup_recent__all_incomplete__warn_no_usable() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let backups = dir.path().join("backups");
    fs::create_dir_all(&backups).unwrap();
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    for name in [
        "vault-2026-08-01T12-00-00.db.bak",
        "vault-2026-08-02T12-00-00.db.bak",
    ] {
        let path = backups.join(name);
        let conn = rusqlite::Connection::open(&path).expect("open incomplete");
        ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).expect("key");
        conn.execute_batch("CREATE TABLE junk(x);").expect("junk");
    }

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "soft warn must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Warn,
        "all-Incomplete must warn; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("no usable") || msg.contains("usable encrypted"),
        "must be no-usable (not stale-on-Incomplete); got {:?}",
        br.message
    );
    assert!(
        !msg.contains("older"),
        "must not age Incomplete as usable; got {:?}",
        br.message
    );
    let rem = br.remediation.as_deref().unwrap_or("");
    assert!(
        rem.contains("ai-brains backup create"),
        "create-only remediation; got {rem}"
    );
    assert!(
        !rem.contains("verify"),
        "remediation must stay create-only; got {rem}"
    );
}

/// T244 AC5: stale usable + fresher Incomplete → ages usable only / still warn stale.
#[test]
fn doctor__backup_recent__stale_usable_plus_fresher_incomplete__warns() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    let backups = dir.path().join("backups");
    let entries: Vec<PathBuf> = fs::read_dir(&backups)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    assert_eq!(entries.len(), 1, "expected one backup file");
    let stale_name = backups.join("vault-2020-01-01T00-00-00.db.bak");
    fs::rename(&entries[0], &stale_name).expect("rename backup to stale timestamp name");

    // Fresher Incomplete residual — must not become the age source.
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let incomplete = backups.join("vault-2099-12-31T23-59-59.db.bak");
    let conn = rusqlite::Connection::open(&incomplete).expect("open incomplete");
    ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).expect("key");
    conn.execute_batch("CREATE TABLE junk(x);").expect("junk");
    drop(conn);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(
        out.status.success(),
        "soft warn must exit 0; out={}",
        combined_output(&out)
    );
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Warn,
        "must age stale usable, not fresher Incomplete; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("older") || msg.contains("stale"),
        "stale-usable message expected; got {:?}",
        br.message
    );
}

/// T225 P2-1: PreT109 (key opens, no `_aibrains_backup_meta`) within age → ok.
///
/// Usable includes PreT109; doctor must not require meta rows for backup_recent.
#[test]
fn doctor__backup_recent__pret109_within_age__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let create = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .output()
        .expect("backup create");
    assert!(
        create.status.success(),
        "backup create failed: {}",
        combined_output(&create)
    );

    let backups = dir.path().join("backups");
    let entries: Vec<PathBuf> = fs::read_dir(&backups)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    assert_eq!(entries.len(), 1, "expected one backup file");
    let bak = &entries[0];

    // Strip meta table → PreT109; keep recent filename so age is within 7d.
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = rusqlite::Connection::open(bak).expect("open bak");
    ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).expect("apply key");
    conn.execute_batch("DROP TABLE IF EXISTS _aibrains_backup_meta;")
        .expect("drop backup meta for PreT109");
    drop(conn);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport JSON");
    let br = backup_recent_check(&report);
    assert_eq!(
        br.severity,
        ai_brains_contracts::doctor::CheckSeverity::Ok,
        "PreT109 within age must be usable ok; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("").to_ascii_lowercase();
    assert!(
        msg.contains("usable") || msg.contains("within"),
        "ok message should mention usable/within; got {:?}",
        br.message
    );
}
