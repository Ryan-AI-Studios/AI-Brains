//! T240 — Project identity convergence hermetic suite.
//!
//! Covers: path alias wins over unique slug; whoami JSON fields;
//! mismatch warn once; export source comments. T206 regressions stay
//! in `project_detect_honesty.rs`.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;
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

fn git_init_with_origin(repo: &Path, origin_url: &str) {
    fs::create_dir_all(repo).expect("repo dir");
    let status = Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git init");
    assert!(status.success(), "git init failed");
    let _ = Command::new("git")
        .args(["config", "user.email", "t240@example.com"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "T240 Test"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let status = Command::new("git")
        .args(["remote", "add", "origin", origin_url])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git remote add");
    assert!(status.success(), "git remote add failed");
}

// ---------------------------------------------------------------------------
// AC3 — path alias wins over unique slug different project
// ---------------------------------------------------------------------------

#[test]
fn project_detect__path_alias_wins_over_unique_slug() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Project A: git slug match only (0 path)
    let proj_a = dir.path().join("proj-slug");
    let id_a = register_project(&vault, &proj_a);
    set_alias(&vault, &id_a, "HonestSlug");

    // Project B: path owner (different from slug project)
    let proj_b = dir.path().join("proj-path");
    let id_b = register_project(&vault, &proj_b);
    set_alias(&vault, &id_b, "path-owner");

    let repo = dir.path().join("checkout");
    git_init_with_origin(&repo, "https://github.com/user/HonestSlug.git");
    let path_str = repo.to_str().expect("utf8 path");
    register_path(&vault, &id_b, path_str);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect path wins");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("path alias") || stdout.contains("path_alias"),
        "expected path alias source; got: {stdout}"
    );
    assert!(
        stdout.contains(&id_b),
        "must select path owner B; got: {stdout}"
    );
    assert!(
        !stdout.contains(&id_a),
        "must not print slug project A on stdout; got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&id_a) && stderr.contains("preferring path"),
        "F6 note must mention slug project A; got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Export source comment
// ---------------------------------------------------------------------------

#[test]
fn project_detect__export_path_alias__source_comment() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-export-path");
    let id = register_project(&vault, &proj);
    set_alias(&vault, &id, "export-path-alias");

    let repo = dir.path().join("export-path-repo");
    git_init_with_origin(&repo, "https://github.com/user/UnrelatedSlug.git");
    register_path(&vault, &id, repo.to_str().expect("utf8"));

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .arg("--export")
        .output()
        .expect("detect --export path");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("export AI_BRAINS_PROJECT_ID={id}")),
        "export line; got: {stdout}"
    );
    assert!(
        stdout.contains("source=path_alias") || stdout.contains("from path_alias"),
        "export must note path_alias source; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// whoami JSON fields
// ---------------------------------------------------------------------------

#[test]
fn project_whoami__json__fields_present() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("whoami-proj");
    let id = register_project(&vault, &proj);
    set_alias(&vault, &id, "whoami-alias");

    let repo = dir.path().join("whoami-repo");
    git_init_with_origin(&repo, "https://github.com/user/whoami-alias.git");
    register_path(&vault, &id, repo.to_str().expect("utf8"));

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("whoami")
        .arg("--format")
        .arg("json")
        .output()
        .expect("whoami json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    // --no-project-context → env null (F17)
    assert!(
        v.get("env_project_id").is_some_and(|x| x.is_null()),
        "env null under --no-project-context; got: {stdout}"
    );
    assert!(
        v.get("effective_project_id").is_some_and(|x| x.is_null()),
        "effective null under --no-project-context; got: {stdout}"
    );
    assert_eq!(
        v.get("path_alias_project_id").and_then(|x| x.as_str()),
        Some(id.as_str()),
        "path owner; got: {stdout}"
    );
    assert_eq!(
        v.get("detect_project_id").and_then(|x| x.as_str()),
        Some(id.as_str()),
        "detect path; got: {stdout}"
    );
    assert!(v.get("git_slug").is_some());
    assert!(v.get("git_toplevel").is_some());
    assert!(v.get("mismatch").is_some_and(|x| x.is_boolean()));
    assert!(v.get("remediations").is_some_and(|x| x.is_array()));
}

// ---------------------------------------------------------------------------
// whoami with project context: mismatch when env ≠ path
// ---------------------------------------------------------------------------

#[test]
fn project_whoami__env_differs_path__mismatch_true() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_env = dir.path().join("env-scope-proj");
    let id_env = register_project(&vault, &proj_env);
    set_alias(&vault, &id_env, "env-scope");

    let proj_path = dir.path().join("path-scope-proj");
    let id_path = register_project(&vault, &proj_path);
    set_alias(&vault, &id_path, "path-scope");

    let repo = dir.path().join("mismatch-repo");
    git_init_with_origin(&repo, "https://github.com/user/Other.git");
    register_path(&vault, &id_path, repo.to_str().expect("utf8"));

    // Write .env so force-set applies when not using --no-project-context.
    // But hermetic still needs vault path flag; use env PROJECT_ID with project context
    // skipped and set both via env for detect/path signals — whoami mismatch is
    // env vs path when env is present. Under --no-project-context env_project_id is null.
    // So run WITHOUT --no-project-context: write .env in repo with wrong id.
    fs::write(
        repo.join(".env"),
        format!("AI_BRAINS_PROJECT_ID={id_env}\n"),
    )
    .expect("write .env");

    let out = hermetic()
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        // shell PROJECT_ID different from .env for shell_project_id field
        .env("AI_BRAINS_PROJECT_ID", &id_path)
        .arg("project")
        .arg("whoami")
        .arg("--format")
        .arg("json")
        .output()
        .expect("whoami mismatch");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(
        v.get("env_project_id").and_then(|x| x.as_str()),
        Some(id_env.as_str()),
        "post-dotenv env from .env; got: {stdout}"
    );
    assert_eq!(
        v.get("path_alias_project_id").and_then(|x| x.as_str()),
        Some(id_path.as_str()),
        "path owner; got: {stdout}"
    );
    assert_eq!(
        v.get("mismatch").and_then(|x| x.as_bool()),
        Some(true),
        "mismatch true; got: {stdout}"
    );
    // shell was id_path, env force-set to id_env → shell reported when differs
    assert_eq!(
        v.get("shell_project_id").and_then(|x| x.as_str()),
        Some(id_path.as_str()),
        "shell pre-dotenv; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Mismatch warn on vault-using command (once; single invocation)
// ---------------------------------------------------------------------------

#[test]
fn project_list__identity_mismatch__warn_on_stderr() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_env = dir.path().join("warn-env-proj");
    let id_env = register_project(&vault, &proj_env);

    let proj_path = dir.path().join("warn-path-proj");
    let id_path = register_project(&vault, &proj_path);

    let work = dir.path().join("warn-work");
    fs::create_dir_all(&work).unwrap();
    register_path(&vault, &id_path, work.to_str().expect("utf8"));
    fs::write(
        work.join(".env"),
        format!("AI_BRAINS_PROJECT_ID={id_env}\n"),
    )
    .expect("write .env");

    let out = hermetic()
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&work)
        .arg("project")
        .arg("list")
        .output()
        .expect("list with mismatch");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("project identity mismatch"),
        "expected mismatch warn; got: {stderr}"
    );
    assert!(
        stderr.contains(&id_env) && stderr.contains(&id_path),
        "warn must include env and path ids; got: {stderr}"
    );
    assert!(
        stderr.contains("project whoami"),
        "warn must hint whoami; got: {stderr}"
    );
}

#[test]
fn project_list__no_project_context__no_mismatch_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_env = dir.path().join("skip-env-proj");
    let id_env = register_project(&vault, &proj_env);

    let proj_path = dir.path().join("skip-path-proj");
    let id_path = register_project(&vault, &proj_path);

    let work = dir.path().join("skip-work");
    fs::create_dir_all(&work).unwrap();
    register_path(&vault, &id_path, work.to_str().expect("utf8"));

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&work)
        .env("AI_BRAINS_PROJECT_ID", &id_env)
        .arg("project")
        .arg("list")
        .output()
        .expect("list skip warn");

    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("project identity mismatch"),
        "must skip warn under --no-project-context; got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Path case normalize (register different casing than cwd when OS allows)
// ---------------------------------------------------------------------------

#[test]
fn project_detect__path_case_normalize__same_owner() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("case-proj");
    let id = register_project(&vault, &proj);
    set_alias(&vault, &id, "case-alias");

    let repo = dir.path().join("CaseRepo");
    git_init_with_origin(&repo, "https://github.com/user/UnrelatedCase.git");

    // Register with alternate casing of the drive letter / path if Windows.
    let path_str = repo.to_str().expect("utf8").to_string();
    let alt = if path_str.len() >= 2 && path_str.as_bytes()[1] == b':' {
        let mut chars: Vec<char> = path_str.chars().collect();
        if chars[0].is_ascii_uppercase() {
            chars[0] = chars[0].to_ascii_lowercase();
        } else {
            chars[0] = chars[0].to_ascii_uppercase();
        }
        chars.into_iter().collect::<String>()
    } else {
        path_str.clone()
    };
    register_path(&vault, &id, &alt);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect case");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&id),
        "case-normalized path must resolve; got: {stdout}"
    );
    assert!(
        stdout.contains("path alias") || stdout.contains("path_alias"),
        "source path alias; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC9 — path alias registered on git toplevel; detect from subdir cwd
// ---------------------------------------------------------------------------

#[test]
fn project_detect__subdir_cwd__uses_toplevel_path_alias() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_slug = dir.path().join("subdir-slug-proj");
    let id_slug = register_project(&vault, &proj_slug);
    set_alias(&vault, &id_slug, "SubdirSlug");

    let proj_path = dir.path().join("subdir-path-proj");
    let id_path = register_project(&vault, &proj_path);
    set_alias(&vault, &id_path, "subdir-path-owner");

    let repo = dir.path().join("subdir-repo");
    git_init_with_origin(&repo, "https://github.com/user/SubdirSlug.git");
    register_path(&vault, &id_path, repo.to_str().expect("utf8"));

    let nested = repo.join("nested").join("deep");
    fs::create_dir_all(&nested).expect("nested cwd");

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&nested)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect from subdir");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&id_path),
        "subdir cwd must resolve path owner via toplevel; got: {stdout}"
    );
    assert!(
        !stdout.contains(&id_slug),
        "must not select slug project on stdout; got: {stdout}"
    );
    assert!(
        stdout.contains("path alias") || stdout.contains("path_alias"),
        "expected path alias source; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// F6 — path 0 mem + slug >0 mem → extra verify note on stderr
// ---------------------------------------------------------------------------

#[test]
fn project_detect__path_zero_mem__extra_verify_note() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Project A (slug): pin a memory so mem > 0
    let proj_a = dir.path().join("zero-mem-slug");
    let id_a = register_project(&vault, &proj_a);
    set_alias(&vault, &id_a, "ZeroMemSlug");
    // session from project .env written by context
    let session_a = {
        let content = fs::read_to_string(proj_a.join(".env")).expect(".env A");
        content
            .lines()
            .find_map(|l| {
                l.strip_prefix("AI_BRAINS_SESSION_ID=")
                    .map(|s| s.trim().to_string())
            })
            .expect("session id in .env A")
    };
    hermetic()
        .current_dir(&proj_a)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id_a)
        .env("AI_BRAINS_SESSION_ID", &session_a)
        .arg("pin")
        .arg("CONSTRAINT: t240 zero-mem fixture memory")
        .assert()
        .success();

    // Project B (path owner): no pins → 0 mem
    let proj_b = dir.path().join("zero-mem-path");
    let id_b = register_project(&vault, &proj_b);
    set_alias(&vault, &id_b, "zero-mem-path-owner");

    let repo = dir.path().join("zero-mem-repo");
    git_init_with_origin(&repo, "https://github.com/user/ZeroMemSlug.git");
    register_path(&vault, &id_b, repo.to_str().expect("utf8"));

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect 0-mem note");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&id_b),
        "path owner wins even at 0 mem; got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("preferring path"),
        "F6 slug note; got: {stderr}"
    );
    assert!(
        stderr.contains("0 memories") && stderr.contains("project list"),
        "F6 zero-mem verify note; got: {stderr}"
    );
}
