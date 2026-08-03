#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T197 vault open UX + key bootstrap process tests (AC1–AC4, AC6, AC8, AC11).

mod common;

use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport, DoctorStatus};
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const ZERO_KEY: &str = common::ZERO_SQLCIPHER_KEY;
const ALT_KEY: &str = "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'";

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

fn init_vault(vault_path: &Path, key: &str) {
    hermetic_with_key(vault_path, key)
        .arg("init")
        .assert()
        .success();
}

fn stderr_lines(stderr: &str) -> usize {
    stderr.lines().filter(|l| !l.trim().is_empty()).count()
}

/// AC1: wrong key on doctor → <20 stderr lines; zero "hmac check failed".
#[test]
fn doctor__wrong_key__no_hmac_spam_stderr_bounded() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault, ZERO_KEY);

    let out = hermetic_with_key(&vault, ALT_KEY)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor wrong key");

    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.to_ascii_lowercase().contains("hmac check failed"),
        "must not spam hmac check failed; stderr={stderr}"
    );
    assert!(
        stderr_lines(&stderr) < 20,
        "stderr must be <20 non-empty lines, got {}; stderr={stderr}",
        stderr_lines(&stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport");
    let open = report
        .checks
        .iter()
        .find(|c| c.name == "vault_open")
        .expect("vault_open");
    assert_eq!(open.severity, CheckSeverity::Fail);
}

/// AC11: missing key → vault_open skipped; exit 1; report emits.
#[test]
fn doctor__missing_key__vault_open_skipped() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault, ZERO_KEY);

    let mut cmd = common::hermetic_bin();
    // Strip hermetic default key to exercise Missing path.
    cmd.env_remove("AI_BRAINS_KEY");
    let out = cmd
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .output()
        .expect("doctor missing key");

    assert_eq!(out.status.code(), Some(1));
    let report: DoctorReport = serde_json::from_slice(&out.stdout).expect("DoctorReport");
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
        "expected missing skip message, got {msg}"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.to_ascii_lowercase().contains("hmac check failed"),
        "no hmac spam on missing key"
    );
    assert!(
        stderr_lines(&stderr) < 20,
        "stderr lines={}",
        stderr_lines(&stderr)
    );
}

/// AC4: AppContext path (recall) missing key → VAULT_KEY_MISSING, not silent zero.
#[test]
fn recall__missing_key__vault_key_missing_code() {
    assert_appcontext_missing_key_family(&["recall", "anything"]);
}

/// AC2: preflight shares AppContext F8 missing-key family.
#[test]
fn preflight__missing_key__vault_key_missing_code() {
    assert_appcontext_missing_key_family(&["preflight", "--summary"]);
}

/// AC2: project list shares AppContext F8 missing-key family.
#[test]
fn project_list__missing_key__vault_key_missing_code() {
    assert_appcontext_missing_key_family(&["project", "list"]);
}

fn assert_appcontext_missing_key_family(args: &[&str]) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault, ZERO_KEY);

    let mut cmd = common::hermetic_bin();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let mut c = cmd
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault);
    for a in args {
        c = c.arg(a);
    }
    let out = c.output().expect("appcontext missing key");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("VAULT_KEY_MISSING") || stderr.contains("Vault key missing:"),
        "expected missing key family for {args:?}, got {stderr}"
    );
    assert!(!stderr.contains("hmac check failed"));
}

/// AC3/AC8: invalid format fails at resolve with VAULT_KEY_FORMAT.
#[test]
fn recall__invalid_format__vault_key_format_code() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault, ZERO_KEY);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("--key")
        .arg("not-a-valid-key")
        .arg("recall")
        .arg("x")
        .output()
        .expect("recall format");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("VAULT_KEY_FORMAT") || stderr.contains("Vault key invalid format:"),
        "got {stderr}"
    );
}

/// AC8: explicit zero without ALLOW → VAULT_KEY_ZERO.
#[test]
fn recall__zero_key_without_allow__vault_key_zero() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    // Init with non-zero so vault exists under real key, then probe zero refuse path.
    init_vault(&vault, ALT_KEY);

    let mut cmd = common::hermetic_bin();
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let out = cmd
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("--key")
        .arg(ZERO_KEY)
        .arg("recall")
        .arg("x")
        .output()
        .expect("recall zero");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("VAULT_KEY_ZERO") || stderr.contains("Vault key refused:"),
        "got {stderr}"
    );
}

/// F19: init without key generates and prints PowerShell/bash examples once.
#[test]
fn init__missing_key__generates_and_prints_bootstrap() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");

    let mut cmd = common::hermetic_bin();
    cmd.env_remove("AI_BRAINS_KEY");
    // Allow zero not needed — generated key is non-zero.
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let out = cmd
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .output()
        .expect("init generate");

    assert!(
        out.status.success(),
        "init must succeed; out={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(vault.exists());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Vault initialized successfully"),
        "got {stdout}"
    );
    assert!(
        stdout.contains("$env:AI_BRAINS_KEY") && stdout.contains("export AI_BRAINS_KEY"),
        "expected PS/bash examples, got {stdout}"
    );
    // Product form appears once in examples.
    assert!(
        stdout.contains("x'"),
        "expected generated key material form, got {stdout}"
    );
}

/// F19: init with provided key does not print generated bootstrap block.
#[test]
fn init__provided_key__no_generate_banner() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("init")
        .output()
        .expect("init with key");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Vault initialized successfully"));
    assert!(
        !stdout.contains("Generated vault key"),
        "must not print generate banner when key provided: {stdout}"
    );
}

/// Wrong key on recall: bounded stderr, Vault locked family.
#[test]
fn recall__wrong_key__vault_locked_no_hmac_spam() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault, ZERO_KEY);

    let out = hermetic_with_key(&vault, ALT_KEY)
        .arg("recall")
        .arg("query")
        .output()
        .expect("recall wrong key");

    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.to_ascii_lowercase().contains("hmac check failed"),
        "hmac spam: {stderr}"
    );
    assert!(
        stderr_lines(&stderr) < 20,
        "stderr lines={}",
        stderr_lines(&stderr)
    );
    assert!(
        stderr.contains("VAULT_LOCKED")
            || stderr.contains("Vault locked:")
            || stderr.contains("Vault is locked"),
        "expected locked family, got {stderr}"
    );
}

/// Smoke: generated init key can open doctor successfully.
#[test]
fn init__generated_key__opens_doctor() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");

    let mut cmd = common::hermetic_bin();
    cmd.env_remove("AI_BRAINS_KEY");
    cmd.env_remove("AI_BRAINS_ALLOW_ZERO_KEY");
    let out = cmd
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("init")
        .output()
        .expect("init");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Extract x'…' from PowerShell line.
    let key = stdout
        .lines()
        .find_map(|line| {
            let marker = "AI_BRAINS_KEY = \"";
            let idx = line.find(marker)?;
            let rest = &line[idx + marker.len()..];
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        })
        .expect("parse generated key from stdout");
    assert!(key.starts_with("x'") && key.len() == 67, "key={key}");

    hermetic_with_key(&vault, &key)
        .arg("doctor")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let report: DoctorReport = serde_json::from_str(s).expect("json");
            report
                .checks
                .iter()
                .any(|c| c.name == "vault_open" && c.severity == CheckSeverity::Ok)
        }));
}

/// Ensure fixture file is not left empty for compiler unused imports.
#[test]
fn vault_key_bootstrap__tempdir_cleanup() {
    let dir = tempdir().unwrap();
    let p = dir.path().join("marker");
    fs::write(&p, b"ok").unwrap();
    assert!(p.exists());
}
