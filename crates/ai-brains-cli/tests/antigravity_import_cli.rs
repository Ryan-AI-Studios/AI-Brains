//! T370 AC3/AC9 — hermetic `antigravity-import` this-project vs `--global`.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

const CID_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa61";
const CID_B: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa62";
const BRAIN_BODY: &str = r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nkeep-me\n</USER_REQUEST>","tool_calls":[]}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","content":"ok","tool_calls":[]}
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

fn write_brain(home: &Path, cid: &str, workspace: &str) {
    let logs = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("brain")
        .join(cid)
        .join(".system_generated")
        .join("logs");
    fs::create_dir_all(&logs).expect("mkdir logs");
    let path = logs.join("transcript.jsonl");
    fs::write(&path, BRAIN_BODY).expect("write transcript");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = fs::File::options()
        .write(true)
        .open(&path)
        .and_then(|f| f.set_modified(past));

    let hist_dir = home.join(".gemini").join("antigravity-cli");
    fs::create_dir_all(&hist_dir).expect("hist dir");
    let hist = hist_dir.join("history.jsonl");
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hist)
        .expect("open hist");
    writeln!(
        f,
        r#"{{"display":"t","timestamp":1000,"workspace":{},"conversationId":"{}"}}"#,
        serde_json::to_string(workspace).expect("ws json"),
        cid
    )
    .expect("write hist");
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
        if let Some(rest) = line.strip_prefix("[Antigravity] Import stats: found=") {
            let n = rest.split_whitespace().next()?;
            return n.parse().ok();
        }
    }
    None
}

#[test]
fn antigravity_import__this_project_history__found_one_global_two() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    let pid = register_project(&vault, &work);
    register_path(&vault, &pid, r"C:\dev\ai-brains");

    let home = root.path().join("user-home");
    write_brain(&home, CID_A, r"C:\dev\AI-Brains");
    write_brain(&home, CID_B, r"C:\dev\other");

    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("USERPROFILE", &home)
        .env("HOME", &home)
        .arg("antigravity-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");
    assert!(stderr.contains("scope=this-project"), "stderr={stderr}");

    let mut cov = common::hermetic_bin();
    strip_harness_homes(&mut cov);
    let cov_out = cov
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("USERPROFILE", &home)
        .env("HOME", &home)
        .arg("capture")
        .arg("coverage")
        .arg("--format")
        .arg("json")
        .arg("--days")
        .arg("30")
        .output()
        .expect("coverage");
    let cov_stdout = String::from_utf8_lossy(&cov_out.stdout);
    assert!(cov_out.status.success(), "cov={}", cov_stdout);
    let json: serde_json::Value = serde_json::from_str(&cov_stdout).expect("json");
    let agy = json["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .find(|s| s["source"] == "agy")
        .expect("agy");
    assert_eq!(agy["vault_sessions"].as_u64(), Some(0), "agy={agy}");

    let mut cmd_g = common::hermetic_bin();
    strip_harness_homes(&mut cmd_g);
    let out_g = cmd_g
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("USERPROFILE", &home)
        .env("HOME", &home)
        .arg("antigravity-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .arg("--global")
        .output()
        .expect("run global");
    let stderr_g = String::from_utf8_lossy(&out_g.stderr);
    assert!(out_g.status.success(), "stderr={stderr_g}");
    assert!(
        found_from_stderr(&stderr_g).is_some_and(|n| n >= 2),
        "stderr={stderr_g}"
    );
    assert!(stderr_g.contains(CID_A), "stderr={stderr_g}");
    assert!(stderr_g.contains(CID_B), "stderr={stderr_g}");
    assert!(stderr_g.contains("scope=machine-wide"), "stderr={stderr_g}");
}

#[test]
fn antigravity_import__help__mentions_global_and_dry_run() {
    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .arg("antigravity-import")
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "stdout={stdout}");
    assert!(stdout.contains("--global"), "stdout={stdout}");
    assert!(stdout.contains("--dry-run"), "stdout={stdout}");
}

#[test]
fn antigravity_import__pid_without_aliases__scopes_cwd_not_global() {
    let root = tempdir().expect("root");
    let vault = root.path().join("v.db");
    init_vault(&vault);
    let work = root.path().join("work");
    let pid = register_project(&vault, &work);

    let home = root.path().join("user-home");
    let ws = work.to_string_lossy();
    write_brain(&home, CID_A, &ws);
    write_brain(&home, CID_B, r"C:\dev\other");

    let mut cmd = common::hermetic_bin();
    strip_harness_homes(&mut cmd);
    let out = cmd
        .current_dir(&work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &pid)
        .env("USERPROFILE", &home)
        .env("HOME", &home)
        .arg("antigravity-import")
        .arg("--days")
        .arg("30")
        .arg("--dry-run")
        .output()
        .expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stderr={stderr}");
    assert_eq!(found_from_stderr(&stderr), Some(1), "stderr={stderr}");
}
