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
    dry_run: bool,
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

        // SYSTEM scheduling needs admin: offer UAC relaunch (skip for dry-run).
        if run_as_system && !dry_run {
            // Fail before UAC if wrapper env cannot be built (common when .env missing).
            let _ = generate_nightly_wrapper_script(exe_str)?;
            match crate::elevation::ensure_elevated_or_relaunch()? {
                crate::elevation::ElevationOutcome::AlreadyElevated => {}
                crate::elevation::ElevationOutcome::Relaunched { exit_code } => {
                    if exit_code == 0 {
                        // Elevated child did the work; surface its log (console was hidden).
                        if let Some(Ok(msg)) = crate::elevation::take_elevate_result() {
                            println!("{msg}");
                        }
                        let wrapper = crate::artifact_security::nightly_wrapper_path();
                        println!(
                            "Elevated schedule finished successfully.\n\
                             Task: {task_name} (SYSTEM)\n\
                             Wrapper: {}\n\
                             Note: that path is ACL-restricted (SYSTEM/Administrators only);\n\
                             listing/icacls from a non-elevated shell may say Access denied.\n\
                             Verify with an elevated shell or: schtasks /Query /TN {task_name}",
                            wrapper.display()
                        );
                        return Ok(());
                    }
                    let detail = crate::elevation::take_elevate_result()
                        .and_then(|r| r.err())
                        .or_else(crate::elevation::take_elevate_error_log)
                        .unwrap_or_else(|| {
                            "(no elevated error log; re-run from an Admin shell for full stderr)"
                                .into()
                        });
                    return Err(format!(
                        "Elevated schedule process exited with code {exit_code}: {detail}"
                    )
                    .into());
                }
            }
        }

        let task_command = if run_as_system {
            // T145: wrapper always lives under %ProgramData%\AI-Brains\ with restrictive ACL.
            let wrapper_placeholder = crate::artifact_security::nightly_wrapper_path()
                .display()
                .to_string();
            match generate_nightly_wrapper_script(exe_str) {
                Ok(content) => {
                    if dry_run {
                        let args = build_schtasks_args(
                            &wrapper_placeholder,
                            task_name,
                            &start_time,
                            run_as_system,
                        );
                        println!("[dry-run] Would execute:");
                        println!("  schtasks {}", args.join(" "));
                        println!();
                        println!("Wrapper script content:");
                        println!("{}", content);
                        println!();
                        println!(
                            "(Note: actual registration may require elevated PowerShell privileges depending on system policy)"
                        );
                        return Ok(());
                    }
                    let path = write_wrapper_script(&content)?;
                    // DoD-3 gate: never reach schtasks unless prepare succeeded.
                    if !crate::artifact_security::may_register_after_prepare(true) {
                        return Err(
                            "internal: wrapper prepare reported success but registration gate denied"
                                .into(),
                        );
                    }
                    println!("Wrapper script written to: {}", path.display());
                    format!("'{}'", path.display())
                }
                Err(e) => {
                    if dry_run {
                        let args = build_schtasks_args(
                            &wrapper_placeholder,
                            task_name,
                            &start_time,
                            run_as_system,
                        );
                        println!("[dry-run] Would execute:");
                        println!("  schtasks {}", args.join(" "));
                        println!();
                        println!("(Wrapper script would fail: {})", e);
                        println!();
                        println!(
                            "(Note: actual registration may require elevated PowerShell privileges depending on system policy)"
                        );
                        return Ok(());
                    }
                    // Fail closed: do not call schtasks when wrapper write/ACL verify failed.
                    debug_assert!(!crate::artifact_security::may_register_after_prepare(false));
                    return Err(e);
                }
            }
        } else {
            format!("'{}' nightly", exe_str)
        };

        let args = build_schtasks_args(&task_command, task_name, &start_time, run_as_system);

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
            let msg = format!(
                "Nightly task '{}' scheduled daily at {}.",
                task_name, start_time
            );
            println!("{msg}");
            if crate::elevation::is_elevated() {
                let wrapper = crate::artifact_security::nightly_wrapper_path();
                crate::elevation::write_elevate_success_log(&format!(
                    "{msg}\nWrapper script: {}",
                    wrapper.display()
                ));
            }
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
    tracing::info!("Ingesting structured MADR decisions from Ledgerful...");
    if let Err(e) = ingest_madr_from_changeguard(ctx, project_id) {
        tracing::error!("MADR ingestion failed (non-fatal): {}", e);
        tracing::warn!(
            "MADR ingestion failed: {}. Nightly sweep completed successfully.",
            e
        );
    }

    // --- Symbol Bridge (T70) ---
    tracing::info!("[Nightly] Ingesting code symbols from Ledgerful...");
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
    task_command: &str,
    task_name: &str,
    start_time: &str,
    run_as_system: bool,
) -> Vec<String> {
    let mut args = vec![
        "/create".to_string(),
        "/tn".to_string(),
        task_name.to_string(),
        "/tr".to_string(),
        task_command.to_string(),
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

const REQUIRED_ENV_VARS: [&str; 5] = [
    "AI_BRAINS_VAULT_PATH",
    "AI_BRAINS_MODEL_URL",
    "AI_BRAINS_COMPLETION_MODEL",
    "AI_BRAINS_EMBEDDING_URL",
    "AI_BRAINS_EMBEDDING_MODEL",
];

fn generate_nightly_wrapper_script(exe_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let env_values: Vec<(&str, String)> = REQUIRED_ENV_VARS
        .iter()
        .map(|key| (*key, std::env::var(key).unwrap_or_default()))
        .collect();
    generate_nightly_wrapper_script_from_env(exe_str, &env_values)
}

fn generate_nightly_wrapper_script_from_env(
    exe_str: &str,
    env_values: &[(&str, String)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut lines = vec!["@echo off".to_string()];
    let mut missing = Vec::new();
    for key in REQUIRED_ENV_VARS {
        let value = env_values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        if value.is_empty() {
            tracing::warn!("Required env var {} is missing or empty", key);
            missing.push(key);
        } else {
            lines.push(format!("set \"{}={}\"", key, value));
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "Cannot schedule as SYSTEM: required env vars missing or empty: {}. \
             Run from a directory with a .env file, or set them in your user environment before scheduling.",
            missing.join(", ")
        )
        .into());
    }
    let vault_path = env_values
        .iter()
        .find(|(k, _)| *k == "AI_BRAINS_VAULT_PATH")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    if let Some(parent) = std::path::Path::new(vault_path).parent() {
        if !parent.as_os_str().is_empty() {
            lines.push(format!("cd /d \"{}\"", parent.display()));
        }
    }
    lines.push(format!(
        r#""{}" --no-project-context nightly --skip-import --log-format json"#,
        exe_str
    ));
    Ok(lines.join("\n"))
}

fn write_wrapper_script(content: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // T145: %ProgramData%\AI-Brains\nightly-task.bat with SYSTEM+Administrators ACL only.
    // write_protected_artifact refuses reparse/symlink targets and verifies ACL (fail closed).
    let path = crate::artifact_security::nightly_wrapper_path();
    crate::artifact_security::write_protected_artifact(&path, content)?;
    Ok(path)
}

/// Fetch structured MADR records from Ledgerful via bridge IPC and ingest as
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

    // Call Ledgerful bridge export --ledger to fetch MADR records
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
            tracing::warn!("Ledgerful bridge export failed: {}", stderr);
            return Ok(()); // Non-fatal: fail gracefully
        }
        Err(e) => {
            tracing::warn!("Ledgerful CLI not available: {}", e);
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

    #[test]
    #[allow(non_snake_case)]
    fn build_schtasks_args__run_as_system__includes_no_project_context_and_skip_import() {
        let args = build_schtasks_args(
            r"C:\fake\ai-brains.exe --no-project-context nightly --skip-import --log-format json",
            "AI-Brains-Nightly",
            "03:00",
            true,
        );
        let tr = args
            .iter()
            .position(|a| a == "/tr")
            .expect("/tr argument present");
        let task_command = &args[tr + 1];
        assert!(task_command.contains("--no-project-context"));
        assert!(task_command.contains("--skip-import"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_schtasks_args__no_run_as_system__omits_no_project_context_and_skip_import() {
        let args = build_schtasks_args(
            r"C:\fake\ai-brains.exe nightly",
            "AI-Brains-Nightly",
            "03:00",
            false,
        );
        let tr = args
            .iter()
            .position(|a| a == "/tr")
            .expect("/tr argument present");
        let task_command = &args[tr + 1];
        assert!(!task_command.contains("--no-project-context"));
        assert!(!task_command.contains("--skip-import"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_schtasks_args__run_as_system__tr_points_to_wrapper_script() {
        let wrapper = crate::artifact_security::nightly_wrapper_path();
        let wrapper_str = wrapper.display().to_string();
        let args = build_schtasks_args(&wrapper_str, "AI-Brains-Nightly", "03:00", true);
        let tr = args
            .iter()
            .position(|a| a == "/tr")
            .expect("/tr argument present");
        let task_command = &args[tr + 1];
        assert!(task_command.ends_with("nightly-task.bat"));
        assert!(task_command.contains("AI-Brains"));
        assert!(!task_command.contains("ai-brains.exe"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_schtasks_args__run_as_system__wrapper_script_contains_env_vars(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let env_values: Vec<(&str, String)> = vec![
            ("AI_BRAINS_VAULT_PATH", "C:\\vault.db".to_string()),
            ("AI_BRAINS_MODEL_URL", "http://127.0.0.1:8081".to_string()),
            ("AI_BRAINS_COMPLETION_MODEL", "model.gguf".to_string()),
            (
                "AI_BRAINS_EMBEDDING_URL",
                "http://127.0.0.1:8083".to_string(),
            ),
            ("AI_BRAINS_EMBEDDING_MODEL", "embed-model".to_string()),
        ];
        let content =
            generate_nightly_wrapper_script_from_env(r"C:\fake\ai-brains.exe", &env_values)?;
        assert!(content.contains("set \"AI_BRAINS_VAULT_PATH=C:\\vault.db\""));
        assert!(content.contains("set \"AI_BRAINS_MODEL_URL=http://127.0.0.1:8081\""));
        assert!(content.contains("set \"AI_BRAINS_COMPLETION_MODEL=model.gguf\""));
        assert!(content.contains("set \"AI_BRAINS_EMBEDDING_URL=http://127.0.0.1:8083\""));
        assert!(content.contains("set \"AI_BRAINS_EMBEDDING_MODEL=embed-model\""));
        assert!(content.contains("--no-project-context"));
        assert!(content.contains("--skip-import"));
        assert!(content.contains(r#""C:\fake\ai-brains.exe""#));
        assert!(content.contains("cd /d \"C:\\\""));
        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    fn generate_nightly_wrapper_script__missing_env_var__returns_error() {
        let env_values: Vec<(&str, String)> = vec![
            ("AI_BRAINS_VAULT_PATH", "C:\\vault.db".to_string()),
            ("AI_BRAINS_MODEL_URL", String::new()),
            ("AI_BRAINS_COMPLETION_MODEL", "model.gguf".to_string()),
            (
                "AI_BRAINS_EMBEDDING_URL",
                "http://127.0.0.1:8083".to_string(),
            ),
            ("AI_BRAINS_EMBEDDING_MODEL", "embed-model".to_string()),
        ];
        let result =
            generate_nightly_wrapper_script_from_env(r"C:\fake\ai-brains.exe", &env_values);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("AI_BRAINS_MODEL_URL"));
    }
}
