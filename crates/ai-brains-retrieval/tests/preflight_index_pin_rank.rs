//! T274 AC6 — Index prefers a leading DECISION pin over a newer session dump.
#![allow(non_snake_case)]

mod common;

use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{build_preflight, word_count};
use ai_brains_store::event_store::{EventStore, SqliteEventStore};

fn append_pinned(
    store: &SqliteEventStore,
    project_id: ProjectId,
    content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    append_pinned_id(
        store,
        project_id,
        MemoryId::new(),
        content,
        Privacy::CloudOk,
    )
}

fn append_pinned_id(
    store: &SqliteEventStore,
    project_id: ProjectId,
    memory_id: MemoryId,
    content: &str,
    privacy: Privacy,
) -> Result<String, Box<dyn std::error::Error>> {
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

const INDEX_EMPTY_AUTHORITY_SOOT: &str =
    "No DECISION/CONSTRAINT pins in scope; showing recent activity";

fn numbered_index_lines(text: &str) -> Vec<&str> {
    let after = text
        .split("--- Memory Index (Briefing) ---")
        .nth(1)
        .unwrap_or("");
    let until_recent = after
        .split("--- Most Recent Memories ---")
        .next()
        .unwrap_or(after);
    until_recent
        .lines()
        .map(str::trim)
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_digit()) && l.contains(". "))
        .collect()
}

/// T327 AC1 — whale-first full-body `break` must not hide a later small pin.
#[test]
fn preflight__index_whale_decision_then_small_pin__small_pin_is_item_1()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T327-ac1-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let whale = format!(
        "DECISION: whale-not-needle padding follows\n{}",
        "padding word ".repeat(2000)
    );
    let small = format!("DECISION: {needle} small fitting pin must remain visible");
    let dump = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let whale_id = append_pinned(&store, project_id, &whale)?;
    let small_id = append_pinned(&store, project_id, &small)?;
    let dump_id = append_pinned(&store, project_id, &dump)?;
    // whale_ts > small_pin_ts so ORDER hits the whale first; dump is newest.
    set_updated_at(&store, &small_id, "2026-08-10T00:00:00+00:00")?;
    set_updated_at(&store, &whale_id, "2026-08-20T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-25T00:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains("DECISION:") && !first.contains("## Objective"),
        "AC1: Index line 1 must be a pin (whale body must not abort pass-1); line={first:?}\n{}",
        ctx.text
    );
    assert!(
        ctx.text.contains(&needle) && ctx.text.contains("DECISION:"),
        "AC1: small pin must still enter after the whale (no full-body break); text=\n{}",
        ctx.text
    );
    Ok(())
}

/// T330 AC1 — short leading DECISION (word_count < 6) is Index 1; F4 absent.
#[test]
fn preflight__index_short_decision_under_six_words__is_item_1()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T330a1{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let pin_content = format!("DECISION: {needle}");
    assert!(
        word_count(&pin_content) < 6,
        "AC1: stored pin must be low-signal by today's meter; wc={} content={pin_content}",
        word_count(&pin_content)
    );
    let dump_body = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let pin_id = append_pinned(&store, project_id, &pin_content)?;
    let dump_id = append_pinned(&store, project_id, &dump_body)?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-30T12:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains(&needle) && first.contains("DECISION:"),
        "AC1: Index line 1 must be the short DECISION pin; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        !ctx.text.contains(INDEX_EMPTY_AUTHORITY_SOOT),
        "AC1: F4 must be absent when a short pin is collected; text=\n{}",
        ctx.text
    );
    Ok(())
}

/// T330 AC2 — capture wrap ASSISTANT: DECISION: {needle} still Index 1 (wc < 6).
#[test]
fn preflight__index_assistant_wrapped_short_decision__is_item_1()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T330a2{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let pin_content = format!("ASSISTANT: DECISION: {needle}");
    assert!(
        word_count(&pin_content) < 6,
        "AC2: wrapped pin must be low-signal by today's meter; wc={} content={pin_content}",
        word_count(&pin_content)
    );
    let dump_body = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let pin_id = append_pinned(&store, project_id, &pin_content)?;
    let dump_id = append_pinned(&store, project_id, &dump_body)?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-30T12:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains(&needle) && first.contains("DECISION:"),
        "AC2: Index line 1 must be the wrapped short DECISION pin; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        !ctx.text.contains(INDEX_EMPTY_AUTHORITY_SOOT),
        "AC2: F4 must be absent; text=\n{}",
        ctx.text
    );
    Ok(())
}

/// T327 AC3 / T330 AC3 — chrome-only: F4 once; numbered Index must not be ## Objective.
#[test]
fn preflight__index_no_authority__f4_honesty_line_once() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    append_pinned(
        &store,
        project_id,
        "## Objective\nReview dump with buried decision: in the skill body. padding word padding word padding word",
    )?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let f4 = ctx.text.matches(INDEX_EMPTY_AUTHORITY_SOOT).count();
    assert_eq!(f4, 1, "AC3: F4 SOOT exactly once; text=\n{}", ctx.text);
    let numbered = numbered_index_lines(&ctx.text);
    assert!(
        numbered.iter().all(|l| !l.contains("## Objective")),
        "AC3: numbered Index must not contain ## Objective; numbered={numbered:?}\n{}",
        ctx.text
    );
    Ok(())
}

/// T330 AC4 — newer chrome + older non-chrome Other: F4 + item 1 is the Other.
#[test]
fn preflight__index_chrome_plus_other__other_is_item_1_with_f4()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let other_body = "Let me verify the SQL for the drain path now.";
    assert!(
        word_count(other_body) >= 6,
        "AC4: Other body must survive low-signal; wc={}",
        word_count(other_body)
    );
    let dump_body = format!(
        "## Objective\nReview dump with buried decision: in the skill body. {}",
        "padding word ".repeat(80)
    );
    let other_id = append_pinned(&store, project_id, other_body)?;
    let dump_id = append_pinned(&store, project_id, &dump_body)?;
    set_updated_at(&store, &other_id, "2020-01-01T00:00:00+00:00")?;
    set_updated_at(&store, &dump_id, "2026-08-30T12:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    assert!(
        ctx.text.contains(INDEX_EMPTY_AUTHORITY_SOOT),
        "AC4: F4 present (no authority pin); text=\n{}",
        ctx.text
    );
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains("Let me verify") && !first.contains("## Objective"),
        "AC4: Index line 1 is the non-chrome Other; line={first:?}\n{}",
        ctx.text
    );
    Ok(())
}

/// T327 AC4 — one pin + 20 dumps → pin first, at most 15 numbered Index lines.
#[test]
fn preflight__index_one_pin_plus_dumps__pin_first_slot_cap_15()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let needle = format!("T327-ac4-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let pin_id = append_pinned(
        &store,
        project_id,
        &format!("DECISION: {needle} slot-cap pin must remain first"),
    )?;
    set_updated_at(&store, &pin_id, "2020-01-01T00:00:00+00:00")?;
    for i in 0..20 {
        let dump_id = append_pinned(
            &store,
            project_id,
            &format!("Let me verify the SQL path for chatter item {i} extra words now."),
        )?;
        set_updated_at(
            &store,
            &dump_id,
            &format!("2026-08-{:02}T12:00:00+00:00", i + 1),
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
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains(&needle) && first.contains("DECISION:"),
        "AC4: Index line 1 is the pin; line={first:?}\n{}",
        ctx.text
    );
    let numbered = numbered_index_lines(&ctx.text);
    assert_eq!(
        numbered.len(),
        15,
        "AC4: Index slot cap fills remaining slots to 15; got {} lines\n{}",
        numbered.len(),
        ctx.text
    );
    Ok(())
}

/// T327 AC6 — same-tick pins: lexicographically smaller memory_id is Index 1.
#[test]
fn preflight__index_same_tick__lower_memory_id_first() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::empty_store()?;
    let project_id = ProjectId::from_uuid(uuid::Uuid::nil());
    let id_a = MemoryId::from_uuid(uuid::Uuid::parse_str(
        "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    )?);
    let id_b = MemoryId::from_uuid(uuid::Uuid::parse_str(
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    )?);
    assert!(
        id_a.to_string() < id_b.to_string(),
        "F48: seed A lex-smaller than B"
    );
    // Insert B first so rowid order would prefer B without memory_id ASC.
    append_pinned_id(
        &store,
        project_id,
        id_b,
        "DECISION: T327-ac6-B later-lex pin must stay second",
        Privacy::CloudOk,
    )?;
    append_pinned_id(
        &store,
        project_id,
        id_a,
        "DECISION: T327-ac6-A earlier-lex pin must win the tie",
        Privacy::CloudOk,
    )?;
    set_updated_at(&store, &id_a.to_string(), "2026-08-15T00:00:00+00:00")?;
    set_updated_at(&store, &id_b.to_string(), "2026-08-15T00:00:00+00:00")?;

    let ctx = build_preflight(
        store.connection(),
        None,
        1500,
        Some(project_id),
        None,
        false,
    )?;
    let first = first_numbered_index_line(&ctx.text);
    assert!(
        first.contains("T327-ac6-A") && first.contains("DECISION:"),
        "AC6: same-tick Index 1 is lex-smaller memory_id A; line={first:?}\n{}",
        ctx.text
    );
    assert!(
        !first.contains("T327-ac6-B"),
        "AC6: B must not win the tie; line={first:?}\n{}",
        ctx.text
    );
    Ok(())
}
