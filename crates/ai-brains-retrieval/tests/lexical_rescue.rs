//! T217 hermetic: multi-token FTS rescue ladder (R0 → R1 → R2).

#![allow(non_snake_case)] // TDD names use __ separators

mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{
    LEXICAL_MATCH_HARD_CAP, LexicalSearchOptions, lexical_search, match_limit_bound,
};

#[test]
fn lexical_rescue__natural_phrase_empty_r0__hits_with_rescue()
-> Result<(), Box<dyn std::error::Error>> {
    // AC1: pin contentful keywords only; NL multi-token empty under R0 → hits with rescue.
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "what did we decide about forget list",
        None,
        None,
        LexicalSearchOptions {
            rescue: true,
            limit: 50,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        !results.is_empty(),
        "rescue=true should hit contentful keywords via R1/R2; got empty"
    );
    assert!(
        results[0].content.contains("forget list"),
        "hit content unexpected: {}",
        results[0].content
    );
    // BM25 rank present (source stays fts on recall path).
    assert!(
        results[0].score.is_some(),
        "FTS hits must carry BM25 rank score"
    );
    Ok(())
}

#[test]
fn lexical_rescue__natural_phrase__empty_when_rescue_false()
-> Result<(), Box<dyn std::error::Error>> {
    // AC14 / M2: rescue=false (forget path) must not OR-widen.
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "what did we decide about forget list",
        None,
        None,
        LexicalSearchOptions {
            rescue: false,
            limit: 50,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        results.is_empty(),
        "rescue=false must stay strict R0 AND; got {} hits",
        results.len()
    );
    Ok(())
}

#[test]
fn lexical_rescue__gibberish_multi_token__empty() -> Result<(), Box<dyn std::error::Error>> {
    // AC3
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "zzzz_no_such_token_aaa bbb ccc",
        None,
        None,
        LexicalSearchOptions {
            rescue: true,
            limit: 50,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        results.is_empty(),
        "true-empty gibberish must stay empty even with rescue"
    );
    Ok(())
}

#[test]
fn lexical_rescue__two_token_hits_without_rescue() -> Result<(), Box<dyn std::error::Error>> {
    // AC4: two-token "forget list" hits without needing ladder
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "forget list",
        None,
        None,
        LexicalSearchOptions::default(),
    )?;
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("forget list"));
    Ok(())
}

#[test]
fn lexical_rescue__single_token_known_pin__hits() -> Result<(), Box<dyn std::error::Error>> {
    // AC2: single-token known pin still hits (ladder N/A)
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "forget",
        None,
        None,
        LexicalSearchOptions::default(),
    )?;
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("forget"));
    Ok(())
}

#[test]
fn lexical_rescue__stopword_phrase__hits_via_r1_contentful_and()
-> Result<(), Box<dyn std::error::Error>> {
    // R1 isolation: contentful tokens equal pin keywords (no extra contentful noise).
    // Query "what about the forget list" → contentful [forget, list] → R1 AND hits.
    let store =
        common::store_with_memory("DECISION: forget list inventory skim", Privacy::CloudOk)?;

    let results = lexical_search(
        store.connection(),
        "what about the forget list",
        None,
        None,
        LexicalSearchOptions {
            rescue: true,
            limit: 50,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        !results.is_empty(),
        "R1 contentful AND should hit forget+list; got empty"
    );
    assert!(results[0].content.contains("forget list"));
    Ok(())
}

#[test]
fn lexical_rescue__sql_limit_bound_respects_caller_and_hard_cap()
-> Result<(), Box<dyn std::error::Error>> {
    // AC15: LIMIT on MATCH path; result count ≤ bound; hard cap 200.
    assert_eq!(match_limit_bound(3), 3);
    assert_eq!(match_limit_bound(500), LEXICAL_MATCH_HARD_CAP);

    // Pin several memories that all match a single token under R0 (explicit pins; no loop).
    let store = common::store_with_memory("alpha shared token one", Privacy::CloudOk)?;
    pin_extra(
        store.connection(),
        "alpha shared token two",
        Privacy::CloudOk,
    )?;
    pin_extra(
        store.connection(),
        "alpha shared token three",
        Privacy::CloudOk,
    )?;
    pin_extra(
        store.connection(),
        "alpha shared token four",
        Privacy::CloudOk,
    )?;
    pin_extra(
        store.connection(),
        "alpha shared token five",
        Privacy::CloudOk,
    )?;

    let results = lexical_search(
        store.connection(),
        "alpha",
        None,
        None,
        LexicalSearchOptions {
            rescue: false,
            limit: 2,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        results.len() <= 2,
        "SQL LIMIT must cap results; got {}",
        results.len()
    );
    assert!(
        !results.is_empty(),
        "expected at least one alpha hit under LIMIT 2"
    );
    Ok(())
}

#[test]
fn lexical_rescue__r2_or_respects_limit_and_excludes_sealed()
-> Result<(), Box<dyn std::error::Error>> {
    // AC15 R2 path + privacy-before-LIMIT: broad OR with sealed noise cannot
    // consume the bound or appear in results.
    let store = common::store_with_memory("hotspot brittle alpha", Privacy::CloudOk)?;
    pin_extra(store.connection(), "hotspot brittle beta", Privacy::CloudOk)?;
    pin_extra(
        store.connection(),
        "hotspot brittle gamma",
        Privacy::CloudOk,
    )?;
    pin_extra(
        store.connection(),
        "hotspot brittle delta",
        Privacy::CloudOk,
    )?;
    pin_extra(
        store.connection(),
        "hotspot brittle sealed-noise",
        Privacy::Sealed,
    )?;
    pin_extra(
        store.connection(),
        "hotspot brittle never-inject-noise",
        Privacy::NeverInject,
    )?;

    // 3 contentful tokens with no stopwords → R1 skipped (c==tokens) → R2 OR.
    let results = lexical_search(
        store.connection(),
        "hotspot brittle fix",
        None,
        None,
        LexicalSearchOptions {
            rescue: true,
            limit: 2,
            exclude_symbol_stubs: false,
            prefer_authority: false,
        },
    )?;
    assert!(
        results.len() <= 2,
        "R2 OR must honor SQL LIMIT; got {}",
        results.len()
    );
    assert!(
        !results.is_empty(),
        "R2 OR should hit contentful CloudOk pins"
    );
    assert!(
        results
            .iter()
            .all(|hit| !hit.content.contains("sealed-noise")
                && !hit.content.contains("never-inject-noise")),
        "non-injectable privacy must not appear in R2 results: {:?}",
        results
            .iter()
            .map(|h| h.content.as_str())
            .collect::<Vec<_>>()
    );
    Ok(())
}

fn pin_extra(
    conn: &ai_brains_store::VaultConnection,
    content: &str,
    privacy: Privacy,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::ids::{MemoryId, ProjectId};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::{Actor, AggregateType, Payload, payload::MemoryPinnedPayload};
    use ai_brains_store::event_store::{EventStore, SqliteEventStore};

    // Re-wrap connection into store for append (same vault).
    let store = SqliteEventStore::new(conn.clone());
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
        privacy,
    )
    .build(payload)?;
    store.append_event(&envelope)?;
    Ok(())
}
