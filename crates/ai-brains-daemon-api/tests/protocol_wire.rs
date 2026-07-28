//! Golden wire tests for DaemonRequest / DaemonResponse (T158).
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_contracts::briefings::{
    BriefingScopeDto, HandlePreviewDto, InspectEvidenceRequest, PersonalBriefingRequest,
    PersonalBriefingResponse, PersonalContinuityBriefingPacket, ProgressiveQueryResponse,
    ProjectBriefingPacket, ProjectBriefingRequest, ProjectBriefingResponse, QueryKnowledgeRequest,
};
use ai_brains_contracts::erasure::{ErasureAcceptedResponse, RequestErasureRequest};
use ai_brains_contracts::knowledge::{
    ConclusionProposedResponse, DecisionProposedResponse, ProposeConclusionRequest,
    ProposeDecisionRequest,
};
use ai_brains_contracts::policy::{POLICY_DENIED_CODE, PolicyDenial};
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::review::{
    ListReviewItemsRequest, ResolveReviewItemRequest, ReviewQueueResponse, ReviewResolvedResponse,
};
use ai_brains_contracts::scopes::{
    API_VERSION, ResolveScopeRequest, ScopeEvidenceDto, ScopeResolvedResponse,
};
use ai_brains_contracts::sources::{InspectSourceRequest, SourceDto};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse, UNSUPPORTED_OPERATION};

fn assert_roundtrip_request(req: DaemonRequest) {
    let json = serde_json::to_string(&req).expect("serialize request");
    let decoded: DaemonRequest = serde_json::from_str(&json).expect("deserialize request");
    let again = serde_json::to_string(&decoded).expect("re-serialize");
    // Structural equality via re-parse to Value (enums may not implement PartialEq).
    let a: serde_json::Value = serde_json::from_str(&json).unwrap();
    let b: serde_json::Value = serde_json::from_str(&again).unwrap();
    assert_eq!(a, b, "request roundtrip mismatch: {json}");
}

fn assert_roundtrip_response(resp: DaemonResponse) {
    let json = serde_json::to_string(&resp).expect("serialize response");
    let decoded: DaemonResponse = serde_json::from_str(&json).expect("deserialize response");
    let again = serde_json::to_string(&decoded).expect("re-serialize");
    let a: serde_json::Value = serde_json::from_str(&json).unwrap();
    let b: serde_json::Value = serde_json::from_str(&again).unwrap();
    assert_eq!(a, b, "response roundtrip mismatch: {json}");
}

// ---------------------------------------------------------------------------
// AC1 — legacy goldens
// ---------------------------------------------------------------------------

#[test]
fn daemon_request__legacy_ping_json__deserializes() {
    let raw = include_str!("fixtures/legacy_ping_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("legacy ping");
    assert!(matches!(decoded, DaemonRequest::Ping));
}

#[test]
fn daemon_request__legacy_shutdown__deserializes() {
    let raw = include_str!("fixtures/legacy_shutdown_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("legacy shutdown");
    assert!(matches!(decoded, DaemonRequest::Shutdown));
}

#[test]
fn daemon_request__legacy_ingest_json__deserializes() {
    let raw = include_str!("fixtures/legacy_ingest_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("legacy ingest");
    match decoded {
        DaemonRequest::Ingest(req) => {
            assert_eq!(req.role, "user");
            assert_eq!(req.content, "hello world");
        }
        other => panic!("expected Ingest, got {other:?}"),
    }
}

#[test]
fn daemon_request__legacy_sync_json__deserializes() {
    let raw = include_str!("fixtures/legacy_sync_request.json");
    let decoded: DaemonRequest = serde_json::from_str(raw).expect("legacy sync");
    match decoded {
        DaemonRequest::Sync(record) => {
            assert_eq!(record.record_kind, "query");
            assert_eq!(record.bridge_version, "0.3");
        }
        other => panic!("expected Sync, got {other:?}"),
    }
}

#[test]
fn daemon_response__legacy_pong__deserializes() {
    let raw = include_str!("fixtures/legacy_pong_response.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("legacy pong");
    assert!(matches!(decoded, DaemonResponse::Pong));
}

#[test]
fn daemon_response__error_api_error__roundtrip() {
    let raw = include_str!("fixtures/legacy_error_response.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("legacy error");
    match decoded {
        DaemonResponse::Error(err) => {
            assert_eq!(err.code, "DAEMON_ERROR");
            assert_eq!(err.message, "queue closed");
        }
        other => panic!("expected Error, got {other:?}"),
    }
    assert_roundtrip_response(DaemonResponse::Error(ApiError::new(
        "DAEMON_ERROR",
        "queue closed",
    )));
}

// ---------------------------------------------------------------------------
// AC3 — unknown type fail-closed
// ---------------------------------------------------------------------------
// Serde-level rejection is the first gate. Live hosts must also surface
// INVALID_REQUEST via `ai_brainsd::dispatch::parse_live_request_line` (see
// `daemon_dispatch_shared` tests) so clients never hang on silent drop.

#[test]
fn daemon_request__unknown_type__fails_deserialize() {
    let raw = r#"{"type":"not_a_real_op","payload":{}}"#;
    let err = serde_json::from_str::<DaemonRequest>(raw).expect_err("unknown must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown variant") || msg.contains("not_a_real_op") || !msg.is_empty(),
        "unexpected error: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — new request variants
// ---------------------------------------------------------------------------

#[test]
fn daemon_request__resolve_scope__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ResolveScope(ResolveScopeRequest {
        api_version: API_VERSION.to_string(),
        cwd: Some("C:/dev/AI-Brains".into()),
        signals: None,
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
    }));
}

#[test]
fn daemon_request__project_briefing__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ProjectBriefing(ProjectBriefingRequest {
        api_version: API_VERSION.to_string(),
        principal_id: Some("p1".into()),
        scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
        cwd: None,
        max_words: Some(500),
        governed_briefing: Some(true),
    }));
}

#[test]
fn daemon_request__personal_briefing__roundtrip() {
    assert_roundtrip_request(DaemonRequest::PersonalBriefing(PersonalBriefingRequest {
        api_version: API_VERSION.to_string(),
        principal_id: None,
        scope: Some("Personal:00000000-0000-0000-0000-0000000000u1".into()),
        max_words: None,
        governed_briefing: None,
    }));
}

#[test]
fn daemon_request__query_knowledge__roundtrip() {
    assert_roundtrip_request(DaemonRequest::QueryKnowledge(QueryKnowledgeRequest {
        api_version: API_VERSION.to_string(),
        query: "budget".into(),
        scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
        principal_id: None,
        limit: Some(5),
    }));
}

#[test]
fn daemon_request__inspect_evidence__roundtrip() {
    assert_roundtrip_request(DaemonRequest::InspectEvidence(InspectEvidenceRequest {
        api_version: API_VERSION.to_string(),
        id: "e1".into(),
        scope: None,
        principal_id: None,
        max_chars: Some(256),
    }));
}

#[test]
fn daemon_request__inspect_source__roundtrip() {
    assert_roundtrip_request(DaemonRequest::InspectSource(InspectSourceRequest {
        api_version: API_VERSION.to_string(),
        id: "src-1".into(),
        principal_id: None,
        scope: None,
    }));
}

#[test]
fn daemon_request__propose_conclusion__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ProposeConclusion(ProposeConclusionRequest {
        api_version: API_VERSION.to_string(),
        principal_id: Some("p1".into()),
        scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
        statement: "x".into(),
        evidence_ids: vec!["e1".into()],
        privacy: None,
        command_id: None,
    }));
}

#[test]
fn daemon_request__propose_decision__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ProposeDecision(ProposeDecisionRequest {
        api_version: API_VERSION.to_string(),
        principal_id: None,
        scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
        title: Some("ADR".into()),
        statement: "use SQLite".into(),
        conclusion_ids: vec![],
        evidence_ids: vec!["e1".into()],
        privacy: Some("LocalOnly".into()),
        command_id: None,
    }));
}

#[test]
fn daemon_request__list_review_items__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ListReviewItems(
        ListReviewItemsRequest::default(),
    ));
}

#[test]
fn daemon_request__resolve_review_item__roundtrip() {
    assert_roundtrip_request(DaemonRequest::ResolveReviewItem(ResolveReviewItemRequest {
        api_version: API_VERSION.to_string(),
        id: "r1".into(),
        resolution: "dismissed".into(),
        principal_id: None,
        note: None,
        scope: None,
        command_id: None,
    }));
}

#[test]
fn daemon_request__request_erasure__roundtrip() {
    assert_roundtrip_request(DaemonRequest::RequestErasure(RequestErasureRequest {
        api_version: API_VERSION.to_string(),
        principal_id: None,
        ids: vec!["agg-1".into()],
        reason: Some("user request".into()),
        scope: None,
        command_id: None,
    }));
}

// ---------------------------------------------------------------------------
// AC2 / AC4 / AC7 — responses
// ---------------------------------------------------------------------------

#[test]
fn daemon_response__scope_resolved__roundtrip() {
    let raw = include_str!("fixtures/scope_resolved_response.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("scope_resolved fixture");
    match &decoded {
        DaemonResponse::ScopeResolved(s) => {
            assert!(!s.authoritative);
            assert_eq!(s.confidence, "Ambiguous");
            assert_eq!(s.alternatives.len(), 2);
            assert_eq!(s.warnings.len(), 1);
            assert_eq!(s.evidence.len(), 1);
        }
        other => panic!("expected ScopeResolved, got {other:?}"),
    }
    assert_roundtrip_response(decoded);
}

#[test]
fn daemon_response__scope_resolved__e1_empty_arrays() {
    let raw = include_str!("fixtures/scope_resolved_e1_empty.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("e1 fixture");
    match decoded {
        DaemonResponse::ScopeResolved(s) => {
            assert!(!s.authoritative);
            assert!(s.evidence.is_empty());
            assert!(s.warnings.is_empty());
            assert!(s.alternatives.is_empty());
        }
        other => panic!("expected ScopeResolved, got {other:?}"),
    }

    // Fields omitted on wire still deserialize to empty arrays.
    let thin = r#"{
      "type":"scope_resolved",
      "payload":{
        "api_version":"1",
        "scope":"",
        "confidence":"Low",
        "authoritative":false
      }
    }"#;
    let thin_decoded: DaemonResponse = serde_json::from_str(thin).expect("thin e1");
    match thin_decoded {
        DaemonResponse::ScopeResolved(s) => {
            assert!(s.evidence.is_empty());
            assert!(s.warnings.is_empty());
            assert!(s.alternatives.is_empty());
        }
        other => panic!("expected ScopeResolved, got {other:?}"),
    }
}

#[test]
fn daemon_response__scope_resolved__authoritative_false() {
    let resp = DaemonResponse::ScopeResolved(ScopeResolvedResponse {
        api_version: API_VERSION.to_string(),
        scope: String::new(),
        confidence: "Low".into(),
        authoritative: false,
        evidence: vec![ScopeEvidenceDto {
            signal: "cwd".into(),
            detail: "heuristic".into(),
        }],
        warnings: vec!["not authoritative".into()],
        alternatives: vec![],
    });
    assert_roundtrip_response(resp);
}

#[test]
fn daemon_response__query_empty_results__e1() {
    let resp = DaemonResponse::QueryKnowledge(ProgressiveQueryResponse::new(
        vec![],
        "Repository:00000000-0000-0000-0000-0000000000a1",
        "DefaultPolicyEvaluator",
        "trace-1",
        false,
    ));
    let json = serde_json::to_value(&resp).expect("serialize");
    let payload = json.get("payload").expect("payload");
    assert_eq!(payload.get("results").unwrap().as_array().unwrap().len(), 0);
    assert_eq!(payload.get("more_available").unwrap(), false);
    assert_roundtrip_response(resp);
}

#[test]
fn daemon_response__review_list_empty__e1() {
    let resp = DaemonResponse::ReviewList(ReviewQueueResponse::new(vec![]));
    let json = serde_json::to_value(&resp).expect("serialize");
    assert_eq!(
        json.get("payload")
            .unwrap()
            .get("items")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_roundtrip_response(resp);
}

#[test]
fn daemon_response__error_policy_denied__roundtrip() {
    let raw = include_str!("fixtures/policy_denied_error.json");
    let decoded: DaemonResponse = serde_json::from_str(raw).expect("policy denied fixture");
    match &decoded {
        DaemonResponse::Error(err) => {
            assert_eq!(err.code, POLICY_DENIED_CODE);
            assert!(err.details.is_some());
        }
        other => panic!("expected Error, got {other:?}"),
    }
    assert_roundtrip_response(decoded);

    let denial = PolicyDenial::new("denied");
    let resp = DaemonResponse::Error(denial.to_api_error());
    match resp {
        DaemonResponse::Error(err) => assert_eq!(err.code, "POLICY_DENIED"),
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn daemon_response__unsupported_operation_helper() {
    let resp = DaemonResponse::unsupported("resolve_scope");
    match resp {
        DaemonResponse::Error(err) => {
            assert_eq!(err.code, UNSUPPORTED_OPERATION);
            assert!(err.message.contains("resolve_scope"));
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn daemon_response__thin_results__roundtrip() {
    assert_roundtrip_response(DaemonResponse::ConclusionProposed(
        ConclusionProposedResponse::new("c1", "proposed"),
    ));
    assert_roundtrip_response(DaemonResponse::DecisionProposed(
        DecisionProposedResponse::new("d1", "proposed"),
    ));
    assert_roundtrip_response(DaemonResponse::ReviewResolved(ReviewResolvedResponse::new(
        "r1",
        "dismissed",
    )));
    assert_roundtrip_response(DaemonResponse::ErasureAccepted(
        ErasureAcceptedResponse::new("erase-1", "accepted"),
    ));
    assert_roundtrip_response(DaemonResponse::Sync { success: true });
}

#[test]
fn daemon_response__project_briefing__roundtrip() {
    let packet = ProjectBriefingPacket::empty_denied(
        "brief-proj-1".into(),
        BriefingScopeDto {
            scope_key: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
            confidence: "Low".into(),
            warnings: vec![],
            alternatives: vec![],
            authoritative: false,
        },
        "policy denied",
    );
    assert_roundtrip_response(DaemonResponse::ProjectBriefing(
        ProjectBriefingResponse::new(packet),
    ));
}

#[test]
fn daemon_response__personal_briefing__roundtrip() {
    let packet = PersonalContinuityBriefingPacket::empty_denied(
        "brief-pers-1".into(),
        "Personal:00000000-0000-0000-0000-0000000000u1",
        "policy denied",
    );
    assert_roundtrip_response(DaemonResponse::PersonalBriefing(
        PersonalBriefingResponse::new(packet),
    ));
}

#[test]
fn daemon_response__evidence_preview__roundtrip() {
    assert_roundtrip_response(DaemonResponse::EvidencePreview(HandlePreviewDto {
        api_version: API_VERSION.to_string(),
        handle_id: "e1".into(),
        kind: "Evidence".into(),
        preview: "snippet".into(),
        truncated: false,
        source_version_id: None,
    }));
}

#[test]
fn daemon_response__source__roundtrip() {
    assert_roundtrip_response(DaemonResponse::Source(SourceDto {
        id: "src-1".into(),
        kind: "File".into(),
        display_name: "README.md".into(),
        locator: Some("C:/dev/AI-Brains/README.md".into()),
        last_observed_at: None,
    }));
}

#[test]
fn daemon_request__ping_serialize_tag() {
    let json = serde_json::to_value(&DaemonRequest::Ping).expect("ser");
    assert_eq!(json.get("type").and_then(|v| v.as_str()), Some("ping"));
}

#[test]
fn daemon_request__resolve_scope_serialize_tag() {
    let json = serde_json::to_value(DaemonRequest::ResolveScope(ResolveScopeRequest::default()))
        .expect("ser");
    assert_eq!(
        json.get("type").and_then(|v| v.as_str()),
        Some("resolve_scope")
    );
}
