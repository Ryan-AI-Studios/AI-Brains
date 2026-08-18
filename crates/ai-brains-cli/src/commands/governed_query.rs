//! Thin CLI surface for progressive query / handle expand / query trace (T152-P1-06 / T202 / T221).

use crate::commands::briefing::cli_principal;
use crate::commands::governed_common::{
    EXIT_POLICY_DENIED, GovernedCliError, OutputFormat, POLICY_DENIED_HINT,
    PROGRESSIVE_RECALL_FALLBACK, UNKNOWN_HANDLE_PREVIEW, emit_json, fail_cp, fail_usage,
};
use crate::context::AppContext;
use ai_brains_contracts::briefings::ProgressiveQueryResponse;
use ai_brains_control_plane::{
    ExpandHandleRequest, GetQueryTraceRequest, ProgressiveQueryRequest, StorePorts, SystemClock,
    expand_handle, get_query_trace, progressive_query, scope_identity_key,
};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_store::SqliteEventStore;

/// F30 progressive usage message (copy-paste example + env).
pub const PROGRESSIVE_PROJECT_USAGE: &str = "project id required. Example:\n  ai-brains query progressive \"why was graph backend replaced?\" --project-id <uuid>\nOr set AI_BRAINS_PROJECT_ID.";

/// F30 expand usage message (copy-paste example + env).
pub const EXPAND_PROJECT_USAGE: &str = "project id required. Example:\n  ai-brains query expand <handle-id> --project-id <uuid>\nOr set AI_BRAINS_PROJECT_ID.";

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
}

pub struct TraceOptions {
    pub trace_id: String,
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

/// Progressive deny/empty honesty (T243 F33). Mutate before `emit_json`.
pub(crate) fn apply_progressive_search_hints(resp: &mut ProgressiveQueryResponse) {
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
        resp.next_step = Some(PROGRESSIVE_RECALL_FALLBACK.to_string());
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
    apply_progressive_search_hints(&mut resp);
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

/// `ai-brains query expand <handle-id>` — bounded handle preview (JSON stdout).
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
    emit_json(&value)?;
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

/// `ai-brains query trace <trace-id>` — fetch a governed query trace (JSON stdout).
///
/// F31: project id is not required; missing traces print `null` and exit 0.
pub fn run_trace(
    ctx: &AppContext,
    options: TraceOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let policy = ports.production_policy();
    let principal = cli_principal();
    let event_store = ports.store();

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
            // Empty-state contract: missing or unauthorized → null JSON.
            crate::commands::identity_warn::note_machine_stdout();
            println!("null");
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

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
        apply_progressive_search_hints(&mut resp);
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
        apply_progressive_search_hints(&mut resp);
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
        apply_progressive_search_hints(&mut resp);
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
