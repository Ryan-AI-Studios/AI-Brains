//! `ai-brains evidence show` — bounded evidence / handle preview (T160).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, expect_daemon_ok, fail_api,
    fail_cp, fail_path, principal_id_wire, resolve_principal,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::briefings::InspectEvidenceRequest;
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::{ExpandHandleRequest, StorePorts, expand_handle, parse_scope_key};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;

pub struct ShowOptions {
    pub id: String,
    pub scope: Option<String>,
    pub format: Option<String>,
    pub max_chars: usize,
    pub principal_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains evidence show <id>`
pub async fn run_show(
    ctx: &AppContext,
    options: ShowOptions,
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
        PathDecision::Daemon => run_show_daemon(&options, format).await,
        PathDecision::Local { .. } => run_show_local(ctx, &options, format),
    }
}

fn run_show_local(
    ctx: &AppContext,
    options: &ShowOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let scope_key = match options.scope.as_deref() {
        Some(s) => s,
        None => {
            return fail_api(
                format,
                ApiError::new("INVALID_PAYLOAD", "evidence show requires --scope"),
            );
        }
    };
    let scope: ScopeRef = match parse_scope_key(scope_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let event_store = ports.store();
    match expand_handle(
        &ports.query,
        &event_store,
        &policy,
        ExpandHandleRequest {
            principal,
            scope,
            handle_id: options.id.clone(),
            privacy: Privacy::LocalOnly,
            max_chars: options.max_chars,
        },
    ) {
        Ok(preview) => match format {
            OutputFormat::Json => emit_json(&preview),
            OutputFormat::Human | OutputFormat::Markdown => {
                emit_human(&format!(
                    "handle: {} ({})\npreview: {}\ntruncated: {}",
                    preview.handle_id, preview.kind, preview.preview, preview.truncated
                ));
                Ok(())
            }
        },
        Err(e) => fail_cp(format, e),
    }
}

async fn run_show_daemon(
    options: &ShowOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::InspectEvidence(InspectEvidenceRequest {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        id: options.id.clone(),
        scope: options.scope.clone(),
        principal_id: principal_id_wire(options.principal_id.as_deref(), &principal),
        max_chars: Some(options.max_chars),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_e) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::EvidencePreview(preview) => match format {
            OutputFormat::Json => emit_json(&preview),
            OutputFormat::Human | OutputFormat::Markdown => {
                emit_human(&format!(
                    "handle: {} ({})\npreview: {}\ntruncated: {}",
                    preview.handle_id, preview.kind, preview.preview, preview.truncated
                ));
                Ok(())
            }
        },
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}
