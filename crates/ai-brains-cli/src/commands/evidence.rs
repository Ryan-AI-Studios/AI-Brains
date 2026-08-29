//! `ai-brains evidence show|list|search` — bounded evidence preview + discovery (T160 / T203).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, apply_authorized_empty_list_next, emit_human,
    emit_json, expect_daemon_ok, fail_api, fail_cp, fail_path, fail_usage,
    format_authorized_empty_next, policy_denied_hint_details, principal_id_wire, resolve_principal,
    resolve_scope_key_for_cli,
};
use crate::commands::governed_namespace::{
    apply_unknown_handle_overlay, namespace_memory_present, wrong_namespace_next_line,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::briefings::{
    EvidenceListItemDto, EvidenceListResponse, InspectEvidenceRequest, ListEvidenceRequest,
    truncate_evidence_list_summary,
};
use ai_brains_contracts::offset_to_utc;
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::{
    ExpandHandleRequest, GovernedQueryStore, PolicyContext, PolicyEvaluator, StorePorts,
    clamp_list_limit, expand_handle, parse_scope_key, scope_identity_key,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::{QueryStore, SqliteEventStore};

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

pub struct ListOptions {
    pub scope: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub format: Option<String>,
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

/// `ai-brains evidence list` / `evidence search`
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
        Ok(preview) => {
            // expand_handle returns denied preview (empty/truncated) rather than PolicyDenied
            // for some paths; still attach hint when kind signals deny if free — keep parity.
            let mut value = serde_json::to_value(&preview)?;
            // F1: probe only after Unknown (skip found/Denied SQL).
            let present = preview.kind == "Unknown"
                && namespace_memory_present(ctx.conn.memory_exists(&preview.handle_id));
            apply_unknown_handle_overlay(&mut value, present);
            emit_evidence_preview(format, &value)
        }
        Err(e) => {
            use ai_brains_control_plane::ControlPlaneError;
            match &e {
                ControlPlaneError::PolicyDenied(_) => fail_api(
                    format,
                    ApiError::new("POLICY_DENIED", e.to_string())
                        .with_details(policy_denied_hint_details()),
                ),
                _ => fail_cp(format, e),
            }
        }
    }
}

async fn run_show_daemon(
    ctx: &AppContext,
    options: &ShowOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::InspectEvidence(InspectEvidenceRequest {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        id: options.id.clone(),
        scope: Some(scope_key.to_string()),
        principal_id: principal_id_wire(&principal),
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
        DaemonResponse::EvidencePreview(preview) => {
            // F2/F30: CLI probes local vault even on daemon path; no EXISTS IPC.
            let mut value = serde_json::to_value(&preview)?;
            let present = preview.kind == "Unknown"
                && namespace_memory_present(ctx.conn.memory_exists(&preview.handle_id));
            apply_unknown_handle_overlay(&mut value, present);
            emit_evidence_preview(format, &value)
        }
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

/// Emit evidence show after T319 overlay (JSON Value may include optional `next_step`).
fn emit_evidence_preview(
    format: OutputFormat,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(value),
        OutputFormat::Human | OutputFormat::Markdown => {
            let handle_id = value
                .get("handle_id")
                .and_then(|h| h.as_str())
                .unwrap_or("");
            let kind = value.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            let preview = value.get("preview").and_then(|p| p.as_str()).unwrap_or("");
            let truncated = value
                .get("truncated")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            emit_human(&format!(
                "handle: {handle_id} ({kind})\npreview: {preview}\ntruncated: {truncated}"
            ));
            if value.get("next_step").and_then(|n| n.as_str()).is_some() {
                emit_human(&wrong_namespace_next_line());
            }
            Ok(())
        }
    }
}

fn run_list_local(
    ctx: &AppContext,
    options: &ListOptions,
    scope_key: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let scope: ScopeRef = match parse_scope_key(scope_key) {
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
                ApiError::new("POLICY_DENIED", "ReadEvidence denied for list_evidence")
                    .with_details(policy_denied_hint_details()),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    let page = clamp_list_limit(options.limit);
    let expected_scope = scope_identity_key(&scope);
    let mut rows = match ports.query.list_evidence_for_scope(
        &expected_scope,
        options.query.as_deref(),
        page + 1,
    ) {
        Ok(rows) => rows,
        Err(e) => return fail_cp(format, e),
    };
    let more_available = rows.len() > page;
    if more_available {
        rows.truncate(page);
    }
    let items: Vec<EvidenceListItemDto> = rows
        .into_iter()
        .map(|r| EvidenceListItemDto {
            id: r.id.to_string(),
            summary: truncate_evidence_list_summary(&r.summary),
            status: r.status,
            source_id: r.source_id.to_string(),
            recorded_at: Some(offset_to_utc(r.recorded_at)),
        })
        .collect();
    let resp = EvidenceListResponse::new(items).with_more(more_available);
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
    let req = DaemonRequest::ListEvidence(ListEvidenceRequest {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        scope: Some(scope_key.to_string()),
        query: options.query.clone(),
        limit: options.limit,
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_e) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::EvidenceList(list) => emit_list(format, &list, None),
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

fn emit_list(
    format: OutputFormat,
    resp: &EvidenceListResponse,
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
                emit_human("evidence: (none)");
                emit_human(&format_authorized_empty_next(pin_count, None));
            } else {
                for item in &resp.items {
                    emit_human(&format!("- {} [{}] {}", item.id, item.status, item.summary));
                }
                if resp.more_available {
                    emit_human("(more available)");
                }
            }
            Ok(())
        }
    }
}
