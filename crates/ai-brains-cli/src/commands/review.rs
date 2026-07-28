//! `ai-brains review list|resolve` — review queue surface (T160).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, ensure_command_id,
    expect_daemon_ok, fail_api, fail_cp, fail_path, principal_id_wire, resolve_principal,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::review::{
    ListReviewItemsRequest, ResolveReviewItemRequest, ReviewItemDto, ReviewQueueResponse,
    ReviewResolvedResponse,
};
use ai_brains_control_plane::{
    GovernedQueryStore, PolicyContext, PolicyEvaluator, StorePorts,
    list_open_review_items_for_scope, parse_scope_key, resolve_review_item, scope_identity_key,
};
use ai_brains_core::ids::ReviewItemId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::str::FromStr;

pub struct ListOptions {
    pub scope: Option<String>,
    pub status: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

pub struct ResolveOptions {
    pub id: String,
    pub resolution: String,
    pub scope: String,
    pub note: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub command_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains review list`
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

    match path {
        PathDecision::Daemon => run_list_daemon(&options, format).await,
        PathDecision::Local { .. } => run_list_local(ctx, &options, format),
    }
}

fn run_list_local(
    ctx: &AppContext,
    options: &ListOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let scope_key = match options.scope.as_deref() {
        Some(s) => s,
        None => {
            return fail_api(
                format,
                ApiError::new("INVALID_PAYLOAD", "review list requires --scope"),
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
        GrantCapability::ReadConclusions,
        &scope,
        &policy_ctx,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return fail_api(
                format,
                ApiError::new(
                    "POLICY_DENIED",
                    "ReadConclusions denied for list_review_items",
                ),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    let scope_key = scope_identity_key(&scope);
    // Scope isolation: same filter as daemon (related conclusion/decision/source + subject).
    let mut items = match list_open_review_items_for_scope(&ports.query, &scope_key) {
        Ok(items) => items,
        Err(e) => return fail_cp(format, e),
    };
    if let Some(status_filter) = options.status.as_deref()
        && !status_filter.eq_ignore_ascii_case("Open")
    {
        items.clear();
    }

    let dtos: Vec<ReviewItemDto> = items
        .into_iter()
        .map(|r| ReviewItemDto {
            id: r.id.to_string(),
            subject: r.subject,
            status: r.status,
            opened_at: None,
        })
        .collect();
    let resp = ReviewQueueResponse::new(dtos);
    emit_list(format, &resp)
}

async fn run_list_daemon(
    options: &ListOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::ListReviewItems(ListReviewItemsRequest {
        api_version: ai_brains_contracts::review::API_VERSION.to_string(),
        principal_id: principal_id_wire(options.principal_id.as_deref(), &principal),
        scope: options.scope.clone(),
        status: options.status.clone(),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ReviewList(queue) => emit_list(format, &queue),
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

fn emit_list(
    format: OutputFormat,
    resp: &ReviewQueueResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            if resp.items.is_empty() {
                emit_human("review items: (none)");
            } else {
                for item in &resp.items {
                    emit_human(&format!("- {} [{}] {}", item.id, item.status, item.subject));
                }
            }
            Ok(())
        }
    }
}

/// `ai-brains review resolve <id> --resolution … --scope …`
///
/// Prefer a Human principal (`--principal-id` or `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID`);
/// the System default often receives APPROVAL_REQUIRED from control-plane.
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
    let path = match governed_common::choose_mutation_path(flags).await {
        Ok(p) => p,
        Err(e) => return fail_path(format, e),
    };

    let command_id = ensure_command_id(options.command_id.as_deref());

    match path {
        PathDecision::Daemon => run_resolve_daemon(&options, &command_id, format).await,
        PathDecision::Local { note } => {
            if let Some(n) = note {
                eprintln!("note: {n}");
            }
            run_resolve_local(ctx, &options, &command_id, format)
        }
    }
}

fn run_resolve_local(
    ctx: &AppContext,
    options: &ResolveOptions,
    command_id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let scope = match parse_scope_key(&options.scope) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let review_item_id = match ReviewItemId::from_str(&options.id) {
        Ok(id) => id,
        Err(_) => {
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("invalid review item id: {}", options.id),
                ),
            );
        }
    };
    let principal = resolve_principal(options.principal_id.as_deref());
    let reason = match options
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
    {
        Some(note) => format!("{} ({})", options.resolution, note),
        None => options.resolution.clone(),
    };

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    match resolve_review_item(
        &ports.writer,
        &ports.query,
        &policy,
        &principal,
        review_item_id,
        &reason,
        Privacy::LocalOnly,
        scope,
    ) {
        Ok(()) => {
            let status = ports
                .query
                .get_review_item(review_item_id)
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or_else(|| "Resolved".to_string());
            let mut resp = ReviewResolvedResponse::new(review_item_id.to_string(), status);
            resp.warnings.push(format!("command_id={command_id}"));
            match format {
                OutputFormat::Json => emit_json(&resp),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "review {} resolved (status={}; command_id={command_id})",
                        resp.id, resp.status
                    ));
                    Ok(())
                }
            }
        }
        Err(e) => fail_cp(format, e),
    }
}

async fn run_resolve_daemon(
    options: &ResolveOptions,
    command_id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::ResolveReviewItem(ResolveReviewItemRequest {
        api_version: ai_brains_contracts::review::API_VERSION.to_string(),
        id: options.id.clone(),
        resolution: options.resolution.clone(),
        principal_id: principal_id_wire(options.principal_id.as_deref(), &principal),
        note: options.note.clone(),
        scope: Some(options.scope.clone()),
        command_id: Some(command_id.to_string()),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(e) => {
            let classified = governed_common::classify_daemon_mutation_error(&e);
            // Pre-send down without require-daemon was already handled by choose_mutation_path
            // re-probe; if we get here after choosing Daemon, treat as ambiguous or unavailable.
            return fail_path(format, classified);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ReviewResolved(mut wire) => {
            if !wire.warnings.iter().any(|w| w.contains("command_id=")) {
                wire.warnings.push(format!("command_id={command_id}"));
            }
            match format {
                OutputFormat::Json => emit_json(&wire),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "review {} resolved (status={}; command_id={command_id})",
                        wire.id, wire.status
                    ));
                    Ok(())
                }
            }
        }
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}
