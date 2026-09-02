//! T276 AC4 / AC5 / AC9 / AC15 — hermetic `--global` pretty tags + JSON freeze.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const LEFTOVER_PROJECT: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const LEFTOVER_SESSION: &str = "22222222-2222-2222-2222-222222222222";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn pin_on(vault: &Path, project_id: &str, session_id: &str, content: &str) {
    common::hermetic_cmd_with_ids(vault, project_id, session_id)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn seed_two_project_vault(vault: &Path, needle: &str) {
    init_vault(vault);
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        pin_on(
            vault,
            LEFTOVER_PROJECT,
            LEFTOVER_SESSION,
            &format!("## Objective\n{repeats}review dump {i} of the leftover remediator"),
        );
    }
    pin_on(
        vault,
        common::DEFAULT_PROJECT,
        common::DEFAULT_SESSION,
        &format!("DECISION: {needle} owner unique pin we must surface"),
    );
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

fn first_owner_hit_line(stdout: &str, needle: &str) -> String {
    stdout
        .lines()
        .find(|l| l.contains(needle) && l.contains("owner unique pin"))
        .unwrap_or("")
        .to_string()
}

#[test]
fn recall__global_pretty__tags_project() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    let needle = format!("T276-cli-tag-{}", uuid::Uuid::new_v4());
    seed_two_project_vault(&vault, &needle);

    let pretty = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg(&needle)
        .arg("--global")
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall pretty global");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "AC4: recall --global must exit 0; stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    let owner_line = first_owner_hit_line(&pretty_out, &needle);
    assert!(
        !owner_line.is_empty(),
        "AC4: owner pin line must be present; stdout={pretty_out}"
    );
    let after_tag = owner_line
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .map(|(_, rest)| rest);
    assert!(
        after_tag.is_some_and(|rest| {
            rest.starts_with(" [session=")
                || rest.starts_with(" [score=")
                || rest.starts_with(" [rank=#")
        }),
        "AC4: leading project tag, one space, then [session= / [score= / [rank=#; owner_line={owner_line:?} stdout={pretty_out}"
    );
}

#[test]
fn recall__scoped_pretty__no_global_tag() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    let needle = format!("T276-cli-scoped-{}", uuid::Uuid::new_v4());
    init_vault(&vault);
    pin_on(
        &vault,
        common::DEFAULT_PROJECT,
        common::DEFAULT_SESSION,
        &format!("DECISION: {needle} scoped only"),
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
        .expect("recall pretty scoped");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "AC9: scoped recall must exit 0; stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    let hit = pretty_out
        .lines()
        .find(|l| l.contains(&needle))
        .unwrap_or("");
    assert!(
        !hit.is_empty(),
        "AC9: scoped hit must be present; stdout={pretty_out}"
    );
    let owner8 = &common::DEFAULT_PROJECT[..8];
    assert!(
        !hit.contains(&format!("[{owner8}]")),
        "AC9: project-scoped pretty must not tag hits with [8hex]; hit={hit:?} stdout={pretty_out}"
    );
}

#[test]
fn recall__global_json__no_project_id_key() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    let needle = format!("T276-cli-json-{}", uuid::Uuid::new_v4());
    seed_two_project_vault(&vault, &needle);

    let json_out = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg(&needle)
        .arg("--global")
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall json global");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "AC5: stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    let json = parse_last_json_object(&stdout);
    let results = json
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        !results.is_empty(),
        "AC5: JSON results must be non-empty; stdout={stdout}"
    );
    for hit in &results {
        assert!(
            hit.get("project_id").is_none(),
            "AC5: RecallResult must not grow a project_id key; keys={:?}",
            hit.as_object().map(|m| m.keys().collect::<Vec<_>>())
        );
    }
}

#[test]
fn sync_query__global_pretty__tags_project() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    let needle = format!("T276-sync-tag-{}", uuid::Uuid::new_v4());
    seed_two_project_vault(&vault, &needle);

    let pretty = common::hermetic_cmd(&vault)
        .arg("sync")
        .arg("query")
        .arg(&needle)
        .arg("--global")
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("sync query pretty global");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "AC15: sync query --global must exit 0; stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    let owner_line = first_owner_hit_line(&pretty_out, &needle);
    assert!(
        !owner_line.is_empty(),
        "AC15: owner pin line must be present; stdout={pretty_out}"
    );
    let after_tag = owner_line
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .map(|(_, rest)| rest);
    assert!(
        after_tag.is_some_and(|rest| {
            rest.starts_with(" [session=")
                || rest.starts_with(" [score=")
                || rest.starts_with(" [rank=#")
        }),
        "AC15: sync pretty --global shares print_pretty_hits tags; owner_line={owner_line:?} stdout={pretty_out}"
    );
}
