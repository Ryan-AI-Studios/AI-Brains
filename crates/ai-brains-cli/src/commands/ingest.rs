use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService, parse_ingest_request};
use ai_brains_contracts::ingest::IngestResponse;
use std::io::{self, Read};

const PREVIEW_MAX_LEN: usize = 100;

pub(crate) const INGEST_EMPTY_STDIN_USAGE: &str = "stdin is empty or not piped. Pipe a JSON turn. Example:\n  echo '{\"session_id\":\"00000000-0000-0000-0000-000000000001\",\"project_id\":\"00000000-0000-0000-0000-000000000000\",\"harness_id\":\"00000000-0000-0000-0000-000000000002\",\"turn_id\":\"00000000-0000-0000-0000-000000000003\",\"role\":\"user\",\"content\":\"hello\",\"privacy\":\"CloudOk\"}' | ai-brains ingest --dry-run";

pub(crate) fn ingest_stdin_needs_usage(is_tty: bool, raw: Option<&str>) -> bool {
    is_tty || raw.is_none_or(|s| s.trim().is_empty())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DryRunIngestRequest {
    turn_id: String,
    session_id: String,
    project_id: String,
    harness_id: String,
    role: String,
    content: String,
    privacy: String,
    #[serde(default)]
    thinking: Option<String>,
    #[serde(default)]
    tx_id: Option<String>,
}

fn truncate_preview(s: &str) -> String {
    if s.chars().count() <= PREVIEW_MAX_LEN {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(PREVIEW_MAX_LEN).collect();
        format!("{}...", truncated)
    }
}

pub fn run(ctx: &AppContext, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    use crate::commands::governed_common::fail_usage;
    use std::io::IsTerminal;

    if io::stdin().is_terminal() {
        return fail_usage(INGEST_EMPTY_STDIN_USAGE);
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if ingest_stdin_needs_usage(false, Some(&input)) {
        return fail_usage(INGEST_EMPTY_STDIN_USAGE);
    }

    if dry_run {
        let req: DryRunIngestRequest =
            serde_json::from_str(&input).map_err(|e| format!("Invalid JSON: {}", e))?;
        if req.content.trim().is_empty() {
            return Err("content field is empty".into());
        }
        if req.role.trim().is_empty() {
            return Err("role field is empty".into());
        }
        let preview = truncate_preview(&req.content);
        println!(
            "[dry-run] Would ingest turn {} for project {} / session {} (role={}): {}",
            req.turn_id, req.project_id, req.session_id, req.role, preview
        );
        return Ok(());
    }

    let request = parse_ingest_request(&input)?;

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let mut sink = StoreSink {
        store: event_store,
        last_error: None,
        #[cfg(feature = "graph")]
        graph_hook: Some(crate::live_graph::LiveGraphHook::new(
            std::sync::Arc::clone(&ctx.conn),
        )),
    };

    let service = CaptureService::new();
    let capture_context = CaptureContext {
        git_working_dir: std::env::current_dir().ok(),
    };

    ctx.ensure_project_and_session_exists(
        &mut sink,
        &service,
        &capture_context,
        request.project_id,
        request.session_id,
        request.harness_id,
        request.privacy,
    )?;

    if let Some(err) = sink.last_error.take() {
        return Err(format!("Failed to auto-initialize context: {}", err).into());
    }

    let outcome = service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to persist turn: {}", err).into());
    }

    let response = IngestResponse {
        event_id: outcome
            .primary_event()
            .map(|e| e.event_id.to_string())
            .unwrap_or_else(|| outcome.events[0].event_id.to_string()),
        processed: true,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn ingest_stdin_needs_usage__tty_or_blank__true() {
        assert!(ingest_stdin_needs_usage(true, None));
        assert!(ingest_stdin_needs_usage(false, Some("")));
        assert!(ingest_stdin_needs_usage(false, Some(" \n")));
    }

    #[test]
    fn ingest_stdin_needs_usage__payload__false() {
        assert!(!ingest_stdin_needs_usage(false, Some("{")));
        let valid = r#"{"session_id":"00000000-0000-0000-0000-000000000001","project_id":"00000000-0000-0000-0000-000000000000","harness_id":"00000000-0000-0000-0000-000000000002","turn_id":"00000000-0000-0000-0000-000000000003","role":"user","content":"hello","privacy":"CloudOk"}"#;
        assert!(!ingest_stdin_needs_usage(false, Some(valid)));
    }

    #[test]
    fn ingest_empty_stdin_usage__contains_example_keys() {
        for key in [
            "session_id",
            "project_id",
            "harness_id",
            "turn_id",
            "role",
            "content",
            "privacy",
        ] {
            assert!(
                INGEST_EMPTY_STDIN_USAGE.contains(key),
                "usage const missing {key}"
            );
        }
        assert!(INGEST_EMPTY_STDIN_USAGE.contains("ai-brains ingest --dry-run"));
        assert!(INGEST_EMPTY_STDIN_USAGE.contains("'{"));
        assert!(INGEST_EMPTY_STDIN_USAGE.contains("}'"));
    }
}
