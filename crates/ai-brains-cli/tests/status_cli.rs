#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T320 unified `status` glance hermetic CLI tests (AC9 / AC10 / AC17).

mod common;

use predicates::prelude::*;
use std::path::Path;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

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

/// AC9: `status --format json` exits 0; envelope keys present.
/// Do **not** assert `daemon.state` Running/Stopped (host IPC — F45).
#[test]
fn status__format_json__parses_envelope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("status")
        .arg("--format")
        .arg("json")
        .output()
        .expect("status --format json");
    assert!(
        out.status.success(),
        "status must exit 0; out={}",
        combined_output(&out)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json object");
    assert_eq!(v["schema_version"], 1);
    for key in ["daemon", "doctor", "graph", "nightly"] {
        assert!(
            v.get(key).is_some_and(|s| s.is_object()),
            "missing object key {key}; got {v}"
        );
    }
    let daemon = &v["daemon"];
    assert!(
        daemon.get("state").is_some() || daemon.get("error").is_some(),
        "daemon must have state or error; got {daemon}"
    );
    // F45: do not assert Running vs Stopped.
}

/// AC10: `--format human` prints daemon: and doctor: lines.
#[test]
fn status__format_human__prints_daemon_and_doctor() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("status")
        .arg("--format")
        .arg("human")
        .output()
        .expect("status --format human");
    assert!(
        out.status.success(),
        "status human must exit 0; out={}",
        combined_output(&out)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("daemon:"),
        "expected daemon: line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("doctor:"),
        "expected doctor: line; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("=== Nightly Status ==="),
        "must not print nightly banner"
    );
}

/// AC17: `status --help` carries F26 examples and “does not replace”.
#[test]
fn status__help__contains_examples_and_does_not_replace() {
    let out = common::hermetic_bin()
        .arg("status")
        .arg("--help")
        .output()
        .expect("status --help");
    assert!(
        out.status.success(),
        "status --help must succeed; out={}",
        combined_output(&out)
    );
    let text = combined_output(&out);
    assert!(
        text.contains("ai-brains status"),
        "help examples; got:\n{text}"
    );
    assert!(
        text.to_ascii_lowercase().contains("does not replace"),
        "F26 does not replace; got:\n{text}"
    );
    assert!(
        text.contains("--format json") || text.contains("status --format json"),
        "json example; got:\n{text}"
    );
}

/// AC10: piped `--format auto` emits JSON (no human `daemon: Running` line).
#[test]
fn status__format_auto_piped__json_envelope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("status")
        .arg("--format")
        .arg("auto")
        .output()
        .expect("status --format auto");
    assert!(
        out.status.success(),
        "status auto must exit 0; out={}",
        combined_output(&out)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // assert_cmd / nextest children are non-TTY → auto resolves to json.
    assert!(
        !stdout.contains("daemon: Running") && !stdout.contains("daemon: Stopped"),
        "piped auto must not be human; got:\n{stdout}"
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json object");
    assert_eq!(v["schema_version"], 1);
    assert!(v.get("daemon").is_some());
}

/// Stay-green: unknown `--format` is clap exit 2.
#[test]
fn status__format_xml__clap_exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    hermetic_with_key(&vault, ZERO_KEY)
        .arg("status")
        .arg("--format")
        .arg("xml")
        .assert()
        .failure()
        .code(predicate::eq(2));
}
