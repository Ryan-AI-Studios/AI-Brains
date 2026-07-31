#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{
    ConclusionId, DeviceId, EvidenceId, MemoryId, PrincipalId, ProjectId, ReplicationEventId,
    SessionId, SourceId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_core::source::SourceKind;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionProposedPayload, DeviceEnrolledPayload, DeviceRevokedPayload, MemoryPinnedPayload,
    PolicyDecisionRecordedPayload, SessionStartedPayload, SourceRegisteredPayload,
};
use ai_brains_events::{Actor, AggregateType, EventKind, Payload};
use serde_json::json;
use uuid::Uuid;

#[test]
fn event_kind_from_payload__conclusion_proposed__matches_kind() {
    let payload = Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id: ConclusionId::new(),
        statement: "claim".into(),
        evidence_ids: vec![],
        proposer: PrincipalId::new(),
        valid_from: None,
        valid_until: None,
        scope: String::new(),
        protected_category: None,
        unsupported: true,
        model_provenance: None,
    });
    assert_eq!(EventKind::from(&payload), EventKind::ConclusionProposed);
}

#[test]
fn event_kind_from_payload__several_known_variants__match() {
    let cases: Vec<(Payload, EventKind)> = vec![
        (
            Payload::SessionStarted(SessionStartedPayload {
                session_id: SessionId::new(),
                project_id: ProjectId::new(),
                tx_id: None,
            }),
            EventKind::SessionStarted,
        ),
        (
            Payload::MemoryPinned(MemoryPinnedPayload {
                memory_id: MemoryId::new(),
                content: "x".into(),
                session_id: None,
                project_id: None,
                tx_id: None,
                rank: None,
                source_tag: None,
                query_text: None,
            }),
            EventKind::MemoryPinned,
        ),
        (
            Payload::SourceRegistered(SourceRegisteredPayload {
                source_id: SourceId::new(),
                kind: SourceKind::File,
                display_name: "f".into(),
                locator: None,
                scope: None,
            }),
            EventKind::SourceRegistered,
        ),
        (
            Payload::PolicyDecisionRecorded(PolicyDecisionRecordedPayload {
                principal_id: PrincipalId::new(),
                capability: GrantCapability::ReadEvidence,
                scope_key: "Repository:x".into(),
                allowed: false,
                reason_code: "missing_grant".into(),
                privacy: Some(Privacy::LocalOnly),
            }),
            EventKind::PolicyDecisionRecorded,
        ),
        (
            Payload::DeviceEnrolled(DeviceEnrolledPayload {
                device_id: DeviceId::new(),
                enrolled_by_device_id: DeviceId::new(),
                status: "local".into(),
                fingerprint_sha256: "aa".repeat(32),
                ed25519_public: "bb".repeat(32),
                x25519_public: "cc".repeat(32),
                schema_version: 1,
                replication_event_id: ReplicationEventId::new(),
                local_seq: 1,
                envelope_id: Uuid::new_v4(),
                signature_hex: "dd".repeat(64),
                body_hex: "ee".repeat(8),
                content_type_code: 0x0010,
            }),
            EventKind::DeviceEnrolled,
        ),
        (
            Payload::DeviceRevoked(DeviceRevokedPayload {
                device_id: DeviceId::new(),
                revoked_by_device_id: DeviceId::new(),
                reason_code: "cli-revoke".into(),
                replication_event_id: ReplicationEventId::new(),
                local_seq: 2,
                envelope_id: Uuid::new_v4(),
                signature_hex: "ff".repeat(64),
                body_hex: "11".repeat(8),
                content_type_code: 0x0011,
            }),
            EventKind::DeviceRevoked,
        ),
    ];
    for (payload, expected) in cases {
        assert_eq!(EventKind::from(&payload), expected);
    }
}

#[test]
fn event_kind_from_payload__unknown_json_with_type_tag__extracts_tag() {
    let payload = Payload::Unknown(json!({
        "type": "FutureThing",
        "extra": 1
    }));
    assert_eq!(
        EventKind::from(&payload),
        EventKind::Unknown("FutureThing".to_string())
    );
}

#[test]
fn event_builder__build__derives_kind_from_payload_not_constructor() {
    // Builder no longer accepts EventKind; kind always matches payload.
    let conclusion_id = ConclusionId::new();
    let envelope = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id,
        statement: "s".into(),
        evidence_ids: vec![EvidenceId::new()],
        proposer: PrincipalId::new(),
        valid_from: None,
        valid_until: None,
        scope: "Personal:x".into(),
        protected_category: None,
        unsupported: false,
        model_provenance: None,
    }))
    .expect("build");

    assert_eq!(envelope.event_type, EventKind::ConclusionProposed);
    assert!(matches!(envelope.payload, Payload::ConclusionProposed(_)));
    // Mismatched pairs are unrepresentable: event_type is always derived.
    assert_eq!(envelope.event_type, EventKind::from(&envelope.payload));
    // aggregate remains explicit (not always 1:1 with payload)
    assert_eq!(envelope.aggregate_type, AggregateType::Conclusion);
    let _ = Uuid::nil(); // keep uuid import used if needed by future cases
}
