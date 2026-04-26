#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_contracts::preflight::PreflightResponse;
use ai_brains_core::privacy::Privacy;

#[test]
fn test_ingest_request_backward_compatibility() {
    // This JSON represents an older version of the contract.
    // It should still parse into the current struct.
    let old_json = r#"
    {
        "session_id": "00000000-0000-0000-0000-000000000001",
        "harness_id": "00000000-0000-0000-0000-000000000002",
        "turn_id": "00000000-0000-0000-0000-000000000003",
        "role": "user",
        "content": "hello",
        "privacy": "LocalOnly"
    }
    "#;

    let request: IngestRequest =
        serde_json::from_str(old_json).expect("Should parse old JSON in test");

    assert_eq!(request.role, "user");
    assert_eq!(request.content, "hello");
    assert_eq!(request.privacy, Privacy::LocalOnly);
    assert!(request.thinking.is_none());
}

#[test]
fn test_preflight_response_backward_compatibility() {
    let old_json = r#"
    {
        "daemon_version": "0.1.0",
        "vault_locked": true,
        "system_healthy": true
    }
    "#;

    let response: PreflightResponse =
        serde_json::from_str(old_json).expect("Should parse old JSON in test");

    assert_eq!(response.daemon_version, "0.1.0");
    assert!(response.vault_locked);
    assert!(response.capabilities.is_empty());
}

#[test]
fn test_new_fields_are_ignored() {
    // Test that adding new fields to the JSON doesn't break deserialization into old structs
    let forward_json = r#"
    {
        "session_id": "00000000-0000-0000-0000-000000000001",
        "harness_id": "00000000-0000-0000-0000-000000000002",
        "turn_id": "00000000-0000-0000-0000-000000000003",
        "role": "user",
        "content": "hello",
        "privacy": "CloudOk",
        "future_field": "some data"
    }
    "#;

    let request: IngestRequest =
        serde_json::from_str(forward_json).expect("Should ignore unknown fields in test");
    assert_eq!(request.privacy, Privacy::CloudOk);
}
