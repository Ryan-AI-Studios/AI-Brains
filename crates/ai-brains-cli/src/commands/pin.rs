use crate::context::{AppContext, StoreSink};
use ai_brains_capture::{CaptureContext, CaptureService};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TransactionId, TurnId};
use ai_brains_core::privacy::Privacy;
use std::io::Read;

const PREVIEW_MAX_LEN: usize = 100;

fn truncate_preview(s: &str) -> String {
    if s.chars().count() <= PREVIEW_MAX_LEN {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(PREVIEW_MAX_LEN).collect();
        format!("{}...", truncated)
    }
}

pub fn run(
    ctx: &AppContext,
    content: String,
    role: String,
    privacy_str: String,
    tags: Vec<String>,
    tx_id: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .map_err(|_| "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first.")?
        .parse::<ProjectId>()?;

    let session_id = std::env::var("AI_BRAINS_SESSION_ID")
        .map_err(|_| "AI_BRAINS_SESSION_ID not set. Run 'ai-brains context' first.")?
        .parse::<SessionId>()?;

    let harness_id = std::env::var("AI_BRAINS_HARNESS_ID")
        .ok()
        .and_then(|s| s.parse::<HarnessId>().ok())
        .unwrap_or_default();

    let tx_id_parsed = tx_id
        .or_else(crate::context::read_ledger_tx_id)
        .map(TransactionId::new);

    let privacy = match privacy_str.to_lowercase().as_str() {
        "cloudok" => Privacy::CloudOk,
        "localonly" => Privacy::LocalOnly,
        "neverinject" => Privacy::NeverInject,
        "sealed" => Privacy::Sealed,
        _ => Privacy::LocalOnly,
    };

    let final_content = if tags.is_empty() {
        content.clone()
    } else {
        format!("TAGS: {}\n{}", tags.join(", "), content)
    };

    let turn_id = TurnId::new();
    let request = IngestRequest {
        session_id,
        project_id,
        harness_id,
        turn_id,
        role,
        content: final_content,
        thinking: None,
        privacy,
        tx_id: tx_id_parsed,
    };

    if dry_run {
        let preview = truncate_preview(&content);
        println!(
            "[dry-run] Would pin memory {} (role={}, privacy={}, tags={:?}): {}",
            turn_id, request.role, privacy_str, tags, preview
        );
        return Ok(());
    }

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
        project_id,
        session_id,
        harness_id,
        privacy,
    )?;

    if let Some(err) = sink.last_error.take() {
        return Err(format!("Failed to auto-initialize context: {}", err).into());
    }

    service.ingest_request(request, capture_context, &mut sink)?;

    if let Some(err) = sink.last_error {
        return Err(format!("Failed to pin memory: {}", err).into());
    }

    // The projection stores turn_id as memory_id; print it so `forget --memory-id` works.
    println!("Memory {} successfully pinned to vault.", turn_id);
    Ok(())
}

/// Read content from stdin instead of a positional argument
pub fn run_stdin(
    ctx: &AppContext,
    role: String,
    privacy_str: String,
    tags: Vec<String>,
    tx_id: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let content = input.trim().to_string();
    if content.is_empty() {
        return Err("stdin content is empty".into());
    }
    run(ctx, content, role, privacy_str, tags, tx_id, dry_run)
}
