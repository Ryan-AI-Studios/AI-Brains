//! T259 — `project rebind-path`: move one path alias to another project.
//!
//! Default is print-only. `--write --yes` appends
//! `RepositoryPathAliasRemoved` + `RepositoryPathAliasAdded` in one store
//! transaction. Historical memories stay on the from-project.

use crate::commands::governed_common::fail_usage;
use crate::commands::project_paths::resolve_project_ref;
use crate::context::AppContext;
use ai_brains_store::QueryStore;
use serde::Serialize;
use std::io::IsTerminal;

#[derive(Debug, Serialize)]
struct RebindPathJson {
    api_version: String,
    path: String,
    from_project_id: String,
    to_project_id: String,
    already_bound: bool,
    written: bool,
    memories_moved: bool,
    events_appended: u32,
}

/// Print-only remediator, or confirmable one-tx path rebind.
pub fn run(
    ctx: &AppContext,
    path: &str,
    to: &str,
    write: bool,
    yes: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if write && !yes {
        return fail_usage("--write requires --yes (no silent path rebind)");
    }

    let normalized = ai_brains_path::normalize_for_location_compare(path);
    if normalized.is_empty() {
        return fail_usage("path normalized to empty; choose a non-empty filesystem path.");
    }

    let dest = resolve_project_ref(ctx, to)?;
    let owner = ctx.conn.find_path_alias_owner(&normalized)?;
    let Some(from) = owner else {
        eprintln!(
            "Path alias '{normalized}' is not registered. Run `ai-brains project register-path <project-id> <path>` first."
        );
        return Err("no path owner; register-path required".into());
    };

    let already_bound = from == dest;
    let written = if write && yes && !already_bound {
        let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        let writer = ai_brains_control_plane::StoreEventWriter::new(event_store);
        ai_brains_control_plane::rebind_path_alias(&writer, path, from, dest)
            .map_err(|e| format!("rebind path alias failed: {e}"))?;
        true
    } else {
        false
    };
    let events_appended = if written { 2 } else { 0 };
    let from_id = from.to_string();
    let to_id = dest.to_string();

    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());
    if use_json {
        let report = RebindPathJson {
            api_version: "1".to_string(),
            path: normalized,
            from_project_id: from_id,
            to_project_id: to_id,
            already_bound,
            written,
            memories_moved: false,
            events_appended,
        };
        crate::commands::identity_warn::print_json_stdout(&report)?;
        return Ok(());
    }

    if already_bound {
        println!("Already bound to {to_id}");
        println!("No path events.");
        return Ok(());
    }

    if written {
        println!("Rebound path alias {normalized}");
        println!("from: {from_id}");
        println!("to:   {to_id}");
        println!("Historical memories stay on {from_id} (memories_moved=false).");
        println!("Nightly Phase 2 will stop walking this path for the from-project.");
    } else {
        println!("Would rebind path alias {normalized}");
        println!("from: {from_id}");
        println!("to:   {to_id}");
        println!("Historical memories stay on {from_id} (memories_moved=false).");
        println!("Nightly Phase 2 would stop walking this path for the from-project.");
        println!("Re-run with --write --yes to apply.");
    }
    Ok(())
}
