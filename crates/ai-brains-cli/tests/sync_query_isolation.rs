#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use tempfile::tempdir;

const PROJECT_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const PROJECT_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const SESSION_A: &str = "11111111-1111-1111-1111-111111111111";

fn ingest_turn(vault_path: &std::path::Path, project_id: &str, content: &str) {
    let turn_json = format!(
        r#"{{
        "session_id": "{SESSION_A}",
        "project_id": "{project_id}",
        "harness_id": "00000000-0000-0000-0000-000000000000",
        "turn_id": "22222222-2222-2222-2222-222222222222",
        "privacy": "LocalOnly",
        "role": "assistant",
        "content": "{content}"
    }}"#
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

fn init_vault(vault_path: &std::path::Path) {
    Command::cargo_bin("ai-brains")
        .unwrap()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

#[test]
fn sync_query_pretty_default_scoped_to_current_project_no_cross_project_results() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "secret_token_a");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_B)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("secret_token_a")
        .output()
        .unwrap();

    assert!(output.status.success(), "sync query must succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("secret_token_a"),
        "pretty query should be scoped to project B and not return project A's secret; got: {stdout}"
    );
}

#[test]
fn sync_query_pretty_global_flag_returns_cross_project_results() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "secret_token_a");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_B)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("secret_token_a")
        .arg("--global")
        .output()
        .unwrap();

    assert!(output.status.success(), "sync query --global must succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("secret_token_a"),
        "pretty query --global should return cross-project results; got: {stdout}"
    );
}

#[test]
fn sync_query_ndjson_remains_scoped_no_regression() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "secret_token_a");

    let output = Command::cargo_bin("ai-brains")
        .unwrap()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_B)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("secret_token_a")
        .arg("--format")
        .arg("ndjson")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "sync query --format ndjson must succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("secret_token_a"),
        "ndjson query should remain scoped to project B; got: {stdout}"
    );
}
