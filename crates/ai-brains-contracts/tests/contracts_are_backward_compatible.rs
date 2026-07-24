#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::briefings::BriefingResponse;
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_contracts::knowledge::KnowledgeItemResponse;
use ai_brains_contracts::policy::{POLICY_DENIED_CODE, PolicyDenial};
use ai_brains_contracts::preflight::PreflightResponse;
use ai_brains_core::privacy::Privacy;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_ingest_request_backward_compatibility() {
    // This JSON represents an older version of the contract.
    // It should still parse into the current struct.
    let old_json = r#"
    {
        "session_id": "00000000-0000-0000-0000-000000000001",
        "project_id": "00000000-0000-0000-0000-000000000000",
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
        "project_id": "00000000-0000-0000-0000-000000000000",
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

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn knowledge_item_response__golden_fixture_parses() {
    let json = fixture("knowledge_item.json");
    let resp: KnowledgeItemResponse =
        serde_json::from_str(&json).expect("knowledge golden must parse");
    assert_eq!(resp.api_version, "1");
    assert_eq!(resp.item.evidence_handles.len(), 1);
    assert_eq!(
        resp.item.evidence_handles[0].evidence_id,
        "00000000-0000-0000-0000-0000000000e1"
    );
}

#[test]
fn briefing_response__minimal_without_optional_timestamps() {
    let json = fixture("briefing_shell.json");
    let resp: BriefingResponse = serde_json::from_str(&json).expect("briefing golden");
    assert_eq!(resp.api_version, "1");
    assert!(resp.briefing.generated_at.is_none());
    assert!(!resp.briefing.evidence_handles.is_empty());
}

#[test]
fn knowledge_item__minimal_json_without_optional_fields_parses() {
    let minimal = r#"
    {
        "api_version": "1",
        "item": {
            "id": "00000000-0000-0000-0000-0000000000a2",
            "kind": "Decision",
            "statement": "Ship ports first",
            "state": "Proposed"
        }
    }
    "#;
    let resp: KnowledgeItemResponse =
        serde_json::from_str(minimal).expect("defaults for evidence_handles");
    assert!(resp.item.evidence_handles.is_empty());
    assert!(resp.item.updated_at.is_none());
}

#[test]
fn policy_denial__has_code_and_message_not_empty_success() {
    let json = fixture("policy_denial.json");
    let denial: PolicyDenial = serde_json::from_str(&json).expect("policy denial golden");
    assert_eq!(denial.code, POLICY_DENIED_CODE);
    assert!(!denial.message.is_empty());
    assert_eq!(denial.api_version, "1");
    let api_err = denial.to_api_error();
    assert_eq!(api_err.code, POLICY_DENIED_CODE);
    assert!(!api_err.message.is_empty());
}

#[test]
fn offset_to_utc_helper__converts_domain_timestamps() {
    use ai_brains_contracts::offset_to_utc;
    use time::OffsetDateTime;
    let t = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("ts");
    let utc = offset_to_utc(t);
    assert_eq!(utc.timestamp(), 1_700_000_000);
}
