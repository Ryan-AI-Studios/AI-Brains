use crate::context::AppContext;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{RecallOptions, recall_full};
use ai_brains_store::{EventStore, QueryStore};
use is_terminal::IsTerminal;
use rusqlite::OptionalExtension;
use std::str::FromStr;

pub struct RecallRunOptions {
    pub query: String,
    pub limit: usize,
    pub project_id: Option<ProjectId>,
    pub session_id: Option<SessionId>,
    pub session_last: bool,
    pub session_prefix: Option<String>,
    pub format: Option<String>,
    pub semantic: bool,
    pub graph_boost: f64,
    pub graph_hop_depth: usize,
    pub quiet: bool,
    pub no_bridge: bool,
    pub global: bool,
    /// Soft F32: one-shot cosine floor override for `--semantic`.
    pub min_score: Option<f64>,
}

fn resolve_format(explicit: Option<&str>, is_tty: bool) -> &str {
    match explicit {
        Some(f) => f,
        None => {
            if is_tty {
                "pretty"
            } else {
                "json"
            }
        }
    }
}

/// F11 / AC8: print threshold honesty only when lexical results are present.
///
/// Conditions: semantic requested, embedding status `ok`, zero semantic hits
/// above the cosine floor, and at least one hit from FTS / substring / hybrid
/// (so "showing lexical" is honest). Bridge-only or non-lexical sources alone
/// must not trigger the line.
pub fn should_show_semantic_threshold_honesty(
    semantic_requested: bool,
    embedding_status: Option<&str>,
    semantic_post_threshold_count: Option<usize>,
    hits: &[ai_brains_retrieval::RecallHit],
) -> bool {
    if !semantic_requested {
        return false;
    }
    if embedding_status != Some("ok") {
        return false;
    }
    if semantic_post_threshold_count != Some(0) {
        return false;
    }
    hits.iter()
        .any(|h| matches!(h.source.as_str(), "fts" | "substring" | "hybrid"))
}

fn session_prefix_pattern(prefix: &str) -> String {
    let escaped = prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("{}%", escaped)
}

fn query_sessions_by_prefix(
    conn: &ai_brains_store::VaultConnection,
    prefix: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let conn_guard = conn.lock()?;
    let pattern = session_prefix_pattern(prefix);
    let mut stmt = conn_guard.prepare(
        "SELECT DISTINCT session_id FROM memory_projection WHERE session_id LIKE ? ESCAPE '\\' ORDER BY session_id",
    )?;
    let rows = stmt.query_map([&pattern], |row| row.get::<_, String>(0))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

fn query_most_recent_session(
    conn: &ai_brains_store::VaultConnection,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let conn_guard = conn.lock()?;
    let session_id: Option<String> = conn_guard
        .query_row(
            "SELECT session_id FROM memory_projection WHERE session_id IS NOT NULL ORDER BY updated_at DESC LIMIT 1",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();
    Ok(session_id)
}

fn resolve_session(
    conn: &ai_brains_store::VaultConnection,
    explicit: Option<SessionId>,
    last: bool,
    session_prefix: Option<&str>,
) -> Result<Option<SessionId>, Box<dyn std::error::Error>> {
    if last {
        let sid = query_most_recent_session(conn)?.ok_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No sessions found in vault.",
            )) as Box<dyn std::error::Error>
        })?;
        return Ok(Some(SessionId::from_str(&sid)?));
    }

    if let Some(raw) = session_prefix {
        if raw.len() == 36
            && let Ok(sid) = SessionId::from_str(raw)
        {
            return Ok(Some(sid));
        }

        if raw.len() < 4 {
            return Err(
                "Session prefix too short; provide at least 4 characters to avoid accidental matches."
                    .into(),
            );
        }

        let matches = query_sessions_by_prefix(conn, raw)?;
        match matches.len() {
            0 => Err(format!(
                "No session matching '{}'. Use 'ai-brains project list' to see sessions.",
                raw
            )
            .into()),
            1 => Ok(Some(SessionId::from_str(&matches[0])?)),
            n => {
                let shown: Vec<String> = matches.iter().take(5).cloned().collect();
                let suffix = format!(" ({} of {} shown)", shown.len(), n);
                let list = shown.join(", ");
                Err(format!(
                    "Ambiguous session prefix '{}'. Matching sessions{}: {}. Provide more characters.",
                    raw, suffix, list
                )
                .into())
            }
        }
    } else {
        Ok(explicit)
    }
}

pub fn run(
    ctx: &AppContext,
    mut options: RecallRunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolved_session_id = resolve_session(
        &ctx.conn,
        options.session_id,
        options.session_last,
        options.session_prefix.as_deref(),
    )?;
    options.session_id = resolved_session_id;
    // F5 / T207: track whether session was generated solely for graph provenance.
    let session_was_generated = options.session_id.is_none();
    let effective_session_id = options.session_id.or_else(|| {
        let generated = SessionId::new();
        tracing::debug!(
            "No session id supplied for recall; using generated session {} for graph provenance.",
            generated
        );
        Some(generated)
    });

    // Attempt to open graph vault next to the main vault
    #[cfg(feature = "graph")]
    let graph_vault = ai_brains_graph::GraphVault::new((*ctx.conn).clone());

    #[cfg(feature = "graph")]
    let graph_search = Some(ai_brains_graph::queries::GraphSearch::new(&graph_vault));

    #[cfg(not(feature = "graph"))]
    let graph_search: Option<ai_brains_retrieval::MockGraphSearch> = None;

    let outcome = recall_full(
        &ctx.conn,
        graph_search.as_ref(),
        &options.query,
        options.limit,
        RecallOptions {
            project_id: options.project_id,
            session_id: options.session_id,
            semantic: options.semantic,
            graph_boost: options.graph_boost,
            graph_hop_depth: options.graph_hop_depth,
            quiet: options.quiet,
            no_bridge: options.no_bridge,
            min_semantic_score: options.min_score,
        },
    )?;
    let hits = outcome.hits;
    let embedding = outcome.embedding;
    let semantic_post_threshold_count = outcome.semantic_post_threshold_count;

    // Emit MemoryPinned events for each recall hit so the graph projector can
    // build session -> memory RECALLS edges.
    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    for (rank, hit) in hits.iter().enumerate() {
        if let Ok(memory_id) = MemoryId::from_str(&hit.memory_id) {
            let ev = EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
                Actor::System,
                ai_brains_core::privacy::Privacy::LocalOnly,
            )
            .build(Payload::MemoryPinned(MemoryPinnedPayload {
                memory_id,
                // T224 F7: raw vault content — display-only strip lives in formatters, not events.
                content: hit.content.clone(),
                session_id: effective_session_id,
                project_id: options.project_id,
                tx_id: None,
                rank: Some(rank as u32),
                source_tag: Some(hit.source.clone()),
                query_text: Some(options.query.clone()),
            }));
            if let Ok(ev) = ev
                && let Err(e) = event_store.append_event(&ev)
            {
                tracing::warn!(
                    "Failed to emit MemoryPinned event for {}: {}",
                    hit.memory_id,
                    e
                );
            }
        }
    }

    // Own the status string before moving `embedding` into the response.
    let embedding_status_owned = embedding.as_ref().map(|e| e.status.clone());

    let response = RecallResponse {
        results: hits
            .iter()
            .map(|h| RecallResult {
                memory_id: h.memory_id.clone(),
                // T224 F6: machine JSON keeps raw role prefixes; pretty path strips display-only.
                content: h.content.clone(),
                source: h.source.clone(),
                score: h.score,
                session_id: h.session_id.clone(),
                staleness: if h.is_plan_demoted {
                    Some("plan".to_string())
                } else {
                    None
                },
                // T218 F5: narrow wire set bm25|rrf|bridge only (never cosine/hybrid).
                score_kind: Some(score_kind_wire(h.score_kind).to_string()),
                cosine: h.cosine,
            })
            .collect(),
        session_id: effective_session_id.map(|s| s.to_string()),
        hint: None,
        // F2: include embedding only when --semantic (status may be ok/unreachable/…).
        embedding,
    };
    let embedding_status = embedding_status_owned.as_deref();

    let format_str = resolve_format(options.format.as_deref(), std::io::stdout().is_terminal());

    match format_str {
        "pretty" => {
            // T228 F2/F29: always-on Scope chrome for pretty (empty + non-empty).
            let scope_line = resolve_active_scope_line(
                ctx.conn.as_ref(),
                options.global,
                options.project_id.as_ref(),
            )?;
            if hits.is_empty() {
                // Empty pretty (T207): Scope → (Session if user) → Embedding ≠ ok → hint.
                // F3: always print hint when format is pretty (no TTY gate).
                // F5: omit Session when it was generated only for graph provenance.
                // F11: --quiet does not suppress Scope or empty hint.
                let session_for_print = if session_was_generated {
                    None
                } else {
                    response.session_id.as_deref()
                };
                let embedding_line = if options.semantic {
                    response
                        .embedding
                        .as_ref()
                        .filter(|e| e.status != "ok")
                        .map(format_embedding_status_line)
                } else {
                    None
                };
                // T231 F12/F37: empty recall pretty includes ledger next-step (sync empty does not).
                let hint = build_recall_hint(
                    &ctx.conn,
                    &options.query,
                    options.semantic,
                    options.global,
                    options.project_id,
                    embedding_status,
                    true,
                )?
                .unwrap_or_default();
                println!(
                    "{}",
                    format_pretty_empty_state(
                        &scope_line,
                        session_for_print,
                        embedding_line.as_deref(),
                        &hint,
                    )
                );
            } else {
                // Non-empty pretty (T228): Scope → Session → Embedding? → hits.
                // F4/F26/F33: no blank between Scope and Session; no empty hint.
                // Badge via shared helper (T211 F11/F37).
                println!("{}", scope_line);
                if let Some(ref sid) = response.session_id {
                    println!("Session: {}", sid);
                }
                // F6: one embedding status line when --semantic and status != ok.
                // T215 F11/AC8: ok + zero post-threshold + lexical present → honesty line.
                if options.semantic
                    && let Some(ref emb) = response.embedding
                {
                    if emb.status != "ok" {
                        print_embedding_status_line(emb);
                    } else if should_show_semantic_threshold_honesty(
                        options.semantic,
                        Some(emb.status.as_str()),
                        semantic_post_threshold_count,
                        &hits,
                    ) {
                        println!(
                            "Embedding: ok (no semantic hits above threshold; showing lexical)"
                        );
                    }
                }
                print_pretty_hits(&hits);
            }
        }
        _ => {
            let mut response = response;
            if response.results.is_empty() {
                // JSON agent path: no sync-query next-step (pretty-only discovery chrome).
                response.hint = build_recall_hint(
                    &ctx.conn,
                    &options.query,
                    options.semantic,
                    options.global,
                    options.project_id,
                    embedding_status,
                    false,
                )?;
            }
            println!("{}", serde_json::to_string(&response)?);
        }
    }

    Ok(())
}

/// Map internal [`ScoreKind`] to the contracts wire `score_kind` (T218 F5).
///
/// Closed set: `bm25` | `rrf` | `bridge` only — never `cosine` or `hybrid`.
pub fn score_kind_wire(kind: ai_brains_retrieval::ScoreKind) -> &'static str {
    use ai_brains_retrieval::ScoreKind;
    match kind {
        ScoreKind::Bm25LowerBetter => "bm25",
        ScoreKind::HigherIsBetter => "rrf",
        ScoreKind::BridgeHigherIsBetter => "bridge",
    }
}

/// Format one non-empty pretty hit line (shared by `recall` + `sync query`, T211 F37).
///
/// Demoted Plan Decisions show `[plan/stale?]` immediately before content (F11).
///
/// **T218 F6:** branch on [`ScoreKind`]:
/// - `HigherIsBetter` (RRF): primary `rank=#n` + optional `sim=0.XX` (not raw ~0.016)
/// - `Bm25LowerBetter`: keep `score={:.3}` (negative rank polarity)
/// - `BridgeHigherIsBetter`: keep readable raw `score={:.3}`
#[allow(clippy::too_many_arguments)] // Shared pretty SOOT; mirrors hit fields + 1-based rank.
pub fn format_pretty_hit_line(
    memory_id: &str,
    content: &str,
    score: Option<f64>,
    session_id: Option<&str>,
    is_plan_demoted: bool,
    score_kind: ai_brains_retrieval::ScoreKind,
    cosine: Option<f64>,
    rank: usize,
) -> String {
    // T224 F3: strip role prefix BEFORE 500-char truncate (display-only).
    let content = crate::commands::display_text::strip_role_prefix(content.trim_start());
    let content = if content.chars().count() > 500 {
        format!("{}...", content.chars().take(500).collect::<String>())
    } else {
        content.to_string()
    };
    let badge = if is_plan_demoted {
        "[plan/stale?] "
    } else {
        ""
    };

    let score_part = pretty_score_bracket(score_kind, score, cosine, rank);

    match (score_part.as_deref(), session_id) {
        (Some(sp), Some(sid)) => {
            let prefix = &sid[..sid.len().min(8)];
            format!("[{sp} | session={prefix}] {memory_id}: {badge}{content}")
        }
        (Some(sp), None) => format!("[{sp}] {memory_id}: {badge}{content}"),
        (None, Some(sid)) => {
            let prefix = &sid[..sid.len().min(8)];
            format!("[session={prefix}] {memory_id}: {badge}{content}")
        }
        (None, None) => format!("{memory_id}: {badge}{content}"),
    }
}

/// Pretty score/rank fragment inside the hit bracket (T218 F6 / AC4).
fn pretty_score_bracket(
    score_kind: ai_brains_retrieval::ScoreKind,
    score: Option<f64>,
    cosine: Option<f64>,
    rank: usize,
) -> Option<String> {
    use ai_brains_retrieval::ScoreKind;
    match score_kind {
        ScoreKind::HigherIsBetter => {
            // Never primary-print raw RRF ~0.016; show 1-based rank + optional sim.
            let mut parts = vec![format!("rank=#{rank}")];
            if let Some(c) = cosine {
                parts.push(format!("sim={c:.2}"));
            }
            Some(parts.join(" | "))
        }
        ScoreKind::Bm25LowerBetter | ScoreKind::BridgeHigherIsBetter => {
            score.map(|s| format!("score={s:.3}"))
        }
    }
}

/// Print non-empty pretty hits with plan/stale badges (T211).
pub fn print_pretty_hits(hits: &[ai_brains_retrieval::RecallHit]) {
    for (i, h) in hits.iter().enumerate() {
        println!(
            "{}",
            format_pretty_hit_line(
                &h.memory_id,
                &h.content,
                h.score,
                h.session_id.as_deref(),
                h.is_plan_demoted,
                h.score_kind,
                h.cosine,
                i + 1,
            )
        );
    }
}

/// Print empty pretty recall body for `sync query` (T207 + T211 F37 + T231 F37).
///
/// Scope + next-action hint; no TTY gate. No session line (sync has no user session).
/// Does **not** suggest `sync query` (no self-mention on the sync empty path).
pub fn print_pretty_empty_sync(
    ctx: &AppContext,
    query: &str,
    global: bool,
    project_id: Option<ProjectId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let scope_line = resolve_active_scope_line(ctx.conn.as_ref(), global, project_id.as_ref())?;
    // T231 F37/AC8b: include_sync_query_hint = false (no circular self-mention).
    let hint = build_recall_hint(&ctx.conn, query, false, global, project_id, None, false)?
        .unwrap_or_default();
    println!(
        "{}",
        format_pretty_empty_state(&scope_line, None, None, &hint)
    );
    Ok(())
}

/// Pretty TTY one-liner for non-ok embedding status (F6). Does not restate full cause.
fn format_embedding_status_line(emb: &ai_brains_contracts::recall::EmbeddingStatusDto) -> String {
    match emb.endpoint.as_deref() {
        Some(ep) => format!("Embedding: {} ({})", emb.status, ep),
        None => format!("Embedding: {}", emb.status),
    }
}

fn print_embedding_status_line(emb: &ai_brains_contracts::recall::EmbeddingStatusDto) {
    println!("{}", format_embedding_status_line(emb));
}

/// Resolve the active Scope chrome line for pretty recall / sync (T228 F29/F30).
///
/// - `global == true` → `"Scope: global"` **without** calling `get_project_by_id` (F30).
/// - else if `Some(pid)` → single-id lookup then [`format_scope_line`].
/// - else → `Scope: project=(none)`.
///
/// Shared SOOT for empty + non-empty recall pretty and empty + non-empty sync vault.
pub(crate) fn resolve_active_scope_line(
    conn: &impl QueryStore,
    global: bool,
    project_id: Option<&ProjectId>,
) -> Result<String, Box<dyn std::error::Error>> {
    if global {
        return Ok("Scope: global".to_string());
    }
    let name_alias = match project_id {
        Some(pid) => conn.get_project_by_id(pid)?,
        None => None,
    };
    Ok(format_scope_line(false, project_id, name_alias.as_ref()))
}

/// Format empty pretty Scope line (F4 / T207).
///
/// - `--global` → `Scope: global`
/// - with project + known alias/name → `Scope: project=<alias-or-name> (<full-uuid>)`
/// - with project, lookup miss → `Scope: project=<full-uuid>`
/// - no project → `Scope: project=(none)`
///
/// Shared SOOT for recall empty-pretty and preflight summary (T207 / T214 F13).
/// Prefer [`resolve_active_scope_line`] at call sites that own a connection.
pub(crate) fn format_scope_line(
    global: bool,
    project_id: Option<&ProjectId>,
    name_alias: Option<&(String, String)>,
) -> String {
    if global {
        return "Scope: global".to_string();
    }
    match project_id {
        None => "Scope: project=(none)".to_string(),
        Some(pid) => {
            let uuid = pid.to_string();
            match name_alias {
                Some((name, alias)) => {
                    let label = if !alias.is_empty() {
                        alias.as_str()
                    } else if !name.is_empty() {
                        name.as_str()
                    } else {
                        ""
                    };
                    if label.is_empty() {
                        format!("Scope: project={}", uuid)
                    } else {
                        format!("Scope: project={} ({})", label, uuid)
                    }
                }
                None => format!("Scope: project={}", uuid),
            }
        }
    }
}

/// Compose empty pretty body (F31 / T207).
///
/// Print order: Scope → optional Session → optional Embedding status → empty hint.
fn format_pretty_empty_state(
    scope_line: &str,
    session_id: Option<&str>,
    embedding_status_line: Option<&str>,
    hint: &str,
) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(4);
    lines.push(scope_line.to_string());
    if let Some(sid) = session_id {
        lines.push(format!("Session: {}", sid));
    }
    if let Some(emb) = embedding_status_line
        && !emb.is_empty()
    {
        lines.push(emb.to_string());
    }
    if !hint.is_empty() {
        lines.push(hint.to_string());
    }
    lines.join("\n")
}

/// Build a contextual hint when recall returns zero results (T111 / T202 F6 / T207 F6 / T231).
///
/// `include_sync_query_hint` (T231 F12/F37): when true, append ledger next-step after the
/// core hint. Recall empty pretty passes true; `print_pretty_empty_sync` passes false
/// so sync empty does not self-mention `sync query`.
fn build_recall_hint(
    conn: &ai_brains_store::VaultConnection,
    query: &str,
    semantic: bool,
    global: bool,
    project_id: Option<ProjectId>,
    embedding_status: Option<&str>,
    include_sync_query_hint: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let project_scoped = !global && project_id.is_some();
    let mut hint = build_recall_hint_core(
        query,
        semantic,
        global,
        embedding_status,
        project_scoped,
        include_sync_query_hint,
    );

    if !global {
        let count = project_memory_count(conn, project_id)?;
        if count < 10 {
            hint.push_str(&format!(
                "\nThis project has only {} memories — results may be limited. Consider importing more sessions.",
                count
            ));
        }
    }

    Ok(Some(hint))
}

/// Core empty-result hint. When embedding status is present and not `ok`, omit the
/// redundant “check embedding model” clause (F6 / F33 / AC15) — next-action only.
///
/// `project_scoped` (F33 / T207): when true (and not global), insert a short
/// “Scoped to this project.” clause without repeating alias/id (F4 owns the name).
///
/// T217: when non-semantic multi-token query has contentful keywords, append
/// “try fewer keywords” via core token helpers (AC7 / AC7b).
///
/// T231 F12/F13/F37/F40: when `include_sync_query_hint`, append ledger next-step
/// after the core lexical/semantic guidance (additive; not a replace).
fn build_recall_hint_core(
    query: &str,
    semantic: bool,
    global: bool,
    embedding_status: Option<&str>,
    project_scoped: bool,
    include_sync_query_hint: bool,
) -> String {
    let scope_clause = if !global && project_scoped {
        " Scoped to this project."
    } else {
        ""
    };

    let mut hint = if global {
        let mut h = format!(
            "No results for '{}' across all projects. The vault may be empty or the query may not match any memories.",
            query
        );
        if !semantic && ai_brains_core::should_suggest_fewer_keywords(query) {
            h.push_str(" try fewer keywords.");
        }
        h
    } else if semantic {
        // Status field (or pretty line) already explains embed cause when != ok.
        let status_explains_cause = embedding_status.is_some_and(|s| s != "ok");
        if status_explains_cause || embedding_status.is_some() {
            // F33: when embedding present, drop redundant model-check clause.
            format!(
                "No results for '{}' (semantic search).{} Try --global to search across all projects, refine the query, or import more memories.",
                query, scope_clause
            )
        } else {
            format!(
                "No results for '{}' (semantic search).{} Try --global to search across all projects, or check if the embedding model is running.",
                query, scope_clause
            )
        }
    } else {
        // T217 AC7/AC7b: fewer-keywords only via core helpers (contentful ≥ 1, tokens ≥ 3).
        let mut h = format!(
            "No results for '{}'.{} Try --semantic for embedding-based search, or --global to search across all projects.",
            query, scope_clause
        );
        if ai_brains_core::should_suggest_fewer_keywords(query) {
            h.push_str(" try fewer keywords.");
        }
        h
    };

    // T231 F13/F40: additive ledger next-step (gated by F37).
    if include_sync_query_hint {
        let q_display = sync_query_hint_query(query);
        hint.push_str(&format!(
            "\nFor vault + Ledgerful ledger in one view: ai-brains sync query \"{}\" --format pretty",
            q_display
        ));
    }

    hint
}

/// Choose a safe query fragment for the F13 next-step line (T231).
///
/// Short single-line queries are echoed; long or multi-line queries become `"…"`.
fn sync_query_hint_query(query: &str) -> &str {
    const MAX_LEN: usize = 80;
    if query.is_empty() || query.contains('\n') || query.contains('\r') || query.len() > MAX_LEN {
        "…"
    } else {
        query
    }
}

fn project_memory_count(
    conn: &ai_brains_store::VaultConnection,
    project_id: Option<ProjectId>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let conn = conn.lock()?;
    let mut sql = "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'".to_string();
    let mut params: Vec<String> = Vec::new();

    if let Some(pid) = project_id {
        sql.push_str(
            " AND (project_id = ? OR EXISTS (\n             SELECT 1 FROM session_projection sp\n             WHERE sp.session_id = memory_projection.session_id\n             AND sp.project_id = ?))",
        );
        let pid_str = pid.to_string();
        params.push(pid_str.clone());
        params.push(pid_str);
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
        .iter()
        .map(|p| p as &dyn rusqlite::types::ToSql)
        .collect();
    let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
    Ok(count as usize)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // test-only expect/unwrap OK
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn resolve_format__explicit_json__returns_json() {
        assert_eq!(resolve_format(Some("json"), true), "json");
        assert_eq!(resolve_format(Some("json"), false), "json");
    }

    #[test]
    #[allow(non_snake_case)]
    fn resolve_format__explicit_pretty__returns_pretty() {
        assert_eq!(resolve_format(Some("pretty"), true), "pretty");
        assert_eq!(resolve_format(Some("pretty"), false), "pretty");
    }

    #[test]
    #[allow(non_snake_case)]
    fn resolve_format__no_explicit_on_tty__returns_pretty() {
        assert_eq!(resolve_format(None, true), "pretty");
    }

    #[test]
    #[allow(non_snake_case)]
    fn resolve_format__no_explicit_not_tty__returns_json() {
        assert_eq!(resolve_format(None, false), "json");
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__no_semantic_no_global__suggests_semantic_and_global() {
        let hint = build_recall_hint_core("query", false, false, None, false, false);
        assert!(
            hint.contains("Try --semantic"),
            "hint should suggest --semantic; got: {}",
            hint
        );
        assert!(
            hint.contains("--global"),
            "hint should suggest --global; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__semantic_no_status__suggests_embedding_model() {
        let hint = build_recall_hint_core("query", true, false, None, false, false);
        assert!(
            hint.contains("semantic search"),
            "hint should mention semantic search; got: {}",
            hint
        );
        assert!(
            hint.contains("embedding model"),
            "hint should suggest checking embedding model when no status; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__semantic_unreachable_status__next_action_only_no_model_clause() {
        // AC15 / F6: when status already explains cause, hint is next-action only.
        let hint = build_recall_hint_core("query", true, false, Some("unreachable"), false, false);
        assert!(
            hint.contains("--global") || hint.contains("refine") || hint.contains("import"),
            "hint should offer next actions; got: {}",
            hint
        );
        assert!(
            !hint.contains("embedding model"),
            "hint must not restate embedding cause when status present; got: {}",
            hint
        );
        assert!(
            !hint.contains("unreachable"),
            "hint must not repeat status string; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__semantic_ok_status__drops_model_check_clause() {
        // F33: embedding present → soft-shorten even when status is ok.
        let hint = build_recall_hint_core("query", true, false, Some("ok"), false, false);
        assert!(!hint.contains("embedding model"), "got: {}", hint);
        assert!(hint.contains("--global"), "got: {}", hint);
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__global_used__notes_all_projects_empty() {
        let hint = build_recall_hint_core("query", false, true, None, false, false);
        assert!(
            hint.contains("across all projects"),
            "hint should note global scope; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn recall_hint__no_results_pretty__hint_core_contains_no_results() {
        let hint = build_recall_hint_core("zzzz", false, false, None, false, false);
        assert!(
            hint.contains("No results for 'zzzz'"),
            "hint should mention 'zzzz'; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__multi_token_contentful_empty__suggests_fewer_keywords() {
        // AC7: ≥3 tokens + contentful ≥1 → "try fewer keywords" + semantic/global.
        let hint = build_recall_hint_core(
            "what did we decide about forget list",
            false,
            false,
            None,
            false,
            false,
        );
        assert!(
            hint.contains("try fewer keywords"),
            "hint should suggest fewer keywords; got: {}",
            hint
        );
        assert!(
            hint.contains("--semantic") && hint.contains("--global"),
            "hint should keep semantic/global guidance; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__all_stopword_multi_token__no_fewer_keywords() {
        // AC7b: all-stopword multi-token must not suggest fewer keywords.
        let hint = build_recall_hint_core(
            "what did we do about this",
            false,
            false,
            None,
            false,
            false,
        );
        assert!(
            !hint.contains("try fewer keywords"),
            "all-stopword hint must not suggest fewer keywords; got: {}",
            hint
        );
        assert!(
            hint.contains("--semantic") || hint.contains("--global"),
            "hint should still offer next actions; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__project_scoped__includes_this_project_clause() {
        // B4 / F6 / F33: project_scoped adds "this project" without requiring alias.
        let hint = build_recall_hint_core("zzzz", false, false, None, true, false);
        assert!(
            hint.contains("this project") || hint.contains("Scoped to this project"),
            "project-scoped hint must mention scoped clause; got: {}",
            hint
        );
        assert!(
            hint.contains("No results"),
            "hint must still say No results; got: {}",
            hint
        );
        assert!(
            hint.contains("--semantic") || hint.contains("--global"),
            "hint must keep next-action; got: {}",
            hint
        );
        // Must not require/embed a specific alias string.
        assert!(
            !hint.contains("test-alias"),
            "core must not embed alias (F4 owns name); got: {}",
            hint
        );
    }

    // --- T231 F12/F13/F37 AC8 ---

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__include_sync_query_hint_true__appends_ledger_next_step() {
        // AC8: flag true includes F13 substring.
        let hint = build_recall_hint_core("probe-term", false, false, None, false, true);
        assert!(
            hint.contains("sync query"),
            "AC8: flag true must include sync query next-step; got: {}",
            hint
        );
        assert!(
            hint.contains("For vault + Ledgerful ledger in one view:"),
            "AC8: F13 lead-in must be present; got: {}",
            hint
        );
        assert!(
            hint.contains("probe-term"),
            "AC8: reasonable-length query echoed in next-step; got: {}",
            hint
        );
        // Core guidance still present (F40 additive).
        assert!(
            hint.contains("--semantic") || hint.contains("--global"),
            "AC8: core hint must remain; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__include_sync_query_hint_false__no_sync_query_self_mention() {
        // AC8b unit: flag false does not include F13.
        let hint = build_recall_hint_core("probe-term", false, false, None, false, false);
        assert!(
            !hint.contains("sync query"),
            "flag false must not mention sync query; got: {}",
            hint
        );
        assert!(
            !hint.contains("Ledgerful ledger in one view"),
            "flag false must not include F13 lead-in; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__global__prints_global() {
        assert_eq!(format_scope_line(true, None, None), "Scope: global");
    }

    /// T228 AC13 / F30: global short-circuits without requiring a project row.
    #[test]
    #[allow(non_snake_case)]
    fn resolve_active_scope_line__global__returns_scope_global_without_project_row() {
        use ai_brains_core::temp_env::TempEnv;
        use ai_brains_crypto::SqlCipherKey;
        use ai_brains_store::VaultConnection;
        use std::sync::Arc;

        const ZERO_KEY: &str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'";
        let dir = tempfile::tempdir().expect("tempdir");
        let vault = dir.path().join("vault.db");
        let _allow = TempEnv::set(ai_brains_store::ALLOW_ZERO_KEY_ENV, "1");
        let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
        let conn = VaultConnection::open(&vault, &key).expect("open vault");
        conn.migrate().expect("migrate");
        let conn = Arc::new(conn);

        // No project row exists. Even with a bogus project_id, global must win
        // without depending on lookup success (F30 short-circuit).
        let pid = ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("pid");
        let line = resolve_active_scope_line(conn.as_ref(), true, Some(&pid))
            .expect("resolve global scope");
        assert_eq!(line, "Scope: global");

        // Project path still works on empty vault (lookup miss → uuid only).
        let project_line = resolve_active_scope_line(conn.as_ref(), false, Some(&pid))
            .expect("resolve project scope");
        assert_eq!(
            project_line,
            "Scope: project=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
        );

        let none_line =
            resolve_active_scope_line(conn.as_ref(), false, None).expect("resolve none scope");
        assert_eq!(none_line, "Scope: project=(none)");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__no_project__prints_none() {
        assert_eq!(
            format_scope_line(false, None, None),
            "Scope: project=(none)"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__project_with_alias__prefers_alias() {
        let pid = ProjectId::from_str("441837f6-5c55-d075-0000-000000000000").unwrap();
        let na = ("Real Name".to_string(), "test-alias".to_string());
        let line = format_scope_line(false, Some(&pid), Some(&na));
        assert_eq!(
            line,
            "Scope: project=test-alias (441837f6-5c55-d075-0000-000000000000)"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__project_empty_alias__uses_name() {
        let pid = ProjectId::from_str("441837f6-5c55-d075-0000-000000000000").unwrap();
        let na = ("Display Name".to_string(), String::new());
        let line = format_scope_line(false, Some(&pid), Some(&na));
        assert_eq!(
            line,
            "Scope: project=Display Name (441837f6-5c55-d075-0000-000000000000)"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_scope_line__project_lookup_miss__uuid_only() {
        let pid = ProjectId::from_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let line = format_scope_line(false, Some(&pid), None);
        assert_eq!(line, "Scope: project=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_empty_state__scope_session_hint_order() {
        let text = format_pretty_empty_state(
            "Scope: global",
            Some("11111111-1111-1111-1111-111111111111"),
            None,
            "No results for 'zzzz' across all projects.",
        );
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "Scope: global");
        assert_eq!(lines[1], "Session: 11111111-1111-1111-1111-111111111111");
        assert!(lines[2].contains("No results"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_empty_state__omits_session_when_none() {
        let text = format_pretty_empty_state(
            "Scope: project=(none)",
            None,
            None,
            "No results for 'zzzz'.",
        );
        assert!(text.starts_with("Scope: project=(none)"));
        assert!(
            !text.contains("Session:"),
            "generated session omitted; got: {text}"
        );
        assert!(text.contains("No results"));
    }

    // F3 empty pretty TTY independence is locked by hermetic
    // `recall_empty__pretty_non_tty__stdout_contains_no_results` (not a unit stub).

    // -----------------------------------------------------------------------
    // F11 / AC8 — semantic threshold honesty gate
    // -----------------------------------------------------------------------

    fn hit_with_source(id: &str, source: &str) -> ai_brains_retrieval::RecallHit {
        ai_brains_retrieval::RecallHit {
            memory_id: id.to_string(),
            content: "c".into(),
            source: source.to_string(),
            score: Some(0.01),
            privacy: None,
            session_id: None,
            updated_at: None,
            is_plan_demoted: false,
            score_kind: ai_brains_retrieval::ScoreKind::HigherIsBetter,
            cosine: None,
        }
    }

    // -----------------------------------------------------------------------
    // T218 F6 / AC4 — pretty ScoreKind branch
    // -----------------------------------------------------------------------

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__higher_is_better__rank_not_raw_rrf__ac4() {
        let line = format_pretty_hit_line(
            "mem-1",
            "content here",
            Some(0.016393), // raw RRF 1/61 — must NOT be primary
            Some("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
            false,
            ai_brains_retrieval::ScoreKind::HigherIsBetter,
            Some(0.72),
            1,
        );
        assert!(line.contains("rank=#1"), "must show rank; got {line}");
        assert!(
            line.contains("sim=0.72"),
            "must show sim when cosine known; got {line}"
        );
        assert!(
            !line.contains("score=0.016"),
            "must not primary-print raw RRF; got {line}"
        );
        assert!(line.contains("session=aaaaaaaa"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__bm25__keeps_score__ac4() {
        let line = format_pretty_hit_line(
            "mem-2",
            "lexical",
            Some(-13.7),
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert!(
            line.contains("score=-13.700"),
            "BM25 must keep score= polarity; got {line}"
        );
        assert!(
            !line.contains("rank="),
            "BM25 must not use rank=; got {line}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__bridge__keeps_score__ac4() {
        let line = format_pretty_hit_line(
            "mem-3",
            "bridge hit",
            Some(0.85),
            None,
            false,
            ai_brains_retrieval::ScoreKind::BridgeHigherIsBetter,
            None,
            2,
        );
        assert!(
            line.contains("score=0.850"),
            "bridge keeps raw score; got {line}"
        );
        assert!(
            !line.contains("rank="),
            "bridge must not use rank=; got {line}"
        );
    }

    // -----------------------------------------------------------------------
    // T224 — role-prefix strip on pretty hit body (display-only; before truncate)
    // -----------------------------------------------------------------------

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__role_prefix_stripped__ac1() {
        let line = format_pretty_hit_line(
            "mem-id",
            "ASSISTANT: DECISION: use SQLCipher",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert!(
            line.contains("DECISION: use SQLCipher"),
            "body must remain after strip; got {line}"
        );
        // Content after memory id must not still lead with ASSISTANT:
        let after_id = line
            .split_once("mem-id: ")
            .map(|(_, rest)| rest)
            .unwrap_or(&line);
        assert!(
            !after_id.starts_with("ASSISTANT:"),
            "pretty body must not lead with ASSISTANT:; got {line}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__trim_start_before_strip__ac2b() {
        let line = format_pretty_hit_line(
            "mem-ws",
            "  ASSISTANT: body",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert!(
            line.contains("mem-ws: body"),
            "trim_start before strip must yield body; got {line}"
        );
        assert!(
            !line.contains("ASSISTANT:"),
            "leading whitespace must not leak role token; got {line}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__user_system_strip_mid_lower_leave__ac2() {
        let user = format_pretty_hit_line(
            "m-u",
            "USER: hello world",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(user, "m-u: hello world");

        let system = format_pretty_hit_line(
            "m-s",
            "SYSTEM: note",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(system, "m-s: note");

        let mid = format_pretty_hit_line(
            "m-mid",
            "text ASSISTANT: x",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(mid, "m-mid: text ASSISTANT: x");

        let lower = format_pretty_hit_line(
            "m-lo",
            "assistant: leave me",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(lower, "m-lo: assistant: leave me");
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__strip_before_truncate__ac1b() {
        // Prefix + long body: strip must run before 500-char cut so ASSISTANT: is gone
        // and body starts the display text (not role chrome left by post-truncate strip).
        let body: String = "X".repeat(600);
        let content = format!("ASSISTANT: {body}");
        let line = format_pretty_hit_line(
            "mem-long",
            &content,
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        let after_id = line
            .strip_prefix("mem-long: ")
            .expect("pretty line must start with memory id");
        assert!(
            after_id.starts_with('X'),
            "strip-before-truncate must start with body, not ASSISTANT:; got {}",
            &after_id[..after_id.len().min(40)]
        );
        assert!(
            !after_id.starts_with("ASSISTANT:"),
            "must not lead with role token after truncate; got {}",
            &after_id[..after_id.len().min(40)]
        );
        // Truncated with ASCII ... (existing formatter marker).
        assert!(
            after_id.ends_with("..."),
            "long body must truncate with ...; got len={}",
            after_id.chars().count()
        );
        // 500 body chars + "..."
        assert_eq!(
            after_id.chars().count(),
            503,
            "500-char take + ... marker; got {}",
            after_id.chars().count()
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn format_pretty_hit_line__multibyte_crlf__ac9() {
        let multibyte = format_pretty_hit_line(
            "m-mb",
            "ASSISTANT: 日本語テスト 🚀 end",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(multibyte, "m-mb: 日本語テスト 🚀 end");

        let crlf = format_pretty_hit_line(
            "m-cr",
            "\r\nASSISTANT: after-crlf",
            None,
            None,
            false,
            ai_brains_retrieval::ScoreKind::Bm25LowerBetter,
            None,
            1,
        );
        assert_eq!(crlf, "m-cr: after-crlf");
    }

    #[test]
    #[allow(non_snake_case)]
    fn score_kind_wire__narrow_set_only__ac5() {
        use ai_brains_retrieval::ScoreKind;
        assert_eq!(score_kind_wire(ScoreKind::Bm25LowerBetter), "bm25");
        assert_eq!(score_kind_wire(ScoreKind::HigherIsBetter), "rrf");
        assert_eq!(score_kind_wire(ScoreKind::BridgeHigherIsBetter), "bridge");
        // Closed set — never cosine/hybrid as wire kinds.
        let allowed = ["bm25", "rrf", "bridge"];
        for k in [
            ScoreKind::Bm25LowerBetter,
            ScoreKind::HigherIsBetter,
            ScoreKind::BridgeHigherIsBetter,
        ] {
            assert!(allowed.contains(&score_kind_wire(k)));
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__fts_hits_zero_sem__true__ac8() {
        let hits = vec![hit_with_source("a", "fts")];
        assert!(should_show_semantic_threshold_honesty(
            true,
            Some("ok"),
            Some(0),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__hybrid_hits_zero_sem__true__ac8() {
        let hits = vec![hit_with_source("a", "hybrid")];
        assert!(should_show_semantic_threshold_honesty(
            true,
            Some("ok"),
            Some(0),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__bridge_only__false__ac8() {
        // Bridge-only must not claim "showing lexical".
        let hits = vec![hit_with_source("b", "bridge")];
        assert!(!should_show_semantic_threshold_honesty(
            true,
            Some("ok"),
            Some(0),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__sem_post_threshold_nonzero__false() {
        let hits = vec![hit_with_source("a", "fts")];
        assert!(!should_show_semantic_threshold_honesty(
            true,
            Some("ok"),
            Some(2),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__status_not_ok__false() {
        let hits = vec![hit_with_source("a", "fts")];
        assert!(!should_show_semantic_threshold_honesty(
            true,
            Some("unreachable"),
            Some(0),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__semantic_not_requested__false() {
        let hits = vec![hit_with_source("a", "fts")];
        assert!(!should_show_semantic_threshold_honesty(
            false,
            Some("ok"),
            Some(0),
            &hits,
        ));
    }

    #[test]
    #[allow(non_snake_case)]
    fn should_show_semantic_threshold_honesty__empty_hits__false() {
        assert!(!should_show_semantic_threshold_honesty(
            true,
            Some("ok"),
            Some(0),
            &[],
        ));
    }
}
