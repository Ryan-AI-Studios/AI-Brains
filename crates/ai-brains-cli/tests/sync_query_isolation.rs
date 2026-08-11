#![allow(clippy::disallowed_methods)]

mod common;

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

    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();
}

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
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

    let output = common::hermetic_bin()
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
    // T207: empty pretty always prints next-action hint that *quotes the query*.
    // Isolation is: no *hit content* from project A — not "query string never appears".
    assert!(
        stdout.contains("No results"),
        "scoped empty pretty must report no hits; got: {stdout}"
    );
    assert!(
        stdout.contains(PROJECT_B) || stdout.contains("project="),
        "empty pretty should name project B scope; got: {stdout}"
    );
    let hit_leaks_secret = stdout.lines().any(|line| {
        let t = line.trim();
        t.contains("secret_token_a")
            && !t.starts_with("No results")
            && !t.contains("No results for")
    });
    assert!(
        !hit_leaks_secret,
        "pretty query must not return project A's secret as a hit; got: {stdout}"
    );
}

#[test]
fn sync_query_pretty_global_flag_returns_cross_project_results() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "secret_token_a");

    let output = common::hermetic_bin()
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
    // Require a non-empty hit path (content line), not merely the empty-hint query echo.
    let hit_has_secret = stdout.lines().any(|line| {
        let t = line.trim();
        t.contains("secret_token_a")
            && !t.starts_with("No results")
            && !t.contains("No results for")
    });
    assert!(
        hit_has_secret,
        "pretty query --global should return cross-project hit content; got: {stdout}"
    );
    assert!(
        !stdout.contains("No results"),
        "global with hits must not print empty hint; got: {stdout}"
    );
    // T228 AC8: non-empty vault section prints Scope: global after vault header.
    assert!(
        stdout.contains("Scope: global"),
        "AC8: non-empty sync query --global pretty must print Scope: global; got: {stdout}"
    );
    let vault_header = "--- AI-Brains Recall ---";
    if let Some(pos) = stdout.find(vault_header) {
        let after = &stdout[pos + vault_header.len()..];
        assert!(
            after.contains("Scope: global"),
            "AC8: Scope: global must appear after vault header; got: {stdout}"
        );
    } else {
        panic!("expected vault header in sync query pretty output; got: {stdout}");
    }
}

#[test]
fn sync_query_ndjson_remains_scoped_no_regression() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "secret_token_a");

    let output = common::hermetic_bin()
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
