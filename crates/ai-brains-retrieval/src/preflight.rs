use std::collections::HashSet;
use std::path::PathBuf;

use crate::GraphSearch;
use crate::ansi::strip_ansi;
use crate::errors::Result;
use crate::preflight_global::{
    GLOBAL_INDEX_FETCH, GLOBAL_INDEX_MAX, GLOBAL_INDEX_PER_PROJECT, GLOBAL_RECENT_MAX,
    GLOBAL_RECENT_PER_PROJECT, GLOBAL_SAFETY_FETCH, GLOBAL_SAFETY_MAX, GLOBAL_SAFETY_PER_PROJECT,
    GLOBAL_SESSION_MAX, GLOBAL_SESSION_PER_PROJECT, prefix_first_line, project_key, project_tag,
    span_count, take_round_robin,
};
use crate::privacy_filter::is_injectable_privacy;
use crate::ranking::{PinKind, classify_pin_kind};
use crate::session_chrome::{bound_not_in_sql, index_marker_glob_sql};
use crate::sessions::active_sessions;
use crate::word_budget::{
    content_word_count, trim_to_word_budget, trim_to_word_budget_no_sentinel, word_count,
};
use ai_brains_contracts::briefings::{
    BriefingScopeDto, BriefingWarningDto, BudgetReportDto, FreshnessSummaryDto,
    ProjectBriefingPacket,
};
use ai_brains_control_plane::{
    BudgetConfig, ProjectBriefingRequest, ScopeResolveInput, StorePorts, SystemClock,
    build_project_briefing, make_principal, render_project_markdown,
};
use ai_brains_core::ids::PrincipalId;
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::VaultConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightContext {
    pub text: String,
    pub word_count: usize,
    /// Distinct non-unknown project ids in the emitted global body (T264 F9).
    /// `Some` iff `global`; governed empty-global is `Some(0)`.
    pub in_context_project_span: Option<u32>,
}

/// Env flag: `AI_BRAINS_GOVERNED_BRIEFING=1` enables typed ProjectBriefingPacket path.
///
/// When enabled, `preflight` builds a governed Project packet (policy + scope
/// authority + budget). Empty-state: unresolved/global → empty packet + warning;
/// grant denial → empty authority sections (`denied` / warnings). See
/// `Docs/OPERATIONS.md` (Governed briefings) and CLI `ai-brains briefing` /
/// `ai-brains query` for the packet / progressive surfaces.
///
/// **One-cycle residual (T152-R1-07):** env-only (and `Option<bool>` API override).
/// Config-file / `.env` loader key `governed_briefing = true` is intentionally not
/// wired yet — process env or explicit option only for this compatibility cycle.
/// Default is off (legacy string-scrape preflight).
pub fn governed_briefing_enabled() -> bool {
    match std::env::var("AI_BRAINS_GOVERNED_BRIEFING") {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// Config-style override: when `Some(true)` force governed; `Some(false)` force legacy;
/// `None` defers to [`governed_briefing_enabled`] env flag.
pub fn build_preflight_with_options(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    scope_paths: Option<Vec<String>>,
    global: bool,
    governed_briefing: Option<bool>,
) -> Result<PreflightContext> {
    let use_governed = governed_briefing.unwrap_or_else(governed_briefing_enabled);
    if use_governed {
        return build_governed_preflight(conn, max_words, project_id, global);
    }
    build_legacy_preflight(conn, graph, max_words, project_id, scope_paths, global)
}

pub fn build_preflight(
    conn: &VaultConnection,
    graph: Option<&GraphSearch>,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    scope_paths: Option<Vec<String>>,
    global: bool,
) -> Result<PreflightContext> {
    build_preflight_with_options(
        conn,
        graph,
        max_words,
        project_id,
        scope_paths,
        global,
        None,
    )
}

/// Typed Project briefing path — routes through control-plane `build_project_briefing`
/// with production policy + T151 scope resolve (no raw SQL authority bypass).
///
/// **Principal:** optional `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` (UUID) selects the
/// Human principal; otherwise a well-known System principal is used. In both
/// cases the principal must be **registered** and hold `ReadDecisions` /
/// `ReadConclusions` grants for the resolved scope — otherwise authority
/// sections are empty (denied), while the governed markdown header still renders.
fn build_governed_preflight(
    conn: &VaultConnection,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    global: bool,
) -> Result<PreflightContext> {
    if global || project_id.is_none() {
        // Empty-state: no project id / global mode → empty packet with warning.
        let packet = empty_governed_packet(
            if global { "global" } else { "unresolved" },
            if global {
                "Global governed preflight returns empty project packet"
            } else {
                "Project id unavailable; governed preflight returned empty packet"
            },
        );
        let text = render_governed_packet_markdown(&packet, max_words);
        return Ok(PreflightContext {
            // F32: content words exclude F2b trailing sentinel chrome.
            word_count: content_word_count(&text),
            text,
            // F9: governed empty-global → Some(0); unresolved is not global.
            in_context_project_span: if global { Some(0) } else { None },
        });
    }

    let store = SqliteEventStore::new(conn.clone());
    let ports = StorePorts::from_store(store);
    let clock = SystemClock;
    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let principal = preflight_principal();

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let resolve = ScopeResolveInput {
        cwd,
        explicit_project_id: project_id,
        force_personal: false,
        personal_user_id: None,
        git_metadata: None,
    };

    let packet = build_project_briefing(
        None::<&ai_brains_control_plane::StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal,
            resolve,
            budget: BudgetConfig {
                max_words,
                ..BudgetConfig::default()
            },
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )?;

    let text = render_governed_packet_markdown(&packet, max_words);
    Ok(PreflightContext {
        // F32: content words exclude F2b trailing sentinel chrome.
        word_count: content_word_count(&text),
        text,
        in_context_project_span: None,
    })
}

/// Resolve the principal used for governed preflight policy evaluation.
fn preflight_principal() -> ai_brains_core::principal::Principal {
    // Optional override: `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID=<uuid>`
    if let Ok(raw) = std::env::var("AI_BRAINS_PREFLIGHT_PRINCIPAL_ID") {
        let trimmed = raw.trim();
        if let Ok(u) = uuid::Uuid::parse_str(trimmed) {
            return make_principal(
                PrincipalKind::Human,
                PrincipalId::from_uuid(u),
                "preflight-human",
            );
        }
    }
    // Well-known System principal for local-vault preflight (must be registered + granted).
    make_principal(
        PrincipalKind::System,
        PrincipalId::from_uuid(uuid::Uuid::from_u128(
            0xA1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2_A1_B2,
        )),
        "preflight-system",
    )
}

fn empty_governed_packet(scope_key: &str, warning: &str) -> ProjectBriefingPacket {
    ProjectBriefingPacket {
        api_version: ai_brains_contracts::briefings::API_VERSION.to_string(),
        briefing_id: uuid::Uuid::nil().to_string(),
        kind: "Project".into(),
        scope: BriefingScopeDto {
            scope_key: scope_key.into(),
            confidence: "Low".into(),
            warnings: vec![warning.into()],
            alternatives: Vec::new(),
            authoritative: false,
        },
        handoff: None,
        decisions: Vec::new(),
        conclusions: Vec::new(),
        constraints: Vec::new(),
        warnings: vec![BriefingWarningDto {
            kind: "other".into(),
            message: warning.into(),
            subject_id: None,
            subject_kind: None,
        }],
        freshness: FreshnessSummaryDto {
            total_sources: 0,
            fresh_count: 0,
            stale_count: 0,
            unavailable_count: 0,
            worst_state: "Unknown".into(),
        },
        ledgerful: None,
        evidence_handles: Vec::new(),
        budget: BudgetReportDto {
            max_words: 0,
            used_words: 0,
            truncated_sections: Vec::new(),
            more_available: false,
        },
        generated_at: None,
        denied: false,
        denial_reason: None,
        denial_hint: None,
    }
}

/// Render governed preflight markdown (header marks governed path; body from control-plane shape).
fn render_governed_packet_markdown(packet: &ProjectBriefingPacket, max_words: usize) -> String {
    // Reuse control-plane markdown then re-tag the header for dual-path tests/CLI recognition.
    let body = render_project_markdown(packet);
    let retagged = body.replacen("# Project Briefing", "# Project Briefing (governed)", 1);
    trim_to_word_budget(&retagged, max_words)
}

fn build_legacy_preflight(
    conn: &VaultConnection,
    _graph: Option<&GraphSearch>,
    max_words: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    scope_paths: Option<Vec<String>>,
    global: bool,
) -> Result<PreflightContext> {
    let active = if global {
        active_sessions(conn, None)?
    } else {
        active_sessions(conn, project_id)?
    };
    let conn = conn.lock()?;

    let project_id_str: Option<String> = project_id.map(|p| p.to_string());

    let mut sections = Vec::new();

    // --- Ledgerful Blended Section (New) ---
    let mut has_cg_intelligence = false;
    if !global
        && let Some(ref pid) = project_id_str
        && let Some(cg_context) = query_ledgerful(pid, scope_paths.as_ref())
    {
        sections.push(cg_context);
        has_cg_intelligence = true;
    }

    // --- Onboarding & Safety Section (Max 15% of budget) ---
    let onboarding_budget = (max_words * 15) / 100;
    let mut safety_raw: Vec<(String, String, (Option<String>, String))> = Vec::new(); // content, ts, (project, memory_id)
    let mut span_ids: Vec<String> = Vec::new();

    let safety_sql = if global {
        "SELECT m.memory_id, m.content, m.updated_at, COALESCE(m.project_id, s.project_id)
         FROM memory_projection m
         LEFT JOIN session_projection s ON m.session_id = s.session_id
         WHERE (m.content LIKE '%CONSTRAINT:%' OR m.content LIKE '%INVARIANT:%' OR m.content LIKE '%HOTSPOT:%')
         AND m.status = 'pinned'
         ORDER BY m.updated_at DESC LIMIT 40"
    } else {
        "SELECT m.memory_id, m.content, m.updated_at, COALESCE(m.project_id, s.project_id)
         FROM memory_projection m
         LEFT JOIN session_projection s ON m.session_id = s.session_id
         WHERE (m.content LIKE '%CONSTRAINT:%' OR m.content LIKE '%INVARIANT:%' OR m.content LIKE '%HOTSPOT:%')
         AND m.status = 'pinned'
         AND (s.project_id = ? OR m.project_id = ?)
         ORDER BY m.updated_at DESC LIMIT 10"
    };
    const _: () = assert!(GLOBAL_SAFETY_FETCH == 40);
    let mut safety_stmt = conn.prepare(safety_sql)?;
    let mut safety_rows = if global {
        safety_stmt.query([])?
    } else if let Some(ref pid) = project_id_str {
        safety_stmt.query(rusqlite::params![pid, pid])?
    } else {
        // Return nothing if no project_id provided and not global
        safety_stmt.query(rusqlite::params![
            Option::<String>::None,
            Option::<String>::None
        ])?
    };
    while let Some(row) = safety_rows.next()? {
        let memory_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let updated_at: String = row.get(2)?;
        let item_project: Option<String> = row.get(3)?;

        // Suppress vault HOTSPOTs if we already have fresh intelligence from the bridge
        if has_cg_intelligence && content.contains("HOTSPOT:") {
            continue;
        }

        safety_raw.push((strip_ansi(&content), updated_at, (item_project, memory_id)));
    }

    // Deduplicate hotspot entries by file path: keep only the most recent score per path.
    // ORDER BY updated_at DESC ensures first occurrence is the freshest.
    let mut safety_entries = dedup_hotspots_keyed(safety_raw);
    if global {
        safety_entries = take_round_robin(
            safety_entries,
            |(_, (pid, _))| project_key(pid.as_deref()),
            GLOBAL_SAFETY_PER_PROJECT,
            GLOBAL_SAFETY_MAX,
        );
    }

    // Rebuild safety_ids from emitted entries so capped-out pins remain visible in Index (T272)
    let safety_ids: HashSet<String> = safety_entries
        .iter()
        .map(|(_, (_, id))| id.clone())
        .collect();

    let mut safety_cleaned: Vec<String> = Vec::new();
    let mut safety_for_skip: Vec<String> = Vec::new();
    if !safety_entries.is_empty() {
        for (entry, (pid, _)) in &safety_entries {
            let stripped = entry.strip_prefix("ASSISTANT: ").unwrap_or(entry);
            safety_for_skip.push(stripped.to_string());
            let tagged = if global {
                if let Some(id) = pid.as_deref() {
                    span_ids.push(id.to_string());
                }
                prefix_first_line(stripped, pid.as_deref())
            } else {
                stripped.to_string()
            };
            safety_cleaned.push(tagged);
        }
        let safety_text = format!(
            "--- Repository Bearings & Safety ---\n{}",
            safety_cleaned.join("\n\n")
        );
        // Intermediate subsection trim: no F2b sentinel (final join applies F2b).
        sections.push(trim_to_word_budget_no_sentinel(
            &safety_text,
            onboarding_budget,
        ));
    }

    let active = if global {
        take_round_robin(
            active,
            |s| project_key(s.project_id.as_deref()),
            GLOBAL_SESSION_PER_PROJECT,
            GLOBAL_SESSION_MAX,
        )
    } else {
        active
    };

    if !active.is_empty() {
        let mut session_texts = Vec::new();
        for session in &active {
            let header = if global {
                format!(
                    "--- Session: {} {} ---",
                    session.session_id,
                    project_tag(session.project_id.as_deref())
                )
            } else {
                format!("--- Session: {} ---", session.session_id)
            };
            let mut session_lines = vec![header];
            let had_turns = !session.turns.is_empty();
            for turn in &session.turns {
                let content = &turn.content;
                // Skip test markers
                if content.starts_with("MANUAL_TEST:") || content.starts_with("VERIFY:") {
                    continue;
                }
                // Skip HOTSPOT content — safety section already has the authoritative copy
                if content.contains("HOTSPOT:") {
                    continue;
                }
                // Skip CONSTRAINT/INVARIANT already shown in safety
                if (content.contains("CONSTRAINT:") || content.contains("INVARIANT:"))
                    && safety_for_skip.iter().any(|e| e.contains(content.as_str()))
                {
                    continue;
                }
                // Skip low-signal turns
                if is_low_signal(content) {
                    continue;
                }
                let truncated = truncate_turn(content);
                session_lines.push(format!("{}: {}", turn.role.to_uppercase(), truncated));
            }
            // Include session if it has unfiltered turns, or if it was empty to begin with
            if session_lines.len() > 1 || !had_turns {
                if global && let Some(id) = session.project_id.as_deref() {
                    span_ids.push(id.to_string());
                }
                session_texts.push(session_lines.join("\n"));
            }
        }
        if !session_texts.is_empty() {
            sections.push(session_texts.join("\n\n"));
        }
    }

    // --- General Memory Index (scoped to current project when project_id is known) ---
    // T274 F11: leading-marker pins first, then recency-fill other injectable rows.
    let mut collected: Vec<(String, String, Option<String>)> = Vec::new(); // content, ts, project
    let mut collected_ids: HashSet<String> = HashSet::new();
    let pass1_sql = index_select_sql(global, &index_marker_glob_sql("m.content"));
    drain_index_pass(
        &conn,
        &pass1_sql,
        global,
        project_id_str.as_deref(),
        &[],
        &safety_ids,
        &mut collected,
        &mut collected_ids,
        max_words,
        true,
    )?;
    let mut pass2_ids: Vec<String> = collected_ids.iter().cloned().collect();
    pass2_ids.sort();
    let pass2_extra = bound_not_in_sql("m.memory_id", pass2_ids.len()).unwrap_or_default();
    let pass2_sql = index_select_sql(global, &pass2_extra);
    drain_index_pass(
        &conn,
        &pass2_sql,
        global,
        project_id_str.as_deref(),
        &pass2_ids,
        &safety_ids,
        &mut collected,
        &mut collected_ids,
        max_words,
        false,
    )?;

    let index_items = if global {
        take_round_robin(
            collected.clone(),
            |(_, _, pid)| project_key(pid.as_deref()),
            GLOBAL_INDEX_PER_PROJECT,
            GLOBAL_INDEX_MAX,
        )
    } else {
        collected.clone()
    };
    let recent_items = if global {
        take_round_robin(
            collected.clone(),
            |(_, _, pid)| project_key(pid.as_deref()),
            GLOBAL_RECENT_PER_PROJECT,
            GLOBAL_RECENT_MAX,
        )
    } else {
        collected.iter().take(3).cloned().collect()
    };

    if !index_items.is_empty() {
        // 1. Build the index section with relative timestamps
        let mut index_lines = vec!["--- Memory Index (Briefing) ---".to_string()];
        for (i, (content, updated_at, pid)) in index_items.iter().enumerate() {
            let first_line = content.lines().next().unwrap_or("Untitled Memory");
            let summary = truncate_index_summary(first_line);
            let ts = relative_timestamp(updated_at);
            let line = if ts.is_empty() {
                format!("{}. {}", i + 1, summary)
            } else {
                format!("{}. {} -- {}", i + 1, summary, ts)
            };
            if global {
                if let Some(id) = pid.as_deref() {
                    span_ids.push(id.to_string());
                }
                index_lines.push(prefix_first_line(&line, pid.as_deref()));
            } else {
                index_lines.push(line);
            }
        }
        let index_text = index_lines.join("\n");

        // 2. Build the detailed section (top 3 most recent memories)
        let mut detailed_text = String::new();
        if !recent_items.is_empty() {
            let mut detailed_entries = Vec::new();
            for (content, updated_at, pid) in &recent_items {
                let ts = relative_timestamp(updated_at);
                let entry = if ts.is_empty() {
                    content.to_string()
                } else {
                    format!("({}) {}", ts, content)
                };
                if global {
                    if let Some(id) = pid.as_deref() {
                        span_ids.push(id.to_string());
                    }
                    detailed_entries.push(prefix_first_line(&entry, pid.as_deref()));
                } else {
                    detailed_entries.push(entry);
                }
            }
            detailed_text = format!(
                "--- Most Recent Memories ---\n\n{}\n\n(Use 'recall' to fetch details for other index items)",
                detailed_entries.join("\n\n")
            );
        }

        // 3. Assemble with budget awareness (content words; ignore F2b chrome).
        let remaining_budget = max_words.saturating_sub(content_word_count(&sections.join("\n\n")));
        let full_text = format!("{}\n\n{}", index_text, detailed_text);

        if word_count(&full_text) <= remaining_budget {
            sections.push(full_text);
        } else if word_count(&index_text) <= remaining_budget {
            sections.push(index_text);
        } else {
            // Intermediate index cut: no F2b; final assembly applies F2b if needed.
            sections.push(trim_to_word_budget_no_sentinel(
                &index_text,
                remaining_budget,
            ));
            sections.push("... [Index Truncated]".to_string());
        }
    }

    if collected.is_empty() && active.is_empty() && !global && project_id.is_none() {
        sections.push("--- AI-Brains: New Repository Detected ---\nThis repository has not been initialized with AI-Brains. No previous memories or safety signals are available for this context. Run 'ai-brains context' to initialize project tracking.".to_string());
    }

    let text = trim_to_word_budget(&sections.join("\n\n"), max_words);
    Ok(PreflightContext {
        // F32: content words exclude F2b trailing sentinel chrome.
        word_count: content_word_count(&text),
        text,
        in_context_project_span: if global {
            Some(span_count(span_ids))
        } else {
            None
        },
    })
}

fn index_select_sql(global: bool, extra: &str) -> String {
    let scope = if global {
        "WHERE m.status = 'pinned'"
    } else {
        "WHERE m.status = 'pinned' AND (s.project_id = ? OR m.project_id = ?)"
    };
    format!(
        "SELECT m.memory_id, m.content, m.privacy, m.updated_at, COALESCE(m.project_id, s.project_id)
         FROM memory_projection m
         LEFT JOIN session_projection s ON m.session_id = s.session_id
         {scope}{extra}
         ORDER BY m.updated_at DESC"
    )
}

#[allow(clippy::too_many_arguments)]
fn drain_index_pass(
    conn: &rusqlite::Connection,
    sql: &str,
    global: bool,
    project_id: Option<&str>,
    extra_ids: &[String],
    safety_ids: &HashSet<String>,
    collected: &mut Vec<(String, String, Option<String>)>,
    collected_ids: &mut HashSet<String>,
    max_words: usize,
    authority_only: bool,
) -> Result<()> {
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    if !global {
        match project_id {
            Some(pid) => {
                params.push(pid.to_string().into());
                params.push(pid.to_string().into());
            }
            None => {
                params.push(rusqlite::types::Value::Null);
                params.push(rusqlite::types::Value::Null);
            }
        }
    }
    for id in extra_ids {
        params.push(id.clone().into());
    }

    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(rusqlite::params_from_iter(params))?;
    while let Some(row) = rows.next()? {
        let memory_id: String = row.get(0)?;
        let privacy: String = row.get(2)?;
        if safety_ids.contains(&memory_id) || collected_ids.contains(&memory_id) {
            continue;
        }
        if !is_injectable_privacy(&privacy) {
            continue;
        }
        let content: String = row.get(1)?;
        let updated_at: String = row.get(3)?;
        let item_project: Option<String> = row.get(4)?;
        let content = strip_ansi(&content);
        if is_low_signal(&content) {
            continue;
        }
        if authority_only && classify_pin_kind(&content) == PinKind::Other {
            continue;
        }
        if global {
            if collected.len() >= GLOBAL_INDEX_FETCH {
                break;
            }
            collected_ids.insert(memory_id);
            collected.push((content, updated_at, item_project));
            continue;
        }
        let candidate = if collected.is_empty() {
            content.clone()
        } else {
            let mut parts: Vec<String> = collected.iter().map(|(c, _, _)| c.clone()).collect();
            parts.push(content.clone());
            parts.join("\n\n")
        };
        if word_count(&candidate) > max_words {
            break;
        }
        collected_ids.insert(memory_id);
        collected.push((content, updated_at, item_project));
    }
    Ok(())
}

fn query_ledgerful(_project_id: &str, scope_paths: Option<&Vec<String>>) -> Option<String> {
    // 1. Create a temp file
    let temp_file = tempfile::NamedTempFile::new().ok()?;
    let temp_path = temp_file.path().to_path_buf();

    // 2. Build the command with optional scope
    let mut cmd = std::process::Command::new("ledgerful");
    cmd.args([
        "bridge",
        "export",
        "--out",
        &temp_path.to_string_lossy(),
        "--hotspots",
    ]);

    // If scope is provided, append --scope <comma-separated paths>
    if let Some(paths) = scope_paths
        && !paths.is_empty()
    {
        cmd.arg("--scope");
        cmd.arg(paths.join(","));
    }

    let output = cmd.output().ok()?;

    if !output.status.success() {
        // Fail-open: if contextual query fails, fall through to generic query
        if scope_paths.is_some() {
            return query_ledgerful_fallback();
        }
        return None;
    }

    // 3. Read the temp file, deserialize records, construct a clean text response
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(&temp_path).ok()?;
    let reader = BufReader::new(file);

    let is_contextual = scope_paths.is_some();
    let mut hotspots = Vec::new();
    for line in reader.lines() {
        let line = line.ok()?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ai_brains_contracts::bridge::BridgeRecord>(&line)
        {
            let payload = record.payload_value();
            if record.record_kind == "hotspot_delta"
                && let Some(path) = payload.get("path").and_then(|v| v.as_str())
            {
                let score = payload.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("");

                if is_contextual {
                    // Parse contextual risk fields for scope-based queries
                    let temporal_coupling =
                        payload.get("temporal_coupling").and_then(|v| v.as_f64());
                    let failure_probability =
                        payload.get("failure_probability").and_then(|v| v.as_f64());

                    let mut entry = format!("- {} (Risk: {:.2}", path, score);
                    if let Some(tc) = temporal_coupling {
                        entry.push_str(&format!(" | Temporal Coupling: {:.2}", tc));
                    }
                    if let Some(fp) = failure_probability {
                        entry.push_str(&format!(" | Failure Prob: {:.1}%", fp * 100.0));
                    }
                    if !reason.is_empty() {
                        entry.push_str(&format!(" | Reason: {}", reason));
                    }
                    entry.push_str(") [Source: Ledgerful Contextual]");
                    hotspots.push(entry);
                } else {
                    hotspots.push(format!(
                        "- {} (Score: {:.2}, Reason: {}) [Source: Ledgerful]",
                        path, score, reason
                    ));
                }
            }
        }
    }

    if hotspots.is_empty() {
        None
    } else if is_contextual {
        Some(format!(
            "--- Ledgerful Intelligence (Contextual Risk) ---\nTop Impacted Hotspots for Current Scope:\n{}",
            hotspots.join("\n")
        ))
    } else {
        Some(format!(
            "--- Ledgerful Intelligence ---\nTop Hotspots:\n{}",
            hotspots.join("\n")
        ))
    }
}

/// Fallback: run a generic (non-scoped) hotspot query when contextual query fails.
/// Implements the fail-open requirement.
fn query_ledgerful_fallback() -> Option<String> {
    let temp_file = tempfile::NamedTempFile::new().ok()?;
    let temp_path = temp_file.path().to_path_buf();

    let output = std::process::Command::new("ledgerful")
        .args([
            "bridge",
            "export",
            "--out",
            &temp_path.to_string_lossy(),
            "--hotspots",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open(&temp_path).ok()?;
    let reader = BufReader::new(file);

    let mut hotspots = Vec::new();
    for line in reader.lines() {
        let line = line.ok()?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<ai_brains_contracts::bridge::BridgeRecord>(&line)
        {
            let payload = record.payload_value();
            if record.record_kind == "hotspot_delta"
                && let Some(path) = payload.get("path").and_then(|v| v.as_str())
            {
                let score = payload.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("");
                hotspots.push(format!(
                    "- {} (Score: {:.2}, Reason: {}) [Source: Ledgerful Fallback]",
                    path, score, reason
                ));
            }
        }
    }

    if hotspots.is_empty() {
        None
    } else {
        Some(format!(
            "--- Ledgerful Intelligence (Fallback - Contextual Unavailable) ---\nTop Hotspots:\n{}",
            hotspots.join("\n")
        ))
    }
}

/// Extract file paths from hotspot table content (lines containing `| crates/` or similar).
fn extract_hotspot_paths(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            line.contains('|') && (line.contains("crates/") || line.contains("scripts/"))
        })
        .filter_map(|line| {
            // Split and collect non-empty segments; last non-empty is the file path.
            // Handles trailing '|' in markdown table rows.
            let parts: Vec<&str> = line
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            parts.last().map(|s| s.to_string())
        })
        .filter(|p| !p.is_empty() && !p.starts_with('-') && !p.starts_with('=') && p != "File Path")
        .collect()
}

/// Deduplicate hotspot entries by keeping only the first (most recent) entry per file path.
/// Non-hotspot entries (CONSTRAINT, INVARIANT) pass through unchanged.
#[cfg(test)]
fn dedup_hotspots(entries: Vec<(String, String)>) -> Vec<String> {
    dedup_hotspots_keyed(entries.into_iter().map(|(c, t)| (c, t, ())).collect())
        .into_iter()
        .map(|(c, _)| c)
        .collect()
}

/// Like [`dedup_hotspots`] but preserves a caller-supplied key (T264 project id).
fn dedup_hotspots_keyed<T: Clone>(entries: Vec<(String, String, T)>) -> Vec<(String, T)> {
    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for (content, _updated_at, extra) in &entries {
        if content.contains("HOTSPOT:") {
            let paths = extract_hotspot_paths(content);
            if paths.is_empty() {
                // Can't parse paths — keep the entry as-is
                result.push((content.clone(), extra.clone()));
                continue;
            }
            let new_paths: Vec<String> = paths
                .into_iter()
                .filter(|p| seen_paths.insert(p.clone()))
                .collect();
            if !new_paths.is_empty() {
                // Rebuild the entry with only the new paths to avoid noise
                let new_paths_set: HashSet<String> = new_paths.into_iter().collect();
                let mut rebuilt_lines = Vec::new();
                for line in content.lines() {
                    let line_paths = extract_hotspot_paths(line);
                    if line_paths.is_empty() {
                        rebuilt_lines.push(line.to_string());
                    } else {
                        if line_paths.iter().any(|p| new_paths_set.contains(p)) {
                            rebuilt_lines.push(line.to_string());
                        }
                    }
                }
                result.push((rebuilt_lines.join("\n"), extra.clone()));
            }
            // If all paths already seen, skip this entry entirely
        } else {
            // CONSTRAINTS, INVARIANTS, etc. — always keep
            result.push((content.clone(), extra.clone()));
        }
    }

    result
}

/// Compute a human-readable relative timestamp from an RFC 3339 string.
fn relative_timestamp(rfc3339_str: &str) -> String {
    let updated = match chrono::DateTime::parse_from_rfc3339(rfc3339_str) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return String::new(),
    };
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(updated);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hr ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!(
            "{} day{} ago",
            duration.num_days(),
            if duration.num_days() == 1 { "" } else { "s" }
        )
    } else if duration.num_days() < 30 {
        format!("{} wk ago", duration.num_days() / 7)
    } else {
        format!("{} mo ago", duration.num_days() / 30)
    }
}

/// Check if content is low-signal — build monitoring, single-word replies, etc.
fn is_low_signal(content: &str) -> bool {
    let wc = word_count(content);
    // Very short (< 6 words): single-word replies like "proceed", "yes", etc.
    if wc < 6 {
        return true;
    }
    // Short (6-15 words): check for build-monitoring patterns
    if wc < 15 {
        let low_signal_patterns = [
            "Waiting for results",
            "Package name is",
            "Errors incoming",
            "Workspace package names",
            "Compile check:",
        ];
        for pat in &low_signal_patterns {
            if content.contains(pat) {
                return true;
            }
        }
    }
    false
}

/// Truncate a memory index title for display.
///
/// Uses Unicode scalar counts (not bytes) so multi-byte characters such as
/// em-dashes never land mid-codepoint and panic on slice.
fn truncate_index_summary(first_line: &str) -> String {
    const MAX_CHARS: usize = 60;
    const KEEP_CHARS: usize = 57;
    if first_line.chars().count() > MAX_CHARS {
        let truncated: String = first_line.chars().take(KEEP_CHARS).collect();
        format!("{truncated}...")
    } else {
        first_line.to_string()
    }
}

/// Truncate turn content to first 3 lines / 150 words, appending "..." if cut.
fn truncate_turn(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let wc = word_count(content);

    if lines.len() <= 3 && wc <= 150 {
        return content.to_string();
    }

    let truncated_lines: Vec<&str> = lines.into_iter().take(3).collect();
    let mut result = truncated_lines.join("\n");
    result = trim_to_word_budget(&result, 150);

    // F38 soft: avoid double truncation chrome when F2b already appended `…`.
    if content_word_count(&result) < wc && !result.ends_with('…') && !result.ends_with("...") {
        result.push_str("\n...");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_index_summary_ascii_under_limit_unchanged() {
        let s = "ASSISTANT: DECISION: short title";
        assert_eq!(truncate_index_summary(s), s);
    }

    #[test]
    fn truncate_index_summary_emdash_near_boundary_no_panic_and_ellipsis() {
        // Repro from C:\dev\dedupe preflight panic: em-dash (—) is 3 UTF-8
        // bytes; a byte slice at 57 was mid-character.
        let s = "ASSISTANT: DECISION: Starting track 0035 CalendarItems — schema v16 cal_* + message_class; extract-calendar (icalendar+chrono-tz); multi-event ICS=archive parent + per-VEVENT single-event natives; extract-pst calendar class branch; job ics_extract; no RR";
        let out = truncate_index_summary(s);
        assert!(out.ends_with("..."), "expected ellipsis, got: {out}");
        assert!(out.is_char_boundary(out.len()));
        assert_eq!(out.chars().count(), 57 + 3); // 57 kept + "..."
        assert!(!out.contains('\u{FFFD}'));
    }

    #[test]
    fn dedup_hotspots_removes_duplicate_paths() {
        let entries = vec![
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots (Risk Density)\n\
                 | Rank | Score | Freq | Comp | File Path |\n\
                 |------+------+------+------+----------------------------------------------------------------------|\n\
                 | 1 | 0.133 | 2 | 2 | crates/ai-brains-cli/tests/cli_capture_smoke.rs |\n\
                 | 2 | 0.133 | 2 | 2 | crates/ai-brains-cli/tests/ingest_reads_json_stdin.rs |"
                    .to_string(),
                "2026-01-01T00:00:00Z".to_string(),
            ),
            (
                "ASSISTANT: CONSTRAINT: Every repo must have AI_BRAINS_PROJECT_ID.".to_string(),
                "2026-01-01T00:00:00Z".to_string(),
            ),
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots (Risk Density)\n\
                 | Rank | Score | Freq | Comp | File Path |\n\
                 |------+------+------+------+----------------------------------------------------------------------|\n\
                 | 1 | 0.2 | 2 | 2 | crates/ai-brains-cli/tests/cli_capture_smoke.rs |\n\
                 | 2 | 0.2 | 2 | 2 | crates/ai-brains-cli/tests/ingest_reads_json_stdin.rs |"
                    .to_string(),
                "2026-01-01T00:00:00Z".to_string(),
            ),
        ];

        let result = dedup_hotspots(entries);
        assert_eq!(
            result.len(),
            2,
            "should keep one HOTSPOT + one CONSTRAINT, got: {:?}",
            result
        );
        assert!(result[0].contains("HOTSPOT:"), "first should be HOTSPOT");
        assert!(
            result[1].contains("CONSTRAINT:"),
            "second should be CONSTRAINT"
        );
    }

    #[test]
    fn extract_hotspot_paths_works() {
        let content = "ASSISTANT: HOTSPOT: Codebase Hotspots\n\
                       | Rank | Score | File Path |\n\
                       |------+-------+------------------------------|\n\
                       | 1 | 0.5 | crates/app/src/main.rs |\n\
                       | 2 | 0.3 | scripts/deploy.ps1 |";

        let paths = extract_hotspot_paths(content);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"crates/app/src/main.rs".to_string()));
        assert!(paths.contains(&"scripts/deploy.ps1".to_string()));
    }

    #[test]
    fn dedup_hotspots_rebuilds_entry_with_only_new_paths() {
        let entries = vec![
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots (Risk Density)\n\
                 | Rank | Score | File Path |\n\
                 |------+------+--------------------------------|\n\
                 | 1 | 0.1 | crates/path1 |\n\
                 | 2 | 0.2 | crates/path2 |"
                    .to_string(),
                "2026-01-01T00:00:00Z".to_string(),
            ),
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots (Risk Density)\n\
                 | Rank | Score | File Path |\n\
                 |------+------+--------------------------------|\n\
                 | 1 | 0.3 | crates/path1 |\n\
                 | 2 | 0.4 | crates/path3 |"
                    .to_string(),
                "2026-01-01T00:00:00Z".to_string(),
            ),
        ];

        let result = dedup_hotspots(entries);
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("crates/path1"));
        assert!(result[0].contains("crates/path2"));
        // Second entry should only contain crates/path3, not crates/path1
        assert!(
            !result[1].contains("crates/path1"),
            "should have filtered out crates/path1 from second entry"
        );
        assert!(result[1].contains("crates/path3"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn dedup_hotspots_keyed__duplicate_path__skip_set_omits_dropped_id() {
        // AC1 — skip ids come from remaining extras after keyed dedup, not a pre-insert set.
        let keep = "keep-id".to_string();
        let drop = "drop-id".to_string();
        let entries = vec![
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots\n\
                 | Rank | Score | File Path |\n\
                 |------+-------+------------------------------|\n\
                 | 1 | 0.5 | crates/ai-brains-cli/src/main.rs |"
                    .to_string(),
                "2026-08-20T12:00:00Z".to_string(),
                ((), keep.clone()),
            ),
            (
                "ASSISTANT: HOTSPOT: Codebase Hotspots\n\
                 | Rank | Score | File Path |\n\
                 |------+-------+------------------------------|\n\
                 | 1 | 0.3 | crates/ai-brains-cli/src/main.rs |"
                    .to_string(),
                "2026-08-20T11:00:00Z".to_string(),
                ((), drop.clone()),
            ),
        ];
        let remaining = dedup_hotspots_keyed(entries);
        let skip: HashSet<String> = remaining.iter().map(|(_, (_, id))| id.clone()).collect();
        assert!(
            skip.contains(&keep),
            "kept extra id must remain in skip set; remaining={remaining:?} skip={skip:?}"
        );
        assert!(
            !skip.contains(&drop),
            "dropped duplicate extra id must not enter skip set; remaining={remaining:?} skip={skip:?}"
        );
        assert_eq!(skip.len(), 1, "exactly the kept extra; skip={skip:?}");
    }
}
