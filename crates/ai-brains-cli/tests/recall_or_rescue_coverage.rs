//! T363 — FTS OR-rescue coverage: unmatched 3-token dumps → query_miss.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const DUMP: &str = "ASSISTANT: review note zzzz only";
const QUERY: &str = "zzzz-t363-nohit";
const HONESTY: &str = "No FTS hits; showing in-scope pins";

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

fn pin_owned(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
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

fn owned_cmd<'a>(vault: &'a Path, work_dir: &'a Path, project_id: &'a str) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("--log-format")
        .arg("off");
    if let Ok(env_content) = fs::read_to_string(work_dir.join(".env")) {
        for line in env_content.lines() {
            if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
                cmd.env("AI_BRAINS_SESSION_ID", rest.trim());
            }
        }
    }
    cmd
}

fn parse_last_json_object(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("JSON parse failed: {e}; line={line}; full={stdout}");
    })
}

fn owned_fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    String,
) {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    pin_owned(&vault, &work, &pid, DUMP);
    (dir, vault, work, pid)
}

#[test]
fn recall_or_rescue__single_token_dump__query_miss() {
    let (_dir, vault, work, pid) = owned_fixture();
    let out = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    assert_eq!(
        v["results"].as_array().map(|a| a.len()),
        Some(0),
        "stdout={stdout}"
    );
    assert_eq!(v["empty_kind"], "query_miss");
    assert_ne!(v["empty_kind"], "scope_unowned");
    assert!(
        v["project_memory_count"].as_u64().unwrap_or(0) >= 1,
        "got {v}"
    );

    let pretty = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(QUERY)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall pretty");
    let pstdout = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        !pstdout.contains(HONESTY),
        "unmatched must not print fill honesty; stdout={pstdout}"
    );
    assert!(
        pstdout.contains("This project has") || pstdout.contains("No results"),
        "census/T111 missing; stdout={pstdout}"
    );
}

#[test]
fn search_or_rescue__single_token_dump__query_miss() {
    let (_dir, vault, work, pid) = owned_fixture();
    let out = owned_cmd(&vault, &work, &pid)
        .arg("search")
        .arg(QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("search json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    assert_eq!(
        v["results"].as_array().map(|a| a.len()),
        Some(0),
        "stdout={stdout}"
    );
    assert_eq!(v["empty_kind"], "query_miss");
}

#[test]
fn sync_query_or_rescue__single_token_dump__no_index_honesty() {
    let (_dir, vault, work, pid) = owned_fixture();
    let out = owned_cmd(&vault, &work, &pid)
        .arg("sync")
        .arg("query")
        .arg(QUERY)
        .arg("--no-bridge")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("sync query pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(HONESTY),
        "sync unmatched must not print fill honesty; stdout={stdout}"
    );
    assert!(
        stdout.contains("This project has") || stdout.contains("No results"),
        "census/T111 missing; stdout={stdout}"
    );
}
