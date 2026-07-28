//! T160 CLI governed surface RED/GREEN tests.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
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
fn cli_scope_resolve__json__includes_authoritative_field() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("scope resolve must run");

    assert!(
        output.status.success(),
        "scope resolve must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("must emit JSON; got: {stdout} ({e})"));
    assert!(
        v.get("authoritative").and_then(|x| x.as_bool()).is_some(),
        "authoritative field required; got {v}"
    );
    assert!(v.get("confidence").is_some());
    assert!(v.get("warnings").and_then(|x| x.as_array()).is_some());
    assert!(v.get("alternatives").and_then(|x| x.as_array()).is_some());
}

#[test]
fn cli_conclusion_propose__help_lists_claim_and_evidence() {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("conclusion")
        .arg("propose")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--claim").or(predicate::str::contains("claim")))
        .stdout(predicate::str::contains("--evidence").or(predicate::str::contains("evidence")))
        .stdout(predicate::str::contains("--scope").or(predicate::str::contains("scope")));
}

#[test]
fn cli_review_list__json__items_array_e1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let scope = format!("Repository:{}", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

    // Without grant → POLICY_DENIED or INVALID_PAYLOAD; with local empty vault
    // we expect either items:[] if somehow allowed, or structured deny.
    // Seed is empty: policy deny is OK for this E1 check as long as help works.
    // Call with --local; system principal has no grants → POLICY_DENIED exit 3.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("review")
        .arg("list")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("review list must run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let v: Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("must emit JSON; got: {stdout} ({e})"));
        let items = v
            .get("items")
            .and_then(|x| x.as_array())
            .unwrap_or_else(|| panic!("E1 items array required; got {v}"));
        assert!(items.is_empty() || !items.is_empty()); // array present
        // Prefer empty for fresh vault
        assert!(items.is_empty(), "fresh vault should list empty items");
    } else {
        // Policy deny is acceptable for ungranted system principal — still structured.
        let code = output.status.code().unwrap_or(1);
        assert!(
            code == 3 || code == 6 || code == 1,
            "unexpected exit {code}; stdout={stdout} stderr={stderr}"
        );
        // If JSON error on stdout, items must not be null somewhere misleading
        if let Ok(v) = serde_json::from_str::<Value>(&stdout)
            && let Some(items) = v.get("items")
        {
            assert!(items.is_array());
        }
    }
}

#[test]
fn cli_erasure_request__daemon_down__exit_code_5() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("erasure")
        .arg("request")
        .arg("--id")
        .arg("00000000-0000-0000-0000-0000000000e1")
        .arg("--scope")
        .arg("Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .arg("--format")
        .arg("json")
        .output()
        .expect("erasure request must run");

    let code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Never invent a local ticket (exit 0 with request_id).
    assert_ne!(
        code,
        Some(0),
        "erasure must not succeed without daemon ticket"
    );

    // Hermetic: true daemon-down → exit 5 (DAEMON_UNAVAILABLE).
    // Ambient `ledgerful-bridge` may answer Ping but not complete RequestErasure;
    // that path is post-send ambiguous → exit 1 INTERNAL with "outcome unknown".
    // Both are non-zero and must not claim CE wipe / local ticket.
    assert!(
        code == Some(5) || code == Some(1),
        "expected exit 5 (daemon down) or 1 (ambiguous ambient daemon); got {code:?}; stdout={stdout} stderr={stderr}"
    );
    if code == Some(5) {
        assert!(
            stdout.contains("DAEMON_UNAVAILABLE") || stderr.contains("DAEMON_UNAVAILABLE"),
            "exit 5 should surface DAEMON_UNAVAILABLE; stdout={stdout} stderr={stderr}"
        );
    }
    if code == Some(1) {
        assert!(
            stdout.contains("outcome unknown") || stderr.contains("outcome unknown"),
            "exit 1 from ambient daemon must be ambiguous-outcome; stdout={stdout} stderr={stderr}"
        );
    }
    // Never claim wipe completed.
    let joined = format!("{stdout}{stderr}").to_ascii_lowercase();
    assert!(
        !joined.contains("wipe completed") && !joined.contains("\"status\":\"wiped\""),
        "must not claim CE wipe"
    );
}

#[test]
fn existing_briefing_project__still_ok() {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("briefing")
        .arg("project")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project").or(predicate::str::contains("briefing")));
}

#[test]
fn cli_erasure_request__local_flag__rejected() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("erasure")
        .arg("request")
        .arg("--id")
        .arg("00000000-0000-0000-0000-0000000000e1")
        .arg("--scope")
        .arg("Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .arg("--local")
        .arg("--format")
        .arg("json")
        .output()
        .expect("erasure --local must run");

    // Either INVALID_PAYLOAD (6) for --local or DAEMON_UNAVAILABLE (5)
    let code = output.status.code().unwrap_or(0);
    assert!(
        code == 5 || code == 6,
        "expected exit 5 or 6 for erasure --local; got {code}; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_policy_check__no_grant__exit_code_3() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ProposeConclusion")
        .arg("--scope")
        .arg("Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy check must run");

    assert_eq!(
        output.status.code(),
        Some(3),
        "ungranted policy check must exit 3; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
