//! T335: session-less MemoryPinned with project_id emits PINNED_IN_PROJECT.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    Actor, AggregateType, MemoryPinnedPayload, Payload, constructors::EventBuilder,
};
use ai_brains_graph::{
    GraphProjector, GraphRebuilder, GraphSearch, GraphVault, SqliteGraphBackend,
};
use ai_brains_store::EventStore;
use uuid::Uuid;

fn pin_envelope(
    memory_id: MemoryId,
    session_id: Option<ai_brains_core::ids::SessionId>,
    project_id: Option<ProjectId>,
    source_tag: Option<String>,
) -> Result<ai_brains_events::Envelope, Box<dyn std::error::Error>> {
    Ok(EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "symbol stub".to_string(),
        session_id,
        project_id,
        tx_id: None,
        rank: None,
        source_tag,
        query_text: None,
    }))?)
}

fn project_all(
    store: &ai_brains_store::SqliteEventStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(SqliteGraphBackend::new(conn));
    let mut projector = GraphProjector::new(backend);
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;
    Ok(())
}

#[test]
fn projector_memory_pinned__project_id_no_session__pinned_in_project_edge()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (_session_id, project_id) = common::append_session(&store)?;
    let project_uuid = Uuid::parse_str(&project_id)?;
    let memory_id = MemoryId::new();

    store.append_event(&pin_envelope(
        memory_id,
        None,
        Some(ProjectId::from_uuid(project_uuid)),
        Some("ledgerful:symbol".to_string()),
    )?)?;

    project_all(&store)?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.count_edge(&memory_id.to_string(), &project_id, "PINNED_IN_PROJECT")?,
        1
    );
    let neighbors = search.get_neighbors(&memory_id.to_string())?;
    assert!(
        neighbors.iter().any(|n| {
            n.label == "PINNED_IN_PROJECT"
                && n.direction == "outgoing"
                && n.external_id == project_id
        }),
        "neighbors: {neighbors:?}"
    );
    Ok(())
}

#[test]
fn projector_memory_pinned__session_and_project__no_pinned_in_project()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (session_id, project_id) = common::append_session(&store)?;
    let session_uuid = Uuid::parse_str(&session_id)?;
    let project_uuid = Uuid::parse_str(&project_id)?;
    let memory_id = MemoryId::new();

    store.append_event(&pin_envelope(
        memory_id,
        Some(ai_brains_core::ids::SessionId::from_uuid(session_uuid)),
        Some(ProjectId::from_uuid(project_uuid)),
        Some("recall".to_string()),
    )?)?;

    project_all(&store)?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.count_edge(&session_id, &memory_id.to_string(), "RECALLS")?,
        1
    );
    assert_eq!(
        search.count_edge(&memory_id.to_string(), &project_id, "PINNED_IN_PROJECT")?,
        0
    );
    Ok(())
}

#[test]
fn projector_memory_pinned__neither_id__no_new_edge() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let memory_id = MemoryId::new();

    store.append_event(&pin_envelope(memory_id, None, None, None)?)?;
    project_all(&store)?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.node_kind(&memory_id.to_string())?,
        Some("memory".to_string())
    );
    let neighbors = search.get_neighbors(&memory_id.to_string())?;
    assert!(
        neighbors.iter().all(|n| n.label != "PINNED_IN_PROJECT"),
        "neighbors: {neighbors:?}"
    );
    assert!(
        neighbors.iter().all(|n| n.label != "RECALLS"),
        "no invented session RECALLS: {neighbors:?}"
    );
    Ok(())
}

#[test]
fn graph_edge_check__pinned_in_project__allows_and_rejects_garbage()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let conn = store.connection();
    let locked = conn.lock().map_err(|e| e.to_string())?;
    locked.execute(
        "INSERT INTO graph_node (kind, external_id) VALUES ('memory', 't335-mem')",
        [],
    )?;
    locked.execute(
        "INSERT INTO graph_node (kind, external_id) VALUES ('project', 't335-proj')",
        [],
    )?;
    locked.execute(
        "INSERT INTO graph_edge (src_id, label, dst_id, weight)
         SELECT s.node_id, 'PINNED_IN_PROJECT', d.node_id, 1.0
         FROM graph_node s, graph_node d
         WHERE s.external_id = 't335-mem' AND d.external_id = 't335-proj'",
        [],
    )?;
    let err = locked
        .execute(
            "INSERT INTO graph_edge (src_id, label, dst_id, weight)
             SELECT s.node_id, 'NOT_A_GRAPH_LABEL', d.node_id, 1.0
             FROM graph_node s, graph_node d
             WHERE s.external_id = 't335-mem' AND d.external_id = 't335-proj'",
            [],
        )
        .expect_err("garbage label must CHECK-fail");
    let msg = err.to_string();
    assert!(
        msg.contains("CHECK") || msg.contains("constraint"),
        "expected CHECK failure, got: {msg}"
    );
    Ok(())
}

#[test]
fn projector_memory_pinned__t70_shaped__rebuild_survives() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::setup_store()?;
    let (_session_id, project_id) = common::append_session(&store)?;
    let project_uuid = Uuid::parse_str(&project_id)?;
    let memory_id = MemoryId::new();

    store.append_event(&pin_envelope(
        memory_id,
        None,
        Some(ProjectId::from_uuid(project_uuid)),
        Some("ledgerful:symbol".to_string()),
    )?)?;

    project_all(&store)?;
    let vault = GraphVault::new(store.connection().clone());
    GraphRebuilder::new(&vault, &store).rebuild()?;
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.count_edge(&memory_id.to_string(), &project_id, "PINNED_IN_PROJECT")?,
        1
    );
    Ok(())
}
