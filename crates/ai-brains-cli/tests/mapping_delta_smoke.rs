#![allow(non_snake_case)] // function_or_feature__condition__expected_result

mod common;

use predicates::prelude::*;
use std::io::Write;
use tempfile::tempdir;

#[test]
#[allow(clippy::disallowed_methods)]
fn test_project_mapping_and_delta_sync() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let vault_path = temp_dir.path().join("vault.db");

    // 1. Init vault
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Setup project context (env project for session id only; F3 must not steal it
    //    for non-unbound hashes).
    let project_id = "00000000-0000-0000-0000-000000001234";
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("context")
        .arg("--new-project")
        .assert()
        .success();

    // Read the session_id that context created so agy-hook targets the same session.
    let env_content = std::fs::read_to_string(temp_dir.path().join(".env"))?;
    let session_id = env_content
        .lines()
        .find(|l| l.starts_with("AI_BRAINS_SESSION_ID"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 3. Create a mock agy transcript (legacy role/content still supported)
    let agy_dir = temp_dir.path().join("agy-chats");
    std::fs::create_dir_all(&agy_dir)?;
    let transcript_path = agy_dir.join("transcript.jsonl");
    let mut file = std::fs::File::create(&transcript_path)?;
    writeln!(
        file,
        r#"{{"role": "user", "content": "hello", "timestamp": "2026-05-24T12:00:00Z"}}"#
    )?;

    // F3/F33: real workspace path as projectHash → path-derived project (NOT env steal).
    // Env AI_BRAINS_PROJECT_ID is set but must not hijack when hash is non-unbound.
    let project_hash = temp_dir.path().join("workspace-proj");
    std::fs::create_dir_all(&project_hash)?;
    let project_hash_str = project_hash.to_string_lossy().to_string();

    let payload = serde_json::json!({
        "transcriptPath": transcript_path.to_string_lossy(),
        "sessionId": session_id,
        "projectHash": project_hash_str
    });

    // 4. Run agy-hook (path-derived bind; diagnostics on stderr)
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("agy-hook")
        .arg("--payload")
        .arg(serde_json::to_string(&payload)?)
        .assert()
        .success()
        .stderr(predicate::str::contains("Auto-linked projectHash"));

    // 5. Verify turn ingested (global: path-derived project ≠ env project)
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("hello")
        .arg("--global")
        .arg("--no-bridge")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));

    // 6. Add another turn and run agy-hook again (delta sync)
    writeln!(
        file,
        r#"{{"role": "assistant", "content": "world", "timestamp": "2026-05-24T12:01:00Z"}}"#
    )?;

    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("agy-hook")
        .arg("--payload")
        .arg(serde_json::to_string(&payload)?)
        .assert()
        .success()
        .stderr(predicate::str::contains("Successfully ingested 1 turns")); // new turn only

    Ok(())
}

/// AC20: unresolved path does not fall back to AI_BRAINS_PROJECT_ID.
#[test]
#[allow(clippy::disallowed_methods)]
fn agy_hook__unresolved_path__no_env_hijack() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let vault_path = temp_dir.path().join("vault.db");

    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let env_project = "00000000-0000-0000-0000-000000009999";
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", env_project)
        .arg("context")
        .arg("--new-project")
        .assert()
        .success();

    let agy_dir = temp_dir.path().join("agy");
    std::fs::create_dir_all(&agy_dir)?;
    let transcript = agy_dir.join("t.jsonl");
    std::fs::write(
        &transcript,
        r#"{"role":"user","content":"no-hijack"}
"#,
    )?;

    let workspace = temp_dir.path().join("other-ws");
    std::fs::create_dir_all(&workspace)?;
    let session = uuid::Uuid::new_v4().to_string();
    let payload = serde_json::json!({
        "transcriptPath": transcript.to_string_lossy(),
        "sessionId": session,
        "projectHash": workspace.to_string_lossy(),
    });

    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", env_project)
        .arg("agy-hook")
        .arg("--payload")
        .arg(serde_json::to_string(&payload)?)
        .assert()
        .success();

    // Recall without global under env project should NOT see the turn (not hijacked).
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .env("AI_BRAINS_PROJECT_ID", env_project)
        .arg("recall")
        .arg("no-hijack")
        .arg("--no-bridge")
        .assert()
        .success();
    // Global should see it under path-derived project
    let mut cmd = common::hermetic_bin();
    cmd.current_dir(temp_dir.path())
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("recall")
        .arg("no-hijack")
        .arg("--global")
        .arg("--no-bridge")
        .assert()
        .success()
        .stdout(predicate::str::contains("no-hijack"));

    Ok(())
}
