#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T206 — Project detect honesty hermetic suite (AC1–AC5, AC10).
//!
//! Pattern: tempdir vault + context for project registration + set-alias;
//! git fixtures via `git init` / `remote add origin`; hermetic_bin +
//! `--no-project-context`; ambient PROJECT_ID stripped by denylist.

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// Hermetic bin with empty home (F34): strips ambient PROJECT_ID and blocks
/// global `~/.ai-brains/.env` gap-fill of developer PROJECT_ID.
fn hermetic_detect() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic_detect()
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
    let out = hermetic_detect()
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
    // Prefer PROJECT_ID from written .env
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
    hermetic_detect()
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

fn git_init_with_origin(repo: &Path, origin_url: &str) {
    fs::create_dir_all(repo).expect("repo dir");
    let status = Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("git init");
    assert!(status.success(), "git init failed");
    // Avoid identity prompts on later git ops.
    let _ = Command::new("git")
        .args(["config", "user.email", "t206@example.com"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "T206 Test"])
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

/// AC5: miss → exit 1 + context guidance (T198 regression also covers this).
#[test]
fn project_detect__miss__exit_1_mentions_context() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(dir.path())
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect miss");

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("No project detected"),
        "expected miss message; got: {stderr}"
    );
    assert!(
        stderr.contains("context") || stderr.contains("ai-brains context"),
        "must mention context; got: {stderr}"
    );
}

/// AC5 export: miss also exit 1 with `#` comment guidance.
#[test]
fn project_detect__export_miss__exit_1_hash_comment() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(dir.path())
        .arg("project")
        .arg("detect")
        .arg("--export")
        .output()
        .expect("detect --export miss");

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("#") && stderr.contains("No project detected"),
        "export miss must be # comment; got: {stderr}"
    );
}

/// AC2: env-only hit (no git); from .env; exit 0.
#[test]
fn project_detect__env_only__from_env_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_dir = dir.path().join("env-only-proj");
    let project_id = register_project(&vault, &proj_dir);
    set_alias(&vault, &project_id, "env-only-alias");

    // Run detect from a non-git sibling dir with PROJECT_ID set.
    let run_dir = dir.path().join("not-a-git-repo");
    fs::create_dir_all(&run_dir).unwrap();

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&run_dir)
        .env("AI_BRAINS_PROJECT_ID", &project_id)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect env-only");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("from .env") || stdout.contains("from env"),
        "expected from .env label; got: {stdout}"
    );
    assert!(
        stdout.contains(&project_id),
        "must print project id; got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("git/env project mismatch"),
        "no mismatch warn without git slug; got: {stderr}"
    );
}

/// AC3: env hit + remote slug mismatch → stderr warn + set-alias; exit 0.
#[test]
fn project_detect__env_hit_git_mismatch__warn_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_dir = dir.path().join("mismatch-proj");
    let project_id = register_project(&vault, &proj_dir);
    set_alias(&vault, &project_id, "test-alias");

    // Git repo with origin slug that does NOT match alias/name.
    let repo = dir.path().join("checkout-folder");
    git_init_with_origin(&repo, "https://github.com/user/AI-Brains.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("AI_BRAINS_PROJECT_ID", &project_id)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect mismatch");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("git/env project mismatch"),
        "expected F4/F35 warn; got: {stderr}"
    );
    assert!(
        stderr.contains("AI-Brains"),
        "warn must mention git slug; got: {stderr}"
    );
    assert!(
        stderr.contains("set-alias") && stderr.contains(&project_id),
        "hint must include set-alias + id; got: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("from .env") || stdout.contains(&project_id),
        "still detects via env; got: {stdout}"
    );
}

/// AC3 export: mismatch warnings appear as `#` comments; exit 0.
#[test]
fn project_detect__export_mismatch__hash_comments_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_dir = dir.path().join("export-mismatch-proj");
    let project_id = register_project(&vault, &proj_dir);
    set_alias(&vault, &project_id, "export-alias");

    let repo = dir.path().join("export-checkout");
    git_init_with_origin(&repo, "https://github.com/user/OtherSlug.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("AI_BRAINS_PROJECT_ID", &project_id)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .arg("--export")
        .output()
        .expect("detect --export mismatch");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("export AI_BRAINS_PROJECT_ID={project_id}")),
        "export line; got: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('#') && l.contains("git/env project mismatch")),
        "warn as # comment; got: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('#') && l.contains("set-alias")),
        "hint as # comment; got: {stdout}"
    );
}

/// AC1: unique git match wins over wrong env PROJECT_ID; prints from git.
#[test]
fn project_detect__unique_git_wins_over_wrong_env() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Project A: wrong env target
    let proj_a = dir.path().join("proj-a");
    let id_a = register_project(&vault, &proj_a);
    set_alias(&vault, &id_a, "wrong-env-alias");

    // Project B: git-matched via alias = remote slug
    let proj_b = dir.path().join("proj-b");
    let id_b = register_project(&vault, &proj_b);
    set_alias(&vault, &id_b, "HonestSlug");

    // Repo dir name intentionally differs from remote slug (AC10 / F31).
    let repo = dir.path().join("my-fork-checkout");
    git_init_with_origin(&repo, "https://github.com/user/HonestSlug.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("AI_BRAINS_PROJECT_ID", &id_a)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect git wins");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("from git"),
        "expected from git; got: {stdout}"
    );
    assert!(
        stdout.contains(&id_b),
        "must select git-matched project B; got: {stdout}"
    );
    assert!(
        !stdout.contains(&id_a),
        "must not print wrong env project A; got: {stdout}"
    );
}

/// AC4: ambiguous ≥2 matches → exit 1 + listed candidates on stderr.
///
/// Two aliases that both *contain* the remote slug (zero exact) → Ambiguous.
#[test]
fn project_detect__ambiguous__exit_1_lists_candidates() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let p1 = dir.path().join("ambig-one");
    let id1 = register_project(&vault, &p1);
    set_alias(&vault, &id1, "proj-alpha-slug");

    let p2 = dir.path().join("ambig-two");
    let id2 = register_project(&vault, &p2);
    set_alias(&vault, &id2, "proj-beta-slug");

    let repo = dir.path().join("ambig-repo");
    git_init_with_origin(&repo, "https://github.com/user/slug.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect ambiguous");

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.to_lowercase().contains("ambiguous"),
        "must say ambiguous; got: {stderr}"
    );
    assert!(
        stderr.contains(&id1) && stderr.contains(&id2),
        "must list both candidates; got: {stderr}"
    );
}

/// AC4 export: ambiguous also exit 1 (fail-closed).
#[test]
fn project_detect__export_ambiguous__exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let p1 = dir.path().join("exp-ambig-one");
    let id1 = register_project(&vault, &p1);
    set_alias(&vault, &id1, "x-sharedtoken");

    let p2 = dir.path().join("exp-ambig-two");
    let id2 = register_project(&vault, &p2);
    set_alias(&vault, &id2, "y-sharedtoken");

    let repo = dir.path().join("exp-ambig-repo");
    git_init_with_origin(&repo, "https://github.com/user/sharedtoken.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .arg("--export")
        .output()
        .expect("detect --export ambiguous");

    assert_eq!(
        out.status.code(),
        Some(1),
        "export ambiguous must fail-closed; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&id1) || stderr.contains("Ambiguous") || stderr.contains("ambiguous"),
        "stderr candidates/comment; got: {stderr}"
    );
    let _ = id2;
}

/// AC10: origin remote preferred over directory name for slug identity.
#[test]
fn project_detect__remote_first_slug__dir_name_ignored() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("remote-first-proj");
    let project_id = register_project(&vault, &proj);
    // Alias matches remote slug, NOT the directory name.
    set_alias(&vault, &project_id, "RealOriginName");

    let repo = dir.path().join("totally-different-dirname");
    git_init_with_origin(&repo, "https://github.com/org/RealOriginName.git");

    let out = hermetic_detect()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .current_dir(&repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("project")
        .arg("detect")
        .output()
        .expect("detect remote-first");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("from git") && stdout.contains(&project_id),
        "remote slug must match alias; got: {stdout}"
    );
}
