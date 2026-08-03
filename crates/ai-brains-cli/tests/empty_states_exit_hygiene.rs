#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T198 — Empty states + exit hygiene hermetic tests (AC1–AC5).

mod common;

use std::fs;
use tempfile::tempdir;

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

/// AC1: `backup verify` with zero discovered backups — human non-blank, exit 0.
#[test]
fn backup_verify__empty__human_message_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("verify")
        .output()
        .expect("backup verify empty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No backups to verify."),
        "expected empty-state human line; got: {stdout}"
    );
}

/// AC1: `backup verify --format json` empty — results[], status ok, message set.
#[test]
fn backup_verify__empty__json_status_ok_message() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("verify")
        .arg("--format")
        .arg("json")
        .output()
        .expect("backup verify json empty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("VerifyOutput JSON");
    assert!(
        v["results"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "results must be []; got: {v}"
    );
    assert_eq!(v["status"], "ok");
    assert_eq!(v["message"], "No backups to verify.");
}

/// AC2: `project list` empty vault — empty-state line after header; exit 0.
#[test]
fn project_list__empty__empty_state_line_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list empty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("project_id"),
        "must keep header; got: {stdout}"
    );
    assert!(
        stdout.contains("No projects registered. (0 projects)"),
        "expected empty-state line; got: {stdout}"
    );
}

/// AC3: dogfood missing governed file — non-empty error body on stdout; exit 6.
#[test]
fn dogfood_compare__missing_governed__stdout_body_exit_6() {
    let dir = tempdir().unwrap();
    let legacy = dir.path().join("legacy.json");
    fs::write(&legacy, r#"{"text": "DECISION: one", "word_count": 2}"#).unwrap();
    let missing = dir.path().join("no-such-governed.json");
    let out_path = dir.path().join("compare.json");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("dogfood")
        .arg("compare")
        .arg("--governed")
        .arg(&missing)
        .arg("--legacy")
        .arg(&legacy)
        .arg("--out")
        .arg(&out_path)
        .output()
        .expect("dogfood missing governed");

    assert_eq!(
        out.status.code(),
        Some(6),
        "expected EXIT_INVALID_PAYLOAD=6; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "error body must not be silent empty; stdout empty"
    );
    assert!(
        stdout.contains("INVALID_PAYLOAD") || stdout.contains("failed to read"),
        "expected INVALID_PAYLOAD envelope or path message; got: {stdout}"
    );
    // Path-related message must include the missing path.
    let path_s = missing.to_string_lossy();
    assert!(
        stdout.contains(path_s.as_ref()) || stdout.contains("failed to read"),
        "message should reference path; got: {stdout}"
    );
}

/// AC5: fingerprint with no enroll — bootstrap next-step on stdout; exit 0.
#[test]
fn device_fingerprint__no_enroll__bootstrap_message_exit_0() {
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
        .expect("device fingerprint no enroll");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No enrolled devices. Run `ai-brains device bootstrap` first."),
        "expected bootstrap next-step; got: {stdout}"
    );
}

/// Soft F8 regression: project detect miss mentions context (exit 1, stderr).
#[test]
fn project_detect__miss__mentions_context_exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Run from empty temp dir so git slug / .env do not resolve a project.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(dir.path())
        .arg("project")
        .arg("detect")
        .output()
        .expect("project detect miss");

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("No project detected"),
        "expected detect miss; got: {stderr}"
    );
    assert!(
        stderr.contains("context") || stderr.contains("ai-brains context"),
        "soft F8: should mention context; got: {stderr}"
    );
}

/// Non-empty verify JSON still carries status; message omitted (AC7 additive).
#[test]
fn backup_verify__nonempty_json__status_ok_no_message() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Create one backup so verify is non-empty.
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("create")
        .assert()
        .success();

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("verify")
        .arg("--format")
        .arg("json")
        .output()
        .expect("backup verify non-empty json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("verify json must contain object line; got: {stdout}"));
    let v: serde_json::Value = serde_json::from_str(json_line).expect("VerifyOutput JSON");
    assert!(
        v["results"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "results must be non-empty; got: {v}"
    );
    assert_eq!(v["status"], "ok");
    assert!(
        v.get("message").is_none() || v["message"].is_null(),
        "message must be omitted on non-empty path; got: {v}"
    );
}
