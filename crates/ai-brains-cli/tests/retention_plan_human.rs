//! T248 — hermetic retention plan human CLI (pretty/json + apply format).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_core::temp_env::TempEnv;
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const RETENTION_ENV_KEYS: &[&str] = &[
    "AI_BRAINS_RETENTION_RAW_TURN_DAYS",
    "AI_BRAINS_RETENTION_EVIDENCE_DAYS",
    "AI_BRAINS_RETENTION_SECRET_DAYS",
    "AI_BRAINS_RETENTION_QUERY_TRACE_DAYS",
    "AI_BRAINS_RETENTION_REVIEW_TRACE_DAYS",
    "AI_BRAINS_RETENTION_DECISION_REVOKED_COOLDOWN_DAYS",
    "AI_BRAINS_RETENTION_ORPHAN_ENVELOPE_DAYS",
    "AI_BRAINS_RETENTION_APPLY_CE",
    "AI_BRAINS_RETENTION_APPLY_CE_ON_NIGHTLY",
];

fn isolate_retention_env() -> Vec<TempEnv> {
    RETENTION_ENV_KEYS.iter().map(TempEnv::remove).collect()
}

fn init_vault(vault: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("init")
        .assert()
        .success();
}

fn parse_pretty_json_object(stdout: &str) -> Value {
    let start = stdout
        .find('{')
        .unwrap_or_else(|| panic!("no JSON object in stdout: {stdout}"));
    serde_json::from_str(&stdout[start..]).unwrap_or_else(|e| {
        panic!("retention JSON parse failed: {e}; full_stdout={stdout}");
    })
}

#[test]
fn retention_plan__format_json__frozen_keys_empty_classes() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("plan")
        .arg("--format")
        .arg("json")
        .output()
        .expect("retention plan --format json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "json plan must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_pretty_json_object(&stdout);
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("expected object; got {v}"));
    for key in ["api_version", "horizons", "classes", "totals", "warnings"] {
        assert!(
            obj.contains_key(key),
            "frozen key {key} missing; keys={:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    assert_eq!(v["api_version"], "1");
    assert!(v["classes"].is_array());
    assert!(
        v["classes"].as_array().is_some_and(|a| a.is_empty()),
        "empty fixture must keep classes=[]; got {}",
        v["classes"]
    );
    let forbidden = ["pretty_matrix", "human", "next_step", "format"];
    for extra in forbidden {
        assert!(!obj.contains_key(extra), "unexpected additive key {extra}");
    }
}

#[test]
fn retention_plan__format_pretty__nothing_to_dispose_and_matrix() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("plan")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("retention plan --format pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "pretty plan must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Nothing to dispose"),
        "pretty empty must say nothing to dispose; got:\n{stdout}"
    );
    assert!(
        stdout.contains("raw_turn"),
        "pretty empty must list raw_turn; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Class matrix"),
        "pretty empty must print Class matrix; got:\n{stdout}"
    );
}

#[test]
fn retention_plan__format_xml__exit_2_no_stdout_json() {
    let _env = isolate_retention_env();
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("retention")
        .arg("plan")
        .arg("--format")
        .arg("xml")
        .output()
        .expect("retention plan --format xml");
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown format must be clap exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim_start().starts_with('{'),
        "xml reject must not emit JSON; stdout={stdout}"
    );
}

#[test]
fn retention_plan__pinned_memory__human_held_inventory_nothing_to_dispose() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let pin = common::hermetic_cmd(&vault)
        .arg("pin")
        .arg("T270 inventory seed unique body")
        .output()
        .expect("pin");
    assert!(
        pin.status.success(),
        "pin must succeed; stderr={}",
        String::from_utf8_lossy(&pin.stderr)
    );

    let before = common::hermetic_cmd(&vault)
        .arg("memory")
        .arg("list")
        .arg("--summary")
        .output()
        .expect("memory list --summary before plan");
    assert!(
        before.status.success(),
        "summary before plan failed; stderr={}",
        String::from_utf8_lossy(&before.stderr)
    );
    let before_stdout = String::from_utf8_lossy(&before.stdout);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("plan")
        .arg("--format")
        .arg("human")
        .output()
        .expect("retention plan --format human");
    assert_eq!(
        out.status.code(),
        Some(0),
        "pinned plan must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Nothing to dispose."),
        "pinned inventory must still say nothing to dispose; got:\n{stdout}"
    );
    assert!(
        stdout.contains("held"),
        "pinned inventory must show held; got:\n{stdout}"
    );
    let legacy_line = stdout
        .lines()
        .find(|l| l.contains("memory_legacy"))
        .unwrap_or("");
    assert!(
        legacy_line
            .split_whitespace()
            .any(|t| t != "0" && t.chars().all(|c| c.is_ascii_digit())),
        "memory_legacy count must be non-zero; line={legacy_line}"
    );
    assert!(
        !stdout.lines().any(|l| l == "Work"),
        "inventory-only must omit Work; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("next: ai-brains retention apply"),
        "inventory-only must omit next:; got:\n{stdout}"
    );

    let json_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("plan")
        .arg("--format")
        .arg("json")
        .output()
        .expect("retention plan --format json");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "pinned json plan must exit 0; stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let json_stdout = String::from_utf8_lossy(&json_out.stdout);
    let v = parse_pretty_json_object(&json_stdout);
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["mode"], "dry_run");
    let classes = v["classes"]
        .as_array()
        .unwrap_or_else(|| panic!("classes array; got {v}"));
    assert!(
        classes.iter().any(|c| c["class"] == "memory_legacy"),
        "json must include memory_legacy class; got {v}"
    );
    assert!(
        v["totals"]["would_held"].as_u64().is_some_and(|n| n >= 1),
        "would_held >= 1; got {}",
        v["totals"]
    );
    assert!(
        !json_stdout.contains("T270 inventory seed unique body"),
        "pin body leaked into JSON"
    );

    let after = common::hermetic_cmd(&vault)
        .arg("memory")
        .arg("list")
        .arg("--summary")
        .output()
        .expect("memory list --summary after plan");
    assert!(
        after.status.success(),
        "summary after plan failed; stderr={}",
        String::from_utf8_lossy(&after.stderr)
    );
    let after_stdout = String::from_utf8_lossy(&after.stdout);
    assert_eq!(
        before_stdout, after_stdout,
        "AC10: plan must not change inventory; before={before_stdout} after={after_stdout}"
    );
}

#[test]
fn retention_apply__without_confirm__invalid_payload_exit_6() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("apply")
        .output()
        .expect("retention apply without confirm");
    assert_eq!(
        out.status.code(),
        Some(6),
        "apply without --confirm must stay INVALID_PAYLOAD exit 6; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("INVALID_PAYLOAD"),
        "expected INVALID_PAYLOAD; got {combined}"
    );
}

#[test]
fn retention_apply__confirm_format_json_and_human__empty_fixture() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let json_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("apply")
        .arg("--confirm")
        .arg("--format")
        .arg("json")
        .output()
        .expect("retention apply --confirm --format json");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "empty apply json must exit 0; stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let json_stdout = String::from_utf8_lossy(&json_out.stdout);
    let v = parse_pretty_json_object(&json_stdout);
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["mode"], "apply");
    assert!(v["totals"]["candidates"].as_u64() == Some(0));

    let human_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("apply")
        .arg("--confirm")
        .arg("--format")
        .arg("human")
        .output()
        .expect("retention apply --confirm --format human");
    assert_eq!(
        human_out.status.code(),
        Some(0),
        "empty apply human must exit 0; stderr={}",
        String::from_utf8_lossy(&human_out.stderr)
    );
    let human_stdout = String::from_utf8_lossy(&human_out.stdout);
    assert!(
        human_stdout.contains("Retention apply"),
        "apply human title missing; got:\n{human_stdout}"
    );
}

#[test]
fn retention_apply__confirm_format_auto__pretty_json_not_human() {
    let _env = isolate_retention_env();
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("retention")
        .arg("apply")
        .arg("--confirm")
        .arg("--format")
        .arg("auto")
        .output()
        .expect("retention apply --confirm --format auto");
    assert_eq!(
        out.status.code(),
        Some(0),
        "apply --format auto must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("Retention apply"),
        "apply auto must not TTY-switch to human; got:\n{stdout}"
    );
    let v = parse_pretty_json_object(&stdout);
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["mode"], "apply");
}
