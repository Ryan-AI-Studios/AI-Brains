//! `ai-brains retention plan|apply` — class-based retention (T166 / P8.4).
//!
//! Dry-run default (R1). Apply requires `--confirm`.
//! CE for envelope classes reuses T165 wipe in-process on the local vault (R2).
//! Projection deletes are never labeled CE (R3). Reports have no plaintext bodies (R4).

use crate::commands::governed_common::{OutputFormat, emit_human, emit_json};
use crate::context::AppContext;
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::retention::RetentionPlanReport;
use ai_brains_control_plane::{
    AllowAllPolicy, RetentionApplyCommand, RetentionConfig, StoreContentEnvelopeWipe, StorePorts,
    SystemClock, apply_retention, make_principal, plan_retention,
};
use ai_brains_core::ids::{PrincipalId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::SqliteEventStore;
use uuid::Uuid;

pub struct PlanOptions {
    pub format: Option<String>,
}

pub struct ApplyOptions {
    pub format: Option<String>,
    pub confirm: bool,
    pub dry_run: bool,
    pub command_id: Option<String>,
    pub scope: Option<String>,
}

/// `ai-brains retention plan [--format json|human]`
pub fn run_plan(ctx: &AppContext, options: PlanOptions) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let config = RetentionConfig::from_env();
    let report = plan_retention(&store, &config)?;
    emit_report(format, &report)
}

/// `ai-brains retention apply --confirm [--format json|human]`
pub fn run_apply(
    ctx: &AppContext,
    options: ApplyOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let format = OutputFormat::parse(options.format.as_deref());

    if options.dry_run && options.confirm {
        return fail(
            format,
            ApiError::new(
                "INVALID_PAYLOAD",
                "cannot combine --dry-run and --confirm; omit --confirm to plan, or pass --confirm alone to apply",
            ),
        );
    }
    if !options.confirm || options.dry_run {
        return fail(
            format,
            ApiError::new(
                "INVALID_PAYLOAD",
                "retention apply requires --confirm (dry-run is default; use `retention plan` for a report without disposal)",
            ),
        );
    }

    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(SqliteEventStore::new((*ctx.conn).clone()));
    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new((*ctx.conn).clone()));
    let config = RetentionConfig::from_env();
    let command_id = options
        .command_id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let scope = match options.scope.as_deref() {
        Some(s) if !s.trim().is_empty() => match ai_brains_control_plane::parse_scope_key(s) {
            Ok(sc) => sc,
            Err(e) => {
                return fail(
                    format,
                    ApiError::new("INVALID_PAYLOAD", format!("invalid --scope: {e}")),
                );
            }
        },
        _ => ScopeRef::Personal(UserId::new()),
    };

    // Operator-local disposal path (parity with nightly projection cleanup).
    // CE still goes through wipe_content_envelope only (R2); AllowAll is local ops trust.
    let principal = make_principal(PrincipalKind::Human, PrincipalId::new(), "retention-cli");

    match apply_retention(
        &store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config,
        RetentionApplyCommand {
            principal,
            scope,
            command_id: command_id.clone(),
            confirm: true,
            dry_run: false,
        },
    ) {
        Ok(mut report) => {
            if !report.warnings.iter().any(|w| w.contains("command_id=")) {
                report.warnings.push(format!("command_id={command_id}"));
            }
            let exit_err =
                report.errors_count > 0 && report.errors.iter().any(|e| e.starts_with("ce_wipe "));
            emit_report(format, &report)?;
            if exit_err {
                return Err("retention apply had CE failures (see report errors)".into());
            }
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("confirm") || msg.contains("dry_run") {
                fail(format, ApiError::new("INVALID_PAYLOAD", msg))
            } else {
                fail(format, ApiError::new("QUERY_FAILED", msg))
            }
        }
    }
}

fn emit_report(
    format: OutputFormat,
    report: &RetentionPlanReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => emit_json(report),
        OutputFormat::Human | OutputFormat::Markdown => {
            let mut lines = Vec::new();
            lines.push(format!(
                "Retention plan (mode={}, generated_at={})",
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

fn fail(format: OutputFormat, err: ApiError) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            let _ = emit_json(&err);
            Err(err.message.into())
        }
        OutputFormat::Human | OutputFormat::Markdown => {
            eprintln!("error: {}", err.message);
            Err(err.message.into())
        }
    }
}
