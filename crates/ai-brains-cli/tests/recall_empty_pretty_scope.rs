//! T207 — Recall empty pretty + scope honesty hermetic locks (AC1–AC4, AC9, AC12).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

/// Parse the last JSON object line from stdout (tracing may precede it).
fn parse_last_json_object(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("recall JSON parse failed: {e}; line={line}; full_stdout={stdout}");
    })
}

/// Register a project via `context` in `work_dir` (writes `.env` there).
fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
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

fn set_alias(vault: &Path, project_id: &str, alias: &str) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("set-alias")
        .arg(project_id)
        .arg(alias)
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// B1 / AC1 — empty pretty non-TTY always prints No results
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__pretty_non_tty__stdout_contains_no_results() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--global")
        .output()
        .expect("recall empty pretty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "empty pretty must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No results"),
        "empty pretty must print No results (F3, not TTY-only); got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// B2 / AC2 — empty pretty prints Scope line
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__pretty__prints_scope_global() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--global")
        .output()
        .expect("recall empty global pretty");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope: global"),
        "empty global pretty must print Scope: global; got: {stdout}"
    );
    assert!(
        stdout.contains("No results"),
        "must also print No results; got: {stdout}"
    );
}

#[test]
fn recall_empty__pretty__prints_scope_project_with_alias() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let work = dir.path().join("proj");
    init_vault(&vault);
    let project_id = register_project(&vault, &work);
    set_alias(&vault, &project_id, "t207-alias");

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .env("AI_BRAINS_PROJECT_ID", &project_id)
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall empty project pretty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:") && stdout.contains("project="),
        "empty project pretty must print Scope project=; got: {stdout}"
    );
    assert!(
        stdout.contains("t207-alias") && stdout.contains(&project_id),
        "Scope should include alias and full uuid; got: {stdout}"
    );
    assert!(
        stdout.contains("No results"),
        "must print No results; got: {stdout}"
    );
    assert!(
        stdout.contains("this project") || stdout.contains("Scoped to this project"),
        "hint should have project-scoped clause; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// B3 / AC3 — empty JSON still has hint + effective_session_id, exit 0
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__json__hint_and_effective_session_id_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .arg("--global")
        .output()
        .expect("recall empty json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "empty json must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let hint = v.get("hint").and_then(|h| h.as_str()).unwrap_or("");
    assert!(
        !hint.is_empty() && hint.contains("No results"),
        "empty json must have non-null hint with No results; full={v}"
    );
    // JSON field serializes as effective_session_id (rename) or session_id.
    let sid = v
        .get("effective_session_id")
        .or_else(|| v.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("");
    assert!(
        !sid.is_empty(),
        "empty json must include effective_session_id; full={v}"
    );
}

// ---------------------------------------------------------------------------
// B5 / AC9 — generated session omitted on empty pretty
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__pretty_no_session_env__omits_generated_session_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic_vault strips AI_BRAINS_SESSION_ID; no --session flag → generated only.
    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--global")
        .output()
        .expect("recall empty pretty no session");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.lines().any(|l| l.starts_with("Session:")),
        "F5: generated session must be omitted on empty pretty; got: {stdout}"
    );
    assert!(
        stdout.contains("Scope:"),
        "must still print Scope; got: {stdout}"
    );
    assert!(
        stdout.contains("No results"),
        "must still print No results; got: {stdout}"
    );
}

#[test]
fn recall_empty__pretty_with_user_session__still_prints_session() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let user_session = "11111111-1111-1111-1111-111111111111";

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--session")
        .arg(user_session)
        // --session with --global clears session in main; use project scope instead
        .env("AI_BRAINS_PROJECT_ID", common::DEFAULT_PROJECT)
        .output()
        .expect("recall empty with user session");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("Session: {user_session}")),
        "user-supplied session must print on empty pretty; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// B6 / AC12 — --quiet empty pretty still Scope + No results
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__pretty_quiet__still_scope_and_no_results() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzznonexistentquery999")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--quiet")
        .arg("--global")
        .output()
        .expect("recall empty pretty quiet");

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "F11: --quiet must not suppress Scope; got: {stdout}"
    );
    assert!(
        stdout.contains("No results"),
        "F11: --quiet must not suppress empty hint; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// B7 / AC1,AC4,AC10,AC11 — non-empty pretty Scope before Session/hits (T228)
// ---------------------------------------------------------------------------

#[test]
fn recall_nonempty__pretty__prints_scope_before_hits() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let unique = "T228unique-memory-seed-xyzzy-content";
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(unique)
        .assert()
        .success();

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg(unique)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall non-empty pretty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "AC1/AC10: non-empty pretty must print Scope; got: {stdout}"
    );
    assert!(
        stdout.contains(unique) || stdout.contains("T228unique"),
        "non-empty pretty must show hit content; got: {stdout}"
    );
    assert!(
        !stdout.contains("No results"),
        "AC4: non-empty must not print empty hint; got: {stdout}"
    );
    // AC11: first chrome line Scope, second Session; no blank between them.
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "need Scope + Session chrome at minimum; got: {stdout}"
    );
    assert!(
        lines[0].starts_with("Scope:"),
        "AC11: first non-empty line must be Scope:; got: {stdout}"
    );
    assert!(
        lines[1].starts_with("Session:"),
        "AC11: second non-empty line must be Session:; got: {stdout}"
    );
    // F26: no blank line between Scope and Session (consecutive in raw stdout).
    let raw_lines: Vec<&str> = stdout.lines().collect();
    if let Some(scope_idx) = raw_lines.iter().position(|l| l.starts_with("Scope:")) {
        assert!(
            scope_idx + 1 < raw_lines.len(),
            "Session should follow Scope; got: {stdout}"
        );
        assert!(
            raw_lines[scope_idx + 1].starts_with("Session:"),
            "F26: Session must immediately follow Scope (no blank); got: {stdout}"
        );
    }
}

// ---------------------------------------------------------------------------
// T228 AC2 — non-empty pretty global → Scope: global
// ---------------------------------------------------------------------------

#[test]
fn recall_nonempty__pretty_global__scope_global() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let unique = "T228global-memory-seed-xyzzy-content";
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(unique)
        .assert()
        .success();

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg(unique)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--global")
        .output()
        .expect("recall non-empty pretty global");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope: global"),
        "AC2: non-empty pretty --global must print Scope: global; got: {stdout}"
    );
    assert!(
        stdout.contains(unique) || stdout.contains("T228global"),
        "must still show hit content; got: {stdout}"
    );
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines
            .first()
            .is_some_and(|l| l.starts_with("Scope: global")),
        "Scope: global must appear before hits; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// T228 AC6 — --quiet non-empty pretty keeps Scope
// ---------------------------------------------------------------------------

#[test]
fn recall_nonempty__pretty_quiet__keeps_scope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let unique = "T228quiet-memory-seed-xyzzy-content";
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(unique)
        .assert()
        .success();

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg(unique)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--quiet")
        .output()
        .expect("recall non-empty pretty quiet");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "AC6: --quiet must not suppress Scope on non-empty pretty; got: {stdout}"
    );
    assert!(
        stdout.contains(unique) || stdout.contains("T228quiet"),
        "must still show hit content; got: {stdout}"
    );
    assert!(
        !stdout.contains("No results"),
        "non-empty must not print empty hint; got: {stdout}"
    );
}
