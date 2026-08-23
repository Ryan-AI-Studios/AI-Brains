//! T285 AC12 — hermetic CLI: tagged pin is recall/search hit #1 vs live chrome dumps.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::path::Path;
use tempfile::tempdir;

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn pin_tagged(vault: &Path, content: &str, tag: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .arg("--tag")
        .arg(tag)
        .assert()
        .success();
}

fn pin(vault: &Path, content: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn first_needle_line<'a>(stdout: &'a str, needle: &str) -> &'a str {
    stdout.lines().find(|l| l.contains(needle)).unwrap_or("")
}

fn seed_review_dumps(vault: &Path, needle: &str) {
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        pin(
            vault,
            &format!("# Review of Track 285: dump {i}\n{repeats}review body"),
        );
    }
}

#[test]
fn recall_and_search__tagged_pin_vs_review_dumps__hit_one__ac12() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T285-rank-needle-{}", uuid::Uuid::new_v4());
    seed_review_dumps(&vault, &needle);
    pin_tagged(
        &vault,
        &format!("DECISION: {needle} we chose rank v2"),
        "t285",
    );

    for cmd in ["recall", "search"] {
        let pretty = common::hermetic_cmd(&vault)
            .arg(cmd)
            .arg(&needle)
            .arg("--limit")
            .arg("5")
            .arg("--format")
            .arg("pretty")
            .arg("--no-bridge")
            .output()
            .unwrap_or_else(|_| panic!("{cmd} pretty"));
        assert_eq!(
            pretty.status.code(),
            Some(0),
            "AC12: {cmd} must exit 0; stderr={}",
            String::from_utf8_lossy(&pretty.stderr)
        );
        let pretty_out = String::from_utf8_lossy(&pretty.stdout);
        let first_hit = first_needle_line(&pretty_out, &needle);
        assert!(
            first_hit.contains("DECISION:"),
            "AC12: {cmd} pretty hit #1 must be the pin; first_hit={first_hit:?} stdout={pretty_out}"
        );
        assert!(
            !first_hit.contains("# AI-Brains Session Onboarding")
                && !first_hit.contains("# Review of Track")
                && !first_hit.contains("## Objective"),
            "AC12: {cmd} pretty hit #1 must not be session chrome; first_hit={first_hit:?}"
        );
    }

    let json_out = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg(&needle)
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall json");
    assert_eq!(json_out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(&stdout);
    let json: serde_json::Value = serde_json::from_str(line).expect("json");
    let first = json
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let raw = first.get("content").and_then(|c| c.as_str()).unwrap_or("");
    assert!(
        raw.contains("TAGS:") && raw.contains("DECISION:"),
        "AC11: JSON content stays raw envelope; got {raw}"
    );
    assert!(
        first.get("is_session").is_none()
            && first.get("pin_kind").is_none()
            && first.get("envelope").is_none(),
        "AC11: no new required keys; keys={:?}",
        first.as_object().map(|m| m.keys().collect::<Vec<_>>())
    );
}

#[test]
fn sync_query__tagged_pin_vs_review_dumps__vault_top_is_pin__ac13() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T285-rank-needle-{}", uuid::Uuid::new_v4());
    seed_review_dumps(&vault, &needle);
    pin_tagged(
        &vault,
        &format!("DECISION: {needle} we chose rank v2"),
        "t285",
    );

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("sync")
        .arg("query")
        .arg(&needle)
        .arg("--no-bridge")
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("sync query");
    assert!(
        out.status.success(),
        "AC13: sync query must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first_hit = first_needle_line(&stdout, &needle);
    assert!(
        first_hit.contains("DECISION:"),
        "AC13: vault section top must be the pin; first_hit={first_hit:?} stdout={stdout}"
    );
    assert!(
        !first_hit.contains("# Review of Track"),
        "AC13: vault top must not be review chrome; first_hit={first_hit:?}"
    );
}
