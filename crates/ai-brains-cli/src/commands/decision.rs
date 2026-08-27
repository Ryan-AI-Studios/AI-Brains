//! `ai-brains decision propose` / `decision in-force` (T160 / T311).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, ensure_command_id,
    expect_daemon_ok, fail_api, fail_cp, fail_path, fail_usage, format_authorized_empty_next,
    policy_denied_hint_details, principal_id_wire, resolve_principal, resolve_scope_key_for_cli,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::knowledge::{
    DecisionProposedResponse, ProposeDecisionRequest as WireProposeDecision,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::{
    NS_PROPOSE_DECISION, PolicyContext, PolicyEvaluator,
    ProposeDecisionRequest as CpProposeDecision, StorePorts, SystemClock, id_from_command,
    parse_scope_key, propose_decision, resolve_in_force,
};
use ai_brains_core::ids::{ConclusionId, DecisionId, EvidenceId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::str::FromStr;

pub struct ProposeOptions {
    pub statement: String,
    pub title: Option<String>,
    pub conclusions: Vec<String>,
    pub evidence: Vec<String>,
    pub scope: String,
    pub format: Option<String>,
    pub principal_id: Option<String>,
    pub command_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// `ai-brains decision propose --statement [--title] [--conclusion] [--evidence] --scope`
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
            ApiError::new("INVALID_PAYLOAD", "statement must be non-empty"),
        );
    }
    let scope = match parse_scope_key(&options.scope) {
        Ok(s) => s,
        Err(e) => return fail_cp(format, e),
    };
    let conclusion_ids = match parse_conclusion_ids(&options.conclusions) {
        Ok(ids) if ids.is_empty() => None,
        Ok(ids) => Some(ids),
        Err(bad) => {
            return fail_api(
                format,
                ApiError::new("INVALID_PAYLOAD", format!("invalid conclusion id: {bad}")),
            );
        }
    };
    let evidence_ids = match parse_evidence_ids(&options.evidence) {
        Ok(ids) if ids.is_empty() => None,
        Ok(ids) => Some(ids),
        Err(bad) => {
            return fail_api(
                format,
                ApiError::new("INVALID_PAYLOAD", format!("invalid evidence id: {bad}")),
            );
        }
    };
    let principal = resolve_principal(options.principal_id.as_deref());
    let decision_id = DecisionId::from_uuid(id_from_command(NS_PROPOSE_DECISION, command_id));
    let title = options
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "Decision".to_string());

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    match propose_decision(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        CpProposeDecision {
            principal,
            scope,
            title,
            statement: options.statement.clone(),
            conclusion_ids,
            evidence_ids,
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: Some(decision_id),
        },
    ) {
        Ok(res) => {
            let mut resp = DecisionProposedResponse::new(res.decision_id.to_string(), "proposed");
            resp.warnings.push(format!("command_id={command_id}"));
            match format {
                OutputFormat::Json => emit_json(&resp),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "decision {} (proposed); command_id={command_id}",
                        resp.decision_id
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
    let req = DaemonRequest::ProposeDecision(WireProposeDecision {
        api_version: ai_brains_contracts::knowledge::API_VERSION.to_string(),
        principal_id: principal_id_wire(&principal),
        scope: options.scope.clone(),
        title: options.title.clone(),
        statement: options.statement.clone(),
        conclusion_ids: options.conclusions.clone(),
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
        DaemonResponse::DecisionProposed(mut wire) => {
            if !wire.warnings.iter().any(|w| w.contains("command_id=")) {
                wire.warnings.push(format!("command_id={command_id}"));
            }
            match format {
                OutputFormat::Json => emit_json(&wire),
                OutputFormat::Human | OutputFormat::Markdown => {
                    emit_human(&format!(
                        "decision {} ({}); command_id={command_id}",
                        wire.decision_id, wire.status
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

fn parse_conclusion_ids(ids: &[String]) -> Result<Vec<ConclusionId>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for s in ids {
        match ConclusionId::from_str(s) {
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

/// `ai-brains decision in-force <TERM>` — local projection read (T311).
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
        GrantCapability::ReadDecisions,
        &scope,
        &policy_ctx,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return fail_api(
                format,
                ApiError::new(
                    "POLICY_DENIED",
                    "ReadDecisions denied for decision in-force",
                )
                .with_details(policy_denied_hint_details()),
            );
        }
        Err(e) => return fail_cp(format, e),
    }

    match resolve_in_force(&ports.query, &SystemClock, &scope_key, &options.term) {
        Ok(resp) => emit_in_force(format, &resp),
        Err(e) => fail_cp(format, e),
    }
}

fn emit_in_force(
    format: OutputFormat,
    resp: &ai_brains_control_plane::InForceResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(resp),
        OutputFormat::Human | OutputFormat::Markdown => {
            if let Some(ruling) = &resp.ruling {
                emit_human(&format!(
                    "Term: {}\nScope: {}\nRuling: {} ({})",
                    resp.term, resp.scope, ruling.title, ruling.decision_id
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
