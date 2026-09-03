//! T352 — `preflight --summary --bind` hermetic suite.
//! Do **not** pass `--no-project-context` on AC2/AC3 bind spawns (disables helper).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_path::normalize_for_location_compare;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

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

fn git_init(repo: &Path) {
    fs::create_dir_all(repo).expect("repo dir");
    let status = Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git init");
    assert!(status.success(), "git init failed");
    let _ = Command::new("git")
        .args(["config", "user.email", "t352@example.com"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "T352 Test"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
}

fn git_init_with_origin(repo: &Path, origin_url: &str) {
    git_init(repo);
    let status = Command::new("git")
        .args(["remote", "add", "origin", origin_url])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git remote add");
    assert!(status.success(), "git remote add failed");
}

fn project_id_from_env(env_path: &Path) -> String {
    let content = fs::read_to_string(env_path).expect(".env");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty(), "empty project id");
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from {env_path:?}: {content}");
}

fn context_no_bind(vault: &Path, cwd: &Path) -> String {
    hermetic()
        .current_dir(cwd)
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .arg("--no-auto-bind")
        .assert()
        .success();
    project_id_from_env(&cwd.join(".env"))
}

fn count_events(vault_path: &Path, event_type: &str) -> i64 {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(
        vault_path.to_str().expect("utf8 vault"),
        &key,
    )
    .expect("open vault");
    let locked = conn.lock().expect("lock");
    locked
        .query_row(
            "SELECT COUNT(*) FROM events WHERE event_type = ?",
            [event_type],
            |r| r.get(0),
        )
        .expect("count")
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

#[test]
fn preflight_summary_bind__unowned_git__registers_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("repo");
    git_init_with_origin(&repo, "https://github.com/example/T352BindPath.git");
    let id = context_no_bind(&vault, &repo);
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    let out = hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .arg("preflight")
        .arg("--summary")
        .arg("--bind")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight bind");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        count_events(&vault, "RepositoryPathAliasAdded") > before,
        "AC2 must append RepositoryPathAliasAdded"
    );
    let expected = normalize_for_location_compare(&repo.to_string_lossy());
    assert!(
        stdout.contains(&format!("path={expected}")),
        "AC2 bound path=; got:\n{stdout}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("Auto-bound path"),
        "AC2 Auto-bound path; got stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn preflight_summary_bind__without_bind__no_path_event() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("nobind");
    git_init_with_origin(&repo, "https://github.com/example/T352NoBind.git");
    let id = context_no_bind(&vault, &repo);
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    let out = hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .arg("preflight")
        .arg("--summary")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight summary");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        count_events(&vault, "RepositoryPathAliasAdded"),
        before,
        "AC3 default summary must not bind"
    );
}

#[test]
fn preflight_summary_bind__no_auto_bind_env__skips() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("envoff");
    git_init_with_origin(&repo, "https://github.com/example/T352EnvOff.git");
    let id = context_no_bind(&vault, &repo);
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    let out = hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .env("AI_BRAINS_NO_AUTO_BIND", "1")
        .arg("preflight")
        .arg("--summary")
        .arg("--bind")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight bind env off");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        count_events(&vault, "RepositoryPathAliasAdded"),
        before,
        "AC5 env disable must skip"
    );
}

#[test]
fn preflight_summary_bind__path_owned_other__no_steal() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("owned");
    git_init_with_origin(&repo, "https://github.com/example/T352NoSteal.git");
    let dest_id = context_no_bind(&vault, &repo);
    let other = dir.path().join("other");
    fs::create_dir_all(&other).expect("other");
    let other_id = context_no_bind(&vault, &other);
    register_path(&vault, &other_id, &repo.to_string_lossy());
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    let out = hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &dest_id)
        .arg("preflight")
        .arg("--summary")
        .arg("--bind")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight bind no steal");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr.contains("auto-bind skip:"),
        "AC6 skip; stderr={stderr}"
    );
    assert_eq!(
        count_events(&vault, "RepositoryPathAliasAdded"),
        before,
        "AC6 must not steal"
    );
}

#[test]
fn preflight_summary_bind__no_project_id__skip_exit_0() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("preflight")
        .arg("--summary")
        .arg("--bind")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight bind none");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        stderr.contains("auto-bind skip: no project id"),
        "AC8 skip; stderr={stderr}"
    );
    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), before);

    let gout = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("preflight")
        .arg("--global")
        .arg("--summary")
        .arg("--bind")
        .arg("--no-hook-prompt")
        .output()
        .expect("preflight global bind");
    let gerr = String::from_utf8_lossy(&gout.stderr);
    assert_eq!(gout.status.code(), Some(0), "stderr={gerr}");
    assert!(
        gerr.contains("auto-bind skip: no project id"),
        "AC8 --global skip; stderr={gerr}"
    );
    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), before);
}

#[test]
fn preflight_summary_bind__json_stdout_clean() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("jsonbind");
    git_init_with_origin(&repo, "https://github.com/example/T352JsonBind.git");
    let id = context_no_bind(&vault, &repo);

    let out = hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .arg("preflight")
        .arg("--summary")
        .arg("--bind")
        .arg("--format")
        .arg("json")
        .arg("--no-hook-prompt")
        .output()
        .expect("json bind");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
    assert!(
        !stdout.contains("Auto-bound path"),
        "AC9 no Auto-bound on stdout; got:\n{stdout}"
    );
    let v: Value = serde_json::from_str(stdout.trim()).expect("json stdout");
    assert_eq!(v["api_version"], "1");
}

#[test]
fn preflight__bind_without_summary__exit_2() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("preflight")
        .arg("--bind")
        .output()
        .expect("bind without summary");
    assert_eq!(out.status.code(), Some(2), "AC4 clap exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--summary"),
        "AC4 stderr mentions --summary; got: {stderr}"
    );
}

#[test]
fn skill__session_start__preflight_summary_bind() {
    let text = include_str!("../../../.claude/skills/ai-brains/SKILL.md");
    assert!(
        text.contains("preflight --summary --bind"),
        "AC7 skill session-start"
    );
}
