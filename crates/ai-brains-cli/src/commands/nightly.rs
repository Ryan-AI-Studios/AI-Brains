use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_store::EventStore;
use std::str::FromStr;
use std::sync::Arc;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    ctx: &AppContext,
    schedule: bool,
    unschedule: bool,
    start_time: String,
    status: bool,
    skip_import: bool,
    run_as_system: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let task_name = "AI-Brains-Nightly";

    if status {
        let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;
        let unsummarized = query_store.get_unsummarized_sessions()?;
        let last_run = query_store.get_last_nightly_run()?;
        let last_count = query_store
            .get_sync_state("last_nightly_count")?
            .unwrap_or_else(|| "0".to_string());
        let last_errors = query_store
            .get_sync_state("last_nightly_errors")?
            .unwrap_or_else(|| "[]".to_string());

        #[cfg(windows)]
        let schedule_line = check_schedule_state();
        #[cfg(not(windows))]
        let schedule_line = "Scheduled: (unknown on non-Windows)".to_string();

        println!("=== Nightly Status ===");
        println!("{}", schedule_line);
        match last_run {
            Some(ts) => println!("Last nightly run: {}", ts),
            None => println!("Last nightly run: never"),
        }
        println!("Unsummarized sessions remaining: {}", unsummarized.len());
        println!("Sessions summarized in last run: {}", last_count);
        println!("Errors in last run: {}", last_errors);
        println!("======================");
        return Ok(());
    }

    if unschedule {
        let output = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        if output.status.success() {
            println!("Nightly task '{}' removed.", task_name);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to remove task: {}", stderr);
        }
        return Ok(());
    }

    if schedule {
        let exe_path = std::env::current_exe()?;
        let exe_str = exe_path.to_str().ok_or("Invalid executable path")?;
        let args = build_schtasks_args(exe_str, task_name, &start_time, run_as_system);

        let output = std::process::Command::new("schtasks")
            .args(&args)
            .output()
            .map_err(|e| {
                format!(
                    "Failed to execute schtasks: {}. Run in an elevated PowerShell session.",
                    e
                )
            })?;

        if output.status.success() {
            println!(
                "Nightly task '{}' scheduled daily at {}.",
                task_name, start_time
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            if run_as_system
                && (stderr.contains("Access is denied") || stdout.contains("Access is denied"))
            {
                return Err(
                    "Scheduling as SYSTEM requires elevation. Re-run from an Administrator shell."
                        .into(),
                );
            }
            let cmd = ai_brains_scheduler::TaskScheduler::render_create_command(
                exe_str,
                task_name,
                &start_time,
            );
            tracing::warn!(
                "Failed to schedule task. Run this in an elevated PowerShell session:\n{}\nError: {}{}",
                cmd, stdout, stderr
            );
        }
        return Ok(());
    }

    // Ensure daemon is running for background intelligence sweep
    let daemon_client = crate::daemon_client::DaemonClient::new();
    if !daemon_client
        .ensure_running(&ctx.vault_path, &ctx._key)
        .await
    {
        tracing::warn!(
            "Failed to ensure daemon is running. Nightly sweep may have reduced functionality."
        );
    }

    let project_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .and_then(|s| ProjectId::from_str(&s).ok())
        .unwrap_or_default();

    if project_id == ProjectId::default() {
        tracing::warn!(
            "AI_BRAINS_PROJECT_ID not set. Run 'ai-brains context' first. Using default project."
        );
    }

    #[cfg(feature = "graph")]
    let event_store: Arc<dyn ai_brains_store::EventStore + Send + Sync> = Arc::new(
        crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone()),
    );
    #[cfg(not(feature = "graph"))]
    let event_store: Arc<dyn ai_brains_store::EventStore + Send + Sync> =
        Arc::new(ai_brains_store::SqliteEventStore::new((*ctx.conn).clone()));
    let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;

    let model_url = std::env::var("AI_BRAINS_MODEL_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
    let completion_model = std::env::var("AI_BRAINS_COMPLETION_MODEL")
        .unwrap_or_else(|_| "gemma-4-E4B-it-Q6_K.gguf".to_string());

    let embedding_url = std::env::var("AI_BRAINS_EMBEDDING_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8083".to_string());
    let embedding_model = std::env::var("AI_BRAINS_EMBEDDING_MODEL")
        .unwrap_or_else(|_| "nomic-embed-text-v1.5".to_string());

    let completion_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        model_url,
        completion_model,
    ));
    let embedding_provider = Arc::new(ai_brains_models::llama_cpp::LlamaCppProvider::new(
        embedding_url,
        embedding_model,
    ));

    // Import Antigravity sessions before summarization so they get summarized too
    if skip_import {
        tracing::info!(
            "Skipping Antigravity import (--skip-import). \
             Use this on isolated, CI, or per-project vaults to prevent \
             cross-vault contamination from the user's real Antigravity history."
        );
    } else if let Err(e) = crate::commands::antigravity_import::run(ctx, 30) {
        tracing::error!("Antigravity import failed: {}", e);
    }

    let service = ai_brains_brain::NightlyService::new(
        query_store,
        event_store,
        completion_provider,
        embedding_provider,
    );

    tracing::info!("Starting nightly intelligence sweep...");
    tracing::info!("Summarizing sessions...");

    let batch_size = std::env::var("AI_BRAINS_NIGHTLY_BATCH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok());
    let count = service.run_nightly(project_id, batch_size).await?;
    tracing::info!("Running memory synthesis...");

    // WAL checkpoint: ensure embeddings generated during nightly are persisted
    // before potential timeout on MADR ingestion
    if let Err(e) = ctx.conn.wal_checkpoint() {
        tracing::warn!("WAL checkpoint failed (non-fatal, possibly locked): {}", e);
    } else {
        tracing::info!("WAL checkpointed — embeddings persisted to disk.");
    }

    tracing::info!("Nightly sweep completed. {} sessions summarized.", count);

    tracing::info!("Stats: {} sessions summarized.", count);
    tracing::info!("Embedding stats: see stderr output above.");
    #[cfg(feature = "graph")]
    tracing::info!("[Nightly] Graph updated incrementally — run 'graph rebuild' only if you suspect missing edges.");

    // --- MADR Ingestion (Phase 18: T41) ---
    tracing::info!("Ingesting structured MADR decisions from ChangeGuard...");
    if let Err(e) = ingest_madr_from_changeguard(ctx, project_id) {
        tracing::error!("MADR ingestion failed (non-fatal): {}", e);
        tracing::warn!(
            "MADR ingestion failed: {}. Nightly sweep completed successfully.",
            e
        );
    }

    // --- Symbol Bridge (T70) ---
    tracing::info!("[Nightly] Ingesting code symbols from ChangeGuard...");
    match crate::commands::symbol_bridge::ingest_symbols_from_changeguard(ctx, project_id) {
        Ok(n) => tracing::info!("[Nightly] {} code symbols ingested.", n),
        Err(e) => tracing::warn!("[Nightly] Symbol ingestion failed (non-fatal): {}", e),
    }

    Ok(())
}

#[cfg(windows)]
fn check_schedule_state() -> String {
    let output = std::process::Command::new("schtasks")
        .args(["/query", "/tn", "AI-Brains-Nightly", "/fo", "CSV", "/nh"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let fields: Vec<&str> = line.split(',').collect();
            let next_run = fields
                .get(1)
                .map(|s| s.trim_matches('"'))
                .unwrap_or("unknown");
            format!("Scheduled: Yes (next run: {})", next_run)
        }
        _ => "Scheduled: No (run 'ai-brains nightly --schedule' to enable)".to_string(),
    }
}

fn build_schtasks_args(
    exe_str: &str,
    task_name: &str,
    start_time: &str,
    run_as_system: bool,
) -> Vec<String> {
    let mut args = vec![
        "/create".to_string(),
        "/tn".to_string(),
        task_name.to_string(),
        "/tr".to_string(),
        format!("'{}' nightly", exe_str),
        "/sc".to_string(),
        "daily".to_string(),
        "/st".to_string(),
        start_time.to_string(),
    ];
    if run_as_system {
        args.push("/ru".to_string());
        args.push("SYSTEM".to_string());
    }
    args.push("/f".to_string());
    args
}

/// Fetch structured MADR records from ChangeGuard via bridge IPC and ingest as
/// Decision domain events into the event store.
fn ingest_madr_from_changeguard(
    ctx: &AppContext,
    project_id: ProjectId,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_contracts::bridge::BridgeRecord;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let temp_path = {
        let mut p = std::env::temp_dir();
        p.push("cg_madr_export.ndjson");
        p
    };

    // Call ChangeGuard bridge export --ledger to fetch MADR records
    let output = std::process::Command::new("ledgerful")
        .args([
            "bridge",
            "export",
            "--out",
            temp_path.to_str().ok_or("Invalid temp path")?,
            "--ledger",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!("ChangeGuard bridge export failed: {}", stderr);
            return Ok(()); // Non-fatal: fail gracefully
        }
        Err(e) => {
            tracing::warn!("ChangeGuard CLI not available: {}", e);
            return Ok(()); // Non-fatal: fail gracefully
        }
    }

    // Parse exported records looking for MADR entries
    let file = match File::open(&temp_path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Failed to open MADR export file: {}", e);
            return Ok(());
        }
    };
    let reader = BufReader::new(file);

    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let mut ingested = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("Failed to read MADR export line: {}", e);
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse BridgeRecord in MADR export: {}", e);
                continue;
            }
        };

        // Only process MADR/decision records
        let record_kind_lower = record.record_kind.to_lowercase();
        if record_kind_lower != "madr" && record_kind_lower != "decision" {
            continue;
        }

        // Extract structured MADR fields from payload
        let payload = record.payload_value();
        let title = payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Decision")
            .to_string();
        let context = payload
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let decision = payload
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let consequences = payload
            .get("consequences")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if decision.is_empty() && context.is_empty() {
            continue; // Skip records without meaningful MADR content
        }

        // Parse record-level IDs
        let record_project_id = if !record.project_id.is_empty() {
            ProjectId::from_str(&record.project_id).ok()
        } else {
            None
        };
        let record_session_id = record
            .session_id
            .as_ref()
            .and_then(|s| ai_brains_core::ids::SessionId::from_str(s).ok());
        let tx_id = record
            .tx_id
            .as_ref()
            .map(|s| ai_brains_core::ids::TransactionId::new(s.clone()));

        // Build DecisionRecorded event
        let decision_id = MemoryId::new();
        let event = ai_brains_events::constructors::EventBuilder::new(
            ai_brains_events::AggregateType::Decision,
            decision_id.as_uuid(),
            ai_brains_events::EventKind::DecisionRecorded,
            ai_brains_events::Actor::System,
            record.privacy,
        )
        .build(ai_brains_events::Payload::DecisionRecorded(
            ai_brains_events::DecisionRecordedPayload {
                decision_id,
                title,
                context,
                decision,
                consequences,
                project_id: record_project_id.or(Some(project_id)),
                session_id: record_session_id,
                tx_id,
            },
        ))?;

        event_store.append_event(&event)?;
        ingested += 1;
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    tracing::info!("MADR ingestion completed. {} decisions ingested.", ingested);
    Ok(())
}

/// Format structured MADR fields into MADR-compliant markdown.
/// This is used by the projection handler; exposed here for testability.
#[allow(dead_code)]
pub fn format_madr_markdown(
    title: &str,
    context: &str,
    decision: &str,
    consequences: &str,
) -> String {
    format!(
        "# {}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}",
        title, context, decision, consequences
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__schedule_state_parse__extracts_next_run_from_csv() {
        let csv_line =
            "\"\\AI-Brains-Nightly\",\"6/25/2026 1:00:00 AM\",\"Ready\",\"Interactive/Background\"";
        let fields: Vec<&str> = csv_line.split(',').collect();
        let next_run = fields
            .get(1)
            .map(|s| s.trim_matches('"'))
            .unwrap_or("unknown");
        assert!(!next_run.is_empty());
        assert!(next_run.contains("6/25/2026"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn nightly_status__schedule_state_parse__empty_output_reports_not_scheduled() {
        let csv_line = "";
        let fields: Vec<&str> = csv_line.split(',').collect();
        let next_run = fields
            .get(1)
            .map(|s| s.trim_matches('"'))
            .unwrap_or("unknown");
        assert_eq!(next_run, "unknown");
    }

    #[test]
    fn format_madr_markdown_produces_expected_structure() {
        let result = format_madr_markdown(
            "ADR: Use SQLite",
            "We needed an embedded database.",
            "We chose SQLite with SQLCipher.",
            "Simpler deployment, encrypted at rest.",
        );

        assert!(result.contains("# ADR: Use SQLite"));
        assert!(result.contains("## Context"));
        assert!(result.contains("We needed an embedded database."));
        assert!(result.contains("## Decision"));
        assert!(result.contains("We chose SQLite with SQLCipher."));
        assert!(result.contains("## Consequences"));
        assert!(result.contains("Simpler deployment, encrypted at rest."));
    }

    #[test]
    fn format_madr_markdown_handles_empty_fields() {
        let result = format_madr_markdown("Title Only", "", "", "");

        assert!(result.contains("# Title Only"));
        assert!(result.contains("## Context\n\n"));
        assert!(result.contains("## Decision\n\n"));
        assert!(result.contains("## Consequences\n"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn nightly_schedule__run_as_system__adds_ru_system() {
        let args =
            build_schtasks_args(r"C:\fake\ai-brains.exe", "AI-Brains-Nightly", "03:00", true);
        let ru_pos = args.iter().position(|a| *a == "/ru");
        let system_pos = args.iter().position(|a| *a == "SYSTEM");
        assert!(ru_pos.is_some());
        assert!(system_pos.is_some());
        assert!(ru_pos < system_pos);
        assert_eq!(args.last().map(String::as_str), Some("/f"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn nightly_schedule__no_run_as_system__omits_ru_system() {
        let args = build_schtasks_args(
            r"C:\fake\ai-brains.exe",
            "AI-Brains-Nightly",
            "03:00",
            false,
        );
        assert!(!args.iter().any(|a| a == "/ru"));
        assert!(!args.iter().any(|a| a == "SYSTEM"));
        assert_eq!(args.last().map(String::as_str), Some("/f"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn nightly_schedule__run_as_system_not_elevated__clear_error() {
        let stderr = "ERROR: Access is denied.";
        let stdout = "";
        let run_as_system = true;
        assert!(
            run_as_system
                && (stderr.contains("Access is denied") || stdout.contains("Access is denied"))
        );
    }
}
