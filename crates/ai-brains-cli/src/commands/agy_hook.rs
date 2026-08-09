use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::agy::{
    AGY_UNBOUND_ALIAS, AGY_UNBOUND_DISPLAY_NAME, agy_env_fallback_allowed,
    generate_turn_id_for_ingest, normalize_agy_project_hash, parse_transcript_for_ingest,
    path_derived_display_name,
};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
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
struct AgyHookPayload {
    pub transcript_path: String,
    pub session_id: String,
    pub project_hash: String,
}

pub fn run(ctx: &AppContext, payload_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload: AgyHookPayload = serde_json::from_str(payload_json)?;
    let transcript_path = PathBuf::from(&payload.transcript_path);

    // Normalize transcript path via ai-brains-path
    let normalized_path =
        ai_brains_path::normalize_project_path(&transcript_path.to_string_lossy())?;

    let session_id = SessionId::from_uuid(
        uuid::Uuid::parse_str(&payload.session_id)
            .map_err(|e| format!("Invalid session ID in agy hook: {}", e))?,
    );

    // Canonical agy Harness ID
    let agy_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000002")?;

    // F3: normalize projectHash BEFORE resolve / ensure alias
    let raw_hash = payload.project_hash.trim();
    let alias_key = normalize_agy_project_hash(raw_hash);

    let original_resolved = ctx.resolve_project_id_from_alias(&alias_key)?;
    let mut project_id = original_resolved;
    let newly_bound = original_resolved.is_none();

    // F3(4): AI_BRAINS_PROJECT_ID only when hash is agy-unbound / empty
    if project_id.is_none()
        && agy_env_fallback_allowed(raw_hash)
        && let Ok(env_pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && let Ok(env_pid) = ProjectId::from_str(&env_pid_str)
    {
        project_id = Some(env_pid);
    }

    // F3(5): path-derived or stable unbound when still unresolved
    let project_id = match project_id {
        Some(pid) => pid,
        None if alias_key == AGY_UNBOUND_ALIAS => {
            ensure_or_create_project(ctx, AGY_UNBOUND_ALIAS, AGY_UNBOUND_DISPLAY_NAME)?
        }
        None => {
            let display = path_derived_display_name(&alias_key);
            ensure_or_create_project(ctx, &alias_key, &display)?
        }
    };

    // Shared parse (step-shaped + legacy + transcript_full prefer) — F1/F2/F29
    let ingestable_turns =
        parse_transcript_for_ingest(std::path::Path::new(normalized_path.canonical()))?;

    // Delta Sync (T49)
    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    let max_turn = query_store
        .get_max_turn_index(&session_id)
        .map_err(|e| format!("Failed to query vault turn state: {}", e))?;
    let next_index = max_turn.map(|m| m + 1).unwrap_or(0);

    if ingestable_turns.len() <= next_index as usize {
        // Diagnostics on stderr (F8 honesty for direct CLI + wrapper)
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

    // Ensure project/session exists
    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        project_id,
        session_id,
        agy_harness,
        Privacy::LocalOnly,
    )?;

    // Auto-link normalized alias to project
    ctx.ensure_project_alias(&mut sink, project_id, alias_key.clone(), Privacy::LocalOnly)?;
    if newly_bound {
        eprintln!(
            "Auto-linked projectHash {} to project {}",
            alias_key, project_id
        );
    }

    let mut turn_count = 0;
    for (i, turn) in ingestable_turns
        .iter()
        .enumerate()
        .skip(next_index as usize)
    {
        let turn_id = generate_turn_id_for_ingest(&session_id, i, turn.step_index);

        let request = IngestRequest {
            session_id,
            project_id,
            harness_id: agy_harness,
            turn_id,
            role: turn.role.clone(),
            content: turn.content.clone(),
            privacy: Privacy::LocalOnly,
            thinking: None,
            tx_id: None,
        };

        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        turn_count += 1;
    }

    eprintln!(
        "Successfully ingested {} turns from agy transcript.",
        turn_count
    );
    Ok(())
}

/// Resolve existing alias or register a new project with that alias.
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

    // Re-check alias in case of concurrent create
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
