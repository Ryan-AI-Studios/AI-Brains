//! `ai-brains grok-hook` — live Stop/SessionEnd capture from Grok Build (T237).

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::grok::{
    GROK_HARNESS_UUID, GROK_UNBOUND_ALIAS, GROK_UNBOUND_DISPLAY_NAME, append_grok_turns,
    grok_env_fallback_allowed, normalize_grok_project_hash, parse_chat_history_file,
    resolve_chat_history_path, resolve_grok_home,
};
use ai_brains_adapters::path_derived_display_name;
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use ai_brains_store::EventStore;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrokHookPayload {
    /// May be empty — resolved from sessionId + workspaceRoot/cwd when missing.
    #[serde(default)]
    pub history_path: String,
    pub session_id: String,
    pub project_hash: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

pub fn run(ctx: &AppContext, payload_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload: GrokHookPayload = serde_json::from_str(payload_json)?;

    let session_id = SessionId::from_uuid(
        uuid::Uuid::parse_str(&payload.session_id)
            .map_err(|e| format!("Invalid session ID in grok hook: {}", e))?,
    );

    let grok_harness = HarnessId::from_str(GROK_HARNESS_UUID)?;

    // F3: normalize projectHash BEFORE resolve / ensure alias
    let raw_hash = payload.project_hash.trim();
    let alias_key = normalize_grok_project_hash(raw_hash);

    let original_resolved = ctx.resolve_project_id_from_alias(&alias_key)?;
    let mut project_id = original_resolved;
    let newly_bound = original_resolved.is_none();

    // Env project id only when unbound/empty
    if project_id.is_none()
        && grok_env_fallback_allowed(raw_hash)
        && let Ok(env_pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && let Ok(env_pid) = ProjectId::from_str(&env_pid_str)
    {
        project_id = Some(env_pid);
    }

    let project_id = match project_id {
        Some(pid) => pid,
        None if alias_key == GROK_UNBOUND_ALIAS => {
            ensure_or_create_project(ctx, GROK_UNBOUND_ALIAS, GROK_UNBOUND_DISPLAY_NAME)?
        }
        None => {
            let display = path_derived_display_name(&alias_key);
            ensure_or_create_project(ctx, &alias_key, &display)?
        }
    };

    // Resolve history path: explicit existing path, else F7 multi-fallback.
    let history_path = resolve_history_path(&payload)?;
    let Some(history_path) = history_path else {
        eprintln!(
            "[ai-brains-grok] skip: could not resolve chat_history for session {}",
            payload.session_id
        );
        return Ok(());
    };

    let ingestable_turns = parse_chat_history_file(&history_path)?;

    // Delta Sync
    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let max_turn = query_store
        .get_max_turn_index(&session_id)
        .map_err(|e| format!("Failed to query vault turn state: {}", e))?;
    let next_index = max_turn.map(|m| m + 1).unwrap_or(0);

    if ingestable_turns.len() <= next_index as usize {
        eprintln!("No new turns to ingest (vault already has {}).", next_index);
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
        grok_harness,
        Privacy::LocalOnly,
    )?;

    ctx.ensure_project_alias(&mut sink, project_id, alias_key.clone(), Privacy::LocalOnly)?;
    if newly_bound {
        eprintln!(
            "Auto-linked projectHash {} to project {}",
            alias_key, project_id
        );
    }

    if let Some(ev) = payload.event.as_deref() {
        eprintln!("[ai-brains-grok] event={ev}");
    }

    // Shared SOOT with batch import: turn-{i} ids + thinking always None (AC3/AC4).
    let turn_count = append_grok_turns(
        &service,
        &mut sink,
        session_id,
        project_id,
        &ingestable_turns,
        next_index as usize,
        &capture_context,
    )?;

    eprintln!(
        "Successfully ingested {} turns from Grok chat_history.",
        turn_count
    );
    Ok(())
}

fn resolve_history_path(
    payload: &GrokHookPayload,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let explicit = payload.history_path.trim();
    if !explicit.is_empty() {
        let p = PathBuf::from(explicit);
        if p.is_file() {
            return Ok(Some(p));
        }
        // Try normalize
        if let Ok(norm) = ai_brains_path::normalize_project_path(explicit) {
            let p = PathBuf::from(norm.canonical());
            if p.is_file() {
                return Ok(Some(p));
            }
        }
    }

    let workspace = payload
        .workspace_root
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            let h = payload.project_hash.trim();
            if h.is_empty() || h.eq_ignore_ascii_case(GROK_UNBOUND_ALIAS) {
                None
            } else {
                Some(payload.project_hash.as_str())
            }
        });
    let cwd = payload.cwd.as_deref().filter(|s| !s.trim().is_empty());

    let grok_home = resolve_grok_home(None).ok_or("cannot resolve GROK_HOME / ~/.grok")?;
    Ok(resolve_chat_history_path(
        Path::new(&grok_home),
        &payload.session_id,
        workspace,
        cwd,
    ))
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
