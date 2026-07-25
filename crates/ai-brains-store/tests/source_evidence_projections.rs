#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, SourceId, SourceVersionId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::DataKey;
use ai_brains_events::{
    Actor, AggregateType, EventKind, Payload,
    constructors::EventBuilder,
    payload::{
        ConclusionProposedPayload, DecisionProposedPayload, EvidenceRecordedPayload,
        EvidenceSupersededPayload, SourceObservedPayload, SourceRegisteredPayload,
        SourceUnavailablePayload, SourceVersionRecordedPayload,
    },
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;
use time::OffsetDateTime;

fn open_store() -> (NamedTempFile, SqliteEventStore) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    (temp_file, SqliteEventStore::new(conn))
}

fn ts() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()
}

#[test]
fn source_and_version_and_evidence__append_events__rows_materialized() {
    let (_temp, store) = open_store();
    let source_id = SourceId::new();
    let version_id = SourceVersionId::new();
    let evidence_id = EvidenceId::new();
    let actor = Actor::System;

    let reg = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SourceRegistered(SourceRegisteredPayload {
        source_id,
        kind: SourceKind::File,
        display_name: "readme".into(),
        locator: Some("/tmp/README.md".into()),
        scope: None,
    }))
    .unwrap();
    store.append_event(&reg).unwrap();

    let observed = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SourceObserved(SourceObservedPayload {
        source_id,
        observed_at: ts(),
        note: Some("poll".into()),
    }))
    .unwrap();
    store.append_event(&observed).unwrap();

    let version = EventBuilder::new(
        AggregateType::Source,
        source_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::SourceVersionRecorded(
        SourceVersionRecordedPayload {
            source_id,
            version_id,
            fingerprint: "v1:deadbeef".into(),
            recorded_at: ts(),
        },
    ))
    .unwrap();
    store.append_event(&version).unwrap();

    let evidence = EventBuilder::new(
        AggregateType::Evidence,
        evidence_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
        evidence_id,
        source_id,
        source_version_id: Some(version_id),
        fingerprint: Some("v1:deadbeef".into()),
        model_provenance: None,
        summary: "file contents".into(),
    }))
    .unwrap();
    store.append_event(&evidence).unwrap();

    let conn = store.connection().lock().unwrap();

    let (name, status, last_obs): (String, String, Option<String>) = conn
        .query_row(
            "SELECT display_name, status, last_observed_at FROM source_projection WHERE source_id = ?",
            [source_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(name, "readme");
    assert_eq!(status, "Active");
    assert!(last_obs.is_some());

    let fp: String = conn
        .query_row(
            "SELECT fingerprint FROM source_version_projection WHERE version_id = ?",
            [version_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(fp, "v1:deadbeef");

    let (summary, e_status): (String, String) = conn
        .query_row(
            "SELECT summary, status FROM evidence_projection WHERE evidence_id = ?",
            [evidence_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(summary, "file contents");
    assert_eq!(e_status, "Active");
}

#[test]
fn source_unavailable_and_evidence_superseded__update_status() {
    let (_temp, store) = open_store();
    let source_id = SourceId::new();
    let evidence_id = EvidenceId::new();
    let superseded_by = EvidenceId::new();
    let actor = Actor::System;

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Source,
                source_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::SourceRegistered(SourceRegisteredPayload {
                source_id,
                kind: SourceKind::GitRepository,
                display_name: "repo".into(),
                locator: None,
                scope: None,
            }))
            .unwrap(),
        )
        .unwrap();

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Evidence,
                evidence_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
                evidence_id,
                source_id,
                source_version_id: None,
                fingerprint: None,
                model_provenance: None,
                summary: "old".into(),
            }))
            .unwrap(),
        )
        .unwrap();

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Source,
                source_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::SourceUnavailable(SourceUnavailablePayload {
                source_id,
                reason: "offline".into(),
                marked_at: ts(),
            }))
            .unwrap(),
        )
        .unwrap();

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Evidence,
                evidence_id.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::EvidenceSuperseded(EvidenceSupersededPayload {
                evidence_id,
                superseded_by,
                reason: "newer".into(),
            }))
            .unwrap(),
        )
        .unwrap();

    let conn = store.connection().lock().unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM source_projection WHERE source_id = ?",
            [source_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "Unavailable");

    let e_status: String = conn
        .query_row(
            "SELECT status FROM evidence_projection WHERE evidence_id = ?",
            [evidence_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(e_status, "Superseded");
}

#[test]
fn conclusion_proposed__materializes_knowledge_dependencies() {
    let (_temp, store) = open_store();
    let source_id = SourceId::new();
    let version_id = SourceVersionId::new();
    let evidence_id = EvidenceId::new();
    let conclusion_id = ConclusionId::new();
    let decision_id = DecisionId::new();
    let actor = Actor::System;

    for envelope in [
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceRegistered(SourceRegisteredPayload {
            source_id,
            kind: SourceKind::File,
            display_name: "a".into(),
            locator: Some("/a".into()),
            scope: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceVersionRecorded(
            SourceVersionRecordedPayload {
                source_id,
                version_id,
                fingerprint: "v1:aa".into(),
                recorded_at: ts(),
            },
        ))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Evidence,
            evidence_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some("v1:aa".into()),
            model_provenance: None,
            summary: "snip".into(),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "X".into(),
            evidence_ids: vec![evidence_id],
            proposer: PrincipalId::new(),
            valid_from: None,
            valid_until: None,
            scope: String::new(),
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Decision,
            decision_id.as_uuid(),
            actor,
            Privacy::LocalOnly,
        )
        .build(Payload::DecisionProposed(DecisionProposedPayload {
            decision_id,
            title: "Use X".into(),
            statement: "we use X".into(),
            proposer: PrincipalId::new(),
            conclusion_ids: Some(vec![conclusion_id]),
            evidence_ids: None,
            valid_from: None,
            valid_until: None,
            scope: String::new(),
        }))
        .unwrap(),
    ] {
        store.append_event(&envelope).unwrap();
    }

    let conn = store.connection().lock().unwrap();

    let conc_deps: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_dependency_projection
             WHERE parent_type = 'Conclusion' AND parent_id = ? AND evidence_id = ?",
            [conclusion_id.to_string(), evidence_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(conc_deps, 1);

    let sv: String = conn
        .query_row(
            "SELECT source_version_id FROM knowledge_dependency_projection
             WHERE parent_type = 'Conclusion' AND parent_id = ?",
            [conclusion_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(sv, version_id.to_string());

    let dec_deps: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM knowledge_dependency_projection
             WHERE parent_type = 'Decision' AND parent_id = ? AND evidence_id = ?",
            [decision_id.to_string(), evidence_id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(dec_deps, 1);
}

#[test]
fn rebuild_projections__restores_source_evidence_dependency_rows() {
    let (_temp, mut store) = open_store();
    let source_id = SourceId::new();
    let version_id = SourceVersionId::new();
    let evidence_id = EvidenceId::new();
    let conclusion_id = ConclusionId::new();
    let actor = Actor::System;

    for envelope in [
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceRegistered(SourceRegisteredPayload {
            source_id,
            kind: SourceKind::File,
            display_name: "b".into(),
            locator: Some("/b".into()),
            scope: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceVersionRecorded(
            SourceVersionRecordedPayload {
                source_id,
                version_id,
                fingerprint: "v1:bb".into(),
                recorded_at: ts(),
            },
        ))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Evidence,
            evidence_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some("v1:bb".into()),
            model_provenance: None,
            summary: "body".into(),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            actor,
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "Y".into(),
            evidence_ids: vec![evidence_id],
            proposer: PrincipalId::new(),
            valid_from: None,
            valid_until: None,
            scope: String::new(),
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .unwrap(),
    ] {
        store.append_event(&envelope).unwrap();
    }

    let before = {
        let conn = store.connection().lock().unwrap();
        let sources: i64 = conn
            .query_row("SELECT COUNT(*) FROM source_projection", [], |r| r.get(0))
            .unwrap();
        let versions: i64 = conn
            .query_row("SELECT COUNT(*) FROM source_version_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let evidence: i64 = conn
            .query_row("SELECT COUNT(*) FROM evidence_projection", [], |r| r.get(0))
            .unwrap();
        let deps: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_dependency_projection",
                [],
                |r| r.get(0),
            )
            .unwrap();
        (sources, versions, evidence, deps)
    };
    assert_eq!(before, (1, 1, 1, 1));

    {
        let conn = store.connection().lock().unwrap();
        conn.execute("DELETE FROM knowledge_dependency_projection", [])
            .unwrap();
        conn.execute("DELETE FROM evidence_projection", []).unwrap();
        conn.execute("DELETE FROM evidence_fts", []).unwrap();
        conn.execute("DELETE FROM source_version_projection", [])
            .unwrap();
        conn.execute("DELETE FROM source_projection", []).unwrap();
    }

    store.rebuild_projections().unwrap();

    let after = {
        let conn = store.connection().lock().unwrap();
        let sources: i64 = conn
            .query_row("SELECT COUNT(*) FROM source_projection", [], |r| r.get(0))
            .unwrap();
        let versions: i64 = conn
            .query_row("SELECT COUNT(*) FROM source_version_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let evidence: i64 = conn
            .query_row("SELECT COUNT(*) FROM evidence_projection", [], |r| r.get(0))
            .unwrap();
        let deps: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_dependency_projection",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let name: String = conn
            .query_row(
                "SELECT display_name FROM source_projection WHERE source_id = ?",
                [source_id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        (sources, versions, evidence, deps, name)
    };
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, before.3);
    assert_eq!(after.4, "b");
}

/// Store append rejects raw ConclusionMarkedStale with both version and reason absent
/// (T149 Codex P2-6 / deferred #14 at store boundary).
#[test]
fn append_event__conclusion_marked_stale_both_none__rejected() {
    use ai_brains_core::ids::ConclusionId;
    use ai_brains_events::payload::ConclusionMarkedStalePayload;
    use ai_brains_store::errors::StoreError;
    use uuid::Uuid;

    let (_temp, store) = open_store();
    let conclusion_id = ConclusionId::new();
    let invalid = ConclusionMarkedStalePayload {
        conclusion_id,
        changed_source_version_id: None,
        unavailable_reason: None,
        source_id: None,
    };
    assert!(invalid.validate().is_err());

    let envelope = ai_brains_events::Envelope {
        event_id: Uuid::new_v4(),
        schema_version: 1,
        aggregate_type: AggregateType::Conclusion,
        aggregate_id: conclusion_id.as_uuid(),
        event_type: EventKind::ConclusionMarkedStale,
        occurred_at: ts(),
        actor: Actor::System,
        causation_id: None,
        correlation_id: None,
        privacy: Privacy::LocalOnly,
        payload: Payload::ConclusionMarkedStale(invalid),
        payload_hash: "deadbeef".into(),
    };

    let err = EventStore::append_event(&store, &envelope).expect_err("must reject both-None");
    assert!(
        matches!(err, StoreError::EventAppendFailed(_)),
        "got {err:?}"
    );

    let count = EventStore::read_all_events(&store).unwrap().len();
    assert_eq!(count, 0, "rejected append must not persist the event");
}

/// Same-timestamp events are ordered by event_id for deterministic replay (P2-7).
#[test]
fn read_all_events__same_timestamp__stable_event_id_order() {
    use uuid::Uuid;

    let (_temp, store) = open_store();
    let t = ts();
    let id_low = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let id_high = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    let mut e_high = EventBuilder::new(
        AggregateType::Source,
        Uuid::new_v4(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::SourceObserved(SourceObservedPayload {
        source_id: SourceId::new(),
        observed_at: t,
        note: Some("high".into()),
    }))
    .unwrap();
    e_high.event_id = id_high;
    e_high.occurred_at = t;

    let mut e_low = EventBuilder::new(
        AggregateType::Source,
        Uuid::new_v4(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::SourceObserved(SourceObservedPayload {
        source_id: SourceId::new(),
        observed_at: t,
        note: Some("low".into()),
    }))
    .unwrap();
    e_low.event_id = id_low;
    e_low.occurred_at = t;

    // Insert high id first; read order must still be event_id ASC.
    EventStore::append_event(&store, &e_high).unwrap();
    EventStore::append_event(&store, &e_low).unwrap();

    let events = EventStore::read_all_events(&store).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_id, id_low);
    assert_eq!(events[1].event_id, id_high);
}
