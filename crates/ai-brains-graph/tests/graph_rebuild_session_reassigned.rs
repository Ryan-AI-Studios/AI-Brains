//! T356 AC13: rebuild after SessionReassigned keeps a single IN_PROJECT edge.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use ai_brains_core::ids::{ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    Actor, AggregateType, Payload,
    constructors::EventBuilder,
    payload::{ProjectRegisteredPayload, SessionReassignedPayload},
};
use ai_brains_graph::{GraphRebuilder, GraphSearch, GraphVault};
use ai_brains_store::EventStore;
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn graph_rebuild__after_session_reassigned__single_in_project_edge_to_dest()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    let (session_id, from_project_id) = common::append_session(&store)?;

    let dest = ProjectId::new();
    let dest_env = EventBuilder::new(
        AggregateType::Project,
        dest.as_uuid(),
        Actor::System,
        Privacy::CloudOk,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id: dest,
        name: "dest".to_string(),
        tx_id: None,
    }))?;
    store.append_event(&dest_env)?;

    let session = SessionId::from_str(&session_id)?;
    let from = ProjectId::from_uuid(Uuid::parse_str(&from_project_id)?);
    let reassign = EventBuilder::new(
        AggregateType::Session,
        session.as_uuid(),
        Actor::System,
        Privacy::CloudOk,
    )
    .build(Payload::SessionReassigned(SessionReassignedPayload {
        session_id: session,
        from_project_id: from,
        to_project_id: dest,
        assigned_by: "human".to_string(),
        suspicious: false,
        confidence: None,
        model_provenance: None,
    }))?;
    store.append_event(&reassign)?;

    let vault = GraphVault::new(store.connection().clone());
    GraphRebuilder::new(&vault, &store).rebuild()?;

    let search = GraphSearch::new(&vault);
    let dest_s = dest.to_string();
    assert_eq!(
        search.count_edge(&session_id, &dest_s, "IN_PROJECT")?,
        1,
        "rebuild must emit dest IN_PROJECT"
    );
    assert_eq!(
        search.count_edge(&session_id, &from_project_id, "IN_PROJECT")?,
        0,
        "rebuild must not resurrect from IN_PROJECT"
    );
    let neighbors = search.get_neighbors(&session_id)?;
    let in_project: Vec<_> = neighbors
        .iter()
        .filter(|n| n.label == "IN_PROJECT" && n.direction == "outgoing")
        .collect();
    assert_eq!(in_project.len(), 1, "neighbors: {neighbors:?}");
    assert_eq!(in_project[0].external_id, dest_s);
    Ok(())
}
