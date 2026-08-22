//! T274 — chrome monopoly must not hide a leading DECISION pin (AC4 / AC12).
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{LexicalSearchOptions, RecallOptions, lexical_search, recall_full};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

fn append_pinned(
    store: &SqliteEventStore,
    content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let memory_id = MemoryId::new();
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let payload = Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.to_string(),
        session_id: None,
        project_id: Some(project_id),
        tx_id: None,
        rank: None,
        source_tag: None,
        query_text: None,
    });
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::CloudOk,
    )
    .build(payload)?;
    store.append_event(&envelope)?;
    Ok(memory_id.to_string())
}

fn default_opts() -> RecallOptions {
    RecallOptions {
        project_id: None,
        session_id: None,
        semantic: false,
        graph_boost: 0.0,
        graph_hop_depth: 0,
        quiet: true,
        no_bridge: true,
        min_semantic_score: None,
        include_symbols: false,
        preferred_project_id: None,
    }
}

/// AC4: 15 chrome rows MATCH the needle + 1 leading DECISION pin → hit #1 is the pin.
#[test]
fn recall_full__chrome_monopoly__authority_pin_is_hit_one() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let needle = format!("T274-rank-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        append_pinned(
            &store,
            &format!("## Objective\n{repeats}review dump {i} of the ranking remediator"),
        )?;
    }
    let pin_id = append_pinned(
        &store,
        &format!("DECISION: {needle} we chose the ranking remediator"),
    )?;

    let outcome = recall_full(store.connection(), None, &needle, 5, default_opts())?;
    assert!(
        !outcome.hits.is_empty(),
        "AC4: recall must return hits; got empty for needle={needle}"
    );
    assert_eq!(
        outcome.hits[0].memory_id,
        pin_id,
        "AC4: pin must be hit #1; first={} content={:?} all={:?}",
        outcome.hits[0].memory_id,
        outcome.hits[0].content,
        outcome
            .hits
            .iter()
            .map(|h| (h.memory_id.as_str(), h.content.as_str()))
            .collect::<Vec<_>>()
    );
    assert!(
        outcome.hits[0].content.starts_with("DECISION:"),
        "AC4: hit #1 content must start with DECISION:; got {}",
        outcome.hits[0].content
    );
    Ok(())
}

/// AC12: forget-style unfiltered MATCH still finds chrome (no two-pass GLOB).
#[test]
fn lexical_search__default_unfiltered__finds_session_chrome()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let needle = format!("T274-forget-chrome-{}", uuid::Uuid::new_v4());
    append_pinned(
        &store,
        &format!("## Objective\n{needle} review dump body for forget match"),
    )?;

    let hits = lexical_search(
        store.connection(),
        &needle,
        None,
        None,
        LexicalSearchOptions::default(),
    )?;
    assert!(
        hits.iter().any(|h| h.content.contains(&needle)),
        "AC12: unfiltered MATCH must find chrome; hits={:?}",
        hits.iter().map(|h| h.content.as_str()).collect::<Vec<_>>()
    );
    Ok(())
}
