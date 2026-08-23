//! T274 AC6 — Index prefers a leading DECISION pin over a newer session dump.
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::build_preflight;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

fn append_pinned(
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

#[test]
fn preflight__index_prefers_leading_decision_over_objective_dump()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T274-index-needle-{}", uuid::Uuid::new_v4());
    let pin_content = format!("DECISION: {needle} Index must list this pin");
    let dump_body = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let pin_id = append_pinned(&store, project_id, &pin_content)?;
    let dump_id = append_pinned(&store, project_id, &dump_body)?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-21T12:00:00+00:00")?;

    let ctx = build_preflight(store.connection(), None, 60, Some(project_id), None, false)?;

    assert!(
        ctx.text.contains("DECISION:") && ctx.text.contains("T274-index-needle-"),
        "AC6: Index must contain the leading DECISION pin; text=\n{}",
        ctx.text
    );
    let decision_count = ctx.text.matches("DECISION:").count();
    assert!(
        decision_count >= 1,
        "AC7: assembled window in_context_decisions >= 1; text=\n{}",
        ctx.text
    );
    Ok(())
}

fn first_numbered_index_line(text: &str) -> &str {
    let after = text
        .split("--- Memory Index (Briefing) ---")
        .nth(1)
        .unwrap_or(text);
    after
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("1."))
        .unwrap_or("")
}

/// T286 AC1/AC2/AC12 — tagged TAGS envelope pin vs a newer `## Objective` dump.
#[test]
fn preflight__index_prefers_tags_envelope_decision_over_objective_dump()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T286i-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let pin_content = format!("ASSISTANT: TAGS: t286\nDECISION: {needle} pin");
    let dump_body = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let pin_id = append_pinned(&store, project_id, &pin_content)?;
    let dump_id = append_pinned(&store, project_id, &dump_body)?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-23T12:00:00+00:00")?;

    let ctx = build_preflight(store.connection(), None, 60, Some(project_id), None, false)?;

    assert!(
        ctx.text.contains("DECISION:") && ctx.text.contains(&needle),
        "AC1: Index must contain the tagged DECISION pin; text=\n{}",
        ctx.text
    );
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        !first.contains("## Objective"),
        "AC1: first numbered Index line must not be ## Objective; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        first.contains("DECISION:"),
        "AC2: first numbered Index line must title DECISION:; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        !first.contains("TAGS:"),
        "AC2: envelope TAGS: must not be the Index title; line={first:?}\n{}",
        ctx.text
    );
    Ok(())
}

/// T286 AC10 — TAGS-only envelope (no following content line) titles Untitled Memory.
#[test]
fn preflight__index_tags_only_envelope__untitled_memory() -> Result<(), Box<dyn std::error::Error>>
{
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    append_pinned(
        &store,
        project_id,
        "ASSISTANT: TAGS: one two three four five six",
    )?;

    let ctx = build_preflight(store.connection(), None, 60, Some(project_id), None, false)?;
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains("Untitled Memory"),
        "AC10: empty envelope title is Untitled Memory; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        !first.contains("TAGS:"),
        "AC10: TAGS-only must not be the title; line={first:?}\n{}",
        ctx.text
    );
    Ok(())
}
