#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_contracts::briefings::{
    BriefingResponse, PersonalContinuityBriefingPacket, ProgressiveQueryResponse,
    ProjectBriefingPacket,
};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_contracts::knowledge::KnowledgeItemResponse;
use ai_brains_contracts::policy::{POLICY_DENIED_CODE, PolicyDenial};
use ai_brains_contracts::preflight::PreflightResponse;
use ai_brains_core::briefing::{
    AuthoritativeClaimKind, AuthoritativeClaimRef, validate_authoritative_claims,
};
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

/// T152-P2-04: golden fixtures must be field-stable under typed serde round-trip.
///
/// Parses fixture → typed DTO → Value, and asserts every field present in the
/// golden JSON is preserved at the same path (catches silent shape drift).
fn assert_golden_field_stable<T>(json: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let golden: serde_json::Value =
        serde_json::from_str(json).expect("golden fixture must be valid JSON");
    let typed: T = serde_json::from_value(golden.clone()).expect("golden must parse as typed DTO");
    let roundtrip = serde_json::to_value(&typed).expect("typed DTO must serialize");
    assert_json_fields_preserved(&golden, &roundtrip, "$");
}

fn assert_json_fields_preserved(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    path: &str,
) {
    match (expected, actual) {
        (serde_json::Value::Object(exp_map), serde_json::Value::Object(act_map)) => {
            for (key, exp_val) in exp_map {
                let child = format!("{path}.{key}");
                // skip_serializing_if may drop null optionals; allow absence when golden is null.
                if exp_val.is_null() && !act_map.contains_key(key) {
                    continue;
                }
                let act_val = act_map
                    .get(key)
                    .unwrap_or_else(|| panic!("golden field missing after round-trip at {child}"));
                assert_json_fields_preserved(exp_val, act_val, &child);
            }
        }
        (serde_json::Value::Array(exp_arr), serde_json::Value::Array(act_arr)) => {
            assert_eq!(
                exp_arr.len(),
                act_arr.len(),
                "array length drift at {path}: golden={} roundtrip={}",
                exp_arr.len(),
                act_arr.len()
            );
            for (i, (ev, av)) in exp_arr.iter().zip(act_arr.iter()).enumerate() {
                assert_json_fields_preserved(ev, av, &format!("{path}[{i}]"));
            }
        }
        _ => {
            assert_eq!(
                expected, actual,
                "value drift at {path}: golden={expected:?} roundtrip={actual:?}"
            );
        }
    }
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

#[test]
fn project_briefing_packet__golden_fixture_parses() {
    let json = fixture("project_briefing_packet.json");
    let packet: ProjectBriefingPacket =
        serde_json::from_str(&json).expect("project briefing golden must parse");
    assert_eq!(packet.api_version, "1");
    assert_eq!(packet.kind, "Project");
    assert_eq!(packet.decisions.len(), 1);
    assert_eq!(packet.conclusions.len(), 1);
    assert!(!packet.decisions[0].evidence_handles.is_empty());
    assert!(!packet.conclusions[0].evidence_handles.is_empty());
    assert_eq!(packet.warnings[0].kind, "stale");
    // Stale is warning-only; no Stale state in current conclusions.
    assert!(packet.conclusions.iter().all(|c| c.state != "Stale"));
    // T152-P2-04: field-stable deep equality vs golden fixture content.
    assert_golden_field_stable::<ProjectBriefingPacket>(&json);
}

#[test]
fn project_briefing_packet__no_personal_nested_field() {
    let json = fixture("project_briefing_packet.json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json value");
    assert!(
        value.get("personal").is_none(),
        "Project packet must not nest a personal field"
    );
    assert!(
        value.get("preferences").is_none(),
        "Project packet must not carry personal preferences"
    );
    let packet: ProjectBriefingPacket = serde_json::from_str(&json).expect("project packet");
    // Round-trip must still lack personal nesting.
    let again = serde_json::to_value(&packet).expect("serialize");
    assert!(again.get("personal").is_none());
}

#[test]
fn personal_briefing_packet__golden_fixture_parses() {
    let json = fixture("personal_briefing_packet.json");
    let packet: PersonalContinuityBriefingPacket =
        serde_json::from_str(&json).expect("personal golden");
    assert_eq!(packet.kind, "Personal");
    assert!(!packet.preferences.is_empty());
    assert!(!packet.grants_applied.is_empty());
    assert!(packet.scope_key.starts_with("Personal:"));
    assert_golden_field_stable::<PersonalContinuityBriefingPacket>(&json);
}

#[test]
fn progressive_query_response__golden_fixture_parses() {
    let json = fixture("progressive_query_response.json");
    let resp: ProgressiveQueryResponse =
        serde_json::from_str(&json).expect("progressive query golden");
    assert_eq!(resp.api_version, "1");
    assert!(!resp.query_trace_id.is_empty());
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].ranking.authority, 80);
    assert!(resp.results[0].state != "Stale");
    assert_golden_field_stable::<ProgressiveQueryResponse>(&json);
}

#[test]
fn project_and_personal_packets__json_round_trip() {
    let project: ProjectBriefingPacket =
        serde_json::from_str(&fixture("project_briefing_packet.json")).unwrap();
    let personal: PersonalContinuityBriefingPacket =
        serde_json::from_str(&fixture("personal_briefing_packet.json")).unwrap();
    let p_json = serde_json::to_string(&project).unwrap();
    let s_json = serde_json::to_string(&personal).unwrap();
    let project2: ProjectBriefingPacket = serde_json::from_str(&p_json).unwrap();
    let personal2: PersonalContinuityBriefingPacket = serde_json::from_str(&s_json).unwrap();
    assert_eq!(project2.briefing_id, project.briefing_id);
    assert_eq!(personal2.briefing_id, personal.briefing_id);
    assert_eq!(project2.decisions.len(), project.decisions.len());
}

#[test]
fn authoritative_claim_validation__without_handles__fails() {
    let claims = vec![AuthoritativeClaimRef {
        kind: AuthoritativeClaimKind::Decision,
        id: "d1".into(),
        evidence_handles: vec![],
    }];
    assert!(validate_authoritative_claims(&claims).is_err());
}

#[test]
fn authoritative_claim_validation__with_handles__ok() {
    let claims = vec![AuthoritativeClaimRef {
        kind: AuthoritativeClaimKind::Conclusion,
        id: "c1".into(),
        evidence_handles: vec!["e1".into()],
    }];
    assert!(validate_authoritative_claims(&claims).is_ok());
}
