use crate::commands::memory::{MemoryListOptions, preview_line, run_inventory};
use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, MemoryForgottenPayload, MemoryRestoredPayload, Payload,
};
use ai_brains_retrieval::{LexicalSearchOptions, lexical_search};
use ai_brains_store::{EventStore, QueryStore};
use std::str::FromStr;

/// Human match / UUID preview budget (T224 F5 — dry-run, single Found, UUID).
const FORGET_PREVIEW_MAX: usize = 100;
/// Multi-match list budget (T224 F5 / M4 — shorter skim; gains `…` via preview_line).
const FORGET_MULTI_PREVIEW_MAX: usize = 80;

/// Forget command. List-forgotten shares the inventory backend (T216 F1/F28).
#[allow(clippy::too_many_arguments)] // clap dispatch surface; list flags share inventory backend
pub fn run(
    ctx: &AppContext,
    memory_id: Option<String>,
    match_query: Option<String>,
    force: bool,
    list_forgotten: bool,
    restore: Option<String>,
    dry_run: bool,
    // List flags when --list-forgotten (F28)
    global: bool,
    limit: Option<usize>,
    format: String,
    tag: Option<String>,
    project_id: Option<ProjectId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    if list_forgotten {
        // F1/F28: forget --list-forgotten ≡ memory list --status forgotten (+ same flags).
        // Clap-passed project_id — no raw env::var on list path (F4).
        let effective_project_id = if global { None } else { project_id };
        return run_inventory(
            ctx,
            MemoryListOptions {
                status: "forgotten".to_string(),
                limit,
                global,
                format,
                summary: false,
                tag,
                project_id: effective_project_id,
            },
        );
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
        // T217: forget stays strict R0 (rescue: false) — never OR-widen match.
        let hits = lexical_search(
            &ctx.conn,
            &query,
            project_id,
            None,
            LexicalSearchOptions::default(),
        )?;

        if hits.is_empty() {
            tracing::info!(
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
                // T224 F5: shared preview_line SOOT (strip + first non-empty + …).
                let preview = preview_line(&hit.content, FORGET_PREVIEW_MAX);
                println!("  {} — {}", hit.memory_id, preview);
            }
            return Ok(());
        }

        if hits.len() == 1 {
            let hit = &hits[0];
            let preview = preview_line(&hit.content, FORGET_PREVIEW_MAX);
            println!("Found: {} — {}", hit.memory_id, preview);

            if !force {
                tracing::info!("Use --force to forget this memory.");
                return Ok(());
            }

            let memory_id = MemoryId::from_str(&hit.memory_id)?;
            let event = EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
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
                // T224 F5/M4: max 80 + role strip; intentional … on cut (was raw 80, no ellipsis).
                let preview = preview_line(&hit.content, FORGET_MULTI_PREVIEW_MAX);
                println!("  {} — {}", hit.memory_id, preview);
            }
            if !force {
                tracing::info!("Use --force to forget all {} memories.", hits.len());
                return Ok(());
            }

            for hit in &hits {
                let memory_id = MemoryId::from_str(&hit.memory_id)?;
                let event = EventBuilder::new(
                    AggregateType::Memory,
                    memory_id.as_uuid(),
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
        let hits = lexical_search(
            &ctx.conn,
            &id_str,
            project_id,
            None,
            LexicalSearchOptions::default(),
        )?;
        let preview = hits
            .iter()
            .find(|h| h.memory_id == id_str)
            .map(|hit| preview_line(&hit.content, FORGET_PREVIEW_MAX));

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
