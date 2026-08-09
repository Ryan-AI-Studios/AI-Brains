//! `ai-brains opencode-import` — batch import OpenCode sessions via list+export (T238).

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::{
    OpenCodeImportOptions, import_opencode_sessions, print_opencode_import_stats,
};
use ai_brains_capture::CaptureService;
use ai_brains_core::ids::ProjectId;
use std::str::FromStr;

pub fn run(
    ctx: &AppContext,
    days: usize,
    force: bool,
    dry_run: bool,
    max_sessions: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        eprintln!("Scanning for OpenCode sessions (dry-run — no vault writes)...");
    } else {
        eprintln!("Scanning for OpenCode sessions...");
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

    // Default project id from env is only used when allow_default_project is true.
    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    let options = OpenCodeImportOptions {
        days,
        force,
        dry_run,
        max_sessions,
        default_project_id: project_id,
        allow_default_project: false,
        list_json_override: None,
        export_json_override_dir: None,
        cursor_path_override: None,
        config_dir_override: None,
        force_missing_binary: false,
        list_cap: max_sessions,
    };

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let stats = import_opencode_sessions(query_store.as_ref(), &service, &mut sink, options)?;

    if let Some(err) = sink.last_error {
        return Err(format!("OpenCode import encountered an error: {err}").into());
    }

    print_opencode_import_stats(&stats);

    if dry_run {
        eprintln!(
            "OpenCode dry-run complete. found={} (sessions/imported_turns remain 0 — no writes).",
            stats.found
        );
    } else if stats.skipped_missing_binary > 0 && stats.sessions == 0 {
        eprintln!("OpenCode import soft-skipped (missing binary).");
    } else if stats.sessions == 0 {
        eprintln!("No new OpenCode sessions found to import.");
    } else {
        eprintln!(
            "OpenCode import complete. Processed {} turn(s) from {} session(s).",
            stats.imported_turns, stats.sessions
        );
    }

    Ok(())
}
