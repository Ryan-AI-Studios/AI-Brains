mod common;

fn run_ingest(
    args: &[&str],
    input: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let vault_path = dir.path().join("vault.db");

    let output = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("ingest")
        .args(args)
        .write_stdin(input)
        .output()?;
    Ok(output)
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
    assert_ne!(
        output.status.code(),
        Some(2),
        "empty content is a field error, not empty-stdin usage"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(
        !stderr.contains("stdin is empty or not piped"),
        "empty content must not be classified as empty stdin; stderr={stderr}"
    );
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

fn assert_empty_stdin_usage(output: &std::process::Output) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "empty/whitespace stdin must be fail_usage exit 2; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stdin is empty or not piped"),
        "usage problem text missing; stderr={stderr}"
    );
    assert!(
        stderr.contains("ingest --dry-run"),
        "copy-paste command missing; stderr={stderr}"
    );
    assert!(
        stderr.contains("session_id"),
        "example payload key missing; stderr={stderr}"
    );
    assert!(
        !stderr.contains("COMMAND_FAILED"),
        "empty stdin must not be COMMAND_FAILED; stderr={stderr}"
    );
    assert!(
        !stderr.contains("EOF while parsing"),
        "empty stdin must not be serde EOF; stderr={stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "usage must not write stdout; stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
#[allow(non_snake_case)]
fn ingest__dry_run__empty_stdin__usage_exit_2() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_ingest(&["--dry-run"], "")?;
    assert_empty_stdin_usage(&output);
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__dry_run__whitespace_stdin__usage_exit_2() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_ingest(&["--dry-run"], "\n  \n")?;
    assert_empty_stdin_usage(&output);
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__live__empty_stdin__usage_exit_2() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_ingest(&[], "")?;
    assert_empty_stdin_usage(&output);
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__dry_run__truncated_object__command_failed() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_ingest(&["--dry-run"], "{")?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "mid-payload parse must stay exit 1; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("COMMAND_FAILED") || combined.contains("Invalid JSON"),
        "truncated object must stay payload error; combined={combined}"
    );
    assert!(
        !combined.contains("stdin is empty or not piped"),
        "truncated object must not be usage; combined={combined}"
    );
    Ok(())
}

#[test]
#[allow(non_snake_case)]
fn ingest__help__contains_example_keys() -> Result<(), Box<dyn std::error::Error>> {
    let output = common::hermetic_bin()
        .arg("ingest")
        .arg("--help")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "ingest --help must exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for key in [
        "session_id",
        "project_id",
        "harness_id",
        "turn_id",
        "role",
        "content",
        "privacy",
    ] {
        assert!(
            combined.contains(key),
            "ingest --help after_help must contain {key}; help={combined}"
        );
    }
    Ok(())
}
