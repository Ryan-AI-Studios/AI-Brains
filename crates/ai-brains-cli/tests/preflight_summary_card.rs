//! T345 — `preflight --summary` session card (path / leftover / last_decision / one next).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_path::normalize_for_location_compare;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const T315_SOOT: &str = r#"next: ai-brains recall "what did we decide""#;
const CONTEXT_NEXT: &str = "next: ai-brains context";
const SHELL_ID: &str = "7d97a456-f2f4-43ea-1f13-211af684ad37";
const FILE_ID: &str = "3581317d-601e-44f7-ab84-fde90aa12d3c";
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

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .arg("--no-auto-bind")
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
            assert!(!id.is_empty());
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from {env_path:?}");
}

fn register_path(vault: &Path, work_dir: &Path, project_id: &str) {
    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_id)
        .arg(work_dir)
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

fn next_lines(stdout: &str) -> Vec<&str> {
    stdout.lines().filter(|l| l.starts_with("next:")).collect()
}

fn card_line_count_before_harness(stdout: &str) -> usize {
    let mut n = 0usize;
    for line in stdout.lines() {
        if line.starts_with("Harnesses installed on machine:") {
            break;
        }
        n += 1;
    }
    n
}

fn run_summary(
    vault: &Path,
    work_dir: &Path,
    extra: &[&str],
    project_id: Option<&str>,
    shell_override: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault);
    if let Some(pid) = project_id {
        cmd.env("AI_BRAINS_PROJECT_ID", pid);
    }
    if let Some(shell) = shell_override {
        cmd.env("AI_BRAINS_PROJECT_ID", shell);
    }
    cmd.arg("preflight")
        .arg("--summary")
        .arg("--no-hook-prompt");
    for a in extra {
        cmd.arg(a);
    }
    let out = cmd.output().expect("preflight --summary");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn preflight_summary_card__path_unbound__next_context() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("unbound");
    let id = register_project(&vault, &proj);

    let (code, stdout, stderr) = run_summary(&vault, &proj, &[], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(stdout.contains("path=—"), "AC1 path=—; got:\n{stdout}");
    let nexts = next_lines(&stdout);
    assert_eq!(nexts.len(), 1, "exactly one next:; got:\n{stdout}");
    assert_eq!(nexts[0], CONTEXT_NEXT);
    assert!(
        !stdout.contains(T315_SOOT),
        "unbound must not T315; got:\n{stdout}"
    );
}

#[test]
fn preflight_summary_card__path_bound__json_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("bound");
    let id = register_project(&vault, &proj);
    register_path(&vault, &proj, &id);

    let (code, stdout, stderr) = run_summary(&vault, &proj, &["--format", "json"], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("json");
    let path = v["path"].as_str().expect("path string");
    let expected = normalize_for_location_compare(&proj.to_string_lossy());
    assert_eq!(path, expected, "AC2 compare-normalized path; got {v}");
}

#[test]
fn preflight_summary_card__path_unbound__json_null_key_present() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("unbound-json");
    let id = register_project(&vault, &proj);

    let (code, stdout, stderr) = run_summary(&vault, &proj, &["--format", "json"], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("json");
    assert!(v.get("path").is_some(), "AC2b path key; got {v}");
    assert!(v["path"].is_null(), "AC2b path null; got {v}");
    assert_eq!(v["api_version"], "1");
}

#[test]
fn preflight_summary_card__path_other_owner__unbound_next_context() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let mine = dir.path().join("mine");
    let other = dir.path().join("other");
    let pid_mine = register_project(&vault, &mine);
    let pid_other = register_project(&vault, &other);
    register_path(&vault, &mine, &pid_other);

    let (code, stdout, stderr) = run_summary(&vault, &mine, &[], Some(&pid_mine), None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.contains("path=—"),
        "F2 other-owner path=—; got:\n{stdout}"
    );
    let nexts = next_lines(&stdout);
    assert_eq!(nexts.len(), 1, "got:\n{stdout}");
    assert_eq!(nexts[0], CONTEXT_NEXT);
}

#[test]
fn preflight_summary_card__last_decision_truncates() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("decision");
    let id = register_project(&vault, &proj);
    register_path(&vault, &proj, &id);
    pin_memory(&vault, &proj, &id, "DECISION: Foo bar unique T345 card");

    let (code, stdout, stderr) = run_summary(&vault, &proj, &[], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.contains("last_decision: Foo bar unique T345 card"),
        "AC5 human last_decision; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("last_decision: DECISION:"),
        "AC5 no doubled marker; got:\n{stdout}"
    );

    let (jcode, jstdout, jstderr) =
        run_summary(&vault, &proj, &["--format", "json"], Some(&id), None);
    assert_eq!(jcode, 0, "stderr={jstderr}");
    let v: Value = serde_json::from_str(jstdout.trim()).expect("json");
    assert_eq!(
        v["last_decision"].as_str(),
        Some("Foo bar unique T345 card"),
        "AC5 json last_decision; got {v}"
    );
}

#[test]
fn preflight_summary_card__grants_win_over_t315() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("grants");
    let id = register_project(&vault, &proj);
    register_path(&vault, &proj, &id);

    let (code, stdout, stderr) = run_summary(&vault, &proj, &[], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");
    let nexts = next_lines(&stdout);
    assert_eq!(nexts.len(), 1, "AC3 exactly one next:; got:\n{stdout}");
    assert!(
        nexts[0].contains("policy bootstrap"),
        "AC3 bootstrap SOOT; got:\n{stdout}"
    );
    assert_ne!(nexts[0], CONTEXT_NEXT);
    assert_ne!(nexts[0], T315_SOOT);
    assert!(
        !stdout.contains("discovery grants empty")
            && !stdout.contains("discovery grants incomplete"),
        "F7 no grants-incomplete human line; got:\n{stdout}"
    );
}

#[test]
fn preflight_summary_card__show_unchanged() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("show");
    fs::create_dir_all(&proj).expect("proj");
    fs::write(
        proj.join(".env"),
        format!("AI_BRAINS_PROJECT_ID={FILE_ID}\nAI_BRAINS_KEY={ZERO_KEY}\n"),
    )
    .expect(".env");

    let out = hermetic()
        .current_dir(&proj)
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .arg("--show")
        .output()
        .expect("context --show");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("path="), "AC6 no path=; got:\n{stdout}");
    assert!(
        !stdout.contains("last_decision:"),
        "AC6 no last_decision; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("--- AI-Brains Preflight Summary ---"),
        "AC6 no preflight banner; got:\n{stdout}"
    );
}

#[test]
fn preflight_summary_card__doctor_preflight__no_bind() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("nobind");
    let id = register_project(&vault, &proj);
    let before_path = count_events(&vault, "RepositoryPathAliasAdded");
    let before_alias = count_events(&vault, "ProjectAliasAdded");

    let (code, _, stderr) = run_summary(&vault, &proj, &[], Some(&id), None);
    assert_eq!(code, 0, "stderr={stderr}");

    hermetic()
        .current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .arg("doctor")
        .arg("--summary")
        .assert()
        .success();

    let after_path = count_events(&vault, "RepositoryPathAliasAdded");
    let after_alias = count_events(&vault, "ProjectAliasAdded");
    assert_eq!(
        after_path, before_path,
        "AC8 preflight/doctor must not bind path"
    );
    assert_eq!(
        after_alias, before_alias,
        "AC8 preflight/doctor must not bind alias"
    );
}

#[test]
fn preflight_summary_card__leftover_shell_vs_file__exact_line() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("leftover");
    let id = register_project(&vault, &proj);

    let mut cmd = hermetic();
    cmd.current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", SHELL_ID)
        .arg("preflight")
        .arg("--summary")
        .arg("--no-hook-prompt")
        .arg("--format")
        .arg("json");
    let out = cmd.output().expect("leftover json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: Value = serde_json::from_str(stdout.trim()).expect("json");
    assert_eq!(
        v["shell_leftover_project_id"].as_str(),
        Some(SHELL_ID),
        "AC10 json leftover; got {v}"
    );

    let mut human = hermetic();
    human
        .current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", SHELL_ID)
        .arg("preflight")
        .arg("--summary")
        .arg("--no-hook-prompt");
    let hout = human.output().expect("leftover human");
    let hstdout = String::from_utf8_lossy(&hout.stdout);
    let expected = format!("shell leftover PROJECT_ID: {SHELL_ID} (.env overrides)");
    assert!(
        hstdout.contains(&expected),
        "AC10 exact leftover line; got:\n{hstdout}"
    );
    let _ = id;
}

#[test]
fn preflight_summary_card__human__le_16_lines_before_harness() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("maxlen");
    let id = register_project(&vault, &proj);

    let mut cmd = hermetic();
    cmd.current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", SHELL_ID)
        .arg("preflight")
        .arg("--summary")
        .arg("--no-hook-prompt");
    let out = cmd.output().expect("max card");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let n = card_line_count_before_harness(&stdout);
    assert!(n <= 16, "AC11 ≤16 lines before harness; got {n}:\n{stdout}");
    let _ = id;
}

#[test]
fn preflight_summary_card__global__omits_json_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_summary(
        &vault,
        dir.path(),
        &["--global", "--format", "json"],
        None,
        None,
    );
    assert_eq!(code, 0, "stderr={stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("json");
    assert_eq!(v["scope"], "global");
    assert!(v.get("path").is_none(), "AC7 omit path; got {v}");
    assert!(v.get("pinned").is_some());
    assert_eq!(v["api_version"], "1");
}
