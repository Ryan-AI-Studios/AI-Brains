//! `ai-brains cursor-import` — batch import Cursor agent-transcripts JSONL (T334).

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::{CursorImportOptions, import_cursor_sessions, print_cursor_import_stats};
use ai_brains_capture::CaptureService;
use ai_brains_core::ids::ProjectId;
use std::str::FromStr;

pub fn run(
    ctx: &AppContext,
    days: usize,
    force: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        eprintln!("Scanning for Cursor sessions (dry-run — no vault writes)...");
    } else {
        eprintln!("Scanning for Cursor sessions...");
    }

    let service = CaptureService::new();
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(
            std::sync::Arc::clone(&ctx.conn),
        )),
    };

    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    let options = CursorImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force,
        home_override: None,
        dry_run,
    };

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let stats = import_cursor_sessions(query_store.as_ref(), &service, &mut sink, options)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Cursor import encountered an error: {err}").into());
    }

    print_cursor_import_stats(&stats);

    if dry_run {
        eprintln!(
            "Cursor dry-run complete. found={} (sessions/imported_turns remain 0 — no writes).",
            stats.found
        );
    } else if stats.sessions == 0 {
        eprintln!("No new Cursor sessions found to import.");
    } else {
        eprintln!(
            "Cursor import complete. Processed {} turn(s) from {} session(s).",
            stats.imported_turns, stats.sessions
        );
    }

    Ok(())
}
