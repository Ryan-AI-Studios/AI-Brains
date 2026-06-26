#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::{Seek, Write};
use std::path::PathBuf;

use tempfile::tempdir;

const PROJECT_ALPHA: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const PROJECT_BETA: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const SESSION_1: &str = "11111111-1111-1111-1111-111111111111";
const SESSION_2: &str = "22222222-2222-2222-2222-222222222222";
const SESSION_3: &str = "33333333-3333-3333-3333-333333333333";
const SESSION_4: &str = "44444444-4444-4444-4444-444444444444";

fn init_vault(vault_path: &std::path::Path) {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn ingest_turn(vault_path: &std::path::Path, project_id: &str, session_id: &str, content: &str) {
    let turn_json = format!(
        r#"{{
            "session_id": "{}",
            "project_id": "{}",
            "harness_id": "00000000-0000-0000-0000-000000000000",
            "turn_id": "{}",
            "privacy": "LocalOnly",
            "role": "user",
            "content": "{}"
        }}"#,
        session_id,
        project_id,
        uuid::Uuid::new_v4(),
        content.replace('"', "\\\"")
    );
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();
}

fn recall_json(
    vault_path: &std::path::Path,
    query: &str,
    extra_args: &[&str],
) -> Vec<serde_json::Value> {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("recall")
        .arg(query)
        .arg("--format")
        .arg("json")
        .args(extra_args)
        .output()
        .expect("recall must run");
    assert!(
        output.status.success(),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("recall must emit valid JSON; got: {stdout} ({e})"));
    parsed["results"]
        .as_array()
        .unwrap_or_else(|| panic!("results must be an array; got: {parsed}"))
        .clone()
}

/// T124: `sync query --no-bridge --format pretty` skips the ChangeGuard
/// Ledger Search section but still runs local recall.
#[test]
#[allow(non_snake_case)]
fn sync_query__no_bridge__skips_changeguard_section() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T124 no-bridge seed content.",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("no-bridge seed")
        .arg("--no-bridge")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("sync query must run");

    assert!(
        output.status.success(),
        "sync query --no-bridge must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("AI-Brains Recall"),
        "local recall section must be present; got: {stdout}"
    );
    assert!(
        !stdout.contains("ChangeGuard Ledger Search"),
        "ChangeGuard section must be skipped with --no-bridge; got: {stdout}"
    );
}

/// T124: `sync query --no-bridge --format ndjson` emits only local records.
#[test]
#[allow(non_snake_case)]
fn sync_query__no_bridge_ndjson__only_local_records() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T124 no-bridge ndjson seed content.",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("no-bridge ndjson seed")
        .arg("--no-bridge")
        .arg("--format")
        .arg("ndjson")
        .output()
        .expect("sync query must run");

    assert!(
        output.status.success(),
        "sync query --no-bridge --format ndjson must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert!(
        !records.is_empty(),
        "local records must be emitted; got: {stdout}"
    );
    for r in &records {
        assert_eq!(
            r["record_kind"].as_str(),
            Some("insight"),
            "all records must be local insight records; got: {r}"
        );
        assert_eq!(
            r["project_id"].as_str(),
            Some(PROJECT_ALPHA),
            "local record project_id must match; got: {r}"
        );
    }
}

/// T127: `sync query --format ndjson` local records carry their source session_id.
#[test]
#[allow(non_snake_case)]
fn sync_query_ndjson__local_record_has_session_id() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T127 ndjson session id seed content.",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("ndjson session id seed")
        .arg("--format")
        .arg("ndjson")
        .arg("--no-bridge")
        .output()
        .expect("sync query must run");

    assert!(
        output.status.success(),
        "sync query --format ndjson must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert!(
        !records.is_empty(),
        "local records must be emitted; got: {stdout}"
    );
    let found = records.iter().any(|r| {
        r["session_id"]
            .as_str()
            .map(|s| s == SESSION_1)
            .unwrap_or(false)
    });
    assert!(
        found,
        "at least one record must have session_id={SESSION_1}; got: {stdout}"
    );
}

/// T125: `recall --session <prefix>` resolves to the full session ID and scopes results.
#[test]
#[allow(non_snake_case)]
fn recall__session_prefix__resolves_to_full_id() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T125 prefix resolution content one",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "T125 prefix resolution content two",
    );

    let results = recall_json(
        &vault_path,
        "prefix resolution content",
        &[
            "--project-id",
            PROJECT_ALPHA,
            "--session-prefix",
            "1111",
            "--no-bridge",
        ],
    );
    assert_eq!(
        results.len(),
        1,
        "prefix recall must resolve to exactly one session; got: {results:?}"
    );
    assert_eq!(
        results[0]["session_id"].as_str(),
        Some(SESSION_1),
        "result must be from session 1; got: {:?}",
        results[0]["session_id"]
    );
}

/// T125: `recall --session <too-short>` is rejected with a clear error.
#[test]
#[allow(non_snake_case)]
fn recall__session_prefix_too_short__rejected() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T125 too short content",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("recall")
        .arg("too short")
        .arg("--session-prefix")
        .arg("ab")
        .output()
        .expect("recall must run");

    assert!(
        !output.status.success(),
        "too-short session prefix must exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("too short") && stderr.contains("4 characters"),
        "error must mention too short and 4 characters; got: {stderr}"
    );
}

/// T125: `recall --session <prefix>` with no matching session errors clearly.
#[test]
#[allow(non_snake_case)]
fn recall__session_prefix_no_match__errors() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T125 no match content",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("recall")
        .arg("no match")
        .arg("--session-prefix")
        .arg("zzzz")
        .output()
        .expect("recall must run");

    assert!(
        !output.status.success(),
        "no-match session prefix must exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No session matching")
            && stderr.contains("zzzz")
            && stderr.contains("project list"),
        "error must mention no session matching and project list; got: {stderr}"
    );
}

/// T125: `recall --session-last` scopes recall to the most recent session.
#[test]
#[allow(non_snake_case)]
fn recall__session_last__scopes_to_most_recent() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T125 session last content",
    );
    // Ensure the second session has a later updated_at by serializing ingest.
    std::thread::sleep(std::time::Duration::from_millis(50));
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "T125 session last content recent",
    );

    let results = recall_json(
        &vault_path,
        "session last content",
        &[
            "--project-id",
            PROJECT_ALPHA,
            "--session-last",
            "--no-bridge",
        ],
    );
    assert_eq!(
        results.len(),
        1,
        "--session-last must scope to the most recent session; got: {results:?}"
    );
    assert_eq!(
        results[0]["session_id"].as_str(),
        Some(SESSION_2),
        "result must be from the most recent session; got: {:?}",
        results[0]["session_id"]
    );
}

/// T130: JSON recall results include a session_id per result matching the source session.
#[test]
#[allow(non_snake_case)]
fn recall_json__each_result_has_session_id() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T130 session field content alpha",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "T130 session field content beta",
    );

    let results = recall_json(
        &vault_path,
        "session field content",
        &["--project-id", PROJECT_ALPHA, "--no-bridge"],
    );
    assert_eq!(
        results.len(),
        2,
        "expected results from both sessions; got: {results:?}"
    );
    let sessions: std::collections::HashSet<String> = results
        .iter()
        .filter_map(|r| r["session_id"].as_str().map(String::from))
        .collect();
    assert!(
        sessions.contains(SESSION_1),
        "results must include session 1; got: {sessions:?}"
    );
    assert!(
        sessions.contains(SESSION_2),
        "results must include session 2; got: {sessions:?}"
    );
}

/// T130: Pretty recall output shows the 8-char session prefix on each result line.
#[test]
#[allow(non_snake_case)]
fn recall_pretty__shows_session_prefix() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T130 pretty session prefix content",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("recall")
        .arg("pretty session prefix")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("recall must run");

    assert!(
        output.status.success(),
        "recall --format pretty must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let prefix = &SESSION_1[..8];
    assert!(
        stdout.contains(&format!("session={}", prefix)),
        "pretty output must contain session=8-char prefix; got: {stdout}"
    );
}

/// T130: The top-level JSON response field is effective_session_id, not session_id.
#[test]
#[allow(non_snake_case)]
fn recall_json__top_level_effective_session_id() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "T130 effective session id content",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ALPHA)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("recall")
        .arg("effective session id")
        .arg("--format")
        .arg("json")
        .arg("--session")
        .arg(SESSION_1)
        .output()
        .expect("recall must run");

    assert!(
        output.status.success(),
        "recall --format json --session must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("recall must emit valid JSON; got: {stdout} ({e})"));
    assert!(
        parsed.get("effective_session_id").is_some(),
        "top-level field must be effective_session_id; got: {parsed}"
    );
    assert!(
        parsed.get("session_id").is_none(),
        "top-level session_id must be renamed to effective_session_id; got: {parsed}"
    );
}

/// T112: Default recall (no --global, no --session) should return memories
/// from all sessions within the current project.
#[test]
#[allow(non_snake_case)]
fn recall__default_scope__searches_all_project_memories() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "alpha-one unique-token",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "alpha-two unique-token",
    );

    let results = recall_json(
        &vault_path,
        "unique-token",
        &["--project-id", PROJECT_ALPHA],
    );
    assert_eq!(
        results.len(),
        2,
        "default recall should find memories from both sessions in the project; got: {results:?}"
    );
}

/// T112: `recall --global` should search across all projects and all sessions.
#[test]
#[allow(non_snake_case)]
fn recall__global_flag__searches_all_projects_and_sessions() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "global-token alpha session one",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "global-token alpha session two",
    );
    ingest_turn(
        &vault_path,
        PROJECT_BETA,
        SESSION_3,
        "global-token beta session three",
    );
    ingest_turn(
        &vault_path,
        PROJECT_BETA,
        SESSION_4,
        "global-token beta session four",
    );

    let results = recall_json(&vault_path, "global-token", &["--global"]);
    assert_eq!(
        results.len(),
        4,
        "--global recall should find memories across all projects and sessions; got: {results:?}"
    );
}

/// T112: `recall --session <ID>` should scope results to the specified session only.
#[test]
#[allow(non_snake_case)]
fn recall__session_flag__scopes_to_specified_session() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "session-scoped content only in session one",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "session-scoped content only in session two",
    );

    let results = recall_json(
        &vault_path,
        "session-scoped",
        &["--project-id", PROJECT_ALPHA, "--session", SESSION_1],
    );
    assert_eq!(
        results.len(),
        1,
        "--session recall should return exactly one hit for the specified session; got: {results:?}"
    );
    let content = results[0]["content"].as_str().unwrap_or("");
    assert!(
        content.contains("session one"),
        "the single result should come from session one; got: {content}"
    );
}

/// T112: When AI_BRAINS_SESSION_ID is set, recall does NOT auto-scope by session.
#[test]
#[allow(non_snake_case)]
fn recall__env_session_id__does_not_auto_scope() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    init_vault(&vault_path);
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_1,
        "env-token in session one",
    );
    ingest_turn(
        &vault_path,
        PROJECT_ALPHA,
        SESSION_2,
        "env-token in session two",
    );

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_SESSION_ID", SESSION_2)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("env-token")
        .arg("--format")
        .arg("json")
        .arg("--project-id")
        .arg(PROJECT_ALPHA)
        .output()
        .expect("recall must run");

    assert!(
        output.status.success(),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("recall must emit valid JSON; got: {stdout} ({e})"));
    let results = parsed["results"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        results.len(),
        2,
        "env AI_BRAINS_SESSION_ID must not auto-scope recall; got: {results:?}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_create__dry_run__does_not_create_file() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--dry-run")
        .output()
        .expect("backup create --dry-run must run");

    assert!(
        output.status.success(),
        "backup create --dry-run must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let backup_dir = dir.path().join("backups");
    assert!(
        !backup_dir.exists() || backup_dir.read_dir().unwrap().next().is_none(),
        "dry-run must not create any backup file"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_create__dry_run__prints_preview() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--dry-run")
        .output()
        .expect("backup create --dry-run must run");

    assert!(
        output.status.success(),
        "backup create --dry-run must exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[dry-run] Would create backup at"),
        "stdout must contain dry-run preview; got: {stdout}"
    );
    assert!(
        stdout.contains("source vault") && stdout.contains(&*vault_path.to_string_lossy()),
        "stdout must mention source vault path; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_list__pre_t109_backup__no_warn_on_stderr() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Create a real backup, then strip its metadata table to simulate pre-T109.
    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");
    assert!(backup_output.status.success(), "backup create failed");
    let stdout = String::from_utf8_lossy(&backup_output.stdout);
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed");

    let key = "x'0000000000000000000000000000000000000000000000000000000000000000'";
    let conn = rusqlite::Connection::open(backup_path).unwrap();
    ai_brains_store::pragmas::apply_key_pragmas(
        &conn,
        &ai_brains_crypto::SqlCipherKey::from_raw(key.to_string()),
    )
    .unwrap();
    conn.execute_batch("DROP TABLE IF EXISTS _aibrains_backup_meta;")
        .unwrap();
    drop(conn);

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "warn")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");

    assert!(output.status.success(), "backup list must exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("WARN") || !stderr.contains("backup metadata"),
        "pre-T109 backup must not emit metadata WARN; got: {stderr}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_list__corrupted_new_backup__stays_warn() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let bogus = backup_dir.join("vault-2026-01-01T00-00-00.db.bak");
    fs::write(&bogus, b"not a valid sqlite database").unwrap();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "warn")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");

    assert!(output.status.success(), "backup list must exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined
            .to_lowercase()
            .contains("could not read backup metadata")
            || combined.to_lowercase().contains("file is not a database"),
        "corrupted backup must emit warning; got: {combined}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_list__source_vault_column_shows_path_end() {
    // Use a long vault path so the source vault path exceeds 40 chars.
    let dir = tempdir().unwrap();
    let subdir = dir
        .path()
        .join("very-long-directory-name-that-makes-path-exceed-forty");
    fs::create_dir_all(&subdir).unwrap();
    let vault_path = subdir.join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");

    assert!(output.status.success(), "backup list must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("vault.db"),
        "Source Vault column should show path end containing vault.db; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_list__parses_nanosecond_timestamp() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .assert()
        .success();

    // Copy the real backup to a nanosecond+timezone name.
    let entries: Vec<_> = fs::read_dir(&backup_dir).unwrap().collect();
    let src = entries.first().unwrap().as_ref().unwrap().path();
    let dst = backup_dir.join("vault-2026-04-28T16-23-52.639348300+00-00.db.bak");
    fs::copy(&src, &dst).unwrap();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");

    assert!(output.status.success(), "backup list must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2026-04-28 16:23:52"),
        "nanosecond timestamp must parse; got: {stdout}"
    );
    assert!(
        !stdout.contains("(unparseable)"),
        "nanosecond timestamp must not show unparseable; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_list__parses_nanosecond_no_timezone() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&backup_dir).unwrap().collect();
    let src = entries.first().unwrap().as_ref().unwrap().path();
    let dst = backup_dir.join("vault-2026-04-28T16-23-52.639348300.db.bak");
    fs::copy(&src, &dst).unwrap();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");

    assert!(output.status.success(), "backup list must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2026-04-28 16:23:52"),
        "nanosecond timestamp must parse; got: {stdout}"
    );
    assert!(
        !stdout.contains("(unparseable)"),
        "nanosecond timestamp must not show unparseable; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_create__default_keep_10__prunes_old_backups() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Seed 12 old backups by creating one real backup and copying it to
    // distinct past timestamps. Use `--output-dir` to ensure predictable paths.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--no-prune")
        .assert()
        .success();
    let real_backup = fs::read_dir(&backup_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let real_bytes = fs::read(&real_backup).unwrap();

    for day in 1..=12 {
        let name = format!("vault-2024-01-{:02}T00-00-00.db.bak", day);
        fs::write(backup_dir.join(&name), &real_bytes).unwrap();
    }

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .output()
        .expect("backup create must run");
    assert!(
        output.status.success(),
        "backup create must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let count = fs::read_dir(&backup_dir).unwrap().count();
    assert_eq!(
        count, 10,
        "default keep=10 should prune oldest backups; found {count}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_create__keep_0__rejected() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--keep")
        .arg("0")
        .output()
        .expect("backup create --keep 0 must run");

    assert!(!output.status.success(), "backup create --keep 0 must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") && stderr.contains("--no-prune"),
        "error must mention invalid and --no-prune; got: {stderr}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_create__no_prune__keeps_all() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--no-prune")
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_secs(2));

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--no-prune")
        .assert()
        .success();

    let count = fs::read_dir(&backup_dir).unwrap().count();
    assert_eq!(count, 2, "--no-prune must keep all backups; found {count}");
}

#[test]
#[allow(non_snake_case)]
fn backup_create__first_run_emits_migration_warning() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Ensure no sentinel by pointing HOME to a fresh temp dir.
    let home_dir = tempdir().unwrap();
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("USERPROFILE", home_dir.path())
        .env("RUST_LOG", "warn")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .output()
        .expect("backup create must run");

    assert!(output.status.success(), "backup create must exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined
            .to_lowercase()
            .contains("default retention changed"),
        "first run must emit retention warning; got: stdout={stdout} stderr={stderr}"
    );

    let output2 = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("USERPROFILE", home_dir.path())
        .env("RUST_LOG", "warn")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .output()
        .expect("second backup create must run");

    assert!(output2.status.success(), "second backup create must exit 0");
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    let combined2 = format!("{stdout2}{stderr2}");
    assert!(
        !combined2
            .to_lowercase()
            .contains("default retention changed"),
        "second run must not emit retention warning; got: {combined2}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_verify__valid_backup__reports_ok() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");
    assert!(output.status.success(), "backup create failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed");

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .arg(backup_path)
        .output()
        .expect("backup verify must run");

    assert!(
        verify_output.status.success(),
        "backup verify must exit 0; stderr={}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(
        stdout.contains("OK"),
        "verify must report OK for valid backup; got: {stdout}"
    );
    assert!(
        !stdout.contains("FAIL"),
        "verify must not report FAIL for valid backup; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_verify__corrupted_backup__reports_fail() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");
    assert!(output.status.success(), "backup create failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let backup_path = PathBuf::from(
        stdout
            .lines()
            .find_map(|l| l.split("Backup created and verified: ").nth(1))
            .expect("backup path must be printed"),
    );

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&backup_path)
        .unwrap();
    file.seek(std::io::SeekFrom::Start(100)).unwrap();
    file.write_all(b"CORRUPTION").unwrap();
    file.sync_all().unwrap();
    drop(file);

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .arg(&backup_path)
        .output()
        .expect("backup verify must run");

    assert!(
        !verify_output.status.success(),
        "backup verify on corrupted backup must exit non-zero"
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(
        stdout.contains("FAIL"),
        "verify must report FAIL for corrupted backup; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn backup_verify_all__mixed__reports_per_file() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Create two valid backups, waiting between them so timestamps differ.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _output2 = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("second backup create must run");

    // Corrupt one of the backups.
    let backup_dir = dir.path().join("backups");
    let mut paths: Vec<PathBuf> = fs::read_dir(&backup_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();

    // Corrupt the older backup so the newer one stays valid. After sorting
    // by name the first entry is the older backup (lower timestamp seconds).
    let older = &paths[0];
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(older)
        .unwrap();
    file.seek(std::io::SeekFrom::Start(100)).unwrap();
    file.write_all(b"CORRUPTION").unwrap();
    file.sync_all().unwrap();
    drop(file);

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .output()
        .expect("backup verify must run");

    assert!(
        !verify_output.status.success(),
        "verify with one corrupted backup must exit non-zero"
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    let output_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.starts_with("vault-") && (l.contains(": OK") || l.contains(": FAIL")))
        .collect();
    let ok_count = output_lines
        .iter()
        .filter(|l| l.trim().ends_with(": OK"))
        .count();
    let fail_count = output_lines
        .iter()
        .filter(|l| l.trim().contains(": FAIL"))
        .count();
    assert_eq!(ok_count, 1, "expected 1 OK; got: {stdout}");
    assert_eq!(fail_count, 1, "expected 1 FAIL; got: {stdout}");
}

#[test]
#[allow(non_snake_case)]
fn backup_verify__json_format() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .arg("--format")
        .arg("json")
        .output()
        .expect("backup verify --format json must run");

    assert!(
        verify_output.status.success(),
        "backup verify json must exit 0; stderr={}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    let json_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| {
            panic!("verify json output must contain a JSON object line; got: {stdout}")
        });
    let parsed: serde_json::Value = serde_json::from_str(json_line)
        .unwrap_or_else(|e| panic!("verify json must be valid JSON; got: {json_line} ({e})"));
    let results = parsed["results"].as_array().cloned().unwrap_or_default();
    assert!(!results.is_empty(), "json results must be non-empty");
    let first = &results[0];
    assert!(first["path"].is_string(), "result must have path");
    assert!(first["status"].is_string(), "result must have status");
    assert!(first["check"].is_string(), "result must have check");
    assert!(first["tables"].is_array(), "result must have tables array");
    assert!(
        first["size_bytes"].is_number(),
        "result must have size_bytes"
    );
}

/// T138: backup verify FAIL includes the error reason in text output.
#[test]
#[allow(non_snake_case)]
fn backup_verify__corrupted_backup__shows_error_reason() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let bogus = backup_dir.join("vault-2026-01-01T00-00-00.db.bak");
    fs::write(&bogus, b"not a valid sqlite database").unwrap();

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .arg(&bogus)
        .output()
        .expect("backup verify must run");

    assert!(
        !verify_output.status.success(),
        "verify on corrupted backup must exit non-zero"
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(
        stdout.contains("FAIL —"),
        "verify output must contain 'FAIL —' with reason; got: {stdout}"
    );
}

/// T138: backup verify --format json includes an error field for FAIL results.
#[test]
#[allow(non_snake_case)]
fn backup_verify__json_includes_error_field() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let bogus = backup_dir.join("vault-2026-01-01T00-00-00.db.bak");
    fs::write(&bogus, b"not a valid sqlite database").unwrap();

    let verify_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("verify")
        .arg(&bogus)
        .arg("--format")
        .arg("json")
        .output()
        .expect("backup verify json must run");

    assert!(
        !verify_output.status.success(),
        "verify json on corrupted backup must exit non-zero"
    );
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    let json_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| {
            panic!("verify json output must contain a JSON object line; got: {stdout}")
        });
    let parsed: serde_json::Value = serde_json::from_str(json_line)
        .unwrap_or_else(|e| panic!("verify json must be valid JSON; got: {json_line} ({e})"));
    let results = parsed["results"].as_array().cloned().unwrap_or_default();
    assert!(!results.is_empty(), "json results must be non-empty");
    let first = &results[0];
    assert_eq!(first["status"].as_str(), Some("fail"));
    assert!(
        first["error"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false),
        "fail result must have a non-empty error field; got: {first}"
    );
}

#[test]
fn test_cli_init_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    let mut cmd = Command::cargo_bin("ai-brains").unwrap();
    cmd.arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));

    assert!(vault_path.exists());
}

#[test]
fn test_cli_ingest_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // Init first
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Ingest
    let mut ingest_cmd = Command::cargo_bin("ai-brains").unwrap();
    let turn_json = r#"{
        "type": "turn",
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "The password for the server is 'antigravity'."
    }"#;

    ingest_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"processed\":true"));
}

#[test]
fn test_cli_context_idempotency() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let env_path = dir.path().join(".env");

    // Init vault first (required for context)
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // First run - initializes context
    let mut cmd1 = Command::cargo_bin("ai-brains").unwrap();
    cmd1.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Context initialized for project"));

    assert!(env_path.exists());
    let content1 = std::fs::read_to_string(&env_path).unwrap();

    // Second run - should be idempotent and succeed without error
    let mut cmd2 = Command::cargo_bin("ai-brains").unwrap();
    cmd2.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Context is already initialized for project",
        ));

    let content2 = std::fs::read_to_string(&env_path).unwrap();
    assert_eq!(
        content1, content2,
        "Context file should not have changed on second run"
    );

    // Third run with --new-session - should replace session and change file contents
    let mut cmd3 = Command::cargo_bin("ai-brains").unwrap();
    cmd3.current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .arg("--new-session")
        .assert()
        .success()
        .stdout(predicate::str::contains("Replacing existing session"));

    let content3 = std::fs::read_to_string(&env_path).unwrap();
    assert_ne!(
        content1, content3,
        "Context file should have changed after --new-session"
    );
}

/// T73: `init` on a vault that already contains ingested data must refuse
/// with a structured error unless `--force` is provided.
#[test]
fn test_init_refuses_populated_vault() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // First init + ingest to populate the vault with one project + one session.
    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Populate the vault so init has data to refuse on."
    }"#;

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // Second init without --force must fail with a clear error.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "init on populated vault must exit non-zero; got: {:?}",
        output
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already") || stderr.contains("Refusing"),
        "stderr should explain the refusal; got: {stderr}"
    );
}

/// T73 companion: with `--force`, init must succeed even on a populated vault.
#[test]
fn test_init_force_overwrites() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "55555555-5555-5555-5555-555555555555",
        "project_id": "66666666-6666-6666-6666-666666666666",
        "harness_id": "77777777-7777-7777-7777-777777777777",
        "turn_id": "88888888-8888-8888-8888-888888888888",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Populate the vault so --force is exercised."
    }"#;

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));
}

/// T74: After init + ingest + recall, the live graph projector should report
/// at least one node and one edge in `ai-brains graph update`. This catches
/// silent graph-projector regressions where ingest/recall succeed but no
/// graph state is written.
///
/// Gated on the `graph` feature because the `graph` subcommand is only
/// compiled in with that feature. Run with:
///   cargo nextest run -p ai-brains-cli --features graph test_graph_health_smoke
#[cfg(feature = "graph")]
#[test]
fn test_graph_health_smoke() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // 1) Init
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2) Ingest one turn so we have a project + session + turn.
    let turn_json = r#"{
        "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "project_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "harness_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "turn_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Anchoring memory for the graph health smoke test."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // 3) Pin a memory so the graph has a `MemoryPinned` event to project.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("pin")
        .arg("T74 graph health smoke seed")
        .assert()
        .success();

    // 4) Recall — T67 wiring emits MemoryPinned events for hits, which the
    //    live graph projector (T69) should immediately apply.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("graph health smoke")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // 5) `graph update` should report live, non-empty graph state.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("graph")
        .arg("update")
        .output()
        .expect("graph update must run");

    assert!(
        output.status.success(),
        "graph update failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("graph update must emit valid JSON, got: {stdout} ({e})"));

    let nodes = parsed["nodes"].as_i64().unwrap_or(-1);
    let edges = parsed["edges"].as_i64().unwrap_or(-1);
    let status = parsed["status"].as_str().unwrap_or("");

    assert!(
        nodes >= 1,
        "graph must contain at least 1 node; got: {parsed}"
    );
    assert!(
        edges >= 1,
        "graph must contain at least 1 edge; got: {parsed}"
    );
    assert_eq!(status, "live", "graph status must be 'live'; got: {parsed}");
}

/// T76: `backup restore --dry-run` must verify integrity and report the plan,
/// but must not overwrite the destination vault and must not prompt.
#[test]
fn test_backup_restore_dry_run() {
    let dir = tempdir().unwrap();
    let source_vault = dir.path().join("source.db");
    let dest_vault = dir.path().join("dest.db");

    // Create source vault with a project so restore has real data to verify.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("init")
        .assert()
        .success();
    let turn_json = r#"{
        "session_id": "99999999-9999-9999-9999-999999999999",
        "project_id": "88888888-8888-8888-8888-888888888888",
        "harness_id": "77777777-7777-7777-7777-777777777777",
        "turn_id": "66666666-6666-6666-6666-666666666666",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Seed the source vault so backup has data."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // Create dest vault and seed it with a different project so we can detect
    // any accidental overwrite.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("init")
        .assert()
        .success();
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("pin")
        .arg("Original content on dest that must survive dry-run")
        .assert()
        .success();

    // Snapshot the dest vault size; dry-run must leave it untouched.
    let dest_size_before = std::fs::metadata(&dest_vault).unwrap().len();

    // Generate a backup of source.
    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("backup")
        .output()
        .expect("backup must run");
    assert!(backup_output.status.success());
    let stdout = String::from_utf8_lossy(&backup_output.stdout);
    // Output is "Backup created and verified: <path>"
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed")
        .trim();
    let backup_path = std::path::PathBuf::from(backup_path);
    assert!(backup_path.exists(), "backup file must exist");

    // Dry-run restore.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));

    // The dest vault must be byte-for-byte untouched.
    let dest_size_after = std::fs::metadata(&dest_vault).unwrap().len();
    assert_eq!(
        dest_size_before, dest_size_after,
        "dry-run must not modify the destination vault"
    );
}

/// T76: `backup restore --force` must skip the interactive confirm prompt.
#[test]
fn test_backup_restore_force_skips_prompt() {
    let dir = tempdir().unwrap();
    let source_vault = dir.path().join("source.db");
    let dest_vault = dir.path().join("dest.db");

    // Build source vault with a project + a backup file.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("init")
        .assert()
        .success();
    let turn_json = r#"{
        "session_id": "44444444-4444-4444-4444-444444444444",
        "project_id": "33333333-3333-3333-3333-333333333333",
        "harness_id": "22222222-2222-2222-2222-222222222222",
        "turn_id": "11111111-1111-1111-1111-111111111111",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "Source content for force-restore test."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&source_vault)
        .arg("backup")
        .output()
        .expect("backup must run");
    assert!(backup_output.status.success());
    let stdout = String::from_utf8_lossy(&backup_output.stdout);
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed")
        .trim();
    let backup_path = std::path::PathBuf::from(backup_path);

    // Init dest so the file exists (required for SQLite restore).
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("init")
        .assert()
        .success();

    // --force must succeed with no stdin (interactive prompt would hang).
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&dest_vault)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault restored from"));
}

/// T116: `backup list` must show the backup filename in the first column,
/// not a truncated full path, and the header must read "Filename".
#[test]
#[allow(non_snake_case)]
fn backup_list__shows_filename_not_full_path() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let backup_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");
    assert!(backup_output.status.success(), "backup create failed");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list must run");
    assert!(
        output.status.success(),
        "backup list must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Filename"),
        "backup list header must contain 'Filename'; got: {stdout}"
    );

    let data_line = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("vault-") && l.contains(".db.bak"))
        .expect("backup list must contain a data line starting with the backup filename");
    let filename = data_line
        .split_whitespace()
        .next()
        .expect("first column must be the filename");
    assert!(
        filename.starts_with("vault-") && filename.ends_with(".db.bak"),
        "first column must be a backup filename; got: {filename}"
    );
    assert!(
        !filename.contains(std::path::MAIN_SEPARATOR),
        "first column must not contain a path separator (truncated full path); got: {filename}"
    );
}

/// T83: `agy-hook --schema` must print the JSON Schema for the payload
/// shape and exit 0. The audit showed that the schema was undocumented
/// and users hit "missing field `transcriptPath`" without a hint.
#[test]
fn test_agy_hook_schema_flag() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("agy-hook")
        .arg("--schema")
        .output()
        .expect("agy-hook --schema must run");

    assert!(
        output.status.success(),
        "agy-hook --schema must exit 0; got: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("agy-hook --schema must emit valid JSON, got: {stdout} ({e})"));
    assert_eq!(parsed["title"].as_str(), Some("AI-Brains agy-hook payload"));
    let required: Vec<&str> = parsed["required"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for field in ["transcriptPath", "sessionId", "projectHash"] {
        assert!(
            required.contains(&field),
            "agy-hook schema must require {field}; got: {required:?}"
        );
    }
}

/// T83: `sync pull --schema` must print the JSON Schema for the NDJSON
/// record shape and exit 0.
#[test]
fn test_sync_pull_schema_flag() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("sync")
        .arg("pull")
        .arg("--schema")
        .output()
        .expect("sync pull --schema must run");

    assert!(
        output.status.success(),
        "sync pull --schema must exit 0; got: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("sync pull --schema must emit valid JSON, got: {stdout} ({e})"));
    assert_eq!(
        parsed["title"].as_str(),
        Some("AI-Brains sync pull NDJSON record")
    );
    assert!(
        parsed["required"]
            .as_array()
            .map(|a| a.iter().any(|v| v.as_str() == Some("bridge_version")))
            .unwrap_or(false),
        "sync pull schema must require bridge_version"
    );
}

/// T82: `context --new-project` with an existing `.env` must rotate the
/// project_id to a fresh UUID. The audit showed that the flag was parsed
/// but ignored when `.env` already existed.
#[test]
fn test_context_new_project_rotates_id() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // First context run: writes an initial .env.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("Context initialized"));

    let env1 = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    let project1 = env1
        .lines()
        .find(|l| l.starts_with("AI_BRAINS_PROJECT_ID"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.to_string())
        .expect("first project_id must be in .env");

    // Second run with --new-project must rotate the project_id.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .arg("--new-project")
        .assert()
        .success();

    let env2 = std::fs::read_to_string(dir.path().join(".env")).unwrap();
    let project2 = env2
        .lines()
        .find(|l| l.starts_with("AI_BRAINS_PROJECT_ID"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.to_string())
        .expect("second project_id must be in .env");

    assert_ne!(
        project1, project2,
        "context --new-project must rotate the project_id; both runs produced {project1}"
    );
}

/// T81: `recall --quiet` from a non-git directory must NOT print the
/// "ChangeGuard bridge query failed, falling back to local FTS5 only:"
/// warning on stderr. The audit showed this warning is emitted on every
/// `recall` call when the cwd is not a git repository.
#[test]
fn test_recall_quiet_silences_bridge_warning() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // Run from a directory that is guaranteed to NOT be a git repository.
    assert!(!dir.path().join(".git").exists());

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // The vault must have at least one memory for recall to hit FTS5.
    let turn_json = r#"{
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "T81 quiet-recall-bridge-warning seed content."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("--quiet")
        .arg("quiet bridge warning")
        .output()
        .expect("recall must run");

    // The CLI must accept --quiet and succeed; if clap rejected the flag,
    // the bridge call would not have run, silently passing this test.
    assert!(
        output.status.success(),
        "recall --quiet must exit 0; got: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("bridge query failed"),
        "recall --quiet must not print bridge-failed warning; got: {stderr}"
    );
    assert!(
        !stderr.contains("falling back"),
        "recall --quiet must not print falling-back message; got: {stderr}"
    );
}

/// T80: when no `.env` exists in cwd, `main()` clears
/// `AI_BRAINS_PROJECT_ID` and `AI_BRAINS_SESSION_ID` even if the caller
/// has set them in their shell. The `--no-project-context` escape hatch
/// preserves those env vars. This test runs the CLI in a tempdir with the
/// env vars exported, and asserts that `pin` succeeds when
/// `--no-project-context` is set.
#[test]
fn test_no_project_context_preserves_env_vars() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    // .env must NOT exist in cwd for the env-clear branch to fire.
    assert!(!dir.path().join(".env").exists());

    Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Export env vars from the test process. Command::cargo_bin inherits
    // by default; assert_cmd::cargo::Command uses std::process::Command
    // which inherits the test process env unless told otherwise.
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env(
            "AI_BRAINS_PROJECT_ID",
            "22222222-2222-2222-2222-222222222222",
        )
        .env(
            "AI_BRAINS_SESSION_ID",
            "11111111-1111-1111-1111-111111111111",
        )
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("pin")
        .arg("T80 env-var preservation test")
        .output()
        .expect("pin must run");

    assert!(
        output.status.success(),
        "pin with --no-project-context must succeed; got: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("successfully pinned"),
        "stdout should confirm the pin; got: {stdout}"
    );
}

#[test]
#[allow(non_snake_case)]
fn preflight__local_env_project_context_overrides_inherited_shell_ids() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let local_project_id = "99999999-9999-9999-9999-999999999999";
    let local_session_id = "88888888-8888-8888-8888-888888888888";
    let inherited_project_id = "77777777-7777-7777-7777-777777777777";
    let inherited_session_id = "66666666-6666-6666-6666-666666666666";

    fs::write(
        dir.path().join(".env"),
        format!(
            "AI_BRAINS_PROJECT_ID={}\nAI_BRAINS_SESSION_ID={}\n",
            local_project_id, local_session_id
        ),
    )
    .unwrap();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", inherited_project_id)
        .env("AI_BRAINS_SESSION_ID", inherited_session_id)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--summary")
        .output()
        .expect("preflight must run");

    assert!(
        output.status.success(),
        "preflight must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("Project: {}", local_project_id)),
        "preflight should scope to local .env project; got: {stdout}"
    );
    assert!(
        !stdout.contains(inherited_project_id),
        "preflight must not silently scope to inherited project; got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("local .env AI_BRAINS_PROJECT_ID overrides inherited shell value"),
        "preflight should warn about inherited project override; got: {stderr}"
    );
    assert!(
        stderr.contains("local .env AI_BRAINS_SESSION_ID overrides inherited shell value"),
        "preflight should warn about inherited session override; got: {stderr}"
    );
}

/// T79: `nightly --skip-import` flag must be present in the help text and
/// accepted by clap. The full pipeline (MADR ingestion, symbol bridge,
/// summaries) cannot run in a smoke test without a live model server, so
/// the test only verifies the flag is plumbed through.
#[test]
fn test_nightly_skip_import_flag_accepted() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // The flag should appear in the help text so users discover it.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("nightly")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--skip-import"));
}

/// T77: `forget --memory-id=<unknown>` must fail with a clear "not found" error
/// instead of silently appending a MemoryForgotten event that matches zero
/// projection rows.
#[test]
fn test_forget_unknown_memory_id_errors() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    let mut init_cmd = Command::cargo_bin("ai-brains").unwrap();
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let unknown_id = "00000000-0000-0000-0000-000000000000";
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("forget")
        .arg(format!("--memory-id={}", unknown_id))
        .arg("--force")
        .output()
        .expect("forget must run");

    assert!(
        !output.status.success(),
        "forget on an unknown --memory-id must exit non-zero; got: {:?}",
        output
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("not in"),
        "stderr should explain the unknown memory id; got: {stderr}"
    );
}

/// T84: `daemon update` subcommand must appear in `daemon --help`.
/// A full stop/install/restart cannot run in CI (requires live cargo workspace),
/// so we only verify the command surface is wired up.
#[test]
fn test_daemon_update_command_exists() {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("daemon")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("update"));
}

/// T113: Shell env vars must win over project `.env` files.
#[test]
#[allow(non_snake_case)]
fn env_var_precedence__shell_overrides_env_file() {
    let dir = tempdir().unwrap();
    let env_path = dir.path().join(".env");
    std::fs::write(&env_path, "AI_BRAINS_MODEL_URL=http://127.0.0.1:9999\n")
        .expect("write project .env");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env("AI_BRAINS_MODEL_URL", "http://127.0.0.1:1")
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("127.0.0.1:1"),
        "daemon status must use the shell env var URL (port :1); got: {stdout}"
    );
    assert!(
        stdout.contains("Closed") || stdout.contains("closed"),
        "daemon status must report the shell URL as closed/unreachable; got: {stdout}"
    );
    assert!(
        !stdout.contains("9999"),
        "daemon status must not use the .env URL (port 9999); got: {stdout}"
    );
}

/// T113: Project `.env` must take precedence over global `~/.ai-brains/.env`.
/// We redirect USERPROFILE to a tempdir, place a global .env there with port
/// 7777, and a project .env with port 8888 in the cwd. The project value must
/// win because it loads first and the global loader is non-override.
#[test]
#[allow(non_snake_case)]
fn env_var_precedence__project_env_overrides_global_env() {
    let project_dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();

    let global_ai_brains = home_dir.path().join(".ai-brains");
    std::fs::create_dir_all(&global_ai_brains).expect("create global .ai-brains dir");
    std::fs::write(
        global_ai_brains.join(".env"),
        "AI_BRAINS_MODEL_URL=http://127.0.0.1:7777\n",
    )
    .expect("write global .env");

    std::fs::write(
        project_dir.path().join(".env"),
        "AI_BRAINS_MODEL_URL=http://127.0.0.1:8888\n",
    )
    .expect("write project .env");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(project_dir.path())
        .env("USERPROFILE", home_dir.path())
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("8888"),
        "daemon status must use the project .env URL (port 8888); got: {stdout}"
    );
    assert!(
        !stdout.contains("7777"),
        "daemon status must not use the global .env URL (port 7777); got: {stdout}"
    );
}

/// T85: `daemon status` must probe the port extracted from `AI_BRAINS_MODEL_URL`
/// rather than the old hardcoded port 8081. We set a distinctive port (9099)
/// that is almost certainly unoccupied and assert it appears in the output.
#[test]
fn test_daemon_status_respects_model_url_env_var() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_MODEL_URL", "http://127.0.0.1:9099")
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("9099"),
        "daemon status must probe configured port 9099 from AI_BRAINS_MODEL_URL; got: {stdout}"
    );
    // Old hardcoded port must NOT appear
    assert!(
        !stdout.contains("Port 8081"),
        "daemon status must not probe hardcoded 8081; got: {stdout}"
    );
}

/// T118: `backup create` progress must go through tracing, not raw stderr.
/// Before migration, "Creating vault backup..." appeared as a raw eprintln!
/// line. After migration, the raw text must NOT appear in stderr.
#[test]
#[allow(non_snake_case)]
fn backup_create__progress_goes_to_tracing_not_stderr() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .output()
        .expect("backup create must run");

    assert!(
        output.status.success(),
        "backup create must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Creating vault backup..."),
        "backup progress must not be raw stderr text; got: {stderr}"
    );
}

/// T118: The scoped tracing filter must keep external crates quiet.
/// `daemon status` exercises TCP connection attempts but no HTTP bodies,
/// so it should not produce `INFO reqwest`/`INFO hyper`/`INFO tokio` lines.
/// Only `ai_brains` / `ai_brains_cli` crate messages should appear at info level.
#[test]
#[allow(non_snake_case)]
fn tracing_filter__external_deps_stay_quiet() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "")
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for external in ["INFO reqwest", "INFO hyper", "INFO tokio", "INFO rusqlite"] {
        assert!(
            !stderr.contains(external),
            "scoped filter must suppress external INFO logs; found {external}; got: {stderr}"
        );
    }
}

/// T85: When `AI_BRAINS_EMBEDDING_URL` is set, `daemon status` probes that port.
#[test]
fn test_daemon_status_respects_embedding_url_env_var() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_EMBEDDING_URL", "http://127.0.0.1:9199")
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("9199"),
        "daemon status must probe configured embedding port 9199; got: {stdout}"
    );
}

/// T94: `daemon status` retries connection handshakes to handle slow startup of backends.
#[test]
fn test_daemon_status_retries_on_slow_startup() {
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    // Bind to a random port to get a free port number, then drop it
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };

    // Spawn a thread that will bind the listener only after a delay (e.g. 250ms)
    let _handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        if let Ok((_socket, _)) = listener.accept() {
            // connection established
        }
    });

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("AI_BRAINS_MODEL_URL", format!("http://127.0.0.1:{}", port))
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Open"),
        "daemon status must retry and find the delayed port Open; got: {stdout}"
    );
}

/// T86: `recall -` must read the query from stdin and return a valid JSON response.
#[test]
fn test_recall_reads_query_from_stdin() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "project_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "harness_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "turn_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "GPU driver fix for VRAM allocation regression."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    // recall - reads query from piped stdin
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("-")
        .write_stdin("GPU driver fix")
        .output()
        .expect("recall - must run");

    assert!(
        output.status.success(),
        "recall - must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("recall - must emit valid JSON; got: {stdout} ({e})"));
    assert!(
        parsed["results"].is_array(),
        "recall - JSON must have a 'results' array; got: {parsed}"
    );
}

/// T86: `preflight --stdin` must accept JSON options from stdin and succeed.
#[test]
fn test_preflight_reads_options_from_stdin() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let stdin_json = r#"{"max_words": 500}"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--stdin")
        .write_stdin(stdin_json)
        .assert()
        .success();
}

/// T86: `--stdin` flag must appear in `preflight --help`.
#[test]
fn test_preflight_stdin_flag_in_help() {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("preflight")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin"));
}

/// T122: When built without the graph feature, `ai-brains graph update`
/// prints a helpful hint and exits 0.
#[cfg(not(feature = "graph"))]
#[test]
#[allow(non_snake_case)]
fn graph__default_build__prints_hint() {
    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("graph")
        .arg("update")
        .output()
        .expect("graph update must run");

    assert!(
        output.status.success(),
        "graph (stub) must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("requires a --features graph build"),
        "stub must print feature hint; got: {stdout}"
    );

    let help_output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("graph")
        .arg("--help")
        .output()
        .expect("graph --help must run");
    assert!(
        help_output.status.success(),
        "graph --help must exit 0; stderr={}",
        String::from_utf8_lossy(&help_output.stderr)
    );
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(
        help_stdout.contains("requires --features graph"),
        "graph --help must mention feature requirement; got: {help_stdout}"
    );
}

/// T128: `daemon status` shows vault info only when the daemon is running.
/// Starting/stopping a daemon in tests is unreliable, so this test asserts
/// the conditional behaviour: if the daemon happens to be running, the vault
/// info lines must be present; if it is stopped, they must be absent.
#[test]
#[allow(non_snake_case)]
fn daemon_status__vault_info_conditional_on_running() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("daemon")
        .arg("status")
        .output()
        .expect("daemon status must run");

    assert!(
        output.status.success(),
        "daemon status must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let running = stdout.contains("Status: Running");
    let stopped = stdout.contains("Status: Stopped");
    assert!(
        running || stopped,
        "daemon status must report a status; got: {stdout}"
    );

    let has_vault_path = stdout.contains("Vault:");
    let has_vault_size = stdout.contains("Vault size:");
    let has_memories = stdout.contains("Memories:");

    if running {
        assert!(
            has_vault_path && has_vault_size && has_memories,
            "vault info must be shown when daemon is running; got: {stdout}"
        );
    } else {
        assert!(
            !has_vault_path && !has_vault_size && !has_memories,
            "vault info must not be shown when daemon is stopped; got: {stdout}"
        );
    }
}

/// T129: `--log-format compact` emits short tracing lines without the full
/// ISO timestamp.
#[test]
#[allow(non_snake_case)]
fn log_format_compact__short_output() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "info")
        .arg("--log-format")
        .arg("compact")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list with compact format must run");

    assert!(
        output.status.success(),
        "backup list --log-format compact must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let iso_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap();
    for line in stderr.lines() {
        assert!(
            !iso_pattern.is_match(line),
            "compact format must not contain full ISO timestamp; got: {line}"
        );
    }
}

/// T129: `--log-format json` emits valid JSON objects on stderr.
#[test]
#[allow(non_snake_case)]
fn log_format_json__valid_json_lines() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "info")
        .arg("--log-format")
        .arg("json")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list with json format must run");

    assert!(
        output.status.success(),
        "backup list --log-format json must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines().filter(|l| !l.trim().is_empty()) {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
            panic!("json format must emit valid JSON per line; got: {line} ({e})")
        });
        assert!(
            parsed.get("timestamp").is_some(),
            "json line must have timestamp; got: {parsed}"
        );
        assert!(
            parsed.get("level").is_some(),
            "json line must have level; got: {parsed}"
        );
        assert!(
            parsed.get("target").is_some() || parsed.get("fields").is_some(),
            "json line must have target or fields; got: {parsed}"
        );
    }
}

/// T129: `--log-format off` suppresses tracing output on stderr.
#[test]
#[allow(non_snake_case)]
fn log_format_off__no_tracing_output() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "info")
        .arg("--log-format")
        .arg("off")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list with off format must run");

    assert!(
        output.status.success(),
        "backup list --log-format off must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.trim().is_empty(),
        "off format must suppress tracing output; got: {stderr}"
    );
}

/// T136: `--log-format minimal` shows level + message with no ISO timestamp.
#[test]
#[allow(non_snake_case)]
fn log_format_minimal__no_timestamp() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("RUST_LOG", "info")
        .arg("--log-format")
        .arg("minimal")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list with minimal format must run");

    assert!(
        output.status.success(),
        "backup list --log-format minimal must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let iso_pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}").unwrap();
    for line in stderr.lines() {
        assert!(
            !iso_pattern.is_match(line),
            "minimal format must not contain ISO timestamp; got: {line}"
        );
    }
}

/// T115: `sync query` with the daemon not running must still return local
/// recall results. The command should not probe or attempt to start the
/// daemon.
#[test]
#[allow(non_snake_case)]
fn sync_query__daemon_down__returns_local_results() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "11111111-1111-1111-1111-111111111111",
        "project_id": "22222222-2222-2222-2222-222222222222",
        "harness_id": "33333333-3333-3333-3333-333333333333",
        "turn_id": "44444444-4444-4444-4444-444444444444",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "T115 sync query local fallback seed content."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env(
            "AI_BRAINS_PROJECT_ID",
            "22222222-2222-2222-2222-222222222222",
        )
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("T115 sync query local fallback seed content")
        .output()
        .expect("sync query must run");

    assert!(
        output.status.success(),
        "sync query must succeed when daemon is not running; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("T115 sync query local fallback seed content."),
        "sync query must return local recall results; got: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("daemon is not running"),
        "sync query must not error about daemon; got: {stderr}"
    );
}

/// T115: `sync query` with the daemon not running must complete quickly and
/// not emit a daemon-is-down error. No probe, no spawn attempt.
#[test]
#[allow(non_snake_case)]
fn sync_query__daemon_down__no_spawn_attempt() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let turn_json = r#"{
        "session_id": "55555555-5555-5555-5555-555555555555",
        "project_id": "66666666-6666-6666-6666-666666666666",
        "harness_id": "77777777-7777-7777-7777-777777777777",
        "turn_id": "88888888-8888-8888-8888-888888888888",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "T115 no spawn attempt seed content."
    }"#;
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env(
            "AI_BRAINS_PROJECT_ID",
            "66666666-6666-6666-6666-666666666666",
        )
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("T115 no spawn attempt seed content")
        .output()
        .expect("sync query must run");

    assert!(
        output.status.success(),
        "sync query must succeed when daemon is not running; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("daemon is not running"),
        "sync query must not error about daemon; got: {stderr}"
    );
    assert!(
        !stderr.contains("daemon is unreachable"),
        "sync query must not error about daemon reachability; got: {stderr}"
    );
}

/// UX: when a project is registered without an alias, the default name
/// should be readable in `project list`. The old form was
/// `Project <full-uuid>` (32 hex chars); the friendly form is
/// `(no alias) — <short-uuid>` (8-char prefix) plus the full id in the
/// dedicated column. This test seeds a project via `context`, runs
/// `project list`, and asserts the new friendly form is present.
#[test]
fn test_project_list_friendly_default_name() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // context creates a ProjectRegistered event with the default name.
    Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("context")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list must run");

    assert!(
        output.status.success(),
        "project list must exit 0; got: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("(no alias)"),
        "project list should show '(no alias) — <short-uuid>' for unnamed projects; got: {stdout}"
    );
}

/// T119 AC5: `backup --dry-run` (top-level, no subcommand) should trigger the
/// same dry-run preview as `backup create --dry-run`.
#[test]
#[allow(non_snake_case)]
fn backup__top_level_dry_run__triggers_preview() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("USERPROFILE", dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("--dry-run")
        .output()
        .expect("backup --dry-run must run");

    assert!(
        output.status.success(),
        "backup --dry-run must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[dry-run] Would create backup at"),
        "top-level backup --dry-run must print preview; got: {stdout}"
    );
    let backup_dir = dir.path().join("backups");
    if backup_dir.exists() {
        let count = fs::read_dir(&backup_dir).unwrap().count();
        assert_eq!(count, 0, "no backup file should be created in dry-run");
    }
}

/// T119 AC3: `backup create --dry-run --keep N` shows prune preview alongside
/// the backup preview.
#[test]
#[allow(non_snake_case)]
fn backup_create__dry_run_keep__shows_prune_preview() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .env("USERPROFILE", dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--dry-run")
        .arg("--keep")
        .arg("5")
        .output()
        .expect("backup create --dry-run --keep must run");

    assert!(
        output.status.success(),
        "backup create --dry-run --keep must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[dry-run] Would create backup at"),
        "must print backup preview; got: {stdout}"
    );
    assert!(
        stdout.contains("[dry-run] Would prune"),
        "must print prune preview; got: {stdout}"
    );
}

/// T126 AC4: `backup create --keep N` (explicit) overrides the default 10.
#[test]
#[allow(non_snake_case)]
fn backup_create__explicit_keep_N__overrides_default() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    for day in 1..=5 {
        let name = format!("vault-2024-01-{:02}T00-00-00.db.bak", day);
        fs::write(backup_dir.join(&name), b"fake").unwrap();
    }

    Command::cargo_bin("ai-brains")
        .unwrap()
        .env("USERPROFILE", dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("create")
        .arg("--keep")
        .arg("3")
        .assert()
        .success();

    let count = fs::read_dir(&backup_dir).unwrap().count();
    assert_eq!(
        count, 3,
        "explicit --keep 3 must leave exactly 3 backups; found {count}"
    );
}

/// T126 AC7: `backup prune --keep 0` is rejected with the same error message.
#[test]
#[allow(non_snake_case)]
fn backup_prune__keep_0__rejected() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("backup")
        .arg("prune")
        .arg("--keep")
        .arg("0")
        .output()
        .expect("backup prune --keep 0 must run");

    assert!(
        !output.status.success(),
        "backup prune --keep 0 must exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") && stderr.contains("--no-prune"),
        "error must mention 'invalid' and '--no-prune'; got: {stderr}"
    );
}

/// T123 AC5: `backup prune` correctly handles nanosecond-precision backups —
/// they are included in the sort order and eligible for pruning.
#[test]
#[allow(non_snake_case)]
fn backup_prune__handles_nanosecond_timestamps() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    let key = ai_brains_crypto::SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    );

    {
        let conn = rusqlite::Connection::open(&vault_path).unwrap();
        ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP); INSERT INTO schema_migrations (name) VALUES ('0001'); CREATE TABLE test (id INTEGER PRIMARY KEY); INSERT INTO test VALUES (1);",
        )
        .unwrap();
    }

    let service = ai_brains_brain::BackupService::new(vault_path.clone(), key.clone())
        .with_output_dir(backup_dir.clone());

    let files = [
        "vault-2024-01-01T00-00-00.db.bak",
        "vault-2024-01-02T00-00-00.123456789+00-00.db.bak",
        "vault-2024-01-03T00-00-00.db.bak",
        "vault-2024-01-04T00-00-00.987654321+00-00.db.bak",
        "vault-2024-01-05T00-00-00.db.bak",
    ];

    for name in &files {
        let path = backup_dir.join(name);
        let src = rusqlite::Connection::open(&vault_path).unwrap();
        ai_brains_store::pragmas::apply_key_pragmas(&src, &key).unwrap();
        let mut dst = rusqlite::Connection::open(&path).unwrap();
        ai_brains_store::pragmas::apply_key_pragmas(&dst, &key).unwrap();
        let backup = rusqlite::backup::Backup::new(&src, &mut dst).unwrap();
        backup
            .run_to_completion(1000, std::time::Duration::ZERO, None)
            .unwrap();
    }

    let result = service.prune_backups(2, None, false).unwrap();
    assert_eq!(result.pruned_count, 3, "should prune 3 oldest backups");
    assert_eq!(result.remaining_count, 2, "should keep 2 newest");
    assert!(
        backup_dir
            .join("vault-2024-01-04T00-00-00.987654321+00-00.db.bak")
            .exists(),
        "newest nanosecond backup must be kept"
    );
    assert!(
        backup_dir.join("vault-2024-01-05T00-00-00.db.bak").exists(),
        "newest seconds backup must be kept"
    );
    assert!(
        !backup_dir.join("vault-2024-01-01T00-00-00.db.bak").exists(),
        "oldest seconds backup must be pruned"
    );
    assert!(
        !backup_dir
            .join("vault-2024-01-02T00-00-00.123456789+00-00.db.bak")
            .exists(),
        "old nanosecond backup must be pruned"
    );
    assert!(
        !backup_dir.join("vault-2024-01-03T00-00-00.db.bak").exists(),
        "middle seconds backup must be pruned"
    );
}

/// T125 AC3: If multiple sessions match the prefix, `recall` prints an error
/// listing up to 5 matching session IDs and the total count.
#[test]
#[allow(non_snake_case)]
fn recall__session_prefix_ambiguous__errors_with_capped_matches() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");

    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    for i in 0..7 {
        let session_id = format!("1111111{}-1111-1111-1111-111111111111", i);
        let turn_json = format!(
            r#"{{"session_id": "{}", "project_id": "{}", "harness_id": "00000000-0000-0000-0000-000000000000", "turn_id": "{}", "privacy": "LocalOnly", "role": "user", "content": "ambiguous session test {}"}}"#,
            session_id,
            PROJECT_ALPHA,
            uuid::Uuid::new_v4(),
            i
        );
        Command::cargo_bin("ai-brains")
            .unwrap()
            .arg("--vault-path")
            .arg(&vault_path)
            .arg("ingest")
            .write_stdin(turn_json)
            .assert()
            .success();
    }

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("ambiguous session test")
        .arg("--session-prefix")
        .arg("1111")
        .arg("--project-id")
        .arg(PROJECT_ALPHA)
        .output()
        .expect("recall must run");

    assert!(
        !output.status.success(),
        "recall with ambiguous prefix must exit non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.to_lowercase().contains("ambiguous"),
        "error must mention 'ambiguous'; got: {combined}"
    );
    assert!(
        combined.contains("of 7 shown"),
        "error must show total count 'of 7 shown'; got: {combined}"
    );
    assert!(
        combined.contains("5 of 7 shown"),
        "error must cap display at 5 matches; got: {combined}"
    );
}
