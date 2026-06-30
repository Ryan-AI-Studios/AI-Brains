#![allow(clippy::disallowed_methods)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// Test that preflight with --scope works without crashing (fail-open when
/// Ledgerful is unavailable).
#[test]
fn test_preflight_with_scope_does_not_crash() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // 1. Initialize the vault
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // 2. Run preflight with --scope flag (Ledgerful likely not available,
    //    but preflight should not crash due to fail-open)
    let mut preflight_cmd = Command::cargo_bin("ai-brains")?;
    preflight_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--scope")
        .arg("crates/ai-brains-cli/src/main.rs,crates/ai-brains-store/src/lib.rs")
        .arg("--pretty")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .assert()
        .success();

    Ok(())
}

/// Test that preflight with scope and without scope both work.
#[test]
fn test_preflight_with_and_without_scope() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // Initialize and pin
    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    // Run preflight without scope
    let mut preflight_no_scope = Command::cargo_bin("ai-brains")?;
    preflight_no_scope
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--pretty")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .assert()
        .success();

    // Run preflight with scope (should not crash even if Ledgerful unavailable)
    let mut preflight_with_scope = Command::cargo_bin("ai-brains")?;
    preflight_with_scope
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--scope")
        .arg("src/lib.rs")
        .arg("--pretty")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .assert()
        .success();

    Ok(())
}

/// Test that scope from AI_BRAINS_SCOPE env var is picked up.
#[test]
fn test_preflight_scope_from_env_var() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let mut preflight_cmd = Command::cargo_bin("ai-brains")?;
    preflight_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--pretty")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .env("AI_BRAINS_SCOPE", "crates/lib.rs,tests/integration.rs")
        .assert()
        .success();

    Ok(())
}

/// Test that preflight JSON output includes word_count even with scope.
#[test]
fn test_preflight_json_output_with_scope() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let mut init_cmd = Command::cargo_bin("ai-brains")?;
    init_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .assert()
        .success();

    let mut preflight_cmd = Command::cargo_bin("ai-brains")?;
    preflight_cmd
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("preflight")
        .arg("--scope")
        .arg("crates/db/src/lib.rs")
        .env(
            "AI_BRAINS_PROJECT_ID",
            "00000000-0000-0000-0000-000000000001",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("\"word_count\":"));

    Ok(())
}
