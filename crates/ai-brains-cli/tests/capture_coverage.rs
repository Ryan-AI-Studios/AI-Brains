//! T337 AC11 / Codex — command-level capture coverage exit contract.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use std::fs;
use tempfile::tempdir;

const CURSOR_SID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa01";

fn init_vault(vault: &std::path::Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault)
        .arg("--no-project-context")
        .arg("init")
        .assert()
        .success();
}

#[test]
fn capture_coverage__no_scope__exit_2() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("v.db");
    init_vault(&vault);
    common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn capture_coverage__cursor_deficit__exit_0() {
    let home = tempdir().expect("home");
    let vault_dir = tempdir().expect("vault");
    let vault = vault_dir.path().join("v.db");
    init_vault(&vault);
    let jsonl = home
        .path()
        .join(".cursor")
        .join("projects")
        .join("c-dev-x")
        .join("agent-transcripts")
        .join(CURSOR_SID)
        .join(format!("{CURSOR_SID}.jsonl"));
    fs::create_dir_all(jsonl.parent().expect("parent")).expect("mkdir");
    fs::write(&jsonl, "{}\n").expect("write jsonl");

    let output = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .arg("--global")
        .arg("--format")
        .arg("json")
        .env("USERPROFILE", home.path())
        .env("HOME", home.path())
        .output()
        .expect("capture coverage");
    assert!(
        output.status.success(),
        "AC11 deficit exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    let sources = parsed["sources"].as_array().expect("sources");
    assert_eq!(sources.len(), 6);
    let cursor = sources
        .iter()
        .find(|s| s["source"] == "cursor")
        .expect("cursor row");
    assert_eq!(cursor["status"], "deficit");
    assert_eq!(cursor["mode"], "import_only");
    let next = cursor["next_step"].as_str().unwrap_or("");
    assert!(next.contains("cursor-import"), "next_step={next}");
}

#[test]
fn capture_coverage__grok_home_env__honored_without_user_home_override() {
    let grok_home = tempdir().expect("grok home");
    let empty_user_home = tempdir().expect("empty user home");
    let vault_dir = tempdir().expect("vault");
    let vault = vault_dir.path().join("v.db");
    init_vault(&vault);
    let history = grok_home
        .path()
        .join("sessions")
        .join("C%3A")
        .join("sid")
        .join("chat_history.jsonl");
    fs::create_dir_all(history.parent().expect("parent")).expect("mkdir");
    fs::write(&history, "{}\n").expect("write grok history");

    let output = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .arg("--global")
        .arg("--format")
        .arg("json")
        .env("USERPROFILE", empty_user_home.path())
        .env("HOME", empty_user_home.path())
        .env("GROK_HOME", grok_home.path())
        .output()
        .expect("capture coverage");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    let grok = parsed["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .find(|s| s["source"] == "grok")
        .expect("grok row");
    assert!(
        grok["disk_eligible"].as_u64().unwrap_or(0) >= 1,
        "GROK_HOME must be consulted when home_override is None; grok={grok}"
    );
}
