#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T265 — compact full-preflight JSON envelope hermetics (AC1 / AC11).

mod common;

use std::path::Path;
use tempfile::tempdir;

fn hermetic() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn run_preflight(vault: &Path, extra: &[&str]) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("preflight");
    for a in extra {
        cmd.arg(a);
    }
    let out = cmd.output().expect("preflight");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn preflight_json_envelope__format_json__required_keys_compact_sections_array() {
    // AC1
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--format", "json"]);
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    let line = stdout.trim();
    assert!(
        !line.contains('\n'),
        "AC1: compact document (no raw newlines at envelope level): {line}"
    );
    let v: serde_json::Value =
        serde_json::from_str(line).unwrap_or_else(|e| panic!("AC1 parse: {e}; {stdout}"));
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"), "AC1 text");
    assert!(obj.contains_key("word_count"), "AC1 word_count");
    let sections = obj.get("sections").expect("AC1 sections present");
    assert!(
        sections.is_array(),
        "AC1 sections is array (never null); got {sections}"
    );
    assert!(!sections.is_null(), "AC1 sections never null");
}

#[test]
fn preflight_json_envelope__empty_vault__sections_empty_or_empty_repo_header() {
    // AC11 — never fabricate empty_repo without the live header.
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--format", "json", "--no-hook-prompt"]);
    assert_eq!(code, 0, "AC11 exit 0; stderr={stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("AC11 parse: {e}; {stdout}"));
    assert!(v.get("text").and_then(|t| t.as_str()).is_some());
    assert!(v.get("word_count").is_some());
    let sections = v
        .get("sections")
        .and_then(|s| s.as_array())
        .expect("AC11 sections array");
    let text = v["text"].as_str().unwrap_or("");
    let has_empty_repo_header = text.contains("--- AI-Brains: New Repository Detected ---");
    if has_empty_repo_header {
        assert!(
            sections
                .iter()
                .any(|s| s.get("id").and_then(|i| i.as_str()) == Some("empty_repo")),
            "AC11: header present → empty_repo section; sections={sections:?}"
        );
    } else {
        assert!(
            sections
                .iter()
                .all(|s| s.get("id").and_then(|i| i.as_str()) != Some("empty_repo")),
            "AC11: must not fabricate empty_repo; sections={sections:?}"
        );
    }
}
