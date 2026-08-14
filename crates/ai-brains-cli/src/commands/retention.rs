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
use ai_brains_contracts::retention::{
    CANONICAL_CLASSES, CLASS_DECISION_APPROVED, CLASS_EVIDENCE, CLASS_MEMORY_LEGACY,
    CLASS_ORPHANED_ENVELOPE, CLASS_QUERY_TRACE, CLASS_RAW_TURN, CLASS_REVIEW_TRACE, CLASS_SECRET,
    CLASS_UNCLASSIFIED, MECHANISM_CE_WIPE, MECHANISM_PROJECTION_DELETE, MECHANISM_SKIP,
    RETENTION_HONESTY_LEGACY_NOT_CE, RETENTION_HONESTY_NOT_NIST_PURGE,
    RETENTION_HONESTY_PRE_ERASE_BACKUP, RETENTION_HONESTY_STREAM_INDEPENDENCE,
    RETENTION_HONESTY_TICKET_NOT_CE, RetentionClassBucket, RetentionPlanReport, is_canonical_class,
    truncate_id,
};
use ai_brains_control_plane::{
    RetentionConfig, StorePorts, cascade_memory_ids_for_keys, execute_retention_projection_deletes,
    finalize_retention_apply, parse_scope_key, plan_retention, prepare_retention_apply,
    scope_identity_key,
};
use ai_brains_core::scope::ScopeRef;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use chrono::{DateTime, Utc};
use is_terminal::IsTerminal;
use std::collections::BTreeMap;

pub struct PlanOptions {
    pub format: String,
}

pub struct ApplyOptions {
    pub format: String,
    pub confirm: bool,
    pub dry_run: bool,
    pub command_id: Option<String>,
    pub scope: Option<String>,
    pub principal_id: Option<String>,
}

/// `ai-brains retention plan [--format auto|pretty|human|text|json|markdown|md]`
pub fn run_plan(ctx: &AppContext, options: PlanOptions) -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve_retention_format(&options.format, std::io::stdout().is_terminal());
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let config = RetentionConfig::from_env();
    let report = plan_retention(&store, &config)?;
    emit_report(resolved, &report)
}

/// `ai-brains retention apply --confirm`
///
/// Sync entry (like other local vault commands). Daemon I/O uses the current
/// Tokio handle so we do not grow a large async state machine on this path.
pub fn run_apply(
    ctx: &AppContext,
    options: ApplyOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Apply never TTY-switches: auto stays JSON even on a TTY (F4).
    let resolved = resolve_retention_format(&options.format, false);
    let format = retention_output_format(resolved);

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
    let scope =
        match resolve_retention_apply_scope(options.scope.as_deref(), plan.totals.would_ce_wipe) {
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
    emit_report(resolved, &report)?;
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
        (true, None) => Err("retention apply with CE candidates requires --scope \
             (e.g. Repository:<uuid> or Personal:<uuid>); \
             refusing random default scope"
            .into()),
        (_, Some(s)) => parse_scope_key(s)
            .map(Some)
            .map_err(|e| format!("invalid --scope: {e}")),
        (false, None) => Ok(None),
    }
}

/// Resolve plan/apply `--format` (T248). Clap rejects unknowns; `_` is fail-closed json.
pub(crate) fn resolve_retention_format(explicit: &str, is_tty: bool) -> &'static str {
    match explicit {
        "pretty" | "human" | "text" | "markdown" | "md" => "human",
        "json" => "json",
        "auto" if is_tty => "human",
        "auto" => "json",
        _ => "json",
    }
}

fn retention_output_format(resolved: &str) -> OutputFormat {
    match resolved {
        "human" => OutputFormat::Human,
        _ => OutputFormat::Json,
    }
}

/// T166 v1 policy default mechanism for a zero-count pretty row (display only).
fn zero_row_mechanism(class: &str) -> &'static str {
    match class {
        CLASS_RAW_TURN | CLASS_QUERY_TRACE | CLASS_REVIEW_TRACE | CLASS_DECISION_APPROVED => {
            MECHANISM_PROJECTION_DELETE
        }
        CLASS_EVIDENCE | CLASS_SECRET | CLASS_ORPHANED_ENVELOPE => MECHANISM_CE_WIPE,
        CLASS_MEMORY_LEGACY | CLASS_UNCLASSIFIED => MECHANISM_SKIP,
        _ => MECHANISM_SKIP,
    }
}

fn format_horizon_display(raw: &str) -> String {
    if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_digit()) {
        format!("{raw}d")
    } else {
        raw.to_string()
    }
}

fn strip_frac_seconds(raw: &str) -> String {
    let Some(dot) = raw.find('.') else {
        return raw.to_string();
    };
    let prefix = &raw[..dot];
    let rest = &raw[dot..];
    let tz_at = rest.find(['+', '-', 'Z', 'z']).unwrap_or(rest.len());
    format!("{prefix}{}", &rest[tz_at..])
}

fn format_generated_at(raw: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return dt
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
    }
    let stripped = strip_frac_seconds(raw);
    if let Ok(dt) = DateTime::parse_from_rfc3339(&stripped) {
        return dt
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
    }
    stripped
}

fn honesty_short_label(warning: &str) -> String {
    if warning == RETENTION_HONESTY_LEGACY_NOT_CE {
        return "projection delete ≠ CE".into();
    }
    if warning == RETENTION_HONESTY_NOT_NIST_PURGE {
        return "not NIST Purge/Destroy".into();
    }
    if warning == RETENTION_HONESTY_STREAM_INDEPENDENCE {
        return "stream A and B independent until subject join".into();
    }
    if warning == RETENTION_HONESTY_TICKET_NOT_CE {
        return "ticket / soft forget ≠ CE".into();
    }
    if warning == RETENTION_HONESTY_PRE_ERASE_BACKUP {
        return "pre-erase backups remain decryptable".into();
    }
    warning.to_string()
}

fn sample_cell(ids: &[String]) -> String {
    if ids.is_empty() {
        "—".into()
    } else {
        ids.join(", ")
    }
}

fn horizon_cell(report: &RetentionPlanReport, class: &str) -> String {
    report
        .horizons
        .get(class)
        .map(|h| format_horizon_display(h))
        .unwrap_or_else(|| "—".into())
}

fn class_bucket_map(report: &RetentionPlanReport) -> BTreeMap<&str, &RetentionClassBucket> {
    report
        .classes
        .iter()
        .map(|c| (c.class.as_str(), c))
        .collect()
}

pub(crate) fn format_retention_pretty(report: &RetentionPlanReport) -> String {
    let mut lines = Vec::new();
    let is_apply = report.mode == "apply";
    let title = if is_apply {
        "Retention apply"
    } else {
        "Retention plan (dry-run)"
    };
    lines.push(format!(
        "{title}  generated {}",
        format_generated_at(&report.generated_at)
    ));
    lines.push(String::new());

    let empty = report.classes.is_empty() || report.totals.candidates == 0;
    if empty {
        lines.push("Nothing to dispose.".into());
        lines.push(String::new());
    } else {
        lines.push("Work".into());
        lines.push(format!(
            "{:<18} {:>5} {:<18} {}",
            "CLASS", "COUNT", "MECHANISM", "SAMPLES"
        ));
        for c in &report.classes {
            if c.candidate_count == 0 {
                continue;
            }
            lines.push(format!(
                "{:<18} {:>5} {:<18} {}",
                c.class,
                c.candidate_count,
                c.mechanism,
                sample_cell(&c.sample_ids)
            ));
        }
        lines.push(String::new());
    }

    lines.push("Class matrix".into());
    lines.push(format!(
        "{:<18} {:<36} {:<18} {:>5}",
        "CLASS", "HORIZON", "MECHANISM", "COUNT"
    ));
    let buckets = class_bucket_map(report);
    for class in CANONICAL_CLASSES {
        let (mechanism, count) = match buckets.get(class) {
            Some(b) => (b.mechanism.as_str(), b.candidate_count),
            None => (zero_row_mechanism(class), 0),
        };
        lines.push(format!(
            "{:<18} {:<36} {:<18} {:>5}",
            class,
            horizon_cell(report, class),
            mechanism,
            count
        ));
    }
    for c in &report.classes {
        if is_canonical_class(&c.class) {
            continue;
        }
        lines.push(format!(
            "{:<18} {:<36} {:<18} {:>5}",
            c.class,
            horizon_cell(report, &c.class),
            c.mechanism,
            c.candidate_count
        ));
    }
    lines.push(String::new());

    lines.push(format!(
        "Totals  candidates={} ce_wipe={} projection_delete={} skip={} held={}",
        report.totals.candidates,
        report.totals.would_ce_wipe,
        report.totals.would_projection_delete,
        report.totals.would_skip,
        report.totals.would_held
    ));

    if !report.warnings.is_empty() {
        lines.push(String::new());
        lines.push("Honesty".into());
        for w in &report.warnings {
            lines.push(format!("  {}", honesty_short_label(w)));
        }
    }

    if report.cascade.parents_marked_for_resynthesis > 0 {
        lines.push(String::new());
        lines.push(format!(
            "Cascade  parents_marked_for_resynthesis={}",
            report.cascade.parents_marked_for_resynthesis
        ));
    }

    if !report.errors.is_empty() {
        lines.push(String::new());
        lines.push("Errors:".into());
        for e in &report.errors {
            lines.push(format!("  {e}"));
        }
    }

    if !is_apply && report.totals.would_ce_wipe + report.totals.would_projection_delete > 0 {
        lines.push(String::new());
        if report.totals.would_ce_wipe > 0 {
            lines
                .push("next: ai-brains retention apply --confirm --scope Repository:<uuid>".into());
        } else {
            lines.push("next: ai-brains retention apply --confirm".into());
        }
    }

    lines.join("\n")
}

fn emit_report(
    resolved: &str,
    report: &RetentionPlanReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match resolved {
        "human" => {
            emit_human(&format_retention_pretty(report));
            Ok(())
        }
        _ => emit_json(report),
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
        let err = resolve_retention_apply_scope(Some("not-a-scope"), 1).expect_err("invalid");
        assert!(
            err.contains("invalid --scope") || err.contains("unparseable"),
            "{err}"
        );
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

    #[test]
    fn resolve_retention_format__auto_tty__human() {
        assert_eq!(resolve_retention_format("auto", true), "human");
    }

    #[test]
    fn resolve_retention_format__auto_pipe__json() {
        assert_eq!(resolve_retention_format("auto", false), "json");
    }

    #[test]
    fn resolve_retention_format__pretty_aliases__human_regardless_of_tty() {
        for token in ["pretty", "human", "text", "markdown", "md"] {
            assert_eq!(
                resolve_retention_format(token, true),
                "human",
                "{token} tty"
            );
            assert_eq!(
                resolve_retention_format(token, false),
                "human",
                "{token} pipe"
            );
        }
    }

    #[test]
    fn resolve_retention_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_retention_format("json", true), "json");
        assert_eq!(resolve_retention_format("json", false), "json");
    }

    #[test]
    fn resolve_retention_format__apply_auto_even_on_tty__json() {
        // Apply always passes is_tty: false; auto must not TTY-switch.
        assert_eq!(resolve_retention_format("auto", false), "json");
        assert_eq!(resolve_retention_format("auto", true), "human");
    }

    #[test]
    fn format_retention_pretty__empty__nothing_to_dispose_matrix_skip_no_next() {
        use ai_brains_contracts::retention::{
            CLASS_MEMORY_LEGACY, RetentionPlanReport, RetentionReportMode,
        };
        let report =
            RetentionPlanReport::empty(RetentionReportMode::DryRun, "2026-08-14T03:12:41Z");
        let text = format_retention_pretty(&report);
        assert!(
            text.contains("Nothing to dispose."),
            "empty pretty must say nothing to dispose; got:\n{text}"
        );
        for class in CANONICAL_CLASSES {
            assert!(text.contains(class), "matrix missing {class}; got:\n{text}");
        }
        assert!(
            text.contains("90d"),
            "numeric horizon must gain d suffix; got:\n{text}"
        );
        assert!(
            text.contains("Totals  candidates=0 ce_wipe=0 projection_delete=0 skip=0 held=0"),
            "exact Totals line missing; got:\n{text}"
        );
        assert!(
            text.contains("projection delete ≠ CE"),
            "honesty short missing; got:\n{text}"
        );
        assert!(
            text.contains("not NIST Purge/Destroy"),
            "NIST short missing; got:\n{text}"
        );
        assert!(
            text.contains("stream A and B independent until subject join"),
            "stream short missing; got:\n{text}"
        );
        assert!(
            text.contains("ticket / soft forget ≠ CE"),
            "ticket short missing; got:\n{text}"
        );
        let legacy_line = text
            .lines()
            .find(|l| l.contains(CLASS_MEMORY_LEGACY))
            .unwrap_or("");
        assert!(
            legacy_line.contains(MECHANISM_SKIP),
            "memory_legacy zero-row must be skip; line={legacy_line}"
        );
        assert!(
            !legacy_line.contains("soft_forget"),
            "memory_legacy must not be soft_forget; line={legacy_line}"
        );
        assert!(
            !legacy_line.contains("held"),
            "memory_legacy must not be held; line={legacy_line}"
        );
        assert!(
            !text.contains("next: ai-brains retention apply"),
            "empty plan must omit next:; got:\n{text}"
        );
        assert!(
            text.contains("Retention plan (dry-run)"),
            "plan title missing; got:\n{text}"
        );
        assert!(
            text.contains("Class matrix"),
            "Class matrix header missing; got:\n{text}"
        );
    }

    fn fixture_raw_turn_report() -> RetentionPlanReport {
        use ai_brains_contracts::retention::{
            CLASS_RAW_TURN, MECHANISM_PROJECTION_DELETE, RetentionCascade, RetentionTotals,
            default_horizon_labels,
        };
        RetentionPlanReport {
            api_version: "1".into(),
            generated_at: "2026-08-14T03:12:41.076921700+00:00".into(),
            mode: "dry_run".into(),
            horizons: default_horizon_labels(),
            classes: vec![RetentionClassBucket {
                class: CLASS_RAW_TURN.into(),
                candidate_count: 2,
                mechanism: MECHANISM_PROJECTION_DELETE.into(),
                sample_ids: vec!["sess:0".into(), "sess:1".into()],
                notes: vec!["event log retained".into()],
            }],
            totals: RetentionTotals {
                candidates: 2,
                would_ce_wipe: 0,
                would_projection_delete: 2,
                would_skip: 0,
                would_held: 0,
            },
            cascade: RetentionCascade::default(),
            warnings: RetentionPlanReport::honesty_warnings(false),
            errors_count: 0,
            errors: Vec::new(),
        }
    }

    #[test]
    fn format_retention_pretty__raw_turn_work__comma_samples_next_confirm_no_scope() {
        let text = format_retention_pretty(&fixture_raw_turn_report());
        assert!(
            text.contains("Work"),
            "non-empty must print Work table; got:\n{text}"
        );
        assert!(
            text.contains("sess:0, sess:1"),
            "samples must be comma-joined; got:\n{text}"
        );
        assert!(
            !text.contains("[\"sess:0\""),
            "samples must not be Debug; got:\n{text}"
        );
        assert!(
            text.contains("next: ai-brains retention apply --confirm"),
            "projection-only next: missing; got:\n{text}"
        );
        assert!(
            !text.contains("--scope"),
            "projection-only next: must omit --scope; got:\n{text}"
        );
    }

    #[test]
    fn format_retention_pretty__ce_wipe__next_includes_scope_and_confirm() {
        use ai_brains_contracts::retention::{
            CLASS_SECRET, MECHANISM_CE_WIPE, RetentionCascade, RetentionTotals,
            default_horizon_labels,
        };
        let report = RetentionPlanReport {
            api_version: "1".into(),
            generated_at: "2026-08-14T03:12:41Z".into(),
            mode: "dry_run".into(),
            horizons: default_horizon_labels(),
            classes: vec![RetentionClassBucket {
                class: CLASS_SECRET.into(),
                candidate_count: 1,
                mechanism: MECHANISM_CE_WIPE.into(),
                sample_ids: vec!["ck-1".into()],
                notes: Vec::new(),
            }],
            totals: RetentionTotals {
                candidates: 1,
                would_ce_wipe: 1,
                would_projection_delete: 0,
                would_skip: 0,
                would_held: 0,
            },
            cascade: RetentionCascade::default(),
            warnings: RetentionPlanReport::honesty_warnings(true),
            errors_count: 0,
            errors: Vec::new(),
        };
        let text = format_retention_pretty(&report);
        assert!(
            text.contains("next: ai-brains retention apply --confirm --scope Repository:<uuid>"),
            "CE next: missing scope placeholder; got:\n{text}"
        );
        assert!(
            text.contains("pre-erase backups remain decryptable"),
            "CE honesty short missing; got:\n{text}"
        );
    }

    #[test]
    fn format_retention_pretty__custom_horizon_45__shows_45d() {
        let mut report = fixture_raw_turn_report();
        report
            .horizons
            .insert(CLASS_RAW_TURN.to_string(), "45".into());
        let text = format_retention_pretty(&report);
        assert!(
            text.contains("45d"),
            "custom numeric horizon must show 45d; got:\n{text}"
        );
        assert!(
            !text
                .lines()
                .any(|l| l.contains(CLASS_RAW_TURN) && l.contains("90d")),
            "must not hardcode 90d when horizon is 45; got:\n{text}"
        );
    }

    #[test]
    fn format_retention_pretty__nanos_timestamp__utc_minutes_no_frac() {
        let text = format_retention_pretty(&fixture_raw_turn_report());
        assert!(
            text.contains("2026-08-14 03:12 UTC"),
            "human timestamp missing; got:\n{text}"
        );
        assert!(
            !text.contains(".076"),
            "nanos must not appear on human title; got:\n{text}"
        );
    }

    #[test]
    fn format_retention_pretty__unknown_warning__echoed() {
        let mut report = fixture_raw_turn_report();
        report.warnings.push("brand_new_honesty_token_xyz".into());
        let text = format_retention_pretty(&report);
        assert!(
            text.contains("brand_new_honesty_token_xyz"),
            "unknown warning must echo; got:\n{text}"
        );
        assert!(
            text.contains("projection delete ≠ CE"),
            "known short still required; got:\n{text}"
        );
    }

    #[test]
    fn format_retention_pretty__apply_mode__title_omits_next() {
        let mut report = fixture_raw_turn_report();
        report.mode = "apply".into();
        let text = format_retention_pretty(&report);
        assert!(
            text.contains("Retention apply"),
            "apply title missing; got:\n{text}"
        );
        assert!(
            !text.contains("next: ai-brains retention apply"),
            "apply human must omit next:; got:\n{text}"
        );
    }
}
