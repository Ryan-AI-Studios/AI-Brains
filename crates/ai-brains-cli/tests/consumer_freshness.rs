//! T361 — consumer freshness (empty_scope / query_miss / scope_unowned / global omit).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::fs;
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

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .arg("--no-auto-bind")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = fs::read_to_string(&env_path).expect(".env");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty());
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from .env");
}

fn register_path(vault: &Path, work_dir: &Path, project_id: &str) {
    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_id)
        .arg(work_dir)
        .assert()
        .success();
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    let env_content = fs::read_to_string(work_dir.join(".env")).expect(".env");
    let mut session_id = String::new();
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            session_id = rest.trim().to_string();
        }
    }
    assert!(!session_id.is_empty());
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

fn parse_last_json(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; line={line}; full={stdout}");
    })
}

fn recall_json(vault: &Path, work_dir: &Path, extra: &[&str]) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("--log-format")
        .arg("off");
    if let Ok(env_content) = fs::read_to_string(work_dir.join(".env")) {
        for line in env_content.lines() {
            if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
                cmd.env("AI_BRAINS_PROJECT_ID", rest.trim());
            }
            if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
                cmd.env("AI_BRAINS_SESSION_ID", rest.trim());
            }
        }
    }
    cmd.arg("recall")
        .arg("zzzz-t361-nohit")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge");
    for a in extra {
        cmd.arg(a);
    }
    let out = cmd.output().expect("recall");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn capture_coverage_empty_owned__empty_kind_empty_scope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);

    let (code, stdout, stderr) = recall_json(&vault, &work, &[]);
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_last_json(&stdout);
    assert_eq!(v["results"].as_array().map(|a| a.len()), Some(0));
    assert_eq!(v["empty_kind"], "empty_scope");
    assert_eq!(v["project_memory_count"], 0);

    let pretty = hermetic()
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzz-t361-nohit")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("pretty");
    assert_eq!(pretty.status.code(), Some(0));
    let pstdout = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pstdout.contains("This project has 0 memories."),
        "got {pstdout}"
    );
}

#[test]
fn recall_empty__non_authority_pin__query_miss() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    pin_memory(
        &vault,
        &work,
        &pid,
        "just a note about widgets without markers",
    );

    let (code, stdout, stderr) = recall_json(&vault, &work, &[]);
    assert_eq!(code, 0, "stderr={stderr} stdout={stdout}");
    let v = parse_last_json(&stdout);
    if v["results"]
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        assert!(
            v.get("empty_kind").is_none() || v["empty_kind"].is_null(),
            "T346 fill omits empty_kind; got {v}"
        );
        return;
    }
    assert_eq!(v["empty_kind"], "query_miss");
    assert_eq!(v["project_memory_count"], 1);
    let hint = v["hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("--semantic") || hint.contains("--global"),
        "got {hint}"
    );
}

#[test]
fn recall_empty__unowned_cwd__scope_unowned() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let _pid = register_project(&vault, &work);

    let (code, stdout, stderr) = recall_json(&vault, &work, &[]);
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_last_json(&stdout);
    assert_eq!(v["empty_kind"], "scope_unowned");
    let hint = v["hint"].as_str().unwrap_or("");
    assert!(hint.contains("project whoami"), "got {hint}");
}

#[test]
fn recall_empty__global__omits_empty_kind() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    fs::create_dir_all(&work).expect("work");

    let (code, stdout, stderr) = recall_json(&vault, &work, &["--global"]);
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_last_json(&stdout);
    assert!(
        v.get("empty_kind").is_none() || v["empty_kind"].is_null(),
        "got {v}"
    );
    assert!(
        v.get("project_memory_count").is_none() || v["project_memory_count"].is_null(),
        "got {v}"
    );
    let hint = v["hint"].as_str().unwrap_or("");
    assert!(hint.contains("across all projects"), "got {hint}");
    assert!(!hint.contains("This project has"), "got {hint}");
}
