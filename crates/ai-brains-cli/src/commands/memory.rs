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
use ai_brains_retrieval::{PinKind, classify_pin_kind, first_contentful_line, is_session_chrome};
use ai_brains_store::{MemoryListFilter, MemoryListRow, MemoryListStatus, QueryStore};
use serde::Serialize;
use std::collections::HashSet;
use std::str::FromStr;

/// Human table project column max chars under `--global` (F8 / PROJECT_COL_MAX).
pub(crate) const PROJECT_COL_MAX: usize = 20;
/// List preview max chars (F9 / F26) — not the same as forget match-preview 100.
pub(crate) const PREVIEW_MAX_CHARS: usize = 80;
/// T316 F4 — max chrome lines skipped after the envelope before fallback.
pub(crate) const PREVIEW_CHROME_WALK: usize = 8;
/// T316 F33 — closed agent preamble prefixes (ASCII-lower); not in session_chrome.
pub(crate) const PREVIEW_AGENT_CHROME_PREFIXES: &[&str] =
    &["let me ", "now let me ", "i'll ", "i will "];

const SCOPE_MISSING_MSG: &str =
    "No project scope. Set AI_BRAINS_PROJECT_ID, run `ai-brains context`, or pass --global.";
const INVALID_STATUS_MSG: &str = "Invalid --status. Use pinned or forgotten.";
const EMPTY_TAG_MSG: &str = "Empty --tag is not allowed.";
/// T331 F35 — copy-not-share Index F4 (61 chars). Do not import retrieval.
const EMPTY_AUTHORITY_HONESTY: &str =
    "No DECISION/CONSTRAINT pins in scope; showing recent activity";

/// T299 F26 — forgotten-empty remediator lines (`Pinned: N` when COUNT Ok + `next:` last).
///
/// `None` = fail-open (omit `Pinned:`); `Some(0)` still prints `Pinned: 0`.
pub(crate) fn forgotten_empty_remediator(pinned: Option<u64>, global: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(n) = pinned {
        lines.push(format!("Pinned: {n}"));
    }
    if global {
        lines.push("next: ai-brains memory list --global".to_string());
    } else {
        lines.push("next: ai-brains memory list".to_string());
    }
    lines
}

// ---------------------------------------------------------------------------
// Pure helpers (F9 / F12 / F8) — unit-tested
// ---------------------------------------------------------------------------

/// True when a one-line preview candidate is leading chrome (T316 F2/F3).
///
/// Authority (`Decision` / `Constraint` / `Hotspot`) is never chrome.
/// Empty → `Other` (not authority-elevated).
pub(crate) fn preview_line_is_chrome(line: &str) -> bool {
    match classify_pin_kind(line) {
        PinKind::Decision | PinKind::Constraint | PinKind::Hotspot => return false,
        PinKind::Other => {}
    }
    if is_session_chrome(line) {
        return true;
    }
    if line.trim_start().starts_with("```") {
        return true;
    }
    let lower = line.trim_start().to_ascii_lowercase();
    PREVIEW_AGENT_CHROME_PREFIXES
        .iter()
        .any(|p| lower.starts_with(p))
}

/// T331 F7 — authority never chrome; session chrome on full body; agent/fence on leading line only.
fn row_is_list_chrome(content: &str) -> bool {
    if classify_pin_kind(content) != PinKind::Other {
        return false;
    }
    is_session_chrome(content) || preview_line_is_chrome(first_contentful_line(content))
}

/// Non-empty body lines after the T285/T287 envelope (role + optional TAGS:).
fn preview_body_lines(content: &str) -> Vec<&str> {
    let after_role = super::display_text::strip_role_prefix(content.trim_start()).trim_start();
    let mut lines = after_role.lines().map(str::trim).filter(|l| !l.is_empty());
    let mut out = Vec::new();
    if let Some(first) = lines.next() {
        if first.to_ascii_lowercase().starts_with("tags:") {
            out.extend(lines);
        } else {
            out.push(first);
            out.extend(lines);
        }
    }
    out
}

/// First non-chrome line within the walk cap; else envelope / TAGS fallback (T316 F1–F5).
pub(crate) fn skip_leading_preview_chrome(content: &str) -> String {
    let trimmed = content.trim_start();
    let contentful = first_contentful_line(trimmed);
    let fallback = if contentful.is_empty() {
        let raw = content
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        super::display_text::strip_role_prefix(raw).to_string()
    } else {
        contentful.to_string()
    };

    let lines = preview_body_lines(trimmed);
    if lines.is_empty() {
        return fallback;
    }

    let mut chrome_skipped = 0usize;
    for line in lines {
        if preview_line_is_chrome(line) {
            if chrome_skipped >= PREVIEW_CHROME_WALK {
                break;
            }
            chrome_skipped += 1;
            continue;
        }
        return line.to_string();
    }
    fallback
}

/// First contentful line after the pin envelope (T287 F6); T316 chrome-skip; char-safe truncate.
///
/// Empty `first_contentful_line` falls back to today's first non-empty line
/// after role strip (may be `TAGS:` — not `""`). `trim_start` so leading
/// blank lines still strip `ASSISTANT:` (existing `preview_line__first_non_empty_line`).
pub(crate) fn preview_line(content: &str, max_chars: usize) -> String {
    let line = skip_leading_preview_chrome(content);
    super::display_text::truncate_preview_chars(&line, max_chars)
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
    let line = super::display_text::strip_role_prefix(first);
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

/// Prefer-fill authority rows then recency-fill (T287 F35).
///
/// Pass-1 order is preserved; pass-2 ids already in pass-1 are skipped;
/// result length is at most `limit`.
pub(crate) fn prefer_fill_authority(
    pass1: Vec<MemoryListRow>,
    pass2: Vec<MemoryListRow>,
    limit: usize,
) -> Vec<MemoryListRow> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for row in pass1 {
        if out.len() >= limit {
            break;
        }
        if seen.insert(row.memory_id.clone()) {
            out.push(row);
        }
    }
    for row in pass2 {
        if out.len() >= limit {
            break;
        }
        if seen.insert(row.memory_id.clone()) {
            out.push(row);
        }
    }
    out
}

/// Empty-GLOB recency fill (T331 F7): authority, then non-chrome Other, then chrome; cap `limit`.
pub(crate) fn recency_fill_empty_authority(
    pool: Vec<MemoryListRow>,
    limit: usize,
) -> Vec<MemoryListRow> {
    let mut authority = Vec::new();
    let mut non_chrome = Vec::new();
    let mut chrome = Vec::new();
    for row in pool {
        if classify_pin_kind(&row.content) != PinKind::Other {
            authority.push(row);
        } else if row_is_list_chrome(&row.content) {
            chrome.push(row);
        } else {
            non_chrome.push(row);
        }
    }
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for row in authority.into_iter().chain(non_chrome).chain(chrome) {
        if out.len() >= limit {
            break;
        }
        if seen.insert(row.memory_id.clone()) {
            out.push(row);
        }
    }
    out
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
    let is_json = opts.format.eq_ignore_ascii_case("json");

    // F43: when --tag, over-fetch candidates then token-filter then page.
    let overfetch = page_limit.saturating_mul(4).max(50).saturating_add(1);

    let mut rows = if is_json || status != MemoryListStatus::Pinned {
        let recency_limit = if tag.is_some() {
            overfetch
        } else {
            page_limit.saturating_add(1)
        };
        let filter = MemoryListFilter {
            status,
            project_id,
            tag: tag.clone(),
            limit: recency_limit,
        };
        let mut rows = ctx.conn.list_memories(&filter)?;
        if let Some(ref t) = tag {
            rows.retain(|r| content_has_tag(&r.content, t));
        }
        rows
    } else {
        // GLOB is a superset of classifier (TAGS envelope matches tagged dumps).
        // LIMIT page then retain Other can empty pass-1 while older DECISION pins
        // still exist (live hole). Over-fetch like F43, then retain, then mix.
        let pass1_limit = overfetch;
        let mut pass1 = ctx.conn.list_authority_memories(&MemoryListFilter {
            status,
            project_id,
            tag: tag.clone(),
            limit: pass1_limit,
        })?;
        pass1.retain(|r| classify_pin_kind(&r.content) != PinKind::Other);
        if let Some(ref t) = tag {
            pass1.retain(|r| content_has_tag(&r.content, t));
        }
        if pass1.is_empty() {
            // T331 F1/F8: empty GLOB+retain → over-fetch recency and row-skip chrome.
            let mut pass2 = ctx.conn.list_memories(&MemoryListFilter {
                status,
                project_id,
                tag: tag.clone(),
                limit: overfetch,
            })?;
            if let Some(ref t) = tag {
                pass2.retain(|r| content_has_tag(&r.content, t));
            }
            recency_fill_empty_authority(pass2, page_limit.saturating_add(1))
        } else {
            let recency_limit = if tag.is_some() {
                overfetch
            } else {
                page_limit.saturating_add(1)
            };
            let mut pass2 = ctx.conn.list_memories(&MemoryListFilter {
                status,
                project_id,
                tag: tag.clone(),
                limit: recency_limit,
            })?;
            if let Some(ref t) = tag {
                pass2.retain(|r| content_has_tag(&r.content, t));
            }
            prefer_fill_authority(pass1, pass2, page_limit.saturating_add(1))
        }
    };

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

    let show_empty_authority_honesty = status == MemoryListStatus::Pinned
        && !rows.is_empty()
        && rows
            .iter()
            .all(|r| classify_pin_kind(&r.content) == PinKind::Other);

    emit_list_human(
        ctx,
        opts.global,
        project_id.as_ref(),
        status,
        page_limit,
        &rows,
        more_available,
        total,
        tag.as_deref(),
        show_empty_authority_honesty,
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
    crate::commands::identity_warn::print_json_stdout(&envelope)
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
    tag: Option<&str>,
    show_empty_authority_honesty: bool,
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
            MemoryListStatus::Forgotten => {
                println!("No forgotten memories.");
                // T299 F1/F2/F27: summary COUNT SoT + stdout remediator; still before F36.
                let pinned = ctx
                    .conn
                    .count_memories(&MemoryListFilter {
                        status: MemoryListStatus::Pinned,
                        project_id: if global { None } else { project_id.copied() },
                        tag: tag.map(str::to_string),
                        limit: 0,
                    })
                    .ok();
                for line in forgotten_empty_remediator(pinned, global) {
                    println!("{line}");
                }
            }
        }
        return Ok(());
    }

    if show_empty_authority_honesty {
        println!("{EMPTY_AUTHORITY_HONESTY}");
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

    // T316 F9: drop T216 F36 stderr forget hint (Windows stderr looks like an error).
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
    crate::commands::identity_warn::print_json_stdout(&envelope)
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

    /// T299 AC10 / F26 — remediator line shapes (fail-open / zero / global).
    #[rstest::rstest]
    #[case::some_n(Some(3), false, &["Pinned: 3", "next: ai-brains memory list"])]
    #[case::none_fail_open(None, false, &["next: ai-brains memory list"])]
    #[case::zero_pins(Some(0), false, &["Pinned: 0", "next: ai-brains memory list"])]
    #[case::global(Some(2), true, &["Pinned: 2", "next: ai-brains memory list --global"])]
    fn forgotten_empty_remediator__cases(
        #[case] pinned: Option<u64>,
        #[case] global: bool,
        #[case] expected: &[&str],
    ) {
        let lines = forgotten_empty_remediator(pinned, global);
        assert_eq!(lines, expected);
        for line in &lines {
            assert!(!line.contains('\n'), "F26 one line each; got {line:?}");
        }
        if pinned.is_none() {
            assert!(
                lines.iter().all(|l| !l.starts_with("Pinned:")),
                "fail-open omits Pinned:; got {lines:?}"
            );
        }
    }

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
    fn preview_line__tags_envelope__decision_not_tags() {
        let out = preview_line("ASSISTANT: TAGS: t287\nDECISION: needle", 80);
        assert!(
            out.contains("DECISION:"),
            "envelope preview must surface DECISION:; got {out:?}"
        );
        assert!(
            !out.starts_with("TAGS:"),
            "preview must not start with TAGS:; got {out:?}"
        );
    }

    #[test]
    fn preview_line__tags_only__fallback_non_empty() {
        let out = preview_line("ASSISTANT: TAGS: only", 80);
        assert!(!out.is_empty(), "TAGS-only fallback must not be empty");
        assert!(
            out.starts_with("TAGS:"),
            "empty contentful falls back to TAGS: line; got {out:?}"
        );
    }

    /// T316 AC1 — session chrome heading skipped when a later body line exists.
    #[test]
    fn preview_line__session_chrome_heading__skips_to_body() {
        let out = preview_line("## Objective\nWe decided SQLCipher", 80);
        assert!(
            out.contains("We decided") || out.contains("SQLCipher"),
            "AC1 preview must surface body, not heading; got {out:?}"
        );
        assert!(
            !out.starts_with("## Objective"),
            "AC1 must not keep ## Objective; got {out:?}"
        );
    }

    /// T316 AC2 — agent preamble skipped to authority line.
    #[test]
    fn preview_line__let_me_verify__skips_to_next() {
        let out = preview_line("Let me verify the clap pin\nCONSTRAINT: freeze ORDER", 80);
        assert!(
            out.starts_with("CONSTRAINT:"),
            "AC2 must skip Let me…; got {out:?}"
        );
    }

    /// T316 AC5 — all-chrome body falls back to first contentful (not empty).
    #[test]
    fn preview_line__all_chrome__fallback_first_contentful() {
        let out = preview_line("## Objective", 80);
        assert_eq!(out, "## Objective", "AC5 all-chrome fallback; got {out:?}");
    }

    /// T316 AC6 — authority one-liner never skipped (even with I'll…).
    #[test]
    fn preview_line__authority_line__never_skipped() {
        let out = preview_line("DECISION: I'll ship T316", 80);
        assert!(
            out.starts_with("DECISION:"),
            "AC6 authority must stay; got {out:?}"
        );
    }

    /// T316 AC19 — fence then Decision: first-non-chrome wins (not envelope-stop).
    #[test]
    fn preview_line__fence_then_decision__keeps_decision() {
        let out = preview_line("```json\nDECISION: needle", 80);
        assert!(
            out.starts_with("DECISION:"),
            "AC19 fence then Decision must keep Decision; got {out:?}"
        );
    }

    /// T316 AC7 — walk cap 8 chrome then body; 9th chrome-only stays fallback.
    #[rstest::rstest]
    #[case::eight_then_body(
        {
            let mut s = String::new();
            for _ in 0..8 {
                s.push_str("## Objective\n");
            }
            s.push_str("We decided after eight");
            s
        },
        "We decided after eight"
    )]
    #[case::nine_chrome_fallback(
        {
            let mut s = String::new();
            for _ in 0..9 {
                s.push_str("## Objective\n");
            }
            s
        },
        "## Objective"
    )]
    fn preview_line__walk_cap__eight(#[case] body: String, #[case] expect_prefix: &str) {
        let out = preview_line(&body, 80);
        assert!(
            out.starts_with(expect_prefix),
            "AC7 walk cap; expect starts with {expect_prefix:?}; got {out:?}"
        );
    }

    fn list_row(id: &str) -> MemoryListRow {
        MemoryListRow {
            memory_id: id.to_string(),
            content: format!("body {id}"),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            project_id: None,
            status: "pinned".to_string(),
        }
    }

    #[rstest::rstest]
    #[case::overlap(vec!["pin"], vec!["dump", "pin"], 10, vec!["pin", "dump"])]
    #[case::authority_only(vec!["pin1", "pin2"], vec![], 10, vec!["pin1", "pin2"])]
    #[case::recency_only(vec![], vec!["dump1", "dump2"], 10, vec!["dump1", "dump2"])]
    #[case::limit(vec!["pin1", "pin2"], vec!["dump1", "dump2"], 3, vec!["pin1", "pin2", "dump1"])]
    fn prefer_fill_authority__cases__expected_ids(
        #[case] pass1: Vec<&str>,
        #[case] pass2: Vec<&str>,
        #[case] limit: usize,
        #[case] expected: Vec<&str>,
    ) {
        let p1: Vec<MemoryListRow> = pass1.iter().copied().map(list_row).collect();
        let p2: Vec<MemoryListRow> = pass2.iter().copied().map(list_row).collect();
        let out = prefer_fill_authority(p1, p2, limit);
        let ids: Vec<String> = out.into_iter().map(|r| r.memory_id).collect();
        let expected: Vec<String> = expected.into_iter().map(str::to_string).collect();
        assert_eq!(ids, expected);
    }

    #[test]
    fn empty_authority_honesty__is_61_chars() {
        assert_eq!(EMPTY_AUTHORITY_HONESTY.len(), 61);
        assert_eq!(
            EMPTY_AUTHORITY_HONESTY,
            "No DECISION/CONSTRAINT pins in scope; showing recent activity"
        );
    }

    fn list_row_with(id: &str, content: &str) -> MemoryListRow {
        MemoryListRow {
            memory_id: id.to_string(),
            content: content.to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            project_id: None,
            status: "pinned".to_string(),
        }
    }

    #[rstest::rstest]
    #[case::authority_then_process_then_chrome(
        vec![
            ("chrome", "## Objective dump"),
            ("proc", "T331 process note about inventory fill"),
            ("pin", "decision: lowercase inventory pin"),
        ],
        10,
        vec!["pin", "proc", "chrome"]
    )]
    #[case::chrome_only_fallback(
        vec![
            ("c1", "## Objective dump one"),
            ("c2", "## Objective dump two"),
        ],
        10,
        vec!["c1", "c2"]
    )]
    #[case::empty_pool(vec![], 10, vec![])]
    #[case::duplicate_ids_once(
        vec![
            ("chrome", "## Objective dump"),
            ("chrome", "## Objective dump again"),
            ("proc", "T331 process unique body"),
        ],
        10,
        vec!["proc", "chrome"]
    )]
    #[case::mid_body_let_me_is_not_chrome(
        vec![
            ("chrome", "## Objective dump"),
            (
                "proc",
                "T331 process inventory note\nLet me verify the helper stays non-chrome",
            ),
        ],
        10,
        vec!["proc", "chrome"]
    )]
    #[case::json_decisions_head_is_chrome(
        vec![
            ("json", "{\n  \"decisions\": [\"x\"]\n}"),
            ("proc", "T331 process note after json dump"),
        ],
        10,
        vec!["proc", "json"]
    )]
    fn recency_fill_empty_authority__cases__expected_ids(
        #[case] pool: Vec<(&str, &str)>,
        #[case] limit: usize,
        #[case] expected: Vec<&str>,
    ) {
        let rows: Vec<MemoryListRow> = pool
            .into_iter()
            .map(|(id, content)| list_row_with(id, content))
            .collect();
        let out = recency_fill_empty_authority(rows, limit);
        let ids: Vec<String> = out.into_iter().map(|r| r.memory_id).collect();
        let expected: Vec<String> = expected.into_iter().map(str::to_string).collect();
        assert_eq!(ids, expected);
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
