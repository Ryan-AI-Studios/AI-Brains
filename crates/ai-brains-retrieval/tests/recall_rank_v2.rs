//! T285 AC4 — tagged pin vs body-MATCH chrome dumps (first-line asymmetric).
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{RecallOptions, recall_full};
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

fn envelope_stripped_starts_with_decision(content: &str) -> bool {
    let after_role = content
        .strip_prefix("ASSISTANT:")
        .or_else(|| content.strip_prefix("USER:"))
        .or_else(|| content.strip_prefix("SYSTEM:"))
        .unwrap_or(content)
        .trim_start();
    let mut lines = after_role.lines().map(str::trim).filter(|l| !l.is_empty());
    let Some(first) = lines.next() else {
        return false;
    };
    let marker = if first.to_ascii_lowercase().starts_with("tags:") {
        lines.next().unwrap_or("")
    } else {
        first
    };
    marker.to_ascii_lowercase().starts_with("decision:")
}

/// AC4: 15 `# Review of Track` dumps repeat the needle in the body + 1 tagged pin
/// → `recall_full` hit #1 is the pin.
#[test]
fn recall_full__tagged_pin_vs_body_match_review_dumps__hit_one__ac4()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let needle = format!("T285-rank-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        append_pinned(
            &store,
            &format!("# Review of Track 285: dump {i}\n{repeats}review body"),
        )?;
    }
    let pin_id = append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t285\nDECISION: {needle} we chose rank v2"),
    )?;

    let outcome = recall_full(store.connection(), None, &needle, 5, default_opts())?;
    assert!(
        !outcome.hits.is_empty(),
        "AC4: recall must return hits; got empty for needle={needle}"
    );
    assert_eq!(
        outcome.hits[0].memory_id,
        pin_id,
        "AC4: tagged pin must be hit #1; first={} content={:?} all={:?}",
        outcome.hits[0].memory_id,
        outcome.hits[0].content,
        outcome
            .hits
            .iter()
            .map(|h| (h.memory_id.as_str(), h.content.as_str()))
            .collect::<Vec<_>>()
    );
    assert!(
        envelope_stripped_starts_with_decision(&outcome.hits[0].content),
        "AC4: hit #1 must start with DECISION: after envelope; got {}",
        outcome.hits[0].content
    );
    Ok(())
}
