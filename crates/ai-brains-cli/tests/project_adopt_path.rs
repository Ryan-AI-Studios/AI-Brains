//! T258 — `project adopt-path` hermetic suite (AC1–AC7, AC12–AC13, AC16).
//!
//! Human-chrome fixtures pass `--format human` (F26). Do not assert
//! `AI_BRAINS_PROJECT_ID=<B>` on `--format auto` (pipe → JSON).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Distinctive dummy key (not `ZERO_SQLCIPHER_KEY`) so AC3 can prove KEY is preserved.
const DUMMY_KEY: &str = "x'deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef'";
const DUMMY_SESSION: &str = "11111111-1111-1111-1111-111111111111";

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

struct AdoptFixture {
    _dir: tempfile::TempDir,
    vault: PathBuf,
    work: PathBuf,
    env_path: PathBuf,
    id_a: String,
    id_b: String,
}

/// Two projects; cwd registered to B; `.env` daily Scope is A + dummy KEY/SESSION.
fn fixture_a_on_b() -> AdoptFixture {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let id_a = register_project(&vault, &proj_a);
    let proj_b = dir.path().join("proj-b");
    let id_b = register_project(&vault, &proj_b);
    assert_ne!(id_a, id_b, "context must mint distinct project IDs");

    let work = dir.path().join("work");
    fs::create_dir_all(&work).expect("work");
    register_path(&vault, &id_b, work.to_str().expect("utf8 work"));

    let env_path = work.join(".env");
    let env_body = format!(
        "AI_BRAINS_PROJECT_ID={id_a}\nAI_BRAINS_KEY={DUMMY_KEY}\nAI_BRAINS_SESSION_ID={DUMMY_SESSION}\n"
    );
    fs::write(&env_path, &env_body).expect("write .env");

    AdoptFixture {
        _dir: dir,
        vault,
        work,
        env_path,
        id_a,
        id_b,
    }
}

fn adopt_cmd(fx: &AdoptFixture) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.current_dir(&fx.work)
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("adopt-path");
    cmd
}

fn env_line(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find(|l| l.starts_with(prefix))
        .map(|l| l.to_string())
}

// ---------------------------------------------------------------------------
// AC1 — print-only names owner, no write (F26 --format human)
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__print_only__names_owner_no_write() {
    let fx = fixture_a_on_b();
    let before = fs::read(&fx.env_path).expect("read .env before");

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("human")
        .output()
        .expect("adopt-path print-only");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&fx.id_b),
        "must name path owner B; got: {stdout}"
    );
    let assign = format!("AI_BRAINS_PROJECT_ID={}", fx.id_b);
    assert!(
        stdout.contains(&assign),
        "must contain exact assignment {assign}; got: {stdout}"
    );
    let after = fs::read(&fx.env_path).expect("read .env after");
    assert_eq!(before, after, "print-only must not change .env bytes");
}

// ---------------------------------------------------------------------------
// AC2 — --write-env without --yes is usage exit 2, no write
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__write_env_without_yes__exit_2_no_write() {
    let fx = fixture_a_on_b();
    let before = fs::read(&fx.env_path).expect("read .env before");

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("human")
        .arg("--write-env")
        .output()
        .expect("adopt-path write without yes");

    assert_eq!(
        out.status.code(),
        Some(2),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--yes"),
        "stderr must mention --yes; got: {stderr}"
    );
    let after = fs::read(&fx.env_path).expect("read .env after");
    assert_eq!(before, after, "--write-env without --yes must not write");
}

// ---------------------------------------------------------------------------
// AC3 — --write-env --yes rewrites only PROJECT_ID
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__write_env_yes__rewrites_only_project_id() {
    let fx = fixture_a_on_b();
    let before = fs::read_to_string(&fx.env_path).expect("read .env before");
    let key_line = env_line(&before, "AI_BRAINS_KEY=").expect("KEY line");
    let session_line = env_line(&before, "AI_BRAINS_SESSION_ID=").expect("SESSION line");

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("human")
        .arg("--write-env")
        .arg("--yes")
        .output()
        .expect("adopt-path write yes");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = fs::read_to_string(&fx.env_path).expect("read .env after");
    let project_line = env_line(&after, "AI_BRAINS_PROJECT_ID=").expect("PROJECT line");
    assert_eq!(
        project_line,
        format!("AI_BRAINS_PROJECT_ID={}", fx.id_b),
        "PROJECT_ID must become B"
    );
    assert_eq!(
        env_line(&after, "AI_BRAINS_KEY=").as_deref(),
        Some(key_line.as_str()),
        "KEY line must be byte-identical"
    );
    assert_eq!(
        env_line(&after, "AI_BRAINS_SESSION_ID=").as_deref(),
        Some(session_line.as_str()),
        "SESSION line must be byte-identical"
    );
    assert!(
        !after.lines().any(|l| l.starts_with("AI_BRAINS_HARNESS_ID")),
        "must not invent HARNESS_ID; got: {after}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — missing .env + --write-env --yes creates PROJECT_ID only
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__missing_env__write_creates_project_id_only() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_b = dir.path().join("proj-b");
    let id_b = register_project(&vault, &proj_b);

    let work = dir.path().join("work-missing");
    fs::create_dir_all(&work).expect("work");
    register_path(&vault, &id_b, work.to_str().expect("utf8"));
    let env_path = work.join(".env");
    assert!(!env_path.exists(), "fixture must start without .env");

    let out = hermetic()
        .current_dir(&work)
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("adopt-path")
        .arg("--format")
        .arg("human")
        .arg("--write-env")
        .arg("--yes")
        .output()
        .expect("adopt-path create .env");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = fs::read_to_string(&env_path).expect("created .env");
    assert_eq!(
        after,
        format!("AI_BRAINS_PROJECT_ID={id_b}\n"),
        "created .env must be PROJECT_ID only"
    );
}

// ---------------------------------------------------------------------------
// AC5 — already bound: exit 0, no rewrite, already_bound chrome
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__already_bound__exit_0_no_rewrite() {
    let fx = fixture_a_on_b();
    let bound = format!(
        "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_KEY={DUMMY_KEY}\nAI_BRAINS_SESSION_ID={DUMMY_SESSION}\n",
        fx.id_b
    );
    fs::write(&fx.env_path, &bound).expect("write already-bound .env");
    let before = fs::read(&fx.env_path).expect("read before");

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("human")
        .output()
        .expect("adopt-path already-bound human");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Already bound to path owner"),
        "already-bound SOOT; got: {stdout}"
    );
    assert!(
        stdout.contains(&fx.id_b),
        "must name path owner; got: {stdout}"
    );
    assert!(
        !stdout.contains("Would set"),
        "must not print Would set when already bound; got: {stdout}"
    );
    assert!(
        !stdout.contains("Re-run with --write-env"),
        "must not print write remediator when already bound; got: {stdout}"
    );
    let after = fs::read(&fx.env_path).expect("read after");
    assert_eq!(before, after, "already-bound must not rewrite");

    let json_out = adopt_cmd(&fx)
        .arg("--format")
        .arg("json")
        .output()
        .expect("adopt-path already-bound json");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_slice(&json_out.stdout).expect("already-bound JSON parses");
    assert_eq!(v["already_bound"], true);
    assert_eq!(v["written"], false);
    assert_eq!(v["from_project_id"], fx.id_b);
    assert_eq!(v["to_project_id"], fx.id_b);
}

// ---------------------------------------------------------------------------
// AC6 — no path owner: exit 1, no write, stderr names register-path
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__no_path_owner__exit_1_no_write() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let id_a = register_project(&vault, &proj_a);

    let work = dir.path().join("work-unregistered");
    fs::create_dir_all(&work).expect("work");
    let env_path = work.join(".env");
    let body = format!("AI_BRAINS_PROJECT_ID={id_a}\nAI_BRAINS_KEY={DUMMY_KEY}\n");
    fs::write(&env_path, &body).expect("write .env");
    let before = fs::read(&env_path).expect("read before");

    let out = hermetic()
        .current_dir(&work)
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("adopt-path")
        .arg("--format")
        .arg("human")
        .output()
        .expect("adopt-path no owner");

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("register-path"),
        "stderr must mention register-path; got: {stderr}"
    );
    let after = fs::read(&env_path).expect("read after");
    assert_eq!(before, after, "no-owner must not write");
}

// ---------------------------------------------------------------------------
// AC7 — whoami mismatch remediations name adopt-path
// ---------------------------------------------------------------------------

#[test]
fn project_whoami__mismatch__remediations_name_adopt_path() {
    let fx = fixture_a_on_b();

    let out = hermetic()
        .current_dir(&fx.work)
        .arg("--vault-path")
        .arg(&fx.vault)
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
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("whoami JSON");
    assert_eq!(v["mismatch"], true);
    let remediations = v["remediations"]
        .as_array()
        .expect("remediations array")
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        remediations.contains("project adopt-path"),
        "remediations must name adopt-path; got: {remediations}"
    );
    let assign = format!("AI_BRAINS_PROJECT_ID={}", fx.id_b);
    assert!(
        remediations.contains(&assign),
        "remediations must contain {assign}; got: {remediations}"
    );
    assert!(
        !remediations.contains("`ai-brains project whoami`"),
        "remediations must not say run whoami; got: {remediations}"
    );
    assert!(
        !remediations.contains("project list"),
        "remediations must not include project list; got: {remediations}"
    );
}

// ---------------------------------------------------------------------------
// AC12 — --yes without --write-env is clap usage exit 2
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__yes_without_write_env__clap_exit_2() {
    let fx = fixture_a_on_b();
    let before = fs::read(&fx.env_path).expect("read before");

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("human")
        .arg("--yes")
        .output()
        .expect("adopt-path --yes only");

    assert_eq!(
        out.status.code(),
        Some(2),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let after = fs::read(&fx.env_path).expect("read after");
    assert_eq!(before, after, "--yes without --write-env must not write");
}

// ---------------------------------------------------------------------------
// AC13 — --format json print-only: frozen keys, written false
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__format_json__print_only_keys() {
    let fx = fixture_a_on_b();

    let out = adopt_cmd(&fx)
        .arg("--format")
        .arg("json")
        .output()
        .expect("adopt-path json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("print-only JSON parses");
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["action"], "adopt-path");
    assert!(v["env_path"].as_str().is_some_and(|p| !p.is_empty()));
    assert_eq!(v["from_project_id"], fx.id_a);
    assert_eq!(v["to_project_id"], fx.id_b);
    assert_eq!(v["written"], false);
    assert_eq!(v["already_bound"], false);
    let keys = v["keys_touched"]
        .as_array()
        .expect("keys_touched array")
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, vec!["AI_BRAINS_PROJECT_ID"]);
}

// ---------------------------------------------------------------------------
// AC16 — --no-project-context uses file PROJECT_ID for already-bound
// ---------------------------------------------------------------------------

#[test]
fn project_adopt_path__no_project_context__file_project_id_already_bound() {
    let fx = fixture_a_on_b();
    let bound = format!(
        "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_KEY={DUMMY_KEY}\nAI_BRAINS_SESSION_ID={DUMMY_SESSION}\n",
        fx.id_b
    );
    fs::write(&fx.env_path, &bound).expect("write file-bound .env");
    let before = fs::read(&fx.env_path).expect("read before");

    let out = hermetic()
        .current_dir(&fx.work)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .env("AI_BRAINS_PROJECT_ID", &fx.id_a)
        .arg("project")
        .arg("adopt-path")
        .arg("--format")
        .arg("human")
        .output()
        .expect("adopt-path no-project-context");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Already bound to path owner"),
        "file-id already-bound; got: {stdout}"
    );
    assert!(
        !stdout.contains("Would set"),
        "must not treat shell A as bind source; got: {stdout}"
    );
    let after = fs::read(&fx.env_path).expect("read after");
    assert_eq!(before, after, "file already-bound must not write");
}
