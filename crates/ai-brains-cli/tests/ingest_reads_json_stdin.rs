use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn ingest_reads_json_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-brains"))
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let input = r#"{
      "session_id":"00000000-0000-0000-0000-000000000001",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000002",
      "turn_id":"00000000-0000-0000-0000-000000000003",
      "role":"user",
      "content":"hello from stdin",
      "privacy":"CloudOk"
    }"#;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let response: ai_brains_contracts::ingest::IngestResponse =
        serde_json::from_str(stdout.trim())?;
    assert!(response.processed);
    assert!(!response.event_id.is_empty());
    Ok(())
}
