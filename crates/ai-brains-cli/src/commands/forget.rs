use crate::context::AppContext;
use ai_brains_core::ids::MemoryId;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, EventKind, MemoryForgottenPayload, MemoryRestoredPayload, Payload,
};
use ai_brains_retrieval::lexical_search;
use ai_brains_store::{EventStore, QueryStore};
use std::str::FromStr;

const PREVIEW_MAX_LEN: usize = 100;

fn truncate_preview(s: &str) -> String {
    if s.chars().count() <= PREVIEW_MAX_LEN {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(PREVIEW_MAX_LEN).collect();
        format!("{}...", truncated)
    }
}

pub fn run(
    ctx: &AppContext,
    memory_id: Option<String>,
    match_query: Option<String>,
    force: bool,
    list_forgotten: bool,
    restore: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    if list_forgotten {
        let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
            .ok()
            .and_then(|s| s.parse().ok());
        let memories = ctx.conn.list_forgotten_memories(project_id)?;
        if memories.is_empty() {
            println!("No forgotten memories.");
        } else {
            println!("Forgotten memories:");
            for (id, content) in &memories {
                let first_line = content.lines().next().unwrap_or(content);
                let truncated: String = first_line.chars().take(80).collect();
                println!("  {} — {}", id, truncated);
            }
        }
        return Ok(());
    }

    if let Some(restore_id) = restore {
        let memory_id = MemoryId::from_str(&restore_id)?;
        if dry_run {
            println!("[dry-run] Would restore memory {}.", memory_id);
            return Ok(());
        }
        let event = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            EventKind::MemoryRestored,
            Actor::User(ai_brains_core::ids::UserId::new()),
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryRestored(MemoryRestoredPayload { memory_id }))?;

        event_store.append_event(&event)?;
        println!("Memory {} restored.", memory_id);
        return Ok(());
    }

    if let Some(query) = match_query {
        let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
            .ok()
            .and_then(|s| s.parse().ok());
        let hits = lexical_search(&ctx.conn, &query, project_id, None)?;

        if hits.is_empty() {
            eprintln!(
                "No memories matching '{}'. Try broader search terms.",
                query
            );
            return Ok(());
        }

        if dry_run {
            let noun = if hits.len() == 1 {
                "memory"
            } else {
                "memories"
            };
            println!(
                "[dry-run] Would forget {} {} matching \"{}\":",
                hits.len(),
                noun,
                query
            );
            for hit in &hits {
                let preview = truncate_preview(hit.content.lines().next().unwrap_or(&hit.content));
                println!("  {} — {}", hit.memory_id, preview);
            }
            return Ok(());
        }

        if hits.len() == 1 {
            let hit = &hits[0];
            let first_line = hit.content.lines().next().unwrap_or(&hit.content);
            println!("Found: {} — {}", hit.memory_id, first_line);

            if !force {
                eprintln!("Use --force to forget this memory.");
                return Ok(());
            }

            let memory_id = MemoryId::from_str(&hit.memory_id)?;
            let event = EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
                EventKind::MemoryForgotten,
                Actor::User(ai_brains_core::ids::UserId::new()),
                Privacy::LocalOnly,
            )
            .build(Payload::MemoryForgotten(MemoryForgottenPayload {
                memory_id,
            }))?;

            event_store.append_event(&event)?;
            println!("Memory {} marked as forgotten.", memory_id);
        } else {
            println!("Found {} matching memories:", hits.len());
            for hit in &hits {
                let first_line: String = hit
                    .content
                    .lines()
                    .next()
                    .unwrap_or(&hit.content)
                    .chars()
                    .take(80)
                    .collect();
                println!("  {} — {}", hit.memory_id, first_line);
            }
            if !force {
                eprintln!("Use --force to forget all {} memories.", hits.len());
                return Ok(());
            }

            for hit in &hits {
                let memory_id = MemoryId::from_str(&hit.memory_id)?;
                let event = EventBuilder::new(
                    AggregateType::Memory,
                    memory_id.as_uuid(),
                    EventKind::MemoryForgotten,
                    Actor::User(ai_brains_core::ids::UserId::new()),
                    Privacy::LocalOnly,
                )
                .build(Payload::MemoryForgotten(MemoryForgottenPayload {
                    memory_id,
                }))?;
                event_store.append_event(&event)?;
            }
            println!("{} memories marked as forgotten.", hits.len());
        }
        return Ok(());
    }

    // Direct UUID forget
    if let Some(id_str) = memory_id {
        let memory_id = MemoryId::from_str(&id_str)?;

        // T77: validate that the memory_id exists in the projection before
        // appending an event that would otherwise silently match zero rows.
        if !ctx.conn.memory_exists(&id_str)? {
            return Err(format!(
                "Memory {} not found. Use 'forget --match' to search, \
                 or 'forget --list-forgotten' to see forgotten memories.",
                id_str
            )
            .into());
        }

        // Show what we're about to forget
        let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
            .ok()
            .and_then(|s| s.parse().ok());
        let hits = lexical_search(&ctx.conn, &id_str, project_id, None)?;
        let preview = hits
            .iter()
            .find(|h| h.memory_id == id_str)
            .map(|hit| truncate_preview(hit.content.lines().next().unwrap_or(&hit.content)));

        if dry_run {
            println!("[dry-run] Would forget memory {}.", id_str);
            if let Some(p) = preview {
                println!("  Preview: {}", p);
            }
            return Ok(());
        }

        if let Some(p) = preview {
            println!("Memory: {} — {}", id_str, p);
        }

        if !force {
            eprint!("Forget this memory? [y/N] ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                return Err("Forget cancelled.".into());
            }
        }

        let event = EventBuilder::new(
            AggregateType::Memory,
            memory_id.as_uuid(),
            EventKind::MemoryForgotten,
            Actor::User(ai_brains_core::ids::UserId::new()),
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryForgotten(MemoryForgottenPayload {
            memory_id,
        }))?;

        event_store.append_event(&event)?;
        println!("Memory {} marked as forgotten.", memory_id);
        return Ok(());
    }

    Err("Specify a memory ID, use --match to search, --list-forgotten to view, or --restore to recover.".into())
}
