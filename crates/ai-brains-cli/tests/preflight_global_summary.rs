#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T214 — Preflight global rollup honesty hermetic suite (AC1–AC5, AC9, AC12).
//!
//! Pattern: tempdir vault + context for multi-project registration + pin;
//! hermetic_bin + isolate_empty_home + `--no-project-context`.

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

fn run_summary(
    vault: &Path,
    preflight_args: &[&str],
    project_env: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault);
    if let Some(pid) = project_env {
        cmd.env("AI_BRAINS_PROJECT_ID", pid);
    }
    // Subcommand flags must follow `preflight` (clap per-command args).
    cmd.arg("preflight");
    for a in preflight_args {
        cmd.arg(a);
    }
    cmd.arg("--summary");
    let out = cmd.output().expect("preflight --summary");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

fn parse_u64_after(stdout: &str, label: &str) -> Option<u64> {
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix(label) {
            return rest.trim().parse().ok();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// AC1 + AC2 + AC5 + AC12 — global summary honesty
// ---------------------------------------------------------------------------

#[test]
fn preflight_global_summary__multi_project__scope_global_and_vault_rollup() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let proj_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "DECISION: use dual vault/in-context counts",
    );
    pin_memory(
        &vault,
        &proj_b,
        &id_b,
        "DECISION: Scope vocabulary is T207 SOOT",
    );

    // AC12: env project still set under --global must not win the label.
    let (code, stdout, stderr) = run_summary(&vault, &["--global"], Some(&id_a));
    assert_eq!(code, 0, "global summary exit 0; stderr={stderr}");
    assert!(
        stdout.contains("Scope: global"),
        "AC1: must contain Scope: global; got:\n{stdout}"
    );
    // Must not present Project: <uuid> as scope identity.
    assert!(
        !stdout.lines().any(|l| l.starts_with("Project:")),
        "AC1: no legacy Project: scope line; got:\n{stdout}"
    );
    // AC2 vault rollup
    let projects =
        parse_u64_after(&stdout, "Projects:").expect("Projects: line required under --global");
    assert!(
        projects >= 2,
        "AC2: Projects >= 2 under multi-project pins; got {projects}\n{stdout}"
    );
    let pinned = parse_u64_after(&stdout, "Pinned memories:").expect("Pinned memories: line");
    assert!(
        pinned >= 2,
        "AC2: Pinned memories >= 2; got {pinned}\n{stdout}"
    );
    // AC5 in-context labels
    assert!(
        stdout.contains("In context") || stdout.contains("In-context"),
        "AC5: In context / In-context labels required; got:\n{stdout}"
    );
    // AC12: Scope remains global despite env project id
    assert!(
        !stdout.contains(&format!("Scope: project={id_a}")),
        "AC12: env project must not become Scope under --global; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — project-scoped summary: Scope project=, no Projects: line
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary__project_scoped__filtered_pinned_no_projects_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let proj_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "DECISION: only A pin for scoped test",
    );
    pin_memory(&vault, &proj_a, &id_a, "CONSTRAINT: A constraint");
    pin_memory(&vault, &proj_b, &id_b, "DECISION: B only");

    let (code, stdout, stderr) = run_summary(&vault, &[], Some(&id_a));
    assert_eq!(code, 0, "project summary exit 0; stderr={stderr}");
    assert!(
        stdout.contains("Scope: project=") && stdout.contains(&id_a),
        "AC3: Scope: project= must include scoped id; got:\n{stdout}"
    );
    assert!(
        !stdout.lines().any(|l| l.starts_with("Projects:")),
        "AC3: no Projects: line when project-scoped; got:\n{stdout}"
    );
    let pinned = parse_u64_after(&stdout, "Pinned memories:").expect("Pinned memories:");
    assert_eq!(
        pinned, 2,
        "AC3: pinned count must reflect project A only (2); got {pinned}\n{stdout}"
    );
    assert!(
        !stdout.lines().any(|l| l.starts_with("Project:")),
        "no legacy Project: line; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — active sessions from SessionStarted (context creates one)
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary__after_context__active_sessions_at_least_one() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-sess");
    let id = register_project(&vault, &proj);
    // context already started a session (SessionStarted → status=active)

    let (code, stdout, stderr) = run_summary(&vault, &["--global"], Some(&id));
    assert_eq!(code, 0, "stderr={stderr}");
    let sessions = parse_u64_after(&stdout, "Active sessions:").expect("Active sessions: line");
    assert!(
        sessions >= 1,
        "AC4: Active sessions >= 1 after context; got {sessions}\n{stdout}"
    );
    // Must not be the dead "Session ID:" text-match path (always 0).
    assert!(
        !stdout.contains("Session ID:"),
        "must not use Session ID: text marker; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC9 — empty vault (init only, no pins)
// ---------------------------------------------------------------------------

#[test]
fn preflight_global_summary__init_only_empty__zeros_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_summary(&vault, &["--global"], None);
    assert_eq!(code, 0, "AC9: exit 0 on empty vault; stderr={stderr}");
    assert!(!stdout.trim().is_empty(), "AC9: non-empty stdout required");
    assert!(
        stdout.contains("Scope: global"),
        "AC9: Scope global; got:\n{stdout}"
    );
    assert_eq!(
        parse_u64_after(&stdout, "Projects:"),
        Some(0),
        "AC9: Projects 0; got:\n{stdout}"
    );
    assert_eq!(
        parse_u64_after(&stdout, "Pinned memories:"),
        Some(0),
        "AC9: Pinned 0; got:\n{stdout}"
    );
    assert_eq!(
        parse_u64_after(&stdout, "Active sessions:"),
        Some(0),
        "AC9: Active sessions 0; got:\n{stdout}"
    );
}
