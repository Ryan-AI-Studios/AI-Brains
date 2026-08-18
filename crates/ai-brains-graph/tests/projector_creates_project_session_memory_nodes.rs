#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::MemoryId;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    Actor, AggregateType, MemoryPinnedPayload, Payload, constructors::EventBuilder,
};
use ai_brains_graph::GraphProjector;
use ai_brains_graph::{GraphSearch, GraphVault};
use ai_brains_store::EventStore;
use uuid::Uuid;

#[test]
fn test_projector_creates_nodes_and_edges() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (session_id, project_id) = common::append_session(&store)?;

    // In Track T29, the graph and store share the same connection
    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(ai_brains_graph::SqliteGraphBackend::new(conn.clone()));
    let mut projector = GraphProjector::new(backend);

    // Replay events through projector
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    // Verify nodes and edges via SQL
    let conn_lock = conn.lock().map_err(|e| e.to_string())?;

    // Check Project Node
    let name: String = conn_lock.query_row(
        "SELECT n.kind FROM graph_node n WHERE n.external_id = ?",
        [project_id.clone()],
        |row| row.get(0),
    )?;
    assert_eq!(name, "project");

    // Check Session link to Project
    let count: i64 = conn_lock.query_row(
        "SELECT count(*) FROM graph_edge e 
         JOIN graph_node s ON e.src_id = s.node_id 
         JOIN graph_node p ON e.dst_id = p.node_id 
         WHERE s.external_id = ? AND p.external_id = ? AND e.label = 'IN_PROJECT'",
        [session_id, project_id],
        |row| row.get(0),
    )?;
    assert_eq!(count, 1);

    Ok(())
}

#[test]
fn test_projector_links_pinned_recall_memory_to_session() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::setup_store()?;
    let (session_id, _project_id) = common::append_session(&store)?;
    let session_uuid = Uuid::parse_str(&session_id)?;
    let memory_id = MemoryId::new();

    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: "recall hit".to_string(),
        session_id: Some(ai_brains_core::ids::SessionId::from_uuid(session_uuid)),
        project_id: None,
        tx_id: None,
        rank: Some(1),
        source_tag: Some("recall".to_string()),
        query_text: Some("recall".to_string()),
    }))?;
    store.append_event(&envelope)?;

    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(ai_brains_graph::SqliteGraphBackend::new(conn.clone()));
    let mut projector = GraphProjector::new(backend);

    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    let memories = search.get_session_memories(&session_id)?;

    assert_eq!(memories, vec![memory_id.to_string()]);
    Ok(())
}

#[test]
fn projector__user_prompt_with_turn_id__memory_node_and_recalls__ac2()
-> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::TurnId;
    use ai_brains_events::UserPromptRecordedPayload;

    let store = common::setup_store()?;
    let (session_id, _project_id) = common::append_session(&store)?;
    let session_uuid = Uuid::parse_str(&session_id)?;
    let turn_id = TurnId::new();

    let envelope = EventBuilder::new(
        AggregateType::Session,
        session_uuid,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: ai_brains_core::ids::SessionId::from_uuid(session_uuid),
        content: "DECISION: t262 ac2".to_string(),
        tx_id: None,
        turn_id: Some(turn_id),
    }))?;
    store.append_event(&envelope)?;

    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(ai_brains_graph::SqliteGraphBackend::new(conn));
    let mut projector = GraphProjector::new(backend);
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.node_kind(&turn_id.to_string())?,
        Some("memory".to_string())
    );
    assert_eq!(
        search.node_kind(&envelope.event_id.to_string())?,
        None,
        "F9: Some(turn_id) must not also emit a turn node at event_id"
    );
    assert_eq!(
        search.count_edge(&session_id, &turn_id.to_string(), "RECALLS")?,
        1
    );
    Ok(())
}

#[test]
fn projector__user_prompt_legacy_none__turn_node_is_event_id__ac3()
-> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::TurnId;
    use ai_brains_events::UserPromptRecordedPayload;

    let store = common::setup_store()?;
    let (session_id, _project_id) = common::append_session(&store)?;
    let session_uuid = Uuid::parse_str(&session_id)?;
    let printed = TurnId::new();

    let envelope = EventBuilder::new(
        AggregateType::Session,
        session_uuid,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
        session_id: ai_brains_core::ids::SessionId::from_uuid(session_uuid),
        content: "legacy hasher path".to_string(),
        tx_id: None,
        turn_id: None,
    }))?;
    store.append_event(&envelope)?;

    let conn = std::sync::Arc::new(store.connection().clone());
    let backend = Box::new(ai_brains_graph::SqliteGraphBackend::new(conn));
    let mut projector = GraphProjector::new(backend);
    for event in store.read_all_events()? {
        projector.apply(&event)?;
    }
    projector.flush()?;

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(
        search.node_kind(&printed.to_string())?,
        None,
        "printed random uuid is not a graph node on the legacy path"
    );
    assert_eq!(
        search.node_kind(&envelope.event_id.to_string())?,
        Some("turn".to_string())
    );
    assert_eq!(
        search.count_edge(&envelope.event_id.to_string(), &session_id, "IN_SESSION")?,
        1
    );
    Ok(())
}
