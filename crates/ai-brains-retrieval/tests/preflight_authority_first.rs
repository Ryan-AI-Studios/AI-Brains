//! T327 — Session Other cap, Recent prefer-authority, sealed skip.
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::build_preflight;
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
/// `active_sessions` loads `LIMIT 5` turns (`sessions.rs`). Do not raise that
/// cap (F19). Window = USER + 4 Other; a separate pinned DECISION proves
/// authority still appears in the assembled text (Index).
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
