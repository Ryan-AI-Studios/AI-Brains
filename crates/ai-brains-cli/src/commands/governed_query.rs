//! Thin CLI surface for progressive query / handle expand / query trace (T152-P1-06).

use crate::commands::briefing::cli_principal;
use crate::context::AppContext;
use ai_brains_control_plane::{
    ExpandHandleRequest, GetQueryTraceRequest, ProgressiveQueryRequest, StorePorts, SystemClock,
    expand_handle, get_query_trace, progressive_query, scope_identity_key,
};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::SqliteEventStore;

pub struct ProgressiveQueryOptions {
    pub query: String,
    pub project_id: Option<ProjectId>,
    pub limit: usize,
    pub dry_run: bool,
}

pub struct ExpandHandleOptions {
    pub handle_id: String,
    pub project_id: Option<ProjectId>,
    pub max_chars: usize,
}

pub struct TraceOptions {
    pub trace_id: String,
}

/// `ai-brains query progressive "<text>"` — governed progressive query (JSON stdout).
pub fn run_progressive(
    ctx: &AppContext,
    options: ProgressiveQueryOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = options
        .project_id
        .ok_or("project id required (--project-id or AI_BRAINS_PROJECT_ID)")?;
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let clock = SystemClock;
    let policy = ports.production_policy();
    let principal = cli_principal();
    let scope = ScopeRef::Repository(project_id);
    let event_store = ports.store();

    let writer = if options.dry_run {
        None
    } else {
        Some(&ports.writer)
    };
    let resp = progressive_query(
        writer,
        &ports.query,
        &event_store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal,
            scope,
            query: options.query,
            privacy: Privacy::LocalOnly,
            limit: options.limit,
            dry_run: options.dry_run,
            at: None,
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

/// `ai-brains query expand <handle-id>` — bounded handle preview (JSON stdout).
pub fn run_expand(
    ctx: &AppContext,
    options: ExpandHandleOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = options
        .project_id
        .ok_or("project id required (--project-id or AI_BRAINS_PROJECT_ID)")?;
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let principal = cli_principal();
    let scope = ScopeRef::Repository(project_id);
    let event_store = ports.store();

    let preview = expand_handle(
        &ports.query,
        &event_store,
        &policy,
        ExpandHandleRequest {
            principal,
            scope: scope.clone(),
            handle_id: options.handle_id,
            privacy: Privacy::LocalOnly,
            max_chars: options.max_chars,
        },
    )?;
    // Include applied scope key for operators debugging cross-scope denials.
    let mut value = serde_json::to_value(&preview)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "applied_scope".into(),
            serde_json::Value::String(scope_identity_key(&scope)),
        );
    }
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

/// `ai-brains query trace <trace-id>` — fetch a governed query trace (JSON stdout).
pub fn run_trace(
    ctx: &AppContext,
    options: TraceOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let principal = cli_principal();
    let event_store = ports.store();

    let trace = get_query_trace(
        &event_store,
        &policy,
        GetQueryTraceRequest {
            principal,
            privacy: Privacy::LocalOnly,
            trace_id: options.trace_id,
        },
    )?;
    match trace {
        Some(t) => println!("{}", serde_json::to_string_pretty(&t)?),
        None => {
            // Empty-state contract: missing or unauthorized → null JSON.
            println!("null");
        }
    }
    Ok(())
}
