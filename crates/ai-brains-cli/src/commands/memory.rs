//! T216 — Memory inventory skim (`memory list` / shared forget list backend).
//!
//! Read-only: SQL + pure formatters only (no models, embeddings, graph, ledgerful).
//! Never appends events.

use crate::commands::governed_common::fail_usage;
use crate::commands::project::{display_label, format_last_activity, truncate_chars};
use crate::commands::recall::format_scope_line;
use crate::context::AppContext;
use ai_brains_control_plane::clamp_list_limit;
use ai_brains_core::ids::ProjectId;
use ai_brains_store::{MemoryListFilter, MemoryListRow, MemoryListStatus, QueryStore};
use serde::Serialize;
use std::str::FromStr;

/// Human table project column max chars under `--global` (F8 / PROJECT_COL_MAX).
pub(crate) const PROJECT_COL_MAX: usize = 20;
/// List preview max chars (F9 / F26) — not the same as forget match-preview 100.
pub(crate) const PREVIEW_MAX_CHARS: usize = 80;

const SCOPE_MISSING_MSG: &str =
    "No project scope. Set AI_BRAINS_PROJECT_ID, run `ai-brains context`, or pass --global.";
const INVALID_STATUS_MSG: &str = "Invalid --status. Use pinned or forgotten.";
const EMPTY_TAG_MSG: &str = "Empty --tag is not allowed.";

// ---------------------------------------------------------------------------
// Pure helpers (F9 / F12 / F8) — unit-tested
// ---------------------------------------------------------------------------

/// First non-empty line; always strip leading USER:/ASSISTANT:/SYSTEM: (case-sensitive
/// token + whitespace); char-safe truncate with `…` (F9/F31).
pub(crate) fn preview_line(content: &str, max_chars: usize) -> String {
    let mut line = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string();
    for prefix in ["USER:", "ASSISTANT:", "SYSTEM:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            line = rest.trim_start().to_string();
            break;
        }
    }
    truncate_preview_chars(&line, max_chars)
}

fn truncate_preview_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let truncated: String = s.chars().take(keep).collect();
    format!("{truncated}…")
}

/// Parse first line if `TAGS: …` (after optional role prefix), split comma, trim,
/// case-insensitive exact token (F12). Handles pin/capture storage shape
/// `ASSISTANT: TAGS: a, b\nbody`.
pub(crate) fn content_has_tag(content: &str, tag: &str) -> bool {
    let first = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let mut line = first;
    for prefix in ["USER:", "ASSISTANT:", "SYSTEM:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            line = rest.trim_start();
            break;
        }
    }
    let Some(rest) = line.strip_prefix("TAGS:") else {
        return false;
    };
    let needle = tag.trim();
    if needle.is_empty() {
        return false;
    }
    rest.split(',')
        .map(str::trim)
        .any(|tok| !tok.is_empty() && tok.eq_ignore_ascii_case(needle))
}

/// Truncate project label column to `max` chars with `…` (F8 AC20).
pub(crate) fn truncate_project_col(label: &str, max: usize) -> String {
    truncate_chars(label, max)
}

// ---------------------------------------------------------------------------
// JSON DTOs (CLI-local, F10/F11/F22 — no contracts freeze)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct MemoryListJson {
    api_version: String,
    scope: String,
    project_id: Option<String>,
    status: String,
    items: Vec<MemoryListItemJson>,
    returned: usize,
    more_available: bool,
    limit: usize,
    total: u64,
}

#[derive(Debug, Serialize)]
struct MemoryListItemJson {
    memory_id: String,
    preview: String,
    updated_at: String,
    project_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct MemorySummaryJson {
    api_version: String,
    scope: String,
    project_id: Option<String>,
    pinned: u64,
    forgotten: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_project: Option<Vec<MemoryByProjectJson>>,
}

#[derive(Debug, Serialize)]
struct MemoryByProjectJson {
    project_id: String,
    label: String,
    pinned: u64,
    forgotten: u64,
}

// ---------------------------------------------------------------------------
// Options + run
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MemoryListOptions {
    pub status: String,
    pub limit: Option<usize>,
    pub global: bool,
    pub format: String,
    pub summary: bool,
    pub tag: Option<String>,
    pub project_id: Option<ProjectId>,
}

/// `ai-brains memory list` entry point.
pub fn run_list(
    ctx: &AppContext,
    opts: MemoryListOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    run_inventory(ctx, opts)
}

/// Shared inventory path for `memory list` and `forget --list-forgotten` (F1/F28).
pub fn run_inventory(
    ctx: &AppContext,
    opts: MemoryListOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Scope (F3): missing project without --global → exit 2.
    if !opts.global && opts.project_id.is_none() {
        return fail_usage(SCOPE_MISSING_MSG);
    }

    // Tag empty string → exit 2 (F12/F44).
    if let Some(ref t) = opts.tag
        && t.trim().is_empty()
    {
        return fail_usage(EMPTY_TAG_MSG);
    }

    // Summary mode ignores --status / --limit (F11/F47).
    if opts.summary {
        return run_summary(ctx, &opts);
    }

    let status = parse_status(&opts.status)?;
    let page_limit = clamp_list_limit(opts.limit);
    let tag = opts.tag.as_ref().map(|s| s.trim().to_string());
    let project_id = if opts.global { None } else { opts.project_id };

    // F43: when --tag, over-fetch candidates then token-filter then page.
    let sql_limit = if tag.is_some() {
        page_limit.saturating_mul(4).max(50).saturating_add(1)
    } else {
        page_limit.saturating_add(1)
    };

    let filter = MemoryListFilter {
        status,
        project_id,
        tag: tag.clone(),
        limit: sql_limit,
    };

    let mut rows = ctx.conn.list_memories(&filter)?;
    if let Some(ref t) = tag {
        rows.retain(|r| content_has_tag(&r.content, t));
    }

    let more_available = rows.len() > page_limit;
    if more_available {
        rows.truncate(page_limit);
    }

    // Total: two-stage when tag set (store count_memories handles tokens).
    let total = ctx.conn.count_memories(&MemoryListFilter {
        status,
        project_id,
        tag: tag.clone(),
        limit: 0, // ignored by count
    })?;

    let is_json = opts.format.eq_ignore_ascii_case("json");
    if is_json {
        return emit_list_json(
            opts.global,
            project_id.as_ref(),
            status,
            page_limit,
            &rows,
            more_available,
            total,
        );
    }

    emit_list_human(
        ctx,
        opts.global,
        project_id.as_ref(),
        status,
        page_limit,
        &rows,
        more_available,
        total,
    )
}

fn parse_status(raw: &str) -> Result<MemoryListStatus, Box<dyn std::error::Error>> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pinned" => Ok(MemoryListStatus::Pinned),
        "forgotten" => Ok(MemoryListStatus::Forgotten),
        // fail_usage → GovernedCliError → process exit 2 (F3/F44).
        // Always Err; trailing Err satisfies Result type for the compiler.
        _ => {
            fail_usage(INVALID_STATUS_MSG)?;
            Err(INVALID_STATUS_MSG.into())
        }
    }
}

fn run_summary(
    ctx: &AppContext,
    opts: &MemoryListOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let tag = opts.tag.as_ref().map(|s| s.trim().to_string());
    let project_id = if opts.global { None } else { opts.project_id };

    let pinned = ctx.conn.count_memories(&MemoryListFilter {
        status: MemoryListStatus::Pinned,
        project_id,
        tag: tag.clone(),
        limit: 0,
    })?;
    let forgotten = ctx.conn.count_memories(&MemoryListFilter {
        status: MemoryListStatus::Forgotten,
        project_id,
        tag: tag.clone(),
        limit: 0,
    })?;

    // F46: under --global, by_project cells use the same two-stage tag filter as totals.
    let by_project = if opts.global {
        Some(build_by_project_rows(ctx, tag.as_deref())?)
    } else {
        None
    };

    let is_json = opts.format.eq_ignore_ascii_case("json");
    if is_json {
        return emit_summary_json(
            opts.global,
            project_id.as_ref(),
            pinned,
            forgotten,
            by_project,
        );
    }

    emit_summary_human(
        ctx,
        opts.global,
        project_id.as_ref(),
        pinned,
        forgotten,
        by_project.as_deref(),
    )
}

/// Global summary by-project rows (F11/F38/F46).
///
/// When `tag` is `Some`, each cell is re-counted with the same two-stage tag filter
/// as top-line totals; projects with both counts 0 after filter are omitted.
fn build_by_project_rows(
    ctx: &AppContext,
    tag: Option<&str>,
) -> Result<Vec<MemoryByProjectJson>, Box<dyn std::error::Error>> {
    let counts: Vec<(String, u64, u64)> = if let Some(tag) = tag {
        let base = ctx.conn.count_memories_by_project()?;
        let mut filtered = Vec::with_capacity(base.len());
        for (project_id, _, _) in base {
            let pid = match ProjectId::from_str(&project_id) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let pinned = ctx.conn.count_memories(&MemoryListFilter {
                status: MemoryListStatus::Pinned,
                project_id: Some(pid),
                tag: Some(tag.to_string()),
                limit: 0,
            })?;
            let forgotten = ctx.conn.count_memories(&MemoryListFilter {
                status: MemoryListStatus::Forgotten,
                project_id: Some(pid),
                tag: Some(tag.to_string()),
                limit: 0,
            })?;
            if pinned > 0 || forgotten > 0 {
                filtered.push((project_id, pinned, forgotten));
            }
        }
        filtered.sort_by(|a, b| {
            let sum_a = a.1.saturating_add(a.2);
            let sum_b = b.1.saturating_add(b.2);
            sum_b.cmp(&sum_a).then_with(|| a.0.cmp(&b.0))
        });
        filtered
    } else {
        ctx.conn.count_memories_by_project()?
    };

    let mut out = Vec::with_capacity(counts.len());
    for (project_id, pinned, forgotten) in counts {
        let (name, alias) = match ProjectId::from_str(&project_id)
            .ok()
            .and_then(|pid| ctx.conn.get_project_by_id(&pid).ok().flatten())
        {
            Some((n, a)) => (n, a),
            None => (String::new(), String::new()),
        };
        let label = display_label(&name, &alias, &project_id);
        out.push(MemoryByProjectJson {
            project_id,
            label,
            pinned,
            forgotten,
        });
    }
    Ok(out)
}

fn emit_list_json(
    global: bool,
    project_id: Option<&ProjectId>,
    status: MemoryListStatus,
    limit: usize,
    rows: &[MemoryListRow],
    more_available: bool,
    total: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<MemoryListItemJson> = rows
        .iter()
        .map(|r| MemoryListItemJson {
            memory_id: r.memory_id.clone(),
            preview: preview_line(&r.content, PREVIEW_MAX_CHARS),
            updated_at: r.updated_at.clone(),
            project_id: r.project_id.clone(),
        })
        .collect();
    let envelope = MemoryListJson {
        api_version: "1".to_string(),
        scope: if global {
            "global".to_string()
        } else {
            "project".to_string()
        },
        project_id: project_id.map(|p| p.to_string()),
        status: status.as_str().to_string(),
        returned: items.len(),
        more_available,
        limit,
        total,
        items,
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

#[allow(clippy::too_many_arguments)] // local human renderer; keeps call site explicit
fn emit_list_human(
    ctx: &AppContext,
    global: bool,
    project_id: Option<&ProjectId>,
    status: MemoryListStatus,
    limit: usize,
    rows: &[MemoryListRow],
    more_available: bool,
    total: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_alias = if !global {
        match project_id {
            Some(pid) => ctx.conn.get_project_by_id(pid)?,
            None => None,
        }
    } else {
        None
    };
    let scope_line = format_scope_line(global, project_id, name_alias.as_ref());
    println!("{scope_line}");
    println!("status={}  limit={}", status.as_str(), limit);

    if rows.is_empty() {
        match status {
            MemoryListStatus::Pinned => println!("No pinned memories."),
            MemoryListStatus::Forgotten => println!("No forgotten memories."),
        }
        return Ok(());
    }

    if global {
        println!(
            "{:<36} {:<20} {:<12} preview",
            "memory_id", "project", "updated"
        );
    } else {
        println!("{:<36} {:<12} preview", "memory_id", "updated");
    }

    for r in rows {
        let updated = format_last_activity(&r.updated_at);
        let preview = preview_line(&r.content, PREVIEW_MAX_CHARS);
        if global {
            let proj_label = match r.project_id.as_deref() {
                Some(pid) if !pid.is_empty() => {
                    let (name, alias) = match ProjectId::from_str(pid)
                        .ok()
                        .and_then(|p| ctx.conn.get_project_by_id(&p).ok().flatten())
                    {
                        Some((n, a)) => (n, a),
                        None => (String::new(), String::new()),
                    };
                    truncate_project_col(&display_label(&name, &alias, pid), PROJECT_COL_MAX)
                }
                _ => "—".to_string(),
            };
            println!(
                "{:<36} {:<20} {:<12} {}",
                r.memory_id, proj_label, updated, preview
            );
        } else {
            println!("{:<36} {:<12} {}", r.memory_id, updated, preview);
        }
    }

    let returned = rows.len();
    if more_available || (total as usize) > returned {
        println!(
            "Showing {} of {}  (more available; raise --limit)",
            returned, total
        );
    } else {
        println!("Showing {} of {}", returned, total);
    }

    // F36: stderr next-step (not on empty / json / summary).
    eprintln!(
        "Use ai-brains forget --memory-id <id> -f to forget, or ai-brains forget --restore <id> for forgotten rows."
    );
    Ok(())
}

fn emit_summary_json(
    global: bool,
    project_id: Option<&ProjectId>,
    pinned: u64,
    forgotten: u64,
    by_project: Option<Vec<MemoryByProjectJson>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let envelope = MemorySummaryJson {
        api_version: "1".to_string(),
        scope: if global {
            "global".to_string()
        } else {
            "project".to_string()
        },
        project_id: project_id.map(|p| p.to_string()),
        pinned,
        forgotten,
        by_project,
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}

fn emit_summary_human(
    ctx: &AppContext,
    global: bool,
    project_id: Option<&ProjectId>,
    pinned: u64,
    forgotten: u64,
    by_project: Option<&[MemoryByProjectJson]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name_alias = if !global {
        match project_id {
            Some(pid) => ctx.conn.get_project_by_id(pid)?,
            None => None,
        }
    } else {
        None
    };
    let scope_line = format_scope_line(global, project_id, name_alias.as_ref());
    println!("{scope_line}");
    println!("Pinned: {pinned}");
    println!("Forgotten: {forgotten}");

    if global {
        match by_project {
            Some(rows) if !rows.is_empty() => {
                println!(
                    "{:<20} {:<36} {:>8} {:>10}",
                    "label", "project_id", "pinned", "forgotten"
                );
                for r in rows {
                    let label = truncate_project_col(&r.label, PROJECT_COL_MAX);
                    println!(
                        "{:<20} {:<36} {:>8} {:>10}",
                        label, r.project_id, r.pinned, r.forgotten
                    );
                }
            }
            _ => println!("No projects with memories."),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn preview_line__role_prefix_stripped_always() {
        assert_eq!(
            preview_line("ASSISTANT: DECISION: use SQLCipher", 80),
            "DECISION: use SQLCipher"
        );
        assert_eq!(preview_line("USER: hello world", 80), "hello world");
        assert_eq!(preview_line("SYSTEM: note", 80), "note");
        // Only leading role token — mid-line not stripped.
        assert_eq!(
            preview_line("text ASSISTANT: still here", 80),
            "text ASSISTANT: still here"
        );
        // Case-sensitive: lowercase not stripped.
        assert_eq!(
            preview_line("assistant: leave me", 80),
            "assistant: leave me"
        );
    }

    #[test]
    fn preview_line__multibyte_truncate__no_panic() {
        let s = "日本語テストプレビュー境界値チェック用の長い行です";
        let out = preview_line(s, 10);
        assert!(out.chars().count() <= 10, "got {out:?}");
        assert!(out.ends_with('…') || out.chars().count() <= 10);
        // Short unchanged.
        assert_eq!(preview_line("short", 80), "short");
    }

    #[test]
    fn preview_line__first_non_empty_line() {
        assert_eq!(
            preview_line("\n\n  ASSISTANT: body line\nsecond", 80),
            "body line"
        );
    }

    #[test]
    fn content_has_tag__exact_token_case_insensitive() {
        assert!(content_has_tag("TAGS: foo, bar\nbody", "foo"));
        assert!(content_has_tag("TAGS: foo, bar\nbody", "FOO"));
        assert!(content_has_tag("TAGS: foo, bar\nbody", "bar"));
        assert!(!content_has_tag("TAGS: foobar\nbody", "foo"));
        assert!(!content_has_tag("TAGS: foo, bar\nbody", "baz"));
        // Capture/pin storage: role prefix before TAGS:
        assert!(content_has_tag("ASSISTANT: TAGS: foo, bar\nbody", "foo"));
        assert!(!content_has_tag("ASSISTANT: TAGS: foobar\nbody", "foo"));
        // Mid-body TAGS: without first-line prefix → false.
        assert!(!content_has_tag("body with mid TAGS: foo elsewhere", "foo"));
        assert!(!content_has_tag(
            "ASSISTANT: body with mid TAGS: foo",
            "foo"
        ));
        assert!(!content_has_tag("no tags here", "foo"));
        assert!(!content_has_tag("TAGS: foo\nbody", ""));
    }

    #[test]
    fn truncate_project_col__max_20_with_ellipsis() {
        let long = "abcdefghijklmnopqrstuvwxyz";
        let out = truncate_project_col(long, PROJECT_COL_MAX);
        assert_eq!(out.chars().count(), PROJECT_COL_MAX);
        assert!(out.ends_with('…'));
        assert_eq!(truncate_project_col("short", 20), "short");
    }
}
