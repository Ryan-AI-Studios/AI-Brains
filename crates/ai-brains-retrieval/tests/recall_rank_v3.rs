//! T312 — recall rank v3 hermetics (authority-OR fill + verbose-Other).
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

/// T312 AC4 stay-green: prose first-line dumps + full-needle tagged pin → pin #1.
#[test]
fn recall_full__prose_dumps_full_needle_pin__hit_one__ac4() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let needle = format!("T312-rank-needle-{}", uuid::Uuid::new_v4());
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        append_pinned(
            &store,
            &format!("Here's the assessment. dump {i}\n{repeats}prose body"),
        )?;
    }
    let pin_id = append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t312\nDECISION: {needle} we chose rank v3"),
    )?;

    let outcome = recall_full(store.connection(), None, &needle, 5, default_opts())?;
    assert!(
        !outcome.hits.is_empty(),
        "AC4: recall must return hits; got empty for needle={needle}"
    );
    assert_eq!(
        outcome.hits[0].memory_id, pin_id,
        "AC4: tagged pin must be hit #1; first={} content={:?}",
        outcome.hits[0].memory_id, outcome.hits[0].content
    );
    assert!(
        envelope_stripped_starts_with_decision(&outcome.hits[0].content),
        "AC4: hit #1 must start with DECISION: after envelope; got {}",
        outcome.hits[0].content
    );
    Ok(())
}

/// T312 AC5: AND-miss / OR-hit (F42) — query `"t312or backend"`; pin lacks `backend`.
#[test]
fn match_query__and_retain_empty__authority_or_fills_pin__ac5()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let uuid = uuid::Uuid::new_v4();
    // Dumps AND-hit both tokens so T217 R0 is non-empty (early return).
    for i in 0..15 {
        append_pinned(
            &store,
            &format!("Here's the assessment. dump {i}\nt312or backend repeated body pad {i}"),
        )?;
    }
    let pin_id = append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t312\nDECISION: t312or {uuid} sqlite graph"),
    )?;

    let outcome = recall_full(
        store.connection(),
        None,
        "t312or backend",
        5,
        default_opts(),
    )?;
    assert!(!outcome.hits.is_empty(), "AC5: recall must return hits");
    assert_eq!(
        outcome.hits[0].memory_id,
        pin_id,
        "AC5: OR-filled pin must be hit #1; first={} content={:?} all={:?}",
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
        "AC5: hit #1 must be DECISION:; got {}",
        outcome.hits[0].content
    );
    assert!(
        !outcome.hits[0].content.contains("backend"),
        "AC5: pin must lack backend token (AND-miss); got {}",
        outcome.hits[0].content
    );
    Ok(())
}

/// T312 F40: when AND-retain already has an authority pin, OR-fill must not
/// displace it with an OR-only distractor.
#[test]
fn match_query__and_retain_nonempty__no_or_distractor__f40()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let uuid = uuid::Uuid::new_v4();
    // AND-hit pin (both tokens) — pass-1 retain nonempty.
    let and_pin = append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t312\nDECISION: t312or backend and-hit {uuid}"),
    )?;
    // OR-only distractor (has t312or, lacks backend) — would win if OR ran on nonempty.
    append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t312\nDECISION: t312or {uuid} or-only distractor"),
    )?;
    for i in 0..5 {
        append_pinned(
            &store,
            &format!("Here's the assessment. dump {i}\nt312or backend pad {i}"),
        )?;
    }

    let outcome = recall_full(
        store.connection(),
        None,
        "t312or backend",
        5,
        default_opts(),
    )?;
    assert!(!outcome.hits.is_empty(), "F40: recall must return hits");
    assert_eq!(
        outcome.hits[0].memory_id, and_pin,
        "F40: AND-retained pin must stay #1 (OR-only distractor must not displace); first={} content={:?}",
        outcome.hits[0].memory_id, outcome.hits[0].content
    );
    assert!(
        outcome.hits[0].content.contains("and-hit"),
        "F40: hit #1 must be the AND pin; got {}",
        outcome.hits[0].content
    );
    Ok(())
}

/// T312 AC14: `--semantic` hermetic F42 fixture — pin in top-3 lexical fallback.
#[test]
fn recall_full__semantic_f42_and_miss__pin_in_top3__ac14() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let uuid = uuid::Uuid::new_v4();
    for i in 0..15 {
        append_pinned(
            &store,
            &format!("Here's the assessment. dump {i}\nt312or backend repeated body pad {i}"),
        )?;
    }
    let pin_id = append_pinned(
        &store,
        &format!("ASSISTANT: TAGS: t312\nDECISION: t312or {uuid} sqlite graph"),
    )?;

    let mut opts = default_opts();
    opts.semantic = true;
    let outcome = recall_full(store.connection(), None, "t312or backend", 5, opts)?;
    let top3: Vec<&str> = outcome
        .hits
        .iter()
        .take(3)
        .map(|h| h.memory_id.as_str())
        .collect();
    assert!(
        top3.contains(&pin_id.as_str()),
        "AC14: pin must be in top-3 lexical fallback; top3={top3:?} embedding={:?}",
        outcome.embedding
    );
    Ok(())
}
