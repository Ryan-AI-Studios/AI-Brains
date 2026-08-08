use crate::commands::harness::{PromptDecision, interpret_consent_answer, should_prompt_install};
use crate::context::AppContext;
use crate::harness::{
    HarnessId, HarnessStatus, InstallOutcome, WiringStatus, collect_status_report, install_agy,
    load_prefs, resolve_home, save_prefs,
};
use ai_brains_contracts::preflight::PreflightContextResponse;
use ai_brains_core::ids::ProjectId;
use ai_brains_retrieval::build_preflight;
use ai_brains_store::QueryStore;
use is_terminal::IsTerminal;

pub struct PreflightRunOptions {
    pub max_words: usize,
    pub project_id: Option<ProjectId>,
    pub pretty: bool,
    pub format: Option<String>,
    pub scope: Vec<String>,
    pub summary: bool,
    pub global: bool,
    /// Never prompt for harness hook install (F24).
    pub no_hook_prompt: bool,
    /// Explicitly install ready harness hooks without interactive prompt (F24).
    pub install_hooks: bool,
    /// `preflight --stdin` mode: never prompt (F24 / AC18).
    pub stdin_mode: bool,
}

pub fn run(
    ctx: &AppContext,
    options: PreflightRunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new((*ctx.conn).clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let scope_paths = if options.scope.is_empty() {
        None
    } else {
        Some(normalize_scope_paths(&options.scope))
    };

    let context = build_preflight(
        &ctx.conn,
        graph_search.as_ref(),
        options.max_words,
        options.project_id,
        scope_paths,
        options.global,
    )?;

    if options.summary {
        print_summary(
            ctx,
            options.global,
            options.project_id,
            &context,
            PreflightHarnessGate {
                no_hook_prompt: options.no_hook_prompt,
                install_hooks: options.install_hooks,
                stdin_mode: options.stdin_mode,
            },
        )?;
        return Ok(());
    }

    // Smart defaulting: If stdout is a TTY and no format is specified, use human mode.
    let is_tty = std::io::stdout().is_terminal();
    let format_str = options.format.unwrap_or_else(|| {
        if is_tty {
            "human".to_string()
        } else {
            "json".to_string()
        }
    });

    let human_mode = options.pretty
        || format_str.eq_ignore_ascii_case("human")
        || format_str.eq_ignore_ascii_case("pretty");

    if human_mode {
        println!("{}", context.text);
    } else {
        let response = PreflightContextResponse {
            text: context.text,
            word_count: context.word_count,
        };
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(())
}

/// Build summary lines (no I/O). Dual count model (T214 F4):
///
/// 1. **Vault (SQL):** `Projects:` only when `global` + `projects_with_pinned` is
///    `Some`; always `Pinned memories` + `Active sessions`.
/// 2. **In context (budget window):** marker scan of rendered text — labels must
///    include the literal `"In context"` / `"In-context"` so they cannot be read
///    as vault totals.
///
/// Argument count is intentional: pure formatter mirrors the dual-block fields
/// one-for-one for unit-testability (T214 F4 / AC locks).
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_preflight_summary_lines(
    scope_line: &str,
    global: bool,
    projects_with_pinned: Option<u64>,
    pinned_memories: u64,
    active_sessions: u64,
    hotspot_count: usize,
    decision_count: usize,
    constraint_count: usize,
    word_count: usize,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::with_capacity(12);
    lines.push("--- AI-Brains Preflight Summary ---".to_string());
    lines.push(scope_line.to_string());
    // Vault block
    if global && let Some(n) = projects_with_pinned {
        lines.push(format!("Projects: {}", n));
    }
    lines.push(format!("Pinned memories: {}", pinned_memories));
    lines.push(format!("Active sessions: {}", active_sessions));
    // In-context block (AC5: literal "In context" prefix)
    lines.push(format!("In context hotspots: {}", hotspot_count));
    lines.push(format!("In context decisions: {}", decision_count));
    lines.push(format!("In context constraints: {}", constraint_count));
    lines.push(format!("Total Word Count: {}", word_count));
    lines.push(String::new());
    lines.push("Use --pretty or --format json for full context.".to_string());
    lines
}

/// Sibling pure formatter for harness summary lines (T235 F8 / AC19).
///
/// Header is exact: `Harnesses installed on machine:`
/// Returns empty vec when every harness is `absent`.
pub(crate) fn format_harness_summary_lines(statuses: &[HarnessStatus]) -> Vec<String> {
    let non_absent: Vec<&HarnessStatus> = statuses
        .iter()
        .filter(|h| h.wiring != WiringStatus::Absent)
        .collect();
    if non_absent.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::with_capacity(non_absent.len() + 3);
    lines.push(String::new());
    lines.push("Harnesses installed on machine:".to_string());
    for h in non_absent {
        let ready = if h.install_ready { "ready" } else { "pending" };
        lines.push(format!(
            "  {} wiring={} ({})",
            h.id,
            match h.wiring {
                WiringStatus::Missing => "missing",
                WiringStatus::Partial => "partial",
                WiringStatus::Ok => "ok",
                WiringStatus::BackendPending => "backend_pending",
                WiringStatus::Unknown => "unknown",
                WiringStatus::Absent => "absent",
            },
            ready
        ));
        if matches!(
            h.wiring,
            WiringStatus::Missing | WiringStatus::Partial | WiringStatus::Unknown
        ) {
            lines.push(format!("    next: {}", h.next_action));
        }
    }
    lines
}

struct PreflightHarnessGate {
    no_hook_prompt: bool,
    install_hooks: bool,
    stdin_mode: bool,
}

/// Print preflight summary with honest Scope + dual vault/in-context counts (T214 F37).
fn print_summary(
    ctx: &AppContext,
    global: bool,
    project_id: Option<ProjectId>,
    context: &ai_brains_retrieval::PreflightContext,
    gate: PreflightHarnessGate,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_alias = if !global {
        match project_id.as_ref() {
            Some(pid) => ctx.conn.get_project_by_id(pid)?,
            None => None,
        }
    } else {
        None
    };
    let scope_line =
        super::recall::format_scope_line(global, project_id.as_ref(), name_alias.as_ref());

    let (projects_with_pinned, pinned_memories, active_sessions) = if global {
        let projects = ctx.conn.count_projects_with_pinned()?;
        let pinned = ctx.conn.count_pinned_memories(None)?;
        let sessions = ctx.conn.count_active_sessions(None)?;
        (Some(projects), pinned, sessions)
    } else {
        let pid = project_id.as_ref();
        let pinned = ctx.conn.count_pinned_memories(pid)?;
        let sessions = ctx.conn.count_active_sessions(pid)?;
        (None, pinned, sessions)
    };

    // Marker scan of budget-window text (F6 / F32: case-sensitive as body).
    let text = &context.text;
    let hotspot_count = text.matches("HOTSPOT:").count();
    let decision_count = text.matches("DECISION:").count();
    let constraint_count = text.matches("CONSTRAINT:").count();

    let lines = format_preflight_summary_lines(
        &scope_line,
        global,
        projects_with_pinned,
        pinned_memories,
        active_sessions,
        hotspot_count,
        decision_count,
        constraint_count,
        context.word_count,
    );
    for line in lines {
        println!("{}", line);
    }

    // T235: harness sibling section + optional TTY consent (never grows T214 arity).
    append_harness_summary_and_maybe_prompt(&gate)?;
    Ok(())
}

fn append_harness_summary_and_maybe_prompt(
    gate: &PreflightHarnessGate,
) -> Result<(), Box<dyn std::error::Error>> {
    let home = resolve_home();
    let report = collect_status_report(home.as_deref());
    let harness_lines = format_harness_summary_lines(&report.harnesses);
    for line in &harness_lines {
        println!("{}", line);
    }
    if harness_lines.is_empty() {
        return Ok(());
    }

    let prefs = home.as_ref().map(|h| load_prefs(h)).unwrap_or_default();
    let declined_agy = prefs.is_declined(HarnessId::Agy);
    let ready_missing: Vec<&HarnessStatus> = report
        .harnesses
        .iter()
        .filter(|h| {
            h.install_ready
                && matches!(h.wiring, WiringStatus::Missing | WiringStatus::Partial)
                && !declined_agy
        })
        .collect();

    let is_tty = std::io::stdout().is_terminal() && std::io::stdin().is_terminal();
    let decision = should_prompt_install(
        is_tty,
        gate.no_hook_prompt,
        gate.stdin_mode,
        !ready_missing.is_empty(),
        declined_agy,
        prefs.auto_install,
    );

    // Explicit --install-hooks: install **ready backends that are present on machine**
    // only (F24). Never write AGY hooks when AGY is absent (Codex CX2 P2).
    // F20: parse-refuse / write failure on explicit install → exit 1 (not silent 0).
    if gate.install_hooks {
        let agy_present_ready = report.harnesses.iter().any(|h| {
            h.id == "agy"
                && h.present
                && h.install_ready
                && matches!(
                    h.wiring,
                    WiringStatus::Missing
                        | WiringStatus::Partial
                        | WiringStatus::BackendPending
                        | WiringStatus::Unknown
                )
        });
        let agy_already_ok = report
            .harnesses
            .iter()
            .any(|h| h.id == "agy" && h.present && matches!(h.wiring, WiringStatus::Ok));
        if let Some(h) = home.as_ref() {
            if agy_already_ok {
                println!("AGY capture hooks already installed. next: ai-brains harness status");
            } else if agy_present_ready {
                report_preflight_install(
                    install_agy(h, false),
                    "Installed ready harness hooks (agy). next: ai-brains harness status",
                    true, // explicit install: fail process on refuse/error
                )?;
            } else {
                println!(
                    "No ready harness present on machine for install-hooks (AGY absent or already ok). next: ai-brains harness status"
                );
            }
        }
        return Ok(());
    }

    match decision {
        PromptDecision::Skip => {}
        PromptDecision::PrintNextActionOnly => {
            if !ready_missing.is_empty() {
                println!("  next: ai-brains harness install --harness agy --dry-run");
            }
        }
        PromptDecision::AutoInstall => {
            if let Some(h) = home.as_ref() {
                // Soft path: print refuse/error but do not fail preflight (F9).
                let _ = report_preflight_install(
                    install_agy(h, false),
                    "Auto-installed AGY capture hooks (auto_install=true).",
                    false,
                );
            }
        }
        PromptDecision::AskOnce => {
            eprint!(
                "Install capture hooks for {}? [Y/n] ",
                ready_missing
                    .iter()
                    .map(|h| h.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            if interpret_consent_answer(&line) {
                if let Some(h) = home.as_ref() {
                    let _ = report_preflight_install(
                        install_agy(h, false),
                        "Installed AGY capture hooks.",
                        false,
                    );
                }
            } else if let Some(h) = home.as_ref() {
                let mut p = load_prefs(h);
                p.mark_declined(HarnessId::Agy, chrono::Utc::now().to_rfc3339());
                if let Err(e) = save_prefs(h, &p) {
                    eprintln!("could not persist decline: {e}");
                } else {
                    println!(
                        "Declined. Re-enable with: ai-brains harness reset-decline --harness agy"
                    );
                }
            }
        }
    }
    Ok(())
}

/// Report AGY install outcomes honestly (F28/AC21 — never claim success on Refused).
///
/// When `fail_on_error` is true (explicit `--install-hooks`), refuse/error returns
/// `Err` so the process exits non-zero (F20). Soft consent/auto paths keep preflight exit 0.
fn report_preflight_install(
    result: Result<InstallOutcome, String>,
    success_msg: &str,
    fail_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match result {
        Ok(InstallOutcome::Installed { .. }) => {
            println!("{success_msg}");
            Ok(())
        }
        Ok(InstallOutcome::DryRun { .. }) => {
            println!("[dry-run] AGY install planned (no writes).");
            Ok(())
        }
        Ok(InstallOutcome::BackendPending { plan }) => {
            eprintln!(
                "AGY install backend pending; no files written. next: {}",
                plan.pending_track.unwrap_or("ai-brains harness status")
            );
            Ok(())
        }
        Ok(InstallOutcome::Refused { path, reason }) => {
            eprintln!(
                "Refused to rewrite {}: {}. Fix or remove the corrupt file, then re-run: ai-brains harness install --harness agy",
                path.display(),
                reason
            );
            if fail_on_error {
                Err(format!("refused rewrite {}: {reason}", path.display()).into())
            } else {
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("harness install failed: {e}");
            if fail_on_error { Err(e.into()) } else { Ok(()) }
        }
    }
}

/// Normalize scope paths for Windows: resolve drive case, UNC prefixes, separator consistency.
fn normalize_scope_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                return None;
            }
            let normalized = std::path::Path::new(trimmed);
            if normalized.exists() {
                Some(
                    std::fs::canonicalize(normalized)
                        .ok()
                        .and_then(|pb| pb.to_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| trimmed.to_string()),
                )
            } else {
                Some(trimmed.replace('\\', "/").to_lowercase())
            }
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::harness::{HarnessStatus, WiringStatus};
    use ai_brains_core::ids::ProjectId;
    use std::str::FromStr;

    #[test]
    fn format_harness_summary_lines__all_absent__empty() {
        let statuses = vec![HarnessStatus {
            id: "grok".into(),
            display_name: "Grok".into(),
            present: false,
            binary: None,
            home_path: None,
            wiring: WiringStatus::Absent,
            install_ready: false,
            targets: vec![],
            next_action: "n/a".into(),
        }];
        assert!(format_harness_summary_lines(&statuses).is_empty());
    }

    #[test]
    fn format_harness_summary_lines__header_and_next_action() {
        let statuses = vec![HarnessStatus {
            id: "agy".into(),
            display_name: "AGY".into(),
            present: true,
            binary: None,
            home_path: Some("/tmp/.gemini".into()),
            wiring: WiringStatus::Missing,
            install_ready: true,
            targets: vec![],
            next_action: "ai-brains harness install --harness agy --dry-run".into(),
        }];
        let lines = format_harness_summary_lines(&statuses);
        let joined = lines.join("\n");
        assert!(
            joined.contains("Harnesses installed on machine:"),
            "exact header F8/F30; got:\n{joined}"
        );
        assert!(
            !joined.to_ascii_lowercase().contains("active harness"),
            "must not say active harness"
        );
        assert!(joined.contains("agy"));
        assert!(joined.contains("wiring=missing"));
        assert!(joined.contains("ai-brains harness install --harness agy --dry-run"));
    }

    /// AC19: format_preflight_summary_lines arity unchanged (compiles with 9 args).
    #[test]
    fn format_preflight_summary_lines__arity_nine_args() {
        let _ = format_preflight_summary_lines("Scope: global", true, Some(0), 0, 0, 0, 0, 0, 0);
    }

    #[test]
    fn normalize_scope_paths_filters_empty() {
        let paths = vec![
            "  ".to_string(),
            "".to_string(),
            "nonexistent/file.rs".to_string(),
        ];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Non-existent paths get lowercased with forward slashes
        assert!(normalized[0].contains("nonexistent/file.rs"));
    }

    #[test]
    fn normalize_scope_paths_normalizes_separators() {
        let paths = vec!["C:\\dev\\src\\lib.rs".to_string()];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Non-existent path: should be lowercased with forward slashes
        let result = &normalized[0];
        assert!(
            !result.contains('\\'),
            "Backslashes should be normalized: {}",
            result
        );
    }

    #[test]
    fn normalize_scope_paths_handles_existing_path() {
        // Use a path we know exists (the project directory)
        let paths = vec!["C:\\dev\\AI-Brains\\src".to_string()];
        let normalized = normalize_scope_paths(&paths);
        assert_eq!(normalized.len(), 1);
        // Canonicalization should produce a valid path string
        assert!(!normalized[0].is_empty());
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_summary_lines__global__scope_and_projects_and_in_context() {
        let lines =
            format_preflight_summary_lines("Scope: global", true, Some(2), 5, 1, 3, 4, 1, 100);
        let joined = lines.join("\n");
        assert!(
            joined.contains("Scope: global"),
            "AC8-style: must contain Scope: global; got:\n{joined}"
        );
        assert!(
            joined.contains("Projects: 2"),
            "global must print Projects line; got:\n{joined}"
        );
        assert!(
            joined.contains("Pinned memories: 5"),
            "pinned vault count; got:\n{joined}"
        );
        assert!(
            joined.contains("Active sessions: 1"),
            "active sessions vault count; got:\n{joined}"
        );
        assert!(
            joined.contains("In context hotspots: 3"),
            "AC5 In context hotspots; got:\n{joined}"
        );
        assert!(
            joined.contains("In context decisions: 4"),
            "AC5 In context decisions; got:\n{joined}"
        );
        assert!(
            joined.contains("In context constraints: 1"),
            "AC5 In context constraints; got:\n{joined}"
        );
        assert!(
            joined.contains("Total Word Count: 100"),
            "word count from field; got:\n{joined}"
        );
        assert!(
            !joined.lines().any(|l| l.starts_with("Project:")),
            "must not print legacy Project: line; got:\n{joined}"
        );
        assert!(
            joined.contains("Use --pretty or --format json for full context."),
            "footer required; got:\n{joined}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_summary_lines__project_scoped__no_projects_line() {
        let pid = ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let scope = format!("Scope: project={}", pid);
        let lines = format_preflight_summary_lines(&scope, false, None, 2, 0, 0, 1, 0, 42);
        let joined = lines.join("\n");
        assert!(joined.contains(&format!("Scope: project={}", pid)));
        assert!(
            !joined.lines().any(|l| l.starts_with("Projects:")),
            "project-scoped must omit Projects: line; got:\n{joined}"
        );
        assert!(joined.contains("Pinned memories: 2"));
        assert!(joined.contains("In context decisions: 1"));
        assert!(!joined.lines().any(|l| l.starts_with("Project:")));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_summary_lines__empty_zeros() {
        let lines =
            format_preflight_summary_lines("Scope: global", true, Some(0), 0, 0, 0, 0, 0, 0);
        let joined = lines.join("\n");
        assert!(joined.contains("Scope: global"));
        assert!(joined.contains("Projects: 0"));
        assert!(joined.contains("Pinned memories: 0"));
        assert!(joined.contains("Active sessions: 0"));
        assert!(joined.contains("In context hotspots: 0"));
        assert!(!joined.is_empty());
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__via_recall__global_soot() {
        // AC8: shared SOOT remains Scope: global
        assert_eq!(
            super::super::recall::format_scope_line(true, None, None),
            "Scope: global"
        );
    }
}
