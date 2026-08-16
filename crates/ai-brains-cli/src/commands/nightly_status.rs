//! T255 nightly `--status` JSON builder, format resolver, and Router display helpers.

use crate::commands::multi_import::{MultiImportStatusView, SourceImportReport};
use crate::commands::nightly::{explain_last_task_result, host_port_from_url};
use serde::Serialize;

/// Local copy of the T248/T249 format resolver (T249 pin: do not extract a shared helper).
///
/// pretty/human/text/markdown/md → human; json → json; auto + TTY → human;
/// auto + pipe → json; unknown → json (fail-closed).
pub(crate) fn resolve_nightly_status_format(explicit: &str, is_tty: bool) -> &'static str {
    match explicit {
        "pretty" | "human" | "text" | "markdown" | "md" => "human",
        "json" => "json",
        "auto" if is_tty => "human",
        "auto" => "json",
        _ => "json",
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EndpointJson {
    pub host_port: String,
    pub model: String,
    pub probe: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RouterJson {
    pub scheduled: bool,
    pub status: Option<String>,
    pub last_result: Option<String>,
    pub last_result_hint: Option<String>,
    pub task_to_run: Option<String>,
}

/// never/unreadable: only `{ "status": "never"|"unreadable" }`.
/// ok: `{ "status": "ok", "at", "agy", "grok", "opencode" }` using existing [`SourceImportReport`].
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MultiImportJson {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agy: Option<SourceImportReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grok: Option<SourceImportReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<SourceImportReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NightlyStatusJson {
    pub schema_version: u32,
    pub scheduled: Option<bool>,
    pub next_run: Option<String>,
    pub last_task_result: Option<String>,
    pub last_task_result_hint: Option<String>,
    pub last_scheduled_run: Option<String>,
    pub action_target: Option<String>,
    pub action_target_missing: bool,
    pub next_step: Option<String>,
    pub last_nightly_run: Option<String>,
    pub unsummarized_sessions: usize,
    pub sessions_summarized_last_run: Option<usize>,
    pub errors_last_run: Option<serde_json::Value>,
    pub errors_last_run_unreadable: bool,
    pub completion: EndpointJson,
    pub embedding: EndpointJson,
    pub multi_import: MultiImportJson,
    pub router: Option<RouterJson>,
}

pub(crate) struct NightlyStatusInput {
    pub scheduled: Option<bool>,
    pub next_run: Option<String>,
    pub last_task_result: Option<String>,
    pub last_scheduled_run: Option<String>,
    pub action_target: Option<String>,
    pub action_target_missing: bool,
    pub next_step: Option<String>,
    pub last_nightly_run: Option<String>,
    pub unsummarized_sessions: usize,
    pub last_count_raw: Option<String>,
    pub last_errors_raw: Option<String>,
    pub completion_url: String,
    pub completion_model: String,
    pub completion_probe: String,
    pub embedding_url: String,
    pub embedding_model: String,
    pub embedding_probe: String,
    pub multi_import: MultiImportStatusView,
    pub router: Option<RouterStatusInput>,
}

pub(crate) struct RouterStatusInput {
    pub found: bool,
    pub status: Option<String>,
    pub last_result: Option<String>,
    pub task_to_run: Option<String>,
}

pub(crate) fn build_nightly_status_json(input: NightlyStatusInput) -> NightlyStatusJson {
    let last_task_result_hint = input
        .last_task_result
        .as_deref()
        .and_then(explain_last_task_result)
        .map(str::to_string);
    let (errors_last_run, errors_last_run_unreadable) =
        parse_errors_last_run(input.last_errors_raw.as_deref());
    NightlyStatusJson {
        schema_version: 1,
        scheduled: input.scheduled,
        next_run: input.next_run,
        last_task_result: input.last_task_result,
        last_task_result_hint,
        last_scheduled_run: input.last_scheduled_run,
        action_target: input.action_target,
        action_target_missing: input.action_target_missing,
        next_step: input.next_step,
        last_nightly_run: input.last_nightly_run,
        unsummarized_sessions: input.unsummarized_sessions,
        sessions_summarized_last_run: parse_sessions_summarized_last_run(
            input.last_count_raw.as_deref(),
        ),
        errors_last_run,
        errors_last_run_unreadable,
        completion: endpoint_json(
            &input.completion_url,
            input.completion_model,
            input.completion_probe,
        ),
        embedding: endpoint_json(
            &input.embedding_url,
            input.embedding_model,
            input.embedding_probe,
        ),
        multi_import: multi_import_json(input.multi_import),
        router: input.router.as_ref().map(router_json_from_input),
    }
}

pub(crate) fn emit_nightly_status_json(
    status: &NightlyStatusJson,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(status)
}

/// Human Router lines (T255 F7/F10/AC6/AC7/AC15). Hint is a following line.
///
/// Do not apply Nightly first-quoted missing-action / `next:` to Router.
pub(crate) fn format_router_status_lines(
    found: bool,
    status: Option<&str>,
    last_result: Option<&str>,
) -> Vec<String> {
    if !found {
        return vec!["Router: not scheduled".to_string()];
    }
    let status = status.map(str::trim).filter(|s| !s.is_empty());
    let last_result = last_result.map(str::trim).filter(|s| !s.is_empty());
    let mut lines = Vec::new();
    match (status, last_result) {
        (Some(st), Some(code)) => {
            // Two spaces before `last result:` (T255 §5.2 / AC6).
            lines.push(format!("Router: {st}  last result: {code}"));
            if let Some(hint) = explain_last_task_result(code) {
                lines.push(hint.to_string());
            }
        }
        (None, Some(code)) => {
            lines.push(format!("Router: last result: {code}"));
            if let Some(hint) = explain_last_task_result(code) {
                lines.push(hint.to_string());
            }
        }
        (Some(st), None) => lines.push(format!("Router: {st}")),
        (None, None) => lines.push("Router:".to_string()),
    }
    lines
}

/// Router JSON `scheduled` is `found` (ONLOGON has no `next_run`).
pub(crate) fn router_json_from_input(input: &RouterStatusInput) -> RouterJson {
    RouterJson {
        scheduled: input.found,
        status: input.status.clone(),
        last_result: input.last_result.clone(),
        last_result_hint: input
            .last_result
            .as_deref()
            .and_then(explain_last_task_result)
            .map(str::to_string),
        task_to_run: input.task_to_run.clone(),
    }
}

fn endpoint_json(url: &str, model: String, probe: String) -> EndpointJson {
    EndpointJson {
        host_port: host_port_from_url(url),
        model,
        probe,
    }
}

fn multi_import_json(view: MultiImportStatusView) -> MultiImportJson {
    match view {
        MultiImportStatusView::Never => MultiImportJson {
            status: "never".to_string(),
            at: None,
            agy: None,
            grok: None,
            opencode: None,
        },
        MultiImportStatusView::Unreadable => MultiImportJson {
            status: "unreadable".to_string(),
            at: None,
            agy: None,
            grok: None,
            opencode: None,
        },
        MultiImportStatusView::Report(report) => MultiImportJson {
            status: "ok".to_string(),
            at: Some(report.at),
            agy: Some(report.agy),
            grok: Some(report.grok),
            opencode: Some(report.opencode),
        },
    }
}

/// F35: missing / non-`usize` → `None`. Stored `"0"` is `Some(0)`.
fn parse_sessions_summarized_last_run(raw: Option<&str>) -> Option<usize> {
    raw?.trim().parse().ok()
}

/// F20: missing → `(null, false)`; JSON array → `(array, false)`; else raw string + `true`.
fn parse_errors_last_run(raw: Option<&str>) -> (Option<serde_json::Value>, bool) {
    match raw {
        None => (None, false),
        Some(s) => match serde_json::from_str::<serde_json::Value>(s) {
            Ok(value) if value.is_array() => (Some(value), false),
            Ok(_) | Err(_) => (Some(serde_json::Value::String(s.to_string())), true),
        },
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::commands::multi_import::MultiImportReport;

    const FROZEN_KEYS: &[&str] = &[
        "schema_version",
        "scheduled",
        "next_run",
        "last_task_result",
        "last_task_result_hint",
        "last_scheduled_run",
        "action_target",
        "action_target_missing",
        "next_step",
        "last_nightly_run",
        "unsummarized_sessions",
        "sessions_summarized_last_run",
        "errors_last_run",
        "errors_last_run_unreadable",
        "completion",
        "embedding",
        "multi_import",
        "router",
    ];

    fn fixture_input() -> NightlyStatusInput {
        NightlyStatusInput {
            scheduled: Some(true),
            next_run: Some("8/16/2026 3:00:00 AM".to_string()),
            last_task_result: Some("1".to_string()),
            last_scheduled_run: Some("8/15/2026 3:00:01 AM".to_string()),
            action_target: Some(r"C:\Users\RyanB\.ai-brains\nightly-run.cmd".to_string()),
            action_target_missing: true,
            next_step: Some("ai-brains nightly --schedule --dry-run".to_string()),
            last_nightly_run: Some("2026-08-02T07:03:58.159733500+00:00".to_string()),
            unsummarized_sessions: 0,
            last_count_raw: Some("0".to_string()),
            last_errors_raw: Some("[]".to_string()),
            completion_url: "http://127.0.0.1:8081".to_string(),
            completion_model: "gemma-4-E4B-it-Q6_K.gguf".to_string(),
            completion_probe: "skipped".to_string(),
            embedding_url: "http://127.0.0.1:8083".to_string(),
            embedding_model: "nomic-embed-text-v1.5".to_string(),
            embedding_probe: "ok".to_string(),
            multi_import: MultiImportStatusView::Never,
            router: Some(RouterStatusInput {
                found: true,
                status: Some("Running".to_string()),
                last_result: Some("267009".to_string()),
                task_to_run: Some(r"C:\llm\router.bat".to_string()),
            }),
        }
    }

    fn to_value(status: &NightlyStatusJson) -> serde_json::Value {
        match serde_json::to_value(status) {
            Ok(v) => v,
            Err(e) => panic!("serialize NightlyStatusJson: {e}"),
        }
    }

    #[test]
    fn resolve_nightly_status_format__auto_tty__human() {
        assert_eq!(resolve_nightly_status_format("auto", true), "human");
    }

    #[test]
    fn resolve_nightly_status_format__auto_pipe__json() {
        assert_eq!(resolve_nightly_status_format("auto", false), "json");
    }

    #[test]
    fn resolve_nightly_status_format__pretty_human_text_markdown_md__human() {
        assert_eq!(resolve_nightly_status_format("pretty", true), "human");
        assert_eq!(resolve_nightly_status_format("pretty", false), "human");
        assert_eq!(resolve_nightly_status_format("human", true), "human");
        assert_eq!(resolve_nightly_status_format("human", false), "human");
        assert_eq!(resolve_nightly_status_format("text", true), "human");
        assert_eq!(resolve_nightly_status_format("text", false), "human");
        assert_eq!(resolve_nightly_status_format("markdown", true), "human");
        assert_eq!(resolve_nightly_status_format("markdown", false), "human");
        assert_eq!(resolve_nightly_status_format("md", true), "human");
        assert_eq!(resolve_nightly_status_format("md", false), "human");
    }

    #[test]
    fn resolve_nightly_status_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_nightly_status_format("json", true), "json");
        assert_eq!(resolve_nightly_status_format("json", false), "json");
    }

    #[test]
    fn resolve_nightly_status_format__unknown__fail_closed_json() {
        assert_eq!(resolve_nightly_status_format("xml", true), "json");
        assert_eq!(resolve_nightly_status_format("JSON", false), "json");
        assert_eq!(resolve_nightly_status_format("Pretty", true), "json");
    }

    #[test]
    fn build_nightly_status_json__fixture__contains_every_frozen_key() {
        let status = build_nightly_status_json(fixture_input());
        assert_eq!(status.schema_version, 1);
        let value = to_value(&status);
        let Some(obj) = value.as_object() else {
            panic!("NightlyStatusJson must serialize as an object");
        };
        for key in FROZEN_KEYS {
            assert!(obj.contains_key(*key), "missing frozen key {key}");
        }
        assert_eq!(obj["schema_version"], 1);
        let Some(router) = obj["router"].as_object() else {
            panic!("scheduled fixture must include router object");
        };
        assert!(
            router.contains_key("task_to_run"),
            "router.task_to_run required when scheduled"
        );
        assert_eq!(router["scheduled"], true);
        assert_eq!(router["task_to_run"], r"C:\llm\router.bat");
    }

    #[test]
    fn build_nightly_status_json__quick__probe_skipped() {
        let mut input = fixture_input();
        input.completion_probe = "skipped".to_string();
        input.embedding_probe = "skipped".to_string();
        let status = build_nightly_status_json(input);
        assert_eq!(status.completion.probe, "skipped");
        assert_eq!(status.embedding.probe, "skipped");
        let value = to_value(&status);
        let Some(completion) = value["completion"].as_object() else {
            panic!("completion object");
        };
        let Some(embedding) = value["embedding"].as_object() else {
            panic!("embedding object");
        };
        assert!(completion.contains_key("host_port"));
        assert!(completion.contains_key("model"));
        assert!(completion.contains_key("probe"));
        assert!(embedding.contains_key("host_port"));
        assert!(embedding.contains_key("model"));
        assert!(embedding.contains_key("probe"));
        assert_eq!(completion["probe"], "skipped");
        assert_eq!(embedding["probe"], "skipped");
        assert_eq!(completion["host_port"], "127.0.0.1:8081");
        assert_eq!(embedding["host_port"], "127.0.0.1:8083");
    }

    #[test]
    fn format_router_status_lines__running_267009__router_and_hint_on_following_line() {
        let lines = format_router_status_lines(true, Some("Running"), Some("267009"));
        assert_eq!(
            lines,
            vec![
                "Router: Running  last result: 267009".to_string(),
                "task still running (SCHED_S_TASK_RUNNING)".to_string(),
            ]
        );
        let json = router_json_from_input(&RouterStatusInput {
            found: true,
            status: Some("Running".to_string()),
            last_result: Some("267009".to_string()),
            task_to_run: Some(r"C:\llm\router.bat".to_string()),
        });
        assert!(json.scheduled);
        assert_eq!(json.last_result.as_deref(), Some("267009"));
        assert_eq!(
            json.last_result_hint.as_deref(),
            Some("task still running (SCHED_S_TASK_RUNNING)")
        );
    }

    #[test]
    fn format_router_status_lines__not_found__not_scheduled_no_next() {
        let lines = format_router_status_lines(false, None, None);
        assert_eq!(lines, vec!["Router: not scheduled".to_string()]);
        assert!(
            lines.iter().all(|line| !line.contains("next:")),
            "Router missing must not suggest nightly --schedule"
        );
        let json = router_json_from_input(&RouterStatusInput {
            found: false,
            status: None,
            last_result: None,
            task_to_run: None,
        });
        assert!(!json.scheduled);
    }

    #[test]
    fn router_json_from_input__found_no_next_run__scheduled_true() {
        // ONLOGON: found=true, no next_run field, last_result present → still scheduled.
        let json = router_json_from_input(&RouterStatusInput {
            found: true,
            status: Some("Ready".to_string()),
            last_result: Some("0".to_string()),
            task_to_run: Some(r"C:\llm\router.bat".to_string()),
        });
        assert!(json.scheduled);
        assert_eq!(json.last_result.as_deref(), Some("0"));
    }

    #[test]
    fn format_router_status_lines__blank_status_267009__last_result_only() {
        let lines = format_router_status_lines(true, None, Some("267009"));
        assert_eq!(
            lines.first().map(String::as_str),
            Some("Router: last result: 267009")
        );
        assert_eq!(
            lines.get(1).map(String::as_str),
            Some("task still running (SCHED_S_TASK_RUNNING)")
        );
        assert!(
            !lines.first().is_some_and(|line| line.contains("Running")),
            "blank status must not invent Running"
        );
    }

    #[test]
    fn build_nightly_status_json__no_scheduler__scheduled_and_router_null() {
        let mut input = fixture_input();
        input.scheduled = None;
        input.router = None;
        let status = build_nightly_status_json(input);
        assert!(status.scheduled.is_none());
        assert!(status.router.is_none());
        let raw = match emit_nightly_status_json(&status) {
            Ok(s) => s,
            Err(e) => panic!("pretty json: {e}"),
        };
        let value: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => panic!("parse pretty json: {e}"),
        };
        assert_eq!(value["scheduled"], serde_json::Value::Null);
        assert_eq!(value["router"], serde_json::Value::Null);
        assert_ne!(value["scheduled"], serde_json::json!(false));
        assert_ne!(value["router"], serde_json::json!(false));
    }

    #[test]
    fn build_nightly_status_json__missing_sync_state__count_and_errors_null() {
        let mut input = fixture_input();
        input.last_count_raw = None;
        input.last_errors_raw = None;
        let status = build_nightly_status_json(input);
        assert!(status.sessions_summarized_last_run.is_none());
        assert!(status.errors_last_run.is_none());
        assert!(!status.errors_last_run_unreadable);
        let value = to_value(&status);
        assert_eq!(
            value["sessions_summarized_last_run"],
            serde_json::Value::Null
        );
        assert_eq!(value["errors_last_run"], serde_json::Value::Null);
        assert_eq!(value["errors_last_run_unreadable"], false);
    }

    #[test]
    fn build_nightly_status_json__corrupt_errors__raw_string_and_unreadable_true() {
        let mut input = fixture_input();
        input.last_errors_raw = Some("not-an-array".to_string());
        let status = build_nightly_status_json(input);
        assert_eq!(
            status.errors_last_run,
            Some(serde_json::Value::String("not-an-array".to_string()))
        );
        assert!(status.errors_last_run_unreadable);
        let value = to_value(&status);
        assert_eq!(value["errors_last_run"], "not-an-array");
        assert_eq!(value["errors_last_run_unreadable"], true);
    }

    #[test]
    fn build_nightly_status_json__stored_zero_count__some_zero() {
        let mut input = fixture_input();
        input.last_count_raw = Some("0".to_string());
        let status = build_nightly_status_json(input);
        assert_eq!(status.sessions_summarized_last_run, Some(0));
        let value = to_value(&status);
        assert_eq!(value["sessions_summarized_last_run"], 0);
    }

    fn source_ok() -> SourceImportReport {
        SourceImportReport {
            status: "ok".to_string(),
            skip_reason: None,
            sessions: 1,
            imported_turns: 2,
            unbound: 0,
            error: None,
            list_capped: None,
            export_errors: None,
            timed_out: None,
            skipped_missing_binary: None,
        }
    }

    #[test]
    fn build_nightly_status_json__multi_import_unreadable__status_only() {
        let mut input = fixture_input();
        input.multi_import = MultiImportStatusView::Unreadable;
        let value = to_value(&build_nightly_status_json(input));
        let Some(mi) = value["multi_import"].as_object() else {
            panic!("multi_import object");
        };
        assert_eq!(mi["status"], "unreadable");
        assert!(!mi.contains_key("at"));
        assert!(!mi.contains_key("agy"));
        assert!(!mi.contains_key("grok"));
        assert!(!mi.contains_key("opencode"));
    }

    #[test]
    fn build_nightly_status_json__multi_import_ok__report_fields() {
        let mut input = fixture_input();
        input.multi_import = MultiImportStatusView::Report(Box::new(MultiImportReport {
            v: 1,
            at: "2026-08-16T00:00:00Z".to_string(),
            agy: source_ok(),
            grok: source_ok(),
            opencode: source_ok(),
        }));
        let value = to_value(&build_nightly_status_json(input));
        let Some(mi) = value["multi_import"].as_object() else {
            panic!("multi_import object");
        };
        assert_eq!(mi["status"], "ok");
        assert_eq!(mi["at"], "2026-08-16T00:00:00Z");
        assert!(mi.contains_key("agy"));
        assert!(mi.contains_key("grok"));
        assert!(mi.contains_key("opencode"));
    }

    #[test]
    fn build_nightly_status_json__errors_object__raw_string_and_unreadable_true() {
        let mut input = fixture_input();
        input.last_errors_raw = Some("{}".to_string());
        let status = build_nightly_status_json(input);
        assert_eq!(
            status.errors_last_run,
            Some(serde_json::Value::String("{}".to_string()))
        );
        assert!(status.errors_last_run_unreadable);
        let value = to_value(&status);
        assert_eq!(value["errors_last_run"], "{}");
        assert_eq!(value["errors_last_run_unreadable"], true);
    }
}
