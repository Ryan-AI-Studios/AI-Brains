//! T231 — Unified search UX hermetics (AC5–AC8b, AC13–AC14).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const PROJECT_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const PROJECT_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const SESSION_A: &str = "11111111-1111-1111-1111-111111111111";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn ingest_turn(vault_path: &Path, project_id: &str, content: &str) {
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

/// True if `s` looks like a UUID (8-4-4-4-12 hex).
fn looks_like_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let lens = [8, 4, 4, 4, 12];
    parts
        .iter()
        .zip(lens.iter())
        .all(|(p, &n)| p.len() == n && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Extract the `Scope: …` line body (after the prefix), if any.
fn scope_line(stdout: &str) -> Option<&str> {
    stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Scope:"))
}

// ---------------------------------------------------------------------------
// AC5 — missing project + --no-project-context → project=(none); no random UUID
// ---------------------------------------------------------------------------

#[test]
fn sync_query__missing_project_env__scope_project_none_no_random_uuid() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);

    // F30: hermetic_bin strips ambient PROJECT_ID; --no-project-context + tempdir
    // prevents project-local .env re-inject.
    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("zzzz-no-hit-ac5")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("sync query missing project");

    assert!(
        output.status.success(),
        "AC5: sync query must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Scope: project=(none)"),
        "AC5: missing project must print Scope: project=(none); got: {stdout}"
    );

    // No random UUID on the Scope line (and no invented project id elsewhere as scope).
    if let Some(line) = scope_line(&stdout) {
        for token in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
            if looks_like_uuid(token) {
                panic!("AC5: Scope line must not contain a random UUID; got: {line}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AC6 — invalid AI_BRAINS_PROJECT_ID → project=(none)
// ---------------------------------------------------------------------------

#[test]
fn sync_query__invalid_project_env__scope_project_none() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);

    // F30: explicit .env on Command survives hermetic strip + --no-project-context clear.
    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", "not-a-uuid")
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("zzzz-no-hit-ac6")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("sync query invalid project");

    assert!(
        output.status.success(),
        "AC6: invalid project must still exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Scope: project=(none)"),
        "AC6: invalid project must print Scope: project=(none); got: {stdout}"
    );
    assert!(
        !stdout.contains("not-a-uuid"),
        "AC6: must not echo invalid env as project id; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC7 — valid project with ingest still scoped (regression)
// ---------------------------------------------------------------------------

#[test]
fn sync_query__valid_project_with_ingest__returns_scoped_hit() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    ingest_turn(&vault_path, PROJECT_A, "unique_scoped_token_ac7");

    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_A)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("unique_scoped_token_ac7")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .output()
        .expect("sync query valid project");

    assert!(
        output.status.success(),
        "AC7: must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let hit = stdout.lines().any(|line| {
        let t = line.trim();
        t.contains("unique_scoped_token_ac7")
            && !t.starts_with("No results")
            && !t.contains("No results for")
    });
    assert!(
        hit,
        "AC7: valid project with ingest must return scoped hit; got: {stdout}"
    );
    assert!(
        stdout.contains(PROJECT_A) || stdout.contains("Scope: project="),
        "AC7: Scope should name project A; got: {stdout}"
    );
    // Cross-check: same token under project B must not hit.
    let empty_b = common::hermetic_bin()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_B)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("unique_scoped_token_ac7")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .output()
        .expect("sync query project B isolation");
    let stdout_b = String::from_utf8_lossy(&empty_b.stdout);
    let leak = stdout_b.lines().any(|line| {
        let t = line.trim();
        t.contains("unique_scoped_token_ac7")
            && !t.starts_with("No results")
            && !t.contains("No results for")
    });
    assert!(
        !leak,
        "AC7: project B must not see project A hit; got: {stdout_b}"
    );
}

// ---------------------------------------------------------------------------
// AC13 — --global → Scope: global
// ---------------------------------------------------------------------------

#[test]
fn sync_query__global_flag__scope_global() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);

    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .env("AI_BRAINS_PROJECT_ID", PROJECT_A)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("zzzz-no-hit-ac13")
        .arg("--global")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("sync query --global");

    assert!(
        output.status.success(),
        "AC13: must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Scope: global"),
        "AC13: --global must print Scope: global; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC14 — ndjson with no project → project_id field empty string
// ---------------------------------------------------------------------------

#[test]
fn sync_query__ndjson_no_project__project_id_field_empty() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);
    // Seed a vault-wide-visible hit so NDJSON must emit ≥1 line with project_id="".
    // Codex P2-01: zero records must not pass — that would hide a None-scope drop regression.
    ingest_turn(&vault_path, PROJECT_A, "ndjson_ac14_token");

    let output = common::hermetic_bin()
        .current_dir(dir.path())
        // no AI_BRAINS_PROJECT_ID → project_id=None → vault-wide (F21)
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("ndjson_ac14_token")
        .arg("--format")
        .arg("ndjson")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("5")
        .output()
        .expect("sync query ndjson no project");

    assert!(
        output.status.success(),
        "AC14: must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut saw_record = false;
    for line in stdout.lines() {
        let t = line.trim();
        if t.is_empty() || !t.starts_with('{') {
            continue;
        }
        let v: Value = serde_json::from_str(t).unwrap_or_else(|e| {
            panic!("AC14: ndjson line must parse as JSON: {e}; line={t}");
        });
        saw_record = true;
        let pid = v
            .get("project_id")
            .and_then(|p| p.as_str())
            .unwrap_or_else(|| panic!("AC14: project_id field missing; line={t}"));
        assert_eq!(
            pid, "",
            "AC14: project_id must be empty string when no project; got {pid:?} in {t}"
        );
        // Content should surface the seeded token (vault-wide None path, not random-empty project).
        let content = v
            .pointer("/payload/content")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        assert!(
            content.contains("ndjson_ac14_token")
                || v.to_string().contains("ndjson_ac14_token"),
            "AC14: vault-wide ndjson record must include seeded content; got {t}"
        );
    }

    assert!(
        saw_record,
        "AC14: seeded vault-wide ndjson must emit ≥1 record with project_id=\"\"; got stdout={stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC8 — recall empty pretty includes F13 sync query next-step
// ---------------------------------------------------------------------------

#[test]
fn recall_empty__pretty__includes_sync_query_next_step() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);

    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("zzzz-no-hit-ac8")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("recall empty pretty");

    assert!(
        output.status.success(),
        "AC8: must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sync query"),
        "AC8: empty recall pretty must include sync query next-step; got: {stdout}"
    );
    assert!(
        stdout.contains("For vault + Ledgerful ledger in one view:"),
        "AC8: F13 lead-in required; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC8b — sync empty pretty does NOT self-mention sync query
// ---------------------------------------------------------------------------

#[test]
fn sync_query_empty__pretty__no_sync_query_self_mention() {
    let dir = tempdir().unwrap();
    let vault_path = dir.path().join("vault.db");
    init_vault(&vault_path);

    let output = common::hermetic_bin()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .arg("sync")
        .arg("query")
        .arg("zzzz-no-hit-ac8b")
        .arg("--format")
        .arg("pretty")
        .arg("--quiet")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("sync query empty pretty");

    assert!(
        output.status.success(),
        "AC8b: must succeed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No results") || stdout.contains("Scope:"),
        "AC8b: empty pretty should still show scope/hint; got: {stdout}"
    );
    assert!(
        !stdout.contains("sync query"),
        "AC8b: sync empty must not self-mention sync query; got: {stdout}"
    );
    assert!(
        !stdout.contains("Ledgerful ledger in one view"),
        "AC8b: must not include F13 lead-in; got: {stdout}"
    );
}
