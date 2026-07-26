#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! Soft perf harness for T152 briefings (not a hard CI gate).
//!
//! Run manually:
//! `cargo nextest run -p ai-brains-control-plane --test briefing_perf_harness --run-ignored all`

use ai_brains_control_plane::{
    BudgetConfig, ProjectBriefingRequest, ProposeConclusionRequest, ProposeDecisionRequest,
    StoreEventWriter, StorePorts, SystemClock, activate_conclusion, approve_decision,
    build_project_briefing, issue_grant, make_principal, propose_conclusion, propose_decision,
    register_principal, render_project_markdown,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use std::path::PathBuf;
use std::time::Instant;

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

#[test]
#[ignore = "slow bench; owner: t152"]
fn project_briefing_perf__synthetic_fixture__soft_thresholds() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "human");
    register_principal(&ports.writer, &clock, &human).unwrap();
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    let policy = ports.production_policy();

    // Seed a modest synthetic fixture.
    for i in 0..40 {
        let d = propose_decision(
            &ports.writer,
            &ports.query,
            &clock,
            &policy,
            ProposeDecisionRequest {
                principal: human.clone(),
                scope: scope.clone(),
                title: format!("D{i}"),
                statement: format!("decision body {i} for synthetic briefing fixture"),
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
            &human,
            d.decision_id,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    for i in 0..40 {
        let c = propose_conclusion(
            &ports.writer,
            &ports.query,
            &clock,
            &policy,
            ProposeConclusionRequest {
                principal: human.clone(),
                scope: scope.clone(),
                statement: format!("active conclusion {i} synthetic fixture body"),
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
            &human,
            c.conclusion_id,
            Privacy::LocalOnly,
        )
        .unwrap();
    }

    let identity = ports.identity_store();
    let req = ProjectBriefingRequest {
        principal: human,
        resolve: ai_brains_control_plane::ScopeResolveInput {
            cwd: PathBuf::from("."),
            explicit_project_id: Some(project),
            force_personal: false,
            personal_user_id: None,
            git_metadata: None,
        },
        budget: BudgetConfig::default(),
        privacy: Privacy::LocalOnly,
        dry_run: true,
        briefing_id: None,
        ledgerful: None,
    };

    // Warm-up
    let _ = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        req.clone(),
    )
    .unwrap();

    let mut samples = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let packet = build_project_briefing(
            None::<&StoreEventWriter>,
            &ports.query,
            &clock,
            &policy,
            &identity,
            req.clone(),
        )
        .unwrap();
        let _md = render_project_markdown(&packet);
        samples.push(start.elapsed().as_millis() as u64);
    }
    samples.sort_unstable();
    let p95 = samples[(samples.len() * 95) / 100];
    eprintln!("project_briefing warm samples(ms)={samples:?} p95={p95}");
    // Soft threshold only (document, do not flake CI): warm p95 target < 200ms locally.
    // We assert a generous bound so ignored runs still surface catastrophic regressions.
    assert!(
        p95 < 5_000,
        "catastrophic regression: warm p95 {p95}ms (soft local target <200ms)"
    );
}
