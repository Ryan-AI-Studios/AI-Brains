#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, DenyAllPolicy, EventWriter, GovernedQueryStore,
    ObserveSourceRequest, ObserveSourceResult, Sha256FingerprinterPort, SourceContent, StorePorts,
    SystemClock, observe_source,
};
use ai_brains_core::ids::{PrincipalId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::DataKey;
use ai_brains_events::Envelope;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::NamedTempFile;
use uuid::Uuid;

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

fn file_req(name: &str, locator: &str, content: &[u8]) -> ObserveSourceRequest {
    // Stable principal/scope so follow-up observes resolve the same source identity.
    ObserveSourceRequest {
        principal: PrincipalId::from_uuid(Uuid::from_u128(1)),
        scope: ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(2))),
        kind: SourceKind::File,
        display_name: name.into(),
        locator: Some(locator.into()),
        content: SourceContent::Bytes(content.to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    }
}

#[test]
fn observe_source__unchanged_fingerprint__changed_false_no_extra_version() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    let req = file_req("readme", "/tmp/README.md", b"hello world\n");
    let first = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req.clone(),
    )
    .expect("first observe");
    assert!(first.changed);
    assert!(first.version_id.is_some());
    assert!(first.evidence_id.is_some());
    assert_eq!(
        ports.query.source_version_count(first.source_id).unwrap(),
        1
    );
    assert_eq!(
        ports
            .query
            .evidence_count_for_source(first.source_id)
            .unwrap(),
        1
    );

    let second = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req)
        .expect("second observe same content");
    assert!(!second.changed);
    assert_eq!(second.source_id, first.source_id);
    assert_eq!(second.fingerprint, first.fingerprint);
    assert_eq!(
        ports.query.source_version_count(first.source_id).unwrap(),
        1,
        "unchanged must not create an extra version"
    );
    assert_eq!(
        ports
            .query
            .evidence_count_for_source(first.source_id)
            .unwrap(),
        1,
        "unchanged must not create extra evidence"
    );
}

#[test]
fn observe_source__changed_content__one_version_and_one_evidence() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    let mut req = file_req("notes", "/tmp/notes.md", b"v1 content\n");
    let first = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req.clone(),
    )
    .expect("v1");
    assert!(first.changed);

    req.content = SourceContent::Bytes(b"v2 content changed\n".to_vec());
    let second: ObserveSourceResult =
        observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req).expect("v2");
    assert!(second.changed);
    assert_eq!(second.source_id, first.source_id);
    assert_ne!(second.fingerprint, first.fingerprint);
    assert_eq!(
        ports.query.source_version_count(first.source_id).unwrap(),
        2
    );
    assert_eq!(
        ports
            .query
            .evidence_count_for_source(first.source_id)
            .unwrap(),
        2
    );
}

#[test]
fn observe_source__policy_deny__returns_policy_denied() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = DenyAllPolicy;

    let err = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        file_req("x", "/x", b"data"),
    )
    .expect_err("must deny");
    assert!(
        matches!(err, ControlPlaneError::PolicyDenied(_)),
        "got {err:?}"
    );
    // No sources registered.
    assert!(
        ports
            .query
            .find_source("", &SourceKind::File, Some("/x"), "x")
            .unwrap()
            .is_none()
    );
}

/// Mock writer that fails: observation must not leave partial projection rows
/// when using a real query store (writer never commits).
#[test]
fn observe_source__writer_failure__no_partial_projection_rows() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    // Seed one successful observation so the vault is non-empty.
    let req = file_req("seed", "/seed", b"seed\n");
    let seeded =
        observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req).expect("seed");
    let versions_before = ports.query.source_version_count(seeded.source_id).unwrap();

    struct FailingWriter {
        calls: Arc<AtomicUsize>,
    }
    impl EventWriter for FailingWriter {
        fn append_events(&self, _events: &[Envelope]) -> ai_brains_control_plane::Result<()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(ControlPlaneError::EventAppend("simulated failure".into()))
        }
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let failing = FailingWriter {
        calls: Arc::clone(&calls),
    };
    let change = file_req("seed", "/seed", b"changed after seed\n");
    // Re-use same principal/scope identity as seeded source path via locator.
    let err = observe_source(&failing, &ports.query, &clock, &fp, &policy, change)
        .expect_err("writer fails");
    assert!(matches!(err, ControlPlaneError::EventAppend(_)));
    assert!(calls.load(Ordering::SeqCst) >= 1);
    assert_eq!(
        ports.query.source_version_count(seeded.source_id).unwrap(),
        versions_before,
        "failed append must not add a version"
    );
    assert_eq!(
        ports
            .query
            .evidence_count_for_source(seeded.source_id)
            .unwrap(),
        1,
        "failed append must not add evidence"
    );
}

/// Real store multi-event append is all-or-nothing (transactional).
#[test]
fn store_append_events__mid_batch_failure__rolls_back_all() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);

    use ai_brains_core::ids::{SourceId, SourceVersionId};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::{SourceRegisteredPayload, SourceVersionRecordedPayload};
    use ai_brains_events::{Actor, AggregateType, Payload};
    use time::OffsetDateTime;

    let source_id = SourceId::new();
    let v1 = SourceVersionId::new();
    let v2 = SourceVersionId::new();
    let actor = Actor::System;
    let ts = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    let reg = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SourceRegistered(SourceRegisteredPayload {
        source_id,
        kind: SourceKind::File,
        display_name: "tx".into(),
        locator: Some("/tx".into()),
        scope: None,
    }))
    .unwrap();

    let ver1 = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SourceVersionRecorded(
        SourceVersionRecordedPayload {
            source_id,
            version_id: v1,
            fingerprint: "v1:aaa".into(),
            recorded_at: ts,
        },
    ))
    .unwrap();

    // Duplicate fingerprint for same source → UNIQUE constraint on second version.
    let ver_dup = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::SourceVersionRecorded(
        SourceVersionRecordedPayload {
            source_id,
            version_id: v2,
            fingerprint: "v1:aaa".into(), // same fingerprint → conflict
            recorded_at: ts,
        },
    ))
    .unwrap();

    let err = EventStore::append_events(&store, &[reg, ver1, ver_dup]);
    assert!(err.is_err(), "duplicate fingerprint must fail the batch");

    let conn = store.connection().lock().unwrap();
    let sources: i64 = conn
        .query_row("SELECT COUNT(*) FROM source_projection", [], |r| r.get(0))
        .unwrap();
    let versions: i64 = conn
        .query_row("SELECT COUNT(*) FROM source_version_projection", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(sources, 0, "rolled back: no source row");
    assert_eq!(versions, 0, "rolled back: no version row");
}
