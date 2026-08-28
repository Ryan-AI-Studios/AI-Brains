//! T312 AC12/AC13/AC10/AC11 — hermetic CLI: authority-OR pin beats prose dumps.
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

fn seed_and_miss_fixture(vault: &Path) -> String {
    let uuid = uuid::Uuid::new_v4();
    for i in 0..15 {
        pin(
            vault,
            &format!("Here's the assessment. dump {i}\nt312or backend repeated body pad {i}"),
        );
    }
    // Also seed a chrome Objective dump that AND-hits.
    pin(
        vault,
        &format!(
            "## Objective\nt312or backend chrome dump {}",
            "x".repeat(200)
        ),
    );
    let pin_body = format!("DECISION: t312or {uuid} sqlite graph");
    pin_tagged(vault, &pin_body, "t312");
    uuid.to_string()
}

fn assert_f42_pretty_hit_one(vault: &Path, cmd: &str, uuid: &str) {
    let pretty = common::hermetic_cmd(vault)
        .arg(cmd)
        .arg("t312or backend")
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
    let first_hit = first_needle_line(&pretty_out, "t312or");
    assert!(
        first_hit.contains("DECISION:") && first_hit.contains(uuid),
        "AC12: {cmd} pretty hit #1 must be the pin; first_hit={first_hit:?} stdout={pretty_out}"
    );
    assert!(
        !first_hit.contains("## Objective")
            && !first_hit.contains("Here's the assessment")
            && !first_hit.contains("# Review of Track"),
        "AC12: {cmd} pretty hit #1 must not be dump/chrome; first_hit={first_hit:?}"
    );
}

#[test]
fn recall__f42_and_miss_or_fill__hit_one__ac12() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let uuid = seed_and_miss_fixture(&vault);
    assert_f42_pretty_hit_one(&vault, "recall", &uuid);

    let json_out = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg("t312or backend")
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
            && first.get("verbose_dump").is_none(),
        "AC11: no new required keys; keys={:?}",
        first.as_object().map(|m| m.keys().collect::<Vec<_>>())
    );
}

#[test]
fn search__f42_and_miss_or_fill__hit_one__ac12() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let uuid = seed_and_miss_fixture(&vault);
    assert_f42_pretty_hit_one(&vault, "search", &uuid);
}

#[test]
fn sync_query__f42_and_miss__vault_top_is_pin__ac13() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let uuid = seed_and_miss_fixture(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("sync")
        .arg("query")
        .arg("t312or backend")
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("sync query");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC13: sync query must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first_hit = first_needle_line(&stdout, "t312or");
    assert!(
        first_hit.contains("DECISION:") && first_hit.contains(&uuid),
        "AC13: vault section top must be the pin; first_hit={first_hit:?} stdout={stdout}"
    );
}

#[test]
fn forget_match__still_finds_verbose_other_dump__ac10() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let marker = format!("t312forget{}", uuid::Uuid::new_v4().as_simple());
    let dump = format!(
        "All non-destructive commands tested. {marker} {}",
        "pad ".repeat(250)
    );
    pin(&vault, &dump);

    let out = common::hermetic_cmd(&vault)
        .arg("forget")
        .arg("--match")
        .arg(&marker)
        .arg("--dry-run")
        .output()
        .expect("forget --match");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC10: forget --match --dry-run must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&marker) || stdout.contains("[dry-run]"),
        "AC10: forget --match must find verbose-Other dump (unfiltered); stdout={stdout}"
    );
}
