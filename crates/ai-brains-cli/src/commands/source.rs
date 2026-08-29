//! `ai-brains source show|list` — inspect and discover registered sources (T160 / T203).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, apply_authorized_empty_list_next, emit_human,
    emit_json, expect_daemon_ok, fail_api, fail_cp, fail_path, fail_usage,
    format_authorized_empty_next, policy_denied_hint_details, principal_id_wire, resolve_principal,
    resolve_scope_key_for_cli,
};
use crate::commands::governed_namespace::{namespace_memory_present, wrong_namespace_source_hint};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::sources::{
    InspectSourceRequest, ListSourcesRequest, SourceDto, SourceListResponse,
};
use ai_brains_control_plane::{
    GovernedQueryStore, PolicyContext, PolicyEvaluator, StorePorts, clamp_list_limit,
    parse_scope_key, scope_identity_key, source_row_to_dto,
};
use ai_brains_core::ids::SourceId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::{QueryStore, SqliteEventStore};
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

pub struct ListOptions {
    pub scope: Option<String>,
    pub limit: Option<usize>,
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

    let scope_key = {
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let ports = StorePorts::from_store(store);
        let identity = ports.identity_store();
        match resolve_scope_key_for_cli(options.scope.as_deref(), &identity) {
            Ok(k) => k,
            Err(msg) => return fail_usage(msg),
        }
    };

    match path {
        PathDecision::Daemon => run_show_daemon(ctx, &options, &scope_key, format).await,
        PathDecision::Local { .. } => run_show_local(ctx, &options, &scope_key, format),
    }
}

/// `ai-brains source list`
pub async fn run_list(
    ctx: &AppContext,
    options: ListOptions,
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

    let scope_key = {
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let ports = StorePorts::from_store(store);
        let identity = ports.identity_store();
        match resolve_scope_key_for_cli(options.scope.as_deref(), &identity) {
            Ok(k) => k,
            Err(msg) => return fail_usage(msg),
        }
    };

    match path {
        PathDecision::Daemon => run_list_daemon(&options, &scope_key, format).await,
        PathDecision::Local { .. } => run_list_local(ctx, &options, &scope_key, format),
    }
}

fn run_show_local(
    ctx: &AppContext,
    options: &ShowOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
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
                ApiError::new("POLICY_DENIED", "ReadEvidence denied for inspect_source")
                    .with_details(policy_denied_hint_details()),
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
            return fail_source_not_found(ctx, format, &options.id);
        }
        Err(e) => return fail_cp(format, e),
    };

    emit_source(format, &dto)
}

async fn run_show_daemon(
    ctx: &AppContext,
    options: &ShowOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::InspectSource(InspectSourceRequest {
        api_version: ai_brains_contracts::sources::API_VERSION.to_string(),
        id: options.id.clone(),
        principal_id: principal_id_wire(&principal),
        scope: Some(scope_key.to_string()),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    // F30: attach hint only when local vault has this UUID as a memory_id.
    // Plain daemon NOT_FOUND (and other errors) stay on expect_daemon_ok (N±1).
    match resp {
        DaemonResponse::Error(err)
            if err.code == "NOT_FOUND"
                && namespace_memory_present(ctx.conn.memory_exists(&options.id)) =>
        {
            fail_api(format, with_wrong_namespace_hint(err))
        }
        other => {
            let resp = expect_daemon_ok(format, other)?;
            match resp {
                DaemonResponse::Source(dto) => emit_source(format, &dto),
                unexpected => Err(format!("unexpected daemon response: {unexpected:?}").into()),
            }
        }
    }
}

/// F4 — source miss stays NOT_FOUND exit 4; additive hint when UUID is a vault memory_id.
fn fail_source_not_found(
    ctx: &AppContext,
    format: OutputFormat,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let err = ApiError::new("NOT_FOUND", format!("source {id}"));
    if namespace_memory_present(ctx.conn.memory_exists(id)) {
        return fail_api(format, with_wrong_namespace_hint(err));
    }
    fail_api(format, err)
}

fn with_wrong_namespace_hint(err: ApiError) -> ApiError {
    let hint = serde_json::Value::String(wrong_namespace_source_hint());
    // Preserve any prior details keys; overlay hint.
    let details = match err.details {
        Some(serde_json::Value::Object(mut existing)) => {
            existing.insert("hint".to_string(), hint);
            serde_json::Value::Object(existing)
        }
        _ => {
            let mut map = serde_json::Map::new();
            map.insert("hint".to_string(), hint);
            serde_json::Value::Object(map)
        }
    };
    ApiError::new(err.code, err.message).with_details(details)
}

fn run_list_local(
    ctx: &AppContext,
    options: &ListOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
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
                ApiError::new("POLICY_DENIED", "ReadEvidence denied for list_sources")
                    .with_details(policy_denied_hint_details()),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    let page = clamp_list_limit(options.limit);
    let expected_scope = scope_identity_key(&scope);
    let mut rows = match ports
        .query
        .list_sources_for_scope(&expected_scope, page + 1)
    {
        Ok(rows) => rows,
        Err(e) => return fail_cp(format, e),
    };
    let more_available = rows.len() > page;
    if more_available {
        rows.truncate(page);
    }
    let items = rows.iter().map(source_row_to_dto).collect();
    let resp = SourceListResponse::new(items).with_more(more_available);
    let pin_count = match &scope {
        ScopeRef::Repository(pid) => ctx.conn.count_pinned_memories(Some(pid)).ok(),
        _ => None,
    };
    emit_list(format, &resp, pin_count)
}

async fn run_list_daemon(
    options: &ListOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::ListSources(ListSourcesRequest {
        api_version: ai_brains_contracts::sources::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        scope: Some(scope_key.to_string()),
        limit: options.limit,
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::SourceList(list) => emit_list(format, &list, None),
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

fn emit_source(format: OutputFormat, dto: &SourceDto) -> Result<(), Box<dyn std::error::Error>> {
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

fn emit_list(
    format: OutputFormat,
    resp: &SourceListResponse,
    pin_count: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            let mut value = serde_json::to_value(resp)?;
            apply_authorized_empty_list_next(&mut value, pin_count);
            emit_json(&value)
        }
        OutputFormat::Human | OutputFormat::Markdown => {
            if resp.items.is_empty() {
                emit_human("sources: (none)");
                emit_human(&format_authorized_empty_next(pin_count, None));
            } else {
                for item in &resp.items {
                    emit_human(&format!(
                        "- {} [{}] {}",
                        item.id, item.kind, item.display_name
                    ));
                }
                if resp.more_available {
                    emit_human("(more available)");
                }
            }
            Ok(())
        }
    }
}
