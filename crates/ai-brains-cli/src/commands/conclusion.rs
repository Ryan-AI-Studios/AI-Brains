//! `ai-brains conclusion propose` / `in-force` (T160 / T323).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, ensure_command_id,
    expect_daemon_ok, fail_api, fail_cp, fail_path, fail_usage, format_authorized_empty_next,
    policy_denied_hint_details, principal_id_wire, resolve_principal, resolve_scope_key_for_cli,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::knowledge::{
    ConclusionProposedResponse, ProposeConclusionRequest as WireProposeConclusion,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::{
    NS_PROPOSE_CONCLUSION, PolicyContext, PolicyEvaluator,
    ProposeConclusionRequest as CpProposeConclusion, StorePorts, SystemClock, id_from_command,
    parse_scope_key, propose_conclusion, resolve_conclusion_in_force,
};
use ai_brains_core::ids::{ConclusionId, EvidenceId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::str::FromStr;

pub struct ProposeOptions {
    /// Claim / statement text (`--claim` or `--statement`).
    pub statement: String,
    pub evidence: Vec<String>,
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub command_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains conclusion propose --claim/--statement + --evidence + --scope`
pub async fn run_propose(
    ctx: &AppContext,
    options: ProposeOptions,
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
        PathDecision::Daemon => run_propose_daemon(&options, &command_id, format).await,
        PathDecision::Local { note } => {
            if let Some(n) = note {
                eprintln!("note: {n}");
            }
            run_propose_local(ctx, &options, &command_id, format)
        }
    }
}

fn run_propose_local(
    ctx: &AppContext,
    options: &ProposeOptions,
    command_id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    if options.statement.trim().is_empty() {
        return fail_api(
            format,
            ApiError::new("INVALID_PAYLOAD", "statement/claim must be non-empty"),
        );
    }
    let scope = match parse_scope_key(&options.scope) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let evidence_ids = match parse_evidence_ids(&options.evidence) {
        Ok(ids) => ids,
        Err(bad) => {
            return fail_api(
                format,
                ApiError::new("INVALID_PAYLOAD", format!("invalid evidence id: {bad}")),
            );
        }
    };
    let principal = resolve_principal(options.principal_id.as_deref());
    let conclusion_id = ConclusionId::from_uuid(id_from_command(NS_PROPOSE_CONCLUSION, command_id));

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    match propose_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        CpProposeConclusion {
            principal,
            scope,
            statement: options.statement.clone(),
            evidence_ids,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: Some(conclusion_id),
        },
    ) {
        Ok(res) => {
            let status = if res.unsupported {
                "unsupported"
            } else {
                "proposed"
            };
            let mut resp = ConclusionProposedResponse::new(res.conclusion_id.to_string(), status);
            resp.warnings.push(format!("command_id={command_id}"));
            match format {
                OutputFormat::Json => emit_json(&resp),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "conclusion {} ({status}); command_id={command_id}",
                        resp.conclusion_id
                    ));
                    Ok(())
                }
            }
        }
        Err(e) => fail_cp(format, e),
    }
}

async fn run_propose_daemon(
    options: &ProposeOptions,
    command_id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let principal = resolve_principal(options.principal_id.as_deref());
    let client = DaemonClient::new();
    let req = DaemonRequest::ProposeConclusion(WireProposeConclusion {
        api_version: ai_brains_contracts::knowledge::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        scope: options.scope.clone(),
        statement: options.statement.clone(),
        evidence_ids: options.evidence.clone(),
        privacy: Some("LocalOnly".into()),
        command_id: Some(command_id.to_string()),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(e) => {
            return fail_path(format, governed_common::classify_daemon_mutation_error(&e));
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    match resp {
        DaemonResponse::ConclusionProposed(mut wire) => {
            if !wire.warnings.iter().any(|w| w.contains("command_id=")) {
                wire.warnings.push(format!("command_id={command_id}"));
            }
            match format {
                OutputFormat::Json => emit_json(&wire),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "conclusion {} ({}); command_id={command_id}",
                        wire.conclusion_id, wire.status
                    ));
                    Ok(())
                }
            }
        }
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

fn parse_evidence_ids(ids: &[String]) -> Result<Vec<EvidenceId>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for s in ids {
        match EvidenceId::from_str(s) {
            Ok(id) => out.push(id),
            Err(_) => return Err(s.clone()),
        }
    }
    Ok(out)
}

pub struct InForceOptions {
    pub term: String,
    pub scope: Option<String>,
    pub format: String,
    pub principal_id: Option<String>,
}

/// `ai-brains conclusion in-force <TERM>` — local projection read (T323).
pub fn run_in_force(
    ctx: &AppContext,
    options: InForceOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(Some(options.format.as_str()));
    if options.term.trim().is_empty() {
        return fail_usage("term must be non-empty");
    }

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let identity = ports.identity_store();
    let scope_key = match resolve_scope_key_for_cli(options.scope.as_deref(), &identity) {
        Ok(k) => k,
        Err(msg) => return fail_usage(msg),
    };
    let scope = match parse_scope_key(&scope_key) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };

    let principal = resolve_principal(options.principal_id.as_deref());
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
                    "ReadConclusions denied for conclusion in-force",
                )
                .with_details(policy_denied_hint_details()),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    match resolve_conclusion_in_force(&ports.query, &SystemClock, &scope_key, &options.term) {
        Ok(resp) => emit_in_force(format, &resp),
        Err(e) => fail_cp(format, e),
    }
}

fn emit_in_force(
    format: OutputFormat,
    resp: &ai_brains_control_plane::ConclusionInForceResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            if let Some(ruling) = &resp.ruling {
                emit_human(&format!(
                    "Term: {}\nScope: {}\nRuling: {} ({})",
                    resp.term, resp.scope, ruling.statement, ruling.conclusion_id
                ));
            } else {
                emit_human(&format!(
                    "No in-force ruling for term \"{}\".\n{}",
                    resp.term,
                    format_authorized_empty_next(None, None)
                ));
            }
            Ok(())
        }
    }
}
