#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::hook::HookResponse;
use serde_json::json;

#[test]
fn test_hook_response_parsing_with_stdout_noise() {
    let raw_output = r#"
    [DEBUG] Starting hook execution...
    Checking environment...
    {"success": true, "result": {"status": "ok"}}
    Hook finished successfully.
    "#;

    let response =
        HookResponse::from_stdout(raw_output).expect("Should parse despite noise in test");

    assert!(response.success);
    assert_eq!(response.result, Some(json!({"status": "ok"})));
}

#[test]
fn test_hook_response_strict_fields() {
    // Ensure that extra fields in the JSON don't break parsing but are ignored by HookResponse
    // (Serde ignores unknown fields by default)
    let raw_output = r#"
    {"success": false, "error": "failed", "noise_field": "ignore me"}
    "#;

    let response: HookResponse = serde_json::from_str(raw_output).expect("Should parse in test");

    assert!(!response.success);
    assert_eq!(response.error, Some("failed".to_string()));
    // We can't easily check that "noise_field" is gone without re-serializing,
    // but the point is it doesn't break the struct.
}

#[test]
fn test_hook_response_parsing_fails_on_invalid_json() {
    let raw_output = "No JSON here, just noise.";
    let response = HookResponse::from_stdout(raw_output);
    assert!(response.is_none());
}
