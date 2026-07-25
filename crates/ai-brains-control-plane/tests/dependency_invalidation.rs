#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, ControlPlaneError, EventWriter, GovernedQueryStore, ObserveSourceRequest,
    Sha256FingerprinterPort, SourceContent, SourceUnavailableRequest, StorePorts, SystemClock,
    mark_source_unavailable, normalize_path_locator, observe_source, scope_identity_key,
    try_mark_stale_payload,
};
use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, SourceId, SourceVersionId, UserId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionProposedPayload, DecisionProposedPayload, EvidenceRecordedPayload,
    SourceRegisteredPayload, SourceVersionRecordedPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore as StoreEventStore;
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

fn ts() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()
}

/// Seed: two sources, two conclusions (each depends on one source's evidence),
/// one decision depending on conclusion A only.
struct Fixture {
    source_a: SourceId,
    conclusion_a: ConclusionId,
    conclusion_b: ConclusionId,
    decision_a: DecisionId,
    principal: PrincipalId,
    scope: ScopeRef,
}

fn seed_two_sources_two_conclusions(ports: &StorePorts) -> Fixture {
    let actor = Actor::System;
    let principal = PrincipalId::new();
    let user = UserId::new();
    let scope = ScopeRef::Personal(user);
    let scope_key = scope_identity_key(&scope);
    let source_a = SourceId::new();
    let source_b = SourceId::new();
    let version_a = SourceVersionId::new();
    let version_b = SourceVersionId::new();
    let evidence_a = EvidenceId::new();
    let evidence_b = EvidenceId::new();
    let conclusion_a = ConclusionId::new();
    let conclusion_b = ConclusionId::new();
    let decision_a = DecisionId::new();

    let events = vec![
        EventBuilder::new(
            AggregateType::Source,
            source_a.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceRegistered(SourceRegisteredPayload {
            source_id: source_a,
            kind: SourceKind::File,
            display_name: "file-a".into(),
            locator: Some(normalize_path_locator("/a.md")),
            scope: Some(scope_key.clone()),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Source,
            source_a.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceVersionRecorded(
            SourceVersionRecordedPayload {
                source_id: source_a,
                version_id: version_a,
                fingerprint: "v1:aaaa".into(),
                recorded_at: ts(),
            },
        ))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Evidence,
            evidence_a.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id: evidence_a,
            source_id: source_a,
            source_version_id: Some(version_a),
            fingerprint: Some("v1:aaaa".into()),
            model_provenance: None,
            summary: "A".into(),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Source,
            source_b.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceRegistered(SourceRegisteredPayload {
            source_id: source_b,
            kind: SourceKind::File,
            display_name: "file-b".into(),
            locator: Some(normalize_path_locator("/b.md")),
            scope: Some(scope_key.clone()),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Source,
            source_b.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::SourceVersionRecorded(
            SourceVersionRecordedPayload {
                source_id: source_b,
                version_id: version_b,
                fingerprint: "v1:bbbb".into(),
                recorded_at: ts(),
            },
        ))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Evidence,
            evidence_b.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id: evidence_b,
            source_id: source_b,
            source_version_id: Some(version_b),
            fingerprint: Some("v1:bbbb".into()),
            model_provenance: None,
            summary: "B".into(),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_a.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id: conclusion_a,
            statement: "depends on A".into(),
            evidence_ids: vec![evidence_a],
            proposer: principal,
            valid_from: None,
            valid_until: None,
            scope: String::new(),
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_b.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id: conclusion_b,
            statement: "depends on B".into(),
            evidence_ids: vec![evidence_b],
            proposer: principal,
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
            decision_a.as_uuid(),
            actor,
            Privacy::LocalOnly,
        )
        .build(Payload::DecisionProposed(DecisionProposedPayload {
            decision_id: decision_a,
            title: "Ship A".into(),
            statement: "we ship based on A".into(),
            proposer: principal,
            conclusion_ids: Some(vec![conclusion_a]),
            evidence_ids: None,
            valid_from: None,
            valid_until: None,
            scope: String::new(),
        }))
        .unwrap(),
    ];

    ports.writer.append_events(&events).unwrap();

    // Keep seeded ids live so projection edges remain meaningful under refactor.
    let _ = (source_b, evidence_a, evidence_b, version_a, version_b);

    Fixture {
        source_a,
        conclusion_a,
        conclusion_b,
        decision_a,
        principal,
        scope,
    }
}

fn count_payload(store: &SqliteEventStore, pred: impl Fn(&Payload) -> bool) -> usize {
    let events = StoreEventStore::read_all_events(store).unwrap();
    events.iter().filter(|e| pred(&e.payload)).count()
}

#[test]
fn invalidate__change_one_source__only_dependent_conclusion_stale() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    // Re-observe source A with different content via observe_source path.
    // Identity matches seeded locator /a.md + scope.
    let req = ObserveSourceRequest {
        principal: fx.principal,
        scope: fx.scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/a.md".into()),
        content: SourceContent::Bytes(b"A content changed substantially\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    };
    let result = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req)
        .expect("observe A changed");
    assert!(result.changed);
    assert_eq!(result.source_id, fx.source_a);

    assert!(
        ports.query.is_conclusion_stale(fx.conclusion_a).unwrap(),
        "conclusion A must be stale"
    );
    assert!(
        !ports.query.is_conclusion_stale(fx.conclusion_b).unwrap(),
        "conclusion B must remain non-stale"
    );

    // Decision never revoked.
    let revokes = count_payload(ports.writer.store(), |p| {
        matches!(p, Payload::DecisionRevoked(_))
    });
    assert_eq!(revokes, 0, "decisions must never be auto-revoked");
}

#[test]
fn invalidate__decision_gets_structured_review_item_not_revoke() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    let req = ObserveSourceRequest {
        principal: fx.principal,
        scope: fx.scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/a.md".into()),
        content: SourceContent::Bytes(b"A mutated again\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    };
    observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req).unwrap();

    let events = StoreEventStore::read_all_events(ports.writer.store()).unwrap();
    let review = events.iter().find_map(|e| match &e.payload {
        Payload::ReviewItemOpened(p) if p.related_decision_id == Some(fx.decision_a) => {
            Some(p.clone())
        }
        _ => None,
    });
    let review = review.expect("ReviewItemOpened for decision A");

    // Structured fields — NOT prose-only subject.contains checks as sole identity.
    assert_eq!(review.subject_kind, ReviewSubjectKind::Decision);
    assert_eq!(review.related_decision_id, Some(fx.decision_a));
    assert_eq!(review.related_source_id, Some(fx.source_a));
    assert_eq!(review.criticality, ReviewCriticality::High);
    assert_eq!(review.subject_id, fx.decision_a.to_string());
    assert_eq!(review.opened_by, fx.principal);

    let revokes = events
        .iter()
        .filter(|e| matches!(e.payload, Payload::DecisionRevoked(_)))
        .count();
    assert_eq!(revokes, 0);
}

#[test]
fn invalidate__source_unavailable__structured_review_and_stale() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;

    let result = mark_source_unavailable(
        &ports.writer,
        &ports.query,
        &clock,
        SourceUnavailableRequest {
            source_id: fx.source_a,
            reason: "network offline".into(),
            opened_by: fx.principal,
            privacy: Privacy::LocalOnly,
            criticality: ReviewCriticality::Critical,
        },
    )
    .expect("unavailable");

    assert!(result.stale_conclusions.contains(&fx.conclusion_a));
    assert!(!result.stale_conclusions.contains(&fx.conclusion_b));
    assert!(ports.query.is_conclusion_stale(fx.conclusion_a).unwrap());
    assert!(!ports.query.is_conclusion_stale(fx.conclusion_b).unwrap());

    let events = StoreEventStore::read_all_events(ports.writer.store()).unwrap();

    let source_review = events.iter().find_map(|e| match &e.payload {
        Payload::ReviewItemOpened(p)
            if p.subject_kind == ReviewSubjectKind::Source
                && p.related_source_id == Some(fx.source_a) =>
        {
            Some(p.clone())
        }
        _ => None,
    });
    let source_review = source_review.expect("source review item");
    assert_eq!(source_review.criticality, ReviewCriticality::Critical);
    assert_eq!(source_review.subject_id, fx.source_a.to_string());

    let decision_review = events.iter().find_map(|e| match &e.payload {
        Payload::ReviewItemOpened(p) if p.related_decision_id == Some(fx.decision_a) => {
            Some(p.clone())
        }
        _ => None,
    });
    let decision_review = decision_review.expect("decision review on unavailable");
    assert_eq!(decision_review.subject_kind, ReviewSubjectKind::Decision);
    assert_eq!(decision_review.criticality, ReviewCriticality::Critical);

    assert!(events.iter().any(
        |e| matches!(&e.payload, Payload::SourceUnavailable(p) if p.source_id == fx.source_a)
    ));
}

#[test]
fn revalidate__same_fingerprint__clears_matching_stale_only() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();

    // Observe source via workflow so fingerprint is real. Same scope for both.
    let scope = ScopeRef::Personal(UserId::new());
    let req_a = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/ra.md".into()),
        content: SourceContent::Bytes(b"stable body A\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs_a = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req_a.clone(),
    )
    .unwrap();

    let req_b = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "file-b".into(),
        locator: Some("/rb.md".into()),
        content: SourceContent::Bytes(b"stable body B\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs_b = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_b).unwrap();

    let evidence_a = obs_a.evidence_id.expect("ev A");
    let evidence_b = obs_b.evidence_id.expect("ev B");
    let conclusion_a = ConclusionId::new();
    let conclusion_b = ConclusionId::new();
    let actor = Actor::System;

    ports
        .writer
        .append_events(&[
            EventBuilder::new(
                AggregateType::Conclusion,
                conclusion_a.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::ConclusionProposed(ConclusionProposedPayload {
                conclusion_id: conclusion_a,
                statement: "A".into(),
                evidence_ids: vec![evidence_a],
                proposer: principal,
                valid_from: None,
                valid_until: None,
                scope: String::new(),
                protected_category: None,
                unsupported: false,
                model_provenance: None,
            }))
            .unwrap(),
            EventBuilder::new(
                AggregateType::Conclusion,
                conclusion_b.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::ConclusionProposed(ConclusionProposedPayload {
                conclusion_id: conclusion_b,
                statement: "B".into(),
                evidence_ids: vec![evidence_b],
                proposer: principal,
                valid_from: None,
                valid_until: None,
                scope: String::new(),
                protected_category: None,
                unsupported: false,
                model_provenance: None,
            }))
            .unwrap(),
        ])
        .unwrap();

    // Mark A unavailable → conclusion A stale; B untouched.
    mark_source_unavailable(
        &ports.writer,
        &ports.query,
        &clock,
        SourceUnavailableRequest {
            source_id: obs_a.source_id,
            reason: "temporary outage".into(),
            opened_by: principal,
            privacy: Privacy::LocalOnly,
            criticality: ReviewCriticality::Medium,
        },
    )
    .unwrap();
    assert!(ports.query.is_conclusion_stale(conclusion_a).unwrap());
    assert!(!ports.query.is_conclusion_stale(conclusion_b).unwrap());

    // Also mark B stale so we can prove clear is matching-source only.
    mark_source_unavailable(
        &ports.writer,
        &ports.query,
        &clock,
        SourceUnavailableRequest {
            source_id: obs_b.source_id,
            reason: "b also out".into(),
            opened_by: principal,
            privacy: Privacy::LocalOnly,
            criticality: ReviewCriticality::Low,
        },
    )
    .unwrap();
    assert!(ports.query.is_conclusion_stale(conclusion_b).unwrap());

    // Re-observe A with same content → changed=false + revalidation clears A's stale only.
    let again = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_a).unwrap();
    assert!(!again.changed);
    assert!(
        !ports.query.is_conclusion_stale(conclusion_a).unwrap(),
        "A stale cleared by same-fingerprint revalidation"
    );
    assert!(
        ports.query.is_conclusion_stale(conclusion_b).unwrap(),
        "B remains stale (different source)"
    );
}

#[test]
fn conclusion_marked_stale__both_none__fails_from_control_plane() {
    let conclusion_id = ConclusionId::new();
    let err = try_mark_stale_payload(conclusion_id, None, None).expect_err("both none");
    assert!(
        matches!(err, ControlPlaneError::InvalidPayload(_)),
        "got {err:?}"
    );

    let err_empty =
        try_mark_stale_payload(conclusion_id, None, Some("  ".into())).expect_err("empty reason");
    assert!(matches!(err_empty, ControlPlaneError::InvalidPayload(_)));

    let ok = try_mark_stale_payload(conclusion_id, Some(SourceVersionId::new()), None)
        .expect("version ok");
    assert!(ok.changed_source_version_id.is_some());

    let ok_reason =
        try_mark_stale_payload(conclusion_id, None, Some("gone".into())).expect("reason ok");
    assert_eq!(ok_reason.unavailable_reason.as_deref(), Some("gone"));
}

/// Revalidation must not clear a stale fact whose version_id ≠ latest version
/// (T149 Codex P2-4).
#[test]
fn revalidate__mismatched_stale_version__not_cleared() {
    use ai_brains_events::payload::ConclusionMarkedStalePayload;

    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();
    let scope = ScopeRef::Personal(UserId::new());

    let req = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "file-m".into(),
        locator: Some("/mismatch.md".into()),
        content: SourceContent::Bytes(b"stable mismatch body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req.clone(),
    )
    .unwrap();
    let evidence_id = obs.evidence_id.expect("ev");
    let conclusion_id = ConclusionId::new();
    let foreign_version = SourceVersionId::new(); // not the source's real version

    ports
        .writer
        .append_events(&[
            EventBuilder::new(
                AggregateType::Conclusion,
                conclusion_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ConclusionProposed(ConclusionProposedPayload {
                conclusion_id,
                statement: "depends".into(),
                evidence_ids: vec![evidence_id],
                proposer: principal,
                valid_from: None,
                valid_until: None,
                scope: String::new(),
                protected_category: None,
                unsupported: false,
                model_provenance: None,
            }))
            .unwrap(),
            EventBuilder::new(
                AggregateType::Conclusion,
                conclusion_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ConclusionMarkedStale(
                ConclusionMarkedStalePayload::try_new(conclusion_id, Some(foreign_version), None)
                    .unwrap(),
            ))
            .unwrap(),
        ])
        .unwrap();

    assert!(ports.query.is_conclusion_stale(conclusion_id).unwrap());
    let fact = ports
        .query
        .latest_stale_fact(conclusion_id)
        .unwrap()
        .expect("stale fact");
    assert_eq!(fact.changed_source_version_id, Some(foreign_version));

    // Same fingerprint re-observe must NOT clear mismatched version stale.
    let again = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req).unwrap();
    assert!(!again.changed);
    assert!(
        ports.query.is_conclusion_stale(conclusion_id).unwrap(),
        "stale for a non-matching version_id must not clear on revalidation"
    );
}

/// Multiple source changes while already stale must not leave permanent Pending
/// rows in invalidation_queue_projection (T149 Codex P2-5).
#[test]
fn multi_change_while_stale__no_permanent_pending_queue_rows() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    for body in [
        b"change-v1\n".as_slice(),
        b"change-v2\n".as_slice(),
        b"change-v3\n".as_slice(),
    ] {
        let req = ObserveSourceRequest {
            principal: fx.principal,
            scope: fx.scope.clone(),
            kind: SourceKind::File,
            display_name: "file-a".into(),
            locator: Some("/a.md".into()),
            content: SourceContent::Bytes(body.to_vec()),
            privacy: Privacy::LocalOnly,
            run_invalidation: true,
        };
        observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req)
            .expect("observe change");
    }

    assert!(ports.query.is_conclusion_stale(fx.conclusion_a).unwrap());

    let conn = ports.writer.store().connection().lock().unwrap();
    let pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invalidation_queue_projection
             WHERE parent_type = 'Conclusion'
               AND parent_id = ?
               AND status = 'Pending'",
            [fx.conclusion_a.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        pending, 0,
        "no Pending queue rows may remain after multi-change while already stale"
    );
}

#[test]
fn path_locator_drive_case_variants__normalize_to_same_identity() {
    let upper = normalize_path_locator(r"C:\Dev\Project\readme.md");
    let lower = normalize_path_locator(r"c:\dev\project\readme.md");
    let slash = normalize_path_locator(r"C:/Dev/Project/readme.md");
    assert_eq!(upper, lower);
    assert_eq!(upper, slash);

    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();
    let scope = ScopeRef::Personal(UserId::new());

    let req_upper = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "readme".into(),
        locator: Some(r"C:\Dev\Project\readme.md".into()),
        content: SourceContent::Bytes(b"drive case body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let a = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_upper).unwrap();

    let req_lower = ObserveSourceRequest {
        principal,
        scope,
        kind: SourceKind::File,
        display_name: "readme".into(),
        locator: Some(r"c:\dev\project\readme.md".into()),
        content: SourceContent::Bytes(b"drive case body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let b = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_lower).unwrap();
    assert_eq!(
        a.source_id, b.source_id,
        "drive-case variants must resolve to the same source"
    );
    assert!(!b.changed, "same content + normalized identity → unchanged");
}

#[test]
fn path_locator_wsl_and_windows__normalize_to_same_source_id() {
    let wsl = normalize_path_locator("/mnt/c/Dev/Project/readme.md");
    let win = normalize_path_locator(r"C:\Dev\Project\readme.md");
    assert_eq!(wsl, win, "WSL and Windows forms must normalize equal");

    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();
    let scope = ScopeRef::Personal(UserId::new());

    let req_win = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "readme".into(),
        locator: Some(r"C:\Dev\Project\readme.md".into()),
        content: SourceContent::Bytes(b"wsl map body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let a = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_win).unwrap();

    let req_wsl = ObserveSourceRequest {
        principal,
        scope,
        kind: SourceKind::File,
        display_name: "readme".into(),
        locator: Some("/mnt/c/Dev/Project/readme.md".into()),
        content: SourceContent::Bytes(b"wsl map body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let b = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_wsl).unwrap();
    assert_eq!(
        a.source_id, b.source_id,
        "WSL and Windows locator forms must resolve to the same source_id"
    );
    assert!(!b.changed);
}

/// Conclusion depending on A and B: unavailable on A must not clear when B revalidates.
#[test]
fn revalidate__unavailable_other_source__not_cleared() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();
    let scope = ScopeRef::Personal(UserId::new());

    let req_a = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/shared-a.md".into()),
        content: SourceContent::Bytes(b"shared A\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs_a = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_a).unwrap();

    let req_b = ObserveSourceRequest {
        principal,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "file-b".into(),
        locator: Some("/shared-b.md".into()),
        content: SourceContent::Bytes(b"shared B\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs_b = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req_b.clone(),
    )
    .unwrap();

    let evidence_a = obs_a.evidence_id.expect("ev A");
    let evidence_b = obs_b.evidence_id.expect("ev B");
    let conclusion_id = ConclusionId::new();

    ports
        .writer
        .append_events(&[EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "depends on A and B".into(),
            evidence_ids: vec![evidence_a, evidence_b],
            proposer: principal,
            valid_from: None,
            valid_until: None,
            scope: String::new(),
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .unwrap()])
        .unwrap();

    mark_source_unavailable(
        &ports.writer,
        &ports.query,
        &clock,
        SourceUnavailableRequest {
            source_id: obs_a.source_id,
            reason: "A offline".into(),
            opened_by: principal,
            privacy: Privacy::LocalOnly,
            criticality: ReviewCriticality::Medium,
        },
    )
    .unwrap();
    assert!(ports.query.is_conclusion_stale(conclusion_id).unwrap());

    let fact = ports
        .query
        .latest_stale_fact(conclusion_id)
        .unwrap()
        .expect("stale");
    assert_eq!(fact.source_id, Some(obs_a.source_id));

    // Re-observe B unchanged — must NOT clear stale caused by A.
    let again_b = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_b).unwrap();
    assert!(!again_b.changed);
    assert!(
        ports.query.is_conclusion_stale(conclusion_id).unwrap(),
        "stale from unavailable A must remain when only B is revalidated"
    );
}

/// `run_invalidation: false` on a changed existing source must not leave Pending queue rows.
#[test]
fn observe_source__run_invalidation_false__no_pending_queue_rows() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    let req = ObserveSourceRequest {
        principal: fx.principal,
        scope: fx.scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/a.md".into()),
        content: SourceContent::Bytes(b"no-inv-change\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let obs = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req)
        .expect("observe change without invalidation");
    assert!(obs.changed);

    let conn = ports.writer.store().connection().lock().unwrap();
    let pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM invalidation_queue_projection
             WHERE status = 'Pending'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        pending, 0,
        "run_invalidation:false must not strand Pending queue rows"
    );
}

/// Observation change + invalidation must be one append path: dependents are
/// stale after a single successful `observe_source` call (T149-F1).
#[test]
fn observe_source__change_with_dependents__stale_in_same_call() {
    let (_tmp, ports) = open_ports();
    let fx = seed_two_sources_two_conclusions(&ports);
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;

    assert!(!ports.query.is_conclusion_stale(fx.conclusion_a).unwrap());

    let req = ObserveSourceRequest {
        principal: fx.principal,
        scope: fx.scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/a.md".into()),
        content: SourceContent::Bytes(b"atomic change body\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    };
    let result = observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req)
        .expect("observe changed");
    assert!(result.changed);
    assert!(result.version_id.is_some());
    // Same call path: version/evidence committed together with stale.
    assert!(
        ports.query.is_conclusion_stale(fx.conclusion_a).unwrap(),
        "stale must land in the same observe_source call (single append batch)"
    );
    assert!(!ports.query.is_conclusion_stale(fx.conclusion_b).unwrap());

    // Writer that fails on append: no version and no stale (no intermediate state).
    struct FailingWriter;
    impl EventWriter for FailingWriter {
        fn append_events(
            &self,
            _events: &[ai_brains_events::Envelope],
        ) -> ai_brains_control_plane::Result<()> {
            Err(ControlPlaneError::EventAppend("simulated".into()))
        }
    }
    let versions_before = ports.query.source_version_count(fx.source_a).unwrap();
    let fail_req = ObserveSourceRequest {
        principal: fx.principal,
        scope: fx.scope.clone(),
        kind: SourceKind::File,
        display_name: "file-a".into(),
        locator: Some("/a.md".into()),
        content: SourceContent::Bytes(b"would-be second change\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    };
    let _ = observe_source(&FailingWriter, &ports.query, &clock, &fp, &policy, fail_req)
        .expect_err("writer fails");
    assert_eq!(
        ports.query.source_version_count(fx.source_a).unwrap(),
        versions_before,
        "failed single batch must not add a version"
    );
}

#[test]
fn observe_source__two_scopes_same_locator__distinct_sources() {
    let (_tmp, ports) = open_ports();
    let clock = SystemClock;
    let fp = Sha256FingerprinterPort::new();
    let policy = AllowAllPolicy;
    let principal = PrincipalId::new();

    let scope_a = ScopeRef::Personal(UserId::new());
    let scope_b = ScopeRef::Personal(UserId::new());
    assert_ne!(scope_identity_key(&scope_a), scope_identity_key(&scope_b));

    let mut req_a = ObserveSourceRequest {
        principal,
        scope: scope_a,
        kind: SourceKind::File,
        display_name: "shared".into(),
        locator: Some("/shared.md".into()),
        content: SourceContent::Bytes(b"scope A content\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: false,
    };
    let a = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &policy,
        req_a.clone(),
    )
    .expect("scope A");

    req_a.scope = scope_b;
    req_a.content = SourceContent::Bytes(b"scope B content\n".to_vec());
    let b =
        observe_source(&ports.writer, &ports.query, &clock, &fp, &policy, req_a).expect("scope B");

    assert_ne!(
        a.source_id, b.source_id,
        "same locator in different scopes must be distinct sources"
    );
}
