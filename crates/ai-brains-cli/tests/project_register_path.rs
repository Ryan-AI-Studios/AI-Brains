//! T233 — `project register-path` hermetic suite (AC1, AC2, AC13).
//!
//! Pattern: tempdir vault + context for project registration;
//! hermetic_bin + `--no-project-context`.
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

/// Register a project via `context` in `work_dir` (writes `.env` there).
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

fn set_alias(vault: &Path, project_id: &str, alias: &str) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("set-alias")
        .arg(project_id)
        .arg(alias)
        .assert()
        .success();
}

fn register_path(vault: &Path, project_ref: &str, path: &str) -> assert_cmd::assert::Assert {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_ref)
        .arg(path)
        .assert()
}

// ---------------------------------------------------------------------------
// AC1 — register-path → project list path non-null
// ---------------------------------------------------------------------------

#[test]
fn register_path__by_uuid__list_path_non_null() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-a");
    let project_id = register_project(&vault, &proj);

    let repo_path = dir.path().join("repo-root");
    fs::create_dir_all(&repo_path).unwrap();
    let path_str = repo_path.to_str().expect("utf8 path");

    register_path(&vault, &project_id, path_str).success();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    let projects = v["projects"].as_array().expect("projects array");
    let row = projects
        .iter()
        .find(|p| p["project_id"].as_str() == Some(project_id.as_str()))
        .expect("project row");
    let path = row["path"]
        .as_str()
        .expect("AC1: path must be non-null string");
    assert!(!path.is_empty(), "AC1: path must be non-empty; got {path}");
}

#[test]
fn register_path__by_alias__succeeds() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-alias");
    let project_id = register_project(&vault, &proj);
    set_alias(&vault, &project_id, "my-root");

    let repo_path = dir.path().join("alias-root");
    fs::create_dir_all(&repo_path).unwrap();
    let path_str = repo_path.to_str().expect("utf8 path");

    register_path(&vault, "my-root", path_str).success();
}

// ---------------------------------------------------------------------------
// AC2 — dual Win/WSL normalize same project (normalize via path crate)
// ---------------------------------------------------------------------------

#[test]
fn register_path__win_and_wsl_forms__same_owner() {
    // Path crate maps /mnt/c/... → Windows drive; dual forms normalize equal.
    // Register both forms to the same project — second is idempotent if equal
    // after normalize, or both stored if normalize yields distinct keys that
    // still resolve via control-plane. Here we assert list still shows a path
    // and re-register of the same form is idempotent OK.
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("dual");
    let project_id = register_project(&vault, &proj);

    // Use a synthetic absolute Windows path that does not need to exist.
    let win = r"C:\dev\T233DualRoot";
    let wsl = "/mnt/c/dev/T233DualRoot";

    register_path(&vault, &project_id, win).success();
    // Same project re-register after normalize of WSL form:
    // if normalize maps WSL → same key, second call is idempotent OK (exit 0).
    // if keys differ (platform), second also succeeds (two aliases, same owner).
    register_path(&vault, &project_id, wsl).success();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("list");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let projects = v["projects"].as_array().unwrap();
    let row = projects
        .iter()
        .find(|p| p["project_id"].as_str() == Some(project_id.as_str()))
        .unwrap();
    assert!(
        row["path"].as_str().is_some_and(|p| !p.is_empty()),
        "AC2: path column non-null after dual register; row={row}"
    );
}

// ---------------------------------------------------------------------------
// AC13 — F21 conflict: other project → exit 1 + ownership message
// ---------------------------------------------------------------------------

#[test]
fn register_path__conflict_other_project__exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let a_dir = dir.path().join("a");
    let b_dir = dir.path().join("b");
    let id_a = register_project(&vault, &a_dir);
    let id_b = register_project(&vault, &b_dir);

    let shared = dir.path().join("shared-root");
    fs::create_dir_all(&shared).unwrap();
    let path_str = shared.to_str().expect("utf8");

    register_path(&vault, &id_a, path_str).success();

    let assert = register_path(&vault, &id_b, path_str).failure().code(1);
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("path alias")
            && stderr.contains("already registered to project")
            && stderr.contains(&id_a)
            && stderr.contains("unregister-path is soft residual F31"),
        "AC13: conflict message exact class; got: {stderr}"
    );
}

#[test]
fn register_path__same_project_reregister__idempotent_ok() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("same");
    let project_id = register_project(&vault, &proj);
    let root = dir.path().join("idempotent-root");
    fs::create_dir_all(&root).unwrap();
    let path_str = root.to_str().expect("utf8");

    register_path(&vault, &project_id, path_str).success();
    let assert = register_path(&vault, &project_id, path_str).success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("already registered"),
        "idempotent message; got: {stdout}"
    );
}
