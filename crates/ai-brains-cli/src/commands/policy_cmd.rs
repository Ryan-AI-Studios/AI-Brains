//! `ai-brains policy show|check|bootstrap` — grant inspection + discovery bootstrap (T160/T210).
//!
//! Show/check are read-only. Bootstrap is a thin CP mutation path (register +
//! issue discovery grants only). No domain logic in the CLI.

use crate::commands::governed_common::{
    OutputFormat, emit_human, emit_json, fail_api, fail_cp, fail_usage, policy_denied_hint_details,
    resolve_principal, resolve_scope_key_for_cli,
};
use crate::context::AppContext;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::scopes::{ScopeGrantDto, ScopeGrantsResponse};
use ai_brains_control_plane::{
    GrantPrincipalStore, PolicyContext, PolicyEvaluator, StorePorts, SystemClock, issue_grant,
    parse_scope_key, register_principal, scope_identity_key,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_store::SqliteEventStore;
use serde::Serialize;

pub struct ShowOptions {
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
}

pub struct CheckOptions {
    pub capability: String,
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
}

pub struct BootstrapOptions {
    pub scope: Option<String>,
    pub dry_run: bool,
    pub principal_id: Option<String>,
    pub format: Option<String>,
}

/// Discovery-class capabilities issued by `policy bootstrap` (F2 — hard set).
const DISCOVERY_CAPS: [GrantCapability; 3] = [
    GrantCapability::ReadEvidence,
    GrantCapability::ReadConclusions,
    GrantCapability::ReadDecisions,
];

/// CLI-local bootstrap response (F10 / F19 — not a contracts DTO).
#[derive(Debug, Serialize)]
struct PolicyBootstrapResponse {
    api_version: String,
    principal_id: String,
    scope: String,
    /// `registered` | `already` | `would_register`
    registered: String,
    grants: Vec<BootstrapGrantEntry>,
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct BootstrapGrantEntry {
    capability: String,
    /// `issued` | `already_present` | `would_issue`
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    grant_id: Option<String>,
}

/// `ai-brains policy show --scope … [--principal-id]`
pub fn run_show(ctx: &AppContext, options: ShowOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let grant_store = ports.grant_store();

    // clap guarantees --scope; validate parse shape only.
    let scope_key = options.scope.as_str();
    if let Err(e) = parse_scope_key(scope_key) {
        return fail_cp(format, e);
    }

    let applied = grant_store.list_applied_grants(principal.id, scope_key, None)?;
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

    if !allowed {
        // Exactly one structured document on deny (JSON: ApiError only; exit 3).
        // F6: structured details.hint for operator remediation (T201).
        return fail_api(
            format,
            ApiError::new(
                "POLICY_DENIED",
                format!(
                    "{} denied for principal {} on {}",
                    options.capability, principal.id, options.scope
                ),
            )
            .with_details(policy_denied_hint_details()),
        );
    }

    #[derive(serde::Serialize)]
    struct CheckResult {
        allowed: bool,
        principal_id: String,
        capability: String,
        scope: String,
    }

    let result = CheckResult {
        allowed: true,
        principal_id: principal.id.to_string(),
        capability: options.capability.clone(),
        scope: options.scope.clone(),
    };

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

/// `ai-brains policy bootstrap [--scope …] [--dry-run] [--principal-id]`
///
/// Issues discovery-class grants (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`)
/// with `Privacy::LocalOnly` for the resolved principal on the resolved scope.
/// Idempotent via `active_grants` probe; registers principal only when missing.
pub fn run_bootstrap(
    ctx: &AppContext,
    options: BootstrapOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let grant_store = ports.grant_store();
    let clock = SystemClock;

    // F5 / F39 — explicit scope or soft-resolve; fail_usage (exit 2) when missing.
    let scope_key = match options
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) => s.to_string(),
        None => match resolve_scope_key_for_cli(None, &ports.identity_store()) {
            Ok(k) => k,
            Err(msg) => return fail_usage(msg),
        },
    };

    let scope_ref = match parse_scope_key(&scope_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    // Canonical key for response (matches parse round-trip).
    let scope_key = scope_identity_key(&scope_ref);

    // F33 — probe principal before register (no re-append noise).
    let existing = match grant_store.get_principal(principal.id) {
        Ok(p) => p,
        Err(e) => return fail_cp(format, e),
    };
    let registered = if options.dry_run {
        if existing.is_some() {
            "already".to_string()
        } else {
            "would_register".to_string()
        }
    } else if existing.is_some() {
        "already".to_string()
    } else {
        if let Err(e) = register_principal(&ports.writer, &clock, &principal) {
            return fail_cp(format, e);
        }
        "registered".to_string()
    };

    // F7 — active_grants (typed ScopeGrant), not list_applied_grants.
    let active = match grant_store.active_grants(principal.id, &scope_ref) {
        Ok(g) => g,
        Err(e) => return fail_cp(format, e),
    };

    let mut grants: Vec<BootstrapGrantEntry> = Vec::with_capacity(DISCOVERY_CAPS.len());
    for cap in DISCOVERY_CAPS {
        let cap_name = capability_label(cap).to_string();
        if active.iter().any(|g| g.capability == cap) {
            grants.push(BootstrapGrantEntry {
                capability: cap_name,
                status: "already_present".to_string(),
                grant_id: None,
            });
            continue;
        }
        if options.dry_run {
            grants.push(BootstrapGrantEntry {
                capability: cap_name,
                status: "would_issue".to_string(),
                grant_id: None,
            });
            continue;
        }
        let grant_id = match issue_grant(
            &ports.writer,
            &clock,
            principal.id,
            scope_ref.clone(),
            cap,
            Privacy::LocalOnly,
        ) {
            Ok(id) => id,
            Err(e) => return fail_cp(format, e),
        };
        grants.push(BootstrapGrantEntry {
            capability: cap_name,
            status: "issued".to_string(),
            grant_id: Some(grant_id.to_string()),
        });
    }

    // F30 — sort by capability name alphabetically.
    grants.sort_by(|a, b| a.capability.cmp(&b.capability));

    let resp = PolicyBootstrapResponse {
        api_version: "1".to_string(),
        principal_id: principal.id.to_string(),
        scope: scope_key,
        registered,
        grants,
        dry_run: options.dry_run,
    };

    match format {
        OutputFormat::Json => emit_json(&resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_human(&format!(
                "policy bootstrap principal {} on {}",
                resp.principal_id, resp.scope
            ));
            emit_human(&format!("registered: {}", resp.registered));
            for g in &resp.grants {
                match &g.grant_id {
                    Some(id) => emit_human(&format!("  {}: {} ({})", g.capability, g.status, id)),
                    None => emit_human(&format!("  {}: {}", g.capability, g.status)),
                }
            }
            if resp.dry_run {
                emit_human("dry-run: no events appended");
            } else {
                emit_human(
                    "next: try `ai-brains source list`, `ai-brains review list`, or `ai-brains briefing project`",
                );
            }
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

fn capability_label(cap: GrantCapability) -> &'static str {
    match cap {
        GrantCapability::ReadEvidence => "ReadEvidence",
        GrantCapability::ReadConclusions => "ReadConclusions",
        GrantCapability::ReadDecisions => "ReadDecisions",
        GrantCapability::ProposeConclusion => "ProposeConclusion",
        GrantCapability::ApproveConclusion => "ApproveConclusion",
        GrantCapability::ProposeDecision => "ProposeDecision",
        GrantCapability::ApproveDecision => "ApproveDecision",
        GrantCapability::Export => "Export",
        GrantCapability::Erase => "Erase",
    }
}
