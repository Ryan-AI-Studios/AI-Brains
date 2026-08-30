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

const T265_SECTION_IDS: [&str; 8] = [
    "safety",
    "session",
    "index",
    "recent",
    "ledgerful",
    "empty_repo",
    "governed",
    "other",
];

fn register_project(vault: &Path, work_dir: &Path) -> String {
    std::fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = std::fs::read_to_string(&env_path).expect(".env after context");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty(), "empty project id in .env");
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from .env after context: {content}");
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    let env_path = work_dir.join(".env");
    let env_content = std::fs::read_to_string(&env_path).expect(".env for pin");
    let mut session_id = String::new();
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            session_id = rest.trim().to_string();
        }
    }
    assert!(!session_id.is_empty(), "SESSION_ID missing from .env");

    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .env("AI_BRAINS_SESSION_ID", &session_id)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn run_preflight_scoped(vault: &Path, extra: &[&str], project_id: &str) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
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

/// T329 AC7 — mixed small pin: JSON required keys frozen; section ids ⊂ T265 set.
#[test]
fn preflight_json_envelope__small_pin__required_keys_no_new_section_id() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t329-json");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: T329-ac7 compact envelope pin that must parse",
    );

    let (code, stdout, stderr) = run_preflight_scoped(
        &vault,
        &["--format", "json", "-m", "1500", "--no-hook-prompt"],
        &id,
    );
    assert_eq!(code, 0, "AC7 exit 0; stderr={stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("AC7 parse: {e}; {stdout}"));
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"), "AC7 text");
    assert!(obj.contains_key("word_count"), "AC7 word_count");
    assert!(obj.contains_key("sections"), "AC7 sections");
    for key in obj.keys() {
        assert!(
            key == "text" || key == "word_count" || key == "sections",
            "AC7: no new envelope key {key:?}; keys={:?}",
            obj.keys().collect::<Vec<_>>()
        );
    }
    let sections = obj
        .get("sections")
        .and_then(|s| s.as_array())
        .expect("AC7 sections array");
    for s in sections {
        let sid = s.get("id").and_then(|i| i.as_str()).unwrap_or("");
        assert!(
            T265_SECTION_IDS.contains(&sid),
            "AC7: unknown section id {sid:?}; sections={sections:?}"
        );
    }
}
