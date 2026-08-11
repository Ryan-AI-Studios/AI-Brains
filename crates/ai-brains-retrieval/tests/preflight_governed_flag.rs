#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T152 Phase F — dual-path preflight (legacy default / governed flag).
//! Governed path uses control-plane `build_project_briefing` + production policy.

mod common;

use ai_brains_control_plane::{
    StorePorts, SystemClock, issue_grant, make_principal, register_principal,
};
use ai_brains_core::ids::{ConclusionId, DecisionId, EvidenceId, PrincipalId, ProjectId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::temp_env::TempEnv;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionActivatedPayload, ConclusionProposedPayload, DecisionApprovedPayload,
    DecisionProposedPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_retrieval::{build_preflight, build_preflight_with_options};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use uuid::Uuid;

/// Well-known System principal id used by governed preflight (see preflight.rs).
fn preflight_system_id() -> PrincipalId {
    PrincipalId::from_uuid(Uuid::from_u128(
        0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2,
    ))
}

fn open_store() -> (tempfile::NamedTempFile, SqliteEventStore, ProjectId) {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let project_id = ProjectId::new();
    (temp_file, SqliteEventStore::new(conn), project_id)
}

fn seed_epistemic(store: &SqliteEventStore, project_id: ProjectId) {
    let scope = format!("Repository:{project_id}");
    let conclusion_id = ConclusionId::new();
    let decision_id = DecisionId::new();
    let evidence_id = EvidenceId::new();
    let proposer = PrincipalId::new();

    let propose_c = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id,
        statement: "Governed path uses typed conclusions".into(),
        evidence_ids: vec![evidence_id],
        proposer,
        valid_from: None,
        valid_until: None,
        scope: scope.clone(),
        protected_category: None,
        unsupported: false,
        model_provenance: None,
    }))
    .unwrap();
    let activate_c = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionActivated(ConclusionActivatedPayload {
        conclusion_id,
    }))
    .unwrap();

    let propose_d = EventBuilder::new(
        AggregateType::Decision,
        decision_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DecisionProposed(DecisionProposedPayload {
        decision_id,
        title: "Ship typed briefings".into(),
        statement: "Use ProjectBriefingPacket for governed preflight".into(),
        proposer,
        conclusion_ids: None,
        evidence_ids: Some(vec![evidence_id]),
        valid_from: None,
        valid_until: None,
        scope: scope.clone(),
    }))
    .unwrap();
    let approve_d = EventBuilder::new(
        AggregateType::Decision,
        decision_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DecisionApproved(DecisionApprovedPayload {
        decision_id,
        proposal_event_id: Uuid::nil(),
        approver: proposer,
        approved_at: ai_brains_core::clock::now(),
    }))
    .unwrap();

    EventStore::append_events(store, &[propose_c, activate_c, propose_d, approve_d]).unwrap();
}

fn register_preflight_principal_with_reads(store: &SqliteEventStore, project_id: ProjectId) {
    let ports = StorePorts::from_store(SqliteEventStore::new(store.connection().clone()));
    let clock = SystemClock;
    let principal = make_principal(
        PrincipalKind::System,
        preflight_system_id(),
        "preflight-system",
    );
    register_principal(&ports.writer, &clock, &principal).unwrap();
    let scope = ScopeRef::Repository(project_id);
    for cap in [
        GrantCapability::ReadDecisions,
        GrantCapability::ReadConclusions,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            principal.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
}

#[test]
fn preflight__flag_off__legacy_path() {
    let _guard = TempEnv::remove("AI_BRAINS_GOVERNED_BRIEFING");
    let store = common::store_with_memory(
        "ASSISTANT: CONSTRAINT: legacy string scrape still works",
        Privacy::CloudOk,
    )
    .unwrap();
    let project_id = ProjectId::from_uuid(Uuid::nil());
    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )
    .unwrap();
    // Legacy path surfaces CONSTRAINT: memory scrape (not Project Briefing header).
    assert!(!ctx.text.contains("# Project Briefing (governed)"));
    assert!(
        ctx.text.contains("CONSTRAINT:")
            || ctx.text.contains("Bearings")
            || ctx.text.contains("Memory Index")
            || ctx.word_count > 0
    );
}

#[test]
fn preflight__flag_on__uses_project_packet_not_decision_string_count() {
    let _guard = TempEnv::set("AI_BRAINS_GOVERNED_BRIEFING", "1");
    let (_tmp, store, project_id) = open_store();
    seed_epistemic(&store, project_id);
    register_preflight_principal_with_reads(&store, project_id);

    // Also pin a DECISION: memory — governed path must not use it for authority.
    {
        use ai_brains_core::ids::MemoryId;
        use ai_brains_events::payload::MemoryPinnedPayload;
        let memory_id = MemoryId::new();
        let env = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            Actor::System,
            Privacy::CloudOk,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: "ASSISTANT: DECISION: this must not count as authority".into(),
            session_id: None,
            project_id: Some(project_id),
            tx_id: None,
            rank: None,
            source_tag: None,
            query_text: None,
        }))
        .unwrap();
        store.append_event(&env).unwrap();
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )
    .unwrap();

    assert!(
        ctx.text.contains("# Project Briefing (governed)"),
        "governed flag must render packet markdown: {}",
        ctx.text
    );
    assert!(ctx.text.contains("Decisions (current authority)"));
    assert!(
        ctx.text.contains("Ship typed briefings") || ctx.text.contains("ProjectBriefingPacket"),
        "with grant, approved decision must appear: {}",
        ctx.text
    );
    // The DECISION: memory string is not used as authority listing.
    assert!(
        !ctx.text.contains("this must not count as authority"),
        "must not string-count DECISION: memories for authority"
    );
}

#[test]
fn preflight__governed_grant_denial__empties_authority_sections() {
    // Epistemic data exists, but preflight principal has no grants → empty decisions/conclusions.
    let (_tmp, store, project_id) = open_store();
    seed_epistemic(&store, project_id);
    // Register principal without Read* grants.
    let ports = StorePorts::from_store(SqliteEventStore::new(store.connection().clone()));
    let principal = make_principal(
        PrincipalKind::System,
        preflight_system_id(),
        "preflight-system",
    );
    register_principal(&ports.writer, &SystemClock, &principal).unwrap();

    let ctx = build_preflight_with_options(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
        Some(true),
    )
    .unwrap();

    assert!(
        ctx.text.contains("# Project Briefing (governed)"),
        "governed header must still render on denial: {}",
        ctx.text
    );
    // Authority sections empty (Denied or _None_).
    assert!(
        !ctx.text.contains("Ship typed briefings"),
        "decision title must not appear without grant: {}",
        ctx.text
    );
    assert!(
        !ctx.text.contains("Governed path uses typed conclusions"),
        "conclusion must not appear without grant: {}",
        ctx.text
    );
    assert!(
        ctx.text.contains("Denied") || ctx.text.contains("_None_") || ctx.text.contains("denied"),
        "denial signal expected: {}",
        ctx.text
    );
    // T227 F29/AC14: shared renderer deny next-step survives default word budget.
    assert!(
        ctx.text.contains("policy bootstrap"),
        "governed preflight denial must retain bootstrap next-step token within budget: {}",
        ctx.text
    );
}

#[test]
fn preflight__explicit_options_override_env() {
    let _guard = TempEnv::set("AI_BRAINS_GOVERNED_BRIEFING", "1");
    let store =
        common::store_with_memory("ASSISTANT: CONSTRAINT: force-legacy", Privacy::CloudOk).unwrap();
    let project_id = ProjectId::from_uuid(Uuid::nil());
    // Force legacy despite env=1.
    let ctx = build_preflight_with_options(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
        Some(false),
    )
    .unwrap();
    assert!(!ctx.text.contains("# Project Briefing (governed)"));
}

#[test]
fn preflight__governed_empty_state__no_project_id() {
    let (_tmp, store, _pid) = open_store();
    let ctx = build_preflight_with_options(
        store.connection(),
        None,
        1500,
        None,
        None,
        false,
        Some(true),
    )
    .unwrap();
    assert!(ctx.text.contains("# Project Briefing (governed)"));
    assert!(
        ctx.text.contains("unavailable")
            || ctx.text.contains("empty")
            || ctx.text.contains("unresolved")
            || ctx.text.contains("Warnings")
    );
}
