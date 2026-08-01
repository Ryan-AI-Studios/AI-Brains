mod common;

#[test]
fn cli_capture_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let input = r#"{
      "session_id":"00000000-0000-0000-0000-000000000011",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000022",
      "turn_id":"00000000-0000-0000-0000-000000000033",
      "role":"assistant",
      "content":"final output",
      "privacy":"CloudOk",
      "thinking":"hidden"
    }"#;

    let output = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--log-format")
        .arg("off")
        .arg("ingest")
        // Isolate from ambient shell noise not covered by hermetic denylist.
        .env_remove("RUST_LOG")
        .write_stdin(input)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "ingest must exit 0; status={:?}; stdout={stdout}; stderr={stderr}",
        output.status
    );
    assert!(
        stderr.trim().is_empty(),
        "ingest stderr must be empty; got: {stderr}"
    );
    let response: ai_brains_contracts::ingest::IngestResponse = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("ingest stdout not IngestResponse JSON ({e}): {stdout}"))?;
    assert!(response.processed);
    Ok(())
}
