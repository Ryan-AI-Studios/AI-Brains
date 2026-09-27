//! `ai-brains codex-import` — batch import Codex rollout JSONL (T253, fail-open).

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::{
    CodexImportOptions, import_codex_sessions, print_codex_import_stats,
    resolve_claude_import_scope_paths,
};
use ai_brains_capture::CaptureService;
use ai_brains_core::ids::ProjectId;
use ai_brains_path::normalize_for_location_compare;
use ai_brains_store::QueryStore;
use std::str::FromStr;

pub fn run(
    ctx: &AppContext,
    days: usize,
    force: bool,
    dry_run: bool,
    global: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        eprintln!("Scanning for Codex sessions (dry-run — no vault writes)...");
    } else {
        eprintln!("Scanning for Codex sessions...");
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

    let parsed_pid = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok());
    let project_id = parsed_pid.unwrap_or_default();

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn QueryStore>;
    let scope_paths = if global {
        None
    } else {
        let aliases = query_store.list_path_aliases()?;
        let fallback = std::env::current_dir().ok().map(|cwd| {
            let use_path = super::project::collect_git_identity(&cwd)
                .ok()
                .and_then(|g| g.toplevel)
                .unwrap_or(cwd);
            normalize_for_location_compare(&use_path.to_string_lossy())
        });
        resolve_claude_import_scope_paths(&aliases, parsed_pid, fallback.as_deref())
    };

    let options = CodexImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force,
        home_override: None,
        dry_run,
        scope_paths,
    };
    let stats = import_codex_sessions(query_store.as_ref(), &service, &mut sink, options)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Codex import encountered an error: {err}").into());
    }

    print_codex_import_stats(&stats);

    if dry_run {
        eprintln!(
            "Codex dry-run complete. found={} (sessions/imported_turns remain 0 — no writes).",
            stats.found
        );
    } else if stats.sessions == 0 {
        eprintln!("No new Codex sessions found to import.");
    } else {
        eprintln!(
            "Codex import complete. Processed {} turn(s) from {} session(s).",
            stats.imported_turns, stats.sessions
        );
    }

    Ok(())
}
