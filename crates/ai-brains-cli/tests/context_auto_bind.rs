//! T344 — fail-closed `context` auto-bind hermetic suite.
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
        .args(["config", "user.email", "t344@example.com"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "T344 Test"])
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
    project_id_from_env(&work_dir.join(".env"))
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

fn context_bind(vault: &Path, cwd: &Path) -> std::process::Output {
    hermetic()
        .current_dir(cwd)
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context bind")
}

fn project_list_json(vault: &Path, cwd: &Path) -> Value {
    let out = hermetic()
        .current_dir(cwd)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list");
    assert!(
        out.status.success(),
        "project list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("list json")
}

fn detail_for<'a>(list: &'a Value, project_id: &str) -> &'a Value {
    list["projects"]
        .as_array()
        .expect("projects array")
        .iter()
        .find(|p| p["project_id"].as_str() == Some(project_id))
        .unwrap_or_else(|| panic!("project {project_id} missing from list"))
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

fn assert_path_bound(list: &Value, project_id: &str, toplevel: &Path) {
    let row = detail_for(list, project_id);
    let path = row["path"].as_str().unwrap_or("");
    assert!(
        !path.is_empty() && path != "null",
        "expected bound path, got {row}"
    );
    let expected = normalize_for_location_compare(&toplevel.to_string_lossy());
    let got = normalize_for_location_compare(path);
    assert_eq!(got, expected, "registered path must be git toplevel");
}

#[test]
fn context_auto_bind__git_unowned__registers_toplevel_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("repo");
    git_init_with_origin(&repo, "https://github.com/example/T344BindPath.git");

    let out = context_bind(&vault, &repo);
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pid = project_id_from_env(&repo.join(".env"));
    let list = project_list_json(&vault, &repo);
    assert_path_bound(&list, &pid, &repo);
}

#[test]
fn context_auto_bind__nested_subdirectory__registers_toplevel_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("nested-root");
    git_init_with_origin(&repo, "https://github.com/example/T344Nested.git");
    let nested = repo.join("src").join("nested");
    fs::create_dir_all(&nested).expect("nested");

    let out = context_bind(&vault, &nested);
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pid = project_id_from_env(&nested.join(".env"));
    let list = project_list_json(&vault, &nested);
    assert_path_bound(&list, &pid, &repo);
    let row = detail_for(&list, &pid);
    let path = row["path"].as_str().unwrap_or("");
    assert!(
        !path.to_lowercase().contains("src"),
        "must not register nested cwd: {path}"
    );
}

#[test]
fn context_auto_bind__second_context__path_idempotent() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("idem");
    git_init_with_origin(&repo, "https://github.com/example/T344Idempotent.git");

    let first = context_bind(&vault, &repo);
    assert!(first.status.success());
    let n = count_events(&vault, "RepositoryPathAliasAdded");
    let second = context_bind(&vault, &repo);
    assert!(second.status.success());
    assert_eq!(
        count_events(&vault, "RepositoryPathAliasAdded"),
        n,
        "second context must not append another path alias"
    );
}

#[test]
fn context_auto_bind__path_owned_other__no_steal() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("owned");
    git_init_with_origin(&repo, "https://github.com/example/T344NoSteal.git");
    let dest_id = register_project(&vault, &repo);
    let other = dir.path().join("other");
    let other_id = register_project(&vault, &other);
    register_path(&vault, &other_id, &repo.to_string_lossy());

    let aliases_before = count_events(&vault, "ProjectAliasAdded");
    let out = context_bind(&vault, &repo);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("auto-bind skip:"),
        "expected skip line, stderr={stderr}"
    );
    assert_eq!(
        count_events(&vault, "ProjectAliasAdded"),
        aliases_before,
        "must not add alias for dest when path owned by other"
    );
    let list = project_list_json(&vault, &repo);
    let dest_row = detail_for(&list, &dest_id);
    assert!(
        dest_row["path"].is_null() || dest_row["path"].as_str().is_none_or(|s| s.is_empty()),
        "dest must not steal path: {dest_row}"
    );
}

#[test]
fn context_auto_bind__unique_slug__sets_alias() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("slug-repo");
    git_init_with_origin(&repo, "https://github.com/example/T344UniqueSlug.git");

    let out = context_bind(&vault, &repo);
    assert!(out.status.success());
    let pid = project_id_from_env(&repo.join(".env"));
    let list = project_list_json(&vault, &repo);
    let row = detail_for(&list, &pid);
    assert_eq!(row["alias"].as_str(), Some("T344UniqueSlug"));
}

#[test]
fn context_auto_bind__nongit_origin__uses_toplevel_dirname_slug() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("T344DirSlug");
    git_init(&repo);

    let out = context_bind(&vault, &repo);
    assert!(out.status.success());
    let pid = project_id_from_env(&repo.join(".env"));
    let list = project_list_json(&vault, &repo);
    let row = detail_for(&list, &pid);
    assert_eq!(row["alias"].as_str(), Some("T344DirSlug"));
}

#[test]
fn context_auto_bind__slug_taken__skips_alias() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let holder = dir.path().join("holder");
    let holder_id = register_project(&vault, &holder);
    set_alias(&vault, &holder_id, "T344TakenSlug");
    let repo = dir.path().join("taken");
    git_init_with_origin(&repo, "https://github.com/example/T344TakenSlug.git");

    let aliases_before = count_events(&vault, "ProjectAliasAdded");
    let out = context_bind(&vault, &repo);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("auto-bind skip:"), "stderr={stderr}");
    assert_eq!(count_events(&vault, "ProjectAliasAdded"), aliases_before);
    let pid = project_id_from_env(&repo.join(".env"));
    let list = project_list_json(&vault, &repo);
    let row = detail_for(&list, &pid);
    assert!(
        row["alias"].as_str().is_none_or(|s| s.is_empty()),
        "dest must not take slug: {row}"
    );
}

#[test]
fn context_auto_bind__show__no_events() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("show-repo");
    git_init_with_origin(&repo, "https://github.com/example/T344ShowSkip.git");
    let _ = register_project(&vault, &repo);
    let before_path = count_events(&vault, "RepositoryPathAliasAdded");
    let before_alias = count_events(&vault, "ProjectAliasAdded");

    hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .arg("--show")
        .assert()
        .success();

    assert_eq!(
        count_events(&vault, "RepositoryPathAliasAdded"),
        before_path
    );
    assert_eq!(count_events(&vault, "ProjectAliasAdded"), before_alias);
}

#[test]
fn context_auto_bind__flag_disables() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("flag");
    git_init_with_origin(&repo, "https://github.com/example/T344FlagOff.git");

    hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .arg("--no-auto-bind")
        .assert()
        .success();

    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), 0);
    let pid = project_id_from_env(&repo.join(".env"));
    let list = project_list_json(&vault, &repo);
    let row = detail_for(&list, &pid);
    assert!(row["path"].is_null() || row["path"].as_str().is_none_or(str::is_empty));
}

#[test]
fn context_auto_bind__env_disables() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("envoff");
    git_init_with_origin(&repo, "https://github.com/example/T344EnvOff.git");

    let mut cmd = hermetic();
    cmd.env("AI_BRAINS_NO_AUTO_BIND", "1")
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .assert()
        .success();

    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), 0);
}

#[test]
fn context_auto_bind__nongit__skips_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("nongit");
    fs::create_dir_all(&work).expect("nongit");

    let out = context_bind(&vault, &work);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("auto-bind skip:"), "stderr={stderr}");
    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), 0);
}

#[test]
fn context_auto_bind__alias_present__skips_alias() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("has-alias");
    git_init_with_origin(&repo, "https://github.com/example/T344HasAlias.git");
    let pid = register_project(&vault, &repo);
    set_alias(&vault, &pid, "existing-label");
    let aliases_before = count_events(&vault, "ProjectAliasAdded");

    let out = context_bind(&vault, &repo);
    assert!(out.status.success());
    assert_eq!(count_events(&vault, "ProjectAliasAdded"), aliases_before);
    let list = project_list_json(&vault, &repo);
    let row = detail_for(&list, &pid);
    assert_eq!(row["alias"].as_str(), Some("existing-label"));
}

#[test]
fn context_auto_bind__doctor_preflight__no_bind() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("query");
    git_init_with_origin(&repo, "https://github.com/example/T344QuerySkip.git");
    let _ = register_project(&vault, &repo);
    let before = count_events(&vault, "RepositoryPathAliasAdded");

    hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .assert()
        .success();
    hermetic()
        .current_dir(&repo)
        .arg("--vault-path")
        .arg(&vault)
        .arg("preflight")
        .arg("--summary")
        .assert()
        .success();

    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), before);
}

#[test]
fn context_auto_bind__already_init__binds_env_pid() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let repo = dir.path().join("already");
    git_init_with_origin(&repo, "https://github.com/example/T344AlreadyInit.git");
    let pid = register_project(&vault, &repo);
    assert_eq!(count_events(&vault, "RepositoryPathAliasAdded"), 0);

    let out = context_bind(&vault, &repo);
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(count_events(&vault, "RepositoryPathAliasAdded") >= 1);
    let list = project_list_json(&vault, &repo);
    assert_path_bound(&list, &pid, &repo);
}

#[test]
fn ambient_denylist__includes_no_auto_bind() {
    assert!(
        common::AMBIENT_DENYLIST.contains(&"AI_BRAINS_NO_AUTO_BIND"),
        "AMBIENT_DENYLIST must strip AI_BRAINS_NO_AUTO_BIND"
    );
}
