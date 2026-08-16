//! T255 hermetic `nightly --status` format contract (AC9 / AC10 / AC14).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_store::VaultConnection;
use tempfile::tempdir;

fn open_temp_vault() -> (tempfile::TempDir, std::path::PathBuf, String) {
    let dir = tempdir().expect("tempdir");
    let vault_path = dir.path().join("vault.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let key_arg = sql_key.expose_secret().to_string();
    {
        let conn = VaultConnection::open(&vault_path, &sql_key).expect("open vault");
        conn.migrate().expect("migrate vault");
    }
    (dir, vault_path, key_arg)
}

fn nightly_status_cmd(
    vault_path: &std::path::Path,
    key_arg: &str,
    extra: &[&str],
) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("--key")
        .arg(key_arg)
        .arg("nightly")
        .arg("--status");
    for arg in extra {
        cmd.arg(arg);
    }
    cmd
}

/// AC9: `--status --format json --quick` is one JSON object, no human header.
#[test]
fn nightly_status__format_json__one_object_no_human_header() {
    let (_dir, vault_path, key_arg) = open_temp_vault();
    let output = nightly_status_cmd(&vault_path, &key_arg, &["--format", "json", "--quick"])
        .output()
        .expect("nightly --status --format json --quick");
    assert!(
        output.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("=== Nightly Status ==="),
        "JSON status must not print the human header; got: {stdout}"
    );
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be one JSON object");
    assert_eq!(value["schema_version"], 1);
    assert!(
        value.get("completion").is_some_and(|c| c.is_object()),
        "completion object required; got: {value}"
    );
    assert!(
        value.get("embedding").is_some_and(|e| e.is_object()),
        "embedding object required; got: {value}"
    );
    assert_eq!(value["multi_import"]["status"], "never");
}

/// AC10: omitted `--format` stays human even when stdout is not a TTY.
#[test]
fn nightly_status__default_format__human_header_even_if_piped() {
    let (_dir, vault_path, key_arg) = open_temp_vault();
    let output = nightly_status_cmd(&vault_path, &key_arg, &["--quick"])
        .output()
        .expect("nightly --status --quick");
    assert!(
        output.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("=== Nightly Status ==="),
        "piped default must stay human; got: {stdout}"
    );
}

/// AC14: `--format json --quick` skips probes (`probe` is the string `skipped`).
#[test]
fn nightly_status__format_json_quick__probe_skipped() {
    let (_dir, vault_path, key_arg) = open_temp_vault();
    let output = nightly_status_cmd(&vault_path, &key_arg, &["--format", "json", "--quick"])
        .output()
        .expect("nightly --status --format json --quick");
    assert!(
        output.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be one JSON object");
    assert_eq!(value["completion"]["probe"], "skipped");
    assert_eq!(value["embedding"]["probe"], "skipped");
}
