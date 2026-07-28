//! `ai-brains source show` — inspect a registered source (T160).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, expect_daemon_ok, fail_api,
    fail_cp, fail_path, principal_id_wire, resolve_principal,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::sources::{InspectSourceRequest, SourceDto};
use ai_brains_control_plane::{
    GovernedQueryStore, PolicyContext, PolicyEvaluator, StorePorts, parse_scope_key,
    scope_identity_key, source_row_to_dto,
};
use ai_brains_core::ids::SourceId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::str::FromStr;

pub struct ShowOptions {
    pub id: String,
    pub scope: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains source show <id>`
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
                ApiError::new("INVALID_PAYLOAD", "source show requires --scope"),
            );
        }
    };
    let scope = match parse_scope_key(scope_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
    match policy.allow(
        principal.id,
        GrantCapability::ReadEvidence,
        &scope,
        &policy_ctx,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return fail_api(
                format,
                ApiError::new("POLICY_DENIED", "ReadEvidence denied for inspect_source"),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    let source_id = match SourceId::from_str(&options.id) {
        Ok(id) => id,
        Err(_) => {
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("invalid source id: {}", options.id),
                ),
            );
        }
    };

    let expected_scope = scope_identity_key(&scope);
    // CP query port owns projection load; CLI only maps + anti-enumeration.
    let dto = match ports.query.get_source(source_id) {
        Ok(Some(row)) if row.scope == expected_scope => source_row_to_dto(&row),
        Ok(Some(_)) | Ok(None) => {
            return fail_api(
                format,
                ApiError::new("NOT_FOUND", format!("source {}", options.id)),
            );
        }
        Err(e) => return fail_cp(format, e),
    };

    emit_source(format, &dto)
}

async fn run_show_daemon(
    options: &ShowOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::InspectSource(InspectSourceRequest {
        api_version: ai_brains_contracts::sources::API_VERSION.to_string(),
        id: options.id.clone(),
        principal_id: principal_id_wire(&principal),
        scope: options.scope.clone(),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::Source(dto) => emit_source(format, &dto),
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

fn emit_source(
    format: OutputFormat,
    dto: &SourceDto,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(dto),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_human(&format!(
                "source: {} ({})\nname: {}\nlocator: {}",
                dto.id,
                dto.kind,
                dto.display_name,
                dto.locator.as_deref().unwrap_or("-")
            ));
            Ok(())
        }
    }
}
