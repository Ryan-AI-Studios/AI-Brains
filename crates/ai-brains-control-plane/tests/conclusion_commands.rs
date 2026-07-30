#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, GovernedQueryStore, ProposeConclusionRequest, StorePorts,
    SystemClock, activate_conclusion, confirm_conclusion, correct_conclusion, make_principal,
    propose_conclusion, reject_conclusion, scope_identity_key,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::protected_category::ProtectedCategory;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use tempfile::NamedTempFile;
use time::OffsetDateTime;

fn open_ports() -> (NamedTempFile, StorePorts) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);
    (temp_file, StorePorts::from_store(store))
}

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent")
}

fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

#[test]
fn propose_conclusion__with_evidence__candidate_supported() {
    let (_t, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::new());
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: "Rust is great".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    assert!(!res.unsupported);
    let row = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .expect("row");
    assert_eq!(row.state, "Candidate");
    assert!(!row.unsupported);
    assert_eq!(row.statement, "Rust is great");
    assert_eq!(row.scope, scope_identity_key(&scope));
}

#[test]
fn propose_conclusion__without_evidence__unsupported_cannot_confirm() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "unsupported claim".into(),
            evidence_ids: vec![],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    assert!(res.unsupported);
    let row = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert!(row.unsupported);
    let err = confirm_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ControlPlaneError::UnsupportedCannotConfirm(_)
    ));
}

#[test]
fn propose_conclusion__preassigned_id_second_call__no_second_event() {
    use ai_brains_core::ids::ConclusionId;
    use ai_brains_events::Payload;
    use ai_brains_store::event_store::EventStore;

    let (_t, ports) = open_ports();
    let conclusion_id = ConclusionId::new();
    let req = ProposeConclusionRequest {
        principal: agent(),
        scope: ScopeRef::Personal(UserId::new()),
        statement: "idempotent claim".into(),
        evidence_ids: vec![EvidenceId::new()],
        privacy: Privacy::LocalOnly,
        valid_from: None,
        valid_until: None,
        protected_category: None,
        conclusion_id: Some(conclusion_id),
    };
    let first = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        req.clone(),
    )
    .unwrap();
    assert_eq!(first.conclusion_id, conclusion_id);
    assert!(!first.unsupported);

    // Second call with grant still succeeds (no second append).
    let second = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        req,
    )
    .unwrap();
    assert_eq!(second.conclusion_id, conclusion_id);
    assert_eq!(second.unsupported, first.unsupported);

    let events = ports.writer.store().read_all_events().unwrap();
    let proposed_count = events
        .iter()
        .filter(|e| {
            matches!(
                &e.payload,
                Payload::ConclusionProposed(p) if p.conclusion_id == conclusion_id
            )
        })
        .count();
    assert_eq!(
        proposed_count, 1,
        "second call with pre-assigned id must not append another ConclusionProposed"
    );
}

#[test]
fn propose_conclusion__preassigned_id_second_call_without_grant__policy_denied() {
    use ai_brains_control_plane::DenyAllPolicy;
    use ai_brains_core::ids::ConclusionId;
    use ai_brains_events::Payload;
    use ai_brains_store::event_store::EventStore;

    let (_t, ports) = open_ports();
    let conclusion_id = ConclusionId::new();
    let req = ProposeConclusionRequest {
        principal: agent(),
        scope: ScopeRef::Personal(UserId::new()),
        statement: "idempotent claim".into(),
        evidence_ids: vec![EvidenceId::new()],
        privacy: Privacy::LocalOnly,
        valid_from: None,
        valid_until: None,
        protected_category: None,
        conclusion_id: Some(conclusion_id),
    };
    propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        req.clone(),
    )
    .unwrap();

    // Replay without grant must not short-circuit past policy.
    let err = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &DenyAllPolicy,
        req,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::PolicyDenied(_)),
        "expected PolicyDenied on second propose without grant, got {err:?}"
    );

    let events = ports.writer.store().read_all_events().unwrap();
    let proposed_count = events
        .iter()
        .filter(|e| {
            matches!(
                &e.payload,
                Payload::ConclusionProposed(p) if p.conclusion_id == conclusion_id
            )
        })
        .count();
    assert_eq!(proposed_count, 1, "deny path must not append");
}

#[test]
fn propose_conclusion__explicit_valid_window__stored_distinct_from_now() {
    let (_t, ports) = open_ports();
    let valid_from = OffsetDateTime::from_unix_timestamp(1_500_000_000).unwrap();
    let valid_until = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Repository(ProjectId::new()),
            statement: "historical".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(valid_from),
            valid_until: Some(valid_until),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let row = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.valid_from, valid_from);
    assert_eq!(row.valid_until, Some(valid_until));
}

#[test]
fn activate_conclusion__non_protected__agent_ok() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "activate me".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let row = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "Active");
}

#[test]
fn confirm_conclusion__protected_without_human__fails() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "delete all data".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: Some(ProtectedCategory::Deletion),
            conclusion_id: None,
        },
    )
    .unwrap();
    let err = confirm_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::ApprovalRequired(_)));
}

#[test]
fn confirm_conclusion__protected_with_human__ok() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "security policy".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: Some(ProtectedCategory::SecurityPolicy),
            conclusion_id: None,
        },
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let row = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "Confirmed");
}

#[test]
fn reject_and_correct_conclusion__supersession_batch() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "old claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    reject_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.conclusion_id,
        "wrong",
        Privacy::LocalOnly,
    )
    .unwrap();
    let rejected = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(rejected.state, "Rejected");

    let res2 = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "to correct".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    // Candidate cannot go directly to Superseded — activate first.
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res2.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let new_id = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res2.conclusion_id,
        "corrected claim".into(),
        vec![EvidenceId::new()],
        "better evidence",
        Privacy::LocalOnly,
        None,
    )
    .unwrap();
    assert_ne!(new_id, res2.conclusion_id);
    let old = ports
        .query
        .get_conclusion(res2.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(old.state, "Superseded");
    assert_eq!(
        old.superseded_by.as_deref(),
        Some(new_id.to_string().as_str())
    );
    let new_row = ports.query.get_conclusion(new_id).unwrap().unwrap();
    assert_eq!(new_row.statement, "corrected claim");
    assert_eq!(new_row.state, "Candidate");
}

#[test]
fn correct_conclusion__from_candidate__fails_invalid_transition() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "still candidate".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let err = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.conclusion_id,
        "should not correct".into(),
        vec![EvidenceId::new()],
        "attempt from candidate",
        Privacy::LocalOnly,
        None,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::InvalidTransition(_)),
        "expected InvalidTransition, got {err:?}"
    );
}

#[test]
fn correct_conclusion__policy_deny__fails() {
    use ai_brains_control_plane::DenyAllPolicy;
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "active then deny".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let err = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &DenyAllPolicy,
        &human(),
        res.conclusion_id,
        "denied correction".into(),
        vec![EvidenceId::new()],
        "policy blocks propose",
        Privacy::LocalOnly,
        None,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::PolicyDenied(_)),
        "expected PolicyDenied, got {err:?}"
    );
}

#[test]
fn correct_conclusion__from_active__succeeds() {
    let (_t, ports) = open_ports();
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "active claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let new_id = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.conclusion_id,
        "corrected active".into(),
        vec![EvidenceId::new()],
        "fix after activate",
        Privacy::LocalOnly,
        None,
    )
    .unwrap();
    let old = ports
        .query
        .get_conclusion(res.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(old.state, "Superseded");
    let new_row = ports.query.get_conclusion(new_id).unwrap().unwrap();
    assert_eq!(new_row.statement, "corrected active");
    assert_eq!(new_row.state, "Candidate");
}

#[test]
fn propose_conclusion__valid_until_not_after_from__rejected() {
    let (_t, ports) = open_ports();
    let t0 = OffsetDateTime::from_unix_timestamp(1_500_000_000).unwrap();
    let err = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "invalid window".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(t0),
            valid_until: Some(t0),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));

    let earlier = OffsetDateTime::from_unix_timestamp(1_400_000_000).unwrap();
    let err2 = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "inverted window".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(t0),
            valid_until: Some(earlier),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err2, ControlPlaneError::InvalidPayload(_)));
}
