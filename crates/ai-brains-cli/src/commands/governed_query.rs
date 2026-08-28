//! Thin CLI surface for progressive query / handle expand / query trace (T152-P1-06 / T202 / T221).

use crate::commands::briefing::cli_principal;
use crate::commands::governed_common::{
    EXIT_POLICY_DENIED, GovernedCliError, OutputFormat, POLICY_DENIED_HINT,
    PROGRESSIVE_RECALL_FALLBACK, UNKNOWN_HANDLE_PREVIEW, collapse_copy_paste_text, emit_human,
    emit_json, fail_cp, fail_usage, format_authorized_empty_next,
};
use crate::context::AppContext;
use ai_brains_contracts::briefings::{API_VERSION, ProgressiveQueryResponse};
use ai_brains_control_plane::{
    ExpandHandleRequest, GetQueryTraceRequest, ProgressiveQueryRequest, StorePorts, SystemClock,
    expand_handle, get_query_trace, progressive_query, scope_identity_key,
};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::{QueryStore, SqliteEventStore};
use serde::Serialize;
use std::io::IsTerminal;

/// F30 progressive usage message (copy-paste example + env).
pub const PROGRESSIVE_PROJECT_USAGE: &str = "project id required. Example:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\nOr set AI_BRAINS_PROJECT_ID.";

/// F30 expand usage message (copy-paste example + env).
pub const EXPAND_PROJECT_USAGE: &str = "project id required. Example:\n  ai-brains query expand <handle-id> --project-id <uuid>\nOr set AI_BRAINS_PROJECT_ID.";

/// T314 AC16 — human Denied second line when DTO `preview` is empty (JSON stays empty; F17).
const DENIED_HANDLE_HUMAN_PREVIEW: &str = "Access denied.";

/// Copy-paste command that persists a query trace (T152 `--dry-run` default is true).
const TRACE_PROGRESSIVE_PERSIST: &str =
    "ai-brains query progressive \"what did we decide\" --dry-run false";

/// T291 F8 — copy-paste remediator for missing/unauthorized `query trace`.
pub const TRACE_MISSING_NEXT_STEP: &str =
    "No persisted trace. Run: ai-brains query progressive \"what did we decide\" --dry-run false";

/// CLI-local missing/unauthorized envelope (T291 F7). Not a `QueryTraceDto`.
#[derive(Debug, Serialize)]
struct MissingTraceEnvelope {
    api_version: String,
    found: bool,
    trace_id: String,
    next_step: String,
}

pub struct ProgressiveQueryOptions {
    pub query: String,
    pub project_id: Option<ProjectId>,
    pub limit: usize,
    pub dry_run: bool,
}

pub struct ExpandHandleOptions {
    pub handle_id: String,
    pub project_id: Option<ProjectId>,
    pub max_chars: usize,
    /// T314 F7 — Trace token set; default `"json"`.
    pub format: String,
}

pub struct TraceOptions {
    pub trace_id: String,
    pub format: String,
}

/// Fill `preview` when expand JSON is `kind=Unknown` with an empty preview (T263 F7).
pub(crate) fn apply_unknown_expand_preview(value: &mut serde_json::Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    if obj.get("kind").and_then(|k| k.as_str()) != Some("Unknown") {
        return;
    }
    let empty = obj
        .get("preview")
        .and_then(|p| p.as_str())
        .is_none_or(|s| s.is_empty());
    if empty {
        obj.insert(
            "preview".to_string(),
            serde_json::Value::String(UNKNOWN_HANDLE_PREVIEW.to_string()),
        );
    }
}

/// Progressive deny/empty honesty (T243 F33 / T290 F33). Mutate before `emit_json`.
pub(crate) fn apply_progressive_search_hints(
    resp: &mut ProgressiveQueryResponse,
    pin_count: Option<u64>,
    query: &str,
) {
    if resp.denied {
        if resp
            .denial_hint
            .as_deref()
            .is_none_or(|h| !h.contains("recall"))
        {
            let base = resp
                .denial_hint
                .clone()
                .unwrap_or_else(|| POLICY_DENIED_HINT.to_string());
            resp.denial_hint = Some(format!("{base} {PROGRESSIVE_RECALL_FALLBACK}"));
        }
    } else if resp.results.is_empty() {
        resp.next_step = Some(format_authorized_empty_next(pin_count, Some(query)));
    }
}

/// `ai-brains query progressive "<text>"` — governed progressive query (JSON stdout).
pub fn run_progressive(
    ctx: &AppContext,
    options: ProgressiveQueryOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(project_id) = options.project_id else {
        return fail_usage(PROGRESSIVE_PROJECT_USAGE);
    };
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let clock = SystemClock;
    let policy = ports.production_policy();
    let principal = cli_principal();
    let scope = ScopeRef::Repository(project_id);
    let event_store = ports.store();

    let writer = if options.dry_run {
        None
    } else {
        Some(&ports.writer)
    };
    let query = options.query.clone();
    let pin_count = ctx.conn.count_pinned_memories(Some(&project_id)).ok();
    // F33: map CP errors (incl. PolicyDenied) via fail_cp → exit 3, never raw `?` → exit 1.
    let mut resp = match progressive_query(
        writer,
        &ports.query,
        &event_store,
        &clock,
        &policy,
        ProgressiveQueryRequest {
            principal,
            scope,
            query: options.query,
            privacy: Privacy::LocalOnly,
            limit: options.limit,
            dry_run: options.dry_run,
            at: None,
        },
    ) {
        Ok(r) => r,
        Err(e) => return fail_cp(OutputFormat::Json, e),
    };
    // F17: ensure in-band bootstrap for stdout-only agents if CP left it empty.
    if resp.denied && resp.denial_hint.is_none() {
        resp.denial_hint = Some(POLICY_DENIED_HINT.to_string());
    }
    apply_progressive_search_hints(&mut resp, pin_count, &query);
    // F2/F3: keep ProgressiveQueryResponse on stdout, then exit 3 on deny (F1/F34).
    emit_json(&resp)?;
    if resp.denied {
        // F4: CODE line then POLICY_DENIED_HINT on stderr; F4b extra recall fallback after HINT.
        eprintln!("POLICY_DENIED: progressive query denied");
        eprintln!("{POLICY_DENIED_HINT}");
        eprintln!("{PROGRESSIVE_RECALL_FALLBACK}");
        return Err(Box::new(GovernedCliError::emitted(
            EXIT_POLICY_DENIED,
            "POLICY_DENIED: progressive query denied",
        )));
    }
    Ok(())
}

/// `ai-brains query expand <handle-id>` — bounded handle preview (JSON default; human kind+preview).
pub fn run_expand(
    ctx: &AppContext,
    options: ExpandHandleOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(project_id) = options.project_id else {
        return fail_usage(EXPAND_PROJECT_USAGE);
    };
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let principal = cli_principal();
    let scope = ScopeRef::Repository(project_id);
    let event_store = ports.store();

    // F33: map CP errors via fail_cp (not raw `?`).
    let preview = match expand_handle(
        &ports.query,
        &event_store,
        &policy,
        ExpandHandleRequest {
            principal,
            scope: scope.clone(),
            handle_id: options.handle_id,
            privacy: Privacy::LocalOnly,
            max_chars: options.max_chars,
        },
    ) {
        Ok(p) => p,
        Err(e) => return fail_cp(OutputFormat::Json, e),
    };
    // Include applied scope key for operators debugging cross-scope denials.
    let mut value = serde_json::to_value(&preview)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "applied_scope".into(),
            serde_json::Value::String(scope_identity_key(&scope)),
        );
    }
    apply_unknown_expand_preview(&mut value);
    // T314 F9 / AC16 — human tokens: kind then preview (two nonempty lines for Unknown/Denied).
    if query_format_is_human(&options.format) {
        let kind = value.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let mut preview_text = value
            .get("preview")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        // JSON Denied keeps empty preview (stay-green); human fills so AC16 is nonempty.
        if preview_text.is_empty() && kind == "Denied" {
            preview_text = DENIED_HANDLE_HUMAN_PREVIEW.to_string();
        }
        if preview_text.is_empty() && kind == "Unknown" {
            preview_text = UNKNOWN_HANDLE_PREVIEW.to_string();
        }
        emit_human(kind);
        emit_human(&preview_text);
    } else {
        emit_json(&value)?;
    }
    // F6/F30: exact kind "Denied" → exit 3 + F4 stderr; Unknown/found stay exit 0.
    if preview.kind == "Denied" {
        eprintln!("POLICY_DENIED: expand handle denied");
        eprintln!("{POLICY_DENIED_HINT}");
        return Err(Box::new(GovernedCliError::emitted(
            EXIT_POLICY_DENIED,
            "POLICY_DENIED: expand handle denied",
        )));
    }
    Ok(())
}

/// Displayed `trace_id` (T291 F15): shared collapse; empty → `<empty>`.
fn sanitize_trace_id(raw: &str) -> String {
    let collapsed = collapse_copy_paste_text(raw);
    if collapsed.is_empty() {
        "<empty>".to_string()
    } else {
        collapsed
    }
}

fn missing_trace_envelope(trace_id: &str) -> MissingTraceEnvelope {
    MissingTraceEnvelope {
        api_version: API_VERSION.to_string(),
        found: false,
        trace_id: sanitize_trace_id(trace_id),
        next_step: TRACE_MISSING_NEXT_STEP.to_string(),
    }
}

/// T314 F32 / T291 F3 — Trace + Expand `--format` human tokens (after clap value_parser).
pub(crate) fn query_format_is_human(format: &str) -> bool {
    match format {
        "human" | "pretty" | "text" | "markdown" | "md" => true,
        "auto" => std::io::stdout().is_terminal(),
        _ => false,
    }
}

/// `--format` after clap `value_parser` (T291 F3). Found path ignores this.
fn missing_trace_is_human(format: &str) -> bool {
    query_format_is_human(format)
}

/// `ai-brains query trace <trace-id>` — fetch a governed query trace.
///
/// T202 F31: project id is not required. T291: missing/unauthorized is a
/// missing-only envelope (or two human lines), exit 0 — not the token `null`.
pub fn run_trace(
    ctx: &AppContext,
    options: TraceOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let principal = cli_principal();
    let event_store = ports.store();

    let requested = options.trace_id.clone();
    let trace = get_query_trace(
        &event_store,
        &policy,
        GetQueryTraceRequest {
            principal,
            privacy: Privacy::LocalOnly,
            trace_id: options.trace_id,
        },
    )?;
    match trace {
        Some(t) => crate::commands::identity_warn::print_json_stdout(&t)?,
        None => {
            if missing_trace_is_human(&options.format) {
                let id = sanitize_trace_id(&requested);
                emit_human(&format!("No trace for {id}."));
                emit_human(&format!("next: {TRACE_PROGRESSIVE_PERSIST}"));
            } else {
                emit_json(&missing_trace_envelope(&requested))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn trace_missing_next_step__frozen__exact_string() {
        assert_eq!(
            TRACE_MISSING_NEXT_STEP,
            "No persisted trace. Run: ai-brains query progressive \"what did we decide\" --dry-run false"
        );
        assert!(
            !TRACE_MISSING_NEXT_STEP.contains('\n') && !TRACE_MISSING_NEXT_STEP.contains('…'),
            "F8 must be one line without U+2026; got {TRACE_MISSING_NEXT_STEP}"
        );
        assert!(
            TRACE_MISSING_NEXT_STEP.contains("query progressive")
                && TRACE_MISSING_NEXT_STEP.contains("--dry-run false"),
            "F8 must name progressive persist; got {TRACE_MISSING_NEXT_STEP}"
        );
        assert!(
            !TRACE_MISSING_NEXT_STEP.contains("--trace"),
            "F8 must not invent --trace; got {TRACE_MISSING_NEXT_STEP}"
        );
        assert!(
            TRACE_MISSING_NEXT_STEP.ends_with(TRACE_PROGRESSIVE_PERSIST),
            "F8 must share TRACE_PROGRESSIVE_PERSIST with human next; got {TRACE_MISSING_NEXT_STEP}"
        );
    }

    #[test]
    fn sanitize_trace_id__newline_dollar_quotes__single_line_safe() {
        let got = sanitize_trace_id("id\nwith $ and `tick` and \"q\"");
        assert!(
            !got.contains('\n') && !got.contains('$') && !got.contains('`') && !got.contains('"'),
            "displayed id must be single-line without interpolators/quotes; got {got:?}"
        );
        assert_eq!(sanitize_trace_id("   $ $  "), "<empty>");
        assert_eq!(sanitize_trace_id(""), "<empty>");
    }

    #[test]
    fn query_trace_dto__serialized__has_no_found_or_next_step() {
        use ai_brains_contracts::briefings::QueryTraceDto;
        let json = serde_json::json!({
            "api_version": "1",
            "query_trace_id": "t",
            "scope": "s",
            "principal": "p",
            "query": "q",
            "applied_policy": "pol",
        });
        let dto: QueryTraceDto = serde_json::from_value(json).expect("QueryTraceDto");
        let ser = serde_json::to_value(&dto).expect("serialize");
        assert!(
            ser.get("found").is_none() && ser.get("next_step").is_none(),
            "QueryTraceDto must not grow found/next_step; got {ser}"
        );
        assert_eq!(ser["query_trace_id"], "t");
    }

    #[test]
    fn progressive_usage_message__includes_example_and_env() {
        assert!(PROGRESSIVE_PROJECT_USAGE.contains("query progressive"));
        assert!(PROGRESSIVE_PROJECT_USAGE.contains("--project-id"));
        assert!(PROGRESSIVE_PROJECT_USAGE.contains("AI_BRAINS_PROJECT_ID"));
    }

    #[test]
    fn expand_usage_message__includes_example_and_env() {
        assert!(EXPAND_PROJECT_USAGE.contains("query expand"));
        assert!(EXPAND_PROJECT_USAGE.contains("--project-id"));
        assert!(EXPAND_PROJECT_USAGE.contains("AI_BRAINS_PROJECT_ID"));
    }

    fn empty_progressive_resp() -> ProgressiveQueryResponse {
        ProgressiveQueryResponse::new(Vec::new(), "scope", "policy", "trace", false)
    }

    #[test]
    fn apply_progressive_search_hints__denied__next_step_none_hint_contains_recall() {
        let mut resp = empty_progressive_resp();
        resp.denied = true;
        resp.denial_hint = Some(POLICY_DENIED_HINT.to_string());
        apply_progressive_search_hints(&mut resp, Some(12), "x");
        assert!(
            resp.next_step.is_none(),
            "denied must omit next_step; got {:?}",
            resp.next_step
        );
        let hint = resp.denial_hint.as_deref().unwrap_or("");
        assert!(
            hint.contains("recall"),
            "denial_hint must contain recall; got {hint}"
        );
        assert!(
            hint.contains("policy bootstrap") || hint.contains("bootstrap"),
            "denial_hint must contain bootstrap; got {hint}"
        );
    }

    #[test]
    fn apply_progressive_search_hints__authorized_empty__next_step_contains_recall() {
        let mut resp = empty_progressive_resp();
        apply_progressive_search_hints(&mut resp, Some(0), "what did we decide about SQLCipher");
        let step = resp.next_step.as_deref().unwrap_or("");
        assert!(
            resp.next_step.is_some(),
            "authorized empty must set next_step"
        );
        assert!(
            step.contains("recall"),
            "next_step must contain recall; got {step}"
        );
        assert!(
            step.contains("SQLCipher") && step.contains("(Pinned: 0)") && !step.contains('…'),
            "authorized empty must copy-paste the operator query + Pinned; got {step}"
        );
        assert!(
            resp.denial_hint.is_none(),
            "authorized empty must omit denial_hint; got {:?}",
            resp.denial_hint
        );
    }

    #[test]
    fn expand_unknown__preview_nonempty() {
        // T263 AC5 / F7
        let mut value = serde_json::json!({
            "kind": "Unknown",
            "preview": "",
            "handle_id": "00000000-0000-0000-0000-000000000000",
        });
        apply_unknown_expand_preview(&mut value);
        let preview = value["preview"].as_str().unwrap_or("");
        assert!(
            !preview.is_empty(),
            "Unknown preview must be a non-empty SOOT; got {value}"
        );
        assert_eq!(value["kind"], "Unknown");
    }

    #[test]
    fn apply_progressive_search_hints__authorized_nonempty__omits_next_step() {
        use ai_brains_contracts::briefings::{ProgressiveQueryHitDto, RankingComponentsDto};
        let mut resp = ProgressiveQueryResponse::new(
            vec![ProgressiveQueryHitDto {
                id: "hit-1".into(),
                kind: "Conclusion".into(),
                statement: "dummy".into(),
                state: "Active".into(),
                evidence_handles: Vec::new(),
                source_versions: Vec::new(),
                freshness: "Fresh".into(),
                conflict_status: None,
                ranking: RankingComponentsDto {
                    authority: 80,
                    valid_time: 50,
                    relevance: None,
                },
            }],
            "scope",
            "policy",
            "trace",
            false,
        );
        apply_progressive_search_hints(&mut resp, Some(12), "SQLCipher");
        assert!(
            resp.next_step.is_none(),
            "authorized hits must omit next_step; got {:?}",
            resp.next_step
        );
        assert!(
            resp.denial_hint.is_none(),
            "authorized hits must omit denial_hint; got {:?}",
            resp.denial_hint
        );
    }
}
