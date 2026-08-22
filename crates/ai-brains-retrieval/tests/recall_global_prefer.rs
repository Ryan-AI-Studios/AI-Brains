//! T276 — leftover volume must not starve `--global` prefer-fill (AC2 / AC3 / AC6).
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{RecallOptions, recall_full};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use std::str::FromStr;

fn append_pinned_on(
    store: &SqliteEventStore,
    project_id: ProjectId,
    content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let memory_id = MemoryId::new();
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

fn opts(preferred: Option<ProjectId>) -> RecallOptions {
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
        preferred_project_id: preferred,
    }
}

fn two_projects() -> Result<(ProjectId, ProjectId), Box<dyn std::error::Error>> {
    Ok((
        ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")?,
        ProjectId::from_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")?,
    ))
}

/// AC2: leftover-like B fills unscoped authority MATCH; owner A pin must still be hit #1
/// once `preferred_project_id = Some(A)`.
///
/// Live T274 two-pass already lifts **chrome** under unscoped MATCH. Leftover
/// **authority** volume (15 DECISION rows) is the T276 starve: pass-1 GLOB fills
/// `candidate_depth(5)=15` and the owner pin never enters without prefer-fill.
#[test]
fn recall_full__global_prefer__owner_pin_beats_leftover_chrome()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let (owner, leftover) = two_projects()?;
    let needle = format!("T276-prefer-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        append_pinned_on(
            &store,
            leftover,
            &format!("DECISION: leftover dump {i} {repeats}"),
        )?;
    }
    let pin_id = append_pinned_on(
        &store,
        owner,
        &format!("DECISION: {needle} owner unique pin we must surface"),
    )?;

    let outcome = recall_full(store.connection(), None, &needle, 5, opts(Some(owner)))?;
    assert!(
        !outcome.hits.is_empty(),
        "AC2: recall must return hits; got empty for needle={needle}"
    );
    assert_eq!(
        outcome.hits[0].memory_id,
        pin_id,
        "AC2: owner pin must be hit #1; first={} content={:?} all={:?}",
        outcome.hits[0].memory_id,
        outcome.hits[0].content,
        outcome
            .hits
            .iter()
            .map(|h| (h.memory_id.as_str(), h.content.as_str()))
            .collect::<Vec<_>>()
    );
    Ok(())
}

/// AC3 / F41: leftover still appears in recall hits (not a SQL drop).
/// Pre-rerank merge is the contract; post-rerank top-5 may still include leftover
/// here because preferred has only 1 pin (remainder > 0).
#[test]
fn recall_full__global_prefer__leftover_still_in_candidates()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let (owner, leftover) = two_projects()?;
    let needle = format!("T276-leftover-still-{}", uuid::Uuid::new_v4());
    let mut leftover_ids = Vec::new();
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        leftover_ids.push(append_pinned_on(
            &store,
            leftover,
            &format!("DECISION: leftover dump {i} {repeats}"),
        )?);
    }
    append_pinned_on(
        &store,
        owner,
        &format!("DECISION: {needle} owner unique pin we must surface"),
    )?;

    let outcome = recall_full(store.connection(), None, &needle, 5, opts(Some(owner)))?;
    let leftover_in_hits = outcome
        .hits
        .iter()
        .any(|h| leftover_ids.iter().any(|id| id == &h.memory_id));
    assert!(
        leftover_in_hits,
        "AC3: leftover matching dump must still appear (label, do not drop); hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

/// AC6: `preferred_project_id: None` keeps the unscoped path (no prefer-fill panic).
#[test]
fn recall_full__preferred_none__no_fill_panic() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let needle = format!("T276-none-{}", uuid::Uuid::new_v4());
    append_pinned_on(
        &store,
        ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")?,
        &format!("DECISION: {needle} unscoped pin"),
    )?;
    let outcome = recall_full(store.connection(), None, &needle, 5, opts(None))?;
    assert!(
        outcome.hits.iter().any(|h| h.content.contains(&needle)),
        "AC6: unscoped preferred=None must still recall; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}
