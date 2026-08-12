//! T242 — env override warning session quiet (cross-process fingerprint markers).
//!
//! Each test uses a **per-test** temp home (USERPROFILE+HOME) so multi-spawn
//! shares a marker cache within a test without polluting operator home or
//! the shared `isolate_empty_home` process-lifetime empty home.

// Test-only: expect/unwrap allowed in hermetics (A9); not production.
#![allow(clippy::disallowed_methods)]

mod common;

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const LOCAL_PROJECT: &str = "99999999-9999-9999-9999-999999999999";
const LOCAL_SESSION: &str = "88888888-8888-8888-8888-888888888888";
const SHELL_PROJECT: &str = "77777777-7777-7777-7777-777777777777";
const SHELL_SESSION: &str = "66666666-6666-6666-6666-666666666666";
const SHELL_PROJECT_ALT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

const WARN_PREFIX: &str = "Warning: local .env overrides inherited shell:";

struct Fixture {
    /// Project cwd with `.env` + vault (kept for tempdir lifetime).
    _project: tempfile::TempDir,
    /// Per-test home for marker cache (kept for tempdir lifetime).
    _home: tempfile::TempDir,
    project_path: PathBuf,
    home_path: PathBuf,
    vault_path: PathBuf,
}

fn setup_project_with_env(env_body: &str) -> Fixture {
    let project = tempdir().expect("project tempdir");
    let home = tempdir().expect("home tempdir");
    let project_path = project.path().to_path_buf();
    let home_path = home.path().to_path_buf();
    let vault_path = project_path.join("vault.db");

    fs::write(project_path.join(".env"), env_body).expect("write .env");

    let mut init = hermetic_with_home(&home_path);
    init.current_dir(&project_path)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Fixture {
        _project: project,
        _home: home,
        project_path,
        home_path,
        vault_path,
    }
}

fn both_keys_env() -> String {
    format!("AI_BRAINS_PROJECT_ID={LOCAL_PROJECT}\nAI_BRAINS_SESSION_ID={LOCAL_SESSION}\n")
}

fn hermetic_with_home(home: &Path) -> Command {
    let mut cmd = common::hermetic_bin();
    cmd.env("USERPROFILE", home);
    cmd.env("HOME", home);
    cmd.env_remove("AI_BRAINS_QUIET_ENV_WARN");
    cmd.env_remove("AI_BRAINS_FORCE_ENV_WARN");
    cmd
}

fn run_preflight(
    fx: &Fixture,
    shell_project: &str,
    shell_session: &str,
    extra: &[(&str, &str)],
) -> std::process::Output {
    let mut cmd = hermetic_with_home(&fx.home_path);
    cmd.current_dir(&fx.project_path)
        .env("AI_BRAINS_PROJECT_ID", shell_project)
        .env("AI_BRAINS_SESSION_ID", shell_session)
        .arg("--vault-path")
        .arg(&fx.vault_path)
        .arg("preflight")
        .arg("--summary");
    for (k, v) in extra {
        cmd.env(*k, *v);
    }
    cmd.output().expect("preflight must run")
}

fn warn_count(stderr: &str) -> usize {
    stderr.matches(WARN_PREFIX).count()
}

fn marker_dir(home: &Path) -> PathBuf {
    home.join(".ai-brains")
        .join("cache")
        .join("env-override-warn")
}

fn list_markers(home: &Path) -> Vec<PathBuf> {
    let dir = marker_dir(home);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("read marker dir") {
        let entry = entry.expect("dir entry");
        if entry.file_type().expect("ft").is_file() {
            out.push(entry.path());
        }
    }
    out.sort();
    out
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__first_process__one_warning_and_empty_marker() {
    let fx = setup_project_with_env(&both_keys_env());
    let out = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert!(
        out.status.success(),
        "preflight must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        warn_count(&stderr),
        1,
        "AC1: first process must warn once; got: {stderr}"
    );
    assert!(
        stderr.contains(SHELL_PROJECT) && stderr.contains(SHELL_SESSION),
        "collapsed SOOT must list old shell values; got: {stderr}"
    );

    let markers = list_markers(&fx.home_path);
    assert_eq!(
        markers.len(),
        1,
        "AC1: exactly one marker under temp home; got {markers:?}"
    );
    let meta = fs::metadata(&markers[0]).expect("marker meta");
    assert_eq!(meta.len(), 0, "AC1: marker must be 0 bytes");
    // AC9: marker under temp home, not repo worktree.
    assert!(
        markers[0].starts_with(&fx.home_path),
        "marker must live under redirected home"
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__second_process_same_fingerprint__zero_warnings() {
    let fx = setup_project_with_env(&both_keys_env());
    let first = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert_eq!(
        warn_count(&String::from_utf8_lossy(&first.stderr)),
        1,
        "seed first warn"
    );

    let second = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert!(
        second.status.success(),
        "second preflight must succeed; stderr={}",
        String::from_utf8_lossy(&second.stderr)
    );
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert_eq!(
        warn_count(&stderr),
        0,
        "AC2: second process same fingerprint → 0 Warning; got: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(
        stdout.contains("Scope: project=") && stdout.contains(LOCAL_PROJECT),
        "AC2: force-set still applies (local Scope); got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__changed_shell_project__warns_again() {
    let fx = setup_project_with_env(&both_keys_env());
    let first = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert_eq!(warn_count(&String::from_utf8_lossy(&first.stderr)), 1);

    let changed = run_preflight(&fx, SHELL_PROJECT_ALT, SHELL_SESSION, &[]);
    assert!(changed.status.success());
    let stderr = String::from_utf8_lossy(&changed.stderr);
    assert_eq!(
        warn_count(&stderr),
        1,
        "AC3: fingerprint change must re-warn; got: {stderr}"
    );
    assert!(
        stderr.contains(SHELL_PROJECT_ALT),
        "re-warn must show new shell project; got: {stderr}"
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__quiet_suppresses_even_without_marker() {
    let fx = setup_project_with_env(&both_keys_env());
    let out = run_preflight(
        &fx,
        SHELL_PROJECT,
        SHELL_SESSION,
        &[("AI_BRAINS_QUIET_ENV_WARN", "1")],
    );
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        warn_count(&stderr),
        0,
        "AC4: quiet → 0 Warning without marker; got: {stderr}"
    );
    assert!(
        list_markers(&fx.home_path).is_empty(),
        "quiet path must not write markers"
    );

    // Quiet wins over force.
    let quiet_force = run_preflight(
        &fx,
        SHELL_PROJECT,
        SHELL_SESSION,
        &[
            ("AI_BRAINS_QUIET_ENV_WARN", "1"),
            ("AI_BRAINS_FORCE_ENV_WARN", "1"),
        ],
    );
    assert_eq!(
        warn_count(&String::from_utf8_lossy(&quiet_force.stderr)),
        0,
        "AC4: quiet wins over force"
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__force_warns_despite_existing_marker() {
    let fx = setup_project_with_env(&both_keys_env());
    let first = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert_eq!(warn_count(&String::from_utf8_lossy(&first.stderr)), 1);
    assert_eq!(list_markers(&fx.home_path).len(), 1);

    let forced = run_preflight(
        &fx,
        SHELL_PROJECT,
        SHELL_SESSION,
        &[("AI_BRAINS_FORCE_ENV_WARN", "1")],
    );
    assert!(forced.status.success());
    let stderr = String::from_utf8_lossy(&forced.stderr);
    assert_eq!(
        warn_count(&stderr),
        1,
        "AC5: FORCE with existing marker still warns; got: {stderr}"
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__session_only__no_warning_no_marker() {
    // PROJECT equal to shell; SESSION differs → session-only Debug path.
    let env_body =
        format!("AI_BRAINS_PROJECT_ID={SHELL_PROJECT}\nAI_BRAINS_SESSION_ID={LOCAL_SESSION}\n");
    let fx = setup_project_with_env(&env_body);
    let out = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        warn_count(&stderr),
        0,
        "AC6: session-only → 0 Warning; got: {stderr}"
    );
    assert!(
        list_markers(&fx.home_path).is_empty(),
        "AC6: session-only must not write markers; got {:?}",
        list_markers(&fx.home_path)
    );
}

#[test]
#[allow(non_snake_case)]
fn env_override_session__both_keys__one_collapsed_line_not_legacy() {
    let fx = setup_project_with_env(&both_keys_env());
    let out = run_preflight(&fx, SHELL_PROJECT, SHELL_SESSION, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(warn_count(&stderr), 1, "AC7: one collapsed line");
    assert!(
        !stderr.contains("local .env AI_BRAINS_PROJECT_ID overrides inherited shell value"),
        "AC7: must not use legacy dual template; got: {stderr}"
    );
    assert!(
        stderr.contains("AI_BRAINS_PROJECT_ID") && stderr.contains("AI_BRAINS_SESSION_ID"),
        "AC7: both keys on one line; got: {stderr}"
    );
}

/// AC3 location clause / F4: different `.env` parent dirs must not share a
/// fingerprint even when shell olds and `.env` IDs are identical.
#[test]
#[allow(non_snake_case)]
fn env_override_session__different_env_parent_location__warns_again() {
    let home = tempdir().expect("shared home");
    let home_path = home.path().to_path_buf();

    let project_a = tempdir().expect("project a");
    let project_b = tempdir().expect("project b");
    let vault_a = project_a.path().join("vault.db");
    let vault_b = project_b.path().join("vault.db");
    let env_body = both_keys_env();
    fs::write(project_a.path().join(".env"), &env_body).expect("env a");
    fs::write(project_b.path().join(".env"), &env_body).expect("env b");

    for (proj, vault) in [(project_a.path(), &vault_a), (project_b.path(), &vault_b)] {
        hermetic_with_home(&home_path)
            .current_dir(proj)
            .arg("--vault-path")
            .arg(vault)
            .arg("init")
            .assert()
            .success();
    }

    let first = hermetic_with_home(&home_path)
        .current_dir(project_a.path())
        .env("AI_BRAINS_PROJECT_ID", SHELL_PROJECT)
        .env("AI_BRAINS_SESSION_ID", SHELL_SESSION)
        .arg("--vault-path")
        .arg(&vault_a)
        .arg("preflight")
        .arg("--summary")
        .output()
        .expect("preflight a");
    assert_eq!(
        warn_count(&String::from_utf8_lossy(&first.stderr)),
        1,
        "first location must warn; stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = hermetic_with_home(&home_path)
        .current_dir(project_b.path())
        .env("AI_BRAINS_PROJECT_ID", SHELL_PROJECT)
        .env("AI_BRAINS_SESSION_ID", SHELL_SESSION)
        .arg("--vault-path")
        .arg(&vault_b)
        .arg("preflight")
        .arg("--summary")
        .output()
        .expect("preflight b");
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert_eq!(
        warn_count(&stderr),
        1,
        "AC3/F4: different .env parent must re-warn (not share empty-cwd marker); got: {stderr}"
    );
    // Two distinct fingerprints under the shared home.
    assert_eq!(
        list_markers(&home_path).len(),
        2,
        "two locations → two markers; got {:?}",
        list_markers(&home_path)
    );
}
