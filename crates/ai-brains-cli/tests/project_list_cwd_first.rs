//! T283 — `project list` human cwd-first (JSON stays memory-desc).
//!
//! Pattern: tempdir vault + context + register-path; hermetic_bin +
//! isolate_empty_home + `--no-project-context`.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

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
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = fs::read_to_string(&env_path).expect(".env after context");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty(), "empty project id in .env");
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from .env after context: {content}");
}

fn register_path(vault: &Path, project_ref: &str, path: &str) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_ref)
        .arg(path)
        .assert()
        .success();
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    let env_path = work_dir.join(".env");
    let env_content = fs::read_to_string(&env_path).expect(".env for pin");
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

/// Leftover-shaped (more pins + registered path) vs smaller cwd owner.
fn leftover_vs_cwd_fixture(dir: &Path) -> (std::path::PathBuf, String, std::path::PathBuf, String) {
    let vault = dir.join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.join("leftover-root");
    let leftover_id = register_project(&vault, &leftover_dir);
    register_path(&vault, &leftover_id, leftover_dir.to_str().expect("utf8"));
    pin_memory(
        &vault,
        &leftover_dir,
        &leftover_id,
        "DECISION: leftover dump seed one",
    );
    pin_memory(
        &vault,
        &leftover_dir,
        &leftover_id,
        "DECISION: leftover dump seed two",
    );

    let cwd_dir = dir.join("cwd-root");
    let cwd_id = register_project(&vault, &cwd_dir);
    register_path(&vault, &cwd_id, cwd_dir.to_str().expect("utf8"));

    (vault, leftover_id, cwd_dir, cwd_id)
}

#[test]
fn project_list__human__cwd_owner_smaller_count__first_data_row() {
    let dir = tempdir().unwrap();
    let (vault, leftover_id, cwd_dir, cwd_id) = leftover_vs_cwd_fixture(dir.path());

    let out = hermetic()
        .current_dir(&cwd_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("label"),
        "AC13: default list is a table; got: {stdout}"
    );
    assert!(
        !stdout.trim_start().starts_with('{'),
        "AC13: default must not be JSON; got: {stdout}"
    );
    let first = stdout.lines().nth(1).expect("first data row");
    assert!(
        first.contains(&cwd_id),
        "AC3: first data row must be cwd owner; line={first}"
    );
    assert!(
        !first.contains(&leftover_id),
        "AC3: first data row must not be leftover; line={first}"
    );
    let cwd_hits = stdout.matches(&cwd_id).count();
    assert_eq!(
        cwd_hits, 1,
        "AC14: cwd project_id once on stdout; got {cwd_hits}; {stdout}"
    );
}

#[test]
fn project_list__json__still_memory_desc() {
    let dir = tempdir().unwrap();
    let (vault, leftover_id, cwd_dir, cwd_id) = leftover_vs_cwd_fixture(dir.path());

    let out = hermetic()
        .current_dir(&cwd_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(v["api_version"], "1");
    assert!(v.get("projects").is_some(), "projects key required");
    assert!(
        v.get("unaliased_count").is_some(),
        "unaliased_count key required"
    );
    let projects = v["projects"].as_array().expect("projects array");
    assert!(
        projects.len() >= 2,
        "expected ≥2 projects; got {}",
        projects.len()
    );
    let first_id = projects[0]["project_id"].as_str().expect("first id");
    assert_eq!(
        first_id, leftover_id,
        "AC4: JSON [0] is larger leftover; got {first_id}"
    );
    let cwd_pos = projects
        .iter()
        .position(|p| p["project_id"].as_str() == Some(cwd_id.as_str()));
    assert!(
        matches!(cwd_pos, Some(i) if i > 0),
        "AC4: cwd id present later; got {cwd_pos:?}"
    );
}

#[test]
fn project_list__human__star_on_leftover_env__cwd_still_first() {
    let dir = tempdir().unwrap();
    let (vault, leftover_id, cwd_dir, cwd_id) = leftover_vs_cwd_fixture(dir.path());

    let out = hermetic()
        .current_dir(&cwd_dir)
        .env("AI_BRAINS_PROJECT_ID", &leftover_id)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first = stdout.lines().nth(1).expect("first data row");
    assert!(
        first.contains(&cwd_id),
        "AC5: first data row still cwd; line={first}"
    );
    assert!(
        !first.contains(&leftover_id),
        "AC5: leftover env must not lead; line={first}"
    );
    let leftover_line = stdout
        .lines()
        .find(|l| l.contains(&leftover_id))
        .expect("leftover row");
    assert!(
        leftover_line.contains('*') || leftover_line.starts_with('*'),
        "AC5: leftover row may show star; line={leftover_line}"
    );
}

#[test]
fn project_list__human__no_path_owner__memory_desc() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.path().join("leftover-root");
    let leftover_id = register_project(&vault, &leftover_dir);
    register_path(
        &vault,
        &leftover_id,
        leftover_dir.to_str().expect("utf8 leftover"),
    );
    pin_memory(
        &vault,
        &leftover_dir,
        &leftover_id,
        "DECISION: leftover dump seed one",
    );
    pin_memory(
        &vault,
        &leftover_dir,
        &leftover_id,
        "DECISION: leftover dump seed two",
    );

    let other_dir = dir.path().join("other-root");
    let other_id = register_project(&vault, &other_dir);

    let out = hermetic()
        .current_dir(&other_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first = stdout.lines().nth(1).expect("first data row");
    assert!(
        first.contains(&leftover_id),
        "AC6: no path-owner keeps memory-desc; line={first}"
    );
    assert!(
        !first.contains(&other_id),
        "AC6: unregistered cwd must not promote; line={first}"
    );
}
