//! T239/T334 — multi-harness nightly import orchestration
//! (agy → grok → opencode → claude → codex → cursor).
//!
//! Calls adapter `import_*_sessions` directly with **per-source** `StoreSink`s
//! (D5/D21). Fail-open per source. Hermetic overrides live on
//! [`MultiImportOptions`] (D20) — never env-based inject for tests.

use crate::context::{AppContext, StoreSink};
use ai_brains_adapters::{
    AntigravityImportOptions, ClaudeImportOptions, CodexImportOptions, CursorImportOptions,
    GrokImportOptions, OpenCodeImportOptions, import_antigravity_sessions, import_claude_sessions,
    import_codex_sessions, import_cursor_sessions, import_grok_sessions, import_opencode_sessions,
};
use ai_brains_capture::CaptureService;
use ai_brains_core::ids::ProjectId;
use ai_brains_store::{EventStore, QueryStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

/// Sync-state key for the last multi-import report JSON (`v:1`).
pub const LAST_MULTI_IMPORT_KEY: &str = "last_multi_import";

/// Options for [`run_multi_harness_import`].
#[derive(Debug, Clone, Default)]
pub struct MultiImportOptions {
    /// Global skip — all sources `skipped` with reason `skip_import` (D3).
    pub skip_import: bool,
    pub skip_agy: bool,
    pub skip_grok: bool,
    pub skip_opencode: bool,
    pub skip_claude: bool,
    pub skip_codex: bool,
    pub skip_cursor: bool,
    /// Lookback days (default 30 — D10).
    pub days: usize,
    pub force: bool,
    /// OpenCode list/export cap (default adapter 100).
    pub max_sessions: usize,
    /// Hermetic: AGY home override (D20).
    pub agy_home_override: Option<PathBuf>,
    /// Hermetic: Grok user-home override (D20).
    pub grok_home_override: Option<PathBuf>,
    /// Hermetic: inject OpenCode list JSON (D20).
    pub opencode_list_json_override: Option<String>,
    /// Hermetic: OpenCode export fixture directory (D20).
    pub opencode_export_dir_override: Option<PathBuf>,
    /// Hermetic: OpenCode cursor path (D20).
    pub opencode_cursor_path_override: Option<PathBuf>,
    /// Hermetic / relocated: OpenCode config dir (D20).
    pub opencode_config_dir_override: Option<PathBuf>,
    /// Hermetic: Claude user-home override (T334).
    pub claude_home_override: Option<PathBuf>,
    /// Hermetic: Codex user-home override (T334).
    pub codex_home_override: Option<PathBuf>,
    /// Hermetic: Cursor user-home override (T334).
    pub cursor_home_override: Option<PathBuf>,
}

impl MultiImportOptions {
    pub fn production(
        skip_import: bool,
        skip_agy: bool,
        skip_grok: bool,
        skip_opencode: bool,
        skip_claude: bool,
        skip_codex: bool,
        skip_cursor: bool,
    ) -> Self {
        let max_sessions = std::env::var("AI_BRAINS_NIGHTLY_OPENCODE_MAX")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(100);
        Self {
            skip_import,
            skip_agy,
            skip_grok,
            skip_opencode,
            skip_claude,
            skip_codex,
            skip_cursor,
            days: 30,
            force: false,
            max_sessions,
            agy_home_override: None,
            grok_home_override: None,
            opencode_list_json_override: None,
            opencode_export_dir_override: None,
            opencode_cursor_path_override: None,
            opencode_config_dir_override: None,
            claude_home_override: None,
            codex_home_override: None,
            cursor_home_override: None,
        }
    }
}

/// Per-source import outcome (D6 / F24).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceImportReport {
    /// `ok` | `error` | `skipped`
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    #[serde(default)]
    pub sessions: usize,
    #[serde(default)]
    pub imported_turns: usize,
    #[serde(default)]
    pub unbound: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// OpenCode health (D6/M4) — only set for the OpenCode source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_capped: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_errors: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timed_out: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped_missing_binary: Option<usize>,
    /// Absolute OpenCode CLI used when resolve succeeded (T339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_bin: Option<String>,
    /// Sorted unique candidates checked; present on unresolved miss (T339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_attempts: Option<Vec<String>>,
}

impl SourceImportReport {
    fn skipped(reason: &str) -> Self {
        Self {
            status: "skipped".to_string(),
            skip_reason: Some(reason.to_string()),
            sessions: 0,
            imported_turns: 0,
            unbound: 0,
            error: None,
            list_capped: None,
            export_errors: None,
            timed_out: None,
            skipped_missing_binary: None,
            resolved_bin: None,
            binary_attempts: None,
        }
    }

    fn ok(sessions: usize, imported_turns: usize, unbound: usize) -> Self {
        Self {
            status: "ok".to_string(),
            skip_reason: None,
            sessions,
            imported_turns,
            unbound,
            error: None,
            list_capped: None,
            export_errors: None,
            timed_out: None,
            skipped_missing_binary: None,
            resolved_bin: None,
            binary_attempts: None,
        }
    }

    fn error(sessions: usize, imported_turns: usize, unbound: usize, error: String) -> Self {
        Self {
            status: "error".to_string(),
            skip_reason: None,
            sessions,
            imported_turns,
            unbound,
            error: Some(error),
            list_capped: None,
            export_errors: None,
            timed_out: None,
            skipped_missing_binary: None,
            resolved_bin: None,
            binary_attempts: None,
        }
    }
}

/// Aggregate multi-import report persisted as `last_multi_import` (D6/D8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiImportReport {
    pub v: u32,
    /// RFC3339 UTC timestamp (D8).
    pub at: String,
    pub agy: SourceImportReport,
    pub grok: SourceImportReport,
    pub opencode: SourceImportReport,
    /// Missing on pre-T334 blobs → [`absent_pre_t334_report`].
    #[serde(default = "absent_pre_t334_report")]
    pub claude: SourceImportReport,
    #[serde(default = "absent_pre_t334_report")]
    pub codex: SourceImportReport,
    #[serde(default = "absent_pre_t334_report")]
    pub cursor: SourceImportReport,
}

fn absent_pre_t334_report() -> SourceImportReport {
    SourceImportReport::skipped("absent_pre_t334")
}

impl MultiImportReport {
    fn new(
        agy: SourceImportReport,
        grok: SourceImportReport,
        opencode: SourceImportReport,
        claude: SourceImportReport,
        codex: SourceImportReport,
        cursor: SourceImportReport,
    ) -> Self {
        Self {
            v: 1,
            at: chrono::Utc::now().to_rfc3339(),
            agy,
            grok,
            opencode,
            claude,
            codex,
            cursor,
        }
    }
}

/// Build a fresh per-source `StoreSink` (D5) so `last_error` cannot leak across sources.
pub fn make_sink(ctx: &AppContext) -> StoreSink {
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(Arc::clone(&ctx.conn))),
    }
}

fn default_project_id() -> ProjectId {
    std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default()
}

fn format_source_error(source: &str, err: &dyn std::fmt::Display) -> String {
    // D22: prefer path/session when the adapter error already embeds them.
    format!("{source}: {err}")
}

/// Run AGY → Grok → OpenCode → Claude → Codex → Cursor import (or skip) and return a typed report.
///
/// Never aborts nightly for a single-source failure (D7). Caller persists the
/// report via [`persist_multi_import_report`].
pub fn run_multi_harness_import(ctx: &AppContext, opts: MultiImportOptions) -> MultiImportReport {
    let days = if opts.days == 0 { 30 } else { opts.days };
    let max_sessions = if opts.max_sessions == 0 {
        100
    } else {
        opts.max_sessions
    };
    let project_id = default_project_id();
    let query_store = ctx.conn.clone() as Arc<dyn QueryStore>;
    let service = CaptureService::new();

    let agy = run_agy_source(ctx, &query_store, &service, &opts, days, project_id);
    let grok = run_grok_source(ctx, &query_store, &service, &opts, days, project_id);
    let opencode = run_opencode_source(
        ctx,
        &query_store,
        &service,
        &opts,
        days,
        max_sessions,
        project_id,
    );
    let claude = run_claude_source(ctx, &query_store, &service, &opts, days, project_id);
    let codex = run_codex_source(ctx, &query_store, &service, &opts, days, project_id);
    let cursor = run_cursor_source(ctx, &query_store, &service, &opts, days, project_id);

    MultiImportReport::new(agy, grok, opencode, claude, codex, cursor)
}

fn source_skipped(
    opts: &MultiImportOptions,
    per_source: bool,
    per_reason: &str,
) -> Option<SourceImportReport> {
    if opts.skip_import {
        return Some(SourceImportReport::skipped("skip_import"));
    }
    if per_source {
        return Some(SourceImportReport::skipped(per_reason));
    }
    None
}

fn run_agy_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_agy, "skip_import_agy") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = AntigravityImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force: opts.force,
        home_override: opts.agy_home_override.clone(),
    };

    match import_antigravity_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            if let Some(err) = sink.last_error {
                // D21: partial counters + error status
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "AGY multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("agy", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    "AGY multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "AGY multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("agy", &e))
        }
    }
}

fn run_grok_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_grok, "skip_import_grok") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = GrokImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force: opts.force,
        home_override: opts.grok_home_override.clone(),
        dry_run: false,
    };

    match import_grok_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            if let Some(err) = sink.last_error {
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "Grok multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("grok", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    "Grok multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Grok multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("grok", &e))
        }
    }
}

fn run_opencode_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    max_sessions: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_opencode, "skip_import_opencode") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = OpenCodeImportOptions {
        days,
        force: opts.force,
        dry_run: false,
        max_sessions,
        default_project_id: project_id,
        allow_default_project: false,
        list_json_override: opts.opencode_list_json_override.clone(),
        export_json_override_dir: opts.opencode_export_dir_override.clone(),
        cursor_path_override: opts.opencode_cursor_path_override.clone(),
        config_dir_override: opts.opencode_config_dir_override.clone(),
        force_missing_binary: false,
        bin_override: None,
        list_cap: max_sessions,
    };

    match import_opencode_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            let mut report = if let Some(err) = sink.last_error {
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "OpenCode multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("opencode", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    list_capped = stats.list_capped,
                    "OpenCode multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            };
            // D6/M4: always surface OpenCode health counters when the source ran.
            report.list_capped = Some(stats.list_capped);
            report.export_errors = Some(stats.export_errors);
            report.timed_out = Some(stats.timed_out);
            report.skipped_missing_binary = Some(stats.skipped_missing_binary);
            report.resolved_bin = stats
                .resolved_bin
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned());
            if stats.skipped_missing_binary > 0 {
                report.binary_attempts = stats.binary_attempts.clone();
            }
            report
        }
        Err(e) => {
            tracing::error!(error = %e, "OpenCode multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("opencode", &e))
        }
    }
}

fn run_claude_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_claude, "skip_import_claude") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = ClaudeImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force: opts.force,
        home_override: opts.claude_home_override.clone(),
        dry_run: false,
    };

    match import_claude_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            if let Some(err) = sink.last_error {
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "Claude multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("claude", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    "Claude multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Claude multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("claude", &e))
        }
    }
}

fn run_codex_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_codex, "skip_import_codex") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = CodexImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force: opts.force,
        home_override: opts.codex_home_override.clone(),
        dry_run: false,
    };

    match import_codex_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            if let Some(err) = sink.last_error {
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "Codex multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("codex", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    "Codex multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Codex multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("codex", &e))
        }
    }
}

fn run_cursor_source(
    ctx: &AppContext,
    query_store: &Arc<dyn QueryStore>,
    service: &CaptureService,
    opts: &MultiImportOptions,
    days: usize,
    project_id: ProjectId,
) -> SourceImportReport {
    if let Some(r) = source_skipped(opts, opts.skip_cursor, "skip_import_cursor") {
        return r;
    }

    let mut sink = make_sink(ctx);
    let options = CursorImportOptions {
        days,
        default_project_id: project_id,
        allow_default_project: false,
        force: opts.force,
        home_override: opts.cursor_home_override.clone(),
        dry_run: false,
    };

    match import_cursor_sessions(query_store.as_ref(), service, &mut sink, options) {
        Ok(stats) => {
            if let Some(err) = sink.last_error {
                tracing::error!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    error = %err,
                    "Cursor multi-import sink error (fail-open)"
                );
                SourceImportReport::error(
                    stats.sessions,
                    stats.imported_turns,
                    stats.unbound_project,
                    format_source_error("cursor", &err),
                )
            } else {
                tracing::info!(
                    sessions = stats.sessions,
                    imported_turns = stats.imported_turns,
                    "Cursor multi-import ok"
                );
                SourceImportReport::ok(stats.sessions, stats.imported_turns, stats.unbound_project)
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Cursor multi-import failed (fail-open)");
            SourceImportReport::error(0, 0, 0, format_source_error("cursor", &e))
        }
    }
}

/// Persist report JSON under `last_multi_import` (D8). Non-fatal on write failure.
pub fn persist_multi_import_report(event_store: &dyn EventStore, report: &MultiImportReport) {
    match serde_json::to_string(report) {
        Ok(json) => {
            if let Err(e) = event_store.set_sync_state(LAST_MULTI_IMPORT_KEY, &json) {
                tracing::warn!(error = %e, "failed to persist last_multi_import (non-fatal)");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to serialize last_multi_import (non-fatal)");
        }
    }
}

/// Status-line rendering for `nightly --status` (D9 / D23 / AC11 / AC12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiImportStatusView {
    Never,
    Unreadable,
    Report(Box<MultiImportReport>),
}

/// Load and parse `last_multi_import` for status display.
pub fn load_multi_import_status(
    query_store: &dyn QueryStore,
) -> Result<MultiImportStatusView, Box<dyn std::error::Error>> {
    match query_store.get_sync_state(LAST_MULTI_IMPORT_KEY)? {
        None => Ok(MultiImportStatusView::Never),
        Some(raw) => match serde_json::from_str::<MultiImportReport>(&raw) {
            Ok(report) if report.v == 1 => Ok(MultiImportStatusView::Report(Box::new(report))),
            Ok(report) => {
                tracing::warn!(
                    version = report.v,
                    "last_multi_import unreadable (unsupported version)"
                );
                Ok(MultiImportStatusView::Unreadable)
            }
            Err(e) => {
                tracing::warn!(error = %e, "last_multi_import unreadable (invalid JSON)");
                Ok(MultiImportStatusView::Unreadable)
            }
        },
    }
}

/// Print Multi-import block to stdout for `nightly --status`.
pub fn print_multi_import_status(view: &MultiImportStatusView) {
    match view {
        MultiImportStatusView::Never => {
            println!("Multi-import: never");
        }
        MultiImportStatusView::Unreadable => {
            println!("Multi-import: unreadable");
        }
        MultiImportStatusView::Report(report) => {
            println!("Multi-import: {}", report.at);
            print_source_line("agy", &report.agy);
            print_source_line("grok", &report.grok);
            print_source_line("opencode", &report.opencode);
            print_source_line("claude", &report.claude);
            print_source_line("codex", &report.codex);
            print_source_line("cursor", &report.cursor);
            if let Some(line) = opencode_cap_warning_line(&report.opencode) {
                println!("{line}");
            }
        }
    }
}

/// Pure OpenCode cap warning for status (AC12) — testable without stdout capture.
pub fn opencode_cap_warning_line(src: &SourceImportReport) -> Option<String> {
    let n = src.list_capped.filter(|n| *n > 0)?;
    Some(format!(
        "  OpenCode import capped — may be incomplete (list_capped={n})"
    ))
}

fn print_source_line(name: &str, src: &SourceImportReport) {
    match src.status.as_str() {
        "skipped" => {
            let reason = src.skip_reason.as_deref().unwrap_or("skipped");
            println!("  {name}: skipped ({reason})");
        }
        "error" => {
            let err = src.error.as_deref().unwrap_or("unknown");
            let mut line = format!(
                "  {name}: error sessions={} turns={} unbound={} err={err}",
                src.sessions, src.imported_turns, src.unbound
            );
            // AC12: surface OpenCode health on degraded/error rows when present
            append_opencode_health(&mut line, name, src);
            println!("{line}");
        }
        _ => {
            let mut line = format!(
                "  {name}: ok sessions={} turns={} unbound={}",
                src.sessions, src.imported_turns, src.unbound
            );
            append_opencode_health(&mut line, name, src);
            println!("{line}");
        }
    }
}

fn append_opencode_health(line: &mut String, name: &str, src: &SourceImportReport) {
    if name != "opencode" {
        return;
    }
    if let Some(n) = src.list_capped {
        line.push_str(&format!(" list_capped={n}"));
    }
    if let Some(n) = src.export_errors {
        line.push_str(&format!(" export_errors={n}"));
    }
    if let Some(n) = src.timed_out {
        line.push_str(&format!(" timed_out={n}"));
    }
    if let Some(bin) = src.resolved_bin.as_deref() {
        line.push_str(&format!(" resolved_bin={bin}"));
    }
    if let Some(n) = src.skipped_missing_binary {
        line.push_str(&format!(" skipped_missing_binary={n}"));
    }
    if src.resolved_bin.is_none()
        && let Some(attempts) = src.binary_attempts.as_ref()
        && !attempts.is_empty()
    {
        const MAX_SHOWN: usize = 3;
        let shown = attempts.iter().take(MAX_SHOWN).cloned().collect::<Vec<_>>();
        let extra = attempts.len().saturating_sub(shown.len());
        let mut bit = shown.join(";");
        if extra > 0 {
            bit.push_str(&format!(";+{extra}"));
        }
        line.push_str(&format!(" binary_attempts={bit}"));
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods, non_snake_case)]

    use super::*;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};
    use tempfile::tempdir;

    fn open_ctx(dir: &Path) -> AppContext {
        let db = dir.join("vault.db");
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        AppContext::from_resolved_key(db, sql_key).expect("open ctx")
    }

    fn filetime_set_mtime(path: &Path, t: SystemTime) -> std::io::Result<()> {
        let f = fs::File::options().write(true).open(path)?;
        f.set_modified(t)?;
        Ok(())
    }

    fn write_agy_brain(home: &Path, conversation_id: &str, body: &str) -> PathBuf {
        let logs = home
            .join(".gemini")
            .join("antigravity-cli")
            .join("brain")
            .join(conversation_id)
            .join(".system_generated")
            .join("logs");
        fs::create_dir_all(&logs).expect("mkdir logs");
        let path = logs.join("transcript.jsonl");
        fs::write(&path, body).expect("write transcript");
        let past = SystemTime::now() - Duration::from_secs(600);
        let _ = filetime_set_mtime(&path, past);
        path
    }

    fn write_grok_session(
        user_home: &Path,
        workspace: &str,
        session_id: &str,
        history_body: &str,
    ) -> PathBuf {
        // percent-encode is done by the adapter discovery; path component uses
        // the same encoding as production via ai_brains_adapters.
        let enc = ai_brains_adapters::percent_encode_path_component(workspace);
        let sess_dir = user_home
            .join(".grok")
            .join("sessions")
            .join(&enc)
            .join(session_id);
        fs::create_dir_all(&sess_dir).expect("mkdir session");
        let history = sess_dir.join("chat_history.jsonl");
        fs::write(&history, history_body).expect("write history");
        let summary = format!(
            r#"{{"info":{{"id":"{session_id}"}},"git_root_dir":{}}}"#,
            serde_json::to_string(workspace).unwrap()
        );
        fs::write(sess_dir.join("summary.json"), summary).expect("write summary");
        let past = SystemTime::now() - Duration::from_secs(600);
        let _ = filetime_set_mtime(&history, past);
        history
    }

    fn sample_opencode_export(session_id: &str, directory: &str, user: &str, asst: &str) -> String {
        format!(
            r#"{{
  "info": {{
    "id": "{session_id}",
    "directory": {dir_json},
    "time": {{ "created": 1700000000000, "updated": 1700000100000 }}
  }},
  "messages": [
    {{
      "info": {{ "role": "user", "id": "msg_u_{session_id}", "time": {{ "created": 1700000001000 }} }},
      "parts": [{{ "type": "text", "text": {user_json} }}]
    }},
    {{
      "info": {{ "role": "assistant", "id": "msg_a_{session_id}", "time": {{ "created": 1700000002000 }} }},
      "parts": [{{ "type": "text", "text": {asst_json} }}]
    }}
  ]
}}"#,
            session_id = session_id,
            dir_json = serde_json::to_string(directory).unwrap(),
            user_json = serde_json::to_string(user).unwrap(),
            asst_json = serde_json::to_string(asst).unwrap(),
        )
    }

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    fn hermetic_fixtures(root: &Path) -> MultiImportOptions {
        let agy_home = root.join("agy-home");
        let grok_home = root.join("grok-home");
        let export_dir = root.join("oc-exports");
        let cursor = root.join("oc-cursor.json");
        let workspace = root.join("ws-shared");
        let claude_home = root.join("claude-home");
        let codex_home = root.join("codex-home");
        let cursor_home = root.join("cursor-home");
        fs::create_dir_all(&agy_home).unwrap();
        fs::create_dir_all(&grok_home).unwrap();
        fs::create_dir_all(&export_dir).unwrap();
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&claude_home).unwrap();
        fs::create_dir_all(&codex_home).unwrap();
        fs::create_dir_all(&cursor_home).unwrap();
        let ws = workspace.to_string_lossy().to_string();

        // AGY
        let cid = "11111111-1111-1111-1111-111111111111";
        let hist_dir = agy_home.join(".gemini").join("antigravity-cli");
        fs::create_dir_all(&hist_dir).unwrap();
        let hist = hist_dir.join("history.jsonl");
        fs::write(
            &hist,
            format!(
                r#"{{"display":"t","timestamp":1000,"workspace":{},"conversationId":"{}"}}"#,
                serde_json::to_string(&ws).unwrap(),
                cid
            ),
        )
        .unwrap();
        write_agy_brain(
            &agy_home,
            cid,
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nmulti-agy\n</USER_REQUEST>","tool_calls":[]}
{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","content":"agy-ok","tool_calls":[]}
"#,
        );

        // Grok
        let gsid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        write_grok_session(
            &grok_home,
            &ws,
            gsid,
            r#"{"type":"user","content":"<user_query>\nmulti-grok\n</user_query>"}
{"type":"assistant","content":"grok-ok"}
"#,
        );

        // OpenCode
        let osid = "ses_multi_t239";
        fs::write(
            export_dir.join(format!("{osid}.json")),
            sample_opencode_export(osid, &ws, "multi-oc", "oc-ok"),
        )
        .unwrap();
        let list = format!(
            r#"[{{"id":{id},"directory":{dir},"updated":{updated}}}]"#,
            id = serde_json::to_string(osid).unwrap(),
            dir = serde_json::to_string(&ws).unwrap(),
            updated = now_ms(),
        );

        MultiImportOptions {
            skip_import: false,
            skip_agy: false,
            skip_grok: false,
            skip_opencode: false,
            skip_claude: false,
            skip_codex: false,
            skip_cursor: false,
            days: 30,
            force: true,
            max_sessions: 100,
            agy_home_override: Some(agy_home),
            grok_home_override: Some(grok_home),
            opencode_list_json_override: Some(list),
            opencode_export_dir_override: Some(export_dir),
            opencode_cursor_path_override: Some(cursor),
            opencode_config_dir_override: None,
            claude_home_override: Some(claude_home),
            codex_home_override: Some(codex_home),
            cursor_home_override: Some(cursor_home),
        }
    }

    fn hermetic_six_fixtures(root: &Path) -> MultiImportOptions {
        let mut opts = hermetic_fixtures(root);
        let claude_home = root.join("claude-home");
        let codex_home = root.join("codex-home");
        let cursor_home = root.join("cursor-home");
        fs::create_dir_all(&claude_home).unwrap();
        fs::create_dir_all(&codex_home).unwrap();
        fs::create_dir_all(&cursor_home).unwrap();
        let ws = root.join("ws-shared").to_string_lossy().to_string();

        let claude_sid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let enc = ai_brains_adapters::percent_encode_path_component(&ws);
        let claude_dir = claude_home.join(".claude").join("projects").join(&enc);
        fs::create_dir_all(&claude_dir).unwrap();
        let claude_path = claude_dir.join(format!("{claude_sid}.jsonl"));
        fs::write(
            &claude_path,
            r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hello-claude-multi"}}
{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"ok-claude-multi"}]}}
"#,
        )
        .unwrap();
        let past = SystemTime::now() - Duration::from_secs(600);
        let _ = filetime_set_mtime(&claude_path, past);

        let codex_sid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let codex_dir = codex_home
            .join(".codex")
            .join("sessions")
            .join("2026")
            .join("08")
            .join("15");
        fs::create_dir_all(&codex_dir).unwrap();
        let codex_path = codex_dir.join(format!("rollout-2026-08-15T12-00-00-{codex_sid}.jsonl"));
        fs::write(
            &codex_path,
            format!(
                r#"{{"timestamp":"2026-08-15T00:00:00Z","type":"session_meta","payload":{{"id":"{codex_sid}","cwd":{cwd}}}}}
{{"timestamp":"2026-08-15T00:00:02Z","type":"response_item","payload":{{"type":"message","role":"user","content":[{{"type":"input_text","text":"hello-codex-multi"}}]}}}}
{{"timestamp":"2026-08-15T00:00:03Z","type":"response_item","payload":{{"type":"message","role":"assistant","content":[{{"type":"output_text","text":"ok-codex-multi"}}]}}}}
"#,
                cwd = serde_json::to_string(&ws).unwrap(),
            ),
        )
        .unwrap();
        let _ = filetime_set_mtime(&codex_path, past);

        let cursor_sid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        let cursor_dir = cursor_home
            .join(".cursor")
            .join("projects")
            .join("c-dev-AI-Brains")
            .join("agent-transcripts")
            .join(cursor_sid);
        fs::create_dir_all(&cursor_dir).unwrap();
        let cursor_path = cursor_dir.join(format!("{cursor_sid}.jsonl"));
        fs::write(
            &cursor_path,
            r#"{"role":"user","message":{"content":[{"type":"text","text":"<user_query>\nhello-cursor-multi\n</user_query>"}]}}
{"role":"assistant","message":{"content":[{"type":"text","text":"ok-cursor-multi"}]}}
"#,
        )
        .unwrap();
        let _ = filetime_set_mtime(&cursor_path, past);

        opts.claude_home_override = Some(claude_home);
        opts.codex_home_override = Some(codex_home);
        opts.cursor_home_override = Some(cursor_home);
        opts
    }

    #[test]
    fn multi_import__hermetic_three_sources__all_ok_with_turns() {
        // AC1
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let opts = hermetic_fixtures(root.path());

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.v, 1);
        assert!(!report.at.is_empty());
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.grok.status, "ok");
        assert_eq!(report.opencode.status, "ok");
        assert!(report.agy.sessions >= 1, "agy sessions: {:?}", report.agy);
        assert!(
            report.agy.imported_turns >= 2,
            "agy turns: {:?}",
            report.agy
        );
        assert!(report.grok.sessions >= 1, "grok: {:?}", report.grok);
        assert!(report.grok.imported_turns >= 2);
        assert!(report.opencode.sessions >= 1, "oc: {:?}", report.opencode);
        assert!(report.opencode.imported_turns >= 2);
        assert_eq!(report.opencode.list_capped, Some(0));
        assert_eq!(report.opencode.export_errors, Some(0));
        assert_eq!(report.opencode.timed_out, Some(0));
        assert_eq!(report.opencode.skipped_missing_binary, Some(0));
        assert_eq!(report.claude.status, "ok");
        assert_eq!(report.codex.status, "ok");
        assert_eq!(report.cursor.status, "ok");
        assert_eq!(report.claude.sessions, 0);
        assert_eq!(report.codex.sessions, 0);
        assert_eq!(report.cursor.sessions, 0);
    }

    #[test]
    fn multi_import__hermetic_six_sources__all_ok_with_turns() {
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let opts = hermetic_six_fixtures(root.path());

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.v, 1);
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.grok.status, "ok");
        assert_eq!(report.opencode.status, "ok");
        assert_eq!(report.claude.status, "ok");
        assert_eq!(report.codex.status, "ok");
        assert_eq!(report.cursor.status, "ok");
        assert!(report.agy.imported_turns >= 2, "agy: {:?}", report.agy);
        assert!(report.grok.imported_turns >= 2, "grok: {:?}", report.grok);
        assert!(
            report.opencode.imported_turns >= 2,
            "oc: {:?}",
            report.opencode
        );
        assert!(
            report.claude.imported_turns >= 2,
            "claude: {:?}",
            report.claude
        );
        assert!(
            report.codex.imported_turns >= 2,
            "codex: {:?}",
            report.codex
        );
        assert!(
            report.cursor.imported_turns >= 2,
            "cursor: {:?}",
            report.cursor
        );
    }

    #[test]
    fn multi_import__malformed_opencode_list__others_still_run() {
        // AC2
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_fixtures(root.path());
        opts.opencode_list_json_override = Some("NOT-VALID-JSON{{{".to_string());

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(
            report.agy.status, "ok",
            "agy must still run: {:?}",
            report.agy
        );
        assert_eq!(
            report.grok.status, "ok",
            "grok must still run: {:?}",
            report.grok
        );
        assert_eq!(report.opencode.status, "error");
        assert!(
            report
                .opencode
                .error
                .as_deref()
                .is_some_and(|e| e.contains("opencode")),
            "error should name source: {:?}",
            report.opencode.error
        );
    }

    #[test]
    fn multi_import__global_skip__all_sources_skipped_no_writes() {
        // AC3
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_fixtures(root.path());
        opts.skip_import = true;

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.agy.status, "skipped");
        assert_eq!(report.agy.skip_reason.as_deref(), Some("skip_import"));
        assert_eq!(report.grok.status, "skipped");
        assert_eq!(report.opencode.status, "skipped");
        assert_eq!(report.claude.status, "skipped");
        assert_eq!(report.codex.status, "skipped");
        assert_eq!(report.cursor.status, "skipped");
        assert_eq!(report.claude.skip_reason.as_deref(), Some("skip_import"));
        assert_eq!(report.codex.skip_reason.as_deref(), Some("skip_import"));
        assert_eq!(report.cursor.skip_reason.as_deref(), Some("skip_import"));
        assert_eq!(report.agy.sessions, 0);
        assert_eq!(report.agy.imported_turns, 0);

        // No turns written
        let turns_agy = ctx
            .conn
            .get_session_turns("11111111-1111-1111-1111-111111111111")
            .expect("query");
        assert!(turns_agy.is_empty());
    }

    #[test]
    fn multi_import__skip_import_grok_only__agy_and_opencode_run() {
        // AC4
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_fixtures(root.path());
        opts.skip_grok = true;

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.grok.status, "skipped");
        assert_eq!(report.grok.skip_reason.as_deref(), Some("skip_import_grok"));
        assert_eq!(report.opencode.status, "ok");
        assert!(report.agy.sessions >= 1);
        assert!(report.opencode.sessions >= 1);
        assert_eq!(report.grok.sessions, 0);
    }

    #[test]
    fn multi_import__skip_import_agy_only__grok_and_opencode_run() {
        // AC4 second per-source case
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_fixtures(root.path());
        opts.skip_agy = true;

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.agy.status, "skipped");
        assert_eq!(report.agy.skip_reason.as_deref(), Some("skip_import_agy"));
        assert_eq!(report.grok.status, "ok");
        assert_eq!(report.opencode.status, "ok");
    }

    #[test]
    fn multi_import__skip_import_cursor_only__others_run() {
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_six_fixtures(root.path());
        opts.skip_cursor = true;

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.cursor.status, "skipped");
        assert_eq!(
            report.cursor.skip_reason.as_deref(),
            Some("skip_import_cursor")
        );
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.grok.status, "ok");
        assert_eq!(report.opencode.status, "ok");
        assert_eq!(report.claude.status, "ok");
        assert_eq!(report.codex.status, "ok");
        assert!(report.agy.imported_turns >= 2);
        assert!(report.claude.imported_turns >= 2);
        assert_eq!(report.cursor.sessions, 0);
    }

    #[test]
    fn multi_import__skip_import_claude_only__others_run() {
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_six_fixtures(root.path());
        opts.skip_claude = true;

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.claude.status, "skipped");
        assert_eq!(
            report.claude.skip_reason.as_deref(),
            Some("skip_import_claude")
        );
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.cursor.status, "ok");
        assert_eq!(report.codex.status, "ok");
        assert!(report.cursor.imported_turns >= 2);
        assert_eq!(report.claude.sessions, 0);
    }

    #[test]
    fn multi_import__per_source_sink__error_on_a_does_not_poison_b() {
        // AC13: OpenCode fails; AGY and Grok still ok with their own sinks
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let mut opts = hermetic_fixtures(root.path());
        opts.opencode_list_json_override = Some("[]".to_string()); // valid empty — ok
        // Force OpenCode error via bad list that parses as non-array object without sessions
        opts.opencode_list_json_override = Some(r#"{"broken":true}"#.to_string());

        let report = run_multi_harness_import(&ctx, opts);
        assert_eq!(report.agy.status, "ok");
        assert_eq!(report.grok.status, "ok");
        // Bad list shape → adapter Err → OpenCode error only (fresh sink per source).
        assert_eq!(
            report.opencode.status, "error",
            "OpenCode must fail for broken list JSON: {:?}",
            report.opencode
        );
        // Critical: AGY/Grok not flipped to error without their own failure
        assert_ne!(report.agy.status, "error");
        assert_ne!(report.grok.status, "error");
    }

    #[test]
    fn multi_import__persist_and_status__matches_report() {
        // AC6
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let opts = hermetic_fixtures(root.path());
        let report = run_multi_harness_import(&ctx, opts);
        let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        persist_multi_import_report(&store, &report);

        let view = load_multi_import_status(ctx.conn.as_ref()).expect("load");
        match view {
            MultiImportStatusView::Report(loaded) => {
                assert_eq!(loaded.v, 1);
                assert_eq!(loaded.agy.status, report.agy.status);
                assert_eq!(loaded.grok.sessions, report.grok.sessions);
                assert_eq!(
                    loaded.opencode.imported_turns,
                    report.opencode.imported_turns
                );
            }
            other => panic!("expected Report, got {other:?}"),
        }
    }

    #[test]
    fn multi_import__status_missing_key__never() {
        // AC6 missing
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let view = load_multi_import_status(ctx.conn.as_ref()).expect("load");
        assert_eq!(view, MultiImportStatusView::Never);
    }

    #[test]
    fn multi_import__status_corrupt_json__unreadable() {
        // AC11
        let root = tempdir().unwrap();
        let vault_dir = root.path().join("vault");
        fs::create_dir_all(&vault_dir).unwrap();
        let ctx = open_ctx(&vault_dir);
        let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        store
            .set_sync_state(LAST_MULTI_IMPORT_KEY, "NOT-JSON{{{")
            .expect("set");
        let view = load_multi_import_status(ctx.conn.as_ref()).expect("load");
        assert_eq!(view, MultiImportStatusView::Unreadable);
    }

    #[test]
    fn multi_import__status_list_capped__surfaces_warning_line() {
        // AC12
        let report = MultiImportReport {
            v: 1,
            at: "2026-08-09T00:00:00Z".to_string(),
            agy: SourceImportReport::skipped("skip_import"),
            grok: SourceImportReport::skipped("skip_import"),
            opencode: {
                let mut s = SourceImportReport::ok(1, 2, 0);
                s.list_capped = Some(3);
                s.export_errors = Some(0);
                s.timed_out = Some(0);
                s.skipped_missing_binary = Some(0);
                s
            },
            claude: SourceImportReport::skipped("skip_import"),
            codex: SourceImportReport::skipped("skip_import"),
            cursor: SourceImportReport::skipped("skip_import"),
        };
        let line = opencode_cap_warning_line(&report.opencode)
            .expect("list_capped>0 must produce warning");
        assert!(
            line.contains("OpenCode import capped"),
            "unexpected line: {line}"
        );
        assert!(line.contains("list_capped=3"));
        assert!(opencode_cap_warning_line(&SourceImportReport::ok(0, 0, 0)).is_none());
        let view = MultiImportStatusView::Report(Box::new(report));
        print_multi_import_status(&view);
    }

    #[test]
    fn multi_import__serde_roundtrip__stable_v1() {
        // F24
        let report = MultiImportReport::new(
            SourceImportReport::ok(1, 2, 0),
            SourceImportReport::skipped("skip_import_grok"),
            {
                let mut s = SourceImportReport::error(0, 0, 0, "opencode: boom".into());
                s.list_capped = Some(1);
                s
            },
            SourceImportReport::ok(3, 4, 0),
            SourceImportReport::ok(5, 6, 0),
            SourceImportReport::ok(7, 8, 0),
        );
        let json = serde_json::to_string(&report).expect("ser");
        let back: MultiImportReport = serde_json::from_str(&json).expect("de");
        assert_eq!(back.v, 1);
        assert_eq!(back.agy.sessions, 1);
        assert_eq!(back.grok.skip_reason.as_deref(), Some("skip_import_grok"));
        assert_eq!(back.opencode.list_capped, Some(1));
        assert_eq!(back.claude.sessions, 3);
        assert_eq!(back.codex.sessions, 5);
        assert_eq!(back.cursor.sessions, 7);
        assert!(json.contains("\"claude\""));
        assert!(json.contains("\"codex\""));
        assert!(json.contains("\"cursor\""));
    }

    #[test]
    fn multi_import__serde_v1_missing_new_keys__absent_pre_t334() {
        let old = r#"{"v":1,"at":"2026-08-16T00:00:00Z","agy":{"status":"ok","sessions":1,"imported_turns":2,"unbound":0},"grok":{"status":"ok","sessions":0,"imported_turns":0,"unbound":0},"opencode":{"status":"skipped","skip_reason":"skip_import_opencode","sessions":0,"imported_turns":0,"unbound":0}}"#;
        let back: MultiImportReport = serde_json::from_str(old).expect("de");
        assert_eq!(back.v, 1);
        assert_eq!(back.agy.sessions, 1);
        assert_eq!(back.claude.status, "skipped");
        assert_eq!(back.claude.skip_reason.as_deref(), Some("absent_pre_t334"));
        assert_eq!(back.codex.skip_reason.as_deref(), Some("absent_pre_t334"));
        assert_eq!(back.cursor.skip_reason.as_deref(), Some("absent_pre_t334"));
    }

    #[test]
    fn source_import_report__resolved_bin_keys__dual_read() {
        let old = r#"{"v":1,"at":"2026-09-01T07:00:22Z","agy":{"status":"ok","sessions":0,"imported_turns":0,"unbound":0},"grok":{"status":"ok","sessions":0,"imported_turns":0,"unbound":0},"opencode":{"status":"ok","sessions":0,"imported_turns":0,"unbound":0,"skipped_missing_binary":1},"claude":{"status":"skipped","skip_reason":"absent_pre_t334"},"codex":{"status":"skipped","skip_reason":"absent_pre_t334"},"cursor":{"status":"skipped","skip_reason":"absent_pre_t334"}}"#;
        let back: MultiImportReport = serde_json::from_str(old).expect("de");
        assert_eq!(back.v, 1);
        assert_eq!(back.opencode.skipped_missing_binary, Some(1));
        assert!(back.opencode.resolved_bin.is_none());
        assert!(back.opencode.binary_attempts.is_none());
        let mut fresh = back.clone();
        fresh.opencode.resolved_bin = None;
        fresh.opencode.binary_attempts = None;
        let value = serde_json::to_value(&fresh).expect("ser");
        assert!(value["opencode"].get("resolved_bin").is_none());
        assert!(value["opencode"].get("binary_attempts").is_none());
        assert_eq!(value["v"], 1);
    }

    #[test]
    fn append_opencode_health__unresolved__prints_attempt_paths() {
        let mut line = String::from("  opencode: ok");
        let src = SourceImportReport {
            status: "ok".to_string(),
            skip_reason: None,
            sessions: 0,
            imported_turns: 0,
            unbound: 0,
            error: None,
            list_capped: None,
            export_errors: None,
            timed_out: None,
            skipped_missing_binary: Some(1),
            resolved_bin: None,
            binary_attempts: Some(vec![
                "C:\\a\\opencode.cmd".into(),
                "C:\\b\\opencode.exe".into(),
            ]),
        };
        append_opencode_health(&mut line, "opencode", &src);
        assert!(
            line.contains("C:\\a\\opencode.cmd"),
            "human F4 must surface attempt paths: {line}"
        );
        assert!(line.contains("skipped_missing_binary=1"), "{line}");
    }
}
