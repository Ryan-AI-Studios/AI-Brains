//! `ai-brains retention plan|apply` — class-based retention (T166 / P8.4).
//!
//! # Dual path (spec §6 freeze)
//!
//! - **`retention plan`** — local dry-run (no disposal).
//! - **`retention apply --confirm`**:
//!   - **Projection** deletes: in-process via [`apply_retention_projections`].
//!   - **CE** rows: require daemon (T165 E8 parity); each key wiped via
//!     `DaemonRequest::WipeContentEnvelope` only. Never `AllowAllPolicy` + local wipe.
//!   - If any CE candidate exists and daemon is down → `DAEMON_UNAVAILABLE` before disposal.
//!   - Projection-only apply may run without daemon.
//!
//! Fixture tests may call `apply_retention` in-process with `AllowAllPolicy`.

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_human, emit_json, ensure_command_id,
    fail_api, fail_path, principal_id_wire, resolve_principal,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::erasure::WipeContentEnvelopeRequest;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::retention::RetentionPlanReport;
use ai_brains_control_plane::{
    RetentionConfig, StorePorts, apply_retention_projections, finalize_retention_apply,
    parse_scope_key, plan_retention, scope_identity_key,
};
use ai_brains_core::ids::UserId;
use ai_brains_core::scope::ScopeRef;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;

pub struct PlanOptions {
    pub format: Option<String>,
}

pub struct ApplyOptions {
    pub format: Option<String>,
    pub confirm: bool,
    pub dry_run: bool,
    pub command_id: Option<String>,
    pub scope: Option<String>,
    pub principal_id: Option<String>,
}

/// `ai-brains retention plan [--format json|human]`
pub fn run_plan(ctx: &AppContext, options: PlanOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let config = RetentionConfig::from_env();
    let report = plan_retention(&store, &config)?;
    emit_report(format, &report)
}

/// `ai-brains retention apply --confirm`
///
/// Sync entry (like other local vault commands). Daemon I/O uses the current
/// Tokio handle so we do not grow a large async state machine on this path.
pub fn run_apply(
    ctx: &AppContext,
    options: ApplyOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());

    if options.dry_run && options.confirm {
        return fail_api(
            format,
            ApiError::new(
                "INVALID_PAYLOAD",
                "cannot combine --dry-run and --confirm; omit --confirm to plan, or pass --confirm alone to apply",
            ),
        );
    }
    if !options.confirm || options.dry_run {
        return fail_api(
            format,
            ApiError::new(
                "INVALID_PAYLOAD",
                "retention apply requires --confirm (dry-run is default; use `retention plan` for a report without disposal)",
            ),
        );
    }

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(SqliteEventStore::new((*ctx.conn).clone()));
    let config = RetentionConfig::from_env();
    let command_id = ensure_command_id(options.command_id.as_deref());
    let principal = resolve_principal(options.principal_id.as_deref());

    let scope = match options.scope.as_deref() {
        Some(s) if !s.trim().is_empty() => match parse_scope_key(s) {
            Ok(sc) => sc,
            Err(e) => {
                return fail_api(
                    format,
                    ApiError::new("INVALID_PAYLOAD", format!("invalid --scope: {e}")),
                );
            }
        },
        _ => ScopeRef::Personal(UserId::new()),
    };
    let scope_wire = scope_identity_key(&scope);

    let plan = match plan_retention(&store, &config) {
        Ok(p) => p,
        Err(e) => {
            return fail_api(format, ApiError::new("QUERY_FAILED", e.to_string()));
        }
    };

    let handle = tokio::runtime::Handle::current();

    if production_apply_requires_daemon(plan.totals.would_ce_wipe) {
        let flags = PathFlags {
            local: false,
            daemon: false,
            require_daemon: true,
        };
        let path = match handle.block_on(governed_common::choose_erasure_path(flags)) {
            Ok(p) => p,
            Err(e) => return fail_path(format, e),
        };
        let PathDecision::Daemon = path else {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        };
    }

    let outcome =
        match apply_retention_projections(&store, &ports.writer, &config, &command_id, true, false)
        {
            Ok(o) => o,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("confirm") || msg.contains("dry_run") {
                    return fail_api(format, ApiError::new("INVALID_PAYLOAD", msg));
                }
                return fail_api(format, ApiError::new("QUERY_FAILED", msg));
            }
        };

    let mut report = outcome.report;
    if !report.warnings.iter().any(|w| w.contains("command_id=")) {
        report.warnings.push(format!("command_id={command_id}"));
    }

    let mut cascade_memory_ids = outcome.pending_cascade_memory_ids;
    if !outcome.pending_ce_keys.is_empty() {
        let client = DaemonClient::new();
        for key in &outcome.pending_ce_keys {
            let req = DaemonRequest::WipeContentEnvelope(WipeContentEnvelopeRequest {
                api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
                principal_id: principal_id_wire(&principal),
                content_key_id: key.clone(),
                scope: scope_wire.clone(),
                reason: Some(format!("retention_apply:{command_id}")),
                command_id: Some(format!("{command_id}:{key}")),
                dry_run: false,
                confirm: true,
            });
            match handle.block_on(client.request(req)) {
                Ok(DaemonResponse::ContentEnvelopeWiped(wire)) => {
                    if wire.status != "wiped" && wire.status != "already_erased" {
                        report
                            .errors
                            .push(format!("ce_wipe {key}: unexpected status {}", wire.status));
                    }
                }
                Ok(DaemonResponse::Error(err)) => {
                    report
                        .errors
                        .push(format!("ce_wipe {key}: {}: {}", err.code, err.message));
                }
                Ok(other) => {
                    report.errors.push(format!(
                        "ce_wipe {key}: unexpected daemon response: {other:?}"
                    ));
                }
                Err(e) => {
                    let classified = governed_common::classify_daemon_mutation_error(&e);
                    report.errors.push(format!("ce_wipe {key}: {classified}"));
                }
            }
        }

        cascade_memory_ids.sort();
        cascade_memory_ids.dedup();
        if let Err(e) = finalize_retention_apply(
            &store,
            &ports.writer,
            &command_id,
            &cascade_memory_ids,
            &mut report,
        ) {
            report.errors.push(format!("finalize_retention_apply: {e}"));
        }
    }

    report.errors_count = report.errors.len() as u64;
    let exit_err = report.errors_count > 0;
    emit_report(format, &report)?;
    if exit_err {
        return Err("retention apply had errors (see report)".into());
    }
    Ok(())
}

/// Whether production apply must require the daemon before mutating.
pub fn production_apply_requires_daemon(would_ce_wipe: u64) -> bool {
    would_ce_wipe > 0
}

fn emit_report(
    format: OutputFormat,
    report: &RetentionPlanReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(report),
        OutputFormat::Human | OutputFormat::Markdown => {
            let title = match report.mode.as_str() {
                "apply" => "Retention apply",
                _ => "Retention plan",
            };
            let mut lines = Vec::new();
            lines.push(format!(
                "{title} (mode={}, generated_at={})",
                report.mode, report.generated_at
            ));
            lines.push(format!(
                "  candidates={} ce_wipe={} projection_delete={} skip={} held={}",
                report.totals.candidates,
                report.totals.would_ce_wipe,
                report.totals.would_projection_delete,
                report.totals.would_skip,
                report.totals.would_held
            ));
            for c in &report.classes {
                lines.push(format!(
                    "  - {} count={} mechanism={} samples={:?}",
                    c.class, c.candidate_count, c.mechanism, c.sample_ids
                ));
            }
            if !report.warnings.is_empty() {
                lines.push("Warnings:".into());
                for w in &report.warnings {
                    lines.push(format!("  ! {w}"));
                }
            }
            if !report.errors.is_empty() {
                lines.push("Errors:".into());
                for e in &report.errors {
                    lines.push(format!("  x {e}"));
                }
            }
            if report.cascade.parents_marked_for_resynthesis > 0 {
                lines.push(format!(
                    "  cascade parents_marked={}",
                    report.cascade.parents_marked_for_resynthesis
                ));
            }
            emit_human(&lines.join("\n"));
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn production_apply_requires_daemon__ce_candidates__true() {
        assert!(production_apply_requires_daemon(1));
    }

    #[test]
    fn production_apply_requires_daemon__projection_only__false() {
        assert!(!production_apply_requires_daemon(0));
    }
}
