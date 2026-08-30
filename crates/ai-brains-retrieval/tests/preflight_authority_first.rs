//! T327 — Session Other cap, Recent prefer-authority, sealed skip.
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{build_preflight, word_count};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

const INDEX_EMPTY_AUTHORITY_SOOT: &str =
    "No DECISION/CONSTRAINT pins in scope; showing recent activity";

fn append_pinned(
    store: &SqliteEventStore,
    project_id: ProjectId,
    content: &str,
    privacy: Privacy,
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
        privacy,
    )
    .build(payload)?;
    store.append_event(&envelope)?;
    Ok(memory_id.to_string())
}

fn set_updated_at(
    store: &SqliteEventStore,
    memory_id: &str,
    ts: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = store.connection().lock()?;
    conn.execute(
        "UPDATE memory_projection SET updated_at = ?1 WHERE memory_id = ?2",
        rusqlite::params![ts, memory_id],
    )?;
    Ok(())
}

fn other_assistant_turn(i: usize) -> String {
    format!("Let me verify the SQL for the drain path item {i} now.")
}

/// T327 AC7 — USER + authority uncapped; Other assistant cap 3 + +K notice.
///
/// `active_sessions` loads `SESSION_TURN_FETCH` turns (`sessions.rs`). Other
/// assistant cap stays 3. Window = USER + 4 Other; a separate pinned DECISION
/// proves authority still appears in the assembled text (Index).
#[test]
fn preflight__session_other_turns__cap_3_plus_k_notice() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let (session_id, project_id) = common::append_active_session(&store)?;
    let sid = session_id.to_string();
    append_pinned(
        &store,
        project_id,
        "DECISION: T327 session pin stays uncapped in preview",
        Privacy::CloudOk,
    )?;
    common::append_turn(&store, &sid, "user", "What is the capital of France today?")?;
    for i in 1..=4 {
        common::append_turn(&store, &sid, "assistant", &other_assistant_turn(i))?;
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;

    assert!(
        ctx.text.contains("What is the capital of France today?"),
        "AC7: USER turn kept; text=\n{}",
        ctx.text
    );
    assert!(
        ctx.text
            .contains("DECISION: T327 session pin stays uncapped"),
        "AC7: authority pin in assembled text; text=\n{}",
        ctx.text
    );
    let session_part = ctx
        .text
        .split("--- Session:")
        .nth(1)
        .and_then(|s| s.split("--- Memory Index").next())
        .unwrap_or("");
    let other_emitted = (1..=4)
        .filter(|i| session_part.contains(&other_assistant_turn(*i)))
        .count();
    assert!(
        other_emitted <= 3,
        "AC7: Other assistant ≤ 3 (got {other_emitted}); text=\n{}",
        ctx.text
    );
    let skipped = 4usize.saturating_sub(other_emitted);
    assert!(
        skipped > 0,
        "AC7: expected skipped Other in the LIMIT-5 window; text=\n{}",
        ctx.text
    );
    let notice = format!("+{skipped} more session turns via recall");
    assert!(
        ctx.text.contains(&notice),
        "AC7: overflow notice {notice:?}; text=\n{}",
        ctx.text
    );
    Ok(())
}

/// T330 AC8 — USER + 8 Other under fetch 20: Other cap 3 + `+5 more`.
#[test]
fn preflight__session_eight_other_turns__cap_3_plus_5_notice()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let (session_id, project_id) = common::append_active_session(&store)?;
    let sid = session_id.to_string();
    common::append_turn(&store, &sid, "user", "What is the capital of France today?")?;
    for i in 1..=8 {
        common::append_turn(&store, &sid, "assistant", &other_assistant_turn(i))?;
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;

    let session_part = ctx
        .text
        .split("--- Session:")
        .nth(1)
        .and_then(|s| s.split("--- Memory Index").next())
        .unwrap_or("");
    assert!(
        session_part.contains("What is the capital of France today?"),
        "AC8: USER turn kept; session=\n{session_part}\nfull=\n{}",
        ctx.text
    );
    let other_emitted = (1..=8)
        .filter(|i| session_part.contains(&other_assistant_turn(*i)))
        .count();
    assert!(
        other_emitted <= 3,
        "AC8: Other assistant ≤ 3 (got {other_emitted}); session=\n{session_part}"
    );
    assert!(
        session_part.contains("+5 more session turns via recall"),
        "AC8: +5 notice (8 Other − cap 3); session=\n{session_part}"
    );
    Ok(())
}

/// T327 AC7 companion — authority assistant in the LIMIT-5 window is not Other-capped.
#[test]
fn preflight__session_authority_assistant__uncapped_among_other()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let (session_id, project_id) = common::append_active_session(&store)?;
    let sid = session_id.to_string();
    let needle = "T327 session authority assistant stays uncapped";
    common::append_turn(
        &store,
        &sid,
        "assistant",
        &format!("DECISION: {needle} in the live session window"),
    )?;
    for i in 1..=4 {
        common::append_turn(&store, &sid, "assistant", &other_assistant_turn(i))?;
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let session_part = ctx
        .text
        .split("--- Session:")
        .nth(1)
        .and_then(|s| s.split("--- Memory Index").next())
        .unwrap_or("");
    assert!(
        session_part.contains(needle) && session_part.contains("DECISION:"),
        "AC7: authority assistant must appear in Session (not Other-capped); session=\n{session_part}\nfull=\n{}",
        ctx.text
    );
    let other_emitted = (1..=4)
        .filter(|i| session_part.contains(&other_assistant_turn(*i)))
        .count();
    assert!(
        other_emitted <= 3,
        "AC7: Other assistant ≤ 3 (got {other_emitted}); session=\n{session_part}"
    );
    assert!(
        session_part.contains("+1 more session turns via recall"),
        "AC7: +1 notice for the skipped Other; session=\n{session_part}"
    );
    Ok(())
}

/// T327 AC9 — Recent prefers the older DECISION over newer Objective dumps.
#[test]
fn preflight__recent_prefers_decision_over_newer_objective()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T327-ac9-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let pin_id = append_pinned(
        &store,
        project_id,
        &format!("DECISION: {needle} recent-authority pin must surface"),
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    // Three newer dumps fill today's recency cap of 3 so the pin is omitted
    // unless Recent prefer-authority runs (F3).
    for i in 0..3 {
        let dump_id = append_pinned(
            &store,
            project_id,
            &format!("## Objective dump-{i} Let me verify the SQL path for recent chatter {i}"),
            Privacy::CloudOk,
        )?;
        set_updated_at(&store, &dump_id, &format!("2026-08-2{i}T12:00:00+00:00"))?;
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let header = "--- Most Recent Memories ---";
    let header_at = ctx
        .text
        .find(header)
        .unwrap_or_else(|| panic!("AC9: Recent header missing; text=\n{}", ctx.text));
    let after = &ctx.text[header_at + header.len()..];
    assert!(
        after.contains(&needle) && after.contains("DECISION:"),
        "AC9: needle/DECISION must appear after Recent header; after=\n{after}\nfull=\n{}",
        ctx.text
    );
    Ok(())
}

/// T327 AC10 — dumps-only Recent recency-fills; F4 is Index-only.
#[test]
fn preflight__recent_dumps_only__recency_fallback_no_f4() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    for i in 0..3 {
        append_pinned(
            &store,
            project_id,
            &format!("## Objective dump-{i} Let me verify the SQL path for recent chatter {i}"),
            Privacy::CloudOk,
        )?;
    }

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let header = "--- Most Recent Memories ---";
    assert!(
        ctx.text.contains(header),
        "AC10: Recent header present (recency fallback); text=\n{}",
        ctx.text
    );
    if let Some(at) = ctx.text.find(header) {
        let recent = &ctx.text[at..];
        assert!(
            !recent.contains(INDEX_EMPTY_AUTHORITY_SOOT),
            "AC10: F4 must not appear in Recent; recent=\n{recent}"
        );
    }
    Ok(())
}

/// T327 AC11 — Sealed DECISION never enters Index/Recent/text.
#[test]
fn preflight__sealed_decision__never_in_index_or_recent() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let sealed_needle = format!("T327-sealed-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    append_pinned(
        &store,
        project_id,
        &format!("DECISION: {sealed_needle} must not leak"),
        Privacy::Sealed,
    )?;
    append_pinned(
        &store,
        project_id,
        "## Objective\nReview dump with buried decision: in the skill body. padding word padding word padding word",
        Privacy::CloudOk,
    )?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    assert!(
        !ctx.text.contains(&sealed_needle),
        "AC11: sealed needle must be absent; text=\n{}",
        ctx.text
    );
    Ok(())
}

const RECENT_HEADER: &str = "--- Most Recent Memories ---";
const WHALE_BODY_TOKEN: &str = "T329WHALEBODY";

fn whale_authority_body(title_needle: &str) -> String {
    format!(
        "DECISION: {title_needle} oversized authority pin title\n{WHALE_BODY_TOKEN}\n{}",
        "padding word ".repeat(2000)
    )
}

fn small_authority_body(needle: &str) -> String {
    format!("DECISION: {needle} compact authority pin that must survive packing")
}

fn recent_slice(text: &str) -> Option<&str> {
    text.find(RECENT_HEADER)
        .map(|at| &text[at + RECENT_HEADER.len()..])
}

/// T329 AC1 — whale + small at max_words=250: Recent header + small survive; whale body dropped.
#[test]
fn preflight__recent_whale_authority_plus_small__header_and_small_survive()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let whale_title = format!("T329-whale-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let small_needle = format!("T329-small-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let small_id = append_pinned(
        &store,
        project_id,
        &small_authority_body(&small_needle),
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &small_id, "2020-01-01T00:00:00+00:00")?;
    let whale_id = append_pinned(
        &store,
        project_id,
        &whale_authority_body(&whale_title),
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &whale_id, "2026-08-30T12:00:00+00:00")?;

    let ctx = build_preflight(store.connection(), None, 250, Some(project_id), None, false)?;

    let header_at = ctx
        .text
        .find(RECENT_HEADER)
        .unwrap_or_else(|| panic!("AC1: Recent header missing; text=\n{}", ctx.text));
    let after = &ctx.text[header_at + RECENT_HEADER.len()..];
    assert!(
        after.contains(&small_needle) && after.contains("DECISION:"),
        "AC1: small DECISION must appear after Recent header; after=\n{after}\nfull=\n{}",
        ctx.text
    );
    assert!(
        !after.contains(WHALE_BODY_TOKEN),
        "AC1: whale body token must be absent from Recent; after=\n{after}"
    );
    let before = &ctx.text[..header_at];
    assert!(
        before.contains("DECISION:") && before.contains(&whale_title),
        "AC1: Index numbered lines still contain whale DECISION title; before=\n{before}"
    );
    Ok(())
}

/// T330 AC11 — whale-only at max_words=250: Recent header + snippet; Index title remains.
#[test]
fn preflight__recent_whale_only__header_and_snippet_under_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let whale_title = format!("T329-whaleonly-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    append_pinned(
        &store,
        project_id,
        &whale_authority_body(&whale_title),
        Privacy::CloudOk,
    )?;

    let ctx = build_preflight(store.connection(), None, 250, Some(project_id), None, false)?;

    let header_at = ctx
        .text
        .find(RECENT_HEADER)
        .unwrap_or_else(|| panic!("AC11: Recent header missing; text=\n{}", ctx.text));
    let recent = &ctx.text[header_at..];
    assert!(
        word_count(recent) <= 250,
        "AC11: Recent section must be a snippet (wc={} > 250); recent=\n{recent}",
        word_count(recent)
    );
    let body = recent
        .split(RECENT_HEADER)
        .nth(1)
        .unwrap_or(recent)
        .split("(Use 'recall' to fetch details for other index items)")
        .next()
        .unwrap_or(recent);
    assert!(
        word_count(body) >= 8,
        "AC11: snippet body must be ≥ MIN_RECENT_BODY_WORDS; wc={} body=\n{body}",
        word_count(body)
    );
    assert!(
        body.contains(&whale_title) && body.contains("DECISION:"),
        "AC11: snippet must retain first-item whale title; body=\n{body}"
    );
    let full_blob = "padding word ".repeat(2000);
    assert!(
        !recent.contains(full_blob.trim()),
        "AC11: Recent must not be the full whale blob; recent=\n{recent}"
    );
    let before = &ctx.text[..header_at];
    assert!(
        before.contains("DECISION:") && before.contains(&whale_title),
        "AC11: Index still contains whale DECISION title; before=\n{before}"
    );
    Ok(())
}

/// T330 AC19 — empty vault (no project): empty_repo banner; no F4 / Index / Recent.
#[test]
fn preflight__empty_vault__empty_repo_banner_no_index_f4_recent()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let ctx = build_preflight(store.connection(), None, 1500, None, None, false)?;
    assert!(
        ctx.text
            .contains("--- AI-Brains: New Repository Detected ---"),
        "AC19: empty_repo banner; text=\n{}",
        ctx.text
    );
    assert!(
        !ctx.text.contains(INDEX_EMPTY_AUTHORITY_SOOT),
        "AC19: F4 must not leak into empty_repo; text=\n{}",
        ctx.text
    );
    assert!(
        !ctx.text.contains("--- Memory Index (Briefing) ---"),
        "AC19: no Memory Index header; text=\n{}",
        ctx.text
    );
    assert!(
        !ctx.text.contains(RECENT_HEADER),
        "AC19: no Recent header; text=\n{}",
        ctx.text
    );
    Ok(())
}

/// T329 AC4 — whale + small at max_words=8000: both bodies appear in Recent.
#[test]
fn preflight__recent_whale_plus_small__large_budget_keeps_whale_body()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let whale_title = format!("T329-fit-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let small_needle = format!("T329-fitsmall-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let small_id = append_pinned(
        &store,
        project_id,
        &small_authority_body(&small_needle),
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &small_id, "2020-01-01T00:00:00+00:00")?;
    let whale_id = append_pinned(
        &store,
        project_id,
        &whale_authority_body(&whale_title),
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &whale_id, "2026-08-30T12:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        8000,
        Some(project_id),
        None,
        false,
    )?;

    let after = recent_slice(&ctx.text)
        .unwrap_or_else(|| panic!("AC4: Recent header missing; text=\n{}", ctx.text));
    assert!(
        after.contains(&small_needle),
        "AC4: small needle in Recent; after=\n{after}"
    );
    assert!(
        after.contains(WHALE_BODY_TOKEN),
        "AC4: whale body token must appear when budget fits; after=\n{after}"
    );
    Ok(())
}
