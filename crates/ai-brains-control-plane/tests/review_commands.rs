#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, GovernedQueryStore, StorePorts, SystemClock, issue_grant,
    make_principal, register_principal, resolve_review_item,
};
use ai_brains_core::ids::{PrincipalId, ReviewItemId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::ReviewItemOpenedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
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

fn open_review_item(ports: &StorePorts) -> ReviewItemId {
    let review_item_id = ReviewItemId::new();
    let opened_by = PrincipalId::new();
    let env = EventBuilder::new(
        AggregateType::ReviewItem,
        review_item_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
        review_item_id,
        subject: "source unavailable".into(),
        opened_by,
        subject_kind: ReviewSubjectKind::Source,
        subject_id: "src".into(),
        criticality: ReviewCriticality::Medium,
        related_conclusion_id: None,
        related_decision_id: None,
        related_source_id: None,
    }))
    .unwrap();
    ports.writer.store().append_event(&env).unwrap();
    review_item_id
}

#[test]
fn resolve_review_item__requires_principal_and_reason() {
    let (_t, ports) = open_ports();
    let review_item_id = open_review_item(&ports);

    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "h");
    let scope = ScopeRef::Personal(UserId::new());

    let err = resolve_review_item(
        &ports.writer,
        &ports.query,
        &AllowAllPolicy,
        &human,
        review_item_id,
        "   ",
        Privacy::LocalOnly,
        scope.clone(),
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));

    resolve_review_item(
        &ports.writer,
        &ports.query,
        &AllowAllPolicy,
        &human,
        review_item_id,
        "acknowledged and revalidated",
        Privacy::LocalOnly,
        scope,
    )
    .unwrap();

    let row = ports
        .query
        .get_review_item(review_item_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.status, "Resolved");
    assert_eq!(
        row.resolution.as_deref(),
        Some("acknowledged and revalidated")
    );
    assert_eq!(
        row.resolved_by.as_deref(),
        Some(human.id.to_string().as_str())
    );
    assert_eq!(row.subject_kind, "Source");
    assert_eq!(row.criticality, "Medium");
}

/// R2-F1: Agent cannot resolve review items even with ReadDecisions (or ApproveDecision grant).
#[test]
fn resolve_review_item__agent_with_read_decisions__denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let review_item_id = open_review_item(&ports);
    let scope = ScopeRef::Personal(UserId::new());

    let agent = make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent");
    register_principal(&ports.writer, &clock, &agent).unwrap();
    issue_grant(
        &ports.writer,
        &clock,
        agent.id,
        scope.clone(),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();
    // Even an ApproveDecision grant must not let agents resolve (kind gate + matrix hard-deny).
    issue_grant(
        &ports.writer,
        &clock,
        agent.id,
        scope.clone(),
        GrantCapability::ApproveDecision,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.policy_evaluator();
    let err = resolve_review_item(
        &ports.writer,
        &ports.query,
        &policy,
        &agent,
        review_item_id,
        "agent tries to close review",
        Privacy::LocalOnly,
        scope.clone(),
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::ApprovalRequired(_)),
        "expected ApprovalRequired for agent, got {err:?}"
    );

    // AllowAll still rejects non-Human before policy.allow.
    let err_allow_all = resolve_review_item(
        &ports.writer,
        &ports.query,
        &AllowAllPolicy,
        &agent,
        review_item_id,
        "agent tries under AllowAll",
        Privacy::LocalOnly,
        scope,
    )
    .unwrap_err();
    assert!(
        matches!(err_allow_all, ControlPlaneError::ApprovalRequired(_)),
        "expected ApprovalRequired under AllowAll, got {err_allow_all:?}"
    );

    let row = ports
        .query
        .get_review_item(review_item_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.status, "Open");
}

/// R2-F1: Human with ApproveDecision grant can resolve via DefaultPolicyEvaluator.
#[test]
fn resolve_review_item__human_with_approve_decision__allowed() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let review_item_id = open_review_item(&ports);
    let scope = ScopeRef::Personal(UserId::new());
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "human");
    register_principal(&ports.writer, &clock, &human).unwrap();
    issue_grant(
        &ports.writer,
        &clock,
        human.id,
        scope.clone(),
        GrantCapability::ApproveDecision,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.policy_evaluator();
    resolve_review_item(
        &ports.writer,
        &ports.query,
        &policy,
        &human,
        review_item_id,
        "human approved resolution",
        Privacy::LocalOnly,
        scope,
    )
    .unwrap();

    let row = ports
        .query
        .get_review_item(review_item_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.status, "Resolved");
    assert_eq!(row.resolution.as_deref(), Some("human approved resolution"));
    assert_eq!(
        row.resolved_by.as_deref(),
        Some(human.id.to_string().as_str())
    );
}

/// R2-F1: Human without ApproveDecision grant is denied by DefaultPolicyEvaluator.
#[test]
fn resolve_review_item__human_without_grant__policy_denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let review_item_id = open_review_item(&ports);
    let scope = ScopeRef::Personal(UserId::new());
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "human");
    register_principal(&ports.writer, &clock, &human).unwrap();
    // ReadDecisions alone is insufficient for review resolve.
    issue_grant(
        &ports.writer,
        &clock,
        human.id,
        scope.clone(),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.policy_evaluator();
    let err = resolve_review_item(
        &ports.writer,
        &ports.query,
        &policy,
        &human,
        review_item_id,
        "should be denied without ApproveDecision",
        Privacy::LocalOnly,
        scope,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::PolicyDenied(_)),
        "expected PolicyDenied without ApproveDecision, got {err:?}"
    );

    let row = ports
        .query
        .get_review_item(review_item_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.status, "Open");
}
