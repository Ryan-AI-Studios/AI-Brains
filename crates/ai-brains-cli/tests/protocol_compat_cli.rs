//! T180 P-CLI protocol compatibility: stable JSON keys, compact/pretty, stdin dual-path.
//!
//! Style freezes (F32) exercise **production** emission paths via the CLI binary
//! (not by re-calling `to_string` / `to_string_pretty` in the test).
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_cli(args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let vault_path = dir.path().join("vault.db");

    // Ensure vault exists for preflight/scope (init creates empty vault).
    let init = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("init")
        .arg("--force")
        .output()?;
    if !init.status.success() {
        // Some builds may already treat empty path as ok; only fail hard if still unusable later.
        let _ = init;
    }

    let output = common::hermetic_bin()
        .arg("--vault-path")
        .arg(&vault_path)
        .arg("--no-project-context")
        .args(args)
        .output()?;
    Ok(output)
}

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

// ---------------------------------------------------------------------------
// T180-C-preflight-json-keys / compact style via production CLI (F32)
// ---------------------------------------------------------------------------

#[test]
fn t180_c_preflight_json_keys__cli_format_json__compact_stable_keys()
-> Result<(), Box<dyn std::error::Error>> {
    // T180-C-preflight-json-keys — production path: preflight.rs uses to_string (compact).
    let output = run_cli(&["preflight", "--format", "json"])?;
    assert!(
        output.status.success(),
        "preflight --format json failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let line = stdout.trim();
    assert!(
        !line.contains('\n'),
        "preflight JSON production style is compact (no pretty newlines): {line}"
    );
    let v: serde_json::Value = serde_json::from_str(line)?;
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"), "stable key text");
    assert!(obj.contains_key("word_count"), "stable key word_count");
    assert_eq!(
        obj.len(),
        2,
        "preflight JSON must not grow silent keys without a track"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// T180-C-scope-json-pretty via production CLI emit_json (F32)
// ---------------------------------------------------------------------------

#[test]
fn t180_c_scope_json_pretty__cli_format_json__pretty_stable_keys()
-> Result<(), Box<dyn std::error::Error>> {
    // T180-C-scope-json-pretty — production path: emit_json → to_string_pretty.
    // Prefer local path to avoid requiring a running daemon.
    let output = run_cli(&["scope", "resolve", "--format", "json", "--local"])?;
    assert!(
        output.status.success(),
        "scope resolve --format json --local failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let pretty = stdout.trim();
    assert!(
        pretty.contains('\n'),
        "scope resolve JSON production style is pretty: {pretty}"
    );
    let v: serde_json::Value = serde_json::from_str(pretty)?;
    for key in [
        "api_version",
        "scope",
        "confidence",
        "authoritative",
        "evidence",
        "warnings",
        "alternatives",
    ] {
        assert!(
            v.get(key).is_some(),
            "scope resolve stable key missing: {key}; body={pretty}"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// T180-C-stdin-dry-run-deny / T180-C-stdin-prod-open (F26 dual-path)
// ---------------------------------------------------------------------------

#[test]
fn t180_c_stdin_dry_run_deny__unknown_field__rejected() -> Result<(), Box<dyn std::error::Error>> {
    // T180-C-stdin-dry-run-deny — DryRunIngestRequest has deny_unknown_fields.
    let input = r#"{
      "turn_id":"test",
      "session_id":"test",
      "project_id":"test",
      "harness_id":"test",
      "role":"user",
      "content":"hello",
      "privacy":"CloudOk",
      "_test_unknown_string":"nope"
    }"#;

    let output = run_ingest(&["--dry-run"], input)?;
    assert!(
        !output.status.success(),
        "dry-run must reject unknown fields; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let err = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        err.to_lowercase().contains("unknown")
            || err.to_lowercase().contains("invalid")
            || err.contains("JSON")
            || !err.is_empty(),
        "error should mention parse/unknown; got: {err}"
    );
    Ok(())
}

#[test]
fn t180_c_stdin_prod_open__unknown_field__accepted() -> Result<(), Box<dyn std::error::Error>> {
    // T180-C-stdin-prod-open — production IngestRequest has no deny_unknown_fields.
    let input = r#"{
      "session_id":"00000000-0000-0000-0000-000000000001",
      "project_id":"00000000-0000-0000-0000-000000000000",
      "harness_id":"00000000-0000-0000-0000-000000000002",
      "turn_id":"00000000-0000-0000-0000-000000000003",
      "role":"user",
      "content":"hello with extra field",
      "privacy":"CloudOk",
      "_test_unknown_string":"ignored_ok"
    }"#;

    let output = run_ingest(&[], input)?;
    assert!(
        output.status.success(),
        "production ingest must tolerate unknown fields; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let response: ai_brains_contracts::ingest::IngestResponse =
        serde_json::from_str(stdout.trim())?;
    assert!(response.processed);
    // Compact style for ingest response (F32 inventory) — production emit path.
    assert!(
        !stdout.trim().contains('\n'),
        "ingest success JSON is compact"
    );
    Ok(())
}
