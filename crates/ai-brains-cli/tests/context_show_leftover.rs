//! T282 — `context --show` leftover shell vs file PROJECT_ID (AC4–AC8).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_crypto::test_support::assert_no_secret_leakage;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::tempdir;

const ENV_ID: &str = "3581317d-601e-44f7-ab84-fde90aa12d3c";
const SHELL_ID: &str = "7d97a456-f2f4-43ea-1f13-211af684ad37";
const DUMMY_KEY: &str = "x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'";
const DUMMY_KEY_BYTES: [u8; 32] = [
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];
const HEADER: &str = "--- Current Context ---";
const NO_ENV_SENTENCE: &str = "No .env file found";
const LEFTOVER_PREFIX: &str = "shell leftover PROJECT_ID: ";
const LEFTOVER_SUFFIX: &str = " (.env overrides)";
const KEY_ASSIGN_PREFIX: &str = "AI_BRAINS_KEY=x'";
const REDACTED_KEY: &str = "AI_BRAINS_KEY=(redacted)";

struct Fixture {
    _dir: tempfile::TempDir,
    cwd: PathBuf,
    vault: PathBuf,
    env_path: PathBuf,
}

fn leftover_line(shell: &str) -> String {
    format!("{LEFTOVER_PREFIX}{shell}{LEFTOVER_SUFFIX}")
}

fn combined_output(output: &Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

fn init_vault(vault: &Path) {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd.arg("--vault-path")
        .arg(vault)
        .arg("--no-project-context")
        .arg("init")
        .assert()
        .success();
}

fn fixture(env_body: Option<&str>) -> Fixture {
    let dir = tempdir().expect("tempdir");
    let cwd = dir.path().join("proj");
    fs::create_dir(&cwd).expect("mkdir proj");
    let env_path = cwd.join(".env");
    if let Some(body) = env_body {
        fs::write(&env_path, body).expect("write .env");
    }
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    Fixture {
        _dir: dir,
        cwd,
        vault,
        env_path,
    }
}

fn show_cmd(fx: &Fixture, shell_id: Option<&str>, extra: &[&str]) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd.current_dir(&fx.cwd)
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("context")
        .arg("--show");
    for arg in extra {
        cmd.arg(arg);
    }
    if let Some(id) = shell_id {
        cmd.env("AI_BRAINS_PROJECT_ID", id);
    }
    cmd
}

fn run_show(fx: &Fixture, shell_id: Option<&str>, extra: &[&str]) -> Output {
    show_cmd(fx, shell_id, extra)
        .output()
        .expect("context --show must spawn")
}

fn assert_success(out: &Output) {
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn context_show__shell_differs_from_file__leftover_once_after_repository() {
    let fx = fixture(Some(&format!("AI_BRAINS_PROJECT_ID={ENV_ID}\n")));
    let out = run_show(&fx, Some(SHELL_ID), &[]);
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains(&format!("AI_BRAINS_PROJECT_ID={ENV_ID}")),
        "must dump file PROJECT_ID; got: {stdout}"
    );
    let leftover = leftover_line(SHELL_ID);
    assert_eq!(
        stdout.matches(leftover.as_str()).count(),
        1,
        "exact leftover once on stdout; got: {stdout}"
    );
    let Some(repo_pos) = stdout.find("Repository:") else {
        panic!("Repository: missing; got: {stdout}");
    };
    let Some(leftover_pos) = stdout.find(leftover.as_str()) else {
        panic!("leftover missing; got: {stdout}");
    };
    assert!(
        leftover_pos > repo_pos,
        "leftover must follow Repository:; got: {stdout}"
    );
    assert!(
        !stderr.contains(leftover.as_str()) || stdout.contains(leftover.as_str()),
        "leftover must not be stderr-only; stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn context_show__shell_equals_file__no_leftover_suffix() {
    let fx = fixture(Some(&format!("AI_BRAINS_PROJECT_ID={ENV_ID}\n")));
    let out = run_show(&fx, Some(ENV_ID), &[]);
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(LEFTOVER_PREFIX),
        "same-id must omit leftover prefix; got: {stdout}"
    );
    assert!(
        !stdout.contains(LEFTOVER_SUFFIX),
        "same-id must omit (.env overrides); got: {stdout}"
    );
}

#[test]
fn context_show__dummy_key_in_file__redacted_no_leak() {
    let body = format!("AI_BRAINS_PROJECT_ID={ENV_ID}\nAI_BRAINS_KEY={DUMMY_KEY}\n");
    let fx = fixture(Some(&body));
    let out = run_show(&fx, Some(SHELL_ID), &[]);
    assert_success(&out);
    let combined = combined_output(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(REDACTED_KEY),
        "KEY file line must print (redacted); got: {stdout}"
    );
    assert!(
        !combined.contains(KEY_ASSIGN_PREFIX),
        "must not print AI_BRAINS_KEY=x'; got: {combined}"
    );
    assert_no_secret_leakage(&combined, &DUMMY_KEY_BYTES);
}

#[test]
fn context_show__and_show_new_project__leave_env_bytes_unchanged() {
    let body = format!(
        "AI_BRAINS_PROJECT_ID={ENV_ID}\nAI_BRAINS_SESSION_ID=11111111-1111-1111-1111-111111111111\n"
    );
    let fx = fixture(Some(&body));
    let before = fs::read(&fx.env_path).expect("snapshot .env");

    let show = run_show(&fx, Some(SHELL_ID), &[]);
    assert_success(&show);
    let after_show = fs::read(&fx.env_path).expect("read after --show");
    assert_eq!(before, after_show, "--show must not write .env");
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(
        stdout.contains(HEADER),
        "--show must print header; got: {stdout}"
    );
    assert!(
        !stdout.contains("initialized") && !stdout.contains("Local .env updated"),
        "--show must not take write path; got: {stdout}"
    );

    let show_new = run_show(&fx, Some(SHELL_ID), &["--new-project"]);
    assert_success(&show_new);
    let after_new = fs::read(&fx.env_path).expect("read after --show --new-project");
    assert_eq!(
        before, after_new,
        "--show --new-project must not write .env"
    );
    let stdout_new = String::from_utf8_lossy(&show_new.stdout);
    assert!(
        stdout_new.contains(HEADER),
        "--show --new-project must print header; got: {stdout_new}"
    );
    assert!(
        !stdout_new.contains("initialized") && !stdout_new.contains("Local .env updated"),
        "--show --new-project must not take write path; got: {stdout_new}"
    );
}

#[test]
fn context_show__no_project_context__leftover_vs_file_not_process_env() {
    let fx = fixture(Some(&format!("AI_BRAINS_PROJECT_ID={ENV_ID}\n")));
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    let out = cmd
        .current_dir(&fx.cwd)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .env("AI_BRAINS_PROJECT_ID", SHELL_ID)
        .arg("context")
        .arg("--show")
        .output()
        .expect("context --show --no-project-context must spawn");
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let leftover = leftover_line(SHELL_ID);
    assert_eq!(
        stdout.matches(leftover.as_str()).count(),
        1,
        "F1/F25: leftover vs file even when --no-project-context keeps process env as shell; got: {stdout}"
    );
    assert!(
        stdout.contains(&format!("AI_BRAINS_PROJECT_ID={ENV_ID}")),
        "dump is the file, not process env; got: {stdout}"
    );
}

#[test]
fn context_show__no_env_file__no_overrides_suffix() {
    let fx = fixture(None);
    let out = run_show(&fx, Some(SHELL_ID), &[]);
    assert_success(&out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(NO_ENV_SENTENCE),
        "missing .env must keep existing sentence; got: {stdout}"
    );
    assert!(
        !stdout.contains(LEFTOVER_SUFFIX),
        "no-env must not print (.env overrides); got: {stdout}"
    );
}
