//! `ai-brains opencode-hook` — live session.idle capture from OpenCode (T238).

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::opencode::{
    OPENCODE_HARNESS_UUID, OPENCODE_UNBOUND_ALIAS, OPENCODE_UNBOUND_DISPLAY_NAME,
    append_opencode_turns, export_session_via_cli, normalize_opencode_project_hash,
    opencode_env_fallback_allowed, parse_export_file, parse_export_json, session_id_from_opencode,
};
use ai_brains_adapters::path_derived_display_name;
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_core::ids::{HarnessId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use ai_brains_store::EventStore;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeHookPayload {
    pub session_id: String,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub worktree: Option<String>,
    #[serde(default)]
    pub project_hash: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub export_path: Option<String>,
    #[serde(default)]
    pub messages_path: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
}

pub fn run(ctx: &AppContext, payload_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload: OpenCodeHookPayload = serde_json::from_str(payload_json)?;

    // F10: child/subagent skip
    if let Some(parent) = payload
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        eprintln!(
            "[ai-brains-opencode] skipped_child_session parentId={parent} session={}",
            payload.session_id
        );
        return Ok(());
    }

    let session_id = session_id_from_opencode(&payload.session_id);
    let oc_harness = HarnessId::from_str(OPENCODE_HARNESS_UUID)?;

    // F20: prefer worktree → directory → projectHash → unbound
    let bind_raw = payload
        .worktree
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            payload
                .directory
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            payload
                .project_hash
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("");

    let alias_key = normalize_opencode_project_hash(bind_raw);
    let original_resolved = ctx.resolve_project_id_from_alias(&alias_key)?;
    let mut project_id = original_resolved;
    let newly_bound = original_resolved.is_none();
    let mut used_env_fallback = false;

    // Env project id only when unbound/empty and alias not already resolved (F21 anti-hijack).
    if project_id.is_none()
        && opencode_env_fallback_allowed(bind_raw)
        && let Ok(env_pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && let Ok(env_pid) = ProjectId::from_str(&env_pid_str)
    {
        project_id = Some(env_pid);
        used_env_fallback = true;
    }

    let project_id = match project_id {
        Some(pid) => pid,
        None if alias_key == OPENCODE_UNBOUND_ALIAS => {
            ensure_or_create_project(ctx, OPENCODE_UNBOUND_ALIAS, OPENCODE_UNBOUND_DISPLAY_NAME)?
        }
        None => {
            let display = path_derived_display_name(&alias_key);
            ensure_or_create_project(ctx, &alias_key, &display)?
        }
    };

    // Prefer messagesPath / exportPath fixture (hermetic + live plugin temp file).
    let turns = load_turns(&payload)?;
    if turns.is_empty() {
        eprintln!(
            "[ai-brains-opencode] no message-only turns for session {}",
            payload.session_id
        );
        return Ok(());
    }

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let max_turn = query_store
        .get_max_turn_index(&session_id)
        .map_err(|e| format!("Failed to query vault turn state: {e}"))?;
    let next_index = max_turn.map(|m| m + 1).unwrap_or(0);

    if turns.len() <= next_index as usize {
        eprintln!("No new turns to ingest (vault already has {next_index}).");
        return Ok(());
    }

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(
            std::sync::Arc::clone(&ctx.conn),
        )),
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        project_id,
        session_id,
        oc_harness,
        Privacy::LocalOnly,
    )?;

    // Env fallback for unbound must not stamp opencode-unbound onto env project.
    if !used_env_fallback {
        ctx.ensure_project_alias(&mut sink, project_id, alias_key.clone(), Privacy::LocalOnly)?;
        if newly_bound {
            eprintln!("Auto-linked OpenCode path {alias_key} to project {project_id}");
        }
    } else {
        eprintln!(
            "[ai-brains-opencode] unbound session routed to env project {project_id} (alias not stamped)"
        );
    }

    if let Some(ev) = payload.event.as_deref() {
        eprintln!("[ai-brains-opencode] event={ev}");
    }

    let turn_count = append_opencode_turns(
        &service,
        &mut sink,
        session_id,
        project_id,
        &turns,
        next_index as usize,
        &capture_context,
    )?;

    eprintln!("Successfully ingested {turn_count} turns from OpenCode export/messages.");
    Ok(())
}

fn load_turns(
    payload: &OpenCodeHookPayload,
) -> Result<Vec<ai_brains_adapters::OpenCodeIngestTurn>, Box<dyn std::error::Error>> {
    // Prefer messagesPath (SDK messages dump) then exportPath
    for key in [
        payload.messages_path.as_deref(),
        payload.export_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let t = key.trim();
        if t.is_empty() {
            continue;
        }
        let p = PathBuf::from(t);
        if p.is_file() {
            return Ok(parse_export_file(&p)?);
        }
        // Normalize attempt
        if let Ok(norm) = ai_brains_path::normalize_project_path(t) {
            let p = PathBuf::from(norm.canonical());
            if p.is_file() {
                return Ok(parse_export_file(&p)?);
            }
        }
    }
    // F12: when plugin could not supply a path, try CLI export (120s, fail-open).
    let sid = payload.session_id.trim();
    if !sid.is_empty() {
        match export_session_via_cli(sid) {
            Ok(turns) if !turns.is_empty() => {
                eprintln!(
                    "[ai-brains-opencode] F12 CLI export fallback for session {sid} ({} turns)",
                    turns.len()
                );
                return Ok(turns);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[ai-brains-opencode] F12 CLI export soft-fail: {e}");
            }
        }
    }
    // Empty document → zero turns (fail-open)
    Ok(parse_export_json(&serde_json::json!({})))
}

fn ensure_or_create_project(
    ctx: &AppContext,
    alias: &str,
    display_name: &str,
) -> Result<ProjectId, Box<dyn std::error::Error>> {
    if let Some(existing) = ctx.resolve_project_id_from_alias(alias)? {
        return Ok(existing);
    }

    let project_id = ProjectId::new();
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let actor = Actor::User(ai_brains_core::ids::UserId::new());
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: display_name.to_string(),
        tx_id: None,
    }))?;
    event_store.append_event(&reg)?;

    if let Some(existing) = ctx.resolve_project_id_from_alias(alias)? {
        return Ok(existing);
    }

    let alias_ev = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
        project_id,
        alias: alias.to_string(),
    }))?;
    event_store.append_event(&alias_ev)?;
    Ok(project_id)
}
