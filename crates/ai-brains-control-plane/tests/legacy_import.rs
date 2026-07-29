#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T167 — legacy memory classification importer tests (§9.1).

use ai_brains_control_plane::{
    ApplyOpts, GovernedQueryStore, ImportActionKind, ImportMechanism, ImportOpts,
    NS_LEGACY_DECISION, NS_LEGACY_EVIDENCE, StorePorts, SystemClock, apply_legacy_import,
    classify_legacy, compute_plan_hash, id_from_command, legacy_conclusion_id, legacy_decision_id,
    legacy_evidence_id, legacy_review_id, legacy_source_id, plan_report_json,
};
use ai_brains_core::ids::{MemoryId, PrincipalId, ProjectId, SessionId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    DecisionRecordedPayload, MemoryForgottenPayload, MemoryPinnedPayload, MemoryRestoredPayload,
    MemorySynthesizedPayload, SessionSummaryCreatedPayload,
};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use uuid::Uuid;

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

fn opts() -> ImportOpts {
    ImportOpts {
        dry_run: true,
        include_truncated_summaries: false,
        default_scope: Some(ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(99)))),
        principal_id: PrincipalId::from_uuid(Uuid::from_u128(7)),
        command_id: Some("test-import-cmd".into()),
    }
}

fn envelope(
    aggregate: AggregateType,
    aggregate_id: Uuid,
    privacy: Privacy,
    payload: Payload,
) -> Envelope {
    EventBuilder::new(aggregate, aggregate_id, Actor::System, privacy)
        .build(payload)
        .unwrap()
}

fn pin_event(
    memory_id: MemoryId,
    content: &str,
    project_id: Option<ProjectId>,
    source_tag: Option<&str>,
) -> Envelope {
    envelope(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: content.into(),
            session_id: None,
            project_id,
            tx_id: None,
            rank: None,
            source_tag: source_tag.map(str::to_string),
            query_text: None,
        }),
    )
}

fn forget_event(memory_id: MemoryId) -> Envelope {
    envelope(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::MemoryForgotten(MemoryForgottenPayload { memory_id }),
    )
}

fn restore_event(memory_id: MemoryId) -> Envelope {
    envelope(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::MemoryRestored(MemoryRestoredPayload { memory_id }),
    )
}

fn synth_event(
    memory_id: MemoryId,
    content: &str,
    project_id: ProjectId,
    source_memory_ids: Vec<MemoryId>,
) -> Envelope {
    envelope(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::MemorySynthesized(MemorySynthesizedPayload {
            memory_id,
            level: 1,
            source_memory_ids,
            project_id,
            content: content.into(),
        }),
    )
}

fn decision_event(
    legacy_decision_memory_id: MemoryId,
    title: &str,
    decision: &str,
    project_id: Option<ProjectId>,
) -> Envelope {
    envelope(
        AggregateType::Decision,
        legacy_decision_memory_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::DecisionRecorded(DecisionRecordedPayload {
            decision_id: legacy_decision_memory_id,
            title: title.into(),
            context: "ctx".into(),
            decision: decision.into(),
            consequences: "none".into(),
            project_id,
            session_id: None,
            tx_id: None,
        }),
    )
}

fn summary_event(memory_id: MemoryId, summary: &str, project_id: Option<ProjectId>) -> Envelope {
    summary_event_with_session(memory_id, summary, project_id, SessionId::new())
}

fn summary_event_with_session(
    memory_id: MemoryId,
    summary: &str,
    project_id: Option<ProjectId>,
    session_id: SessionId,
) -> Envelope {
    envelope(
        AggregateType::Session,
        session_id.as_uuid(),
        Privacy::LocalOnly,
        Payload::SessionSummaryCreated(SessionSummaryCreatedPayload {
            session_id,
            project_id,
            memory_id,
            summary: summary.into(),
        }),
    )
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

#[test]
fn classify__memory_pinned__evidence_not_conclusion() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(1));
    let events = vec![pin_event(
        mid,
        "CONSTRAINT: never mutate live vault",
        None,
        None,
    )];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let evidence: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Evidence)
        .collect();
    let conclusions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Conclusion)
        .collect();
    assert_eq!(evidence.len(), 1);
    assert_eq!(conclusions.len(), 0);
    assert_eq!(plan.totals.evidence, 1);
    assert_eq!(plan.totals.conclusion, 0);
    assert_eq!(evidence[0].mechanism, ImportMechanism::WouldAppend);
}

#[test]
fn classify__memory_synthesized__candidate_only() {
    let project = ProjectId::from_uuid(Uuid::from_u128(10));
    let src = MemoryId::from_uuid(Uuid::from_u128(2));
    let synth = MemoryId::from_uuid(Uuid::from_u128(3));
    let events = vec![
        pin_event(src, "source pin content", Some(project), None),
        synth_event(synth, "derived claim", project, vec![src]),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let conclusions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Conclusion)
        .collect();
    assert_eq!(conclusions.len(), 1);
    assert_eq!(conclusions[0].unsupported, Some(false));
    assert_eq!(conclusions[0].mechanism, ImportMechanism::WouldAppend);
    // No confirmed/approved kinds in plan.
    assert!(
        plan.actions
            .iter()
            .all(|a| a.kind != ImportActionKind::Skip || a.reason_code != "conclusion_confirmed")
    );
    assert_eq!(plan.totals.conclusion, 1);

    // Apply and assert projection state is Candidate (not Confirmed) — T167-R1-05.
    let (_t, ports) = open_ports();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let cid = legacy_conclusion_id(conclusions[0].original_event_id);
    let row = ports
        .query
        .get_conclusion(cid)
        .unwrap()
        .expect("conclusion projected");
    assert_eq!(row.state, "Candidate");
    assert!(!row.unsupported);
    assert_eq!(row.statement, "derived claim");
    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    assert!(
        !all.iter()
            .any(|e| e.event_type == EventKind::ConclusionConfirmed),
        "import must not emit ConclusionConfirmed"
    );
}

#[test]
fn classify__decision_recorded__proposed_plus_review() {
    let project = ProjectId::from_uuid(Uuid::from_u128(11));
    let leg = MemoryId::from_uuid(Uuid::from_u128(4));
    let events = vec![decision_event(
        leg,
        "Use ports",
        "CP uses ports",
        Some(project),
    )];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let decisions: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Decision)
        .collect();
    let reviews: Vec<_> = plan
        .actions
        .iter()
        .filter(|a| a.kind == ImportActionKind::Review)
        .collect();
    assert_eq!(decisions.len(), 1);
    assert_eq!(reviews.len(), 1);
    assert_eq!(plan.totals.decision, 1);
    assert_eq!(plan.totals.review, 1);
    // No DecisionApproved path.
    assert!(decisions.iter().all(|a| a.reason_code == "legacy_decision"));
}

#[test]
fn classify__forgotten_memory__excluded() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(5));
    let events = vec![pin_event(mid, "will forget", None, None), forget_event(mid)];
    let plan = classify_legacy(&events, &opts()).unwrap();
    assert_eq!(plan.totals.evidence, 0);
    let forgotten = plan
        .actions
        .iter()
        .find(|a| a.reason_code == "forgotten")
        .expect("forgotten skip");
    assert_eq!(forgotten.mechanism, ImportMechanism::Skip);
}

#[test]
fn classify__forgotten_then_restored__included() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(6));
    let events = vec![
        pin_event(mid, "restored pin", None, None),
        forget_event(mid),
        restore_event(mid),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    assert_eq!(plan.totals.evidence, 1);
    assert!(plan.actions.iter().any(
        |a| a.kind == ImportActionKind::Evidence && a.mechanism == ImportMechanism::WouldAppend
    ));
}

#[test]
fn classify__synth_referencing_forgotten_source__unsupported_true() {
    let project = ProjectId::from_uuid(Uuid::from_u128(12));
    let child = MemoryId::from_uuid(Uuid::from_u128(7));
    let synth = MemoryId::from_uuid(Uuid::from_u128(8));
    let events = vec![
        pin_event(child, "child", Some(project), None),
        forget_event(child),
        synth_event(synth, "parent synth", project, vec![child]),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let c = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Conclusion)
        .expect("conclusion");
    assert_eq!(c.unsupported, Some(true));
    assert_eq!(c.reason_code, "forgotten_source");
    assert!(c.evidence_ids.is_empty());
}

#[test]
fn classify__unknown_payload__unresolved() {
    let env = Envelope {
        event_id: Uuid::from_u128(100),
        schema_version: 1,
        aggregate_type: AggregateType::System,
        aggregate_id: Uuid::nil(),
        event_type: EventKind::Unknown("FutureThing".into()),
        occurred_at: time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
        actor: Actor::System,
        causation_id: None,
        correlation_id: None,
        privacy: Privacy::Sealed,
        payload: Payload::Unknown(serde_json::json!({"type": "FutureThing", "x": 1})),
        payload_hash: "deadbeef".into(),
    };
    let plan = classify_legacy(&[env], &opts()).unwrap();
    assert_eq!(plan.totals.unresolved, 1);
    assert!(
        plan.actions
            .iter()
            .any(|a| a.kind == ImportActionKind::Unresolved && a.reason_code == "unknown_payload")
    );
}

#[test]
fn classify__idempotent_ids__stable_v5() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(20));
    let e1 = pin_event(mid, "same", None, None);
    let e2 = pin_event(mid, "same", None, None);
    // Force same event_id for pure id derivation check via helpers.
    let id_a = legacy_evidence_id(Some(&mid), e1.event_id);
    let id_b = legacy_evidence_id(Some(&mid), e2.event_id);
    // memory_id preferred → same even if event_ids differ
    assert_eq!(id_a, id_b);

    let project = ProjectId::from_uuid(Uuid::from_u128(21));
    let synth_mid = MemoryId::from_uuid(Uuid::from_u128(22));
    let s = synth_event(synth_mid, "claim", project, vec![mid]);
    let c1 = legacy_conclusion_id(s.event_id);
    let c2 = legacy_conclusion_id(s.event_id);
    assert_eq!(c1, c2);

    let d = decision_event(
        MemoryId::from_uuid(Uuid::from_u128(23)),
        "t",
        "d",
        Some(project),
    );
    assert_eq!(
        legacy_decision_id(d.event_id),
        legacy_decision_id(d.event_id)
    );
    assert_eq!(legacy_review_id(d.event_id), legacy_review_id(d.event_id));
}

#[test]
fn classify__evidence_id_prefers_memory_id() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(30));
    let event_id = Uuid::from_u128(999);
    let from_memory = legacy_evidence_id(Some(&mid), event_id);
    let from_event = legacy_evidence_id(None, event_id);
    assert_ne!(from_memory, from_event);
    assert_eq!(
        from_memory,
        ai_brains_core::ids::EvidenceId::from_uuid(id_from_command(
            NS_LEGACY_EVIDENCE,
            &mid.to_string()
        ))
    );
    assert_eq!(
        from_event,
        ai_brains_core::ids::EvidenceId::from_uuid(id_from_command(
            NS_LEGACY_EVIDENCE,
            &event_id.to_string()
        ))
    );
}

#[test]
fn classify__decision_id_not_memory_id_cast() {
    let legacy_mem = MemoryId::from_uuid(Uuid::from_u128(40));
    let d = decision_event(legacy_mem, "title", "body", None);
    let governed = legacy_decision_id(d.event_id);
    // Must not equal MemoryId bytes cast as DecisionId.
    let cast = ai_brains_core::ids::DecisionId::from_uuid(legacy_mem.as_uuid());
    assert_ne!(governed, cast);
    assert_eq!(
        governed,
        ai_brains_core::ids::DecisionId::from_uuid(id_from_command(
            NS_LEGACY_DECISION,
            &d.event_id.to_string()
        ))
    );
}

#[test]
fn classify__missing_scope_without_default__skipped() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(50));
    let mut o = opts();
    o.default_scope = None;
    let events = vec![pin_event(mid, "no project no default", None, None)];
    let plan = classify_legacy(&events, &o).unwrap();
    assert_eq!(plan.totals.evidence, 0);
    assert!(
        plan.actions
            .iter()
            .any(|a| a.reason_code == "missing_scope" && a.mechanism == ImportMechanism::Skip)
    );
}

#[test]
fn classify__default_scope_fallback__used() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(51));
    let user = UserId::from_uuid(Uuid::from_u128(77));
    let mut o = opts();
    o.default_scope = Some(ScopeRef::Personal(user));
    let events = vec![pin_event(mid, "fallback scope", None, None)];
    let plan = classify_legacy(&events, &o).unwrap();
    assert_eq!(plan.totals.evidence, 1);
    let ev = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Evidence)
        .unwrap();
    assert_eq!(
        ev.scope_key.as_deref(),
        Some(format!("Personal:{user}").as_str())
    );
}

#[test]
fn classify__session_summary__evidence_digest() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(60));
    let project = ProjectId::from_uuid(Uuid::from_u128(61));
    let session = SessionId::from_uuid(Uuid::from_u128(62));
    let events = vec![summary_event_with_session(
        mid,
        "session digest summary",
        Some(project),
        session,
    )];
    let plan = classify_legacy(&events, &opts()).unwrap();
    assert_eq!(plan.totals.evidence, 1);
    assert_eq!(plan.totals.conclusion, 0);
    let ev = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Evidence)
        .unwrap();
    assert_eq!(ev.reason_code, "legacy_summary");
    assert_eq!(ev.content.as_deref(), Some("session digest summary"));
    // §5.1 / T167-R1-03: session_id on plan action metadata.
    assert_eq!(ev.session_id.as_deref(), Some(session.to_string().as_str()));

    // Apply: session_id durable in Evidence summary provenance.
    let (_t, ports) = open_ports();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let recorded = all
        .iter()
        .find(|e| e.event_type == EventKind::EvidenceRecorded)
        .expect("evidence recorded");
    match &recorded.payload {
        Payload::EvidenceRecorded(p) => {
            assert!(
                p.summary.contains(&format!("[session_id:{session}]")),
                "summary missing session provenance: {}",
                p.summary
            );
            assert!(p.summary.contains("session digest summary"));
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[test]
fn classify__preserves_source_tag_metadata() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(70));
    let events = vec![pin_event(
        mid,
        "tagged pin",
        None,
        Some("changeguard:symbol"),
    )];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let ev = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Evidence)
        .unwrap();
    assert_eq!(ev.source_tag.as_deref(), Some("changeguard:symbol"));

    // T167-R1-02: source_tag durable after apply (no changeguard→ledgerful rewrite).
    let (_t, ports) = open_ports();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let recorded = all
        .iter()
        .find(|e| e.event_type == EventKind::EvidenceRecorded)
        .expect("evidence recorded");
    match &recorded.payload {
        Payload::EvidenceRecorded(p) => {
            assert!(
                p.summary.contains("[source_tag:changeguard:symbol]"),
                "summary missing source_tag sidecar: {}",
                p.summary
            );
            assert!(p.summary.contains("tagged pin"));
            assert!(!p.summary.contains("ledgerful:symbol"));
            assert!(p.fingerprint.is_some());
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// plan_hash
// ---------------------------------------------------------------------------

#[test]
fn plan_hash__same_input_same_hash() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(80));
    let events = vec![pin_event(mid, "body", None, Some("tag"))];
    let p1 = classify_legacy(&events, &opts()).unwrap();
    let p2 = classify_legacy(&events, &opts()).unwrap();
    assert_eq!(p1.plan_hash, p2.plan_hash);
    assert!(!p1.plan_hash.is_empty());
}

#[test]
fn plan_hash__reordered_actions_same_hash() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(81));
    let events = vec![pin_event(mid, "body", None, None)];
    let mut plan = classify_legacy(&events, &opts()).unwrap();
    let h1 = compute_plan_hash(&plan.actions).unwrap();
    plan.actions.reverse();
    let h2 = compute_plan_hash(&plan.actions).unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn plan_hash__omits_body_plaintext() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(82));
    // Same memory_id / structure; different body only.
    let e1 = pin_event(mid, "body-alpha-plaintext-secret", None, Some("t"));
    let mut e2 = pin_event(mid, "body-beta-different-text", None, Some("t"));
    // Force identical event_ids so original_event_id keys match.
    e2.event_id = e1.event_id;
    let p1 = classify_legacy(&[e1], &opts()).unwrap();
    let p2 = classify_legacy(&[e2], &opts()).unwrap();
    assert_eq!(p1.plan_hash, p2.plan_hash);
    assert_ne!(
        p1.actions[0].content.as_deref(),
        p2.actions[0].content.as_deref()
    );
}

// ---------------------------------------------------------------------------
// Report / fixture
// ---------------------------------------------------------------------------

#[test]
fn report__no_full_plaintext_by_default() {
    let mid = MemoryId::from_uuid(Uuid::from_u128(90));
    let secret = "SUPER_SECRET_BODY_MUST_NOT_APPEAR";
    let events = vec![pin_event(mid, secret, None, None)];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let json = plan_report_json(&plan, false).unwrap();
    assert!(!json.contains(secret));
    assert!(json.contains("plan_hash"));
    assert!(json.contains("legacy_pin") || json.contains("evidence"));
}

#[test]
fn classify__fixture_legacy_v1__frozen_plan_totals() {
    let path = fixture_path("legacy-v1-events.ndjson");
    let raw = std::fs::read_to_string(&path).expect("fixture present");
    let mut events = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let env: Envelope = serde_json::from_str(line).expect("valid envelope");
        events.push(env);
    }
    // Fixture has project_id on pin — no default_scope needed for evidence.
    let mut o = opts();
    o.default_scope = None;
    let plan = classify_legacy(&events, &o).unwrap();
    // Frozen: 1 MemoryPinned → evidence; project/session/turns → skip; no unresolved.
    assert_eq!(plan.totals.evidence, 1, "fixture pin → evidence");
    assert_eq!(plan.totals.conclusion, 0);
    assert_eq!(plan.totals.decision, 0);
    assert_eq!(plan.totals.review, 0);
    assert_eq!(plan.totals.unresolved, 0);
    assert_eq!(plan.totals.skipped, 4, "project + session + 2 turns");
    assert!(!plan.plan_hash.is_empty());
}

fn fixture_path(name: &str) -> PathBuf {
    // workspace root relative to this test crate
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../fixtures/governed-memory");
    p.push(name);
    p
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

#[test]
fn apply__second_run__zero_new_aggregates() {
    let (_t, ports) = open_ports();
    let mid = MemoryId::from_uuid(Uuid::from_u128(100));
    let events = vec![pin_event(mid, "import me", None, None)];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let r1 = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    assert!(r1.applied >= 1);
    assert!(r1.legacy_import_applied);
    assert_eq!(r1.source_registered, 1);

    let r2 = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    // Second run: evidence already present → already_imported; may re-register source? No — get_source hits.
    assert_eq!(r2.source_registered, 0);
    assert!(r2.already_imported >= 1);
    assert_eq!(r2.applied, 0);
}

#[test]
fn apply__ensures_legacy_source_once() {
    let (_t, ports) = open_ports();
    let m1 = MemoryId::from_uuid(Uuid::from_u128(110));
    let m2 = MemoryId::from_uuid(Uuid::from_u128(111));
    let events = vec![
        pin_event(m1, "pin one", None, None),
        pin_event(m2, "pin two", None, None),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let r = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    assert_eq!(r.source_registered, 1);
    let sid = legacy_source_id();
    let row = ports.query.get_source(sid).unwrap().expect("source row");
    assert!(row.kind.contains("LegacyAiBrains") || row.display_name.contains("Legacy"));
}

#[test]
fn apply__appends_legacy_import_applied() {
    let (_t, ports) = open_ports();
    let mid = MemoryId::from_uuid(Uuid::from_u128(120));
    let events = vec![pin_event(mid, "audit me", None, None)];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let r = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    assert!(r.legacy_import_applied);
    assert_eq!(r.plan_hash, plan.plan_hash);

    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let audit: Vec<_> = all
        .iter()
        .filter(|e| e.event_type == EventKind::LegacyImportApplied)
        .collect();
    assert_eq!(audit.len(), 1);
    match &audit[0].payload {
        Payload::LegacyImportApplied(p) => {
            assert_eq!(p.plan_hash, plan.plan_hash);
            assert!(p.sample_ids.iter().all(|s| !s.contains("audit me")));
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[test]
fn apply__does_not_call_observe_source() {
    let (_t, ports) = open_ports();
    let mid = MemoryId::from_uuid(Uuid::from_u128(130));
    let events = vec![pin_event(mid, "no observe", None, None)];
    let plan = classify_legacy(&events, &opts()).unwrap();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();

    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    assert!(
        !all.iter()
            .any(|e| e.event_type == EventKind::SourceVersionRecorded),
        "import must not emit SourceVersionRecorded (observe_source side-effect)"
    );
    assert!(
        !all.iter()
            .any(|e| e.event_type == EventKind::SourceObserved),
        "import must not emit SourceObserved"
    );
    // SourceRegistered is expected once (ensure-source), not via observe.
    assert_eq!(
        all.iter()
            .filter(|e| e.event_type == EventKind::SourceRegistered)
            .count(),
        1
    );
}

#[test]
fn apply__without_confirm__refuses() {
    let (_t, ports) = open_ports();
    let mid = MemoryId::from_uuid(Uuid::from_u128(140));
    let plan = classify_legacy(&[pin_event(mid, "x", None, None)], &opts()).unwrap();
    let err = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: false },
    )
    .unwrap_err();
    assert!(err.to_string().contains("confirm"));
}

#[test]
fn apply__decision_and_review__projected() {
    let (_t, ports) = open_ports();
    let project = ProjectId::from_uuid(Uuid::from_u128(150));
    let leg = MemoryId::from_uuid(Uuid::from_u128(151));
    let events = vec![decision_event(
        leg,
        "Ship T167",
        "Import under-promotes",
        Some(project),
    )];
    let plan = classify_legacy(&events, &opts()).unwrap();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let did = legacy_decision_id(
        plan.actions
            .iter()
            .find(|a| a.kind == ImportActionKind::Decision)
            .unwrap()
            .original_event_id,
    );
    let row = ports.query.get_decision(did).unwrap().expect("decision");
    assert_eq!(row.state, "Proposed");
    let rid = legacy_review_id(
        plan.actions
            .iter()
            .find(|a| a.kind == ImportActionKind::Review)
            .unwrap()
            .original_event_id,
    );
    let rev = ports.query.get_review_item(rid).unwrap().expect("review");
    assert_eq!(
        rev.related_decision_id.as_deref(),
        Some(did.to_string().as_str())
    );
    assert!(
        rev.status.eq_ignore_ascii_case("open") || rev.status.eq_ignore_ascii_case("Open"),
        "review status={}",
        rev.status
    );
}

#[test]
fn apply__include_truncated_summaries__still_full_content() {
    // T167-R1-01: flag must not truncate apply bodies / fingerprints.
    let mid = MemoryId::from_uuid(Uuid::from_u128(160));
    let long_body = "A".repeat(200) + "FULL_PIN_TAIL_MARKER";
    let mut o = opts();
    o.include_truncated_summaries = true;
    let events = vec![pin_event(mid, &long_body, None, None)];
    let plan = classify_legacy(&events, &o).unwrap();
    let ev = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Evidence)
        .expect("evidence action");
    assert_eq!(
        ev.content.as_deref(),
        Some(long_body.as_str()),
        "ImportAction.content must stay full when include_truncated_summaries=true"
    );

    // Report may truncate; full body must not appear when flag drives report path.
    let report_json = plan_report_json(&plan, true).unwrap();
    assert!(report_json.contains("truncated_summary"));
    assert!(
        !report_json.contains("FULL_PIN_TAIL_MARKER"),
        "report truncated_summary must not include full tail"
    );

    let (_t, ports) = open_ports();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let recorded = all
        .iter()
        .find(|e| e.event_type == EventKind::EvidenceRecorded)
        .expect("evidence");
    match &recorded.payload {
        Payload::EvidenceRecorded(p) => {
            assert!(
                p.summary.contains("FULL_PIN_TAIL_MARKER"),
                "applied summary must use full content"
            );
            assert_eq!(p.summary.len(), long_body.len());
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn apply__duplicate_evidence_id_in_plan__already_imported() {
    // T167-R1-04: same derived EvidenceId twice in one plan must not double-append.
    let mid = MemoryId::from_uuid(Uuid::from_u128(170));
    let events = vec![pin_event(mid, "shared memory pin", None, None)];
    let mut plan = classify_legacy(&events, &opts()).unwrap();
    let first = plan
        .actions
        .iter()
        .find(|a| a.kind == ImportActionKind::Evidence)
        .cloned()
        .expect("evidence");
    // Inject a second WouldAppend with the same derived_id (simulates pin+summary collision
    // that slipped past classify, or a hand-built plan).
    let mut dup = first.clone();
    dup.original_event_id = Uuid::from_u128(171);
    plan.actions.push(dup);
    plan.totals.evidence += 1;

    let (_t, ports) = open_ports();
    let report = apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    assert_eq!(report.applied, 1, "only one new evidence append");
    assert!(
        report.already_imported >= 1,
        "second same id → already_imported"
    );

    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let evidence_count = all
        .iter()
        .filter(|e| e.event_type == EventKind::EvidenceRecorded)
        .count();
    assert_eq!(evidence_count, 1, "must not double-append EvidenceRecorded");
}

#[test]
fn classify__pin_and_summary_same_memory__one_evidence() {
    // Classify collapse: one memory_id → one Evidence WouldAppend.
    let mid = MemoryId::from_uuid(Uuid::from_u128(180));
    let project = ProjectId::from_uuid(Uuid::from_u128(181));
    let events = vec![
        pin_event(mid, "pin body", Some(project), None),
        summary_event(mid, "summary body", Some(project)),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    let would = plan
        .actions
        .iter()
        .filter(|a| {
            a.kind == ImportActionKind::Evidence && a.mechanism == ImportMechanism::WouldAppend
        })
        .count();
    assert_eq!(would, 1);
    assert_eq!(plan.totals.evidence, 1);
    assert!(
        plan.actions
            .iter()
            .any(|a| a.reason_code == "already_imported" && a.mechanism == ImportMechanism::Skip),
        "second memory_id hit should skip as already_imported"
    );
}

#[test]
fn apply__legacy_import_applied__evidence_count_matches_plan() {
    // T167-R1-07: audit evidence_count aligned with plan totals.
    let project = ProjectId::from_uuid(Uuid::from_u128(190));
    let m1 = MemoryId::from_uuid(Uuid::from_u128(191));
    let m2 = MemoryId::from_uuid(Uuid::from_u128(192));
    let synth = MemoryId::from_uuid(Uuid::from_u128(193));
    let events = vec![
        pin_event(m1, "pin-a", Some(project), None),
        pin_event(m2, "pin-b", Some(project), None),
        synth_event(synth, "claim", project, vec![m1, m2]),
    ];
    let plan = classify_legacy(&events, &opts()).unwrap();
    assert_eq!(plan.totals.evidence, 2);
    assert_eq!(plan.totals.conclusion, 1);

    let (_t, ports) = open_ports();
    apply_legacy_import(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &plan,
        &ApplyOpts { confirm: true },
    )
    .unwrap();
    let store = ports.writer.store();
    let all = store.read_all_events().unwrap();
    let audit = all
        .iter()
        .find(|e| e.event_type == EventKind::LegacyImportApplied)
        .expect("audit event");
    match &audit.payload {
        Payload::LegacyImportApplied(p) => {
            assert_eq!(p.evidence_count, plan.totals.evidence);
            assert_eq!(p.conclusion_count, plan.totals.conclusion);
            assert_eq!(p.decision_count, plan.totals.decision);
            assert_eq!(p.review_count, plan.totals.review);
        }
        other => panic!("unexpected: {other:?}"),
    }
}
