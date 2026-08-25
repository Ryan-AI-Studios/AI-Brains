#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T251 — `device status` discoverability hermetics (AC1–AC8 / AC12).
//! T298 — this-machine + short honesty on empty/enrolled status surfaces.

mod common;

use tempfile::tempdir;

const T198_EMPTY: &str = "No enrolled devices. Run `ai-brains device bootstrap` first.";
const DEVICE_STATUS_NEXT: &str = "next: ai-brains replicate status";
const DEVICE_STATUS_HONESTY: &str = "local-only; not PQ; not remote wipe";
const T298_HOST: &str = "T298-HOST";
const THIS_MACHINE_PREFIX: &str = "  this machine:    ";

fn with_t298_host(mut cmd: assert_cmd::Command) -> assert_cmd::Command {
    cmd.env("COMPUTERNAME", T298_HOST);
    cmd.env_remove("HOSTNAME");
    cmd
}

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

/// T251 AC1 + T298 AC1: empty vault four-line body (T198 + this-machine + honesty + next:).
#[test]
fn device_status__empty_vault__outputs_hint_and_next_replicate_status() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = with_t298_host(common::hermetic_bin())
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
    assert!(
        stdout.contains(&format!("this machine: {T298_HOST} (not enrolled)")),
        "expected this-machine not-enrolled; got: {stdout}"
    );
    assert!(
        stdout.contains(DEVICE_STATUS_HONESTY),
        "expected short honesty const; got: {stdout}"
    );
    assert_eq!(
        last_nonempty_line(&stdout),
        Some(DEVICE_STATUS_NEXT),
        "last non-empty data line must be next pointer; got: {stdout}"
    );
    let nonempty: Vec<&str> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        nonempty.len(),
        4,
        "empty device status must be exactly four non-empty lines; got: {stdout}"
    );
}

/// T251 AC3 + T298 AC2: enrolled vault roster + this-machine fingerprint + honesty + next: last.
#[test]
fn device_status__enrolled_vault__outputs_table_and_next_replicate_status() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let out = with_t298_host(common::hermetic_bin())
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
        stdout.contains("this machine:"),
        "enrolled status must print this machine:; got: {stdout}"
    );
    assert!(
        !stdout.contains("(not enrolled)") && !stdout.contains("fingerprint unavailable"),
        "enrolled happy path must not claim not enrolled / unavailable; got: {stdout}"
    );
    let this_line = stdout
        .lines()
        .find(|l| l.contains("this machine:"))
        .unwrap_or("");
    assert!(
        this_line.contains('-'),
        "enrolled this-machine must be hyphen fingerprint; got: {this_line}"
    );
    assert!(
        stdout.contains(DEVICE_STATUS_HONESTY),
        "enrolled status must print short honesty; got: {stdout}"
    );
    assert_eq!(
        last_nonempty_line(&stdout),
        Some(DEVICE_STATUS_NEXT),
        "last non-empty data line must be next pointer; got: {stdout}"
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

/// T251 AC4 + T298 AC3 empty: `device list` keeps T198 and does not grow T298 lines.
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
        !stdout.contains("next:")
            && !stdout.contains("this machine:")
            && !stdout.contains(DEVICE_STATUS_HONESTY),
        "device list must not contain next:/this-machine/honesty; got: {stdout}"
    );
}

/// T251 AC4 + T298 AC3 enrolled: `device list` shows local and does not grow T298 lines.
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
        !stdout.contains("next:")
            && !stdout.contains("this machine:")
            && !stdout.contains(DEVICE_STATUS_HONESTY),
        "device list must not contain next:/this-machine/honesty; got: {stdout}"
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

/// T251 AC7 + T298 AC6 / AC16: empty replicate status adds this-machine; keeps honesty + hint.
#[test]
fn replicate_status__empty_vault__still_prints_enrolled_count_honesty_hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = with_t298_host(common::hermetic_bin())
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
        stdout.contains("enrolled_count") && stdout.contains('0'),
        "replicate status must keep enrolled_count 0; got: {stdout}"
    );
    assert!(
        stdout.contains("this machine:")
            && stdout.contains("(not enrolled)")
            && stdout.contains(T298_HOST),
        "replicate status empty must name this-machine not enrolled; got: {stdout}"
    );
    let this_line = stdout
        .lines()
        .find(|l| l.contains("this machine:"))
        .unwrap_or("");
    assert!(
        this_line.starts_with(THIS_MACHINE_PREFIX),
        "this machine line must use exact 19-char prefix; got: {this_line:?} (len {})",
        "  this machine:    ".len()
    );
    assert_eq!(THIS_MACHINE_PREFIX.len(), 19);
    assert!(
        stdout.contains("honesty") || stdout.contains("not PQ"),
        "replicate status must keep honesty; got: {stdout}"
    );
    assert!(
        stdout.contains("bootstrap") && stdout.contains("hint"),
        "replicate status empty must keep bootstrap hint; got: {stdout}"
    );
    assert!(
        !stdout.contains("sync: running") && !stdout.contains("replication: running"),
        "must not claim sync is running; got: {stdout}"
    );
}

/// T298 AC7: JSON keys frozen; no this_machine.
#[test]
fn replicate_status__format_json__keys_frozen_no_this_machine() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = with_t298_host(common::hermetic_bin())
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .arg("--format")
        .arg("json")
        .output()
        .expect("replicate status --format json");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json object on stdout");
    let obj = v.as_object().expect("object");
    let keys: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    let expected: std::collections::BTreeSet<&str> = [
        "relay",
        "enrolled_count",
        "cursors",
        "gap_or_blocked",
        "devices",
        "honesty",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        keys, expected,
        "JSON key set must stay frozen; got: {keys:?}"
    );
    assert!(!obj.contains_key("this_machine"));
    assert!(!stdout.contains("this machine"));
    assert_eq!(obj.get("enrolled_count").and_then(|n| n.as_u64()), Some(0));
}

/// T298 AC8: --quiet stays relay-only.
#[test]
fn replicate_status__quiet__relay_only_no_this_machine() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = with_t298_host(common::hermetic_bin())
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .arg("--quiet")
        .output()
        .expect("replicate status --quiet");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("not configured"),
        "quiet must print relay line; got: {stdout}"
    );
    assert!(
        !stdout.contains("this machine:"),
        "quiet must not print this machine:; got: {stdout}"
    );
}

/// T298 AC9: enrolled fingerprint shared across device status / replicate status / fingerprint.
#[test]
fn device_and_replicate_status__enrolled__share_fingerprint_label() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let fp_out = with_t298_host(common::hermetic_bin())
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("fingerprint")
        .output()
        .expect("device fingerprint enrolled");
    let fp = String::from_utf8_lossy(&fp_out.stdout).trim().to_string();
    assert!(
        fp_out.status.success() && fp.contains('-'),
        "fingerprint must be hyphen form; got: {fp}"
    );

    let status_out = with_t298_host(common::hermetic_bin())
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("status")
        .output()
        .expect("device status enrolled");
    let status_stdout = String::from_utf8_lossy(&status_out.stdout);
    assert!(
        status_stdout.contains(&fp),
        "device status this-machine must contain fingerprint; got: {status_stdout}"
    );

    let rep_out = with_t298_host(common::hermetic_bin())
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .output()
        .expect("replicate status enrolled");
    let rep_stdout = String::from_utf8_lossy(&rep_out.stdout);
    assert!(
        rep_stdout.contains(&fp),
        "replicate status this-machine must contain fingerprint; got: {rep_stdout}"
    );
}

/// T251 AC5 companion + T298 AC4: fingerprint empty stays T198 one-liner (no T298 lines).
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
        !stdout.contains("next:") && !stdout.contains("this machine:"),
        "fingerprint empty must not contain next:/this machine:; got: {stdout}"
    );
}
