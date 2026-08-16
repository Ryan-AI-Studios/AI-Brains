use crate::commands::governed_common::{
    DISCOVERY_CAP_LABELS, POLICY_BOOTSTRAP_SOOT_SHORT, discovery_active_count, resolve_principal,
};
use crate::commands::harness::{PromptDecision, interpret_consent_answer, should_prompt_install};
use crate::context::AppContext;
use crate::harness::prefs::HarnessHookPrefs;
use crate::harness::{
    HarnessId, HarnessStatus, InstallOutcome, WiringStatus, collect_status_report, install_agy,
    install_grok, install_opencode, load_prefs, resolve_home, save_prefs,
};
use ai_brains_contracts::preflight::PreflightContextResponse;
use ai_brains_control_plane::{StorePorts, parse_scope_key, scope_identity_key};
use ai_brains_core::ids::ProjectId;
use ai_brains_retrieval::build_preflight;
use ai_brains_store::QueryStore;
use ai_brains_store::SqliteEventStore;
use serde::Serialize;
use std::io::IsTerminal;

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
    /// Tighter pretty item/line caps (human/pretty only). JSON and `--summary` ignore this.
    pub compact: bool,
}

/// CLI-local summary JSON envelope (T220). Never grows `PreflightContextResponse`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct PreflightSummaryJson {
    pub api_version: String,
    pub scope: String,
    pub project_id: Option<String>,
    /// Present only when `scope == "global"` (omit under project/none).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<u64>,
    pub pinned: u64,
    pub active_sessions: u64,
    pub in_context_hotspots: usize,
    pub in_context_decisions: usize,
    pub in_context_constraints: usize,
    /// Full preflight budget-window word count (`context.word_count`), not summary size.
    pub word_count: usize,
    /// T241 F3: present when project-scoped discovery grants incomplete (`active_count < 3`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grants_status: Option<String>,
    /// T241 F3: short bootstrap SOOT when discovery incomplete; omit when complete/global.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<String>,
}

/// Pure builder for preflight summary JSON (T220 F6). Unit-testable without vault I/O.
///
/// Argument count mirrors `format_preflight_summary_lines` dual-block fields one-for-one.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_preflight_summary_json(
    global: bool,
    project_id: Option<&ProjectId>,
    projects_with_pinned: Option<u64>,
    pinned_memories: u64,
    active_sessions: u64,
    hotspot_count: usize,
    decision_count: usize,
    constraint_count: usize,
    word_count: usize,
) -> PreflightSummaryJson {
    // F29 three-valued scope: global | project | none (never "project" with null id).
    let (scope, project_id_str, projects) = if global {
        ("global".to_string(), None, projects_with_pinned)
    } else if let Some(pid) = project_id {
        ("project".to_string(), Some(pid.to_string()), None)
    } else {
        ("none".to_string(), None, None)
    };
    PreflightSummaryJson {
        api_version: "1".to_string(),
        scope,
        project_id: project_id_str,
        projects,
        pinned: pinned_memories,
        active_sessions,
        in_context_hotspots: hotspot_count,
        in_context_decisions: decision_count,
        in_context_constraints: constraint_count,
        word_count,
        grants_status: None,
        next_step: None,
    }
}

/// Format a post-hoc discovery-grants summary line (T241 F3 / AC9).
///
/// Returns `None` when complete (`active_count == 3`) so callers omit OK density.
pub(crate) fn format_grants_incomplete_line(active_count: usize) -> Option<String> {
    if active_count >= 3 {
        return None;
    }
    let status = if active_count == 0 {
        "discovery grants empty (0 of 3)".to_string()
    } else {
        format!("discovery grants incomplete ({active_count} of 3)")
    };
    Some(format!("{status}; {POLICY_BOOTSTRAP_SOOT_SHORT}"))
}

/// Status string for incomplete discovery only (T241 F3 JSON `grants_status`).
pub(crate) fn format_grants_status(active_count: usize) -> Option<String> {
    if active_count >= 3 {
        return None;
    }
    if active_count == 0 {
        Some("discovery grants empty (0 of 3)".to_string())
    } else {
        Some(format!("discovery grants incomplete ({active_count} of 3)"))
    }
}

/// Probe discovery grant active_count for project-scoped preflight (T241 F3).
///
/// Uses the **same** `project_id` that scopes the summary (flag/env/`--project-id`),
/// not a separate ambient soft-resolve that could disagree with an explicit id.
/// Global / missing project / list errors → `None` (no grants line).
fn probe_discovery_active_count(
    ctx: &AppContext,
    global: bool,
    project_id: Option<&ProjectId>,
) -> Option<usize> {
    if global {
        return None;
    }
    let pid = project_id?;
    let ports = StorePorts::from_store(SqliteEventStore::new((*ctx.conn).clone()));
    // Canonical Repository scope for the summary's project (CX2: do not re-resolve ambient).
    let raw_scope = format!("Repository:{pid}");
    let scope_key = match parse_scope_key(&raw_scope) {
        Ok(s) => scope_identity_key(&s),
        Err(_) => return None,
    };
    let principal = resolve_principal(None);
    let grants = ports
        .grant_store()
        .list_applied_grants(principal.id, &scope_key, Some(&DISCOVERY_CAP_LABELS))
        .ok()?;
    Some(discovery_active_count(
        grants.iter().map(|g| g.capability.as_str()),
    ))
}

/// Emit install/status chatter: stdout for human path; stderr when JSON mode (T220 M1).
fn emit_status(json_mode: bool, msg: &str) {
    if json_mode {
        eprintln!("{msg}");
    } else {
        println!("{msg}");
    }
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
        // T220 F2: summary honors --format json case-insensitively; else human.
        let json_mode = options
            .format
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("json"));
        print_summary(
            ctx,
            options.global,
            options.project_id,
            &context,
            PreflightHarnessGate {
                no_hook_prompt: options.no_hook_prompt,
                install_hooks: options.install_hooks,
                stdin_mode: options.stdin_mode,
                json_mode,
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
        // F6/F6b: Scope header via CLI-only alias lookup (mirror print_summary).
        let name_alias = if !options.global {
            match options.project_id.as_ref() {
                Some(pid) => ctx.conn.get_project_by_id(pid)?,
                None => None,
            }
        } else {
            None
        };
        let scope = super::recall::format_scope_line(
            options.global,
            options.project_id.as_ref(),
            name_alias.as_ref(),
        );
        let pretty_body = if options.compact {
            format_preflight_pretty_body_with(&context.text, &PrettyCaps::compact())
        } else {
            format_preflight_pretty_body(&context.text)
        };
        println!("{scope}\n\n{pretty_body}");
    } else {
        // JSON path: raw post-F1 context.text + word_count only (no Scope/caps chrome).
        let response = PreflightContextResponse {
            text: context.text,
            word_count: context.word_count,
        };
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pretty body formatter (T219 F9/F10/F29/F31/F37 + T250 PrettyCaps)
// ---------------------------------------------------------------------------

/// Display-only caps for human/pretty preflight body (F29 constants-first).
pub(crate) const PRETTY_SAFETY_MAX_ITEMS: usize = 8;
pub(crate) const PRETTY_TURNS_PER_SESSION: usize = 6;
pub(crate) const PRETTY_MAX_SESSIONS: usize = 3;
pub(crate) const PRETTY_INDEX_MAX: usize = 15;
pub(crate) const PRETTY_RECENT_MAX: usize = 3;
pub(crate) const PRETTY_LINE_MAX: usize = 140;
pub(crate) const PRETTY_COMPACT_LINE_MAX: usize = 100;

/// Human/pretty display caps (T250). JSON / `--summary` never consult this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PrettyCaps {
    pub safety_max: usize,
    pub turns_per_session: usize,
    pub max_sessions: usize,
    pub index_max: usize,
    pub recent_max: usize,
    /// Session + Recent (and Safety when `first_line_only`).
    pub line_max: usize,
    /// Safety / Recent: emit first non-empty line of each taken item only.
    pub first_line_only: bool,
}

impl PrettyCaps {
    pub(crate) fn standard() -> Self {
        Self {
            safety_max: PRETTY_SAFETY_MAX_ITEMS,
            turns_per_session: PRETTY_TURNS_PER_SESSION,
            max_sessions: PRETTY_MAX_SESSIONS,
            index_max: PRETTY_INDEX_MAX,
            recent_max: PRETTY_RECENT_MAX,
            line_max: PRETTY_LINE_MAX,
            first_line_only: false,
        }
    }

    pub(crate) fn compact() -> Self {
        Self {
            safety_max: 3,
            turns_per_session: 2,
            max_sessions: 1,
            index_max: 5,
            recent_max: 2,
            line_max: PRETTY_COMPACT_LINE_MAX,
            first_line_only: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrettySectionKind {
    Safety,
    Session,
    Index,
    Recent,
    Other,
}

/// Full-line legacy section header: starts with `---` and ends with `---` after trim.
/// Does **not** treat `#` / `##` markdown as headers (F14 / AC14).
fn is_legacy_section_header(line: &str) -> bool {
    let t = line.trim();
    t.len() >= 7 && t.starts_with("---") && t.ends_with("---")
}

fn classify_section_header(header: &str) -> PrettySectionKind {
    let t = header.trim();
    if t.contains("Repository Bearings") || t.contains("Bearings & Safety") {
        PrettySectionKind::Safety
    } else if t.contains("Memory Index") {
        PrettySectionKind::Index
    } else if t.contains("Most Recent Memories") {
        PrettySectionKind::Recent
    } else if t.starts_with("--- Session:") || t.starts_with("--- Session ") {
        PrettySectionKind::Session
    } else {
        PrettySectionKind::Other
    }
}

/// Split content lines into blank-line-separated item blocks (safety / recent).
fn split_item_blocks(lines: &[&str]) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            if !cur.is_empty() {
                blocks.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(*line);
        }
    }
    if !cur.is_empty() {
        blocks.push(cur.join("\n"));
    }
    blocks
}

/// Pretty-only timestamp-then-role chrome strip (T250 F5).
///
/// `(inner) ROLE: body` → role-stripped body when `inner` is ≤32 chars and the
/// text after `)` (then whitespace) has a leading role token. Inner >32 is
/// fail-closed (not a timestamp). Does not change [`super::display_text::strip_role_prefix`].
pub(crate) fn strip_pretty_chrome(line: &str) -> &str {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix('(')
        && let Some(close) = rest.find(')')
    {
        let inner = &rest[..close];
        if inner.chars().count() <= 32 {
            let after_paren = &rest[close + 1..];
            // F5: `)` then whitespace then a leading role token (fail-closed).
            if after_paren.starts_with(|c: char| c.is_whitespace()) {
                let after = after_paren.trim_start();
                if super::display_text::has_leading_role_prefix(after) {
                    return super::display_text::strip_role_prefix(after);
                }
            }
        }
    }
    super::display_text::strip_role_prefix(trimmed)
}

fn first_non_empty_line(block: &str) -> &str {
    block.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
}

fn is_recall_hint_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("(Use 'recall'") || t.starts_with("(Use \"recall\"")
}

/// Chrome-strip then optional Unicode line-cap. Headers / F31 notices never call this.
fn display_pretty_line(line: &str, line_cap: Option<usize>) -> String {
    let stripped = strip_pretty_chrome(line);
    match line_cap {
        Some(max) => super::display_text::truncate_preview_chars(stripped, max),
        None => stripped.to_string(),
    }
}

fn emit_item_block(block: &str, caps: &PrettyCaps, line_cap: Option<usize>) -> String {
    if caps.first_line_only {
        display_pretty_line(first_non_empty_line(block), line_cap)
    } else {
        block
            .lines()
            .map(|l| display_pretty_line(l, line_cap))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// True when a trimmed line starts a retrieval session turn (`ROLE: …`).
///
/// Multi-line turns (from `truncate_turn`) emit only the first line with a role
/// prefix; continuation lines must not count as extra turns (T219 M1).
/// Token list SOOT: [`super::display_text::has_leading_role_prefix`] (T224 AC10).
/// Counts the **original** retrieval-emitted line (before chrome strip).
fn is_session_turn_start(line: &str) -> bool {
    super::display_text::has_leading_role_prefix(line.trim_start())
}

/// Format full preflight body for human/pretty display (T219 / T250 standard).
///
/// Pure string transform: section caps, F31 notices, chrome/role strip, blank
/// line after each emitted `---` header. Orphan headers with zero content omitted.
pub(crate) fn format_preflight_pretty_body(text: &str) -> String {
    format_preflight_pretty_body_with(text, &PrettyCaps::standard())
}

/// Format full preflight body with explicit [`PrettyCaps`] (T250).
pub(crate) fn format_preflight_pretty_body_with(text: &str, caps: &PrettyCaps) -> String {
    // Parse into (optional header, content lines) sections.
    // Text before the first header is a prologue (Other, no header).
    let mut sections: Vec<(Option<String>, Vec<String>)> = Vec::new();
    let mut cur_header: Option<String> = None;
    let mut cur_lines: Vec<String> = Vec::new();

    for line in text.split('\n') {
        let line = line.trim_end_matches('\r');
        if is_legacy_section_header(line) {
            if cur_header.is_some() || !cur_lines.is_empty() {
                sections.push((cur_header.take(), std::mem::take(&mut cur_lines)));
            }
            cur_header = Some(line.to_string());
        } else {
            cur_lines.push(line.to_string());
        }
    }
    if cur_header.is_some() || !cur_lines.is_empty() {
        sections.push((cur_header, cur_lines));
    }

    let mut out_parts: Vec<String> = Vec::new();
    let mut sessions_emitted: usize = 0;
    let mut sessions_skipped: usize = 0;
    // Index of the last emitted session part in `out_parts` (F31 M2 placement).
    let mut last_session_part_idx: Option<usize> = None;

    for (header, lines) in sections {
        let kind = header
            .as_deref()
            .map(classify_section_header)
            .unwrap_or(PrettySectionKind::Other);

        // Session cap: count session headers; skip overflow sessions.
        if kind == PrettySectionKind::Session && sessions_emitted >= caps.max_sessions {
            sessions_skipped += 1;
            continue;
        }

        let mut body_lines: Vec<String> = Vec::new();

        match kind {
            PrettySectionKind::Safety => {
                let raw_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
                let blocks = split_item_blocks(&raw_refs);
                let total = blocks.len();
                let take_n = total.min(caps.safety_max);
                // Compact first-line-caps Safety; standard does not line-cap Safety.
                let safety_cap = caps.first_line_only.then_some(caps.line_max);
                for block in blocks.into_iter().take(take_n) {
                    body_lines.push(emit_item_block(&block, caps, safety_cap));
                }
                if total > caps.safety_max {
                    let n = total - caps.safety_max;
                    body_lines.push(format!("+{n} more safety entries — ai-brains memory list"));
                }
            }
            PrettySectionKind::Session => {
                // Count logical turns on original lines (before chrome).
                // Continuation lines of multi-line turns belong to the open turn.
                let mut turn_count = 0usize;
                let mut turn_total = 0usize;
                let mut in_open_turn = false;
                let session_cap = Some(caps.line_max);
                for line in &lines {
                    if line.trim().is_empty() {
                        if in_open_turn && turn_count <= caps.turns_per_session {
                            // Preserve blank only when we still show the open turn body.
                            body_lines.push(String::new());
                        }
                        continue;
                    }
                    if is_session_turn_start(line) {
                        turn_total += 1;
                        in_open_turn = true;
                        if turn_count < caps.turns_per_session {
                            body_lines.push(display_pretty_line(line, session_cap));
                            turn_count += 1;
                        } else {
                            // Past cap: do not emit further turns or their continuations.
                            in_open_turn = false;
                        }
                    } else if in_open_turn && turn_count <= caps.turns_per_session {
                        // Continuation of a displayed turn (multi-line truncate_turn).
                        body_lines.push(display_pretty_line(line, session_cap));
                    }
                }
                // Drop trailing blanks introduced above.
                while body_lines.last().is_some_and(|l| l.is_empty()) {
                    body_lines.pop();
                }
                if turn_total > caps.turns_per_session {
                    let n = turn_total - caps.turns_per_session;
                    body_lines.push(format!("+{n} more turns in session"));
                }
            }
            PrettySectionKind::Index => {
                let mut index_items: Vec<String> = Vec::new();
                let mut other: Vec<String> = Vec::new();
                for line in &lines {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    // Numbered index lines: "1. …"
                    let is_numbered = trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && trimmed.contains(". ");
                    if is_numbered {
                        // Chrome/role strip on the summary portion after "N. " (never line-cap).
                        if let Some((num, rest)) = trimmed.split_once(". ") {
                            let stripped = display_pretty_line(rest, None);
                            index_items.push(format!("{num}. {stripped}"));
                        } else {
                            index_items.push(display_pretty_line(trimmed, None));
                        }
                    } else {
                        other.push(display_pretty_line(line, None));
                    }
                }
                let total = index_items.len();
                let take_n = total.min(caps.index_max);
                for item in index_items.into_iter().take(take_n) {
                    body_lines.push(item);
                }
                if total > caps.index_max {
                    let n = total - caps.index_max;
                    body_lines.push(format!("+{n} more via recall"));
                }
                body_lines.extend(other);
            }
            PrettySectionKind::Recent => {
                let raw_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
                let blocks = split_item_blocks(&raw_refs);
                let recent_cap = Some(caps.line_max);
                let mut taken = 0usize;
                for block in blocks {
                    if is_recall_hint_line(&block) {
                        continue;
                    }
                    if taken >= caps.recent_max {
                        continue;
                    }
                    let emitted = emit_item_block(&block, caps, recent_cap);
                    if emitted.lines().all(|l| l.trim().is_empty()) {
                        continue;
                    }
                    body_lines.push(emitted);
                    taken += 1;
                }
                // Preserve trailing recall hint (AI1 L4) — never line-capped.
                for line in &lines {
                    if is_recall_hint_line(line) {
                        body_lines.push(line.trim().to_string());
                    }
                }
            }
            PrettySectionKind::Other => {
                // Chrome/role-strip only. Do not line-cap or item-cap Other / `#` / `##`.
                for line in &lines {
                    if line.trim().is_empty() {
                        body_lines.push(String::new());
                    } else {
                        body_lines.push(display_pretty_line(line, None));
                    }
                }
                // Trim trailing blanks for orphan detection.
                while body_lines.last().is_some_and(|l| l.is_empty()) {
                    body_lines.pop();
                }
            }
        }

        // F37 / AC18: omit orphan headers with zero content after caps.
        let has_content = body_lines.iter().any(|l| !l.trim().is_empty());
        if header.is_some() && !has_content {
            continue;
        }

        if kind == PrettySectionKind::Session {
            sessions_emitted += 1;
        }

        let mut section_out = String::new();
        if let Some(h) = header {
            // F10: blank line after each emitted --- header.
            section_out.push_str(h.trim());
            section_out.push('\n');
            section_out.push('\n');
        }

        match kind {
            PrettySectionKind::Safety | PrettySectionKind::Recent => {
                section_out.push_str(&body_lines.join("\n\n"));
            }
            _ => {
                // Join with single newlines; preserve intentional blanks already in body_lines.
                section_out.push_str(&body_lines.join("\n"));
            }
        }

        // Trim trailing whitespace-only from section.
        let section_out = section_out.trim_end().to_string();
        if !section_out.is_empty() {
            if kind == PrettySectionKind::Session {
                last_session_part_idx = Some(out_parts.len());
            }
            out_parts.push(section_out);
        }
    }

    // Sessions count overflow notice (F31 / M2) — attach to last *session* part,
    // not the final out_part (which is often Memory Index / Recent).
    if sessions_skipped > 0 {
        let notice = format!("+{sessions_skipped} more sessions");
        if let Some(idx) = last_session_part_idx {
            if let Some(part) = out_parts.get_mut(idx) {
                part.push('\n');
                part.push_str(&notice);
            }
        } else {
            out_parts.push(notice);
        }
    }

    out_parts.join("\n\n")
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
    /// T220: summary JSON path — pure stdout JSON; status on stderr; no AskOnce.
    json_mode: bool,
}

/// Print preflight summary with honest Scope + dual vault/in-context counts (T214 F37).
///
/// When `gate.json_mode` (T220): emit pretty `PreflightSummaryJson` only on stdout;
/// harness human block is omitted; install status goes to stderr.
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

    // T241 F3: post-hoc discovery grants line (does not change 9-arg formatters).
    let grants_count = probe_discovery_active_count(ctx, global, project_id.as_ref());

    if gate.json_mode {
        let mut envelope = build_preflight_summary_json(
            global,
            project_id.as_ref(),
            projects_with_pinned,
            pinned_memories,
            active_sessions,
            hotspot_count,
            decision_count,
            constraint_count,
            context.word_count,
        );
        if let Some(n) = grants_count {
            envelope.grants_status = format_grants_status(n);
            if envelope.grants_status.is_some() {
                envelope.next_step = Some(POLICY_BOOTSTRAP_SOOT_SHORT.to_string());
            }
        }
        // Pretty summary JSON (memory-list family); T180 full path stays compact.
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        // M1: still run install side effects; never pollute stdout.
        append_harness_summary_and_maybe_prompt(&gate)?;
        return Ok(());
    }

    let mut lines = format_preflight_summary_lines(
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
    if let Some(n) = grants_count
        && let Some(line) = format_grants_incomplete_line(n)
    {
        lines.push(line);
    }
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
    // T220 F8: never print harness human block on stdout under JSON summary.
    if !gate.json_mode {
        for line in &harness_lines {
            println!("{}", line);
        }
    }
    // JSON path may still need install-hooks even when harness_lines empty (absent).
    if harness_lines.is_empty() && !gate.install_hooks {
        return Ok(());
    }
    // Human path: no harness rows → nothing further.
    if harness_lines.is_empty() {
        // install_hooks with empty report: still report honestly (never silent no-op).
        if gate.install_hooks {
            if home.is_some() {
                emit_status(
                    gate.json_mode,
                    "No ready harness present on machine for install-hooks (absent or already ok). next: ai-brains harness status",
                );
            } else {
                // AC8b / M1: USERPROFILE+HOME unset must not silently skip --install-hooks.
                emit_status(
                    gate.json_mode,
                    "No user home resolved (USERPROFILE/HOME unset); install-hooks skipped. next: ai-brains harness status",
                );
            }
            return Ok(());
        }
        return Ok(());
    }

    let prefs = home.as_ref().map(|h| load_prefs(h)).unwrap_or_default();
    // Per-harness decline: declining AGY must not suppress Grok (and vice versa).
    let ready_missing = ready_missing_not_declined(&report.harnesses, &prefs);

    // T220 F8: JSON path is always non-interactive (never AskOnce).
    let is_tty =
        !gate.json_mode && std::io::stdout().is_terminal() && std::io::stdin().is_terminal();
    // Declined harnesses are already filtered from ready_missing; pass declined=false
    // so remaining ready+missing backends (e.g. Grok when only Agy declined) still prompt.
    let decision = should_prompt_install(
        is_tty,
        gate.no_hook_prompt || gate.json_mode,
        gate.stdin_mode || gate.json_mode,
        !ready_missing.is_empty(),
        false,
        // JSON summary: never auto-install as a side effect of orientation JSON.
        if gate.json_mode {
            false
        } else {
            prefs.auto_install
        },
    );

    // Explicit --install-hooks: install **ready backends that are present on machine**
    // only (F24). Never write hooks when harness is absent (Codex CX2 P2).
    // F20: parse-refuse / write failure on explicit install → exit 1 (not silent 0).
    if gate.install_hooks {
        if let Some(h) = home.as_ref() {
            let mut installed_any = false;
            // T238/T253: OpenCode is install_ready. Claude/Codex writers are on
            // `harness install`; do not expand this --install-hooks loop (hotspot).
            for hid in [HarnessId::Agy, HarnessId::Grok, HarnessId::Opencode] {
                let row = report.harnesses.iter().find(|r| r.id == hid.as_str());
                let Some(row) = row else { continue };
                if !row.present || !row.install_ready {
                    continue;
                }
                if matches!(row.wiring, WiringStatus::Ok) {
                    emit_status(
                        gate.json_mode,
                        &format!(
                            "{} capture hooks already installed. next: ai-brains harness status",
                            hid.display_name()
                        ),
                    );
                    continue;
                }
                if matches!(
                    row.wiring,
                    WiringStatus::Missing
                        | WiringStatus::Partial
                        | WiringStatus::BackendPending
                        | WiringStatus::Unknown
                ) {
                    let result = match hid {
                        HarnessId::Agy => install_agy(h, false),
                        HarnessId::Grok => install_grok(h, false),
                        HarnessId::Opencode => install_opencode(h, false),
                        _ => continue,
                    };
                    report_preflight_install(
                        result,
                        hid.display_name(),
                        hid.as_str(),
                        &format!(
                            "Installed ready harness hooks ({}). next: ai-brains harness status",
                            hid.as_str()
                        ),
                        true,
                        gate.json_mode,
                    )?;
                    installed_any = true;
                }
            }
            if !installed_any {
                emit_status(
                    gate.json_mode,
                    "No ready harness present on machine for install-hooks (absent or already ok). next: ai-brains harness status",
                );
            }
        } else {
            // Non-empty harness report but no resolvable home (rare) — still not silent.
            emit_status(
                gate.json_mode,
                "No user home resolved (USERPROFILE/HOME unset); install-hooks skipped. next: ai-brains harness status",
            );
        }
        return Ok(());
    }

    match decision {
        PromptDecision::Skip => {}
        PromptDecision::PrintNextActionOnly => {
            if !ready_missing.is_empty() {
                let ids: Vec<&str> = ready_missing.iter().map(|h| h.id.as_str()).collect();
                // JSON path: no harness human block on stdout; skip next-action chatter.
                if !gate.json_mode {
                    println!(
                        "  next: ai-brains harness install --harness {} --dry-run",
                        ids.first().copied().unwrap_or("agy")
                    );
                }
            }
        }
        PromptDecision::AutoInstall => {
            if let Some(h) = home.as_ref() {
                // Soft path: print refuse/error but do not fail preflight (F9).
                for row in &ready_missing {
                    let result = match row.id.as_str() {
                        "agy" => install_agy(h, false),
                        "grok" => install_grok(h, false),
                        "opencode" => install_opencode(h, false),
                        _ => continue,
                    };
                    let label = parse_harness_id_soft(&row.id)
                        .map(|id| id.display_name())
                        .unwrap_or(row.id.as_str());
                    let _ = report_preflight_install(
                        result,
                        label,
                        row.id.as_str(),
                        &format!(
                            "Auto-installed {} capture hooks (auto_install=true).",
                            label
                        ),
                        false,
                        gate.json_mode,
                    );
                }
            }
        }
        PromptDecision::AskOnce => {
            // JSON mode forces non-interactive above; this arm is human-only.
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
                    for row in &ready_missing {
                        let result = match row.id.as_str() {
                            "agy" => install_agy(h, false),
                            "grok" => install_grok(h, false),
                            "opencode" => install_opencode(h, false),
                            _ => continue,
                        };
                        let label = parse_harness_id_soft(&row.id)
                            .map(|id| id.display_name())
                            .unwrap_or(row.id.as_str());
                        let _ = report_preflight_install(
                            result,
                            label,
                            row.id.as_str(),
                            &format!("Installed {} capture hooks.", label),
                            false,
                            gate.json_mode,
                        );
                    }
                }
            } else if let Some(h) = home.as_ref() {
                let mut p = load_prefs(h);
                for row in &ready_missing {
                    if let Ok(id) = parse_harness_id_soft(&row.id) {
                        p.mark_declined(id, chrono::Utc::now().to_rfc3339());
                    }
                }
                if let Err(e) = save_prefs(h, &p) {
                    eprintln!("could not persist decline: {e}");
                } else {
                    emit_status(
                        gate.json_mode,
                        "Declined. Re-enable with: ai-brains harness reset-decline --harness all",
                    );
                }
            }
        }
    }
    Ok(())
}

fn parse_harness_id_soft(s: &str) -> Result<HarnessId, ()> {
    match s {
        "agy" => Ok(HarnessId::Agy),
        "grok" => Ok(HarnessId::Grok),
        "opencode" => Ok(HarnessId::Opencode),
        "claude" => Ok(HarnessId::Claude),
        "codex" => Ok(HarnessId::Codex),
        _ => Err(()),
    }
}

/// Ready-to-install harnesses the user has not declined (per-harness filter).
///
/// Declining AGY must not suppress a ready+missing Grok row (and vice versa).
fn ready_missing_not_declined<'a>(
    harnesses: &'a [HarnessStatus],
    prefs: &HarnessHookPrefs,
) -> Vec<&'a HarnessStatus> {
    harnesses
        .iter()
        .filter(|h| {
            if !h.install_ready
                || !matches!(h.wiring, WiringStatus::Missing | WiringStatus::Partial)
            {
                return false;
            }
            match parse_harness_id_soft(&h.id) {
                Ok(id) => !prefs.is_declined(id),
                Err(()) => true,
            }
        })
        .collect()
}

/// Report harness install outcomes honestly (F28/AC21 — never claim success on Refused).
///
/// When `fail_on_error` is true (explicit `--install-hooks`), refuse/error returns
/// `Err` so the process exits non-zero (F20). Soft consent/auto paths keep preflight exit 0.
/// When `json_mode` (T220 M1), success/dry-run status lines go to stderr so stdout stays pure JSON.
fn report_preflight_install(
    result: Result<InstallOutcome, String>,
    harness_label: &str,
    harness_cli_id: &str,
    success_msg: &str,
    fail_on_error: bool,
    json_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match result {
        Ok(InstallOutcome::Installed { .. }) => {
            emit_status(json_mode, success_msg);
            Ok(())
        }
        Ok(InstallOutcome::DryRun { .. }) => {
            emit_status(
                json_mode,
                &format!("[dry-run] {harness_label} install planned (no writes)."),
            );
            Ok(())
        }
        Ok(InstallOutcome::BackendPending { plan }) => {
            eprintln!(
                "{harness_label} install backend pending; no files written. next: {}",
                plan.pending_track.unwrap_or("ai-brains harness status")
            );
            Ok(())
        }
        Ok(InstallOutcome::Refused { path, reason }) => {
            eprintln!(
                "Refused to rewrite {}: {}. Fix or remove the corrupt file, then re-run: ai-brains harness install --harness {harness_cli_id}",
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
            eprintln!("{harness_label} install failed: {e}");
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

    /// T241 AC9: post-hoc grants line for incomplete discovery; complete omits.
    #[test]
    fn format_grants_incomplete_line__empty_and_partial__contains_bootstrap() {
        let empty = format_grants_incomplete_line(0).expect("empty line");
        assert!(empty.contains("empty") || empty.contains("0 of 3"));
        assert!(empty.contains("policy bootstrap"));
        assert!(empty.contains(POLICY_BOOTSTRAP_SOOT_SHORT));

        let partial = format_grants_incomplete_line(2).expect("partial line");
        assert!(partial.contains("incomplete") && partial.contains("2 of 3"));
        assert!(partial.contains("policy bootstrap"));

        assert!(
            format_grants_incomplete_line(3).is_none(),
            "complete grants omit density line"
        );
    }

    /// T241 AC9: 9-arg formatters still compile; post-hoc append does not change arity.
    #[test]
    fn preflight_summary__post_hoc_grants_append__nine_arg_formatters() {
        let mut lines =
            format_preflight_summary_lines("Scope: project=aaa", false, None, 0, 0, 0, 0, 0, 0);
        let before = lines.len();
        if let Some(g) = format_grants_incomplete_line(0) {
            lines.push(g);
        }
        assert_eq!(lines.len(), before + 1);
        assert!(lines.last().unwrap().contains("policy bootstrap"));

        let env = build_preflight_summary_json(false, None, None, 0, 0, 0, 0, 0, 0);
        assert!(env.grants_status.is_none());
        assert!(env.next_step.is_none());
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

    // ---------------------------------------------------------------------------
    // T220 AC9 — pure summary JSON envelope (global / project / none)
    // ---------------------------------------------------------------------------

    #[test]
    fn build_preflight_summary_json__global__projects_key_present() {
        let env = build_preflight_summary_json(true, None, Some(2), 5, 1, 3, 4, 1, 100);
        assert_eq!(env.api_version, "1");
        assert_eq!(env.scope, "global");
        assert_eq!(env.project_id, None);
        assert_eq!(env.projects, Some(2));
        assert_eq!(env.pinned, 5);
        assert_eq!(env.active_sessions, 1);
        assert_eq!(env.in_context_hotspots, 3);
        assert_eq!(env.in_context_decisions, 4);
        assert_eq!(env.in_context_constraints, 1);
        assert_eq!(env.word_count, 100);
        let s = serde_json::to_string_pretty(&env).expect("serialize");
        assert!(
            s.contains("\"projects\""),
            "global must emit projects key: {s}"
        );
        assert!(
            s.contains("\"api_version\": \"1\""),
            "api_version; got:\n{s}"
        );
        let v: serde_json::Value = serde_json::from_str(&s).expect("parse");
        assert_eq!(v["scope"], "global");
        assert!(v["project_id"].is_null());
        assert_eq!(v["projects"], 2);
    }

    #[test]
    fn build_preflight_summary_json__project_scoped__omits_projects_key() {
        let pid = ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let env = build_preflight_summary_json(false, Some(&pid), None, 2, 0, 0, 1, 0, 42);
        assert_eq!(env.scope, "project");
        assert_eq!(env.project_id, Some(pid.to_string()));
        assert_eq!(env.projects, None);
        let s = serde_json::to_string(&env).expect("serialize");
        assert!(
            !s.contains("\"projects\""),
            "project-scoped must omit projects key (F34); got: {s}"
        );
        let v: serde_json::Value = serde_json::from_str(&s).expect("parse");
        assert_eq!(v["scope"], "project");
        assert_eq!(v["project_id"], pid.to_string());
        assert!(v.get("projects").is_none(), "no projects key: {v}");
    }

    #[test]
    fn build_preflight_summary_json__none_scope__no_projects_key() {
        let env = build_preflight_summary_json(false, None, None, 0, 0, 0, 0, 0, 0);
        assert_eq!(env.scope, "none");
        assert_eq!(env.project_id, None);
        assert_eq!(env.projects, None);
        let s = serde_json::to_string(&env).expect("serialize");
        assert!(
            !s.contains("\"projects\""),
            "scope none must omit projects key; got: {s}"
        );
        let v: serde_json::Value = serde_json::from_str(&s).expect("parse");
        assert_eq!(v["scope"], "none");
        assert!(v["project_id"].is_null());
        assert!(v.get("projects").is_none());
    }

    #[test]
    fn ready_missing_not_declined__agy_declined__keeps_grok() {
        // Declining AGY must not suppress ready+missing Grok (per-harness decline).
        let statuses = vec![
            HarnessStatus {
                id: "agy".into(),
                display_name: "AGY".into(),
                present: true,
                binary: None,
                home_path: Some("/tmp/.gemini".into()),
                wiring: WiringStatus::Missing,
                install_ready: true,
                targets: vec![],
                next_action: "install agy".into(),
            },
            HarnessStatus {
                id: "grok".into(),
                display_name: "Grok".into(),
                present: true,
                binary: None,
                home_path: Some("/tmp/.grok".into()),
                wiring: WiringStatus::Missing,
                install_ready: true,
                targets: vec![],
                next_action: "install grok".into(),
            },
        ];
        let mut prefs = HarnessHookPrefs::default();
        prefs.mark_declined(HarnessId::Agy, "2026-01-01T00:00:00Z");
        let ready = ready_missing_not_declined(&statuses, &prefs);
        assert_eq!(ready.len(), 1, "only Grok should remain: {ready:?}");
        assert_eq!(ready[0].id, "grok");

        // Both declined → empty (should_prompt Skip via !has_ready_missing).
        prefs.mark_declined(HarnessId::Grok, "2026-01-01T00:00:00Z");
        let ready_both = ready_missing_not_declined(&statuses, &prefs);
        assert!(
            ready_both.is_empty(),
            "all declined → no prompt candidates: {ready_both:?}"
        );

        // Neither declined → both candidates.
        let prefs_none = HarnessHookPrefs::default();
        let ready_all = ready_missing_not_declined(&statuses, &prefs_none);
        assert_eq!(ready_all.len(), 2);
    }

    // ---------------------------------------------------------------------------
    // T219 — format_preflight_pretty_body pure units (AC6 / AC14 / AC18)
    // ---------------------------------------------------------------------------

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__over_cap_sections__f31_wording() {
        // Safety: 10 blank-separated items → cap 8 + safety notice (not index wording).
        let mut safety_items = Vec::new();
        for i in 1..=10 {
            safety_items.push(format!("CONSTRAINT: safety item {i}"));
        }
        let safety = format!(
            "--- Repository Bearings & Safety ---\n{}",
            safety_items.join("\n\n")
        );

        // Index: 18 numbered lines → cap 15 + recall notice.
        let mut index_lines = vec!["--- Memory Index (Briefing) ---".to_string()];
        for i in 1..=18 {
            index_lines.push(format!("{i}. ASSISTANT: DECISION: index item {i}"));
        }
        let index = index_lines.join("\n");

        // Sessions: 4 sessions × 8 turns → max 3 sessions + turn notices + sessions notice.
        let mut sessions = Vec::new();
        for s in 1..=4 {
            let mut lines = vec![format!(
                "--- Session: 00000000-0000-0000-0000-00000000000{s} ---"
            )];
            for t in 1..=8 {
                lines.push(format!("ASSISTANT: turn {t} content for session {s}"));
            }
            sessions.push(lines.join("\n"));
        }

        let text = format!("{safety}\n\n{}\n\n{index}", sessions.join("\n\n"));
        let out = format_preflight_pretty_body(&text);

        assert!(
            out.contains("+2 more safety entries — ai-brains memory list"),
            "AC6 safety F31 wording; got:\n{out}"
        );
        assert!(
            out.contains("+3 more via recall"),
            "AC6 index F31 wording; got:\n{out}"
        );
        assert!(
            out.contains("+2 more turns in session"),
            "AC6 session turns F31; got:\n{out}"
        );
        assert!(
            out.contains("+1 more sessions"),
            "AC6 sessions count F31; got:\n{out}"
        );
        // L1: exact index N is 3 — reject wrong N without tautology.
        assert!(
            !out.contains("+2 more via recall"),
            "index overflow must be +3 not +2; got:\n{out}"
        );
        // M2: sessions notice must appear before Memory Index, not only as trailing index chrome.
        let sessions_notice_at = out
            .find("+1 more sessions")
            .expect("+1 more sessions present");
        let index_at = out
            .find("--- Memory Index (Briefing) ---")
            .expect("Memory Index header present");
        assert!(
            sessions_notice_at < index_at,
            "M2: sessions notice must precede Memory Index; notice@{sessions_notice_at} index@{index_at}\n{out}"
        );
        // No displayed index line begins with ASSISTANT:
        for line in out.lines() {
            if line
                .trim()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
                && line.contains(". ")
            {
                let after = line.split_once(". ").map(|(_, r)| r).unwrap_or(line);
                assert!(
                    !after.starts_with("ASSISTANT:"),
                    "index line must strip role; got {line}"
                );
            }
            if line.starts_with("ASSISTANT:") {
                panic!("session/display line must strip ASSISTANT: prefix; got {line}");
            }
        }
        // Blank line after --- header.
        assert!(
            out.contains("--- Repository Bearings & Safety ---\n\n"),
            "F10 blank after header; got:\n{out}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__governed_markdown__preserves_hash_headers() {
        // AC14: ## lines are not treated as --- section headers; preserved.
        let text = "# Project Briefing (governed)\n\n## Decisions\n\n- ship T219\n\n## Constraints\n\n- no unwrap";
        let out = format_preflight_pretty_body(text);
        assert!(
            out.contains("## Decisions"),
            "must preserve ##; got:\n{out}"
        );
        assert!(
            out.contains("## Constraints"),
            "must preserve ##; got:\n{out}"
        );
        assert!(
            out.contains("# Project Briefing (governed)"),
            "must preserve #; got:\n{out}"
        );
        assert!(!out.contains("---"), "must not invent --- headers");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__orphan_header__omitted() {
        // AC18: --- Header --- with zero following items is omitted.
        let text =
            "--- Orphan Section ---\n\n--- Repository Bearings & Safety ---\n\nCONSTRAINT: keep me";
        let out = format_preflight_pretty_body(text);
        assert!(
            !out.contains("Orphan Section"),
            "orphan header must be omitted; got:\n{out}"
        );
        assert!(
            out.contains("--- Repository Bearings & Safety ---"),
            "non-empty section kept; got:\n{out}"
        );
        assert!(out.contains("CONSTRAINT: keep me"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__role_strip_on_session_and_index() {
        let text = "--- Session: aaaa ---\nASSISTANT: DECISION: alpha\nUSER: beta\n\n--- Memory Index (Briefing) ---\n1. ASSISTANT: DECISION: gamma -- 1 day ago";
        let out = format_preflight_pretty_body(text);
        assert!(out.contains("DECISION: alpha"));
        assert!(out.contains("beta"));
        assert!(out.contains("1. DECISION: gamma"));
        assert!(!out.lines().any(|l| l.starts_with("ASSISTANT:")));
        assert!(!out.lines().any(|l| l.starts_with("USER:")));
    }

    /// M1: multi-line turns (truncate_turn shape) count as one turn each.
    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__multiline_turns__cap_by_role_starts() {
        // 3 logical turns, each 3 physical lines → 9 lines total.
        // Cap is 6 turns; must keep all 3 turns fully (no false +N more turns).
        let text = "\
--- Session: bbbb ---
ASSISTANT: line1 of turn1
continuation of turn1
more of turn1
USER: line1 of turn2
continuation of turn2
more of turn2
SYSTEM: line1 of turn3
continuation of turn3
more of turn3
";
        let out = format_preflight_pretty_body(text);
        assert!(
            !out.contains("more turns in session"),
            "3 multi-line turns must not trip the 6-turn cap; got:\n{out}"
        );
        assert!(out.contains("line1 of turn1"));
        assert!(out.contains("more of turn1"));
        assert!(out.contains("line1 of turn2"));
        assert!(out.contains("more of turn3"));
        // 7 logical single-line turns → overflow +1.
        let mut lines = vec!["--- Session: cccc ---".to_string()];
        for t in 1..=7 {
            lines.push(format!("ASSISTANT: only turn {t}"));
        }
        let out7 = format_preflight_pretty_body(&lines.join("\n"));
        assert!(
            out7.contains("+1 more turns in session"),
            "7 role-starts must overflow by 1; got:\n{out7}"
        );
        assert!(out7.contains("only turn 6"));
        assert!(!out7.contains("only turn 7"));
    }

    /// M2: +N more sessions attaches to last session, before later Index section.
    #[test]
    #[allow(non_snake_case)]
    fn format_preflight_pretty_body__sessions_notice__before_index() {
        let mut parts = Vec::new();
        for s in 1..=4 {
            parts.push(format!(
                "--- Session: 00000000-0000-0000-0000-00000000000{s} ---\nASSISTANT: s{s} turn"
            ));
        }
        parts.push(
            "--- Memory Index (Briefing) ---\n1. ASSISTANT: DECISION: keep index".to_string(),
        );
        let out = format_preflight_pretty_body(&parts.join("\n\n"));
        assert!(out.contains("+1 more sessions"), "got:\n{out}");
        let notice = out.find("+1 more sessions").expect("notice");
        let index = out.find("--- Memory Index (Briefing) ---").expect("index");
        assert!(
            notice < index,
            "sessions notice must be before Memory Index; notice@{notice} index@{index}\n{out}"
        );
        assert!(out.contains("1. DECISION: keep index"));
    }

    // ---------------------------------------------------------------------------
    // T250 — PrettyCaps line-cap, chrome strip, compact (AC1–AC9)
    // ---------------------------------------------------------------------------

    #[test]
    fn pretty_caps_standard__t219_item_caps() {
        let c = PrettyCaps::standard();
        assert_eq!(c.safety_max, 8);
        assert_eq!(c.turns_per_session, 6);
        assert_eq!(c.max_sessions, 3);
        assert_eq!(c.index_max, 15);
        assert_eq!(c.recent_max, 3);
        assert_eq!(c.line_max, 140);
        assert!(!c.first_line_only);
    }

    #[test]
    fn format_preflight_pretty_body_with__session_line_over_140__capped_ellipsis() {
        let body = "x".repeat(200);
        let text = format!("--- Session: aaaa ---\nASSISTANT: {body}");
        let out = format_preflight_pretty_body_with(&text, &PrettyCaps::standard());
        let display = out
            .lines()
            .find(|l| l.contains('x'))
            .expect("session body line");
        assert!(
            display.chars().count() <= 140,
            "AC1 session line must be ≤140; got {} `{display}`",
            display.chars().count()
        );
        assert!(display.ends_with('…'), "truncated session must end with …");
        assert!(!display.starts_with("ASSISTANT:"));
    }

    #[test]
    fn format_preflight_pretty_body_with__recent_line_over_140__capped_ellipsis() {
        let body = "x".repeat(200);
        let text = format!(
            "--- Most Recent Memories ---\n\n(just now) ASSISTANT: {body}\n\n(Use 'recall' to fetch details for other index items)"
        );
        let out = format_preflight_pretty_body_with(&text, &PrettyCaps::standard());
        let display = out
            .lines()
            .find(|l| l.contains('x'))
            .expect("recent body line");
        assert!(
            display.chars().count() <= 140,
            "AC1 recent line must be ≤140; got {} `{display}`",
            display.chars().count()
        );
        assert!(display.ends_with('…'), "truncated recent must end with …");
        assert!(out.contains("(Use 'recall' to fetch details for other index items)"));
    }

    #[test]
    fn format_preflight_pretty_body_with__session_line_80__unchanged() {
        let body = "y".repeat(80);
        let text = format!("--- Session: aaaa ---\nASSISTANT: {body}");
        let out = format_preflight_pretty_body_with(&text, &PrettyCaps::standard());
        assert!(
            out.contains(&body),
            "80-char line must be unchanged; got:\n{out}"
        );
        assert!(
            !out.contains('…'),
            "80-char line must not truncate; got:\n{out}"
        );
    }

    #[test]
    fn pretty_caps_compact__item_caps_and_f31_and_recent_hint() {
        let safety = "--- Repository Bearings & Safety ---\n\
CONSTRAINT: safety item 1\n\n\
CONSTRAINT: safety item 2\n\n\
CONSTRAINT: safety item 3\n\n\
CONSTRAINT: safety item 4\n\n\
CONSTRAINT: safety item 5\n\n\
CONSTRAINT: safety item 6\n\n\
CONSTRAINT: safety item 7\n\n\
CONSTRAINT: safety item 8\n\n\
CONSTRAINT: safety item 9\n\n\
CONSTRAINT: safety item 10";
        let sessions = "--- Session: 00000000-0000-0000-0000-000000000001 ---\n\
ASSISTANT: turn 1 content for session 1\n\
ASSISTANT: turn 2 content for session 1\n\
ASSISTANT: turn 3 content for session 1\n\
ASSISTANT: turn 4 content for session 1\n\n\
--- Session: 00000000-0000-0000-0000-000000000002 ---\n\
ASSISTANT: turn 1 content for session 2\n\
ASSISTANT: turn 2 content for session 2";
        let index = "--- Memory Index (Briefing) ---\n\
1. ASSISTANT: DECISION: index item 1\n\
2. ASSISTANT: DECISION: index item 2\n\
3. ASSISTANT: DECISION: index item 3\n\
4. ASSISTANT: DECISION: index item 4\n\
5. ASSISTANT: DECISION: index item 5\n\
6. ASSISTANT: DECISION: index item 6\n\
7. ASSISTANT: DECISION: index item 7\n\
8. ASSISTANT: DECISION: index item 8";
        let recent = "--- Most Recent Memories ---\n\n\
(just now) ASSISTANT: first item line\nsecond line of first item\n\n\
(just now) ASSISTANT: second item line\nsecond line of second item\n\n\
(just now) ASSISTANT: third item should drop\n\n\
(Use 'recall' to fetch details for other index items)";
        let text = format!("{safety}\n\n{sessions}\n\n{index}\n\n{recent}");
        let out = format_preflight_pretty_body_with(&text, &PrettyCaps::compact());

        assert!(
            out.contains("+7 more safety entries — ai-brains memory list"),
            "AC3 compact safety F31; got:\n{out}"
        );
        assert!(out.contains("safety item 3"), "got:\n{out}");
        assert!(
            !out.contains("safety item 4"),
            "compact safety cap 3; got:\n{out}"
        );
        assert!(
            out.contains("+2 more turns in session"),
            "AC3 compact turns F31; got:\n{out}"
        );
        assert!(out.contains("turn 2 content for session 1"), "got:\n{out}");
        assert!(
            !out.contains("turn 3 content for session 1"),
            "compact turns cap 2; got:\n{out}"
        );
        assert!(
            out.contains("+1 more sessions"),
            "AC3 compact sessions F31; got:\n{out}"
        );
        assert!(
            !out.contains("turn 1 content for session 2"),
            "compact max_sessions 1; got:\n{out}"
        );
        assert!(
            out.contains("+3 more via recall"),
            "AC3 compact index F31; got:\n{out}"
        );
        assert!(out.contains("5. DECISION: index item 5"), "got:\n{out}");
        assert!(
            !out.contains("index item 6"),
            "compact index cap 5; got:\n{out}"
        );
        assert!(out.contains("first item line"), "got:\n{out}");
        assert!(out.contains("second item line"), "got:\n{out}");
        assert!(
            !out.contains("second line of first item"),
            "compact first_line_only Recent; got:\n{out}"
        );
        assert!(
            !out.contains("third item should drop"),
            "compact recent cap 2; got:\n{out}"
        );
        assert!(
            out.contains("(Use 'recall' to fetch details for other index items)"),
            "AC3 keep recall hint; got:\n{out}"
        );
    }

    #[test]
    fn strip_pretty_chrome__just_now_assistant__strips_to_decision() {
        assert_eq!(
            strip_pretty_chrome("(just now) ASSISTANT: DECISION: x"),
            "DECISION: x"
        );
    }

    #[test]
    fn strip_pretty_chrome__ten_hr_ago_user__strips_to_hi() {
        assert_eq!(strip_pretty_chrome("(10 hr ago) USER: hi"), "hi");
    }

    #[test]
    fn strip_pretty_chrome__999_mo_ago_assistant__strips_to_x() {
        assert_eq!(strip_pretty_chrome("(999 mo ago) ASSISTANT: x"), "x");
    }

    #[test]
    fn strip_pretty_chrome__mid_line_assistant__unchanged() {
        assert_eq!(
            strip_pretty_chrome("text ASSISTANT: x"),
            "text ASSISTANT: x"
        );
    }

    #[test]
    fn strip_pretty_chrome__lowercase_assistant__unchanged() {
        assert_eq!(strip_pretty_chrome("assistant: leave"), "assistant: leave");
    }

    #[test]
    fn strip_pretty_chrome__inner_33_chars_with_role__fail_closed() {
        let inner = "abcdefghijklmnopqrstuvwxyz0123456";
        assert_eq!(inner.chars().count(), 33);
        let line = format!("({inner}) ASSISTANT: x");
        assert_eq!(strip_pretty_chrome(&line), line.as_str());
    }

    #[test]
    fn strip_pretty_chrome__no_whitespace_after_paren__fail_closed() {
        assert_eq!(
            strip_pretty_chrome("(note)ASSISTANT: body"),
            "(note)ASSISTANT: body"
        );
    }

    #[test]
    fn format_preflight_pretty_body__long_header_and_notice__not_line_capped() {
        let long_h = "H".repeat(160);
        let header = format!("--- {long_h} ---");
        let text = format!("{header}\nkeep-other-body");
        let out = format_preflight_pretty_body(&text);
        assert!(
            out.contains(&long_h),
            "AC6 header must stay full length; got:\n{out}"
        );

        let safety = "--- Repository Bearings & Safety ---\n\
CONSTRAINT: s1\n\nCONSTRAINT: s2\n\nCONSTRAINT: s3\n\nCONSTRAINT: s4\n\n\
CONSTRAINT: s5\n\nCONSTRAINT: s6\n\nCONSTRAINT: s7\n\nCONSTRAINT: s8\n\n\
CONSTRAINT: s9\n\nCONSTRAINT: s10";
        let out_s = format_preflight_pretty_body(safety);
        let notice = "+2 more safety entries — ai-brains memory list";
        assert!(
            out_s.contains(notice),
            "AC6 full F31 wording; got:\n{out_s}"
        );
        let notice_line = out_s
            .lines()
            .find(|l| l.contains("more safety entries"))
            .expect("notice line");
        assert_eq!(notice_line, notice);
        assert!(!notice_line.contains('…'));
    }

    #[test]
    fn format_preflight_pretty_body_with__governed_hash_headers__full_body_both_caps() {
        let body = "z".repeat(200);
        let text = format!(
            "# Project Briefing (governed)\n\n## Decisions\n\n{body}\n\n## Constraints\n\n- no unwrap"
        );
        let std = format_preflight_pretty_body_with(&text, &PrettyCaps::standard());
        assert!(
            std.contains("## Decisions"),
            "standard must keep ##; got:\n{std}"
        );
        assert!(
            std.contains("# Project Briefing (governed)"),
            "standard must keep #; got:\n{std}"
        );
        assert!(
            std.contains(&body),
            "AC7 200-char ## body stays full on standard; got:\n{std}"
        );
        let cmp = format_preflight_pretty_body_with(&text, &PrettyCaps::compact());
        assert!(
            cmp.contains("## Decisions"),
            "compact must keep ##; got:\n{cmp}"
        );
        assert!(
            cmp.contains("# Project Briefing (governed)"),
            "compact must keep #; got:\n{cmp}"
        );
        assert!(
            cmp.contains(&body),
            "AC7 200-char ## body stays full on compact; got:\n{cmp}"
        );
    }
}
