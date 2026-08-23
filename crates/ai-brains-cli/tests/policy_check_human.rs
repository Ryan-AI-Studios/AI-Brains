//! T292 — `policy check` Family A human allow/deny + JSON freeze.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use rstest::rstest;
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

/// T241 F14 SHORT — human deny line 2 must match exactly.
const POLICY_BOOTSTRAP_SOOT_SHORT: &str =
    "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap`";

const SCOPE: &str = "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

/// F27: System bootstrap — omit `--principal-id` (hermetic_bin strips ambient).
fn system_bootstrap(vault: &Path) {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("system policy bootstrap");
    assert_eq!(
        out.status.code(),
        Some(0),
        "bootstrap must succeed; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

/// T292 AC2 — allow human after System bootstrap (omit principal both sides).
#[test]
fn policy_check__allow__format_human__allowed_line_not_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    system_bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ReadEvidence")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("human")
        .output()
        .expect("policy check allow human");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    assert!(
        !trimmed.starts_with('{'),
        "AC2: must not be a JSON object; got {stdout}"
    );
    assert!(
        trimmed.contains("allowed:") && trimmed.contains("ReadEvidence"),
        "AC2: allow line shape; got {stdout}"
    );
    assert_eq!(
        trimmed.lines().count(),
        1,
        "AC2: exactly one stdout line; got {stdout:?}"
    );
}

/// T292 AC3 — deny human two lines + SHORT; stderr has no POLICY_DENIED:.
#[test]
fn policy_check__deny__format_human__denied_plus_short_exit_3() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ProposeConclusion")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("human")
        .output()
        .expect("policy check deny human");

    assert_eq!(
        out.status.code(),
        Some(3),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let trimmed = stdout.trim();
    let lines: Vec<&str> = trimmed.lines().collect();
    assert!(
        !trimmed.starts_with('{'),
        "AC3: must not be a JSON object; got {stdout}"
    );
    assert_eq!(
        lines.len(),
        2,
        "AC3: exactly two stdout lines; got {stdout:?}"
    );
    assert_eq!(lines[0], "denied: ProposeConclusion");
    assert_eq!(
        lines[1], POLICY_BOOTSTRAP_SOOT_SHORT,
        "AC3: line 2 must equal SHORT exactly"
    );
    assert!(
        serde_json::from_str::<Value>(trimmed).is_err(),
        "AC3: stdout must not parse as ApiError JSON; got {stdout}"
    );
    assert!(
        !stderr.contains("POLICY_DENIED:"),
        "AC3: stderr must stay empty of POLICY_DENIED:; got {stderr}"
    );
}

/// T292 AC4 — JSON allow keys frozen (no next_step / found).
#[test]
fn policy_check__allow__format_json__keys_frozen() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    system_bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ReadEvidence")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check allow json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["allowed"], true);
    assert_eq!(v["capability"], "ReadEvidence");
    assert!(v.get("principal_id").and_then(|p| p.as_str()).is_some());
    assert!(v.get("scope").and_then(|s| s.as_str()).is_some());
    assert!(v.get("next_step").is_none(), "AC4: no next_step; got {v}");
    assert!(v.get("found").is_none(), "AC4: no found; got {v}");
}

/// T292 AC7 — non-TTY omit `--format` stays JSON (auto + pipe).
#[test]
fn policy_check__allow__omit_format__pipe_still_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    system_bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ReadEvidence")
        .arg("--scope")
        .arg(SCOPE)
        .output()
        .expect("policy check omit format");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("pipe default must be JSON");
    assert_eq!(v["allowed"], true);
    assert_eq!(v["capability"], "ReadEvidence");
}

/// T292 AC7 deny — pipe omit-format stays one POLICY_DENIED JSON document.
#[test]
fn policy_check__deny__omit_format__pipe_still_json_api_error() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ProposeConclusion")
        .arg("--scope")
        .arg(SCOPE)
        .output()
        .expect("policy check deny omit format");

    assert_eq!(
        out.status.code(),
        Some(3),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("pipe deny default must be JSON");
    assert_eq!(v["code"], "POLICY_DENIED");
    let hint = v
        .pointer("/details/hint")
        .and_then(|h| h.as_str())
        .unwrap_or("");
    assert!(!hint.is_empty(), "details.hint must be non-empty; got {v}");
}

/// T292 AC12 — pretty/md/text ≡ human allow line (rstest per alias).
#[rstest]
#[case("pretty")]
#[case("text")]
#[case("markdown")]
#[case("md")]
fn policy_check__allow__format_pretty_aliases__human_line(#[case] token: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    system_bootstrap(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ReadEvidence")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg(token)
        .output()
        .unwrap_or_else(|e| panic!("policy check --format {token}: {e}"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "{token}: stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with("allowed: true (ReadEvidence on "),
        "AC12: {token} must ≡ human allow; got {stdout}"
    );
    assert!(
        !trimmed.starts_with('{'),
        "AC12: {token} not JSON; got {stdout}"
    );
}
