use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{parse_ingest_request, CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestResponse;
use std::io::{self, Read};

const PREVIEW_MAX_LEN: usize = 100;

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
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

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
        event_id: outcome.events[0].event_id.to_string(),
        processed: true,
    };
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}
