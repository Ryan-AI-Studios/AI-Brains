//! `ai-brains policy show|check` — thin grant inspection (T160).
//!
//! Read-only via grant store. No grant mutation in T160.

use crate::commands::governed_common::{
    OutputFormat, emit_human, emit_json, fail_api, fail_cp, resolve_principal,
};
use crate::context::AppContext;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::scopes::{ScopeGrantDto, ScopeGrantsResponse};
use ai_brains_control_plane::{PolicyContext, PolicyEvaluator, StorePorts, parse_scope_key};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_store::SqliteEventStore;

pub struct ShowOptions {
    pub scope: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
}

pub struct CheckOptions {
    pub capability: String,
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
}

/// `ai-brains policy show [--scope] [--principal-id]`
pub fn run_show(ctx: &AppContext, options: ShowOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let grant_store = ports.grant_store();

    let scope_key = match options.scope.as_deref() {
        Some(s) => {
            // Validate parse when provided
            if let Err(e) = parse_scope_key(s) {
                return fail_cp(format, e);
            }
            s.to_string()
        }
        None => {
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    "policy show requires --scope (scope identity key)",
                ),
            );
        }
    };

    let applied = grant_store.list_applied_grants(principal.id, &scope_key, None)?;
    let grants: Vec<ScopeGrantDto> = applied
        .into_iter()
        .map(|g| ScopeGrantDto {
            grant_id: g.grant_id,
            principal_id: principal.id.to_string(),
            scope: g.scope_key,
            capability: g.capability,
            privacy: g.privacy,
        })
        .collect();
    let resp = ScopeGrantsResponse::new(grants);
    match format {
        OutputFormat::Json => emit_json(&resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            if resp.grants.is_empty() {
                emit_human(&format!(
                    "grants for principal {} on {scope_key}: (none)",
                    principal.id
                ));
            } else {
                emit_human(&format!(
                    "grants for principal {} on {scope_key}:",
                    principal.id
                ));
                for g in &resp.grants {
                    emit_human(&format!(
                        "  - {} ({}, privacy={})",
                        g.capability, g.grant_id, g.privacy
                    ));
                }
            }
            Ok(())
        }
    }
}

/// `ai-brains policy check --capability ProposeConclusion --scope …`
pub fn run_check(
    ctx: &AppContext,
    options: CheckOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let scope = match parse_scope_key(&options.scope) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let capability = match parse_capability_label(&options.capability) {
        Some(c) => c,
        None => {
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("unknown capability: {}", options.capability),
                ),
            );
        }
    };

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
    let allowed = match policy.allow(principal.id, capability, &scope, &policy_ctx) {
        Ok(v) => v,
        Err(e) => return fail_cp(format, e),
    };

    #[derive(serde::Serialize)]
    struct CheckResult {
        allowed: bool,
        principal_id: String,
        capability: String,
        scope: String,
    }

    let result = CheckResult {
        allowed,
        principal_id: principal.id.to_string(),
        capability: options.capability.clone(),
        scope: options.scope.clone(),
    };

    if !allowed {
        // Structured denial with exit 3 for scripts.
        match format {
            OutputFormat::Json => {
                let _ = emit_json(&result);
                return fail_api(
                    format,
                    ApiError::new(
                        "POLICY_DENIED",
                        format!(
                            "{} denied for principal {} on {}",
                            options.capability, principal.id, options.scope
                        ),
                    ),
                );
            }
            OutputFormat::Human | OutputFormat::Markdown => {
                return fail_api(
                    format,
                    ApiError::new(
                        "POLICY_DENIED",
                        format!(
                            "{} denied for principal {} on {}",
                            options.capability, principal.id, options.scope
                        ),
                    ),
                );
            }
        }
    }

    match format {
        OutputFormat::Json => emit_json(&result),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_human(&format!(
                "allowed: true ({} on {})",
                options.capability, options.scope
            ));
            Ok(())
        }
    }
}

fn parse_capability_label(raw: &str) -> Option<GrantCapability> {
    match raw.trim() {
        "ReadEvidence" => Some(GrantCapability::ReadEvidence),
        "ReadConclusions" => Some(GrantCapability::ReadConclusions),
        "ReadDecisions" => Some(GrantCapability::ReadDecisions),
        "ProposeConclusion" => Some(GrantCapability::ProposeConclusion),
        "ApproveConclusion" => Some(GrantCapability::ApproveConclusion),
        "ProposeDecision" => Some(GrantCapability::ProposeDecision),
        "ApproveDecision" => Some(GrantCapability::ApproveDecision),
        "Export" => Some(GrantCapability::Export),
        "Erase" => Some(GrantCapability::Erase),
        _ => None,
    }
}
