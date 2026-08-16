//! `ai-brains claude-hook` — live UPS/Stop/SessionEnd capture from Claude Code (T253).
//!
//! Ingests payload `prompt` / `lastAssistantMessage` directly after T234 filter.
//! Does **not** parse `transcript_path`. Unrecognized / Grok-shaped stdin: exit 0.

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::claude::{
    CLAUDE_HARNESS_UUID, CLAUDE_UNBOUND_ALIAS, CLAUDE_UNBOUND_DISPLAY_NAME,
    accept_claude_live_payload, claude_env_fallback_allowed, generate_claude_live_turn_id,
    normalize_claude_project_hash, session_id_from_claude,
};
use ai_brains_adapters::path_derived_display_name;
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use ai_brains_store::EventStore;
use std::str::FromStr;

pub fn run(ctx: &AppContext, payload_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mapped = match accept_claude_live_payload(payload_json) {
        Ok(Some(m)) => m,
        Ok(None) => {
            eprintln!("[ai-brains-claude] skip: unrecognized hook stdin");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let session_id = session_id_from_claude(&mapped.session_id);
    let claude_harness = HarnessId::from_str(CLAUDE_HARNESS_UUID)?;

    let raw_hash = mapped.project_hash.trim();
    let alias_key = normalize_claude_project_hash(raw_hash);
    let original_resolved = ctx.resolve_project_id_from_alias(&alias_key)?;
    let mut project_id = original_resolved;
    let newly_bound = original_resolved.is_none();
    let mut used_env_fallback = false;

    if project_id.is_none()
        && claude_env_fallback_allowed(raw_hash)
        && let Ok(env_pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && let Ok(env_pid) = ProjectId::from_str(&env_pid_str)
    {
        project_id = Some(env_pid);
        used_env_fallback = true;
    }

    let project_id = match project_id {
        Some(pid) => pid,
        None if alias_key == CLAUDE_UNBOUND_ALIAS => {
            ensure_or_create_project(ctx, CLAUDE_UNBOUND_ALIAS, CLAUDE_UNBOUND_DISPLAY_NAME)?
        }
        None => {
            let display = path_derived_display_name(&alias_key);
            ensure_or_create_project(ctx, &alias_key, &display)?
        }
    };

    let mut planned: Vec<(&str, String)> = Vec::new();
    if let Some(p) = mapped.prompt.clone() {
        planned.push(("user", p));
    }
    if let Some(a) = mapped.last_assistant.clone() {
        planned.push(("assistant", a));
    }
    if planned.is_empty() {
        eprintln!(
            "[ai-brains-claude] skip: empty prompt/lastAssistantMessage after message-only filter"
        );
        return Ok(());
    }

    let query_store = ctx.conn.clone() as std::sync::Arc<dyn ai_brains_store::QueryStore>;
    // Query-error fail-open: do not treat errors as an empty vault (would re-ingest).
    // QueryStore has no per-turn-id lookup; get_session_turns is role+content only.
    if skip_on_query_error(
        query_store.get_max_turn_index(&session_id),
        "ai-brains-claude",
    )
    .is_none()
    {
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
        claude_harness,
        Privacy::LocalOnly,
    )?;

    if !used_env_fallback {
        ctx.ensure_project_alias(&mut sink, project_id, alias_key.clone(), Privacy::LocalOnly)?;
        if newly_bound {
            eprintln!("Auto-linked projectHash {alias_key} to project {project_id}");
        }
    } else {
        eprintln!(
            "[ai-brains-claude] unbound session routed to env project {project_id} (alias not stamped)"
        );
    }

    eprintln!("[ai-brains-claude] event={}", mapped.event);

    let stable = mapped
        .uuid
        .as_deref()
        .or(mapped.turn_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let both = planned.len() > 1;
    let mut turn_count = 0;
    for (role, content) in planned {
        let role_suffix = if both { Some(role) } else { None };
        let turn_id = generate_claude_live_turn_id(&session_id, &mapped.event, stable, role_suffix);
        // F15: skip re-fire of the same event+uuid/turn_id. Without a vendor id
        // `{event}:stable` is not unique per UPS — do not content-match.
        if stable.is_some() {
            let key = live_turn_sync_key(&session_id, &turn_id);
            match skip_on_query_error(query_store.get_sync_state(&key), "ai-brains-claude") {
                None => return Ok(()),
                Some(Some(_)) => {
                    eprintln!("[ai-brains-claude] skip existing {role} turn");
                    continue;
                }
                Some(None) => {}
            }
        }
        let request = IngestRequest {
            session_id,
            project_id,
            harness_id: claude_harness,
            turn_id,
            role: role.to_string(),
            content,
            privacy: Privacy::LocalOnly,
            thinking: None,
            tx_id: None,
        };
        service.ingest_request(request, capture_context.clone(), &mut sink)?;
        if sink.last_error.is_some() {
            break;
        }
        if stable.is_some() {
            sink.set_sync_state(&live_turn_sync_key(&session_id, &turn_id), "1");
        }
        turn_count += 1;
    }

    if let Some(err) = sink.last_error {
        return Err(format!("Claude hook ingest error: {err}").into());
    }
    eprintln!("Successfully ingested {turn_count} turn(s) from Claude hook payload.");
    Ok(())
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

fn live_turn_sync_key(session_id: &SessionId, turn_id: &TurnId) -> String {
    format!("live-turn:{session_id}:{turn_id}")
}

/// Query-error fail-open: skip ingest (exit 0). Do not use `unwrap_or_else` empty.
fn skip_on_query_error<T, E: std::fmt::Display>(result: Result<T, E>, label: &str) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("[{label}] skip: vault query failed ({e}); not ingesting");
            None
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::skip_on_query_error;

    #[test]
    fn claude_hook__query_error__skip_not_empty_vault() {
        assert!(
            skip_on_query_error(Result::<(), &str>::Err("locked"), "ai-brains-claude").is_none()
        );
        assert_eq!(
            skip_on_query_error(Result::<i32, &str>::Ok(1), "ai-brains-claude"),
            Some(1)
        );
    }
}
