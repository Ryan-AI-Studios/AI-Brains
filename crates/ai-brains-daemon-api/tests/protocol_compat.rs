//! T180 protocol compatibility gap-fill: additive helper, honesty, bridge capture, fixture drift.
//!
//! Elevate map for the existing T158 suite lives in `protocol_wire.rs` (comments) and
//! `Docs/PROTOCOL-COMPAT.md` §9.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_contracts::bridge::BridgePayload;
use ai_brains_contracts::briefings::{ProjectBriefingRequest, QueryKnowledgeRequest};
use ai_brains_contracts::erasure::RequestErasureRequest;
use ai_brains_contracts::knowledge::ProposeConclusionRequest;
use ai_brains_contracts::review::ListReviewItemsRequest;
use ai_brains_contracts::scopes::{API_VERSION, ResolveScopeRequest, ScopeResolvedResponse};
use ai_brains_contracts::sources::InspectSourceRequest;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};

// ---------------------------------------------------------------------------
// F29 — shared additive-field helper
// ---------------------------------------------------------------------------

/// Assert that a public wire DTO deserializes when unknown top-level fields are present.
///
/// Serde's default (no `deny_unknown_fields`) is the forward-compat contract for public wire.
/// Do **not** apply this helper to dry-run-only types that intentionally deny unknowns.
pub fn assert_deserializes_with_extra_fields<T: serde::de::DeserializeOwned>(
    base_fixture_json: &str,
) {
    let mut val: serde_json::Value =
        serde_json::from_str(base_fixture_json).expect("fixture JSON must parse");
    if let Some(obj) = val.as_object_mut() {
        obj.insert(
            "_test_unknown_string".into(),
            serde_json::Value::String("unknown_value".into()),
        );
        obj.insert("_test_unknown_number".into(), serde_json::json!(42));
        obj.insert(
            "_test_unknown_object".into(),
            serde_json::json!({"nested": true}),
        );
    } else {
        panic!("fixture must be a JSON object for additive-field injection");
    }
    let _: T =
        serde_json::from_value(val).expect("public wire DTO must tolerate additive unknown fields");
}

// ---------------------------------------------------------------------------
// T180-D-additive-extra-field
// ---------------------------------------------------------------------------

#[test]
fn t180_d_additive_extra_field__resolve_scope_request__tolerates_unknowns() {
    // T180-D-additive-extra-field
    let base = serde_json::to_string(&ResolveScopeRequest {
        api_version: API_VERSION.to_string(),
        cwd: Some("C:/dev/AI-Brains".into()),
        signals: None,
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<ResolveScopeRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__project_briefing_request__tolerates_unknowns() {
    let base = serde_json::to_string(&ProjectBriefingRequest {
        api_version: API_VERSION.to_string(),
        principal_id: None,
        scope: None,
        cwd: None,
        max_words: None,
        governed_briefing: None,
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<ProjectBriefingRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__query_knowledge_request__tolerates_unknowns() {
    let base = serde_json::to_string(&QueryKnowledgeRequest {
        api_version: API_VERSION.to_string(),
        query: "budget".into(),
        scope: None,
        principal_id: None,
        limit: Some(5),
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<QueryKnowledgeRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__propose_conclusion_request__tolerates_unknowns() {
    let base = serde_json::to_string(&ProposeConclusionRequest {
        api_version: API_VERSION.to_string(),
        principal_id: None,
        scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
        statement: "x".into(),
        evidence_ids: vec![],
        privacy: None,
        command_id: None,
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<ProposeConclusionRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__list_review_items_request__tolerates_unknowns() {
    let base = serde_json::to_string(&ListReviewItemsRequest::default()).expect("ser");
    assert_deserializes_with_extra_fields::<ListReviewItemsRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__request_erasure_request__tolerates_unknowns() {
    let base = serde_json::to_string(&RequestErasureRequest {
        api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
        principal_id: None,
        ids: vec!["agg-1".into()],
        reason: None,
        scope: None,
        command_id: None,
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<RequestErasureRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__inspect_source_request__tolerates_unknowns() {
    let base = serde_json::to_string(&InspectSourceRequest {
        api_version: ai_brains_contracts::sources::API_VERSION.to_string(),
        id: "src-1".into(),
        principal_id: None,
        scope: None,
    })
    .expect("ser");
    assert_deserializes_with_extra_fields::<InspectSourceRequest>(&base);
}

#[test]
fn t180_d_additive_extra_field__daemon_request_envelope__legacy_ping() {
    // Wire envelope is also public: unknown sibling fields beside type must not break known ops
    // when injected into payload-bearing bodies. Ping has no payload object — inject at root
    // only works if DaemonRequest ignores unknown fields at the enum wrapper level.
    // For adjacent-tagged enums, unknown fields next to `type` are typically ignored by serde.
    let mut val: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy_ping_request.json")).expect("fixture");
    if let Some(obj) = val.as_object_mut() {
        obj.insert("_test_unknown_string".into(), "x".into());
    }
    let decoded: DaemonRequest =
        serde_json::from_value(val).expect("DaemonRequest must ignore additive root fields");
    assert!(matches!(decoded, DaemonRequest::Ping));
}

// ---------------------------------------------------------------------------
// T180-D-api-version-presence / honesty (F25 / F33)
// ---------------------------------------------------------------------------

#[test]
fn t180_d_api_version_presence__scope_resolved_serializes_field() {
    // T180-D-api-version-presence
    let resp = ScopeResolvedResponse {
        api_version: API_VERSION.to_string(),
        scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
        confidence: "Low".into(),
        authoritative: false,
        evidence: vec![],
        warnings: vec![],
        alternatives: vec![],
    };
    let v = serde_json::to_value(&resp).expect("ser");
    assert_eq!(v.get("api_version").and_then(|x| x.as_str()), Some("1"));
}

#[test]
fn t180_d_api_version_honesty__version_2_accepted_on_resolve_scope() {
    // T180 — honesty: api_version is declarative, not enforced (update when enforcement lands).
    let raw = r#"{
        "api_version": "2",
        "cwd": "C:/dev/AI-Brains",
        "force_personal": false
    }"#;
    let decoded: ResolveScopeRequest =
        serde_json::from_str(raw).expect("api_version 2 must still deserialize today");
    assert_eq!(decoded.api_version, "2");
}

#[test]
fn t180_d_api_version_honesty__version_1_accepted_on_resolve_scope() {
    let raw = r#"{
        "api_version": "1",
        "cwd": "C:/dev/AI-Brains",
        "force_personal": false
    }"#;
    let decoded: ResolveScopeRequest =
        serde_json::from_str(raw).expect("api_version 1 must deserialize");
    assert_eq!(decoded.api_version, "1");
}

#[test]
fn t180_d_api_version_honesty__banana_accepted_on_query_knowledge() {
    // Extreme honesty case: any string is accepted after deserialize (no runtime gate).
    let raw = r#"{
        "api_version": "banana",
        "query": "hello"
    }"#;
    let decoded: QueryKnowledgeRequest =
        serde_json::from_str(raw).expect("unenforced api_version accepts arbitrary strings today");
    assert_eq!(decoded.api_version, "banana");
}

// ---------------------------------------------------------------------------
// T180-D-fixture-drift (F37) — N−1 proof for key goldens
// ---------------------------------------------------------------------------

#[test]
fn t180_d_fixture_drift__legacy_ping__roundtrip_stable() {
    // T180-D-fixture-drift-legacy-ping
    let raw = include_str!("fixtures/legacy_ping_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("de");
    let again = serde_json::to_value(&decoded).expect("ser");
    let original: serde_json::Value = serde_json::from_str(raw).expect("parse");
    assert_eq!(
        again, original,
        "legacy ping golden must match current serialize (full structural)"
    );
    assert!(matches!(decoded, DaemonRequest::Ping));
}

#[test]
fn t180_d_fixture_drift__scope_resolved__reparse_matches() {
    // T180-D-fixture-drift-scope-resolved
    let raw = include_str!("fixtures/scope_resolved_response.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("de");
    let again = serde_json::to_value(&decoded).expect("ser");
    let original: serde_json::Value = serde_json::from_str(raw).expect("parse");
    // Structural equality after normalize (both Values).
    assert_eq!(
        again, original,
        "scope_resolved golden must match current serialize"
    );
}

#[test]
fn t180_d_fixture_drift__policy_denied__reparse_matches() {
    let raw = include_str!("fixtures/policy_denied_error.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("de");
    let again = serde_json::to_value(&decoded).expect("ser");
    let original: serde_json::Value = serde_json::from_str(raw).expect("parse");
    assert_eq!(
        again, original,
        "policy_denied golden must match current serialize"
    );
}

// ---------------------------------------------------------------------------
// T180-D-bridge-unknown (F31) — capture policy (opposite of fail-closed types)
// ---------------------------------------------------------------------------

#[test]
fn t180_d_bridge_unknown__unknown_shape__captured_as_unknown() {
    // T180-D-bridge-unknown
    let raw = r#"{
        "type": "FutureBridgeThing",
        "extra": 99,
        "nested": { "ok": true }
    }"#;
    let decoded: BridgePayload = serde_json::from_str(raw).expect("unknown bridge must capture");
    match decoded {
        BridgePayload::Unknown(v) => {
            assert_eq!(
                v.get("type").and_then(|t| t.as_str()),
                Some("FutureBridgeThing")
            );
            assert_eq!(v.get("extra").and_then(|e| e.as_i64()), Some(99));
        }
        other => panic!("expected BridgePayload::Unknown, got {other:?}"),
    }
}

#[test]
fn t180_d_bridge_unknown__roundtrip_preserves_value() {
    let raw = serde_json::json!({
        "type": "TotallyNew",
        "foo": "bar"
    });
    let payload = BridgePayload::Unknown(raw.clone());
    let ser = serde_json::to_value(&payload).expect("ser");
    let back: BridgePayload = serde_json::from_value(ser).expect("de");
    match back {
        BridgePayload::Unknown(v) => assert_eq!(v, raw),
        other => panic!("expected Unknown, got {other:?}"),
    }
}

#[test]
fn t180_d_bridge_known_query__still_typed() {
    let raw = r#"{"type":"Query","text":"what is the decision?"}"#;
    let decoded: BridgePayload = serde_json::from_str(raw).expect("known query");
    match decoded {
        BridgePayload::Query { text, .. } => assert_eq!(text, "what is the decision?"),
        other => panic!("expected Query, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T180-D-fixture-governed (B4 file-backed N−1 for high-traffic ops)
// ---------------------------------------------------------------------------

#[test]
fn t180_d_fixture_governed__resolve_scope_request__deserializes() {
    // T180-D-fixture-governed-resolve-scope
    let raw = include_str!("fixtures/governed_resolve_scope_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("governed resolve_scope fixture");
    match decoded {
        DaemonRequest::ResolveScope(req) => {
            assert_eq!(req.api_version, "1");
            assert_eq!(req.cwd.as_deref(), Some("C:/dev/AI-Brains"));
            assert!(!req.force_personal);
        }
        other => panic!("expected ResolveScope, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// F25 — eight module-local API_VERSION constants (not P-SYNC; P-SYNC is docs-only)
// ---------------------------------------------------------------------------

#[test]
fn t180_f25_api_version_constants__all_modules__are_one() {
    // Documents the eight module-local API_VERSION = "1" constants (F25).
    // P-SYNC index (T180-S-index) is documentation-only in Docs/PROTOCOL-COMPAT.md §9.5.
    assert_eq!(ai_brains_contracts::scopes::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::briefings::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::knowledge::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::review::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::erasure::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::retention::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::policy::API_VERSION, "1");
    assert_eq!(ai_brains_contracts::sources::API_VERSION, "1");
}
