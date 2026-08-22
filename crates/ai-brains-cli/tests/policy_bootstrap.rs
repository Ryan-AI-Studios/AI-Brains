//! T210 hermetic locks — `policy bootstrap` discovery grants.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const PRINCIPAL: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const PROJECT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SCOPE: &str = "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn policy_bootstrap(vault: &Path, dry_run: bool) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json");
    if dry_run {
        cmd.arg("--dry-run");
    }
    cmd
}

fn policy_check(vault: &Path, capability: &str) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg(capability)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json");
    cmd
}

/// AC1 — before bootstrap, ReadEvidence check is denied (exit 3).
#[test]
fn policy_bootstrap__before__policy_check_read_evidence_exit_3() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = policy_check(&vault, "ReadEvidence")
        .output()
        .expect("policy check");

    assert_eq!(
        out.status.code(),
        Some(3),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["code"], "POLICY_DENIED");
}

/// AC2 — first bootstrap registers principal and issues three discovery grants.
#[test]
fn policy_bootstrap__first_run__registers_and_issues_three() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = policy_bootstrap(&vault, false)
        .output()
        .expect("policy bootstrap");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["principal_id"], PRINCIPAL);
    assert_eq!(v["scope"], SCOPE);
    assert_eq!(v["registered"], "registered");
    assert_eq!(v["dry_run"], false);

    let grants = v["grants"].as_array().expect("grants array");
    assert_eq!(grants.len(), 3);

    // F30 — sorted alphabetically by capability name.
    let names: Vec<&str> = grants
        .iter()
        .map(|g| g["capability"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["ReadConclusions", "ReadDecisions", "ReadEvidence"]
    );

    for g in grants {
        assert_eq!(g["status"], "issued", "grant={g}");
        assert!(
            g["grant_id"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "issued must include grant_id; got {g}"
        );
    }
}

/// AC3 — after bootstrap, three discovery checks are allowed (exit 0).
#[test]
fn policy_bootstrap__after__three_checks_allowed() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let boot = policy_bootstrap(&vault, false).output().expect("bootstrap");
    assert_eq!(boot.status.code(), Some(0), "bootstrap failed");

    for cap in ["ReadEvidence", "ReadConclusions", "ReadDecisions"] {
        let out = policy_check(&vault, cap).output().expect("check");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{cap} denied; stderr={} stdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        let v: Value = serde_json::from_slice(&out.stdout).expect("json");
        assert_eq!(v["allowed"], true, "{cap}: {v}");
    }
}

/// AC4 — after bootstrap, source list + review list exit 0 (empty OK).
#[test]
fn policy_bootstrap__after__source_and_review_list_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let boot = policy_bootstrap(&vault, false).output().expect("bootstrap");
    assert_eq!(boot.status.code(), Some(0), "bootstrap failed");

    let source = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("source list");
    assert_eq!(
        source.status.code(),
        Some(0),
        "source list; stderr={} stdout={}",
        String::from_utf8_lossy(&source.stderr),
        String::from_utf8_lossy(&source.stdout)
    );
    let sv: Value = serde_json::from_slice(&source.stdout).expect("source json");
    assert!(sv["items"].as_array().expect("items").is_empty());

    let review = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("review")
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("review list");
    assert_eq!(
        review.status.code(),
        Some(0),
        "review list; stderr={} stdout={}",
        String::from_utf8_lossy(&review.stderr),
        String::from_utf8_lossy(&review.stdout)
    );
}

/// AC5 — second bootstrap is no-op: already_present + registered already.
#[test]
fn policy_bootstrap__second_run__already_present() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let first = policy_bootstrap(&vault, false)
        .output()
        .expect("first bootstrap");
    assert_eq!(first.status.code(), Some(0));

    let second = policy_bootstrap(&vault, false)
        .output()
        .expect("second bootstrap");
    assert_eq!(
        second.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&second.stderr),
        String::from_utf8_lossy(&second.stdout)
    );
    let v: Value = serde_json::from_slice(&second.stdout).expect("json");
    assert_eq!(v["registered"], "already");
    assert_eq!(v["dry_run"], false);
    let grants = v["grants"].as_array().expect("grants");
    assert_eq!(grants.len(), 3);
    for g in grants {
        assert_eq!(g["status"], "already_present", "grant={g}");
        assert!(
            g.get("grant_id").is_none() || g["grant_id"].is_null(),
            "already_present must omit grant_id; got {g}"
        );
    }
}

/// AC6 / F9 — dry-run reports plan and appends zero events (no register, no issue).
#[test]
fn policy_bootstrap__dry_run__no_grants() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = policy_bootstrap(&vault, true)
        .output()
        .expect("dry-run bootstrap");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["registered"], "would_register");
    let grants = v["grants"].as_array().expect("grants");
    assert_eq!(grants.len(), 3);
    for g in grants {
        assert_eq!(g["status"], "would_issue", "grant={g}");
    }

    // Grants still absent — check still deny.
    let check = policy_check(&vault, "ReadEvidence")
        .output()
        .expect("check after dry-run");
    assert_eq!(
        check.status.code(),
        Some(3),
        "dry-run must not issue grants; stdout={}",
        String::from_utf8_lossy(&check.stdout)
    );

    // F9 zero-append lock for register: a second dry-run must still report
    // would_register (get_principal still None). A buggy dry-run that called
    // register_principal would flip this to "already" while still printing
    // would_register on the first response (status computed before mutation).
    let second_dry = policy_bootstrap(&vault, true)
        .output()
        .expect("second dry-run");
    assert_eq!(second_dry.status.code(), Some(0));
    let v2: Value = serde_json::from_slice(&second_dry.stdout).expect("json");
    assert_eq!(
        v2["registered"], "would_register",
        "dry-run must not register principal; second dry-run must still would_register; got {v2}"
    );

    // And the first *real* bootstrap after dry-run must still register (not already).
    let real = policy_bootstrap(&vault, false)
        .output()
        .expect("real bootstrap after dry-run");
    assert_eq!(real.status.code(), Some(0));
    let v3: Value = serde_json::from_slice(&real.stdout).expect("json");
    assert_eq!(
        v3["registered"], "registered",
        "first real bootstrap after dry-run must register; got {v3}"
    );
}

// ---------------------------------------------------------------------------
// T241 AC4–AC6 — cold-start discoverability around show/check
// ---------------------------------------------------------------------------

/// T241 AC4 — empty show human contains short SOOT; exit 0.
#[test]
fn policy_show__empty_grants__human_contains_bootstrap_soot() {
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
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("human")
        .output()
        .expect("policy show human");

    assert_eq!(
        out.status.code(),
        Some(0),
        "empty show must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(none)"),
        "human empty must show (none); got {stdout}"
    );
    assert!(
        stdout.contains("policy bootstrap"),
        "human empty must include short SOOT; got {stdout}"
    );
    assert!(
        stdout.contains("policy bootstrap --dry-run")
            || stdout.contains("`ai-brains policy bootstrap"),
        "short SOOT dry-run then apply; got {stdout}"
    );
}

/// T241 AC5 — empty show JSON has grants:[] + next_step; non-empty omits next_step.
#[test]
fn policy_show__empty_and_nonempty__json_next_step() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let empty = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show empty json");
    assert_eq!(empty.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&empty.stdout).expect("json");
    let grants = v["grants"].as_array().expect("grants array");
    assert!(grants.is_empty(), "empty grants:[] expected; got {v}");
    assert!(
        !v["grants"].is_null(),
        "grants must be [] not null; got {v}"
    );
    let next = v["next_step"].as_str().unwrap_or("");
    assert!(
        next.contains("policy bootstrap"),
        "empty next_step must name bootstrap; got {v}"
    );

    let boot = policy_bootstrap(&vault, false).output().expect("bootstrap");
    assert_eq!(boot.status.code(), Some(0));

    let filled = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show filled json");
    assert_eq!(filled.status.code(), Some(0));
    let v2: Value = serde_json::from_slice(&filled.stdout).expect("json");
    let grants2 = v2["grants"].as_array().expect("grants");
    assert!(
        !grants2.is_empty(),
        "after bootstrap grants non-empty; {v2}"
    );
    assert!(
        v2.get("next_step").is_none(),
        "non-empty must omit next_step key (not null); got {v2}"
    );
    let raw = String::from_utf8_lossy(&filled.stdout);
    assert!(
        !raw.contains("next_step"),
        "serialized JSON must omit next_step entirely; got {raw}"
    );
}

/// T241 CX1 P2 — explicit empty `--capability` is INVALID_PAYLOAD (exit 6), not usage catalog.
#[test]
fn policy_check__empty_capability_string__invalid_payload_not_usage() {
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
        .arg("")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check empty capability");

    assert_eq!(
        out.status.code(),
        Some(6),
        "empty capability must be INVALID_PAYLOAD exit 6; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("INVALID_PAYLOAD") || combined.contains("unknown capability"),
        "expected INVALID_PAYLOAD/unknown capability; got {combined}"
    );
    assert!(
        !combined.contains("required arguments were not provided"),
        "must not use clap required-arg English; got {combined}"
    );
}

/// T241 AC6/F30 — policy check without --capability → exit 2 + discovery catalog; no clap text.
#[test]
fn policy_check__missing_capability__exit_2_catalog_no_clap_required() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check no capability");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing --capability must exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("required arguments were not provided"),
        "must not be clap required-arg text: {stderr}"
    );
    assert!(
        stderr.contains("--capability is required") || stderr.contains("Valid capabilities"),
        "catalog header expected: {stderr}"
    );
    for name in ["ReadEvidence", "ReadConclusions", "ReadDecisions"] {
        assert!(
            stderr.contains(name),
            "catalog must list discovery cap {name}; got {stderr}"
        );
    }
}

/// T210 AC7 / T280 AC5 — deny details.hint names dry-run bootstrap and omits `--scope …`.
#[test]
fn policy_bootstrap__deny_hint__contains_bootstrap() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = policy_check(&vault, "ReadEvidence")
        .output()
        .expect("deny check");
    assert_eq!(out.status.code(), Some(3));
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    let hint = v
        .pointer("/details/hint")
        .and_then(|h| h.as_str())
        .unwrap_or("");
    assert!(
        !hint.is_empty() && hint.contains("policy bootstrap") && hint.contains("--dry-run"),
        "hint must name dry-run bootstrap; got {hint:?}"
    );
    assert!(
        hint.contains("omit --scope") || hint.contains("authoritative"),
        "hint must omit required --scope; got {hint:?}"
    );
    assert!(
        !hint.contains("--scope …"),
        "hint must not require --scope ellipsis; got {hint:?}"
    );
}

/// AC8 — omit --scope with --no-project-context → fail_usage exit 2.
#[test]
fn policy_bootstrap__no_scope_no_context__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic_bin strips AI_BRAINS_PROJECT_ID; --no-project-context keeps it unset.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("bootstrap no scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "soft-resolve fail must exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("scope") || stderr.contains("--scope"),
        "usage message should mention scope; got {stderr}"
    );
}

/// F2/F3 — bootstrap never issues dangerous caps (Erase / Approve*).
#[test]
fn policy_bootstrap__after__dangerous_caps_still_denied() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let boot = policy_bootstrap(&vault, false).output().expect("bootstrap");
    assert_eq!(boot.status.code(), Some(0));

    for cap in ["Erase", "ApproveConclusion", "ApproveDecision", "Export"] {
        let out = policy_check(&vault, cap).output().expect("check");
        assert_eq!(
            out.status.code(),
            Some(3),
            "{cap} must remain denied after discovery bootstrap; stdout={}",
            String::from_utf8_lossy(&out.stdout)
        );
    }

    // policy show lists only the three discovery caps.
    let show = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy show");
    assert_eq!(show.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&show.stdout).expect("json");
    let grants = v["grants"].as_array().expect("grants");
    assert_eq!(
        grants.len(),
        3,
        "expected exactly three discovery grants; got {v}"
    );
    let mut caps: Vec<&str> = grants
        .iter()
        .map(|g| g["capability"].as_str().unwrap())
        .collect();
    caps.sort();
    assert_eq!(
        caps,
        vec!["ReadConclusions", "ReadDecisions", "ReadEvidence"]
    );
    for g in grants {
        let privacy = g["privacy"].as_str().unwrap_or("");
        assert!(
            privacy.eq_ignore_ascii_case("LocalOnly") || privacy == "local_only",
            "F6 LocalOnly; got privacy={privacy:?} grant={g}"
        );
    }
}

/// T275 AC3 / F3 — denied JSON keeps `denied: true`, empty arrays (not null), bootstrap hint; exit 0.
#[test]
fn briefing_project__no_grants__json_denied_empty_arrays() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing json deny");
    assert_eq!(
        out.status.code(),
        Some(0),
        "briefing deny stays exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("briefing json");
    assert_eq!(v["denied"], true, "packet={v}");
    let decisions = v["decisions"].as_array();
    assert!(
        decisions.is_some() && !v["decisions"].is_null(),
        "decisions must be [] not null; got {v}"
    );
    assert!(
        decisions.expect("decisions array").is_empty(),
        "denied decisions must be empty; got {v}"
    );
    let conclusions = v["conclusions"].as_array();
    assert!(
        conclusions.is_some() && !v["conclusions"].is_null(),
        "conclusions must be [] not null; got {v}"
    );
    assert!(
        conclusions.expect("conclusions array").is_empty(),
        "denied conclusions must be empty; got {v}"
    );
    let hint = v["denial_hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("policy bootstrap"),
        "denial_hint must name policy bootstrap; got {v}"
    );
}

/// T275 AC4 — CLI System `policy bootstrap` then `briefing project` JSON `denied: false`.
///
/// Omit `--principal-id` so bootstrap matches `cli_principal()` System default
/// (T210 `bbbb…` Human trap; F31 / F36).
#[test]
fn policy_bootstrap__after_system__briefing_project_denied_false() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let boot = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy bootstrap");
    assert_eq!(
        boot.status.code(),
        Some(0),
        "bootstrap failed; stderr={} stdout={}",
        String::from_utf8_lossy(&boot.stderr),
        String::from_utf8_lossy(&boot.stdout)
    );

    let json = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing json");
    assert_eq!(
        json.status.code(),
        Some(0),
        "briefing json; stderr={} stdout={}",
        String::from_utf8_lossy(&json.stderr),
        String::from_utf8_lossy(&json.stdout)
    );
    let v: Value = serde_json::from_slice(&json.stdout).expect("briefing json");
    assert_eq!(v["denied"], false, "packet={v}");

    let human = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing human");
    assert_eq!(
        human.status.code(),
        Some(0),
        "briefing human; stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        !md.contains("**Denied:**"),
        "human must not show Denied after System bootstrap: {md}"
    );
}

/// T275 AC5 — same System bootstrap path: `evidence list` JSON exit 0 (`items` may be `[]`).
///
/// Omit `--principal-id` so bootstrap matches `cli_principal()` System default
/// (T210 `bbbb…` Human trap; F31 / F36).
#[test]
fn policy_bootstrap__after_system__evidence_list_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let boot = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy bootstrap");
    assert_eq!(
        boot.status.code(),
        Some(0),
        "bootstrap failed; stderr={} stdout={}",
        String::from_utf8_lossy(&boot.stderr),
        String::from_utf8_lossy(&boot.stdout)
    );

    let evidence = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("evidence list");
    assert_eq!(
        evidence.status.code(),
        Some(0),
        "evidence list; stderr={} stdout={}",
        String::from_utf8_lossy(&evidence.stderr),
        String::from_utf8_lossy(&evidence.stdout)
    );
    let v: Value = serde_json::from_slice(&evidence.stdout).expect("evidence json");
    assert!(
        v["items"].as_array().is_some(),
        "items must be an array (empty OK); got {v}"
    );
}
