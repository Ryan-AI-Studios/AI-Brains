//! T294 — already-initialized `context` upserts `.env` dest into the vault (AC3–AC8, AC15).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_store::event_store::EventStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
/// Hashed-shape leftover dest (tail zeros; not a v4). F15 / AC3.
const DEST_PROJECT: &str = "a1a61a6f-578a-683a-0000-000000000000";
const DEST_SESSION: &str = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

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

fn event_count(vault_path: &Path) -> usize {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(
        vault_path.to_str().expect("utf8 vault"),
        &key,
    )
    .expect("open vault");
    let store = ai_brains_store::event_store::SqliteEventStore::new(conn);
    store.read_all_events().expect("read events").len()
}

fn session_projection_exists(vault_path: &Path, session_id: &str) -> bool {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(
        vault_path.to_str().expect("utf8 vault"),
        &key,
    )
    .expect("open vault");
    let guard = conn.lock().expect("lock vault");
    let mut stmt = guard
        .prepare("SELECT 1 FROM session_projection WHERE session_id = ?1")
        .expect("prepare session_projection");
    stmt.exists([session_id]).expect("session exists query")
}

fn rich_dest_env() -> String {
    format!(
        "# comment\n\nAI_BRAINS_PROJECT_ID={DEST_PROJECT}\nAI_BRAINS_SESSION_ID={DEST_SESSION}\nAI_BRAINS_KEY={ZERO_KEY}\n"
    )
}

struct UpsertFixture {
    _dir: tempfile::TempDir,
    vault: PathBuf,
    from_id: String,
    rebind_root: PathBuf,
    dest_dir: PathBuf,
    dest_env: PathBuf,
}

fn fixture_upsert() -> UpsertFixture {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let work_a = dir.path().join("proj-a");
    let from_id = register_project(&vault, &work_a);

    let rebind_root = dir.path().join("rebind-root");
    fs::create_dir_all(&rebind_root).expect("rebind root");
    register_path(
        &vault,
        &from_id,
        rebind_root.to_str().expect("utf8 rebind root"),
    );

    let dest_dir = dir.path().join("dest-proj");
    fs::create_dir_all(&dest_dir).expect("dest dir");
    let dest_env = dest_dir.join(".env");
    fs::write(&dest_env, rich_dest_env()).expect("write rich .env");

    UpsertFixture {
        _dir: dir,
        vault,
        from_id,
        rebind_root,
        dest_dir,
        dest_env,
    }
}

fn context_cmd(fx: &UpsertFixture) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.current_dir(&fx.dest_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("context");
    cmd
}

fn run_context(fx: &UpsertFixture) -> Output {
    context_cmd(fx).output().expect("context must spawn")
}

fn project_list_json(vault: &Path) -> serde_json::Value {
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list");
    assert!(
        out.status.success(),
        "project list failed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("project list json")
}

fn list_contains_project(v: &serde_json::Value, id: &str) -> bool {
    v.get("projects")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .any(|row| row.get("project_id").and_then(|x| x.as_str()) == Some(id))
}

#[test]
fn context__already_initialized_foreign_hashed_id__upserts_env_bytes_unchanged() {
    let fx = fixture_upsert();
    let before_bytes = fs::read(&fx.dest_env).expect("read .env before");

    let out = run_context(&fx);
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC3: exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Context is already initialized for project"),
        "AC3: already initialized; got: {stdout}"
    );
    let vault_lines = stdout
        .lines()
        .filter(|l| *l == "Vault: project and session present.")
        .count();
    assert_eq!(vault_lines, 1, "AC3: Vault: once; got: {stdout}");
    assert!(
        !stdout.contains("Local .env updated successfully."),
        "AC3: must not rewrite .env chrome; got: {stdout}"
    );
    let after_bytes = fs::read(&fx.dest_env).expect("read .env after");
    assert_eq!(before_bytes, after_bytes, "AC3: .env bytes must be equal");

    let list = project_list_json(&fx.vault);
    assert!(
        list_contains_project(&list, DEST_PROJECT),
        "AC3: project list must contain dest; got: {list}"
    );
    assert!(
        session_projection_exists(&fx.vault, DEST_SESSION),
        "AC3/P2-2: session_projection must contain dest session"
    );
}

#[test]
fn context__already_initialized_foreign_hashed_id__rebind_print_only_dest_exists() {
    let fx = fixture_upsert();
    let upsert = run_context(&fx);
    assert_eq!(
        upsert.status.code(),
        Some(0),
        "precondition upsert; stderr={}",
        String::from_utf8_lossy(&upsert.stderr)
    );

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.rebind_root.to_str().expect("utf8"))
        .arg("--to")
        .arg(DEST_PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("rebind print-only");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC4: print-only dest exists exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.to_lowercase().contains("not found in vault"),
        "AC4: must not say dest-missing; got: {stdout}"
    );
    assert!(
        stdout.contains(&fx.from_id) || stdout.contains(DEST_PROJECT),
        "AC4: names parties; got: {stdout}"
    );
}

#[test]
fn context__already_initialized_second_run__event_count_unchanged() {
    let fx = fixture_upsert();
    let first = run_context(&fx);
    assert_eq!(
        first.status.code(),
        Some(0),
        "first upsert; stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );
    let before = event_count(&fx.vault);
    let before_env = fs::read(&fx.dest_env).expect("env before second");

    let second = run_context(&fx);
    assert_eq!(
        second.status.code(),
        Some(0),
        "AC15: second exit 0; stderr={}",
        String::from_utf8_lossy(&second.stderr)
    );
    let stdout = String::from_utf8_lossy(&second.stdout);
    let vault_lines = stdout
        .lines()
        .filter(|l| *l == "Vault: project and session present.")
        .count();
    assert_eq!(vault_lines, 1, "AC15: Vault: once; got: {stdout}");
    assert_eq!(
        event_count(&fx.vault),
        before,
        "AC15: event_count unchanged"
    );
    let after_env = fs::read(&fx.dest_env).expect("env after second");
    assert_eq!(before_env, after_env, "AC15: .env bytes equal");
}

#[test]
fn context__session_only_env__skips_ensure_no_vault_line() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let cwd = dir.path().join("session-only");
    fs::create_dir_all(&cwd).expect("cwd");
    let env_path = cwd.join(".env");
    let body = format!("AI_BRAINS_SESSION_ID={DEST_SESSION}\n");
    fs::write(&env_path, &body).expect("write .env");
    let before = fs::read(&env_path).expect("before");

    let out = hermetic()
        .current_dir(&cwd)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .output()
        .expect("context");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC6: exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("Vault:"),
        "AC6: no Vault: line; got: {stdout}"
    );
    assert_eq!(
        fs::read(&env_path).expect("after"),
        before,
        "AC6: .env equal"
    );

    let list = project_list_json(&vault);
    // Hashed/discovered id for this cwd must not appear (no mint on skip).
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&cwd.to_string_lossy().to_lowercase(), &mut hasher);
    let hash = std::hash::Hasher::finish(&hasher);
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&hash.to_be_bytes());
    let discovered = uuid::Uuid::from_bytes(bytes).to_string();
    assert!(
        !list_contains_project(&list, &discovered),
        "AC6: must not mint discovered id {discovered}; list={list}"
    );
}

#[test]
fn context__invalid_session_uuid__exit_1_env_unchanged() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let before_events = event_count(&vault);
    let cwd = dir.path().join("bad-session");
    fs::create_dir_all(&cwd).expect("cwd");
    let env_path = cwd.join(".env");
    let body = format!("AI_BRAINS_PROJECT_ID={DEST_PROJECT}\nAI_BRAINS_SESSION_ID=not-a-uuid\n");
    fs::write(&env_path, &body).expect("write .env");
    let before = fs::read(&env_path).expect("before");

    let out = hermetic()
        .current_dir(&cwd)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("context")
        .output()
        .expect("context");
    assert_eq!(
        out.status.code(),
        Some(1),
        "AC7: exit 1; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("AI_BRAINS_SESSION_ID"),
        "AC7: stderr names SESSION_ID; got: {stderr}"
    );
    assert_eq!(
        fs::read(&env_path).expect("after"),
        before,
        "AC7: .env equal"
    );
    assert_eq!(
        event_count(&vault),
        before_events,
        "AC7: no events appended"
    );
}

#[test]
fn context__show_on_foreign_dest__does_not_upsert() {
    let fx = fixture_upsert();
    let before = fs::read(&fx.dest_env).expect("before");

    let out = hermetic()
        .current_dir(&fx.dest_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("context")
        .arg("--show")
        .output()
        .expect("context --show");
    assert!(
        out.status.success(),
        "AC8: --show exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read(&fx.dest_env).expect("after"),
        before,
        "AC8: .env equal"
    );
    let list = project_list_json(&fx.vault);
    assert!(
        !list_contains_project(&list, DEST_PROJECT),
        "AC8: --show must not upsert dest; list={list}"
    );
}

#[test]
fn context__help__names_already_initialized_vault_truth() {
    let out = hermetic()
        .arg("context")
        .arg("--help")
        .output()
        .expect("context --help");
    assert!(out.status.success(), "help exit 0");
    let help = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(
        (help.contains("already") || help.contains("initialized"))
            && (help.contains("vault")
                || help.contains("does not rewrite")
                || help.contains(".env")),
        "AC9: help dual-truth; got: {help}"
    );
    assert!(
        !(help.contains("set-alias") && help.contains("7d97a456") && help.contains("ai-brains")),
        "AC14: must not recommend set-alias leftover dump; got: {help}"
    );
}
