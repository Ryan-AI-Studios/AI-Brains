//! T346 — Index-authority fill when FTS + T105 LIKE retain 0.
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{
    RecallHit, RecallOptions, has_fts_arm, merge_bridge_then_local, recall_full,
};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

const UNMATCHED: &str = "zzzzt346nomatch";
const NIL_PROJECT: uuid::Uuid = uuid::Uuid::nil();

fn append_pinned(
    store: &SqliteEventStore,
    content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let memory_id = MemoryId::new();
    let project_id = ProjectId::from_uuid(NIL_PROJECT);
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

fn scoped_opts() -> RecallOptions {
    RecallOptions {
        project_id: Some(ProjectId::from_uuid(NIL_PROJECT)),
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

fn global_opts() -> RecallOptions {
    RecallOptions {
        project_id: None,
        ..scoped_opts()
    }
}

#[test]
fn recall_index_fill__fts_empty_authority_pin__honesty_and_hits()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let pin_id = append_pinned(&store, "DECISION: we chose the empty-rescue path")?;

    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, scoped_opts())?;
    assert_eq!(
        outcome.hits.len(),
        1,
        "AC1: unmatched query must fill the authority pin; hits={:?}",
        outcome.hits.iter().map(|h| &h.source).collect::<Vec<_>>()
    );
    assert_eq!(outcome.hits[0].memory_id, pin_id);
    assert_eq!(outcome.hits[0].source, "index");
    assert!(
        outcome.hits[0].score.is_none(),
        "AC1/AC10: fill omits score; got {:?}",
        outcome.hits[0].score
    );
    Ok(())
}

#[test]
fn recall_index_fill__lexical__embedding_none_still_fills() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    append_pinned(&store, "DECISION: lexical fill must not probe embed")?;

    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, scoped_opts())?;
    assert!(
        outcome.embedding.is_none(),
        "AC2: lexical fill must leave embedding None; got {:?}",
        outcome.embedding
    );
    assert!(
        outcome.hits.iter().any(|h| h.source == "index"),
        "AC2: fill still happens without --semantic"
    );
    Ok(())
}

#[test]
fn recall_index_fill__classify__decision_constraint_not_hotspot_other()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let decision = append_pinned(&store, "DECISION: classify keep")?;
    let constraint = append_pinned(&store, "CONSTRAINT: classify keep")?;
    let invariant = append_pinned(&store, "INVARIANT: classify keep")?;
    let hotspot = append_pinned(&store, "HOTSPOT: classify drop")?;
    let other = append_pinned(&store, "unmarked other classify drop")?;

    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, scoped_opts())?;
    let ids: Vec<&str> = outcome.hits.iter().map(|h| h.memory_id.as_str()).collect();
    assert!(
        ids.contains(&decision.as_str()),
        "AC3: DECISION must fill; ids={ids:?}"
    );
    assert!(
        ids.contains(&constraint.as_str()),
        "AC3: CONSTRAINT must fill; ids={ids:?}"
    );
    assert!(
        ids.contains(&invariant.as_str()),
        "AC3: INVARIANT must fill; ids={ids:?}"
    );
    assert!(
        !ids.contains(&hotspot.as_str()),
        "AC3: HOTSPOT must not fill; ids={ids:?}"
    );
    assert!(
        !ids.contains(&other.as_str()),
        "AC3: Other must not fill; ids={ids:?}"
    );
    assert!(
        outcome.hits.iter().all(|h| h.source == "index"),
        "AC3: fill hits are source=index"
    );
    Ok(())
}

#[test]
fn recall_index_fill__contentless__no_fill() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, "DECISION: contentless must not fill")?;

    for q in ["", "   ", "the the the"] {
        let outcome = recall_full(store.connection(), None, q, 5, scoped_opts())?;
        assert!(
            outcome.hits.is_empty(),
            "AC7: contentless {q:?} must not fill; hits={}",
            outcome.hits.len()
        );
    }
    Ok(())
}

#[test]
fn recall_index_fill__global__no_fill_t111_hint() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, "DECISION: global must not fill")?;

    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, global_opts())?;
    assert!(
        outcome.hits.is_empty(),
        "AC8: --global must not Index-fill; hits={}",
        outcome.hits.len()
    );
    Ok(())
}

#[test]
fn recall_index_fill__no_pins__t111_hint() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, scoped_opts())?;
    assert!(
        outcome.hits.is_empty(),
        "AC9: no authority pins → empty; hits={}",
        outcome.hits.len()
    );
    Ok(())
}

#[test]
fn recall_index_fill__source_index__not_fts_arm() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, "DECISION: source is index not fts")?;

    let outcome = recall_full(store.connection(), None, UNMATCHED, 5, scoped_opts())?;
    assert!(!outcome.hits.is_empty(), "AC10: expected fill hits");
    assert!(
        outcome.hits.iter().all(|h| h.source == "index"),
        "AC10: source must be index; got {:?}",
        outcome.hits.iter().map(|h| &h.source).collect::<Vec<_>>()
    );
    assert!(
        !has_fts_arm(&outcome.hits),
        "AC10: fill-only hits must not count as FTS arm"
    );
    assert!(
        outcome.hits.iter().all(|h| h.score.is_none()),
        "AC10: fill omits score"
    );
    assert!(
        outcome
            .hits
            .iter()
            .all(|h| h.score_kind == ai_brains_retrieval::ScoreKind::Bm25LowerBetter),
        "AC10: score_kind stays bm25 wire"
    );
    Ok(())
}

#[test]
fn recall_index_fill__phase2c__bridge_hit_does_not_suppress() {
    let index = RecallHit::index(
        "index-1".into(),
        "DECISION: fill must survive bridge merge".into(),
        None,
        None,
    );
    let bridge = RecallHit::bridge(
        "bridge-1".into(),
        "ledger insight".into(),
        Some(12.0),
        "bridge".into(),
        None,
        None,
    );
    let blended = merge_bridge_then_local(&[bridge], vec![index], 2);
    assert!(
        blended.iter().any(|h| h.source == "index"),
        "AC11: Phase 4 bridge-first merge must retain source=index; got {:?}",
        blended.iter().map(|h| &h.source).collect::<Vec<_>>()
    );
    assert!(
        blended.iter().any(|h| h.source == "bridge"),
        "AC11: bridge hit still present"
    );
}
