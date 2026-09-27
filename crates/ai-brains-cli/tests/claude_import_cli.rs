//! T368 AC3 — hermetic `claude-import` this-project vs `--global`.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use ai_brains_adapters::encode_claude_project_folder;
use ai_brains_path::normalize_for_location_compare;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

const SID_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa11";
const SID_B: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa12";
const JSONL: &str = r#"{"type":"user","message":{"role":"user","content":"hi"}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ok"}]}}
"#;

fn init_vault(vault: &Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault)
        .arg("--no-project-context")
        .arg("init")
        .assert()
        .success();
}

fn strip_harness_homes(cmd: &mut assert_cmd::Command) {
    cmd.env_remove("CURSOR_HOME");
    cmd.env_remove("GROK_HOME");
    cmd.env_remove("CLAUDE_HOME");
    cmd.env_remove("CODEX_HOME");
}

fn write_session(claude_home: &Path, folder: &str, sid: &str) {
    let dir = claude_home.join("projects").join(folder);
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join(format!("{sid}.jsonl"));
    fs::write(&path, JSONL).expect("write");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = fs::File::options()
        .write(true)
        .open(&path)
        .and_then(|f| f.set_modified(past));
}

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work");
    let out = common::hermetic_bin()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let content = fs::read_to_string(work_dir.join(".env")).expect(".env");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            return rest.trim().to_string();
        }
    }
    panic!("no project id in .env: {content}");
}

fn register_path(vault: &Path, project_id: &str, path: &str) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_id)
        .arg(path)
        .assert()
        .success();
}

fn found_from_stderr(stderr: &str) -> Option<usize> {
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("[Claude] Import stats: found=") {
            let n = rest.split_whitespace().next()?;
            return n.parse().ok();
        }
    }
    None
}

#[test]
fn claude_import__this_project_hyphen_folder__found_one_global_two() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    let pid = register_project(&vault, &work);
    register_path(&vault, &pid, r"C:\dev\ai-brains");

    let claude_home = root.path().join("claude-home");
    write_session(&claude_home, "C--dev-AI-Brains", SID_A);
    write_session(&claude_home, "C--dev-other", SID_B);

    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("CLAUDE_HOME", &claude_home)
        .arg("claude-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");
    assert!(stderr.contains("scope=this-project"), "stderr={stderr}");

    let mut cmd_g = common::hermetic_bin();
    strip_harness_homes(&mut cmd_g);
    let out_g = cmd_g
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("CLAUDE_HOME", &claude_home)
        .arg("claude-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .arg("--global")
        .output()
        .expect("run global");
    let stderr_g = String::from_utf8_lossy(&out_g.stderr);
    assert!(out_g.status.success(), "stderr={stderr_g}");
    assert_eq!(found_from_stderr(&stderr_g), Some(2), "stderr={stderr_g}");
    assert!(stderr_g.contains("scope=machine-wide"), "stderr={stderr_g}");
}

#[test]
fn claude_import__help__mentions_global() {
    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .arg("claude-import")
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "stdout={stdout}");
    assert!(stdout.contains("--global"), "stdout={stdout}");
}

#[test]
fn claude_import__pid_without_aliases__scopes_cwd_not_global() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    let pid = register_project(&vault, &work);

    let claude_home = root.path().join("claude-home");
    let folder =
        encode_claude_project_folder(&normalize_for_location_compare(&work.to_string_lossy()));
    write_session(&claude_home, &folder, SID_A);
    write_session(&claude_home, "C--dev-other", SID_B);

    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("CLAUDE_HOME", &claude_home)
        .arg("claude-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");
}
