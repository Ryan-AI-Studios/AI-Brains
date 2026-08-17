//! T257 — identity warning + JSON stdout hygiene hermetic suite.
//!
//! Red on today's tree: JSON-effective commands still eprintln T240 SOOT
//! and `scope resolve` JSON `warnings[]` is empty. T240 list hermetic
//! (AC4) stays in `project_identity_convergence.rs`.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const SOOT_PHRASE: &str = "project identity mismatch";
const TOKEN_PREFIX: &str = "project_identity_mismatch";

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

struct MismatchFixture {
    _dir: tempfile::TempDir,
    vault: PathBuf,
    work: PathBuf,
    id_env: String,
    id_path: String,
}

fn mismatch_fixture() -> MismatchFixture {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_env = dir.path().join("env-proj");
    let id_env = register_project(&vault, &proj_env);

    let proj_path = dir.path().join("path-proj");
    let id_path = register_project(&vault, &proj_path);

    let work = dir.path().join("work");
    fs::create_dir_all(&work).expect("work dir");
    register_path(&vault, &id_path, work.to_str().expect("utf8"));
    fs::write(
        work.join(".env"),
        format!("AI_BRAINS_PROJECT_ID={id_env}\n"),
    )
    .expect("write work .env");

    MismatchFixture {
        _dir: dir,
        vault,
        work,
        id_env,
        id_path,
    }
}

fn expected_token(id_env: &str, id_path: &str) -> String {
    format!("{TOKEN_PREFIX} env={id_env} path={id_path}")
}

fn run_in_fixture(fx: &MismatchFixture, args: &[&str]) -> std::process::Output {
    let mut cmd = hermetic();
    cmd.arg("--vault-path").arg(&fx.vault).current_dir(&fx.work);
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("hermetic command")
}

fn stdout_str(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_str(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn assert_exit_0(out: &std::process::Output, label: &str) {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{label} exit; stderr={}",
        stderr_str(out)
    );
}

fn assert_no_soot(text: &str, stream: &str) {
    assert!(
        !text.contains(SOOT_PHRASE) && !text.contains("Warning: project identity"),
        "{stream} must not contain T240 SOOT; got: {text}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — scope JSON: parse, token, no SOOT on either stream
// ---------------------------------------------------------------------------

#[test]
fn scope_resolve_json__mismatch__stdout_parses_token_no_soot() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["scope", "resolve", "--format", "json", "--local"]);
    assert_exit_0(&out, "scope resolve --format json");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    assert_no_soot(&stderr, "stderr");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout one JSON object");
    let warnings = v
        .get("warnings")
        .and_then(|w| w.as_array())
        .expect("warnings array");
    let token = expected_token(&fx.id_env, &fx.id_path);
    assert!(
        warnings.iter().any(|w| w.as_str() == Some(token.as_str())),
        "warnings must contain {token}; got: {warnings:?}"
    );
    assert!(
        !stdout.contains("Warning:"),
        "stdout must not contain Warning:; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC9 — concat(stdout, stderr) parses as one JSON value
// ---------------------------------------------------------------------------

#[test]
fn scope_resolve_json__mismatch__concat_streams_parse() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["scope", "resolve", "--format", "json", "--local"]);
    assert_exit_0(&out, "scope resolve --format json");
    let combined = format!("{}{}", stdout_str(&out), stderr_str(&out));
    serde_json::from_str::<serde_json::Value>(combined.trim())
        .expect("AC9: concat(stdout, stderr) must parse as one JSON value");
}

// ---------------------------------------------------------------------------
// AC5 — whoami JSON: mismatch true, no stderr SOOT
// ---------------------------------------------------------------------------

#[test]
fn whoami_json__mismatch__no_stderr_soot() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["project", "whoami", "--format", "json"]);
    assert_exit_0(&out, "project whoami --format json");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    assert_no_soot(&stderr, "stderr");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("whoami JSON");
    assert_eq!(
        v.get("mismatch").and_then(|x| x.as_bool()),
        Some(true),
        "mismatch true; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC6 — whoami human: no SOOT either stream; remediations name adopt-path
// ---------------------------------------------------------------------------

#[test]
fn whoami_human__mismatch__no_stderr_soot() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["project", "whoami", "--format", "human"]);
    assert_exit_0(&out, "project whoami --format human");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    assert_no_soot(&stderr, "stderr");
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("adopt-path"),
        "human remediations must name adopt-path; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// AC7 — nightly status JSON: parse, no SOOT, no warnings key
// ---------------------------------------------------------------------------

#[test]
fn nightly_status_json__mismatch__no_soot_no_warnings_key() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["nightly", "--status", "--format", "json", "--quick"]);
    assert_exit_0(&out, "nightly --status --format json --quick");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    assert_no_soot(&stderr, "stderr");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("nightly JSON");
    assert!(
        v.get("warnings").is_none(),
        "F15: nightly status must not grow a warnings key; got: {v}"
    );
}

// ---------------------------------------------------------------------------
// AC14 — scope human: stdout clean, stderr still has T240 SOOT
// ---------------------------------------------------------------------------

#[test]
fn scope_resolve_human__mismatch__stderr_soot_stdout_clean() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["scope", "resolve", "--format", "human", "--local"]);
    assert_exit_0(&out, "scope resolve --format human");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    assert!(
        stderr.contains(SOOT_PHRASE),
        "AC14: human non-remediator must still print T240 SOOT on stderr; got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC8 — dry-run preview stdout has no SOOT; warn not between preview lines
// ---------------------------------------------------------------------------

#[test]
fn nightly_schedule_dry_run__stdout_preview_has_no_soot() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(&fx, &["nightly", "--schedule", "--dry-run"]);
    assert_exit_0(&out, "nightly --schedule --dry-run");
    let stdout = stdout_str(&out);
    let stderr = stderr_str(&out);
    assert_no_soot(&stdout, "stdout");
    if stderr.contains(SOOT_PHRASE) {
        let lines: Vec<&str> = stdout.lines().collect();
        assert!(
            lines.len() >= 2,
            "dry-run preview is two lines; got: {stdout:?}"
        );
        assert!(
            lines[0].contains("[dry-run] Would execute:"),
            "first preview line; got: {stdout}"
        );
        assert!(
            lines[1].contains("schtasks"),
            "second preview line; got: {stdout}"
        );
        assert!(
            !lines.iter().any(|l| l.contains(SOOT_PHRASE)),
            "SOOT must not sit between preview lines; stdout={stdout}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC13 — --no-project-context and --global still skip T240 SOOT
// ---------------------------------------------------------------------------

#[test]
fn project_list__no_project_context__no_soot() {
    let fx = mismatch_fixture();
    let out = {
        let mut cmd = hermetic();
        cmd.arg("--no-project-context")
            .arg("--vault-path")
            .arg(&fx.vault)
            .current_dir(&fx.work)
            .arg("project")
            .arg("list");
        cmd.output().expect("list npc")
    };
    assert_exit_0(&out, "project list --no-project-context");
    assert_no_soot(&stderr_str(&out), "stderr");
    assert_no_soot(&stdout_str(&out), "stdout");
}

#[test]
fn recall_global__mismatch__no_soot() {
    let fx = mismatch_fixture();
    let out = run_in_fixture(
        &fx,
        &[
            "recall",
            "zzzz-t257-no-hit",
            "--global",
            "--format",
            "pretty",
            "--no-bridge",
            "--quiet",
        ],
    );
    assert_exit_0(&out, "recall --global");
    assert_no_soot(&stderr_str(&out), "stderr");
    assert_no_soot(&stdout_str(&out), "stdout");
}
