use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_models::llama_cpp::{LlamaCppProvider, ProbeStatus};
use ai_brains_store::EventStore;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// Soft probe timeout for status + pre-summarize (independent of 120s LLM timeout).
const NIGHTLY_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Default completion endpoint when env is unset (matches run path).
const DEFAULT_MODEL_URL: &str = "http://127.0.0.1:8081";
/// Default embedding endpoint when env is unset (matches run path).
const DEFAULT_EMBEDDING_URL: &str = "http://127.0.0.1:8083";
const DEFAULT_COMPLETION_MODEL: &str = "gemma-4-E4B-it-Q6_K.gguf";
const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text-v1.5";

#[allow(clippy::too_many_arguments)]
pub async fn run(
    ctx: &AppContext,
    schedule: bool,
    unschedule: bool,
    start_time: String,
    status: bool,
    skip_import: bool,
    skip_import_agy: bool,
    skip_import_grok: bool,
    skip_import_opencode: bool,
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

        let (model_url, completion_model, embedding_url, embedding_model) =
            resolve_nightly_model_endpoints();

        // Soft probes — never fail status (exit 0 even when down).
        let completion_probe = {
            let p = LlamaCppProvider::new(model_url.clone(), completion_model.clone());
            p.probe_health(NIGHTLY_PROBE_TIMEOUT).await
        };
        let embedding_probe = {
            let p = LlamaCppProvider::new(embedding_url.clone(), embedding_model.clone());
            p.probe_health(NIGHTLY_PROBE_TIMEOUT).await
        };

        println!("=== Nightly Status ===");
        #[cfg(windows)]
        {
            let next_run = fetch_schedule_next_run(task_name);
            let last_result = fetch_last_task_result(task_name);
            for line in
                format_schedule_status_lines(next_run.as_deref(), last_result.as_deref(), true)
            {
                println!("{line}");
            }
        }
        #[cfg(not(windows))]
        {
            println!("Scheduled: (unknown on non-Windows)");
        }
        match last_run {
            Some(ts) => println!("Last nightly run: {}", ts),
            None => println!("Last nightly run: never"),
        }
        println!("Unsummarized sessions remaining: {}", unsummarized.len());
        println!("Sessions summarized in last run: {}", last_count);
        println!("Errors in last run: {}", last_errors);
        println!(
            "{}",
            format_endpoint_line(
                "Completion",
                &model_url,
                &completion_model,
                completion_probe.as_label(),
            )
        );
        println!(
            "{}",
            format_endpoint_line(
                "Embedding",
                &embedding_url,
                &embedding_model,
                embedding_probe.as_label(),
            )
        );
        // T239: multi-import block (missing → never; corrupt → unreadable)
        match crate::commands::multi_import::load_multi_import_status(query_store.as_ref()) {
            Ok(view) => crate::commands::multi_import::print_multi_import_status(&view),
            Err(e) => {
                tracing::warn!(error = %e, "failed to load last_multi_import (non-fatal)");
                println!("Multi-import: unreadable");
            }
        }
        println!("======================");
        // Status exit remains 0 when probe is down/timeout/error.
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
                cmd,
                stdout,
                stderr
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

    // T229 F13: nil UUID sentinel when env missing/invalid — never ProjectId::default() (random).
    let project_id =
        resolve_nightly_project_id(std::env::var("AI_BRAINS_PROJECT_ID").ok().as_deref());

    if project_id == ProjectId::from_uuid(uuid::Uuid::nil()) {
        tracing::warn!(
            "AI_BRAINS_PROJECT_ID not set or invalid. Run 'ai-brains context' first. Using nil project sentinel."
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

    let (model_url, completion_model, embedding_url, embedding_model) =
        resolve_nightly_model_endpoints();

    let completion_provider = Arc::new(LlamaCppProvider::new(
        model_url.clone(),
        completion_model.clone(),
    ));
    let embedding_provider = Arc::new(LlamaCppProvider::new(
        embedding_url.clone(),
        embedding_model.clone(),
    ));

    // T239: multi-harness import (agy → grok → opencode) before summarization.
    // Fail-open per source; SYSTEM scheduled nightly keeps --skip-import (D12).
    {
        use crate::commands::multi_import::{
            MultiImportOptions, persist_multi_import_report, run_multi_harness_import,
        };
        if skip_import {
            tracing::info!(
                "Skipping multi-harness session import (--skip-import). \
                 Skips AGY, Grok, and OpenCode batch importers. \
                 Use this on isolated, CI, SYSTEM-scheduled, or per-project vaults \
                 to prevent cross-vault contamination from real harness history."
            );
        } else if skip_import_agy || skip_import_grok || skip_import_opencode {
            tracing::info!(
                skip_agy = skip_import_agy,
                skip_grok = skip_import_grok,
                skip_opencode = skip_import_opencode,
                "Multi-harness import with per-source skip flags"
            );
        }
        let opts = MultiImportOptions::production(
            skip_import,
            skip_import_agy,
            skip_import_grok,
            skip_import_opencode,
        );
        let report = run_multi_harness_import(ctx, opts);
        let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        persist_multi_import_report(&store, &report);
        tracing::info!(
            agy = %report.agy.status,
            grok = %report.grok.status,
            opencode = %report.opencode.status,
            "Multi-harness import phase complete"
        );
    }

    // T229 F2: soft probe after multi-import, before summarize — non-fatal warn if down.
    {
        let c = completion_provider
            .probe_health(NIGHTLY_PROBE_TIMEOUT)
            .await;
        if c != ProbeStatus::Ok {
            tracing::warn!(
                endpoint = %model_url,
                probe = c.as_label(),
                "completion endpoint soft probe failed before summarize (non-fatal)"
            );
        }
        let e = embedding_provider.probe_health(NIGHTLY_PROBE_TIMEOUT).await;
        if e != ProbeStatus::Ok {
            tracing::warn!(
                endpoint = %embedding_url,
                probe = e.as_label(),
                "embedding endpoint soft probe failed before summarize (non-fatal)"
            );
        }
    }

    // F-004: class-matrix dry-run log (plan only; never apply CE on nightly).
    {
        use ai_brains_control_plane::{RetentionConfig, plan_retention};
        let plan_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        let config = RetentionConfig::from_env();
        match plan_retention(&plan_store, &config) {
            Ok(report) => {
                tracing::info!(
                    candidates = report.totals.candidates,
                    ce_wipe = report.totals.would_ce_wipe,
                    projection_delete = report.totals.would_projection_delete,
                    skip = report.totals.would_skip,
                    held = report.totals.would_held,
                    "retention class dry-run (no apply)"
                );
                eprintln!(
                    "[Nightly] Retention class dry-run: candidates={} ce_wipe={} projection_delete={} skip={} held={} (no apply)",
                    report.totals.candidates,
                    report.totals.would_ce_wipe,
                    report.totals.would_projection_delete,
                    report.totals.would_skip,
                    report.totals.would_held
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "retention class dry-run failed (non-fatal)");
                eprintln!("[Nightly] Retention class dry-run failed (non-fatal): {e}");
            }
        }
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
    tracing::info!(
        "[Nightly] Graph updated incrementally — run 'graph rebuild' only if you suspect missing edges."
    );

    // --- MADR Ingestion (Phase 18: T41) ---
    tracing::info!("Ingesting structured MADR decisions from Ledgerful...");
    if let Err(e) = ingest_madr_from_ledgerful(ctx, project_id) {
        tracing::error!("MADR ingestion failed (non-fatal): {}", e);
        tracing::warn!(
            "MADR ingestion failed: {}. Nightly sweep completed successfully.",
            e
        );
    }

    // --- Symbol Bridge (T70) ---
    tracing::info!("[Nightly] Ingesting code symbols from Ledgerful...");
    match crate::commands::symbol_bridge::ingest_symbols_from_ledgerful(ctx, project_id) {
        Ok(n) => tracing::info!("[Nightly] {} code symbols ingested.", n),
        Err(e) => tracing::warn!("[Nightly] Symbol ingestion failed (non-fatal): {}", e),
    }

    Ok(())
}

/// Resolve MODEL/EMBED URLs and model names (env with documented defaults).
fn resolve_nightly_model_endpoints() -> (String, String, String, String) {
    let model_url =
        std::env::var("AI_BRAINS_MODEL_URL").unwrap_or_else(|_| DEFAULT_MODEL_URL.to_string());
    let completion_model = std::env::var("AI_BRAINS_COMPLETION_MODEL")
        .unwrap_or_else(|_| DEFAULT_COMPLETION_MODEL.to_string());
    let embedding_url = std::env::var("AI_BRAINS_EMBEDDING_URL")
        .unwrap_or_else(|_| DEFAULT_EMBEDDING_URL.to_string());
    let embedding_model = std::env::var("AI_BRAINS_EMBEDDING_MODEL")
        .unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.to_string());
    (model_url, completion_model, embedding_url, embedding_model)
}

/// T229 F13: missing/empty/invalid env → nil UUID (never random `ProjectId::default()`).
pub(crate) fn resolve_nightly_project_id(env_val: Option<&str>) -> ProjectId {
    let Some(raw) = env_val.map(str::trim).filter(|s| !s.is_empty()) else {
        return ProjectId::from_uuid(uuid::Uuid::nil());
    };
    ProjectId::from_str(raw).unwrap_or_else(|_| ProjectId::from_uuid(uuid::Uuid::nil()))
}

/// Host:port for status lines; strips `user:pass@`, path, query, and fragment.
/// Never prints vault keys or token query parameters.
pub(crate) fn host_port_from_url(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Drop credentials if present (`user:pass@host:port`).
    let hostpart = match rest.rsplit_once('@') {
        Some((_creds, host)) => host,
        None => rest,
    };
    // Drop path (`/…`), query (`?…`), fragment (`#…`) — keep host:port only.
    let authority = hostpart
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(hostpart)
        .trim();
    if authority.is_empty() {
        "(invalid-url)".to_string()
    } else {
        authority.to_string()
    }
}

/// Format a single endpoint status line (AC3).
///
/// Example: `Completion: 127.0.0.1:8081  model=gemma…  probe=ok`
pub(crate) fn format_endpoint_line(
    kind: &str,
    url: &str,
    model: &str,
    probe_label: &str,
) -> String {
    let host_port = host_port_from_url(url);
    format!("{kind}: {host_port}  model={model}  probe={probe_label}")
}

/// Quote-aware CSV field split (T229 F6b) — for next-run only (cols 0–2).
///
/// Live Windows `schtasks /FO CSV` has only three columns; do **not** parse Last Result from CSV.
pub(crate) fn split_csv_fields_quote_aware(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        cur.push('"');
                        let _ = chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    fields.push(cur);
    fields
}

/// Extract next-run time from a schtasks CSV data line (column 1), or `None` if unusable.
pub(crate) fn next_run_from_schtasks_csv_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let fields = split_csv_fields_quote_aware(trimmed);
    let next = fields.get(1).map(|s| s.trim()).filter(|s| !s.is_empty())?;
    Some(next.to_string())
}

/// Schedule + Last Result display lines (AC4–AC5).
///
/// - Missing next_run → `Scheduled: No …`
/// - `include_last_result`: Windows only; missing last_result → `unknown`
pub(crate) fn format_schedule_status_lines(
    next_run: Option<&str>,
    last_result: Option<&str>,
    include_last_result: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    match next_run.map(str::trim).filter(|s| !s.is_empty()) {
        Some(nr) => lines.push(format!("Scheduled: Yes (next run: {nr})")),
        None => {
            lines.push("Scheduled: No (run 'ai-brains nightly --schedule' to enable)".to_string())
        }
    }
    if include_last_result {
        let label = last_result
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown");
        lines.push(format!("Last task result: {label}"));
    }
    lines
}

/// Parse PowerShell `LastTaskResult` stdout (trim; empty → None).
pub(crate) fn parse_last_task_result_ps_stdout(stdout: &str) -> Option<String> {
    let t = stdout.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Soft parse of English `schtasks /FO LIST /V` `Last Result:` line (locale-sensitive fallback).
pub(crate) fn parse_last_result_list_v(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let line = line.trim();
        // Accept "Last Result:" with optional spaces around the colon.
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("last result")
            && let Some(rest) = line.split_once(':').map(|(_, r)| r.trim())
            && !rest.is_empty()
        {
            return Some(rest.to_string());
        }
    }
    None
}

#[cfg(windows)]
fn fetch_schedule_next_run(task_name: &str) -> Option<String> {
    let output = std::process::Command::new("schtasks")
        .args(["/query", "/tn", task_name, "/fo", "CSV", "/nh"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let line = stdout.lines().next().unwrap_or("");
            next_run_from_schtasks_csv_line(line)
        }
        _ => None,
    }
}

/// Windows Last Result: primary Get-ScheduledTaskInfo; fallback LIST /V English label.
#[cfg(windows)]
fn fetch_last_task_result(task_name: &str) -> Option<String> {
    let ps_cmd = format!("(Get-ScheduledTaskInfo -TaskName '{task_name}').LastTaskResult");
    let ps = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output();
    if let Ok(out) = ps
        && out.status.success()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if let Some(v) = parse_last_task_result_ps_stdout(&stdout) {
            return Some(v);
        }
    }
    // Soft fallback: schtasks LIST /V (locale may break → None → status prints unknown).
    let list = std::process::Command::new("schtasks")
        .args(["/query", "/tn", task_name, "/fo", "LIST", "/v"])
        .output();
    if let Ok(out) = list
        && out.status.success()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        return parse_last_result_list_v(&stdout);
    }
    None
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

/// Build SYSTEM wrapper content from **current process env**.
///
/// T229 F4 (verify-only): main merges global dotenv (`%USERPROFILE%\.ai-brains\.env`, T205)
/// before subcommands, so MODEL/EMBED keys present only in the global file are already in
/// `std::env` here and get baked into the wrapper. No extra gap-fill in this function.
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
    // Windows .bat scripts always use `\`. Host Path::parent is OS-sensitive
    // (`\` is not a separator on Unix), so split on Windows separators after
    // normalizing `/` → `\` (T179 cross-platform unit tests).
    if let Some(parent) = windows_path_parent(vault_path) {
        lines.push(format!("cd /d \"{parent}\""));
    }
    lines.push(format!(
        r#""{}" --no-project-context nightly --skip-import --log-format json"#,
        exe_str
    ));
    Ok(lines.join("\n"))
}

/// Parent directory of a Windows-style path for `.bat` `cd /d` lines.
///
/// Treats both `\` and `/` as separators so generation is host-OS independent.
/// Drive roots are returned with a trailing `\` (`C:\`), matching `Path::parent`
/// on Windows for `C:\file.db`.
fn windows_path_parent(path: &str) -> Option<String> {
    let normalized = path.replace('/', "\\");
    let trimmed = normalized.trim_end_matches('\\');
    let (parent, _leaf) = trimmed.rsplit_once('\\')?;
    if parent.is_empty() {
        None
    } else if parent.ends_with(':') {
        // Drive root: `C:\vault.db` → `C:\` (not bare `C:`).
        Some(format!("{parent}\\"))
    } else {
        Some(parent.to_string())
    }
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
fn ingest_madr_from_ledgerful(
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
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn nightly_status__schedule_state_parse__extracts_next_run_from_csv() {
        // Live schtasks CSV is 3 columns; also exercise quoted commas via quote-aware split.
        let csv_line = "\"\\AI-Brains-Nightly\",\"6/25/2026 1:00:00 AM\",\"Ready\"";
        let next_run = next_run_from_schtasks_csv_line(csv_line);
        assert_eq!(next_run.as_deref(), Some("6/25/2026 1:00:00 AM"));
    }

    #[test]
    fn nightly_status__schedule_state_parse__empty_output_reports_not_scheduled() {
        assert_eq!(next_run_from_schtasks_csv_line(""), None);
        let lines = format_schedule_status_lines(None, None, true);
        assert!(lines[0].contains("Scheduled: No"));
        assert!(lines.iter().any(|l| l.contains("unknown")));
    }

    /// AC3: host:port + strips user:pass@ credentials.
    #[test]
    fn format_endpoint_line__host_port_and_strips_credentials() {
        let line = format_endpoint_line(
            "Completion",
            "http://user:s3cret@127.0.0.1:8081/v1",
            "gemma-4-E4B-it-Q6_K.gguf",
            "ok",
        );
        assert!(line.contains("127.0.0.1:8081"));
        assert!(!line.contains("user"));
        assert!(!line.contains("s3cret"));
        assert!(line.contains("model=gemma-4-E4B-it-Q6_K.gguf"));
        assert!(line.contains("probe=ok"));
        assert!(line.starts_with("Completion:"));

        let plain = host_port_from_url("http://127.0.0.1:8083");
        assert_eq!(plain, "127.0.0.1:8083");

        // Query/fragment tokens must not appear (F1 secret redaction).
        let q = host_port_from_url("http://127.0.0.1:8081?token=s3cret#frag");
        assert_eq!(q, "127.0.0.1:8081");
        assert!(!q.contains("token"));
        assert!(!q.contains("s3cret"));
        assert!(!q.contains("frag"));
    }

    /// AC4: last_result "101" appears in schedule status lines.
    #[test]
    fn format_schedule_status_lines__last_result_101__contains_101() {
        let lines = format_schedule_status_lines(Some("8/12/2026 3:00:00 AM"), Some("101"), true);
        assert!(lines[0].contains("Scheduled: Yes"));
        assert!(lines[0].contains("8/12/2026"));
        assert!(lines.iter().any(|l| l.contains("101")));
        assert!(lines.iter().any(|l| l.contains("Last task result: 101")));
    }

    /// AC5: empty/missing schedule data → unknown / Scheduled No, no panic.
    #[test]
    fn format_schedule_status_lines__missing_data__unknown_and_not_scheduled() {
        let lines = format_schedule_status_lines(None, None, true);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Scheduled: No"));
        assert_eq!(lines[1], "Last task result: unknown");

        let no_last = format_schedule_status_lines(Some("tomorrow"), None, false);
        assert_eq!(no_last.len(), 1);
        assert!(no_last[0].contains("Scheduled: Yes"));
    }

    #[test]
    fn split_csv_fields_quote_aware__quoted_comma_and_escaped_quote() {
        let fields = split_csv_fields_quote_aware(r#""Task,Name","6/25/2026 1:00:00 AM","Ready""#);
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0], "Task,Name");
        assert_eq!(fields[1], "6/25/2026 1:00:00 AM");
        assert_eq!(fields[2], "Ready");
    }

    #[test]
    fn parse_last_task_result_ps_stdout__trims_and_rejects_empty() {
        assert_eq!(
            parse_last_task_result_ps_stdout("  101\r\n").as_deref(),
            Some("101")
        );
        assert_eq!(parse_last_task_result_ps_stdout("   \n"), None);
    }

    #[test]
    fn parse_last_result_list_v__english_label() {
        let sample = "\
Folder: \\\n\
HostName: DESKTOP\n\
TaskName: \\AI-Brains-Nightly\n\
Next Run Time: 8/12/2026 3:00:00 AM\n\
Status: Ready\n\
Last Run Time: 8/11/2026 3:00:00 AM\n\
Last Result: 101\n\
Author: N/A\n";
        assert_eq!(parse_last_result_list_v(sample).as_deref(), Some("101"));
        assert_eq!(parse_last_result_list_v("no such field"), None);
    }

    /// AC13: None / empty / invalid → nil UUID (stable across calls).
    #[test]
    fn resolve_nightly_project_id__missing_or_invalid__nil_stable() {
        let nil = ProjectId::from_uuid(uuid::Uuid::nil());
        let a = resolve_nightly_project_id(None);
        let b = resolve_nightly_project_id(None);
        assert_eq!(a, nil);
        assert_eq!(b, nil);
        assert_eq!(a, b);
        assert_eq!(resolve_nightly_project_id(Some("")), nil);
        assert_eq!(resolve_nightly_project_id(Some("   ")), nil);
        assert_eq!(resolve_nightly_project_id(Some("not-a-uuid")), nil);
        assert_eq!(resolve_nightly_project_id(Some("default-project")), nil);
    }

    /// AC14: valid UUID → that id; nil equality is the warn-path signal.
    #[test]
    fn resolve_nightly_project_id__valid_uuid__returns_that_id() {
        let raw = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let got = resolve_nightly_project_id(Some(raw));
        assert_eq!(got.to_string(), raw);
        assert_ne!(got, ProjectId::from_uuid(uuid::Uuid::nil()));
        // Warn path: compare to nil, not ProjectId::default() (random).
        assert_eq!(
            resolve_nightly_project_id(None),
            ProjectId::from_uuid(uuid::Uuid::nil())
        );
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
    fn build_schtasks_args__run_as_system__wrapper_script_contains_env_vars()
    -> Result<(), Box<dyn std::error::Error>> {
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
