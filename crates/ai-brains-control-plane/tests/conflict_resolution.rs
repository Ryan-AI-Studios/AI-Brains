#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    AllowAllPolicy, GovernedQueryStore, OpenClaimConflictRequest, ProposeConclusionRequest,
    ProposeDecisionRequest, StorePorts, SystemClock, activate_conclusion, approve_decision,
    current_successor, equal_authority_conflict, make_principal, open_claim_conflict,
    prefer_decision_over_candidate, propose_conclusion, propose_decision, resolve_claim_conflict,
    resolve_scope_preference, select_conclusions_valid_at,
};
use ai_brains_core::ids::{ConclusionId, EvidenceId, PrincipalId, ProjectId, UserId, WorkspaceId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::ConclusionProposedPayload;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use tempfile::NamedTempFile;
use time::OffsetDateTime;

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

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent")
}
fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

/// Append a ConclusionProposed with an explicit envelope.occurred_at (recorded time).
fn append_conclusion_with_recorded_time(
    store: &SqliteEventStore,
    conclusion_id: ConclusionId,
    statement: &str,
    scope: &str,
    valid_from: OffsetDateTime,
    valid_until: Option<OffsetDateTime>,
    occurred_at: OffsetDateTime,
) {
    let mut env = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id,
        statement: statement.into(),
        evidence_ids: vec![EvidenceId::new()],
        proposer: PrincipalId::new(),
        valid_from: Some(valid_from),
        valid_until,
        scope: scope.into(),
        protected_category: None,
        unsupported: false,
        model_provenance: None,
    }))
    .unwrap();
    env.occurred_at = occurred_at;
    store.append_event(&env).unwrap();
}

/// Scenario 1: bitemporal valid time — selection uses valid_from/until, not occurred_at.
/// Control vault B holds identical valid windows with **swapped** recorded times.
#[test]
fn conflict_scenario1__valid_time_selection_ignores_occurred_at_swap() {
    // --- Vault A: primary pair via propose (system clock recorded times) ---
    let (_t_a, ports_a) = open_ports();
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let statement = "API is v2";

    // Claim A valid [T0, T1), Claim B valid [T1, +∞)
    let t0 = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();
    let t1 = OffsetDateTime::from_unix_timestamp(1_650_000_000).unwrap();
    let t_mid = OffsetDateTime::from_unix_timestamp(1_620_000_000).unwrap();
    let t_late = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    let a = propose_conclusion(
        &ports_a.writer,
        &ports_a.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: statement.into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(t0),
            valid_until: Some(t1),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let b = propose_conclusion(
        &ports_a.writer,
        &ports_a.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: statement.into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(t1),
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();

    let at_mid =
        select_conclusions_valid_at(&ports_a.query, &scope_key, Some(statement), t_mid).unwrap();
    assert_eq!(at_mid.len(), 1);
    assert_eq!(at_mid[0].id, a.conclusion_id);

    let at_late =
        select_conclusions_valid_at(&ports_a.query, &scope_key, Some(statement), t_late).unwrap();
    assert_eq!(at_late.len(), 1);
    assert_eq!(at_late[0].id, b.conclusion_id);
    assert_ne!(at_mid[0].valid_from, at_mid[0].recorded_at);

    // --- Vault B: same valid windows, swapped/reversed recorded times ---
    // Claim with valid [t0,t1) gets LATER occurred_at than claim with valid [t1,∞).
    let (_t_b, ports_b) = open_ports();
    let id_early_window = ConclusionId::new();
    let id_late_window = ConclusionId::new();
    let recorded_early = OffsetDateTime::from_unix_timestamp(1_500_000_000).unwrap();
    let recorded_late = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
    assert!(recorded_late > recorded_early);
    assert_ne!(recorded_early, t0);
    assert_ne!(recorded_late, t1);

    // Early domain window, late recorded time
    append_conclusion_with_recorded_time(
        ports_b.writer.store(),
        id_early_window,
        statement,
        &scope_key,
        t0,
        Some(t1),
        recorded_late,
    );
    // Late domain window, early recorded time
    append_conclusion_with_recorded_time(
        ports_b.writer.store(),
        id_late_window,
        statement,
        &scope_key,
        t1,
        None,
        recorded_early,
    );

    let mid_b =
        select_conclusions_valid_at(&ports_b.query, &scope_key, Some(statement), t_mid).unwrap();
    assert_eq!(mid_b.len(), 1, "selection must be unique at t_mid");
    assert_eq!(
        mid_b[0].id, id_early_window,
        "valid-window [t0,t1) must win at t_mid even when its occurred_at is later"
    );
    assert_eq!(mid_b[0].valid_from, t0);
    assert_eq!(mid_b[0].recorded_at, recorded_late);
    assert_ne!(mid_b[0].valid_from, mid_b[0].recorded_at);

    let late_b =
        select_conclusions_valid_at(&ports_b.query, &scope_key, Some(statement), t_late).unwrap();
    assert_eq!(late_b.len(), 1);
    assert_eq!(
        late_b[0].id, id_late_window,
        "valid-window [t1,∞) must win at t_late even when its occurred_at is earlier"
    );
    assert_eq!(late_b[0].recorded_at, recorded_early);
}

/// Scenario 2: repository vs workspace — repository wins only in repo context.
#[test]
fn conflict_scenario2__repository_wins_only_in_repo_context() {
    let (_t, ports) = open_ports();
    let project = ProjectId::new();
    let workspace = WorkspaceId::new();
    let repo_scope = ScopeRef::Repository(project);
    let ws_scope = ScopeRef::Workspace(workspace);
    let statement = "lint rules";

    let repo = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: repo_scope.clone(),
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
    let ws = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ws_scope.clone(),
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

    let repo_row = ports
        .query
        .get_conclusion(repo.conclusion_id)
        .unwrap()
        .unwrap();
    let ws_row = ports
        .query
        .get_conclusion(ws.conclusion_id)
        .unwrap()
        .unwrap();
    let candidates = vec![repo_row.clone(), ws_row.clone()];

    let in_repo = resolve_scope_preference(
        &candidates,
        &ai_brains_control_plane::scope_identity_key(&repo_scope),
    )
    .unwrap();
    assert_eq!(in_repo.id, repo.conclusion_id);

    let in_ws = resolve_scope_preference(
        &candidates,
        &ai_brains_control_plane::scope_identity_key(&ws_scope),
    )
    .unwrap();
    assert_eq!(in_ws.id, ws.conclusion_id);
}

/// Scenario 3: approved decision beats agent candidate; candidate still listed.
#[test]
fn conflict_scenario3__decision_beats_candidate_still_listed() {
    let (_t, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::new());
    let cand = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: "use postgres".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: human(),
            scope: scope.clone(),
            title: "DB choice".into(),
            statement: "use sqlite".into(),
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
        &human(),
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let decisions = ports.query.list_decisions(None, Some("Approved")).unwrap();
    let conclusions = ports
        .query
        .list_conclusions_by_scope_state(None, Some("Candidate"))
        .unwrap();
    let (chosen, remaining) = prefer_decision_over_candidate(&decisions, &conclusions);
    assert!(chosen.is_some());
    assert_eq!(chosen.unwrap().id, dec.decision_id);
    assert!(remaining.iter().any(|c| c.id == cand.conclusion_id));
}

/// Scenario 4: equal authority incompatible claims → unresolved conflict (not merged prose).
#[test]
fn conflict_scenario4__equal_authority_opens_conflict_not_merge() {
    let (_t, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::new());
    let scope_key = ai_brains_control_plane::scope_identity_key(&scope);
    let a = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: "deploy friday".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let b = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: scope.clone(),
            statement: "do not deploy friday".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    let ra = ports
        .query
        .get_conclusion(a.conclusion_id)
        .unwrap()
        .unwrap();
    let rb = ports
        .query
        .get_conclusion(b.conclusion_id)
        .unwrap()
        .unwrap();
    assert_ne!(
        ra.statement, rb.statement,
        "scenario requires incompatible statements"
    );
    let meta = equal_authority_conflict(&ra, &rb).expect("incompatible equal-authority conflict");
    assert_ne!(meta.0, meta.1);
    assert!(
        meta.2.to_ascii_lowercase().contains("equal-authority"),
        "explanation was: {:?}",
        meta.2
    );
    // Resolution descriptor is not a merge of the two statements.
    assert!(
        !meta.2.contains("deploy friday") || !meta.2.contains("do not deploy friday"),
        "must not merge both claim texts into explanation: {}",
        meta.2
    );

    let conflict_id = open_claim_conflict(
        &ports.writer,
        OpenClaimConflictRequest {
            claim_a_kind: "Conclusion".into(),
            claim_a_id: a.conclusion_id.to_string(),
            claim_b_kind: "Conclusion".into(),
            claim_b_id: b.conclusion_id.to_string(),
            scope: scope_key,
            explanation: meta.2,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            conflict_id: None,
        },
    )
    .unwrap();
    let open = ports.query.list_open_claim_conflicts().unwrap();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].id, conflict_id);
    assert_eq!(open[0].status, "Open");
    // Not a merged prose claim — two distinct claim ids.
    assert_ne!(open[0].claim_a_id, open[0].claim_b_id);
    assert_eq!(open[0].claim_a_id, a.conclusion_id.to_string());
    assert_eq!(open[0].claim_b_id, b.conclusion_id.to_string());

    resolve_claim_conflict(
        &ports.writer,
        &ports.query,
        &human(),
        conflict_id,
        "prefer A after review",
        Privacy::LocalOnly,
    )
    .unwrap();
    let resolved = ports
        .query
        .get_claim_conflict(conflict_id)
        .unwrap()
        .unwrap();
    assert_eq!(resolved.status, "Resolved");
    assert!(
        !resolved
            .resolution
            .as_deref()
            .unwrap_or("")
            .contains("do not deploy friday")
            || !resolved
                .resolution
                .as_deref()
                .unwrap_or("")
                .contains("deploy friday"),
        "resolution must not be a merge of both statements: {:?}",
        resolved.resolution
    );
}

/// Scenario 5: superseded historical vs current successor (activate then correct).
#[test]
fn conflict_scenario5__superseded_historical_vs_current_successor() {
    use ai_brains_control_plane::correct_conclusion;
    let (_t, ports) = open_ports();
    let old = propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: agent(),
            scope: ScopeRef::Personal(UserId::new()),
            statement: "historical claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    // Correct requires Active/Confirmed/Stale/Disputed — activate Candidate first.
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &agent(),
        old.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let new_id = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &human(),
        old.conclusion_id,
        "current claim".into(),
        vec![EvidenceId::new()],
        "supersede with better evidence",
        Privacy::LocalOnly,
    )
    .unwrap();
    let hist = ports
        .query
        .get_conclusion(old.conclusion_id)
        .unwrap()
        .unwrap();
    assert_eq!(hist.state, "Superseded");
    assert_eq!(current_successor(&hist), Some(new_id.to_string().as_str()));
    let current = ports.query.get_conclusion(new_id).unwrap().unwrap();
    assert_eq!(current.state, "Candidate");
    assert_eq!(current.statement, "current claim");
    assert_ne!(current.id, old.conclusion_id);
}
