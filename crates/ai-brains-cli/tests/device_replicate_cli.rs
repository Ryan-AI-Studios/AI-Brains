#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T176 CLI smoke: device bootstrap / list / fingerprint / second bootstrap /
//! package-export / enroll / revoke / replicate.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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

fn bootstrap(vault_path: &std::path::Path) {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success()
        .stdout(predicate::str::contains("status=local"))
        .stdout(predicate::str::contains("signed_control"));
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

#[test]
fn cli_package_export__public_only__no_raw_seeds() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("peer.bin");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("Enrollment package written"));

    let bytes = fs::read(&out).expect("package");
    assert_eq!(
        bytes.len(),
        ai_brains_sync::ENROLLMENT_PACKAGE_LEN,
        "public package length"
    );
    // No raw seeds sidecar by default (ID-2).
    let seeds = out.with_extension("seeds");
    assert!(!seeds.exists(), "must not write raw .seeds next to package");
}

#[test]
fn cli_enroll__after_bootstrap_and_package__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let package = dir.path().join("peer.bin");
    let export = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&package)
        .output()
        .expect("export");
    assert!(export.status.success());
    let export_out = String::from_utf8_lossy(&export.stdout);
    let peer_id = export_out
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("enroll")
        .arg("--package")
        .arg(&package)
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("Enrolled peer"))
        .stdout(predicate::str::contains("signed DeviceEnrolled by"));

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains(&peer_id));
}

#[test]
fn cli_revoke__after_enroll__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let package = dir.path().join("peer.bin");
    let export = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&package)
        .output()
        .expect("export");
    assert!(export.status.success());
    let export_out = String::from_utf8_lossy(&export.stdout);
    let peer_id = export_out
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("enroll")
        .arg("--package")
        .arg(&package)
        .arg("--yes")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("revoke")
        .arg(&peer_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Revoked"))
        .stdout(predicate::str::contains("Signed DeviceRevoked"));
}

#[cfg(windows)]
#[test]
fn cli_package_export__write_private_key_dpapi__ok() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("peer.bin");
    let priv_path = dir.path().join("peer.key.dpapi");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&out)
        .arg("--write-private-key")
        .arg(&priv_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("DPAPI-protected private key"));

    let wrapped = fs::read(&priv_path).expect("private key file");
    assert!(
        wrapped.len() > 64,
        "DPAPI blob should be larger than raw 64-byte seeds"
    );
    // Still no raw .seeds
    assert!(!out.with_extension("seeds").exists());
}
