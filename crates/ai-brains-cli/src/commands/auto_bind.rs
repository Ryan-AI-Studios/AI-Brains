//! T344 — fail-closed `context` auto-bind (git toplevel path + unique-slug alias).
//!
//! Queries (doctor / default preflight / `--show`) must not call this.
//! `preflight --summary --bind` is the explicit writer. Never `process::exit`.

use crate::commands::project::{
    collect_git_identity, resolve_path_alias_for_location, sanitize_alias_suggestion,
};
use crate::context::AppContext;
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload, ProjectAliasAddedPayload};
use ai_brains_path::normalize_for_location_compare;
use ai_brains_store::{EventStore, QueryStore};

pub struct AutoBindOpts {
    pub no_auto_bind: bool,
    /// T352 F9: success announcements go to stderr so JSON stdout stays clean.
    pub json_stdout: bool,
}

fn env_no_auto_bind_truthy() -> bool {
    match std::env::var("AI_BRAINS_NO_AUTO_BIND") {
        Ok(v) => {
            let t = v.trim();
            t.eq_ignore_ascii_case("1")
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn argv_no_project_context() -> bool {
    std::env::args().any(|a| a == "--no-project-context")
}

fn skip(reason: &str) {
    eprintln!("auto-bind skip: {reason}");
}

pub(crate) fn skip_no_project_id() {
    skip("no project id");
}

pub(crate) fn warn_auto_bind(result: Result<(), Box<dyn std::error::Error>>) {
    if let Err(e) = result {
        tracing::warn!(error = %e, "auto-bind failed");
    }
}

fn announce(msg: String, json_stdout: bool) {
    if json_stdout {
        eprintln!("{msg}");
    } else {
        println!("{msg}");
    }
}

fn exact_slug_hit_other(
    projects: &[(String, String, String, usize)],
    slug: &str,
    self_id: &str,
) -> bool {
    projects.iter().any(|(pid, name, alias, _)| {
        pid != self_id
            && (name.eq_ignore_ascii_case(slug)
                || (!alias.is_empty() && alias.eq_ignore_ascii_case(slug)))
    })
}

fn exact_slug_hit_self(
    projects: &[(String, String, String, usize)],
    slug: &str,
    self_id: &str,
) -> bool {
    projects.iter().any(|(pid, name, alias, _)| {
        pid == self_id
            && (name.eq_ignore_ascii_case(slug)
                || (!alias.is_empty() && alias.eq_ignore_ascii_case(slug)))
    })
}

/// Best-effort bind. Skips print one stderr line and return `Ok`. Real IO/event
/// failures return `Err` so `context` can warn and still exit 0.
pub fn maybe_auto_bind(
    ctx: &AppContext,
    project_id: ProjectId,
    opts: AutoBindOpts,
) -> Result<(), Box<dyn std::error::Error>> {
    if opts.no_auto_bind || env_no_auto_bind_truthy() || argv_no_project_context() {
        skip("disabled");
        return Ok(());
    }

    let cwd = std::env::current_dir()?;

    let git = collect_git_identity(&cwd)?;
    let Some(ref toplevel) = git.toplevel else {
        skip("no git");
        return Ok(());
    };

    let pid_str = project_id.to_string();
    let projects = ctx.conn.list_projects()?;
    if !projects.iter().any(|(pid, _, _, _)| pid == &pid_str) {
        skip("project not in vault");
        return Ok(());
    }

    let owner = resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)?;
    let owned_by_other = owner
        .as_ref()
        .is_some_and(|o| o.as_str() != pid_str.as_str());

    if owned_by_other {
        skip("path owned by other project");
        return Ok(());
    }

    if owner.is_none() {
        let raw = toplevel.to_string_lossy();
        let normalized = normalize_for_location_compare(&raw);
        if normalized.is_empty() {
            skip("empty path");
            return Ok(());
        }
        if let Some(existing) = ctx.conn.find_path_alias_owner(&normalized)? {
            if existing != project_id {
                skip("path owned by other project");
                return Ok(());
            }
        } else {
            let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
            let writer = ai_brains_control_plane::StoreEventWriter::new(event_store);
            ai_brains_control_plane::register_path_alias(&writer, &raw, project_id)?;
            announce(
                format!("Auto-bound path {normalized} for project {pid_str}"),
                opts.json_stdout,
            );
        }
    }

    let row = projects.iter().find(|(pid, _, _, _)| pid == &pid_str);
    let alias = row.map(|r| r.2.as_str()).unwrap_or("");
    if !alias.is_empty() {
        skip("alias present");
        return Ok(());
    }

    let Some(slug_raw) = git.slug.as_deref().filter(|s| !s.is_empty()) else {
        skip("slug empty");
        return Ok(());
    };
    let suggestion = sanitize_alias_suggestion(slug_raw);
    if suggestion.is_empty() {
        skip("slug empty");
        return Ok(());
    }

    let projects = ctx.conn.list_projects()?;
    if exact_slug_hit_self(&projects, &suggestion, &pid_str) {
        skip("slug already self");
        return Ok(());
    }
    if exact_slug_hit_other(&projects, &suggestion, &pid_str) {
        skip("slug taken");
        return Ok(());
    }

    if let Some(existing) = ctx.conn.resolve_project_id_from_alias(&suggestion)? {
        if existing == project_id {
            skip("slug already self");
            return Ok(());
        }
        skip("slug taken");
        return Ok(());
    }

    let event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
        project_id,
        alias: suggestion.clone(),
    }))?;
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    event_store.append_event(&event)?;
    announce(
        format!("Auto-set alias '{suggestion}' for project {pid_str}"),
        opts.json_stdout,
    );
    Ok(())
}
