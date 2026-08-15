#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T251 — `device status` discoverability hermetics (AC1–AC8 / AC12).

mod common;

use tempfile::tempdir;

const T198_EMPTY: &str = "No enrolled devices. Run `ai-brains device bootstrap` first.";
const DEVICE_STATUS_NEXT: &str = "next: ai-brains replicate status";

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn bootstrap(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success();
}

fn last_nonempty_line(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .map(str::trim)
        .rev()
        .find(|line| !line.is_empty())
}

/// AC1 / AC2: empty vault `device status` is recognized, T198 plural + next:.
#[test]
fn device_status__empty_vault__outputs_hint_and_next_replicate_status() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("status")
        .output()
        .expect("device status empty");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "device status empty must exit 0; stderr={stderr}"
    );
    assert!(
        !stdout.contains("unrecognized subcommand") && !stderr.contains("unrecognized subcommand"),
        "device status must be a recognized subcommand; stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains(T198_EMPTY),
        "expected exact T198 plural line; got: {stdout}"
    );
    assert_eq!(
        last_nonempty_line(&stdout),
        Some(DEVICE_STATUS_NEXT),
        "last non-empty data line must be next pointer; got: {stdout}"
    );
}

/// AC3: enrolled vault prints roster + always-appended next:.
#[test]
fn device_status__enrolled_vault__outputs_table_and_next_replicate_status() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("status")
        .output()
        .expect("device status enrolled");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "device status enrolled must exit 0; stderr={stderr}"
    );
    assert!(
        stdout.contains("DEVICE_ID") || stdout.contains("local"),
        "expected enrolled roster (DEVICE_ID or local); got: {stdout}"
    );
    assert!(
        stdout.contains(DEVICE_STATUS_NEXT),
        "enrolled status must append next pointer; got: {stdout}"
    );
}

/// AC8: no `--format` on status — clap unexpected argument, exit 2.
#[test]
fn device_status__with_format_json_flag__fails_exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("status")
        .arg("--format")
        .arg("json")
        .output()
        .expect("device status --format json");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "unexpected --format must be clap exit 2; stdout={stdout} stderr={stderr}"
    );
    assert!(
        stderr.contains("unexpected argument"),
        "expected clap unexpected-argument for --format; stderr={stderr}"
    );
    assert!(
        !stderr.contains("unrecognized subcommand"),
        "status must be recognized; --format is the unexpected arg; stderr={stderr}"
    );
    assert!(
        !stdout.trim().starts_with('{'),
        "must not silently emit JSON; stdout={stdout}"
    );
}

/// AC4 empty: `device list` keeps T198 and does not grow next:.
#[test]
fn device_list__empty_vault__does_not_contain_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("list")
        .output()
        .expect("device list empty");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains(T198_EMPTY),
        "empty list must keep T198 plural; got: {stdout}"
    );
    assert!(
        !stdout.contains("next:"),
        "device list must not contain next:; got: {stdout}"
    );
}

/// AC4 enrolled: `device list` shows local and does not grow next:.
#[test]
fn device_list__enrolled_vault__does_not_contain_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("list")
        .output()
        .expect("device list enrolled");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("local"),
        "enrolled list must contain local; got: {stdout}"
    );
    assert!(
        !stdout.contains("next:"),
        "device list must not contain next:; got: {stdout}"
    );
}

/// AC6: `device --help` lists status and after_help names the command.
#[test]
fn device_status__help__lists_status() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("device")
        .arg("--help")
        .output()
        .expect("device --help");

    assert!(
        out.status.success(),
        "device --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        stdout.contains("status"),
        "device --help must list status subcommand; got: {stdout}"
    );
    assert!(
        combined.contains("ai-brains device status"),
        "combined help/after_help must contain `ai-brains device status`; got: {combined}"
    );
}

/// AC7: `replicate status` empty vault is unchanged (honesty + bootstrap hint).
#[test]
fn replicate_status__empty_vault__still_prints_enrolled_count_honesty_hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .output()
        .expect("replicate status empty");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("enrolled_count"),
        "replicate status must keep enrolled_count; got: {stdout}"
    );
    assert!(
        stdout.contains("honesty") || stdout.contains("not PQ"),
        "replicate status must keep honesty; got: {stdout}"
    );
    assert!(
        stdout.contains("bootstrap") && stdout.contains("hint"),
        "replicate status empty must keep bootstrap hint; got: {stdout}"
    );
}

/// AC5 companion: fingerprint empty stays T198 one-liner (no next:).
#[test]
fn device_fingerprint__empty_vault__does_not_contain_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("fingerprint")
        .output()
        .expect("device fingerprint empty");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains(T198_EMPTY),
        "fingerprint empty must stay T198 plural; got: {stdout}"
    );
    assert!(
        !stdout.contains("next:"),
        "fingerprint empty must not contain next:; got: {stdout}"
    );
}
