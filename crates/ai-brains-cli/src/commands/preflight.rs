use crate::context::AppContext;
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
        print_summary(ctx, options.global, options.project_id, &context)?;
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

/// Print preflight summary with honest Scope + dual vault/in-context counts (T214 F37).
fn print_summary(
    ctx: &AppContext,
    global: bool,
    project_id: Option<ProjectId>,
    context: &ai_brains_retrieval::PreflightContext,
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
    Ok(())
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
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::ids::ProjectId;
    use std::str::FromStr;

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
