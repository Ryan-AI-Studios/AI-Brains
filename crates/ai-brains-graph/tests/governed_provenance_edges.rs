//! T149 Phase G / R7 — governed provenance edges in the feature-gated graph projector.
//!
//! - Rebuild is idempotent
//! - OBSERVED_FROM / DERIVED_FROM / SUPPORTED_BY / SUPERSEDES / CONTAINS present for fixture
//! - Historical DERIVED_FROM retained after ConclusionMarkedStale

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{
    ConclusionId, DecisionId, EvidenceId, PrincipalId, ProjectId, SourceId, SourceVersionId,
    WorkspaceId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::source::SourceKind;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionMarkedStalePayload, ConclusionProposedPayload, DecisionProposedPayload,
    EvidenceRecordedPayload, EvidenceSupersededPayload, RepositoryJoinedWorkspacePayload,
    SourceRegisteredPayload, SourceVersionRecordedPayload, WorkspaceRegisteredPayload,
};
use ai_brains_events::{Actor, AggregateType, EventKind, Payload};
use ai_brains_graph::{
    GraphProjector, GraphRebuilder, GraphSearch, GraphVault, SqliteGraphBackend,
};
use ai_brains_store::EventStore;
use time::OffsetDateTime;

fn append(
    store: &impl EventStore,
    aggregate_type: AggregateType,
    aggregate_id: uuid::Uuid,
    kind: EventKind,
    payload: Payload,
) -> Result<uuid::Uuid, Box<dyn std::error::Error>> {
    let envelope = EventBuilder::new(
        aggregate_type,
        aggregate_id,
        kind,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(payload)?;
    let event_id = envelope.event_id;
    store.append_event(&envelope)?;
    Ok(event_id)
}

struct GovernedFixture {
    source_id: SourceId,
    version_id: SourceVersionId,
    evidence_old: EvidenceId,
    evidence_new: EvidenceId,
    conclusion_id: ConclusionId,
    decision_id: DecisionId,
    workspace_id: WorkspaceId,
    project_id: ProjectId,
}

fn governed_fixture(
    store: &impl EventStore,
) -> Result<GovernedFixture, Box<dyn std::error::Error>> {
    let source_id = SourceId::new();
    let version_id = SourceVersionId::new();
    let evidence_id = EvidenceId::new();
    let evidence_new = EvidenceId::new();
    let conclusion_id = ConclusionId::new();
    let decision_id = DecisionId::new();
    let workspace_id = WorkspaceId::new();
    let project_id = ProjectId::new();
    let principal = PrincipalId::new();
    let ts = OffsetDateTime::from_unix_timestamp(1_700_000_000)?;

    append(
        store,
        AggregateType::Source,
        source_id.as_uuid(),
        EventKind::SourceRegistered,
        Payload::SourceRegistered(SourceRegisteredPayload {
            source_id,
            kind: SourceKind::File,
            display_name: "spec.md".into(),
            locator: Some("/tmp/spec.md".into()),
            scope: None,
        }),
    )?;

    append(
        store,
        AggregateType::Source,
        source_id.as_uuid(),
        EventKind::SourceVersionRecorded,
        Payload::SourceVersionRecorded(SourceVersionRecordedPayload {
            source_id,
            version_id,
            fingerprint: "v1:abc".into(),
            recorded_at: ts,
        }),
    )?;

    append(
        store,
        AggregateType::Evidence,
        evidence_id.as_uuid(),
        EventKind::EvidenceRecorded,
        Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some("v1:abc".into()),
            model_provenance: None,
            summary: "file snippet".into(),
        }),
    )?;

    append(
        store,
        AggregateType::Evidence,
        evidence_new.as_uuid(),
        EventKind::EvidenceRecorded,
        Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id: evidence_new,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some("v1:abc".into()),
            model_provenance: None,
            summary: "newer snippet".into(),
        }),
    )?;

    append(
        store,
        AggregateType::Evidence,
        evidence_id.as_uuid(),
        EventKind::EvidenceSuperseded,
        Payload::EvidenceSuperseded(EvidenceSupersededPayload {
            evidence_id,
            superseded_by: evidence_new,
            reason: "newer observation".into(),
        }),
    )?;

    append(
        store,
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        EventKind::ConclusionProposed,
        Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "X holds".into(),
            evidence_ids: vec![evidence_new],
            proposer: principal,
        }),
    )?;

    append(
        store,
        AggregateType::Decision,
        decision_id.as_uuid(),
        EventKind::DecisionProposed,
        Payload::DecisionProposed(DecisionProposedPayload {
            decision_id,
            title: "Ship it".into(),
            statement: "We ship".into(),
            proposer: principal,
            conclusion_ids: Some(vec![conclusion_id]),
        }),
    )?;

    append(
        store,
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        EventKind::WorkspaceRegistered,
        Payload::WorkspaceRegistered(WorkspaceRegisteredPayload {
            workspace_id,
            name: "ws".into(),
        }),
    )?;

    append(
        store,
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        EventKind::RepositoryJoinedWorkspace,
        Payload::RepositoryJoinedWorkspace(RepositoryJoinedWorkspacePayload {
            workspace_id,
            project_id,
        }),
    )?;

    Ok(GovernedFixture {
        source_id,
        version_id,
        evidence_old: evidence_id,
        evidence_new,
        conclusion_id,
        decision_id,
        workspace_id,
        project_id,
    })
}

#[test]
fn governed_edges__present_for_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let fx = governed_fixture(&store)?;

    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(SqliteGraphBackend::new(conn.clone()));
    let mut projector = GraphProjector::new(backend);

    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);

    // Evidence → SourceVersion OBSERVED_FROM
    assert_eq!(
        search.count_edge(
            &fx.evidence_new.to_string(),
            &fx.version_id.to_string(),
            "OBSERVED_FROM"
        )?,
        1
    );
    assert_eq!(
        search.count_edge(
            &fx.evidence_old.to_string(),
            &fx.version_id.to_string(),
            "OBSERVED_FROM"
        )?,
        1
    );

    // Conclusion → Evidence DERIVED_FROM
    assert_eq!(
        search.count_edge(
            &fx.conclusion_id.to_string(),
            &fx.evidence_new.to_string(),
            "DERIVED_FROM"
        )?,
        1
    );

    // Decision → Conclusion SUPPORTED_BY
    assert_eq!(
        search.count_edge(
            &fx.decision_id.to_string(),
            &fx.conclusion_id.to_string(),
            "SUPPORTED_BY"
        )?,
        1
    );

    // successor SUPERSEDES predecessor
    assert_eq!(
        search.count_edge(
            &fx.evidence_new.to_string(),
            &fx.evidence_old.to_string(),
            "SUPERSEDES"
        )?,
        1
    );

    // Workspace CONTAINS Project
    assert_eq!(
        search.count_edge(
            &fx.workspace_id.to_string(),
            &fx.project_id.to_string(),
            "CONTAINS"
        )?,
        1
    );

    // Source CONTAINS SourceVersion
    assert_eq!(
        search.count_edge(
            &fx.source_id.to_string(),
            &fx.version_id.to_string(),
            "CONTAINS"
        )?,
        1
    );

    Ok(())
}

#[test]
fn governed_rebuild__is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let _ = governed_fixture(&store)?;

    let vault = GraphVault::new(store.connection().clone());
    let rebuilder = GraphRebuilder::new(&vault, &store);

    rebuilder.rebuild()?;
    let search = GraphSearch::new(&vault);
    let nodes_1 = search.node_count()?;
    let edges_1 = search.edge_count()?;
    assert!(nodes_1 > 0);
    assert!(edges_1 > 0);

    rebuilder.rebuild()?;
    let nodes_2 = search.node_count()?;
    let edges_2 = search.edge_count()?;
    assert_eq!(
        nodes_1, nodes_2,
        "node count must be stable across rebuilds"
    );
    assert_eq!(
        edges_1, edges_2,
        "edge count must be stable across rebuilds"
    );

    Ok(())
}

#[test]
fn conclusion_marked_stale__retains_historical_derived_from()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let fx = governed_fixture(&store)?;

    // Mark conclusion stale after edges exist.
    let stale = ConclusionMarkedStalePayload::try_new(
        fx.conclusion_id,
        Some(SourceVersionId::new()),
        None,
    )?;
    append(
        &store,
        AggregateType::Conclusion,
        fx.conclusion_id.as_uuid(),
        EventKind::ConclusionMarkedStale,
        Payload::ConclusionMarkedStale(stale),
    )?;

    let vault = GraphVault::new(store.connection().clone());
    GraphRebuilder::new(&vault, &store).rebuild()?;

    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.count_edge(
            &fx.conclusion_id.to_string(),
            &fx.evidence_new.to_string(),
            "DERIVED_FROM"
        )?,
        1,
        "stale must not erase historical DERIVED_FROM edges"
    );

    Ok(())
}
