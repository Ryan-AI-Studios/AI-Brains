//! T323 AC1–AC7 / AC11–AC13 / AC17 — conclusion in-force resolver.
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, EventWriter, ProposeConclusionRequest, StorePorts,
    SystemClock, activate_conclusion, confirm_conclusion, correct_conclusion, make_principal,
    propose_conclusion, reject_conclusion, resolve_conclusion_in_force, scope_identity_key,
};
use ai_brains_core::ids::{ConclusionId, EvidenceId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::ConclusionSupersededPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use rstest::rstest;
use tempfile::NamedTempFile;

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

fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

fn propose_confirmed(
    ports: &StorePorts,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
    statement: &str,
) -> ConclusionId {
    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: principal.clone(),
            scope,
            statement: statement.into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        principal,
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    res.conclusion_id
}

fn correct_and_confirm(
    ports: &StorePorts,
    principal: &ai_brains_core::principal::Principal,
    old_id: ConclusionId,
    new_statement: &str,
) -> ConclusionId {
    let new_id = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        principal,
        old_id,
        new_statement.into(),
        vec![EvidenceId::new()],
        "replaced",
        Privacy::LocalOnly,
        None,
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        principal,
        new_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    new_id
}

#[test]
fn resolve_conclusion_in_force__superseded_root__current_confirmed_in_force() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let c1 = propose_confirmed(&ports, &principal, scope.clone(), "Term: workspace_id");
    let c2 = correct_and_confirm(&ports, &principal, c1, "Term: workspace_id v2 — corrected");

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    let ruling = resp.ruling.expect("current confirmed must be in force");
    assert_eq!(ruling.conclusion_id, c2.to_string());
    assert_eq!(ruling.state, "in_force");
    assert_eq!(resp.chain.len(), 1);
    assert_eq!(resp.chain[0].conclusion_id, c1.to_string());
    assert_eq!(resp.chain[0].status, format!("superseded_by:{c2}"));
}

#[test]
fn resolve_conclusion_in_force__three_hop_chain__tip_ruling_len2() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let c1 = propose_confirmed(&ports, &principal, scope.clone(), "Term: workspace_id");
    let c2 = correct_and_confirm(&ports, &principal, c1, "Term: workspace_id hop2");
    let c3 = correct_and_confirm(&ports, &principal, c2, "Term: workspace_id hop3");

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    let ruling = resp.ruling.expect("tip confirmed must be in force");
    assert_eq!(ruling.conclusion_id, c3.to_string());
    assert_eq!(ruling.state, "in_force");
    assert_eq!(resp.chain.len(), 2);
    assert_eq!(resp.chain[0].conclusion_id, c1.to_string());
    assert_eq!(resp.chain[0].status, format!("superseded_by:{c2}"));
    assert_eq!(resp.chain[1].conclusion_id, c2.to_string());
    assert_eq!(resp.chain[1].status, format!("superseded_by:{c3}"));
}

#[test]
fn resolve_conclusion_in_force__successor_term__empty_chain() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let c1 = propose_confirmed(&ports, &principal, scope.clone(), "Term: workspace_id");
    let c2 = correct_and_confirm(&ports, &principal, c1, "Term: successor_id");

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "successor_id")
        .unwrap();
    let ruling = resp.ruling.expect("successor is in force");
    assert_eq!(ruling.conclusion_id, c2.to_string());
    assert_eq!(ruling.state, "in_force");
    assert!(
        resp.chain.is_empty(),
        "matching the successor is not a walk"
    );
}

#[test]
fn resolve_conclusion_in_force__rejected_root__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: principal.clone(),
            scope,
            statement: "Term: workspace_id".into(),
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
        &principal,
        res.conclusion_id,
        "nope",
        Privacy::LocalOnly,
    )
    .unwrap();

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
}

#[test]
fn resolve_conclusion_in_force__unknown_term__none() {
    let (_t, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
    assert_eq!(resp.term, "workspace_id");
    assert_eq!(resp.scope, scope_key);
}

#[rstest]
#[case::empty("")]
#[case::whitespace("   ")]
#[case::tab("\t")]
fn resolve_conclusion_in_force__empty_term__err(#[case] term: &str) {
    let (_t, ports) = open_ports();
    let scope_key = "Personal:00000000-0000-0000-0000-000000000001";
    let err = resolve_conclusion_in_force(&ports.query, &SystemClock, scope_key, term)
        .expect_err("empty term must err");
    assert!(
        matches!(err, ControlPlaneError::InvalidPayload(_)),
        "expected InvalidPayload; got {err:?}"
    );
}

#[test]
fn resolve_conclusion_in_force__other_scope_row__not_visible() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope_a = ScopeRef::Personal(UserId::new());
    let scope_b = ScopeRef::Repository(ProjectId::new());
    let key_b = scope_identity_key(&scope_b);

    let _c1 = propose_confirmed(&ports, &principal, scope_a, "Term: workspace_id");

    let resp =
        resolve_conclusion_in_force(&ports.query, &SystemClock, &key_b, "workspace_id").unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
}

#[test]
fn resolve_conclusion_in_force__cycle__error() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let c1 = propose_confirmed(&ports, &principal, scope, "Term: workspace_id");

    let env = EventBuilder::new(
        AggregateType::Conclusion,
        c1.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionSuperseded(ConclusionSupersededPayload {
        conclusion_id: c1,
        superseded_by: c1,
        reason: "self".into(),
    }))
    .unwrap();
    ports.writer.append_events(&[env]).unwrap();

    let err = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .expect_err("self-cycle must err");
    assert!(
        matches!(err, ControlPlaneError::InvalidTransition(_)),
        "expected InvalidTransition; got {err:?}"
    );
}

#[test]
fn resolve_conclusion_in_force__active_only__in_force() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: principal.clone(),
            scope,
            statement: "Term: workspace_id".into(),
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
        &principal,
        res.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    let ruling = resp.ruling.expect("Active tip must be in force");
    assert_eq!(ruling.conclusion_id, res.conclusion_id.to_string());
    assert_eq!(ruling.state, "in_force");
}

#[test]
fn resolve_conclusion_in_force__candidate_only__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let _res = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: principal.clone(),
            scope,
            statement: "Term: workspace_id".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    assert!(resp.ruling.is_none(), "Candidate-only must not be in force");
    assert!(resp.chain.is_empty());
}

#[test]
fn resolve_conclusion_in_force__uncorrected_successor_candidate__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let c1 = propose_confirmed(&ports, &principal, scope.clone(), "Term: workspace_id");
    let c2 = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        c1,
        "Term: workspace_id v2 — candidate tip".into(),
        vec![EvidenceId::new()],
        "replaced",
        Privacy::LocalOnly,
        None,
    )
    .unwrap();

    let resp = resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id")
        .unwrap();
    assert!(
        resp.ruling.is_none(),
        "uncorrected Candidate tip must not rule; tip={c2}"
    );
    assert_eq!(resp.chain.len(), 1);
    assert_eq!(resp.chain[0].conclusion_id, c1.to_string());
    assert_eq!(resp.chain[0].status, format!("superseded_by:{c2}"));
}
