#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, SourceId, SourceVersionId,
};
use ai_brains_core::model_provenance::ModelProvenance;
use ai_brains_core::source::SourceKind;
use ai_brains_events::{
    ConclusionProposedPayload, DecisionApprovedPayload, DecisionProposedPayload,
    EvidenceRecordedPayload, Payload, SourceRegisteredPayload,
};
use time::OffsetDateTime;
use uuid::Uuid;

fn roundtrip(payload: Payload) -> Payload {
    let v = serde_json::to_value(&payload).expect("serialize");
    serde_json::from_value(v).expect("deserialize")
}

#[test]
fn source_registered__roundtrip() {
    let p = Payload::SourceRegistered(SourceRegisteredPayload {
        source_id: SourceId::from_uuid(Uuid::from_u128(1)),
        kind: SourceKind::GitRepository,
        display_name: "repo".to_string(),
        locator: Some("https://example.com/r.git".to_string()),
        scope: Some("Personal:00000000-0000-0000-0000-000000000009".to_string()),
    });
    assert_eq!(roundtrip(p.clone()), p);
}

#[test]
fn evidence_recorded__with_model_provenance__roundtrip() {
    let p = Payload::EvidenceRecorded(EvidenceRecordedPayload {
        evidence_id: EvidenceId::from_uuid(Uuid::from_u128(2)),
        source_id: SourceId::from_uuid(Uuid::from_u128(1)),
        source_version_id: Some(SourceVersionId::from_uuid(Uuid::from_u128(3))),
        fingerprint: Some("abc".to_string()),
        model_provenance: Some(ModelProvenance {
            provider: "ollama".to_string(),
            model: "qwen".to_string(),
            model_version: Some("3.5".to_string()),
            workflow_version: None,
            deployment: None,
            endpoint_class: None,
            usage: None,
            template_id: None,
            input_ids: None,
            output_hash: None,
            started_at: None,
            completed_at: None,
        }),
        summary: "snippet".to_string(),
    });
    assert_eq!(roundtrip(p.clone()), p);
}

#[test]
fn conclusion_proposed__roundtrip() {
    let p = Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(4)),
        statement: "X is true".to_string(),
        evidence_ids: vec![EvidenceId::from_uuid(Uuid::from_u128(2))],
        proposer: PrincipalId::from_uuid(Uuid::from_u128(9)),
        valid_from: None,
        valid_until: None,
        scope: String::new(),
        protected_category: None,
        unsupported: false,
        model_provenance: None,
    });
    assert_eq!(roundtrip(p.clone()), p);
}

#[test]
fn remaining_governed_payloads__roundtrip_subset() {
    use ai_brains_core::ids::{
        BriefingId, ContentKeyId, GrantId, ProjectId, QueryTraceId, ReviewItemId, TombstoneId,
        WorkspaceId,
    };
    use ai_brains_core::privacy::Privacy;
    use ai_brains_core::scope::{GrantCapability, ScopeRef};
    use ai_brains_events::*;
    use time::OffsetDateTime;

    let ts = OffsetDateTime::from_unix_timestamp(1_700_000_200).expect("ts");
    let cases = vec![
        Payload::SourceObserved(SourceObservedPayload {
            source_id: SourceId::from_uuid(Uuid::from_u128(1)),
            observed_at: ts,
            note: Some("ok".into()),
        }),
        Payload::SourceVersionRecorded(SourceVersionRecordedPayload {
            source_id: SourceId::from_uuid(Uuid::from_u128(1)),
            version_id: SourceVersionId::from_uuid(Uuid::from_u128(2)),
            fingerprint: "fp".into(),
            recorded_at: ts,
        }),
        Payload::SourceUnavailable(SourceUnavailablePayload {
            source_id: SourceId::from_uuid(Uuid::from_u128(1)),
            reason: "offline".into(),
            marked_at: ts,
        }),
        Payload::EvidenceSuperseded(EvidenceSupersededPayload {
            evidence_id: EvidenceId::from_uuid(Uuid::from_u128(3)),
            superseded_by: EvidenceId::from_uuid(Uuid::from_u128(4)),
            reason: "newer".into(),
        }),
        Payload::ConclusionActivated(ConclusionActivatedPayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
        }),
        Payload::ConclusionConfirmed(ConclusionConfirmedPayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
            approver: PrincipalId::from_uuid(Uuid::from_u128(9)),
            confirmed_at: ts,
        }),
        Payload::ConclusionMarkedStale(ConclusionMarkedStalePayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
            changed_source_version_id: Some(SourceVersionId::from_uuid(Uuid::from_u128(2))),
            unavailable_reason: None,
            source_id: Some(SourceId::from_uuid(Uuid::from_u128(1))),
        }),
        Payload::ConclusionDisputed(ConclusionDisputedPayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
            disputant: PrincipalId::from_uuid(Uuid::from_u128(9)),
            reason: "conflict".into(),
        }),
        Payload::ConclusionSuperseded(ConclusionSupersededPayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
            superseded_by: ConclusionId::from_uuid(Uuid::from_u128(6)),
            reason: "better".into(),
        }),
        Payload::ConclusionRejected(ConclusionRejectedPayload {
            conclusion_id: ConclusionId::from_uuid(Uuid::from_u128(5)),
            rejector: PrincipalId::from_uuid(Uuid::from_u128(9)),
            reason: "no".into(),
        }),
        Payload::DecisionSuperseded(DecisionSupersededPayload {
            decision_id: DecisionId::from_uuid(Uuid::from_u128(7)),
            superseded_by: DecisionId::from_uuid(Uuid::from_u128(8)),
            reason: "supersede".into(),
        }),
        Payload::DecisionRevoked(DecisionRevokedPayload {
            decision_id: DecisionId::from_uuid(Uuid::from_u128(7)),
            revoker: PrincipalId::from_uuid(Uuid::from_u128(9)),
            reason: "revoke".into(),
        }),
        Payload::WorkspaceRegistered(WorkspaceRegisteredPayload {
            workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(11)),
            name: "ws".into(),
        }),
        Payload::RepositoryJoinedWorkspace(RepositoryJoinedWorkspacePayload {
            workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(11)),
            project_id: ProjectId::from_uuid(Uuid::from_u128(12)),
        }),
        Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
            grant_id: GrantId::from_uuid(Uuid::from_u128(13)),
            principal_id: PrincipalId::from_uuid(Uuid::from_u128(9)),
            scope: ScopeRef::Repository(ProjectId::from_uuid(Uuid::from_u128(12))),
            capability: GrantCapability::ReadEvidence,
            privacy: Privacy::LocalOnly,
        }),
        Payload::ScopeGrantRevoked(ScopeGrantRevokedPayload {
            grant_id: GrantId::from_uuid(Uuid::from_u128(13)),
            reason: "expired".into(),
        }),
        Payload::PolicyDecisionRecorded(ai_brains_events::PolicyDecisionRecordedPayload {
            principal_id: PrincipalId::from_uuid(Uuid::from_u128(9)),
            capability: GrantCapability::ProposeConclusion,
            scope_key: format!("Repository:{}", ProjectId::from_uuid(Uuid::from_u128(12))),
            allowed: false,
            reason_code: "missing_grant".into(),
            privacy: Some(Privacy::LocalOnly),
        }),
        Payload::PrincipalRegistered(PrincipalRegisteredPayload {
            principal_id: PrincipalId::from_uuid(Uuid::from_u128(9)),
            kind: "Human".into(),
            display_name: "Ryan".into(),
            bound_source_kinds: Vec::new(),
            bound_capabilities: Vec::new(),
        }),
        Payload::ReviewItemOpened(ReviewItemOpenedPayload {
            review_item_id: ReviewItemId::from_uuid(Uuid::from_u128(14)),
            subject: "review".into(),
            opened_by: PrincipalId::from_uuid(Uuid::from_u128(9)),
            subject_kind: ai_brains_core::review::ReviewSubjectKind::Decision,
            subject_id: DecisionId::from_uuid(Uuid::from_u128(7)).to_string(),
            criticality: ai_brains_core::review::ReviewCriticality::High,
            related_conclusion_id: None,
            related_decision_id: Some(DecisionId::from_uuid(Uuid::from_u128(7))),
            related_source_id: Some(SourceId::from_uuid(Uuid::from_u128(1))),
        }),
        Payload::ReviewItemResolved(ReviewItemResolvedPayload {
            review_item_id: ReviewItemId::from_uuid(Uuid::from_u128(14)),
            resolution: "done".into(),
            resolved_by: PrincipalId::from_uuid(Uuid::from_u128(9)),
        }),
        Payload::BriefingGenerated(BriefingGeneratedPayload {
            briefing_id: BriefingId::from_uuid(Uuid::from_u128(15)),
            kind: "Preflight".into(),
            evidence_ids: vec![EvidenceId::from_uuid(Uuid::from_u128(3))],
            query_trace_id: Some(QueryTraceId::from_uuid(Uuid::from_u128(16))),
        }),
        Payload::QueryTraceRecorded(QueryTraceRecordedPayload {
            query_trace_id: QueryTraceId::from_uuid(Uuid::from_u128(16)),
            query_text: "q".into(),
            evidence_ids: vec![EvidenceId::from_uuid(Uuid::from_u128(3))],
            scope: "Repository:00000000-0000-0000-0000-000000000001".into(),
            principal_id: PrincipalId::from_uuid(Uuid::from_u128(9)).to_string(),
            applied_policy: "DefaultPolicyEvaluator".into(),
            ranking_json: r#"{"order":["policy"]}"#.into(),
            freshness_summary: Some("fresh=1".into()),
            conflict_summary: None,
        }),
        Payload::ContentErasureRequested(ContentErasureRequestedPayload {
            content_key_id: ContentKeyId::from_uuid(Uuid::from_u128(17)),
            requester: PrincipalId::from_uuid(Uuid::from_u128(9)),
            reason: "gdpr".into(),
        }),
        Payload::ContentErased(ContentErasedPayload {
            content_key_id: ContentKeyId::from_uuid(Uuid::from_u128(17)),
            tombstone_id: TombstoneId::from_uuid(Uuid::from_u128(18)),
        }),
        Payload::ErasureTicketAccepted(ErasureTicketAcceptedPayload {
            request_id: "00000000-0000-0000-0000-000000000019".into(),
            requester: PrincipalId::from_uuid(Uuid::from_u128(9)),
            target_ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: Some("Personal:00000000-0000-0000-0000-0000000000u1".into()),
        }),
    ];
    for p in cases {
        assert_eq!(roundtrip(p.clone()), p);
    }
}

#[test]
fn decision_proposed_and_approved__locked_shape__roundtrip() {
    let decision_id = DecisionId::from_uuid(Uuid::from_u128(5));
    let proposal = Payload::DecisionProposed(DecisionProposedPayload {
        decision_id,
        title: "Use ports".to_string(),
        statement: "Control plane uses ports-only in P1".to_string(),
        proposer: PrincipalId::from_uuid(Uuid::from_u128(9)),
        conclusion_ids: Some(vec![ConclusionId::from_uuid(Uuid::from_u128(4))]),
        evidence_ids: None,
        valid_from: None,
        valid_until: None,
        scope: String::new(),
    });
    assert_eq!(roundtrip(proposal.clone()), proposal);

    let proposal_event_id = Uuid::from_u128(100);
    let approved_at = OffsetDateTime::from_unix_timestamp(1_700_000_100).expect("ts");
    let approved = Payload::DecisionApproved(DecisionApprovedPayload {
        decision_id,
        proposal_event_id,
        approver: PrincipalId::from_uuid(Uuid::from_u128(10)),
        approved_at,
    });
    let back = roundtrip(approved.clone());
    assert_eq!(back, approved);
    match back {
        Payload::DecisionApproved(p) => {
            assert_eq!(p.decision_id, decision_id);
            assert_eq!(p.proposal_event_id, proposal_event_id);
            assert_eq!(p.approver, PrincipalId::from_uuid(Uuid::from_u128(10)));
            assert_eq!(p.approved_at, approved_at);
        }
        other => panic!("expected DecisionApproved, got {other:?}"),
    }
}

#[test]
fn PrincipalRegistered__old_json_without_bindings__deserializes_empty_defaults() {
    use ai_brains_events::PrincipalRegisteredPayload;

    let principal_id = PrincipalId::from_uuid(Uuid::from_u128(9));
    let json = serde_json::json!({
        "principal_id": principal_id,
        "kind": "Human",
        "display_name": "Ryan"
    });
    let payload: PrincipalRegisteredPayload = serde_json::from_value(json)
        .expect("old PrincipalRegistered JSON deserializes with empty binding defaults");
    assert_eq!(payload.principal_id, principal_id);
    assert_eq!(payload.kind, "Human");
    assert_eq!(payload.display_name, "Ryan");
    assert!(payload.bound_source_kinds.is_empty());
    assert!(payload.bound_capabilities.is_empty());
}

#[test]
fn ScopeGrantIssued__old_json_without_privacy__defaults_local_only() {
    use ai_brains_core::ids::{GrantId, PrincipalId as Pid, ProjectId};
    use ai_brains_core::privacy::Privacy;
    use ai_brains_core::scope::{GrantCapability, ScopeRef};
    use ai_brains_events::ScopeGrantIssuedPayload;

    let grant_id = GrantId::from_uuid(Uuid::from_u128(13));
    let principal_id = Pid::from_uuid(Uuid::from_u128(9));
    let project_id = ProjectId::from_uuid(Uuid::from_u128(12));
    let json = serde_json::json!({
        "grant_id": grant_id,
        "principal_id": principal_id,
        "scope": { "Repository": project_id },
        "capability": "ReadEvidence"
    });
    let payload: ScopeGrantIssuedPayload = serde_json::from_value(json)
        .expect("old ScopeGrantIssued JSON deserializes with LocalOnly privacy default");
    assert_eq!(payload.privacy, Privacy::LocalOnly);
    assert_eq!(payload.capability, GrantCapability::ReadEvidence);
    assert_eq!(payload.scope, ScopeRef::Repository(project_id));
}
