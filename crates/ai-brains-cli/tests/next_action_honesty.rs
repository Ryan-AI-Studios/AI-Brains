//! T267 — next-action remediator honesty (harness ok = none; list footer F3/F3b).
//!
//! AC2–AC4 / AC6 / AC7 / AC9 / AC10 / AC16. AC5 / AC8 live in existing suites.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn hermetic_harness_cmd(home: &Path) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin_no_key();
    cmd.env("USERPROFILE", home);
    cmd.env("HOME", home);
    cmd.env("PATH", "");
    cmd
}

fn hermetic_vault() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic_vault()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic_vault()
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
    hermetic_vault()
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
    hermetic_vault()
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

    hermetic_vault()
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
        .args(["config", "user.email", "t267@example.com"])
        .current_dir(repo)
        .env("GIT_TERMINAL_PROMPT", "0")
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "T267 Test"])
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

fn example_line(stderr: &str) -> String {
    stderr
        .lines()
        .find(|l| l.contains("Example: ai-brains project set-alias"))
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// AC2 / AC3 / AC9 — all five present + wiring=ok
// ---------------------------------------------------------------------------

fn install_all_ready(home: &Path) {
    let out = hermetic_harness_cmd(home)
        .args(["harness", "install", "--harness", "all-ready", "--yes"])
        .output()
        .expect("all-ready --yes");
    assert_eq!(
        out.status.code(),
        Some(0),
        "all-ready --yes must exit 0; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn harness_status__all_ok__omits_self_next() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    install_all_ready(home);

    let out = hermetic_harness_cmd(home)
        .args(["harness", "status"])
        .output()
        .expect("harness status");
    assert_eq!(
        out.status.code(),
        Some(0),
        "status must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("next: ai-brains harness status"),
        "AC2: ok rows must omit next: harness status; got: {stdout}"
    );
}

#[test]
fn harness_status__all_ok__json_next_action_none() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    install_all_ready(home);

    let out = hermetic_harness_cmd(home)
        .args(["harness", "status", "--format", "json"])
        .output()
        .expect("harness status json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "json status must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], 1);
    let harnesses = v["harnesses"].as_array().expect("harnesses");
    assert_eq!(harnesses.len(), 5, "five harness rows; got {harnesses:?}");
    for h in harnesses {
        assert!(h.get("id").is_some(), "id key required; row={h}");
        assert!(h.get("wiring").is_some(), "wiring key required; row={h}");
        assert!(
            h.get("next_action").is_some(),
            "next_action key required; row={h}"
        );
        if h["wiring"] == "ok" {
            assert_eq!(
                h["next_action"], "none",
                "AC3: ok next_action must be none; row={h}"
            );
        }
    }
}

#[test]
fn harness_status__all_ok__omits_ready_trailers() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    install_all_ready(home);

    let out = hermetic_harness_cmd(home)
        .args(["harness", "status"])
        .output()
        .expect("harness status");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "stdout={stdout}");
    assert!(
        !stdout.contains("ready: ai-brains harness install"),
        "AC9: no ready-trailer install lines when every present is ok; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — one present + missing wiring still names install --dry-run
// ---------------------------------------------------------------------------

#[test]
fn harness_status__present_missing__next_is_install() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    fs::create_dir_all(home.join(".grok")).expect("present grok home");

    let out = hermetic_harness_cmd(home)
        .args(["harness", "status"])
        .output()
        .expect("harness status missing");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("next: ai-brains harness install --harness grok --dry-run"),
        "AC4: missing grok must print install remediator; got: {stdout}"
    );
    assert!(
        stdout.contains("Grok ready: ai-brains harness install --harness grok --dry-run")
            || stdout.contains("install --harness grok --dry-run"),
        "AC9 sibling: missing id still prints install remediator; got: {stdout}"
    );

    let json_out = hermetic_harness_cmd(home)
        .args(["harness", "status", "--format", "json"])
        .output()
        .expect("harness status json missing");
    let v: serde_json::Value = serde_json::from_slice(&json_out.stdout).expect("valid JSON");
    let grok = v["harnesses"]
        .as_array()
        .expect("harnesses")
        .iter()
        .find(|h| h["id"] == "grok")
        .expect("grok row");
    assert_eq!(grok["wiring"], "missing");
    assert_eq!(
        grok["next_action"], "ai-brains harness install --harness grok --dry-run",
        "AC4 JSON next_action is the install command; row={grok}"
    );
}

// ---------------------------------------------------------------------------
// AC10 — install success still nexts status (F7 freeze)
// ---------------------------------------------------------------------------

#[test]
fn harness_install__success__next_is_status() {
    let dir = tempdir().unwrap();
    let home = dir.path();
    let out = hermetic_harness_cmd(home)
        .args(["harness", "install", "--harness", "grok", "--yes"])
        .output()
        .expect("grok --yes");
    assert_eq!(
        out.status.code(),
        Some(0),
        "install must exit 0; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("next: ai-brains harness status"),
        "AC10: install success next is status; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC6 — cwd owner unaliased + leftover high-memory; footer names owner + slug
// ---------------------------------------------------------------------------

#[test]
fn project_list__footer__cwd_owner_unaliased__picks_owner_slug() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.path().join("crawlx");
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

    let owner_dir = dir.path().join("owner-repo");
    git_init_with_origin(&owner_dir, "https://github.com/user/AI-Brains.git");
    let owner_id = register_project(&vault, &owner_dir);
    register_path(&vault, &owner_id, owner_dir.to_str().expect("utf8"));

    let out = hermetic_vault()
        .current_dir(&owner_dir)
        .env("GIT_TERMINAL_PROMPT", "0")
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    let example = example_line(&stderr);
    assert!(
        example.contains(&owner_id) && example.contains("AI-Brains"),
        "AC6: Example must name path-owner + slug; got: {example}"
    );
    assert!(
        !example.contains(&leftover_id),
        "AC6: Example must not pair leftover with slug; got: {example}"
    );
}

// ---------------------------------------------------------------------------
// AC7 — multi-path leftover + aliased owner + orphan → pick orphan
// ---------------------------------------------------------------------------

#[test]
fn project_list__footer__multipath_leftover_plus_orphan__picks_orphan() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.path().join("crawlx");
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
    let extra = dir.path().join("dedupe");
    fs::create_dir_all(&extra).expect("dedupe dir");
    register_path(&vault, &leftover_id, extra.to_str().expect("utf8"));

    let owner_dir = dir.path().join("owner-repo");
    git_init_with_origin(&owner_dir, "https://github.com/user/AI-Brains.git");
    let owner_id = register_project(&vault, &owner_dir);
    register_path(&vault, &owner_id, owner_dir.to_str().expect("utf8"));
    set_alias(&vault, &owner_id, "this-repo");

    let orphan_dir = dir.path().join("orphan-proj");
    let orphan_id = register_project(&vault, &orphan_dir);

    let out = hermetic_vault()
        .current_dir(&owner_dir)
        .env("GIT_TERMINAL_PROMPT", "0")
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    let example = example_line(&stderr);
    assert!(
        example.contains(&orphan_id),
        "AC7: Example must name the orphan; got: {example}"
    );
    assert!(
        !example.contains(&format!("set-alias {leftover_id}")),
        "AC7: must not suggest leftover; got: {example}"
    );
    assert!(
        !example.contains(&format!("{leftover_id} AI-Brains")),
        "AC7: must not pair leftover+AI-Brains; got: {example}"
    );
}

// ---------------------------------------------------------------------------
// AC16 — leftover-only last resort uses path basename, not cwd slug
// ---------------------------------------------------------------------------

#[test]
fn project_list__footer__leftover_only__basename_not_cwd_slug() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.path().join("crawlx");
    let leftover_id = register_project(&vault, &leftover_dir);
    register_path(&vault, &leftover_id, leftover_dir.to_str().expect("utf8"));
    pin_memory(
        &vault,
        &leftover_dir,
        &leftover_id,
        "DECISION: leftover dump seed",
    );

    let owner_dir = dir.path().join("owner-repo");
    git_init_with_origin(&owner_dir, "https://github.com/user/AI-Brains.git");
    let owner_id = register_project(&vault, &owner_dir);
    register_path(&vault, &owner_id, owner_dir.to_str().expect("utf8"));
    set_alias(&vault, &owner_id, "this-repo");

    let out = hermetic_vault()
        .current_dir(&owner_dir)
        .env("GIT_TERMINAL_PROMPT", "0")
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    let example = example_line(&stderr);
    assert!(
        example.contains(&format!("set-alias {leftover_id} crawlx")),
        "AC16: leftover-only suggestion is path basename; got: {example}"
    );
    assert!(
        !example.contains(&format!("set-alias {leftover_id} AI-Brains")),
        "AC16: must not use cwd slug for leftover; got: {example}"
    );
}

/// CX1 P1: missing `git` must not fail `project list` (best-effort identity).
#[test]
fn project_list__footer__git_unavailable__exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let leftover_dir = dir.path().join("crawlx");
    let leftover_id = register_project(&vault, &leftover_dir);
    register_path(&vault, &leftover_id, leftover_dir.to_str().expect("utf8"));

    let out = hermetic_vault()
        .current_dir(&leftover_dir)
        .env("PATH", "")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list no git");
    assert_eq!(
        out.status.code(),
        Some(0),
        "missing git must not fail list; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    let example = example_line(&stderr);
    assert!(
        example.contains(&leftover_id),
        "footer still names the unaliased project; got: {example}"
    );
}
