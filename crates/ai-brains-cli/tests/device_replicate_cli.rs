#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T176 CLI smoke: device bootstrap / list / fingerprint / second bootstrap / replicate.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn init_vault(vault_path: &std::path::Path) {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

#[test]
fn cli_device_bootstrap__temp_vault__lists_local() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success()
        .stdout(predicate::str::contains("status=local"))
        .stdout(predicate::str::contains("fingerprint:"));

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
fn cli_device_fingerprint__hyphen_form() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("fingerprint")
        .output()
        .expect("fingerprint");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().expect("line");
    let groups: Vec<&str> = line.trim().split('-').collect();
    assert_eq!(groups.len(), 16, "expected 16 hyphen groups: {line}");
    for g in groups {
        assert_eq!(g.len(), 4);
    }
}

#[test]
fn bootstrap__second_call__err() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("already enrolled")
                .or(predicate::str::contains("Bootstrap already enrolled")),
        );
}

#[test]
fn cli_replicate_push__no_relay__structured_err() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("push")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("relay not configured").or(predicate::str::contains("T177")),
        );

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("not configured"));
}
