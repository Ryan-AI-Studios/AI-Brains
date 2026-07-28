//! `ai-brains scope resolve` — governed scope resolution (T160 / #20).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_json, emit_scope_human, expect_daemon_ok,
    fail_cp, fail_path,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_control_plane::{ScopeResolveInput, StorePorts, resolve_scope};
use ai_brains_core::ids::{ProjectId, UserId};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::path::PathBuf;
use std::str::FromStr;

pub struct ResolveOptions {
    pub format: Option<String>,
    pub cwd: Option<String>,
    pub project_id: Option<ProjectId>,
    pub force_personal: bool,
    pub personal_user_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains scope resolve`
pub async fn run_resolve(
    ctx: &AppContext,
    options: ResolveOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let flags = PathFlags {
        local: options.local,
        daemon: options.daemon,
        require_daemon: options.require_daemon,
    };
    let path = match governed_common::choose_read_path(flags).await {
        Ok(p) => p,
        Err(e) => return fail_path(format, e),
    };

    match path {
        PathDecision::Daemon => run_resolve_daemon(&options, format).await,
        PathDecision::Local { .. } => run_resolve_local(ctx, &options, format),
    }
}

fn run_resolve_local(
    ctx: &AppContext,
    options: &ResolveOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let identity = ports.identity_store();
    let cwd = options
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let personal_user_id = options
        .personal_user_id
        .as_deref()
        .and_then(|s| UserId::from_str(s).ok());
    let input = ScopeResolveInput {
        cwd,
        explicit_project_id: options.project_id,
        force_personal: options.force_personal,
        personal_user_id,
        git_metadata: None,
    };
    let resolved = match resolve_scope(&input, &identity) {
        Ok(r) => r,
        Err(e) => return fail_cp(format, e),
    };
    let wire = governed_common::map_resolved_scope(&resolved);
    match format {
        OutputFormat::Json => emit_json(&wire),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_scope_human(&wire);
            Ok(())
        }
    }
}

async fn run_resolve_daemon(
    options: &ResolveOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let req = DaemonRequest::ResolveScope(ResolveScopeRequest {
        api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
        cwd: options.cwd.clone().or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        }),
        signals: None,
        explicit_project_id: options.project_id.map(|p| p.to_string()),
        force_personal: options.force_personal,
        personal_user_id: options.personal_user_id.clone(),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_e) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ScopeResolved(wire) => match format {
            OutputFormat::Json => emit_json(&wire),
            OutputFormat::Human | OutputFormat::Markdown => {
                emit_scope_human(&wire);
                Ok(())
            }
        },
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}
