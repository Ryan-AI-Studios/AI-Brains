#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T176 CLI smoke: device bootstrap / list / fingerprint / second bootstrap /
//! package-export / enroll / revoke / replicate.

mod common;

use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn bootstrap(vault_path: &std::path::Path) {
    common::hermetic_bin()
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

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success()
        .stdout(predicate::str::contains("status=local"))
        .stdout(predicate::str::contains("fingerprint:"));

    common::hermetic_bin()
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

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success();

    let output = common::hermetic_bin()
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

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .assert()
        .success();

    common::hermetic_bin()
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
fn cli_replicate_push__no_config__err() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("push")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("relay not configured")
                .or(predicate::str::contains("fake-relay"))
                .or(predicate::str::contains("AI_BRAINS_SYNC_FAKE_RELAY_PATH")),
        );

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("not configured"));
}

#[test]
fn cli_replicate_push__fake_relay__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let relay = dir.path().join("fake-relay");
    init_vault(&vault);
    bootstrap(&vault);

    // Bootstrap does not yet enqueue via ReplicateEngine outbox, so push may be 0.
    // Still must exit 0, report structured count, and create the fake-relay marker.
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("push")
        .arg("--fake-relay")
        .arg(&relay)
        .assert()
        .success()
        .stdout(predicate::str::contains("replicate push"))
        .stdout(predicate::str::contains("pushed"));

    let marker = relay.join(ai_brains_sync::FAKE_RELAY_MARKER);
    assert!(
        marker.exists(),
        "FileFakeRelay must write marker at {}",
        marker.display()
    );

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("status")
        .arg("--fake-relay")
        .arg(&relay)
        .assert()
        .success()
        .stdout(predicate::str::contains("file:"));
}

#[test]
fn cli_replicate_push__format_json__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let relay = dir.path().join("fake-relay");
    init_vault(&vault);
    bootstrap(&vault);

    let output = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("push")
        .arg("--fake-relay")
        .arg(&relay)
        .arg("--format")
        .arg("json")
        .output()
        .expect("push json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json object");
    assert_eq!(v["ok"], true);
    assert!(v["pushed"].is_number(), "pushed must be a number: {v}");
    let relay_s = v["relay"].as_str().expect("relay string");
    assert!(
        relay_s.starts_with("file:"),
        "relay should be file: path, got {relay_s}"
    );
    assert!(
        v.get("pulled_peers").is_none(),
        "push must not emit pulled_peers"
    );
}

#[test]
fn cli_replicate_pull__format_json__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let relay = dir.path().join("fake-relay");
    init_vault(&vault);
    bootstrap(&vault);

    // Ensure relay exists (push creates marker).
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("push")
        .arg("--fake-relay")
        .arg(&relay)
        .assert()
        .success();

    let output = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("replicate")
        .arg("pull")
        .arg("--fake-relay")
        .arg(&relay)
        .arg("--format")
        .arg("json")
        .output()
        .expect("pull json");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json object");
    assert_eq!(v["ok"], true);
    // Envelope apply count (not peer count); renamed from pulled_peers (P3).
    assert!(
        v["applied"].is_number(),
        "applied must be envelope count number: {v}"
    );
    let relay_s = v["relay"].as_str().expect("relay string");
    assert!(
        relay_s.starts_with("file:"),
        "relay should be file: path, got {relay_s}"
    );
    assert!(
        v.get("pulled_peers").is_none(),
        "pull JSON must use applied, not pulled_peers"
    );
}

#[test]
fn cli_package_export__public_only__no_raw_seeds() {
    let dir = tempdir().unwrap();
    // package-export does not read the vault, but the CLI still requires a vault path
    // (global gate). Hermetic: never rely on ambient AI_BRAINS_VAULT_PATH (Linux CI has none).
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let out = dir.path().join("peer.bin");

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
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
    let export = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&package)
        .output()
        .expect("export");
    assert!(
        export.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&export.stderr)
    );
    let export_out = String::from_utf8_lossy(&export.stdout);
    let peer_id = export_out
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    common::hermetic_bin()
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

    common::hermetic_bin()
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
    let export = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&package)
        .output()
        .expect("export");
    assert!(
        export.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&export.stderr)
    );
    let export_out = String::from_utf8_lossy(&export.stdout);
    let peer_id = export_out
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("enroll")
        .arg("--package")
        .arg(&package)
        .arg("--yes")
        .assert()
        .success();

    common::hermetic_bin()
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

#[test]
fn bootstrap__appends_device_enrolled_event_log_sov() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let count = count_events_of_type(&vault, "DeviceEnrolled").expect("query events");
    assert!(
        count >= 1,
        "bootstrap must append ≥1 DeviceEnrolled row to events (SOV), got {count}"
    );
}

#[test]
fn revoke__self__fails_adr0018_l4() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("bootstrap")
        .output()
        .expect("bootstrap");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let local_id = stdout
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("revoke")
        .arg(&local_id)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("self-revoke")
                .or(predicate::str::contains("sole authority"))
                .or(predicate::str::contains("ADR-0018")),
        );
}

#[test]
fn revoke__peer_after_enroll__ok_and_event_log() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    bootstrap(&vault);

    let package = dir.path().join("peer.bin");
    let export = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("package-export")
        .arg("--out")
        .arg(&package)
        .output()
        .expect("export");
    assert!(
        export.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&export.stderr)
    );
    let export_out = String::from_utf8_lossy(&export.stdout);
    let peer_id = export_out
        .lines()
        .find_map(|l| l.strip_prefix("device_id: "))
        .expect("device_id line")
        .trim()
        .to_string();

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("enroll")
        .arg("--package")
        .arg(&package)
        .arg("--yes")
        .assert()
        .success();

    let enrolled = count_events_of_type(&vault, "DeviceEnrolled").expect("enrolled events");
    assert!(
        enrolled >= 2,
        "bootstrap + enroll must produce ≥2 DeviceEnrolled events, got {enrolled}"
    );

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
        .arg("device")
        .arg("revoke")
        .arg(&peer_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Revoked"));

    let revoked = count_events_of_type(&vault, "DeviceRevoked").expect("revoked events");
    assert!(
        revoked >= 1,
        "peer revoke must append DeviceRevoked to events, got {revoked}"
    );
}

fn count_events_of_type(
    vault_path: &Path,
    event_type: &str,
) -> Result<i64, Box<dyn std::error::Error>> {
    // T197: hermetic vaults use explicit zero key; open requires ALLOW_ZERO_KEY.
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(vault_path, &key)?;
    let locked = conn.lock()?;
    let count: i64 = locked.query_row(
        "SELECT COUNT(*) FROM events WHERE event_type = ?",
        [event_type],
        |r| r.get(0),
    )?;
    Ok(count)
}

#[cfg(windows)]
#[test]
fn cli_package_export__write_private_key_dpapi__ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let out = dir.path().join("peer.bin");
    let priv_path = dir.path().join("peer.key.dpapi");

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault)
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
