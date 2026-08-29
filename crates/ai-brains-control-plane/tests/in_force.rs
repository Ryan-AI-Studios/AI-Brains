//! T311 AC1–AC7 — decision in-force resolver.
#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, GovernedQueryStore, ProposeDecisionRequest, StorePorts,
    SystemClock, approve_decision, make_principal, propose_decision, resolve_in_force,
    resolve_in_force_at, revoke_decision, scope_identity_key, supersede_decision,
};
use ai_brains_core::ids::{DecisionId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use tempfile::NamedTempFile;
use time::Duration;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

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
    propose_approved_with_valid_from(ports, principal, scope, title, statement, None)
}

fn valid_from_2020() -> OffsetDateTime {
    OffsetDateTime::parse("2020-01-01T00:00:00Z", &Rfc3339).unwrap()
}

fn propose_approved_with_valid_from(
    ports: &StorePorts,
    principal: &ai_brains_core::principal::Principal,
    scope: ScopeRef,
    title: &str,
    statement: &str,
    valid_from: Option<OffsetDateTime>,
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
            valid_from,
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

#[test]
fn resolve_in_force_at__as_of_before_supersede__prior_approved() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id",
        "first ruling",
        Some(valid_from_2020()),
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

    let hop_at = ports.query.get_decision(d1).unwrap().unwrap().updated_at;
    let before = hop_at - Duration::NANOSECOND;
    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(before),
    )
    .unwrap();
    let ruling = resp
        .ruling
        .as_ref()
        .expect("prior approved must be in force before hop");
    assert_eq!(ruling.decision_id, d1.to_string());
    assert_eq!(ruling.state, "in_force");
    assert!(resp.chain.is_empty(), "hop not taken → empty chain");
    let v = serde_json::to_value(&resp).unwrap();
    assert!(v.get("as_of").is_some(), "AC3 to_value must include as_of");
    assert!(!v["as_of"].is_null());
}

#[test]
fn resolve_in_force_at__as_of_at_supersede__successor() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id",
        "first ruling",
        Some(valid_from_2020()),
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

    let hop_at = ports.query.get_decision(d1).unwrap().unwrap().updated_at;
    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(hop_at),
    )
    .unwrap();
    let ruling = resp.ruling.as_ref().expect("at hop → successor");
    assert_eq!(ruling.decision_id, d2.to_string());
    assert_eq!(resp.chain.len(), 1);
    assert_eq!(resp.chain[0].decision_id, d1.to_string());
    let v = serde_json::to_value(&resp).unwrap();
    assert!(v.get("as_of").is_some(), "AC4 to_value must include as_of");
}

#[test]
fn resolve_in_force_at__as_of_before_valid_from__none() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let _d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope,
        "Term: workspace_id",
        "a ruling",
        Some(valid_from_2020()),
    );

    let past = OffsetDateTime::parse("1970-01-01T00:00:00Z", &Rfc3339).unwrap();
    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(past),
    )
    .unwrap();
    assert!(resp.ruling.is_none());
}

#[test]
fn resolve_in_force_at__revoked_as_of_before_revoke__prior_approved() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope,
        "Term: workspace_id",
        "revoked ruling",
        Some(valid_from_2020()),
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

    let hop_at = ports.query.get_decision(d1).unwrap().unwrap().updated_at;
    let before = hop_at - Duration::NANOSECOND;
    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(before),
    )
    .unwrap();
    let ruling = resp.ruling.expect("before revoke → prior approved");
    assert_eq!(ruling.decision_id, d1.to_string());

    let now_resp =
        resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap();
    assert!(now_resp.ruling.is_none());
    assert!(now_resp.chain.is_empty());
}

#[test]
fn resolve_in_force_at__none__matches_four_arg() {
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

    let four = resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap();
    let at_none =
        resolve_in_force_at(&ports.query, &SystemClock, &scope_key, "workspace_id", None).unwrap();
    assert_eq!(four, at_none);
    let d2_s = d2.to_string();
    assert_eq!(
        four.ruling.as_ref().map(|r| r.decision_id.as_str()),
        Some(d2_s.as_str())
    );
    assert!(four.as_of.is_none());
    let v = serde_json::to_value(&four).unwrap();
    assert!(v.get("as_of").is_none(), "omit as_of when None");
}

#[test]
fn resolve_in_force_at__broken_successor_after_hop__none() {
    // After hop is due (as_of >= updated_at) but successor id is broken, do not
    // treat the Superseded parent as an as-of ruling (Codex P2 / F6(b) guard).
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope,
        "Term: workspace_id",
        "first ruling",
        Some(valid_from_2020()),
    );
    // Self-supersede creates a cycle normally; instead set a nonsense successor
    // via cycle-style self hop is InvalidTransition on now-path. Use a fresh
    // UUID that was never projected as superseded_by by calling supersede with
    // a non-existent successor — API requires existing decision. Simulate by
    // superseding to d1 itself then querying as-of at hop: cycle errors on
    // None-path; for as-of after hop, walk hits cycle → error. Prefer missing
    // successor: approve d2 in another scope so hop stops on scope mismatch.
    let other = ScopeRef::Personal(UserId::new());
    let d2 = propose_approved(
        &ports,
        &principal,
        other,
        "Term: workspace_id v2",
        "other scope",
    );
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        d2,
        "cross-scope",
        Privacy::LocalOnly,
    )
    .unwrap();

    let hop_at = ports.query.get_decision(d1).unwrap().unwrap().updated_at;
    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(hop_at),
    )
    .unwrap();
    assert!(
        resp.ruling.is_none(),
        "cross-scope successor after hop must not revive superseded parent; got {:?}",
        resp.ruling
    );
}

#[test]
fn resolve_in_force_at__as_of_mid_three_chain__prefix_only() {
    let (_t, ports) = open_ports();
    let principal = human();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = scope_identity_key(&scope);

    let d1 = propose_approved_with_valid_from(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id",
        "first",
        Some(valid_from_2020()),
    );
    let d2 = propose_approved(
        &ports,
        &principal,
        scope.clone(),
        "Term: workspace_id v2",
        "second",
    );
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d1,
        d2,
        "d1->d2",
        Privacy::LocalOnly,
    )
    .unwrap();
    let d1_hop = ports.query.get_decision(d1).unwrap().unwrap().updated_at;

    let d3 = propose_approved(&ports, &principal, scope, "Term: workspace_id v3", "third");
    supersede_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &principal,
        d2,
        d3,
        "d2->d3",
        Privacy::LocalOnly,
    )
    .unwrap();
    let d2_hop = ports.query.get_decision(d2).unwrap().unwrap().updated_at;

    // Mid: after D1 hop, before D2 hop → ruling D2, chain prefix len 1.
    assert!(
        d1_hop < d2_hop,
        "expected distinct hops; d1_hop={d1_hop} d2_hop={d2_hop}"
    );
    let mid = d2_hop - Duration::NANOSECOND;
    assert!(mid >= d1_hop, "mid must be after or at d1 hop");

    let resp = resolve_in_force_at(
        &ports.query,
        &SystemClock,
        &scope_key,
        "workspace_id",
        Some(mid),
    )
    .unwrap();
    let ruling = resp.ruling.expect("mid-chain ruling is D2");
    assert_eq!(ruling.decision_id, d2.to_string());
    assert_eq!(resp.chain.len(), 1, "prefix only (not full today chain)");
    assert_eq!(resp.chain[0].decision_id, d1.to_string());

    let today = resolve_in_force(&ports.query, &SystemClock, &scope_key, "workspace_id").unwrap();
    assert_eq!(today.chain.len(), 2);
    let d3_s = d3.to_string();
    assert_eq!(
        today.ruling.as_ref().map(|r| r.decision_id.as_str()),
        Some(d3_s.as_str())
    );
}
