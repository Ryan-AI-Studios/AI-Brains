//! T311 AC1–AC7 — decision in-force resolver.
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, ProposeDecisionRequest, StorePorts, SystemClock,
    approve_decision, make_principal, propose_decision, resolve_in_force, revoke_decision,
    scope_identity_key, supersede_decision,
};
use ai_brains_core::ids::{DecisionId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
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

fn propose_approved(
    ports: &StorePorts,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
    title: &str,
    statement: &str,
) -> DecisionId {
    let res = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: principal.clone(),
            scope,
            title: title.into(),
            statement: statement.into(),
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
        principal,
        res.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    res.decision_id
}

#[test]
fn resolve_in_force__superseded_root__current_approved_in_force() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id",
        "first ruling",
    );
    let d2 = propose_approved(
        &ports,
        &principal,
        scope,
        "Term: workspace_id v2",
        "second ruling",
    );
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        d2,
        "replaced",
        Privacy::LocalOnly,
    )
    .unwrap();

    let resp = resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap();
    let ruling = resp.ruling.expect("current approved must be in force");
    assert_eq!(ruling.decision_id, d2.to_string());
    assert_eq!(ruling.state, "in_force");
    assert_eq!(resp.chain.len(), 1);
    assert_eq!(resp.chain[0].decision_id, d1.to_string());
    assert_eq!(resp.chain[0].status, format!("superseded_by:{d2}"));
}

#[test]
fn resolve_in_force__successor_term__empty_chain() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id",
        "first ruling",
    );
    let d2 = propose_approved(
        &ports,
        &principal,
        scope,
        "Term: successor_id",
        "second ruling",
    );
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        d2,
        "replaced",
        Privacy::LocalOnly,
    )
    .unwrap();

    let resp = resolve_in_force(&ports.query, &SystemClock, &scope_key, "successor_id").unwrap();
    let ruling = resp.ruling.expect("successor is in force");
    assert_eq!(ruling.decision_id, d2.to_string());
    assert_eq!(ruling.state, "in_force");
    assert!(
        resp.chain.is_empty(),
        "matching the successor is not a walk"
    );
}

#[test]
fn resolve_in_force__revoked_root__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved(
        &ports,
        &principal,
        scope,
        "Term: workspace_id",
        "revoked ruling",
    );
    revoke_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        "withdrawn",
        Privacy::LocalOnly,
    )
    .unwrap();

    let resp = resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
}

#[test]
fn resolve_in_force__unknown_term__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);
    propose_approved(&ports, &principal, scope, "Term: workspace_id", "a ruling");

    let resp = resolve_in_force(&ports.query, &SystemClock, &scope_key, "no_such_term").unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
    let v = serde_json::to_value(&resp).unwrap();
    assert!(v.get("ruling").is_some(), "F4/AC10 ruling key must exist");
    assert!(v["ruling"].is_null());
}

#[test]
fn resolve_in_force__empty_term__err() {
    let (_t, ports) = open_ports();
    let scope_key = scope_identity_key(&ScopeRef::Personal(UserId::new()));
    let err = resolve_in_force(&ports.query, &SystemClock, &scope_key, "   ").unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::InvalidPayload(ref m) if m.contains("non-empty")),
        "expected InvalidPayload for empty term, got {err:?}"
    );
}

#[test]
fn resolve_in_force__other_scope_row__not_visible() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope_a = ScopeRef::Repository(ProjectId::new());
    let scope_b = ScopeRef::Repository(ProjectId::new());
    let key_a = scope_identity_key(&scope_a);

    propose_approved(
        &ports,
        &principal,
        scope_b,
        "Term: workspace_id",
        "other project",
    );

    let resp = resolve_in_force(&ports.query, &SystemClock, &key_a, "workspace_id").unwrap();
    assert!(resp.ruling.is_none());
    assert!(resp.chain.is_empty());
}

#[test]
fn resolve_in_force__cycle__error() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved(&ports, &principal, scope, "Term: workspace_id", "cyclic");
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        d1,
        "self",
        Privacy::LocalOnly,
    )
    .unwrap();

    let err = resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::InvalidTransition(_)),
        "expected InvalidTransition on cycle, got {err:?}"
    );
}
