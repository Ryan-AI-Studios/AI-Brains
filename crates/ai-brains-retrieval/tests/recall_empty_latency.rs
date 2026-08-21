//! T261 — contentless recall short-circuit (AC1–AC5 / AC14 / AC15 / AC17).
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{
    LexicalSearchOptions, RecallOptions, lexical_search, recall_full, substring_fallback,
};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

const DECISION_SPACES: &str = "DECISION: we chose foo   for the bar path";
const STUB_FOO: &str = "Module foo (src/foo.rs:1)";
const FORGET_LIST: &str = "DECISION: forget list inventory stays project scoped";

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
    }
}

#[test]
fn recall_full__empty_query__no_hits__ac1() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;

    let outcome = recall_full(store.connection(), None, "", 5, default_opts())?;
    assert!(
        outcome.hits.is_empty(),
        "AC1: empty query must return no hits; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        outcome.embedding.is_none(),
        "AC1: embedding must be None when semantic is false; got={:?}",
        outcome.embedding
    );
    Ok(())
}

#[test]
fn recall_full__whitespace__no_hits__ac2() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;

    let outcome = recall_full(store.connection(), None, "   ", 5, default_opts())?;
    assert!(
        outcome.hits.is_empty(),
        "AC2: whitespace must not substring match-all; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__all_stopword__no_hits__ac3() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;

    for query in ["the the the", "what is the"] {
        let outcome = recall_full(store.connection(), None, query, 5, default_opts())?;
        assert!(
            outcome.hits.is_empty(),
            "AC3: all-stopword {query:?} must return no hits; hits={:?}",
            outcome
                .hits
                .iter()
                .map(|h| h.content.as_str())
                .collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[test]
fn recall_full__contentful_still_searches__ac4() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, FORGET_LIST)?;

    let outcome = recall_full(store.connection(), None, "forget list", 5, default_opts())?;
    assert!(
        outcome
            .hits
            .iter()
            .any(|h| h.content.contains("forget list inventory")),
        "AC4: contentful query must still search; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__semantic_contentless__embedding_skipped__ac5()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;
    // If the short-circuit is missing, fetch_embedding hits this dead port.
    let _guard = TempEnv::set("AI_BRAINS_EMBEDDING_URL", "http://127.0.0.1:1");

    let outcome = recall_full(
        store.connection(),
        None,
        "",
        5,
        RecallOptions {
            semantic: true,
            ..default_opts()
        },
    )?;
    assert!(
        outcome.hits.is_empty(),
        "AC5: semantic contentless must return no hits; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    let embedding = outcome
        .embedding
        .as_ref()
        .expect("AC5: embedding present when semantic");
    assert_eq!(
        embedding.status, "skipped",
        "AC5: status must be skipped; embedding={embedding:?}"
    );
    assert_eq!(
        embedding.detail.as_deref(),
        Some("contentless_query"),
        "AC5: detail must be contentless_query; embedding={embedding:?}"
    );
    assert_eq!(
        embedding.endpoint, None,
        "AC5: endpoint must be None; embedding={embedding:?}"
    );
    assert_eq!(
        outcome.semantic_post_threshold_count,
        Some(0),
        "AC5: post-threshold count must be Some(0)"
    );
    Ok(())
}

#[test]
fn recall_full__symbols_contentless__still_empty__ac17() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, STUB_FOO)?;

    append_pinned(&store, DECISION_SPACES)?;
    let outcome = recall_full(
        store.connection(),
        None,
        "the the the",
        5,
        RecallOptions {
            include_symbols: true,
            ..default_opts()
        },
    )?;
    assert!(
        outcome.hits.is_empty(),
        "AC17: --symbols must not override contentless empty; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn substring_fallback__whitespace__empty_before_count__ac14()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;

    let hits = substring_fallback(store.connection(), "   ", None, None, 5, false)?;
    assert!(
        hits.is_empty(),
        "AC14: whitespace substring must return [] before COUNT; hits={:?}",
        hits.iter().map(|h| h.content.as_str()).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn lexical_search__all_stopword__still_matches__ac15() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_SPACES)?;

    let hits = lexical_search(
        store.connection(),
        "the the the",
        None,
        None,
        LexicalSearchOptions {
            rescue: false,
            limit: 5,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        hits.iter().any(|h| h.content.contains("we chose foo")),
        "AC15: forget-path lexical_search must still MATCH stopwords; hits={:?}",
        hits.iter().map(|h| h.content.as_str()).collect::<Vec<_>>()
    );
    Ok(())
}
