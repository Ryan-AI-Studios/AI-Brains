//! T201 — CLI exit-code + envelope contract hermetic locks (F18 / AC2–AC5 / AC3b / AC8 / AC11).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use tempfile::tempdir;

const SAMPLE_SCOPE: &str = "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// T226 AC1 — policy show omit scope + non-authoritative → fail_usage exit 2
// ---------------------------------------------------------------------------

#[test]
fn policy_show__missing_scope_no_context__exit_2_fail_usage() {
    // T226 F3/M2: soft-resolve fail_usage (runtime exit 2), not clap "required arguments".
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show missing scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --scope must exit 2 (fail_usage); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("required arguments were not provided"),
        "must not be clap required-argument text: {stderr}"
    );
    assert!(
        stderr.contains("--scope") || stderr.contains("scope resolve"),
        "fail_usage template expected: {stderr}"
    );
    assert!(
        stderr.contains("not filled silently") || stderr.contains("not authoritative"),
        "non-authoritative note expected: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// T226 AC2 — policy check omit scope + non-authoritative → fail_usage exit 2
// ---------------------------------------------------------------------------

#[test]
fn policy_check__missing_scope_no_context__exit_2_fail_usage() {
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
        .arg("ReadEvidence")
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check missing scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --scope must exit 2 (fail_usage); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("required arguments were not provided"),
        "must not be clap required-argument text: {stderr}"
    );
    assert!(
        stderr.contains("--scope") || stderr.contains("scope resolve"),
        "fail_usage template expected: {stderr}"
    );
    assert!(
        stderr.contains("not filled silently") || stderr.contains("not authoritative"),
        "non-authoritative note expected: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// (2) / AC3 — review list missing --scope → exit 2
// ---------------------------------------------------------------------------

#[test]
fn review_list__missing_scope__exit_2() {
    // T203 F37: soft-resolve fail_usage (runtime exit 2), not clap "required arguments".
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("review")
        .arg("list")
        .output()
        .expect("review list missing scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --scope must exit 2 (fail_usage); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("required arguments were not provided"),
        "must not be clap required-argument text: {stderr}"
    );
    assert!(
        stderr.contains("--scope") || stderr.contains("scope resolve"),
        "fail_usage template expected: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC3b — erasure request missing --scope → exit 2
// ---------------------------------------------------------------------------

#[test]
fn erasure_request__missing_scope__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("erasure")
        .arg("request")
        .arg("--id")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .output()
        .expect("erasure request missing scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --scope must exit 2 (clap USAGE); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// (3) / AC4 — POLICY_DENIED exit 3 + details.hint non-empty
// ---------------------------------------------------------------------------

#[test]
fn policy_check__deny__exit_3_details_hint() {
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
        .arg(SAMPLE_SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check deny");

    assert_eq!(
        out.status.code(),
        Some(3),
        "deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("must emit ApiError JSON on stdout; got: {stdout} ({e})"));
    assert_eq!(
        v.get("code").and_then(|c| c.as_str()),
        Some("POLICY_DENIED"),
        "deny envelope code; got {v}"
    );
    let hint = v
        .pointer("/details/hint")
        .and_then(|h| h.as_str())
        .unwrap_or("");
    assert!(
        !hint.is_empty(),
        "details.hint must be non-empty string; got {v}"
    );
    // T210: bootstrap is primary remediation; show remains secondary in the same string.
    assert!(
        hint.contains("bootstrap"),
        "hint must mention policy bootstrap; got {hint}"
    );
}

// ---------------------------------------------------------------------------
// (4a) / AC8 — graph feature-off exit 2 + FEATURE_UNAVAILABLE
// ---------------------------------------------------------------------------

#[cfg(not(feature = "graph"))]
#[test]
fn graph_update__feature_off__exit_2_feature_unavailable() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("graph")
        .arg("update")
        .output()
        .expect("graph update stub");

    assert_eq!(
        out.status.code(),
        Some(2),
        "graph feature-off must exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("FEATURE_UNAVAILABLE"),
        "must prefix FEATURE_UNAVAILABLE; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// (4b) — vault key missing → exit 1 + VAULT_KEY_* family
// ---------------------------------------------------------------------------

#[test]
fn recall__missing_key__exit_1_vault_key_family() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // T199 helper: ambient strip + no key inject + --no-project-context (dotenv hygiene).
    let out = common::hermetic_bin_no_key()
        .arg("--vault-path")
        .arg(&vault)
        .arg("recall")
        .arg("anything")
        .output()
        .expect("recall missing key");

    assert_eq!(
        out.status.code(),
        Some(1),
        "vault key missing must exit 1; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("VAULT_KEY_MISSING")
            || stderr.contains("VAULT_KEY_")
            || stderr.contains("Vault key missing")
            || stderr.contains("VAULT_LOCKED"),
        "expected VAULT_KEY_* / locked family; got {stderr}"
    );
}

// ---------------------------------------------------------------------------
// (5) — success exit 0 sample (policy show empty grants)
// ---------------------------------------------------------------------------

#[test]
fn policy_show__with_scope_empty_vault__exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(SAMPLE_SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show with scope");

    assert_eq!(
        out.status.code(),
        Some(0),
        "policy show with scope on empty vault must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("must emit grants JSON; got: {stdout} ({e})"));
    assert!(
        v.get("grants")
            .and_then(|g| g.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "fresh vault grants should be []; got {v}"
    );
}

// ---------------------------------------------------------------------------
// (6) — INVALID_PAYLOAD exit 6 (unknown capability)
// ---------------------------------------------------------------------------

#[test]
fn policy_check__unknown_capability__exit_6_invalid_payload() {
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
        .arg("NotARealCapability")
        .arg("--scope")
        .arg(SAMPLE_SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check unknown capability");

    assert_eq!(
        out.status.code(),
        Some(6),
        "unknown capability must exit 6; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("INVALID_PAYLOAD"),
        "expected INVALID_PAYLOAD envelope; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC11 / T226 AC3 — help: soft-default for show/check; clap-required for erasure
// ---------------------------------------------------------------------------

/// Required clap long options appear in the Usage line as `--scope <SCOPE>` without
/// surrounding `[]`. Optional ones appear as `[--scope <SCOPE>]`.
/// Retained for erasure (T226 F12/M3) — do not use for policy show/check.
fn assert_help_scope_required(stdout: &str, cmd: &str) {
    let usage_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Usage:"))
        .unwrap_or("");
    assert!(
        !usage_line.is_empty(),
        "{cmd} help must include Usage line; got: {stdout}"
    );
    assert!(
        usage_line.contains("--scope")
            && !usage_line.contains("[--scope")
            && !usage_line.contains("[--scope <SCOPE>]"),
        "{cmd} Usage must require --scope (not optional [--scope]); usage={usage_line}"
    );
    assert!(
        stdout.contains("--scope <SCOPE>") || stdout.contains("--scope <scope>"),
        "{cmd} help must document --scope <SCOPE>; got: {stdout}"
    );
}

#[test]
fn policy_show__help__scope_optional_soft_default() {
    // T226 AC3/O4: --scope optional; soft-resolve when authoritative.
    // Clap required form is `Usage: … [OPTIONS] --scope <SCOPE>`; optional is
    // `Usage: … [OPTIONS]` only (flag documented under Options, not Usage).
    let out = common::hermetic_bin()
        .arg("policy")
        .arg("show")
        .arg("--help")
        .output()
        .expect("policy show --help");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--scope"),
        "policy show help must still document --scope; got: {stdout}"
    );
    assert!(
        stdout.contains("soft-resolves") || stdout.contains("soft-resolve"),
        "policy show help must mention soft-resolve; got: {stdout}"
    );
    let usage_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Usage:"))
        .unwrap_or("");
    assert!(
        usage_line.contains("[OPTIONS]"),
        "policy show Usage must include [OPTIONS]; usage={usage_line}"
    );
    assert!(
        !usage_line.contains("--scope"),
        "policy show Usage must not hard-require --scope (regression lock); usage={usage_line}"
    );
}

#[test]
fn policy_check__help__scope_optional_soft_default() {
    let out = common::hermetic_bin()
        .arg("policy")
        .arg("check")
        .arg("--help")
        .output()
        .expect("policy check --help");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--scope"),
        "policy check help must still document --scope; got: {stdout}"
    );
    assert!(
        stdout.contains("soft-resolves") || stdout.contains("soft-resolve"),
        "policy check help must mention soft-resolve; got: {stdout}"
    );
    let usage_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Usage:"))
        .unwrap_or("");
    assert!(
        usage_line.contains("[OPTIONS]"),
        "policy check Usage must include [OPTIONS]; usage={usage_line}"
    );
    assert!(
        !usage_line.contains("--scope"),
        "policy check Usage must not hard-require --scope (regression lock); usage={usage_line}"
    );
}

#[test]
fn review_list__help__scope_optional_soft_default() {
    // T203: --scope is optional; soft-resolve or fail_usage (not clap-required).
    let out = common::hermetic_bin()
        .arg("review")
        .arg("list")
        .arg("--help")
        .output()
        .expect("review list --help");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--scope"),
        "review list help must still document --scope; got: {stdout}"
    );
    // Must not claim required-only Usage line (soft-default).
    let usage_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Usage:"))
        .unwrap_or("");
    assert!(
        !usage_line.contains("--scope") || usage_line.contains("[OPTIONS]"),
        "review list Usage should not hard-require --scope; usage={usage_line}"
    );
}

#[test]
fn erasure_request__help__scope_required() {
    let out = common::hermetic_bin()
        .arg("erasure")
        .arg("request")
        .arg("--help")
        .output()
        .expect("erasure request --help");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_help_scope_required(&stdout, "erasure request");
}

// ---------------------------------------------------------------------------
// T226 AC7 — malformed explicit --scope → exit 6 class (fail_cp)
// ---------------------------------------------------------------------------

#[test]
fn policy_show__malformed_explicit_scope__exit_6_class() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg("not-a-key")
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show malformed scope");

    assert_eq!(
        out.status.code(),
        Some(6),
        "malformed explicit --scope must exit 6; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    // fail_cp Json: ApiError on stdout with INVALID_PAYLOAD (control-plane class).
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("INVALID_PAYLOAD") || stdout.contains("unparseable"),
        "AC7 must surface control-plane payload error; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// T226 AC8 — policy check missing --capability stays clap-required
// ---------------------------------------------------------------------------

#[test]
fn policy_check__missing_capability__clap_required_exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Scope may be present or omit; capability stays clap-required (opposite of AC1/AC2).
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--scope")
        .arg(SAMPLE_SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check missing capability");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --capability must exit 2 (clap USAGE); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("required arguments were not provided"),
        "expected clap English for missing --capability; got: {stderr}"
    );
}
