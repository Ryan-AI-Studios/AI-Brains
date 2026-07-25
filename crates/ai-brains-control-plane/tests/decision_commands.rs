#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, GovernedQueryStore, ProposeConclusionRequest,
    ProposeDecisionRequest, StorePorts, SystemClock, approve_decision, make_principal,
    propose_conclusion, propose_decision, revoke_decision, supersede_decision,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
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
    (
        temp_file,
        StorePorts::from_store(SqliteEventStore::new(conn)),
    )
}

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent")
}
fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

#[test]
fn propose_decision__yields_proposed_state() {
    let (_t, ports) = open_ports();
    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "support".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "Ship".into(),
            statement: "we ship".into(),
            conclusion_ids: Some(vec![conc.conclusion_id]),
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    let row = ports.query.get_decision(res.decision_id).unwrap().unwrap();
    assert_eq!(row.state, "Proposed");
    assert_eq!(row.title, "Ship");
    assert!(row.proposal_event_id.is_some());
}

#[test]
fn approve_decision__agent_rejected_human_ok() {
    let (_t, ports) = open_ports();
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "T".into(),
            statement: "S".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    let err = approve_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &agent(),
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::ApprovalRequired(_)));

    approve_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let row = ports.query.get_decision(res.decision_id).unwrap().unwrap();
    assert_eq!(row.state, "Approved");
    assert!(row.approver.is_some());
}

#[test]
fn supersede_and_revoke_decision__state_updates() {
    let (_t, ports) = open_ports();
    let first = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: human(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "v1".into(),
            statement: "old".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    approve_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        first.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let second = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: human(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "v2".into(),
            statement: "new".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        first.decision_id,
        second.decision_id,
        "replaced",
        Privacy::LocalOnly,
    )
    .unwrap();
    let old = ports
        .query
        .get_decision(first.decision_id)
        .unwrap()
        .unwrap();
    assert_eq!(old.state, "Superseded");
    assert_eq!(
        old.superseded_by.as_deref(),
        Some(second.decision_id.to_string().as_str())
    );

    let third = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: human(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "v3".into(),
            statement: "rev".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    revoke_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        third.decision_id,
        "no longer needed",
        Privacy::LocalOnly,
    )
    .unwrap();
    let rev = ports
        .query
        .get_decision(third.decision_id)
        .unwrap()
        .unwrap();
    assert_eq!(rev.state, "Revoked");
}

#[test]
fn propose_decision__with_evidence_ids__materializes_support_rows() {
    let (_t, ports) = open_ports();
    let evidence_id = EvidenceId::new();
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "Evidence-backed".into(),
            statement: "supported by evidence".into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![evidence_id]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();

    let conn = ports.query.store().connection().lock().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM decision_support_projection
             WHERE decision_id = ? AND evidence_id = ? AND conclusion_id = ''",
            rusqlite::params![res.decision_id.to_string(), evidence_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "evidence link must materialize in decision_support"
    );
}

#[test]
fn approve_decision__missing_proposal_event_id__fails_closed() {
    let (_t, ports) = open_ports();
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "T".into(),
            statement: "S".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();

    // Corrupt proposal linkage so approve must fail closed (no nil substitution).
    {
        let conn = ports.query.store().connection().lock().unwrap();
        conn.execute(
            "UPDATE decision_projection SET proposal_event_id = NULL WHERE decision_id = ?",
            [res.decision_id.to_string()],
        )
        .unwrap();
    }

    let err = approve_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            ControlPlaneError::NotFound(_) | ControlPlaneError::InvalidPayload(_)
        ),
        "expected fail-closed NotFound/InvalidPayload, got {err:?}"
    );
}

#[test]
fn approve_decision__malformed_proposal_event_id__fails_closed() {
    let (_t, ports) = open_ports();
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "T".into(),
            statement: "S".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();

    {
        let conn = ports.query.store().connection().lock().unwrap();
        conn.execute(
            "UPDATE decision_projection SET proposal_event_id = 'not-a-uuid' WHERE decision_id = ?",
            [res.decision_id.to_string()],
        )
        .unwrap();
    }

    let err = approve_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::InvalidPayload(_)),
        "expected InvalidPayload for unparsable proposal_event_id, got {err:?}"
    );
}

#[test]
fn propose_decision__valid_until_not_after_from__rejected() {
    let (_t, ports) = open_ports();
    let t0 = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let err = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            title: "T".into(),
            statement: "S".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: Some(t0),
            valid_until: Some(t0),
            decision_id: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
}

/// R1-F1: Human without ApproveDecision grant is denied by DefaultPolicyEvaluator.
#[test]
fn approve_decision__human_without_grant__policy_denied() {
    use ai_brains_control_plane::{issue_grant, register_principal};
    use ai_brains_core::scope::GrantCapability;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let scope = ScopeRef::Personal(UserId::new());
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();

    // Propose via AllowAll (agent not registered is fine for AllowAll).
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: agent(),
            scope: scope.clone(),
            title: "T".into(),
            statement: "S".into(),
            conclusion_ids: None,
            evidence_ids: None,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();

    let policy = ports.policy_evaluator();
    let err = approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::PolicyDenied(_)),
        "expected PolicyDenied without grant, got {err:?}"
    );

    // With grant → allow.
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        scope,
        GrantCapability::ApproveDecision,
        Privacy::LocalOnly,
    )
    .unwrap();
    let policy2 = ports.policy_evaluator();
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy2,
        &human_p,
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let row = ports.query.get_decision(res.decision_id).unwrap().unwrap();
    assert_eq!(row.state, "Approved");
}
