//! `ai-brains policy show|check|bootstrap` — grant inspection + discovery bootstrap (T160/T210).
//!
//! Show/check are read-only. Bootstrap is a thin CP mutation path (register +
//! issue discovery grants only). No domain logic in the CLI.

use crate::commands::governed_common::{
    DISCOVERY_CAP_LABELS, EXIT_POLICY_DENIED, GovernedCliError, OutputFormat,
    POLICY_BOOTSTRAP_SOOT_SHORT, capability_required_usage_message, emit_human, emit_json,
    fail_api, fail_cp, fail_usage, policy_denied_hint_details, resolve_principal,
    resolve_scope_key_for_cli,
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
use std::io::IsTerminal;

pub struct ShowOptions {
    /// Optional — soft-resolves when authoritative (T226).
    pub scope: Option<String>,
    pub format: Option<String>,
    pub principal_id: Option<String>,
}

pub struct CheckOptions {
    /// Optional — missing → fail_usage catalog (T241 F6); not clap-required.
    pub capability: Option<String>,
    /// Optional — soft-resolves when authoritative (T226).
    pub scope: Option<String>,
    /// Family A token from clap (`auto` default). Not `Option` — clap always supplies.
    pub format: String,
    pub principal_id: Option<String>,
}

/// T292 F2 — human allow line (exact shape; not a wire contract).
pub(crate) fn format_policy_check_allow_line(capability: &str, scope: &str) -> String {
    format!("allowed: true ({capability} on {scope})")
}

/// T292 F7 — human deny line 1 (exact shape; line 2 is `POLICY_BOOTSTRAP_SOOT_SHORT`).
pub(crate) fn format_policy_check_deny_line(capability: &str) -> String {
    format!("denied: {capability}")
}

fn check_output_format(resolved: &str) -> OutputFormat {
    match resolved {
        "human" => OutputFormat::Human,
        _ => OutputFormat::Json,
    }
}

pub struct BootstrapOptions {
    pub scope: Option<String>,
    pub dry_run: bool,
    pub principal_id: Option<String>,
    pub format: Option<String>,
}

/// Discovery-class capabilities issued by `policy bootstrap` (F2 — hard set).
/// Labels live in `governed_common::DISCOVERY_CAP_LABELS` (T241 F6b).
const DISCOVERY_CAPS: [GrantCapability; 3] = [
    GrantCapability::ReadEvidence,
    GrantCapability::ReadConclusions,
    GrantCapability::ReadDecisions,
];

// Compile-time lock: enum array length matches shared labels.
const _: () = assert!(DISCOVERY_CAPS.len() == DISCOVERY_CAP_LABELS.len());

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

/// `ai-brains policy show [--scope …] [--principal-id]`
pub fn run_show(ctx: &AppContext, options: ShowOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let grant_store = ports.grant_store();

    // T226: soft-resolve omitted --scope; always canonicalize (F23/M1).
    let raw_key = match resolve_scope_key_for_cli(options.scope.as_deref(), &ports.identity_store())
    {
        Ok(k) => k,
        Err(msg) => return fail_usage(msg),
    };
    let scope_ref = match parse_scope_key(&raw_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let scope_key = scope_identity_key(&scope_ref);

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
    let mut resp = ScopeGrantsResponse::new(grants);
    // T241 F5/F32: next_step only when grants empty.
    if resp.grants.is_empty() {
        resp.next_step = Some(POLICY_BOOTSTRAP_SOOT_SHORT.to_string());
    }
    match format {
        OutputFormat::Json => emit_json(&resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            if resp.grants.is_empty() {
                emit_human(&format!(
                    "grants for principal {} on {scope_key}: (none)",
                    principal.id
                ));
                emit_human(POLICY_BOOTSTRAP_SOOT_SHORT);
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

/// `ai-brains policy check --capability ProposeConclusion [--scope …]`
pub fn run_check(
    ctx: &AppContext,
    options: CheckOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // T292 F1/F3: clap value_parser accepted the token; resolve auto via TTY (not OutputFormat::parse).
    let resolved = crate::commands::format_resolve::resolve_human_json_format(
        &options.format,
        std::io::stdout().is_terminal(),
    );
    let format = check_output_format(resolved);
    let principal = resolve_principal(options.principal_id.as_deref());

    // T241 F6: only omitted `--capability` (None) → fail_usage catalog (exit 2).
    // Explicit empty/whitespace still goes through parse → INVALID_PAYLOAD (not usage).
    let Some(cap_raw) = options.capability.as_deref() else {
        return fail_usage(capability_required_usage_message());
    };
    let cap_label = cap_raw.trim();

    // T226: soft-resolve omitted --scope; always canonicalize (F23/M1).
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let raw_key = match resolve_scope_key_for_cli(options.scope.as_deref(), &ports.identity_store())
    {
        Ok(k) => k,
        Err(msg) => return fail_usage(msg),
    };
    let scope_ref = match parse_scope_key(&raw_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let scope_key = scope_identity_key(&scope_ref);

    let capability = match parse_capability_label(cap_label) {
        Some(c) => c,
        None => {
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    format!("unknown capability: {cap_label}"),
                ),
            );
        }
    };

    let policy = ports.production_policy();
    let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
    let allowed = match policy.allow(principal.id, capability, &scope_ref, &policy_ctx) {
        Ok(v) => v,
        Err(e) => return fail_cp(format, e),
    };

    if !allowed {
        match format {
            // T292 F7: human deny is two stdout lines + exit 3; skip fail_api (stderr empty).
            OutputFormat::Human | OutputFormat::Markdown => {
                emit_human(&format_policy_check_deny_line(cap_label));
                emit_human(POLICY_BOOTSTRAP_SOOT_SHORT);
                return Err(Box::new(GovernedCliError::emitted(
                    EXIT_POLICY_DENIED,
                    format!(
                        "POLICY_DENIED: {cap_label} denied for principal {} on {scope_key}",
                        principal.id
                    ),
                )));
            }
            // T292 F6 / T160 R1-01: exactly one ApiError JSON document on stdout.
            OutputFormat::Json => {
                return fail_api(
                    format,
                    ApiError::new(
                        "POLICY_DENIED",
                        format!(
                            "{cap_label} denied for principal {} on {scope_key}",
                            principal.id
                        ),
                    )
                    .with_details(policy_denied_hint_details()),
                );
            }
        }
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
        capability: cap_label.to_string(),
        scope: scope_key.clone(),
    };

    match format {
        OutputFormat::Json => emit_json(&result),
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_human(&format_policy_check_allow_line(cap_label, &scope_key));
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

    // F5 / F39 / T226 F21 — single helper: explicit wins, empty=omit, soft-resolve, fail_usage.
    let raw_key = match resolve_scope_key_for_cli(options.scope.as_deref(), &ports.identity_store())
    {
        Ok(k) => k,
        Err(msg) => return fail_usage(msg),
    };

    let scope_ref = match parse_scope_key(&raw_key) {
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

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::{format_policy_check_allow_line, format_policy_check_deny_line};
    use crate::commands::governed_common::POLICY_BOOTSTRAP_SOOT_SHORT;

    /// T292 AC1 — allow line exact shape.
    #[test]
    fn format_policy_check_allow_line__read_evidence__exact_string() {
        let line = format_policy_check_allow_line("ReadEvidence", "Repository:aaaa-bbbb");
        assert_eq!(line, "allowed: true (ReadEvidence on Repository:aaaa-bbbb)");
        assert!(!line.starts_with('{'));
        assert!(line.contains("allowed:"));
        assert!(line.contains("ReadEvidence"));
    }

    /// T292 AC1 — deny line exact + SHORT freeze.
    #[test]
    fn format_policy_check_deny_line__propose__exact_string() {
        let line = format_policy_check_deny_line("ProposeConclusion");
        assert_eq!(line, "denied: ProposeConclusion");
        assert_eq!(
            POLICY_BOOTSTRAP_SOOT_SHORT,
            "next: run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap`"
        );
    }
}
