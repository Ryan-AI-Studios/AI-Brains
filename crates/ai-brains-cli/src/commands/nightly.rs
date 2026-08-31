use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_models::llama_cpp::{LlamaCppProvider, ProbeStatus};
use ai_brains_store::EventStore;
use std::io::IsTerminal;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// Soft probe timeout for nightly **run** pre-summarize (independent of 120s LLM timeout).
const NIGHTLY_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
/// Interactive `nightly --status` probe timeout (parallel; Windows closed-loopback may wait the full window).
const NIGHTLY_STATUS_PROBE_TIMEOUT: Duration = Duration::from_millis(750);

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
    skip_import_claude: bool,
    skip_import_codex: bool,
    skip_import_cursor: bool,
    skip_graduation: bool,
    graduation_dry_run: bool,
    run_as_system: bool,
    dry_run: bool,
    quick: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let task_name = "AI-Brains-Nightly";

    if status {
        let query_store = ctx.conn.clone() as Arc<dyn ai_brains_store::QueryStore>;
        let unsummarized = query_store.get_unsummarized_sessions()?;
        let last_run = query_store.get_last_nightly_run()?;
        let last_count_raw = query_store.get_sync_state("last_nightly_count")?;
        let last_errors_raw = query_store.get_sync_state("last_nightly_errors")?;

        let (model_url, completion_model, embedding_url, embedding_model) =
            resolve_nightly_model_endpoints();

        // Soft probes — never fail status (exit 0 even when down / timeout / skipped).
        // `--quick` must not construct LlamaCppProvider or call probe_health (F19).
        let (completion_label, embedding_label) = if quick {
            ("skipped", "skipped")
        } else {
            let (c, e) = tokio::join!(
                async {
                    let p = LlamaCppProvider::new(model_url.clone(), completion_model.clone());
                    p.probe_health(NIGHTLY_STATUS_PROBE_TIMEOUT).await
                },
                async {
                    let p = LlamaCppProvider::new(embedding_url.clone(), embedding_model.clone());
                    p.probe_health(NIGHTLY_STATUS_PROBE_TIMEOUT).await
                },
            );
            (c.as_label(), e.as_label())
        };

        let resolved = crate::commands::nightly_status::resolve_nightly_status_format(
            &format,
            std::io::stdout().is_terminal(),
        );

        #[cfg(windows)]
        let nightly_sched = fetch_schedule_snapshot(task_name);
        #[cfg(windows)]
        let router_sched = fetch_schedule_snapshot("AI-Brains-Router");

        #[cfg(windows)]
        let task_to_run = nightly_sched.snap.task_to_run.clone();
        #[cfg(not(windows))]
        let task_to_run: Option<String> = None;

        let multi_import =
            match crate::commands::multi_import::load_multi_import_status(query_store.as_ref()) {
                Ok(view) => view,
                Err(e) => {
                    tracing::warn!(error = %e, "failed to load last_multi_import (non-fatal)");
                    crate::commands::multi_import::MultiImportStatusView::Unreadable
                }
            };

        if resolved == "json" {
            let action_target = task_to_run.as_deref().and_then(first_quoted_action_target);
            let action_target_missing = action_target
                .as_ref()
                .is_some_and(|p| !std::path::Path::new(p).exists());
            let next_step = if action_target_missing {
                Some("ai-brains nightly --schedule --dry-run".to_string())
            } else {
                None
            };

            #[cfg(windows)]
            let scheduled = Some(nightly_sched.snap.next_run.is_some());
            #[cfg(not(windows))]
            let scheduled = None;
            #[cfg(windows)]
            let next_run = nightly_sched.snap.next_run.clone();
            #[cfg(not(windows))]
            let next_run = None;
            #[cfg(windows)]
            let last_task_result = nightly_sched.snap.last_result.clone();
            #[cfg(not(windows))]
            let last_task_result = None;
            #[cfg(windows)]
            let last_scheduled_run = nightly_sched.snap.last_run_time.clone();
            #[cfg(not(windows))]
            let last_scheduled_run = None;

            #[cfg(windows)]
            let router = Some(crate::commands::nightly_status::RouterStatusInput {
                found: router_sched.found,
                status: router_sched.snap.status.clone(),
                last_result: router_sched.snap.last_result.clone(),
                task_to_run: router_sched.snap.task_to_run.clone(),
            });
            #[cfg(not(windows))]
            let router = None;

            let input = crate::commands::nightly_status::NightlyStatusInput {
                scheduled,
                next_run,
                last_task_result,
                last_scheduled_run,
                action_target,
                action_target_missing,
                next_step,
                last_nightly_run: last_run,
                unsummarized_sessions: unsummarized.len(),
                last_count_raw,
                last_errors_raw,
                completion_url: model_url,
                completion_model,
                completion_probe: completion_label.to_string(),
                embedding_url,
                embedding_model,
                embedding_probe: embedding_label.to_string(),
                multi_import,
                router,
            };
            let status_json = crate::commands::nightly_status::build_nightly_status_json(input);
            crate::commands::identity_warn::print_json_stdout(&status_json)?;
            return Ok(());
        }

        println!("=== Nightly Status ===");
        println!("{}", crate::commands::nightly_status::NIGHTLY_TASK_HEADING);
        #[cfg(windows)]
        {
            for line in format_status_schedule_block(
                nightly_sched.snap.next_run.as_deref(),
                nightly_sched.snap.last_result.as_deref(),
                nightly_sched.snap.last_run_time.as_deref(),
                nightly_sched.snap.task_to_run.as_deref(),
                true,
            ) {
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
        println!(
            "Sessions summarized in last run: {}",
            last_count_raw.as_deref().unwrap_or("0")
        );
        println!(
            "Errors in last run: {}",
            last_errors_raw.as_deref().unwrap_or("[]")
        );
        let embedding_human = crate::commands::nightly_status::format_probe_label_human(
            embedding_label,
            NIGHTLY_STATUS_PROBE_TIMEOUT.as_millis(),
        );
        for line in crate::commands::nightly_status::completion_status_human_lines(
            &model_url,
            &completion_model,
            completion_label,
            NIGHTLY_STATUS_PROBE_TIMEOUT.as_millis(),
        ) {
            println!("{line}");
        }
        println!(
            "{}",
            format_endpoint_line(
                "Embedding",
                &embedding_url,
                &embedding_model,
                &embedding_human,
            )
        );
        #[cfg(windows)]
        {
            for line in crate::commands::nightly_status::format_router_status_lines(
                router_sched.found,
                router_sched.snap.status.as_deref(),
                router_sched.snap.last_result.as_deref(),
            ) {
                println!("{line}");
            }
        }
        crate::commands::multi_import::print_multi_import_status(&multi_import);
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
            let cmd = format!("'{}' nightly", exe_str);
            if dry_run {
                let args = build_schtasks_args(&cmd, task_name, &start_time, false);
                println!("{}", format_schedule_dry_run_preview(&args));
                return Ok(());
            }
            cmd
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

    // T239/T334: multi-harness import (agy → grok → opencode → claude → codex → cursor)
    // before summarization.
    // Fail-open per source; SYSTEM scheduled nightly keeps --skip-import (D12).
    {
        use crate::commands::multi_import::{
            MultiImportOptions, persist_multi_import_report, run_multi_harness_import,
        };
        if skip_import {
            tracing::info!(
                "Skipping multi-harness session import (--skip-import). \
                 Skips AGY, Grok, OpenCode, Claude, Codex, and Cursor batch importers. \
                 Use this on isolated, CI, SYSTEM-scheduled, or per-project vaults \
                 to prevent cross-vault contamination from real harness history."
            );
        } else if skip_import_agy
            || skip_import_grok
            || skip_import_opencode
            || skip_import_claude
            || skip_import_codex
            || skip_import_cursor
        {
            tracing::info!(
                skip_agy = skip_import_agy,
                skip_grok = skip_import_grok,
                skip_opencode = skip_import_opencode,
                skip_claude = skip_import_claude,
                skip_codex = skip_import_codex,
                skip_cursor = skip_import_cursor,
                "Multi-harness import with per-source skip flags"
            );
        }
        let opts = MultiImportOptions::production(
            skip_import,
            skip_import_agy,
            skip_import_grok,
            skip_import_opencode,
            skip_import_claude,
            skip_import_codex,
            skip_import_cursor,
        );
        let report = run_multi_harness_import(ctx, opts);
        let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
        persist_multi_import_report(&store, &report);
        tracing::info!(
            agy = %report.agy.status,
            grok = %report.grok.status,
            opencode = %report.opencode.status,
            claude = %report.claude.status,
            codex = %report.codex.status,
            cursor = %report.cursor.status,
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
    let graduation = if skip_graduation {
        ai_brains_brain::GraduationMode::Skip
    } else if graduation_dry_run {
        ai_brains_brain::GraduationMode::DryRun
    } else {
        ai_brains_brain::GraduationMode::Run
    };
    let count = service
        .run_nightly_with(project_id, batch_size, graduation)
        .await?;
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

    // --- Phase 2 multi-root bridge (T233): path aliases = SOOT ---
    // Phase 1 above used env project_id (nil SOOT T229) for summarize only.
    run_phase2_multi_root_bridge(ctx)?;

    Ok(())
}

/// AC7 zero-alias user hint (must mention `register-path`).
pub(crate) const PHASE2_ZERO_ALIAS_HINT: &str = "\
[Nightly] Phase 2 skipped: no path aliases. Register roots with:\n  ai-brains project register-path <project_id|alias> <path>";

/// Whether a Phase 2 root path exists on disk (AC5 skip vs bridge).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase2RootStatus {
    Ok,
    Missing,
}

/// Classify a registered path alias root for Phase 2 (pure; hermetic tests).
pub(crate) fn phase2_root_status(path: &str) -> Phase2RootStatus {
    if std::path::Path::new(path).exists() {
        Phase2RootStatus::Ok
    } else {
        Phase2RootStatus::Missing
    }
}

/// One Phase 2 path-alias row: `(project_id, normalized_path)`.
pub(crate) type Phase2Alias = (ProjectId, String);

/// Partition aliases into existing roots vs missing (skip) paths (AC5).
pub(crate) fn filter_existing_roots(
    aliases: Vec<Phase2Alias>,
) -> (Vec<Phase2Alias>, Vec<Phase2Alias>) {
    let mut existing = Vec::new();
    let mut missing = Vec::new();
    for (id, path) in aliases {
        match phase2_root_status(&path) {
            Phase2RootStatus::Ok => existing.push((id, path)),
            Phase2RootStatus::Missing => missing.push((id, path)),
        }
    }
    (existing, missing)
}

/// Phase 2 root accounting after MAX_ROOTS truncate (T254 F6 / AC13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Phase2RootCounts {
    pub total: usize,
    pub ok: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// Account one existing-root symbol result. Missing roots are counted separately as skipped.
pub(crate) fn account_phase2_symbol_result(counts: &mut Phase2RootCounts, symbol_ok: bool) {
    if symbol_ok {
        counts.ok += 1;
    } else {
        counts.failed += 1;
    }
}

/// Shared invoke plan for ledgerful CLI with explicit root cwd (AC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InvokePlan {
    pub cwd: std::path::PathBuf,
    pub args: Vec<String>,
}

/// MADR bridge export plan: `current_dir = root` (never Task Scheduler System32).
pub(crate) fn madr_export_invoke_plan(root: &std::path::Path, out: &std::path::Path) -> InvokePlan {
    InvokePlan {
        cwd: root.to_path_buf(),
        args: vec![
            "bridge".to_string(),
            "export".to_string(),
            "--out".to_string(),
            out.to_string_lossy().into_owned(),
            "--ledger".to_string(),
        ],
    }
}

/// Phase 2: foreach registered path alias, MADR + symbols with explicit root.
fn run_phase2_multi_root_bridge(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_store::QueryStore;
    use std::path::PathBuf;

    let mut aliases = ctx.conn.list_path_aliases()?;
    if aliases.is_empty() {
        tracing::info!(
            "[Nightly] Phase 2: no path aliases registered; bridge no-op (register-path)"
        );
        eprintln!("{PHASE2_ZERO_ALIAS_HINT}");
        return Ok(());
    }

    // F28: already ORDER BY normalized_path ASC from store.
    if let Ok(max_s) = std::env::var("AI_BRAINS_NIGHTLY_MAX_ROOTS")
        && let Ok(max_n) = max_s.parse::<usize>()
        && max_n > 0
        && aliases.len() > max_n
    {
        tracing::info!(
            total = aliases.len(),
            max = max_n,
            "[Nightly] Phase 2: truncating roots via AI_BRAINS_NIGHTLY_MAX_ROOTS"
        );
        aliases.truncate(max_n);
    }

    let total = aliases.len();
    let (existing, missing) = filter_existing_roots(aliases);
    let roots_skipped = missing.len();
    let mut counts = Phase2RootCounts {
        total,
        ok: 0,
        skipped: roots_skipped,
        failed: 0,
    };

    tracing::info!(
        bridge_roots_total = total,
        "[Nightly] Phase 2 multi-root bridge starting"
    );

    for (alias_project_id, normalized_path) in missing {
        tracing::warn!(
            path = %normalized_path,
            project_id = %alias_project_id,
            "[Nightly] Phase 2: root missing; skip"
        );
    }

    for (alias_project_id, normalized_path) in existing {
        let root = PathBuf::from(&normalized_path);

        tracing::info!(
            path = %normalized_path,
            project_id = %alias_project_id,
            "[Nightly] Phase 2: bridging root"
        );

        // MADR per root (F27); empty BridgeRecord.project_id → alias owner (F12).
        if let Err(e) = ingest_madr_from_ledgerful(ctx, alias_project_id, &root) {
            tracing::warn!(
                path = %normalized_path,
                error = %e,
                "[Nightly] MADR ingestion failed for root (non-fatal; continue)"
            );
        }

        match crate::commands::symbol_bridge::ingest_symbols_from_ledgerful(
            ctx,
            alias_project_id,
            &root,
        ) {
            Ok(n) => {
                tracing::info!(
                    path = %normalized_path,
                    symbols = n,
                    "[Nightly] symbols ingested for root"
                );
                account_phase2_symbol_result(&mut counts, true);
            }
            Err(e) => {
                tracing::warn!(
                    path = %normalized_path,
                    error = %e,
                    "[Nightly] Symbol ingestion failed for root (non-fatal; continue)"
                );
                account_phase2_symbol_result(&mut counts, false);
            }
        }
    }

    tracing::info!(
        bridge_roots_total = counts.total,
        bridge_roots_ok = counts.ok,
        bridge_roots_skipped = counts.skipped,
        bridge_roots_failed = counts.failed,
        "[Nightly] Phase 2 multi-root bridge complete"
    );
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
/// Production callers are `cfg(windows)`; pure helpers stay available for unit tests on all OSes.
#[cfg_attr(not(windows), allow(dead_code))]
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
#[cfg_attr(not(windows), allow(dead_code))]
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
#[cfg_attr(not(windows), allow(dead_code))]
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
        if let Some(hint) = explain_last_task_result(label) {
            lines.push(hint.to_string());
        }
    }
    lines
}

/// Windows status schedule block: Scheduled → Last task result → hint? → Last scheduled run → action missing?
///
/// Vault `Last nightly run:` is printed by the caller after this block (F5).
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn format_status_schedule_block(
    next_run: Option<&str>,
    last_result: Option<&str>,
    last_run_time: Option<&str>,
    task_to_run: Option<&str>,
    include_last_result: bool,
) -> Vec<String> {
    let mut lines = format_schedule_status_lines(next_run, last_result, include_last_result);
    if let Some(t) = last_run_time.map(str::trim).filter(|s| !s.is_empty()) {
        lines.push(format!("Last scheduled run: {t}"));
    }
    lines.extend(format_status_action_missing(task_to_run));
    lines
}

/// Parsed English `schtasks /FO LIST /V` fields (T247 F3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SchtasksListV {
    pub next_run: Option<String>,
    pub last_run_time: Option<String>,
    pub last_result: Option<String>,
    pub task_to_run: Option<String>,
    pub status: Option<String>,
}

/// Skip empty / `N/A` / `"N/A"` LIST /V values.
fn list_v_usable_value(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    let unquoted = t.trim_matches('"').trim();
    if unquoted.eq_ignore_ascii_case("n/a") {
        return None;
    }
    Some(t.to_string())
}

/// Parse English `schtasks /query /fo LIST /v` stdout into one struct.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn parse_schtasks_list_v(stdout: &str) -> SchtasksListV {
    let mut parsed = SchtasksListV::default();
    for line in stdout.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = list_v_usable_value(value);
        match key.as_str() {
            "next run time" => parsed.next_run = value,
            "last run time" => parsed.last_run_time = value,
            "last result" => parsed.last_result = value,
            "task to run" => parsed.task_to_run = value,
            "status" => parsed.status = value,
            _ => {}
        }
    }
    parsed
}

/// Operator hint for a Task Scheduler Last Result code (F4).
///
/// Hint is a **following line**, not a suffix on `Last task result: N`.
pub(crate) fn explain_last_task_result(raw: &str) -> Option<&'static str> {
    let s = raw.trim();
    let code = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        s.parse::<u32>().ok()?
    };
    match code {
        0 => None,
        1 => Some("process failed / missing action / CLI error"),
        101 => Some("Rust panic/abort"),
        267009 => Some("task still running (SCHED_S_TASK_RUNNING)"),
        267014 => Some("task terminated (SCHED_S_TASK_TERMINATED)"),
        _ => None,
    }
}

/// First quoted `.cmd` / `.bat` / `.exe` token from Task To Run (F6).
///
/// Accepts `"` or `'` quotes; unquoted / non-script tokens → `None`.
pub(crate) fn first_quoted_action_target(task_to_run: &str) -> Option<String> {
    let quote_pos = task_to_run.find(['"', '\''])?;
    let quote = task_to_run.as_bytes().get(quote_pos).copied()?;
    let rest = task_to_run.get(quote_pos + 1..)?;
    let end = rest.find(quote as char)?;
    let token = rest.get(..end)?;
    if token.is_empty() {
        return None;
    }
    let lower = token.to_ascii_lowercase();
    if lower.ends_with(".cmd") || lower.ends_with(".bat") || lower.ends_with(".exe") {
        Some(token.to_string())
    } else {
        None
    }
}

/// Missing-action status lines (F6). Existing product exe + args → no lines.
pub(crate) fn format_status_action_missing(task_to_run: Option<&str>) -> Vec<String> {
    let Some(raw) = task_to_run.map(str::trim).filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    let Some(path) = first_quoted_action_target(raw) else {
        return Vec::new();
    };
    if std::path::Path::new(&path).exists() {
        return Vec::new();
    }
    vec![
        format!("Action target missing: {path}"),
        "next: ai-brains nightly --schedule --dry-run".to_string(),
    ]
}

/// Parse PowerShell `LastTaskResult` stdout (trim; empty → None).
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn parse_last_task_result_ps_stdout(stdout: &str) -> Option<String> {
    let t = stdout.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Soft parse of English `schtasks /FO LIST /V` `Last Result:` line (locale-sensitive fallback).
/// T247 production uses `parse_schtasks_list_v`; this helper stays for T229 unit coverage.
#[allow(dead_code)]
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

/// LIST /V foundness (F34): `found` is true only when the spawn succeeds.
/// Visibility `pub(crate)` for T320 status glance (behavior unchanged).
#[cfg(windows)]
pub(crate) struct ScheduleSnapshot {
    pub(crate) found: bool,
    pub(crate) snap: SchtasksListV,
}

/// Windows schedule snapshot: LIST /V first; PS Last Result only after successful LIST /V
/// with a missing `last_result`; CSV next-run only when LIST /V missed `next_run`.
/// Non-zero LIST /V (task missing) → `found: false`, all None, **no** PowerShell (F3/F34).
/// Visibility `pub(crate)` for T320 status glance (behavior unchanged).
#[cfg(windows)]
pub(crate) fn fetch_schedule_snapshot(task_name: &str) -> ScheduleSnapshot {
    let list = std::process::Command::new("schtasks")
        .args(["/query", "/tn", task_name, "/fo", "LIST", "/v"])
        .output();
    match list {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut parsed = parse_schtasks_list_v(&stdout);
            if parsed.last_result.is_none() {
                parsed.last_result = fetch_last_task_result_ps(task_name);
            }
            if parsed.next_run.is_none() {
                parsed.next_run = fetch_schedule_next_run(task_name);
            }
            ScheduleSnapshot {
                found: true,
                snap: parsed,
            }
        }
        _ => ScheduleSnapshot {
            found: false,
            snap: SchtasksListV::default(),
        },
    }
}

/// Last Result fallback: `Get-ScheduledTaskInfo` only (locale miss after successful LIST /V).
#[cfg(windows)]
fn fetch_last_task_result_ps(task_name: &str) -> Option<String> {
    let ps_cmd = format!("(Get-ScheduledTaskInfo -TaskName '{task_name}').LastTaskResult");
    let ps = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output();
    if let Ok(out) = ps
        && out.status.success()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        return parse_last_task_result_ps_stdout(&stdout);
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

fn format_schedule_dry_run_preview(args: &[String]) -> String {
    format!("[dry-run] Would execute:\n  schtasks {}", args.join(" "))
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
        r#""{}" --no-project-context nightly --skip-import --skip-graduation --log-format json"#,
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
///
/// `root` is the registered path-alias directory; Ledgerful is spawned with
/// `.current_dir(root)` so Task Scheduler System32 cwd cannot zero the export.
/// Empty `BridgeRecord.project_id` falls back to `alias_project_id` (F12).
fn ingest_madr_from_ledgerful(
    ctx: &AppContext,
    alias_project_id: ProjectId,
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_contracts::bridge::BridgeRecord;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let temp_path = {
        let mut p = std::env::temp_dir();
        // Per-root temp name avoids clobber when multi-root (sequential still safer).
        let safe = alias_project_id.to_string().replace('-', "");
        p.push(format!("cg_madr_export_{safe}.ndjson"));
        p
    };

    // Call Ledgerful bridge export --ledger with explicit root (T233 F9/F27).
    let plan = madr_export_invoke_plan(root, &temp_path);
    let output = std::process::Command::new("ledgerful")
        .current_dir(&plan.cwd)
        .args(&plan.args)
        .output();

    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(
                root = %root.display(),
                "Ledgerful bridge export failed: {}",
                stderr
            );
            return Ok(()); // Non-fatal: fail gracefully
        }
        Err(e) => {
            tracing::warn!(
                root = %root.display(),
                "Ledgerful CLI not available: {}",
                e
            );
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

        // Parse record-level IDs. F12: empty BridgeRecord.project_id → alias owner
        // (not Phase-1 env/nil alone).
        let record_project_id = if !record.project_id.is_empty() {
            ProjectId::from_str(&record.project_id).ok()
        } else {
            None
        };
        let effective_project_id = record_project_id.unwrap_or(alias_project_id);
        let record_session_id = record
            .session_id
            .as_ref()
            .and_then(|s| ai_brains_core::ids::SessionId::from_str(s).ok());
        let tx_id_raw = record.tx_id.as_deref().filter(|s| !s.is_empty());
        let tx_id = tx_id_raw.map(|s| ai_brains_core::ids::TransactionId::new(s.to_string()));

        // T233 Codex R2 P2: stable decision aggregate id (tx preferred, else content)
        // so dual path-aliases (Win+WSL) and re-runs do not append duplicate DecisionRecorded.
        let decision_id =
            madr_stable_decision_id(effective_project_id, tx_id_raw, &title, &decision, &context);
        if madr_decision_already_ingested(&event_store, decision_id.as_uuid()) {
            continue;
        }

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
                project_id: Some(effective_project_id),
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

/// Stable MemoryId for a MADR decision (T233 multi-root / re-run idempotency).
///
/// Prefer `tx_id` when present; otherwise hash title+decision+context under the
/// effective project. Dual Win/WSL path aliases for the same project therefore
/// share one aggregate and skip on second root / second nightly.
pub(crate) fn madr_stable_decision_id(
    project_id: ProjectId,
    tx_id: Option<&str>,
    title: &str,
    decision: &str,
    context: &str,
) -> MemoryId {
    let key = match tx_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(tx) => format!("madr:{}:tx:{}", project_id, tx),
        None => format!(
            "madr:{}:content:{}|{}|{}",
            project_id, title, decision, context
        ),
    };
    MemoryId::from_uuid(uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        key.as_bytes(),
    ))
}

/// True when the decision aggregate already has events (prior DecisionRecorded).
fn madr_decision_already_ingested(event_store: &dyn EventStore, decision_uuid: uuid::Uuid) -> bool {
    event_store
        .read_events(decision_uuid)
        .map(|events| !events.is_empty())
        .unwrap_or(false)
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
    fn parse_schtasks_list_v__english_fixture__extracts_next_last_result_and_action() {
        let sample = "\
Folder: \\\n\
HostName: DESKTOP\n\
TaskName: \\AI-Brains-Nightly\n\
Next Run Time: 8/14/2026 3:00:00 AM\n\
Status: Ready\n\
Logon Mode: Interactive only\n\
Last Run Time: 8/13/2026 3:00:01 AM\n\
Last Result: 1\n\
Author: N/A\n\
Task To Run: \"C:\\Users\\RyanB\\.ai-brains\\nightly-run.cmd\"\n\
Start In: N/A\n";
        let parsed = parse_schtasks_list_v(sample);
        assert_eq!(parsed.next_run.as_deref(), Some("8/14/2026 3:00:00 AM"));
        assert_eq!(
            parsed.last_run_time.as_deref(),
            Some("8/13/2026 3:00:01 AM")
        );
        assert_eq!(parsed.last_result.as_deref(), Some("1"));
        assert_eq!(
            parsed.task_to_run.as_deref(),
            Some(r#""C:\Users\RyanB\.ai-brains\nightly-run.cmd""#)
        );
        assert_eq!(parsed.status.as_deref(), Some("Ready"));
    }

    #[test]
    fn parse_schtasks_list_v__english_fixture__extracts_status_running() {
        let sample = "\
TaskName: \\AI-Brains-Router\n\
Next Run Time: N/A\n\
Status: Running\n\
Last Run Time: 8/16/2026 8:00:00 AM\n\
Last Result: 267009\n\
Task To Run: C:\\llm\\router.bat\n";
        let parsed = parse_schtasks_list_v(sample);
        assert_eq!(parsed.status.as_deref(), Some("Running"));
        assert_eq!(parsed.last_result.as_deref(), Some("267009"));
        assert_eq!(parsed.task_to_run.as_deref(), Some(r"C:\llm\router.bat"));
        assert_eq!(parsed.next_run, None);
    }

    #[test]
    fn parse_schtasks_list_v__missing_english_labels__fields_none() {
        let garbled = "\
Ordner: \\\n\
Nächster Start: 14.08.2026 03:00:00\n\
Letztes Ergebnis: 1\n\
Auszuführende Aufgabe: C:\\x.cmd\n";
        let parsed = parse_schtasks_list_v(garbled);
        assert_eq!(parsed.next_run, None);
        assert_eq!(parsed.last_run_time, None);
        assert_eq!(parsed.last_result, None);
        assert_eq!(parsed.task_to_run, None);

        let na = "\
Next Run Time: N/A\n\
Last Run Time: \"N/A\"\n\
Last Result: N/A\n\
Task To Run: N/A\n";
        assert_eq!(parse_schtasks_list_v(na), SchtasksListV::default());
    }

    #[test]
    fn explain_last_task_result__0__no_hint() {
        assert_eq!(explain_last_task_result("0"), None);
        assert_eq!(explain_last_task_result("  0  "), None);
    }

    #[test]
    fn explain_last_task_result__1__mentions_fail_or_missing() {
        let hint = match explain_last_task_result("1") {
            Some(h) => h,
            None => panic!("expected hint for Last Result 1"),
        };
        let lower = hint.to_ascii_lowercase();
        assert!(
            lower.contains("fail") || lower.contains("missing"),
            "hint must mention fail/missing: {hint}"
        );
    }

    #[test]
    fn explain_last_task_result__101__mentions_panic() {
        let hint = match explain_last_task_result("101") {
            Some(h) => h,
            None => panic!("expected hint for Last Result 101"),
        };
        assert!(
            hint.to_ascii_lowercase().contains("panic"),
            "hint must mention panic: {hint}"
        );
    }

    #[test]
    fn explain_last_task_result__0x65__equiv_101() {
        assert_eq!(
            explain_last_task_result("0x65"),
            explain_last_task_result("101")
        );
        assert_eq!(
            explain_last_task_result("0X65"),
            explain_last_task_result("101")
        );
    }

    #[test]
    fn explain_last_task_result__267009__running_sched_s() {
        let hint = match explain_last_task_result("267009") {
            Some(h) => h,
            None => panic!("expected hint for 267009"),
        };
        assert!(
            hint.to_ascii_lowercase().contains("running"),
            "hint must mention running: {hint}"
        );
        assert!(
            hint.contains("SCHED_S_TASK_RUNNING"),
            "hint must mention SCHED_S_TASK_RUNNING: {hint}"
        );
    }

    #[test]
    fn explain_last_task_result__0x41301__equiv_267009() {
        assert_eq!(
            explain_last_task_result("0x41301"),
            explain_last_task_result("267009")
        );
    }

    #[test]
    fn explain_last_task_result__267014__terminated() {
        let hint = match explain_last_task_result("267014") {
            Some(h) => h,
            None => panic!("expected hint for 267014"),
        };
        assert!(
            hint.to_ascii_lowercase().contains("terminated"),
            "hint must mention terminated: {hint}"
        );
        assert!(
            hint.contains("SCHED_S_TASK_TERMINATED"),
            "hint must mention SCHED_S_TASK_TERMINATED: {hint}"
        );
        assert_eq!(
            explain_last_task_result("0x41306"),
            explain_last_task_result("267014")
        );
    }

    #[test]
    fn explain_last_task_result__99__none() {
        assert_eq!(explain_last_task_result("99"), None);
        assert_eq!(explain_last_task_result("not-a-code"), None);
    }

    #[test]
    fn format_status_schedule_block__order__result_hint_then_last_scheduled() {
        let lines = format_status_schedule_block(
            Some("8/14/2026 3:00:00 AM"),
            Some("101"),
            Some("8/13/2026 3:00:01 AM"),
            None,
            true,
        );
        assert_eq!(lines[0], "Scheduled: Yes (next run: 8/14/2026 3:00:00 AM)");
        assert_eq!(lines[1], "Last task result: 101");
        assert!(
            lines[2].to_ascii_lowercase().contains("panic"),
            "hint must be a following line: {lines:?}"
        );
        assert_eq!(lines[3], "Last scheduled run: 8/13/2026 3:00:01 AM");
    }

    #[test]
    fn format_status_action_missing__absent_cmd__next_step_dry_run() {
        let dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("tempdir: {e}"),
        };
        let missing = dir.path().join("nightly-run.cmd");
        let task = format!("\"{}\"", missing.display());
        let lines = format_status_action_missing(Some(&task));
        let joined = lines.join("\n");
        assert!(
            joined.contains("Action target missing:"),
            "expected missing line: {joined}"
        );
        assert!(
            joined.contains("nightly --schedule --dry-run"),
            "expected dry-run next step: {joined}"
        );
    }

    #[test]
    fn format_status_action_missing__product_exe_nightly__no_missing_line() {
        let dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => panic!("tempdir: {e}"),
        };
        let exe = dir.path().join("ai-brains.exe");
        if let Err(e) = std::fs::write(&exe, b"") {
            panic!("touch exe: {e}");
        }
        let task = format!("\"{}\" nightly", exe.display());
        let lines = format_status_action_missing(Some(&task));
        assert!(
            lines.is_empty(),
            "existing product exe must not report missing: {lines:?}"
        );
    }

    #[test]
    fn format_status_action_missing__unquoted_or_non_script__no_missing_line() {
        assert!(format_status_action_missing(Some(r"C:\missing\nightly-run.cmd")).is_empty());
        assert!(format_status_action_missing(Some(r#""C:\notes.txt""#)).is_empty());
        assert!(format_status_action_missing(None).is_empty());
    }

    #[test]
    fn format_endpoint_line__quick__probe_skipped() {
        let line = format_endpoint_line(
            "Completion",
            "http://127.0.0.1:8081",
            "gemma-4-E4B-it-Q6_K.gguf",
            "skipped",
        );
        assert!(
            line.contains("probe=skipped"),
            "string-literal skipped label: {line}"
        );
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

    // --- Phase 2 multi-root hermetics (T233-R2 / AC3, AC5, AC7) ---

    /// AC7: zero-alias message points operators at `register-path`.
    #[test]
    fn phase2_zero_alias_hint__contains_register_path() {
        assert!(
            PHASE2_ZERO_ALIAS_HINT.contains("register-path"),
            "hint must mention register-path: {PHASE2_ZERO_ALIAS_HINT}"
        );
        assert!(PHASE2_ZERO_ALIAS_HINT.contains("Phase 2"));
    }

    /// AC5: missing path roots are classified as skip.
    #[test]
    fn phase2_root_status__missing_path__missing() {
        let missing = r"C:\path\that\definitely\does\not\exist\ai-brains-t233-missing";
        assert_eq!(phase2_root_status(missing), Phase2RootStatus::Missing);
    }

    /// AC5: existing path roots are Ok.
    #[test]
    fn phase2_root_status__existing_tempdir__ok() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir
            .path()
            .to_str()
            .ok_or("temp path not utf-8")?
            .to_string();
        assert_eq!(phase2_root_status(&path), Phase2RootStatus::Ok);
        Ok(())
    }

    /// AC5: filter partitions existing vs missing roots.
    #[test]
    fn filter_existing_roots__partitions_missing_and_ok() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempfile::tempdir()?;
        let ok_path = dir
            .path()
            .to_str()
            .ok_or("temp path not utf-8")?
            .to_string();
        let missing_path =
            r"C:\path\that\definitely\does\not\exist\ai-brains-t233-filter".to_string();
        let id_ok = ProjectId::from_uuid(uuid::Uuid::from_u128(1));
        let id_miss = ProjectId::from_uuid(uuid::Uuid::from_u128(2));
        let (existing, missing) = filter_existing_roots(vec![
            (id_ok, ok_path.clone()),
            (id_miss, missing_path.clone()),
        ]);
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].0, id_ok);
        assert_eq!(existing[0].1, ok_path);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].0, id_miss);
        assert_eq!(missing[0].1, missing_path);
        Ok(())
    }

    /// AC13: symbol Ok / Err / missing-skip account for every considered root.
    #[test]
    fn phase2_root_counts__one_ok_one_failed_one_skipped__add_up() {
        let mut counts = Phase2RootCounts {
            total: 3,
            ok: 0,
            skipped: 1,
            failed: 0,
        };
        account_phase2_symbol_result(&mut counts, true);
        account_phase2_symbol_result(&mut counts, false);
        assert_eq!(counts.ok, 1);
        assert_eq!(counts.skipped, 1);
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.ok + counts.skipped + counts.failed, counts.total);
    }

    /// Codex R2 P2: same tx under same project → same decision id (dual alias / re-run).
    #[test]
    fn madr_stable_decision_id__same_tx__same_id() {
        let pid = ProjectId::from_uuid(uuid::Uuid::from_u128(42));
        let a = madr_stable_decision_id(pid, Some("tx-abc"), "t", "d", "c");
        let b = madr_stable_decision_id(pid, Some("tx-abc"), "other", "other", "other");
        assert_eq!(a, b, "tx_id identity must dominate content");
    }

    /// Codex R2 P2: content key stable without tx; different content → different id.
    #[test]
    fn madr_stable_decision_id__content_key__stable_and_distinct() {
        let pid = ProjectId::from_uuid(uuid::Uuid::from_u128(7));
        let a = madr_stable_decision_id(pid, None, "Title", "Decide X", "Ctx");
        let b = madr_stable_decision_id(pid, None, "Title", "Decide X", "Ctx");
        let c = madr_stable_decision_id(pid, None, "Title", "Decide Y", "Ctx");
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Different project → different id for same content.
        let other = ProjectId::from_uuid(uuid::Uuid::from_u128(8));
        let d = madr_stable_decision_id(other, None, "Title", "Decide X", "Ctx");
        assert_ne!(a, d);
    }

    /// AC3: MADR export plan pins cwd to root (not System32 / process cwd).
    #[test]
    fn madr_export_invoke_plan__cwd_is_root_and_args_ledger_export() {
        let root = std::path::PathBuf::from(r"C:\dev\example-root");
        let out = std::path::PathBuf::from(r"C:\temp\cg_madr_export.ndjson");
        let plan = madr_export_invoke_plan(&root, &out);
        assert_eq!(plan.cwd, root);
        assert!(plan.args.iter().any(|a| a == "bridge"));
        assert!(plan.args.iter().any(|a| a == "export"));
        assert!(plan.args.iter().any(|a| a == "--ledger"));
        assert!(plan.args.iter().any(|a| a == "--out"));
        let out_pos = plan
            .args
            .iter()
            .position(|a| a == "--out")
            .expect("--out present");
        let out_arg = plan.args.get(out_pos + 1).map(String::as_str);
        let expected = out.to_str().expect("utf-8 out path");
        assert_eq!(out_arg, Some(expected));
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
            r"C:\fake\ai-brains.exe --no-project-context nightly --skip-import --skip-graduation --log-format json",
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
        assert!(task_command.contains("--skip-graduation"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_schtasks_args__run_as_system__includes_skip_graduation() {
        let args = build_schtasks_args(
            r"C:\fake\ai-brains.exe --no-project-context nightly --skip-import --skip-graduation --log-format json",
            "AI-Brains-Nightly",
            "03:00",
            true,
        );
        let tr = args
            .iter()
            .position(|a| a == "/tr")
            .expect("/tr argument present");
        let task_command = &args[tr + 1];
        assert!(task_command.contains("--skip-graduation"));
        assert!(task_command.contains("--skip-import"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn user_principal_schedule_dry_run_preview__quoted_exe_nightly_no_ru() {
        let args = build_schtasks_args(
            r"'C:\fake\ai-brains.exe' nightly",
            "AI-Brains-Nightly",
            "03:00",
            false,
        );
        let preview = format_schedule_dry_run_preview(&args);
        assert!(preview.starts_with("[dry-run] Would execute:"), "{preview}");
        assert!(preview.contains("/tr"), "{preview}");
        assert!(
            preview.contains(r"'C:\fake\ai-brains.exe' nightly"),
            "{preview}"
        );
        assert!(!preview.contains("/ru"), "{preview}");
        assert!(preview.contains("/f"), "{preview}");
        assert!(preview.contains("/create"), "{preview}");
        assert!(
            !preview.contains("--skip-graduation"),
            "user-principal schedule must graduate; got {preview}"
        );
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
        assert!(!task_command.contains("--skip-graduation"));
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
        assert!(content.contains("--skip-graduation"));
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
