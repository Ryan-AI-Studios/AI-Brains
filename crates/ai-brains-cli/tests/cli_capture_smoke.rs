use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn cli_capture_smoke() -> Result<(), Box<dyn std::error::Error>> {
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
      "session_id":"00000000-0000-0000-0000-000000000011",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000022",
      "turn_id":"00000000-0000-0000-0000-000000000033",
      "role":"assistant",
      "content":"final output",
      "privacy":"CloudOk",
      "thinking":"hidden"
    }"#;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout)?;
    let response: ai_brains_contracts::ingest::IngestResponse =
        serde_json::from_str(stdout.trim())?;
    assert!(response.processed);
    Ok(())
}
