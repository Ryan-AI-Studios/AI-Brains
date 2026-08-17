//! T259 — `project list-paths` filters + `project rebind-path` hermetics
//! (AC1–AC10, AC16–AC17).
//!
//! Human-chrome fixtures pass `--format human` (F14). Do not assert
//! chrome on `--format auto` (pipe → JSON).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_store::event_store::EventStore;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

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

fn list_paths_json(vault: &Path, extra: &[&str]) -> (i32, serde_json::Value, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("list-paths")
        .args(extra)
        .arg("--format")
        .arg("json");
    let out = cmd.output().expect("list-paths json");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    if code != 0 {
        return (
            code,
            serde_json::Value::Null,
            format!("stdout={stdout}\nstderr={stderr}"),
        );
    }
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("list-paths stdout must be one pretty JSON object: {e}; stdout={stdout}")
    });
    (code, v, stderr)
}

fn assert_t254_path_keys(v: &serde_json::Value) {
    assert_eq!(v["api_version"], "1");
    let paths = v["paths"].as_array().expect("paths array");
    for row in paths {
        let obj = row.as_object().expect("path object");
        for key in ["project_id", "label", "alias", "normalized_path", "exists"] {
            assert!(obj.contains_key(key), "F10 frozen key {key} missing: {row}");
        }
    }
}

fn memory_count_for(vault: &Path, project_id: &str) -> usize {
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list json");
    assert!(
        out.status.success(),
        "project list must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("list json");
    v["projects"]
        .as_array()
        .expect("projects")
        .iter()
        .find(|p| p["project_id"].as_str() == Some(project_id))
        .and_then(|p| p["memory_count"].as_u64())
        .expect("memory_count") as usize
}

fn owner_of(v: &serde_json::Value, normalized: &str) -> Option<String> {
    v["paths"].as_array().and_then(|rows| {
        rows.iter().find_map(|row| {
            if row["normalized_path"].as_str() == Some(normalized) {
                row["project_id"].as_str().map(ToOwned::to_owned)
            } else {
                None
            }
        })
    })
}

struct SharedFixture {
    _dir: tempfile::TempDir,
    vault: PathBuf,
    shared_id: String,
    singleton_id: String,
    path_a: PathBuf,
    path_b: PathBuf,
    path_c: PathBuf,
}

/// One project owns two registered paths; another owns one.
fn fixture_shared_and_singleton() -> SharedFixture {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let shared_id = register_project(&vault, &dir.path().join("proj-shared"));
    let singleton_id = register_project(&vault, &dir.path().join("proj-single"));
    assert_ne!(shared_id, singleton_id);

    let path_a = dir.path().join("aaa-shared");
    let path_b = dir.path().join("mmm-shared");
    let path_c = dir.path().join("zzz-single");
    fs::create_dir_all(&path_a).unwrap();
    fs::create_dir_all(&path_b).unwrap();
    fs::create_dir_all(&path_c).unwrap();
    register_path(&vault, &shared_id, path_a.to_str().expect("utf8"));
    register_path(&vault, &shared_id, path_b.to_str().expect("utf8"));
    register_path(&vault, &singleton_id, path_c.to_str().expect("utf8"));

    SharedFixture {
        _dir: dir,
        vault,
        shared_id,
        singleton_id,
        path_a,
        path_b,
        path_c,
    }
}

struct RebindFixture {
    _dir: tempfile::TempDir,
    vault: PathBuf,
    work_a: PathBuf,
    id_a: String,
    id_b: String,
    path: PathBuf,
}

/// Path registered to A; dest project B exists; one memory pinned on A.
fn fixture_rebind() -> RebindFixture {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let work_a = dir.path().join("proj-a");
    let work_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &work_a);
    let id_b = register_project(&vault, &work_b);
    assert_ne!(id_a, id_b);

    let path = dir.path().join("rebind-root");
    fs::create_dir_all(&path).unwrap();
    register_path(&vault, &id_a, path.to_str().expect("utf8"));
    pin_memory(&vault, &work_a, &id_a, "DECISION: stay on from-project");

    RebindFixture {
        _dir: dir,
        vault,
        work_a,
        id_a,
        id_b,
        path,
    }
}

fn rebind_cmd(fx: &RebindFixture) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .arg("--to")
        .arg(&fx.id_b);
    cmd
}

fn assert_no_leftover_as_ai_brains(text: &str) {
    let lower = text.to_ascii_lowercase();
    let has_set_alias = lower.contains("set-alias");
    let has_leftover = lower.contains("7d97a456");
    let has_ai_brains = text.contains("AI-Brains");
    assert!(
        !(has_set_alias && has_leftover && has_ai_brains),
        "AC15: must not recommend set-alias leftover as AI-Brains; got: {text}"
    );
}

// ---------------------------------------------------------------------------
// AC1 — --shared-only keeps only multi-root owner rows; keys stay T254 F10
// ---------------------------------------------------------------------------

#[test]
fn list_paths__shared_only__multi_root_id_only() {
    let fx = fixture_shared_and_singleton();
    let (code, v, err) = list_paths_json(&fx.vault, &["--shared-only"]);
    assert_eq!(code, 0, "AC1: --shared-only exit 0; {err}");
    assert_t254_path_keys(&v);
    let paths = v["paths"].as_array().expect("paths");
    assert_eq!(paths.len(), 2, "AC1: only shared-owner rows; got {paths:?}");
    for row in paths {
        assert_eq!(
            row["project_id"].as_str(),
            Some(fx.shared_id.as_str()),
            "AC1: singleton must be filtered out; {row}"
        );
    }
    let n0 = paths[0]["normalized_path"].as_str().expect("n0");
    let n1 = paths[1]["normalized_path"].as_str().expect("n1");
    assert!(n0 < n1, "AC1: ASC by normalized_path; {n0:?} then {n1:?}");
}

// ---------------------------------------------------------------------------
// AC2 — --project filter + unknown dest exit 1
// ---------------------------------------------------------------------------

#[test]
fn list_paths__project_filter__only_that_owner() {
    let fx = fixture_shared_and_singleton();
    let (code, v, err) = list_paths_json(&fx.vault, &["--project", &fx.shared_id]);
    assert_eq!(code, 0, "AC2: --project exit 0; {err}");
    let paths = v["paths"].as_array().expect("paths");
    assert_eq!(paths.len(), 2, "AC2: exactly shared-owner rows; {paths:?}");
    for row in paths {
        assert_eq!(row["project_id"].as_str(), Some(fx.shared_id.as_str()));
    }
}

#[test]
fn list_paths__project_unknown__exit_1() {
    let fx = fixture_shared_and_singleton();
    let missing = "00000000-0000-4000-8000-000000000099";
    let (code, _, combined) = list_paths_json(&fx.vault, &["--project", missing]);
    assert_eq!(code, 1, "AC2: unknown dest exit 1; {combined}");
}

// ---------------------------------------------------------------------------
// AC16 — empty filter copy (not T254 empty-register next-step)
// ---------------------------------------------------------------------------

#[test]
fn list_paths__filter_empty__no_match_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let bare = register_project(&vault, &dir.path().join("proj-bare"));

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list-paths")
        .arg("--project")
        .arg(&bare)
        .arg("--format")
        .arg("human")
        .output()
        .expect("list-paths empty filter");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC16: empty filter exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No path aliases match."),
        "AC16: empty-filter copy; got: {stdout}"
    );
    assert!(
        !stdout.contains("No path aliases registered."),
        "AC16: must not use T254 empty-register next-step; got: {stdout}"
    );
    assert!(
        !stdout.contains("register-path"),
        "AC16: must not print T254 next-step; got: {stdout}"
    );

    let (code, v, err) = list_paths_json(&vault, &["--project", &bare]);
    assert_eq!(code, 0, "AC16 json; {err}");
    let paths = v["paths"].as_array().expect("paths");
    assert!(paths.is_empty(), "AC16: paths:[]; got {paths:?}");

    let only_single = dir.path().join("only-single");
    fs::create_dir_all(&only_single).unwrap();
    register_path(&vault, &bare, only_single.to_str().expect("utf8"));
    let (code, v, err) = list_paths_json(&vault, &["--shared-only"]);
    assert_eq!(code, 0, "AC16 shared-only empty; {err}");
    assert!(
        v["paths"].as_array().expect("paths").is_empty(),
        "AC16: no multi-root owner → paths:[]"
    );
}

// ---------------------------------------------------------------------------
// AC17 — --project + --shared-only is intersection
// ---------------------------------------------------------------------------

#[test]
fn list_paths__project_and_shared_only__intersection() {
    let fx = fixture_shared_and_singleton();
    let (code, v, err) = list_paths_json(&fx.vault, &["--project", &fx.shared_id, "--shared-only"]);
    assert_eq!(code, 0, "AC17 shared intersection; {err}");
    let paths = v["paths"].as_array().expect("paths");
    assert_eq!(
        paths.len(),
        2,
        "AC17: shared + shared-only → two rows; {paths:?}"
    );

    let (code, v, err) =
        list_paths_json(&fx.vault, &["--project", &fx.singleton_id, "--shared-only"]);
    assert_eq!(code, 0, "AC17 singleton intersection; {err}");
    let paths = v["paths"].as_array().expect("paths");
    assert!(
        paths.is_empty(),
        "AC17: singleton ∩ shared-only → []; {paths:?}"
    );
    let _ = (&fx.path_a, &fx.path_b, &fx.path_c);
}

// ---------------------------------------------------------------------------
// AC3 — print-only names from/to, no events
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__print_only__names_from_to_no_events() {
    let fx = fixture_rebind();
    let before = event_count(&fx.vault);

    let out = rebind_cmd(&fx)
        .arg("--format")
        .arg("human")
        .output()
        .expect("rebind print-only");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC3: print-only exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(&fx.id_a), "AC3: names from; got: {stdout}");
    assert!(stdout.contains(&fx.id_b), "AC3: names to; got: {stdout}");
    let lower = stdout.to_ascii_lowercase();
    assert!(
        lower.contains("memories stay") || lower.contains("memories_moved"),
        "AC3: honesty that memories stay; got: {stdout}"
    );
    assert_eq!(event_count(&fx.vault), before, "AC3: no events");

    let (_, v, _) = list_paths_json(&fx.vault, &[]);
    let norm = ai_brains_path::normalize_for_location_compare(fx.path.to_str().expect("utf8"));
    assert_eq!(
        owner_of(&v, &norm).as_deref(),
        Some(fx.id_a.as_str()),
        "AC3: owner still A"
    );
}

// ---------------------------------------------------------------------------
// AC4 — --write without --yes is usage exit 2
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__write_without_yes__exit_2_no_events() {
    let fx = fixture_rebind();
    let before = event_count(&fx.vault);

    let out = rebind_cmd(&fx)
        .arg("--format")
        .arg("human")
        .arg("--write")
        .output()
        .expect("rebind --write");
    assert_eq!(
        out.status.code(),
        Some(2),
        "AC4: --write without --yes exit 2; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--yes"),
        "AC4: stderr names --yes; {stderr}"
    );
    assert_eq!(event_count(&fx.vault), before, "AC4: no events");
}

// ---------------------------------------------------------------------------
// AC5 — --write --yes rebinds owner; memories stay; +2 events
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__write_yes__rebinds_owner_memories_stay() {
    let fx = fixture_rebind();
    let before_events = event_count(&fx.vault);
    let before_mem = memory_count_for(&fx.vault, &fx.id_a);
    assert!(before_mem >= 1, "fixture pinned a memory on A");

    let out = rebind_cmd(&fx)
        .arg("--format")
        .arg("human")
        .arg("--write")
        .arg("--yes")
        .output()
        .expect("rebind write");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC5: write exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let (_, v, _) = list_paths_json(&fx.vault, &[]);
    let norm = ai_brains_path::normalize_for_location_compare(fx.path.to_str().expect("utf8"));
    assert_eq!(
        owner_of(&v, &norm).as_deref(),
        Some(fx.id_b.as_str()),
        "AC5: owner is dest B"
    );
    assert_ne!(
        owner_of(&v, &norm).as_deref(),
        Some(fx.id_a.as_str()),
        "AC5: path no longer on A"
    );
    assert_eq!(
        memory_count_for(&fx.vault, &fx.id_a),
        before_mem,
        "AC5: A's memory_count unchanged"
    );
    assert_eq!(
        event_count(&fx.vault),
        before_events + 2,
        "AC5: +2 events (Removed+Added)"
    );
    let _ = &fx.work_a;
}

// ---------------------------------------------------------------------------
// AC6 — already bound
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__already_bound__exit_0_no_events() {
    let fx = fixture_rebind();
    let before = event_count(&fx.vault);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .arg("--to")
        .arg(&fx.id_a)
        .arg("--format")
        .arg("human")
        .output()
        .expect("already-bound human");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC6 human; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Already bound"),
        "AC6: Already bound; got: {stdout}"
    );
    assert!(
        !stdout.contains("Would rebind"),
        "AC6: no Would rebind; got: {stdout}"
    );
    assert!(
        !stdout.contains("Re-run with --write"),
        "AC6: no write nudge; got: {stdout}"
    );
    assert_eq!(event_count(&fx.vault), before);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .arg("--to")
        .arg(&fx.id_a)
        .arg("--format")
        .arg("json")
        .output()
        .expect("already-bound json");
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    assert_eq!(v["already_bound"], true);
    assert_eq!(v["written"], false);
    assert_eq!(v["memories_moved"], false);
    assert_eq!(
        v["from_project_id"].as_str(),
        v["to_project_id"].as_str(),
        "AC6: from == to"
    );
}

// ---------------------------------------------------------------------------
// AC7 — no owner
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__no_owner__exit_1() {
    let fx = fixture_rebind();
    let before = event_count(&fx.vault);
    let missing = fx._dir.path().join("not-registered");
    fs::create_dir_all(&missing).unwrap();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(missing.to_str().expect("utf8"))
        .arg("--to")
        .arg(&fx.id_b)
        .arg("--format")
        .arg("human")
        .output()
        .expect("no-owner");
    assert_eq!(
        out.status.code(),
        Some(1),
        "AC7: no owner exit 1; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("register-path"),
        "AC7: names register-path; {combined}"
    );
    assert_eq!(event_count(&fx.vault), before);
}

// ---------------------------------------------------------------------------
// AC8 — dest missing
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__dest_missing__exit_1() {
    let fx = fixture_rebind();
    let before = event_count(&fx.vault);
    let missing = "00000000-0000-4000-8000-000000000077";

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .arg("--to")
        .arg(missing)
        .arg("--format")
        .arg("human")
        .output()
        .expect("dest missing");
    assert_eq!(
        out.status.code(),
        Some(1),
        "AC8: dest missing exit 1; stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_no_leftover_as_ai_brains(&combined);
    assert_eq!(event_count(&fx.vault), before);
}

// ---------------------------------------------------------------------------
// AC9 — clap usage
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__yes_without_write__clap_exit_2() {
    let fx = fixture_rebind();
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .arg("--to")
        .arg(&fx.id_b)
        .arg("--yes")
        .output()
        .expect("--yes without --write");
    assert_eq!(
        out.status.code(),
        Some(2),
        "AC9: --yes without --write clap 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&fx.vault)
        .arg("project")
        .arg("rebind-path")
        .arg(fx.path.to_str().expect("utf8"))
        .output()
        .expect("missing --to");
    assert_eq!(
        out.status.code(),
        Some(2),
        "AC9: missing --to clap 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// AC10 — print-only JSON keys
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__format_json__print_only_keys() {
    let fx = fixture_rebind();
    let out = rebind_cmd(&fx)
        .arg("--format")
        .arg("json")
        .output()
        .expect("rebind json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC10: json exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON object");
    assert!(v.is_object());
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["written"], false);
    assert_eq!(v["memories_moved"], false);
    assert_eq!(v["already_bound"], false);
    assert_eq!(v["events_appended"], 0);
    assert_eq!(v["from_project_id"].as_str(), Some(fx.id_a.as_str()));
    assert_eq!(v["to_project_id"].as_str(), Some(fx.id_b.as_str()));
    assert!(v["path"].as_str().is_some_and(|p| !p.is_empty()));
    assert!(
        v["from_project_id"].is_string(),
        "AC10: from_project_id is uuid string, never null"
    );
}

// ---------------------------------------------------------------------------
// AC15 — new help / errors never leftover-as-AI-Brains
// ---------------------------------------------------------------------------

#[test]
fn project_rebind_path__help__no_leftover_as_ai_brains() {
    let out = hermetic()
        .arg("--no-project-context")
        .arg("project")
        .arg("rebind-path")
        .arg("--help")
        .output()
        .expect("rebind help");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_no_leftover_as_ai_brains(&combined);
}
