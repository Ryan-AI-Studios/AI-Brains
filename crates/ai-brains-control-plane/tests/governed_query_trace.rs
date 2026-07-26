#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T152 Phase E — progressive query + traces.

use ai_brains_control_plane::{
    Clock, ExpandHandleRequest, GetQueryTraceRequest, ProgressiveQueryRequest,
    ProposeConclusionRequest, ProposeDecisionRequest, StoreEventWriter, StorePorts, SystemClock,
    activate_conclusion, approve_decision, expand_handle, get_query_trace, issue_grant,
    make_principal, progressive_query, propose_conclusion, propose_decision, register_principal,
    try_mark_stale_payload,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;

fn open_ports() -> (tempfile::NamedTempFile, StorePorts) {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
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

fn grant_all_reads(ports: &StorePorts, principal: PrincipalId, scope: ScopeRef) {
    let clock = SystemClock;
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
        GrantCapability::ReadEvidence,
        GrantCapability::ProposeConclusion,
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            principal,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
}

#[test]
fn progressive_query__returns_handles_freshness_trace_id() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();

    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            title: "Authority first".into(),
            statement: "Rank by authority before vectors".into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![EvidenceId::new()]),
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
        &clock,
        &policy,
        &human_p,
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "Deterministic briefings are required".into(),
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
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Stale conclusion should not appear as current truth.
    let stale = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "Stale authority claim about vectors".into(),
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
        &clock,
        &policy,
        &human_p,
        stale.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let stale_payload =
        try_mark_stale_payload(stale.conclusion_id, None, Some("source rotated".into())).unwrap();
    let env = EventBuilder::new(
        AggregateType::Conclusion,
        stale.conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionMarkedStale(stale_payload))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[env]).unwrap();

    let store = ports.store();
    // dry_run: no side effects (no projection, no event).
    let dry = progressive_query(
        None::<&StoreEventWriter>,
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            query: "authority deterministic".into(),
            privacy: Privacy::LocalOnly,
            limit: 10,
            dry_run: true,
            at: None,
        },
    )
    .unwrap();
    assert!(
        get_query_trace(
            &store,
            &policy,
            GetQueryTraceRequest {
                principal: human_p.clone(),
                privacy: Privacy::LocalOnly,
                trace_id: dry.query_trace_id.clone(),
            },
        )
        .unwrap()
        .is_none(),
        "dry_run must not write query_trace_projection"
    );

    // Non-dry-run: event-only persist; projection applied via append pipeline.
    let resp = progressive_query(
        Some(&ports.writer),
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            query: "authority deterministic".into(),
            privacy: Privacy::LocalOnly,
            limit: 10,
            dry_run: false,
            at: None,
        },
    )
    .unwrap();

    assert!(!resp.denied);
    assert!(!resp.query_trace_id.is_empty());
    assert!(!resp.results.is_empty());
    assert!(resp.results.iter().all(|r| r.state != "Stale"));
    assert!(
        !resp
            .results
            .iter()
            .any(|r| r.id == stale.conclusion_id.to_string())
    );
    // Decision should rank above conclusion (authority).
    if resp.results.len() >= 2 {
        let decision_idx = resp.results.iter().position(|r| r.kind == "Decision");
        let conclusion_idx = resp.results.iter().position(|r| r.kind == "Conclusion");
        if let (Some(di), Some(ci)) = (decision_idx, conclusion_idx) {
            assert!(di < ci, "decision should rank before conclusion");
        }
    }
    let freshness = resp.freshness_summary.as_ref().expect("freshness summary");
    assert!(
        freshness.stale_count >= 1,
        "stale in-scope must count even when excluded from hits: {:?}",
        freshness
    );
    assert_eq!(freshness.worst_state, "Stale");
    for hit in &resp.results {
        assert!(hit.ranking.authority > 0);
    }

    let trace = get_query_trace(
        &store,
        &policy,
        GetQueryTraceRequest {
            principal: human_p.clone(),
            privacy: Privacy::LocalOnly,
            trace_id: resp.query_trace_id.clone(),
        },
    )
    .unwrap()
    .expect("trace must be persisted via event projection");
    assert_eq!(trace.query_trace_id, resp.query_trace_id);
    assert!(trace.query.contains("authority"));
    assert_eq!(trace.scope, format!("Repository:{project}"));
    assert_eq!(trace.principal, human_p.id.to_string());
    assert_eq!(trace.applied_policy, "DefaultPolicyEvaluator");
    assert!(
        trace.ranking_json.get("order").is_some() || trace.ranking_json.get("hits").is_some(),
        "ranking_json must rehydrate: {}",
        trace.ranking_json
    );

    // Expand decision handle (decision id as handle).
    let preview = expand_handle(
        &ports.query,
        &store,
        &policy,
        ExpandHandleRequest {
            principal: human_p,
            scope,
            handle_id: dec.decision_id.to_string(),
            privacy: Privacy::LocalOnly,
            max_chars: 64,
        },
    )
    .unwrap();
    assert!(preview.kind.starts_with("Decision"));
    assert!(!preview.preview.is_empty());
    // Bounded — not a full raw dump of unrelated content.
    assert!(preview.preview.chars().count() <= 64 || !preview.truncated);
}

#[test]
fn progressive_query__stale_not_current_truth() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();

    let stale = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "only stale match for unique_token_xyz".into(),
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
        &clock,
        &policy,
        &human_p,
        stale.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let stale_payload =
        try_mark_stale_payload(stale.conclusion_id, None, Some("gone".into())).unwrap();
    let env = EventBuilder::new(
        AggregateType::Conclusion,
        stale.conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionMarkedStale(stale_payload))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[env]).unwrap();

    let store = ports.store();
    let resp = progressive_query(
        None::<&StoreEventWriter>,
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p,
            scope,
            query: "unique_token_xyz".into(),
            privacy: Privacy::LocalOnly,
            limit: 10,
            dry_run: true,
            at: None,
        },
    )
    .unwrap();

    assert!(
        resp.results.is_empty() || resp.results.iter().all(|r| r.state != "Stale"),
        "stale must not be current truth: {:?}",
        resp.results
    );
    assert!(!resp.results.iter().any(|r| r.state == "Stale"));
    let freshness = resp.freshness_summary.as_ref().expect("freshness");
    assert!(
        freshness.stale_count >= 1,
        "stale excluded from hits still counted in freshness: {:?}",
        freshness
    );
}

#[test]
fn progressive_query__non_dry_run__rebuild_preserves_ranking_scope_principal() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "rebuild_trace_token survives projection rebuild".into(),
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
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let mut store = ports.store();
    let resp = progressive_query(
        Some(&ports.writer),
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            query: "rebuild_trace_token".into(),
            privacy: Privacy::LocalOnly,
            limit: 10,
            dry_run: false,
            at: None,
        },
    )
    .unwrap();
    assert!(!resp.results.is_empty());

    let before = get_query_trace(
        &store,
        &policy,
        GetQueryTraceRequest {
            principal: human_p.clone(),
            privacy: Privacy::LocalOnly,
            trace_id: resp.query_trace_id.clone(),
        },
    )
    .unwrap()
    .expect("trace before rebuild");
    assert_eq!(before.scope, format!("Repository:{project}"));
    assert_eq!(before.principal, human_p.id.to_string());
    assert!(before.ranking_json.get("hits").is_some());

    store.rebuild_projections().unwrap();

    let after = get_query_trace(
        &store,
        &policy,
        GetQueryTraceRequest {
            principal: human_p.clone(),
            privacy: Privacy::LocalOnly,
            trace_id: resp.query_trace_id.clone(),
        },
    )
    .unwrap()
    .expect("trace after rebuild");
    assert_eq!(after.scope, before.scope);
    assert_eq!(after.principal, before.principal);
    assert_eq!(after.applied_policy, before.applied_policy);
    assert_eq!(after.ranking_json, before.ranking_json);
    assert_eq!(after.query, before.query);
}

#[test]
fn progressive_query__future_and_expired_claims__excluded_from_results() {
    use time::Duration;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();
    let now = clock.now().unwrap();

    // Future conclusion (valid_from in the future).
    let future = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "future_claim_token will start later".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now + Duration::days(30)),
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        future.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Expired conclusion (valid_until already passed).
    let expired = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "expired_claim_token already ended".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now - Duration::days(60)),
            valid_until: Some(now - Duration::days(1)),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        expired.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // In-window conclusion for control.
    let current = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "current_claim_token is valid now".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now - Duration::days(1)),
            valid_until: Some(now + Duration::days(30)),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        current.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let store = ports.store();
    let resp = progressive_query(
        None::<&StoreEventWriter>,
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p,
            scope,
            query: "claim_token".into(),
            privacy: Privacy::LocalOnly,
            limit: 20,
            dry_run: true,
            at: Some(now),
        },
    )
    .unwrap();

    assert!(
        !resp
            .results
            .iter()
            .any(|r| r.id == future.conclusion_id.to_string()),
        "future claims must not appear in progressive results"
    );
    assert!(
        !resp
            .results
            .iter()
            .any(|r| r.id == expired.conclusion_id.to_string()),
        "expired claims must not appear in progressive results"
    );
    assert!(
        resp.results
            .iter()
            .any(|r| r.id == current.conclusion_id.to_string()),
        "in-window claims must appear: {:?}",
        resp.results
    );
}

#[test]
fn expand_handle__cross_scope__denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let scope_a = ScopeRef::Repository(project_a);
    let scope_b = ScopeRef::Repository(project_b);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope_a.clone());
    grant_all_reads(&ports, human_p.id, scope_b.clone());
    let policy = ports.production_policy();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope_a.clone(),
            statement: "owned by scope A".into(),
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
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let store = ports.store();
    // Request expand under scope B for a handle owned by scope A.
    let preview = expand_handle(
        &ports.query,
        &store,
        &policy,
        ExpandHandleRequest {
            principal: human_p,
            scope: scope_b,
            handle_id: conc.conclusion_id.to_string(),
            privacy: Privacy::LocalOnly,
            max_chars: 128,
        },
    )
    .unwrap();
    assert_eq!(preview.kind, "Denied");
    assert!(preview.preview.is_empty());
}

#[test]
fn expand_handle__decisions_only_grant__cannot_expand_conclusion() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    // Seed conclusion under AllowAll.
    use ai_brains_control_plane::AllowAllPolicy;
    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "conclusion body for capability gate".into(),
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
        &clock,
        &AllowAllPolicy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Principal has only ReadDecisions (not ReadConclusions / ReadEvidence).
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        scope.clone(),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();
    let policy = ports.production_policy();
    let store = ports.store();
    let preview = expand_handle(
        &ports.query,
        &store,
        &policy,
        ExpandHandleRequest {
            principal: human_p,
            scope,
            handle_id: conc.conclusion_id.to_string(),
            privacy: Privacy::LocalOnly,
            max_chars: 128,
        },
    )
    .unwrap();
    assert_eq!(
        preview.kind, "Denied",
        "ReadDecisions alone must not expand conclusion handles"
    );
}

#[test]
fn get_query_trace__cross_principal__denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let owner = human();
    let other = make_principal(PrincipalKind::Agent, PrincipalId::new(), "other-agent");
    register_principal(&ports.writer, &clock, &owner).unwrap();
    register_principal(&ports.writer, &clock, &other).unwrap();
    grant_all_reads(&ports, owner.id, scope.clone());
    grant_all_reads(&ports, other.id, scope.clone());
    let policy = ports.production_policy();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: owner.clone(),
            scope: scope.clone(),
            statement: "trace ownership check token".into(),
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
        &clock,
        &policy,
        &owner,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let store = ports.store();
    let resp = progressive_query(
        Some(&ports.writer),
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: owner.clone(),
            scope,
            query: "ownership".into(),
            privacy: Privacy::LocalOnly,
            limit: 10,
            dry_run: false,
            at: None,
        },
    )
    .unwrap();

    // Owner can read.
    assert!(
        get_query_trace(
            &store,
            &policy,
            GetQueryTraceRequest {
                principal: owner,
                privacy: Privacy::LocalOnly,
                trace_id: resp.query_trace_id.clone(),
            },
        )
        .unwrap()
        .is_some()
    );
    // Cross-principal denied (None, not error leak).
    assert!(
        get_query_trace(
            &store,
            &policy,
            GetQueryTraceRequest {
                principal: other,
                privacy: Privacy::LocalOnly,
                trace_id: resp.query_trace_id,
            },
        )
        .unwrap()
        .is_none(),
        "cross-principal must not read another principal's query trace"
    );
}

/// T152-FRESH-P2: Active conclusions with zero evidence handles are not authoritative hits.
#[test]
fn progressive_query__active_without_evidence__not_in_hits() {
    use ai_brains_control_plane::AllowAllPolicy;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_all_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();

    // Propose with empty evidence via AllowAll (production path rejects empty evidence).
    let bare = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "unsupported_active_token without evidence".into(),
            evidence_ids: vec![],
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
        &clock,
        &AllowAllPolicy,
        &human_p,
        bare.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Control: supported Active conclusion.
    let supported = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "supported_active_token with evidence".into(),
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
        &clock,
        &policy,
        &human_p,
        supported.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let store = ports.store();
    let resp = progressive_query(
        None::<&StoreEventWriter>,
        &ports.query,
        &store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal: human_p,
            scope,
            query: "active_token".into(),
            privacy: Privacy::LocalOnly,
            limit: 20,
            dry_run: true,
            at: None,
        },
    )
    .unwrap();

    assert!(
        !resp
            .results
            .iter()
            .any(|h| h.id == bare.conclusion_id.to_string()),
        "Active with zero evidence handles must not be an authoritative hit"
    );
    assert!(
        resp.results
            .iter()
            .any(|h| h.id == supported.conclusion_id.to_string()),
        "supported Active conclusion must still rank"
    );
    for hit in &resp.results {
        if hit.kind == "Conclusion" && hit.state == "Active" {
            assert!(
                !hit.evidence_handles.is_empty(),
                "Active hits must carry evidence handles"
            );
        }
    }
}
