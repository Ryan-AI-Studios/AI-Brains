//! T260 — default recall excludes T70 symbol stubs (AC3–AC9 / AC16–AC17).
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{LexicalSearchOptions, RecallOptions, lexical_search, recall_full};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

const DECISION_FOO: &str = "DECISION: we chose foo for the bar path";
const STUB_FOO: &str = "Module foo (src/foo.rs:1)";
const KIND_PREFIX_NON_LOCATOR: &str = "Module foo (draft: notes)";
const LOWERCASE_MODULE_LOCATOR: &str = "module foo (src/foo.rs:1)";

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

fn symbols_opts() -> RecallOptions {
    RecallOptions {
        include_symbols: true,
        ..default_opts()
    }
}

#[test]
fn recall_full__default_excludes_symbol_stub__ac3() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, STUB_FOO)?;

    let outcome = recall_full(
        store.connection(),
        None,
        "what did we decide about foo",
        5,
        default_opts(),
    )?;

    assert!(
        outcome
            .hits
            .iter()
            .any(|h| h.content.contains("DECISION: we chose foo")),
        "AC3: DECISION must be present; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        outcome
            .hits
            .iter()
            .all(|h| !h.content.contains("Module foo (src/foo.rs:1)")),
        "AC3: T70 stub must be absent from default recall; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__kind_prefix_non_locator__survives_default__ac16()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, KIND_PREFIX_NON_LOCATOR)?;

    let outcome = recall_full(
        store.connection(),
        None,
        "foo draft notes",
        5,
        default_opts(),
    )?;

    assert!(
        outcome
            .hits
            .iter()
            .any(|h| h.content == KIND_PREFIX_NON_LOCATOR),
        "AC16: kind-prefix + non-digit locator must survive default GLOB; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__lowercase_module_locator__survives_default__ac17()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, LOWERCASE_MODULE_LOCATOR)?;

    let outcome = recall_full(
        store.connection(),
        None,
        "module foo src",
        5,
        default_opts(),
    )?;

    assert!(
        outcome
            .hits
            .iter()
            .any(|h| h.content == LOWERCASE_MODULE_LOCATOR),
        "AC17: lowercase kind + real locator must survive GLOB; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn lexical_search__default_still_returns_symbol__ac9() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(STUB_FOO, Privacy::CloudOk)?;
    let results = lexical_search(
        store.connection(),
        "Module foo",
        None,
        None,
        LexicalSearchOptions::default(),
    )?;
    assert!(
        results.iter().any(|m| m.content == STUB_FOO),
        "AC9/F10: default lexical_search must still return stubs (forget path); got {:?}",
        results
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__symbols_includes_stub__ac4() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, STUB_FOO)?;

    let outcome = recall_full(
        store.connection(),
        None,
        "what did we decide about foo",
        5,
        symbols_opts(),
    )?;

    assert!(
        outcome.hits.iter().any(|h| h.content == STUB_FOO),
        "AC4: --symbols must include the stub; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__duplicate_symbol_content__deduped__ac6() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, STUB_FOO)?;
    append_pinned(&store, STUB_FOO)?;
    append_pinned(&store, STUB_FOO)?;
    append_pinned(&store, DECISION_FOO)?;

    let outcome = recall_full(store.connection(), None, "foo", 5, symbols_opts())?;
    let stub_hits: Vec<_> = outcome
        .hits
        .iter()
        .filter(|h| h.content == STUB_FOO)
        .collect();
    assert_eq!(
        stub_hits.len(),
        1,
        "AC6: identical stub content must collapse to one row; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| (&h.memory_id, h.content.as_str()))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__default_excludes_strongest_stub__ac7() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, STUB_FOO)?;

    let default = recall_full(store.connection(), None, "foo", 5, default_opts())?;
    assert!(
        default.hits.iter().all(|h| h.content != STUB_FOO),
        "AC7: default must drop the stub even if it is the strongest MATCH; hits={:?}",
        default
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );

    let mixed = recall_full(store.connection(), None, "foo", 5, symbols_opts())?;
    let decision_pos = mixed.hits.iter().position(|h| h.content == DECISION_FOO);
    let stub_pos = mixed.hits.iter().position(|h| h.content == STUB_FOO);
    assert!(
        decision_pos.is_some() && stub_pos.is_some(),
        "AC7 --symbols: both present; hits={:?}",
        mixed
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        decision_pos < stub_pos,
        "AC7 --symbols: DECISION must rank above stub; hits={:?}",
        mixed
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recall_full__semantic_default_no_f11_stub__ac8() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    append_pinned(&store, DECISION_FOO)?;
    append_pinned(&store, STUB_FOO)?;

    let mut opts = default_opts();
    opts.semantic = true;
    let outcome = recall_full(
        store.connection(),
        None,
        "what did we decide about foo",
        5,
        opts,
    )?;
    assert!(
        outcome.hits.iter().all(|h| h.content != STUB_FOO),
        "AC8: default semantic remainder must not be a stub; hits={:?}",
        outcome
            .hits
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    if outcome.embedding.as_ref().is_some_and(|e| e.status == "ok")
        && outcome.semantic_post_threshold_count == Some(0)
    {
        assert!(
            outcome.hits.is_empty()
                || outcome
                    .hits
                    .iter()
                    .any(|h| matches!(h.source.as_str(), "fts" | "substring" | "hybrid")),
            "AC8: F11 remainder is non-stub lexical or empty"
        );
    }
    Ok(())
}
