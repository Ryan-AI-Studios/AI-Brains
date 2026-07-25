#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    ControlPlaneError, GovernedQueryStore, StorePorts, make_principal, resolve_review_item,
};
use ai_brains_core::ids::{PrincipalId, ReviewItemId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
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

#[test]
fn resolve_review_item__requires_principal_and_reason() {
    let (_t, ports) = open_ports();
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

    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "h");

    let err = resolve_review_item(
        &ports.writer,
        &ports.query,
        &human,
        review_item_id,
        "   ",
        Privacy::LocalOnly,
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));

    resolve_review_item(
        &ports.writer,
        &ports.query,
        &human,
        review_item_id,
        "acknowledged and revalidated",
        Privacy::LocalOnly,
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
