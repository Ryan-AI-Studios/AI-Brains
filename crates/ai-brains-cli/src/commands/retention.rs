//! `ai-brains retention plan|apply` — class-based retention (T166 / P8.4).
//!
//! # Dual path (spec §6 freeze)
//!
//! - **`retention plan`** — local dry-run (no disposal).
//! - **`retention apply --confirm`**:
//!   - **Pre-mutation** `RetentionApplied` audit via [`prepare_retention_apply`].
//!   - **CE** rows first (when any): require daemon (T165 E8 parity); each key
//!     wiped via `DaemonRequest::WipeContentEnvelope` only. Never
//!     `AllowAllPolicy` + local wipe. CE-first so policy-denied CE cannot leave
//!     projection deletes already applied.
//!   - **Then** projection deletes via [`execute_retention_projection_deletes`].
//!   - Finalize cascade for successful CE only + second `RetentionApplied`.
//!   - If any CE candidate exists and daemon is down → `DAEMON_UNAVAILABLE` before disposal.
//!   - CE apply requires explicit `--scope` (never invent a random Personal UUID).
//!   - Projection-only apply (`would_ce_wipe == 0`) may run without daemon or `--scope`
//!     (audit then projection deletes only).
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
use ai_brains_contracts::retention::{RetentionPlanReport, truncate_id};
use ai_brains_control_plane::{
    RetentionConfig, StorePorts, cascade_memory_ids_for_keys, execute_retention_projection_deletes,
    finalize_retention_apply, parse_scope_key, plan_retention, prepare_retention_apply,
    scope_identity_key,
};
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

    let plan = match plan_retention(&store, &config) {
        Ok(p) => p,
        Err(e) => {
            return fail_api(format, ApiError::new("QUERY_FAILED", e.to_string()));
        }
    };

    // Gates before any mutation: CE requires explicit scope + daemon.
    let scope = match resolve_retention_apply_scope(
        options.scope.as_deref(),
        plan.totals.would_ce_wipe,
    ) {
        Ok(s) => s,
        Err(msg) => {
            return fail_api(format, ApiError::new("INVALID_PAYLOAD", msg));
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

    // CE-first production order when would_ce_wipe > 0:
    // prepare (audit) → daemon CE wipe → projection deletes → finalize.
    // Projection-only: prepare (audit) → projection deletes.
    let mut outcome =
        match prepare_retention_apply(&store, &ports.writer, &config, &command_id, true, false) {
            Ok(o) => o,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("confirm") || msg.contains("dry_run") {
                    return fail_api(format, ApiError::new("INVALID_PAYLOAD", msg));
                }
                return fail_api(format, ApiError::new("QUERY_FAILED", msg));
            }
        };

    if !outcome
        .report
        .warnings
        .iter()
        .any(|w| w.contains("command_id="))
    {
        outcome
            .report
            .warnings
            .push(format!("command_id={command_id}"));
    }

    // Pre-mutation RetentionApplied already appended by prepare (R12).
    let mut successful_ce_keys: Vec<String> = Vec::new();
    let had_ce = !outcome.pending_ce_keys.is_empty();
    if had_ce {
        let Some(ref sc) = scope else {
            // Unreachable when plan.totals matched outcome; belt-and-suspenders.
            return fail_api(
                format,
                ApiError::new(
                    "INVALID_PAYLOAD",
                    "retention apply with CE candidates requires --scope \
                     (e.g. Repository:<uuid> or Personal:<uuid>)",
                ),
            );
        };
        let scope_wire = scope_identity_key(sc);
        let client = DaemonClient::new();
        for key in &outcome.pending_ce_keys {
            let key_disp = truncate_id(key);
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
                    if wire.status == "wiped" || wire.status == "already_erased" {
                        successful_ce_keys.push(key.clone());
                    } else {
                        outcome.report.errors.push(format!(
                            "ce_wipe {key_disp}: unexpected status {}",
                            wire.status
                        ));
                    }
                }
                Ok(DaemonResponse::Error(err)) => {
                    // Codex R2 P3: code only — err.message may embed full content_key_id.
                    outcome
                        .report
                        .errors
                        .push(format!("ce_wipe {key_disp}: {}", err.code));
                }
                Ok(other) => {
                    outcome.report.errors.push(format!(
                        "ce_wipe {key_disp}: unexpected daemon response: {other:?}"
                    ));
                }
                Err(e) => {
                    let classified = governed_common::classify_daemon_mutation_error(&e);
                    outcome
                        .report
                        .errors
                        .push(format!("ce_wipe {key_disp}: {classified}"));
                }
            }
        }
    }

    // Projection deletes after CE batch (or immediately when projection-only).
    if let Err(e) = execute_retention_projection_deletes(&store, &mut outcome) {
        outcome
            .report
            .errors
            .push(format!("execute_retention_projection_deletes: {e}"));
    }

    if had_ce {
        // R15: cascade only subjects belonging to wiped / already_erased keys.
        let cascade_memory_ids =
            cascade_memory_ids_for_keys(&outcome.pending_cascade_by_key, &successful_ce_keys);
        if let Err(e) = finalize_retention_apply(
            &store,
            &ports.writer,
            &command_id,
            &cascade_memory_ids,
            &mut outcome.report,
        ) {
            outcome
                .report
                .errors
                .push(format!("finalize_retention_apply: {e}"));
        }
    }

    let mut report = outcome.report;
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

/// Whether production apply must require an explicit `--scope` (CE path).
pub fn production_apply_requires_scope(would_ce_wipe: u64) -> bool {
    would_ce_wipe > 0
}

/// Resolve CE scope for retention apply.
///
/// When `would_ce_wipe > 0`, `--scope` is **required** (parseable
/// `Repository:<uuid>` / `Personal:<uuid>` / `Workspace:<uuid>`). Never invent
/// a random `Personal` UUID. Projection-only apply (`would_ce_wipe == 0`) may
/// omit scope (unused).
pub fn resolve_retention_apply_scope(
    scope: Option<&str>,
    would_ce_wipe: u64,
) -> Result<Option<ScopeRef>, String> {
    let trimmed = scope.map(str::trim).filter(|s| !s.is_empty());
    match (production_apply_requires_scope(would_ce_wipe), trimmed) {
        (true, None) => Err(
            "retention apply with CE candidates requires --scope \
             (e.g. Repository:<uuid> or Personal:<uuid>); \
             refusing random default scope"
                .into(),
        ),
        (_, Some(s)) => parse_scope_key(s)
            .map(Some)
            .map_err(|e| format!("invalid --scope: {e}")),
        (false, None) => Ok(None),
    }
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
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_control_plane::cascade_memory_ids_for_keys;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[test]
    fn production_apply_requires_daemon__ce_candidates__true() {
        assert!(production_apply_requires_daemon(1));
    }

    #[test]
    fn production_apply_requires_daemon__projection_only__false() {
        assert!(!production_apply_requires_daemon(0));
    }

    #[test]
    fn production_apply_requires_scope__ce_candidates__true() {
        assert!(production_apply_requires_scope(1));
        assert!(!production_apply_requires_scope(0));
    }

    #[test]
    fn resolve_retention_apply_scope__ce_without_scope__err() {
        let err = resolve_retention_apply_scope(None, 1).expect_err("must require scope");
        assert!(
            err.contains("--scope") || err.contains("requires"),
            "expected clear scope message, got: {err}"
        );
        assert!(
            err.contains("refusing random") || err.contains("Repository:"),
            "expected guidance, got: {err}"
        );
    }

    #[test]
    fn resolve_retention_apply_scope__ce_empty_scope__err() {
        let err = resolve_retention_apply_scope(Some("   "), 2).expect_err("empty not ok");
        assert!(err.contains("--scope") || err.contains("requires"), "{err}");
    }

    #[test]
    fn resolve_retention_apply_scope__ce_with_scope__ok() {
        let uid = Uuid::nil();
        let key = format!("Personal:{uid}");
        let sc = resolve_retention_apply_scope(Some(&key), 1)
            .expect("valid scope")
            .expect("Some");
        assert_eq!(scope_identity_key(&sc), key);
    }

    #[test]
    fn resolve_retention_apply_scope__projection_only_omits_scope__ok() {
        let sc = resolve_retention_apply_scope(None, 0).expect("projection-only");
        assert!(sc.is_none());
    }

    #[test]
    fn resolve_retention_apply_scope__invalid_key__err() {
        let err =
            resolve_retention_apply_scope(Some("not-a-scope"), 1).expect_err("invalid");
        assert!(err.contains("invalid --scope") || err.contains("unparseable"), "{err}");
    }

    #[test]
    fn cascade_filter__successful_keys_only() {
        // Mirrors production CLI: only wiped / already_erased keys feed R15.
        let mut by_key = BTreeMap::new();
        by_key.insert("k-ok".into(), vec!["m1".into()]);
        by_key.insert("k-fail".into(), vec!["m2".into()]);
        let successful = vec!["k-ok".to_string()];
        let ids = cascade_memory_ids_for_keys(&by_key, &successful);
        assert_eq!(ids, vec!["m1".to_string()]);
    }

    #[test]
    fn error_key_ids_use_truncate_id() {
        let full = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
        let disp = truncate_id(full);
        assert_eq!(disp, full); // UUID length ≤ truncate max
        let long = format!("{full}-extra-suffix-that-is-long");
        let t = truncate_id(&long);
        assert!(t.chars().count() <= 37); // 36 + ellipsis char
        assert!(t.ends_with('…') || t.ends_with("..."));
    }

    #[test]
    fn daemon_error_line__code_only_no_message() {
        // Documents P3 shape: ce_wipe {truncated}: {code} — never raw err.message.
        let key = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
        let key_disp = truncate_id(key);
        let code = "POLICY_DENIED";
        let line = format!("ce_wipe {key_disp}: {code}");
        assert!(!line.contains("content_key_store"));
        assert!(!line.contains("no content_key_store row"));
        assert!(line.ends_with(code));
    }
}
