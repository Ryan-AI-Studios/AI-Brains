//! T274 AC14 / AC15 — hermetic CLI: unique pin needle is recall / sync-query hit #1.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
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

fn pin(vault: &Path, content: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn parse_last_json_object(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; line={line}; full_stdout={stdout}");
    })
}

fn recall_json_contents(v: &Value) -> Vec<String> {
    v.get("results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|hit| {
                    hit.get("content")
                        .and_then(|c| c.as_str())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn recall__unique_pin_needle__hit_one() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T274-rank-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        pin(
            &vault,
            &format!("## Objective\n{repeats}review dump {i} of the ranking remediator"),
        );
    }
    pin(
        &vault,
        &format!("DECISION: {needle} we chose the ranking remediator"),
    );

    let pretty = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg(&needle)
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall pretty");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "AC14: recall must exit 0; stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    let first_hit = pretty_out
        .lines()
        .find(|l| l.contains(&needle))
        .unwrap_or("");
    assert!(
        first_hit.contains("DECISION:"),
        "AC14: pretty hit #1 must be the pin; first_hit={first_hit:?} stdout={pretty_out}"
    );
    assert!(
        !first_hit.contains("## Objective"),
        "AC14: pretty hit #1 must not be session chrome; first_hit={first_hit:?} stdout={pretty_out}"
    );

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
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    let json = parse_last_json_object(&stdout);
    let hits = recall_json_contents(&json);
    assert!(
        !hits.is_empty(),
        "AC14: JSON results must be non-empty; stdout={stdout}"
    );
    let first_raw = hits[0].as_str();
    let first = first_raw.strip_prefix("ASSISTANT: ").unwrap_or(first_raw);
    assert!(
        first.starts_with("DECISION:"),
        "AC14: JSON hit #1 must be the pin (leading DECISION: after ASSISTANT: strip); hits={hits:?}"
    );
    assert!(
        !first.starts_with("## Objective"),
        "AC14: JSON hit #1 must not be session chrome; hits={hits:?}"
    );
    let obj = json
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(Value::Null);
    assert!(obj.get("memory_id").is_some());
    assert!(obj.get("content").is_some());
    assert!(obj.get("source").is_some());
    assert!(
        obj.get("is_session").is_none() && obj.get("pin_kind").is_none(),
        "AC13: no new required/session keys; keys={:?}",
        obj.as_object().map(|m| m.keys().collect::<Vec<_>>())
    );
}

#[test]
fn sync_query__unique_pin_needle__vault_top_is_pin() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T274-rank-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        pin(
            &vault,
            &format!("## Objective\n{repeats}review dump {i} of the ranking remediator"),
        );
    }
    pin(
        &vault,
        &format!("DECISION: {needle} we chose the ranking remediator"),
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
        "AC15: sync query must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first_hit = stdout.lines().find(|l| l.contains(&needle)).unwrap_or("");
    assert!(
        first_hit.contains("DECISION:"),
        "AC15: vault section top must be the pin; first_hit={first_hit:?} stdout={stdout}"
    );
}
