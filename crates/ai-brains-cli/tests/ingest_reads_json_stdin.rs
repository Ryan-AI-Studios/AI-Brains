use std::io::Write;
use std::process::{Command, Stdio};

fn run_ingest(
    args: &[&str],
    input: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let mut child = Command::new(env!("CARGO_BIN_EXE_ai-brains"))
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    Ok(child.wait_with_output()?)
}

#[test]
fn ingest_reads_json_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{
      "session_id":"00000000-0000-0000-0000-000000000001",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000002",
      "turn_id":"00000000-0000-0000-0000-000000000003",
      "role":"user",
      "content":"hello from stdin",
      "privacy":"CloudOk"
    }"#;

    let output = run_ingest(&[], input)?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let response: ai_brains_contracts::ingest::IngestResponse =
        serde_json::from_str(stdout.trim())?;
    assert!(response.processed);
    assert!(!response.event_id.is_empty());
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__dry_run__accepts_placeholder_uuids() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{
      "turn_id":"test",
      "session_id":"test",
      "project_id":"test",
      "harness_id":"test",
      "role":"user",
      "content":"hello",
      "privacy":"CloudOk"
    }"#;

    let output = run_ingest(&["--dry-run"], input)?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("[dry-run] Would ingest turn test"));
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__dry_run__errors_on_empty_content() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{
      "turn_id":"test",
      "session_id":"test",
      "project_id":"test",
      "harness_id":"test",
      "role":"user",
      "content":"",
      "privacy":"CloudOk"
    }"#;

    let output = run_ingest(&["--dry-run"], input)?;
    assert!(!output.status.success());
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__non_dry_run__still_validates_uuids() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{
      "turn_id":"test",
      "session_id":"test",
      "project_id":"test",
      "harness_id":"test",
      "role":"user",
      "content":"hello",
      "privacy":"CloudOk"
    }"#;

    let output = run_ingest(&[], input)?;
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("UUID"));
    Ok(())
}
