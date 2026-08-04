use crate::context::AppContext;
use ai_brains_contracts::recall::{RecallResponse, RecallResult};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_retrieval::{RecallOptions, recall_full};
use ai_brains_store::EventStore;
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
        },
    )?;
    let hits = outcome.hits;
    let embedding = outcome.embedding;

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
            .into_iter()
            .map(|h| RecallResult {
                memory_id: h.memory_id,
                content: h.content,
                source: h.source,
                score: h.score,
                session_id: h.session_id,
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
            if let Some(ref sid) = response.session_id {
                println!("Session: {}", sid);
            }
            // F6: one embedding status line when --semantic and status != ok.
            if options.semantic
                && let Some(ref emb) = response.embedding
                && emb.status != "ok"
            {
                print_embedding_status_line(emb);
            }
            for r in &response.results {
                let content = if r.content.chars().count() > 500 {
                    format!("{}...", r.content.chars().take(500).collect::<String>())
                } else {
                    r.content.clone()
                };
                match &r.session_id {
                    Some(sid) => {
                        let prefix = &sid[..sid.len().min(8)];
                        if let Some(s) = r.score {
                            println!(
                                "[score={:.3} | session={}] {}: {}",
                                s, prefix, r.memory_id, content
                            );
                        } else {
                            println!("[session={}] {}: {}", prefix, r.memory_id, content);
                        }
                    }
                    None => {
                        if let Some(s) = r.score {
                            println!("[score={:.3}] {}: {}", s, r.memory_id, content);
                        } else {
                            println!("{}: {}", r.memory_id, content);
                        }
                    }
                }
            }
            if response.results.is_empty()
                && let Some(ref hint) = build_recall_hint(
                    &ctx.conn,
                    &options.query,
                    options.semantic,
                    options.global,
                    options.project_id,
                    embedding_status,
                )?
                && std::io::stdout().is_terminal()
            {
                println!("{}", hint);
            }
        }
        _ => {
            let mut response = response;
            if response.results.is_empty() {
                response.hint = build_recall_hint(
                    &ctx.conn,
                    &options.query,
                    options.semantic,
                    options.global,
                    options.project_id,
                    embedding_status,
                )?;
            }
            println!("{}", serde_json::to_string(&response)?);
        }
    }

    Ok(())
}

/// Pretty TTY one-liner for non-ok embedding status (F6). Does not restate full cause.
fn print_embedding_status_line(emb: &ai_brains_contracts::recall::EmbeddingStatusDto) {
    match emb.endpoint.as_deref() {
        Some(ep) => println!("Embedding: {} ({})", emb.status, ep),
        None => println!("Embedding: {}", emb.status),
    }
}

/// Build a contextual hint when recall returns zero results (T111 / T202 F6).
fn build_recall_hint(
    conn: &ai_brains_store::VaultConnection,
    query: &str,
    semantic: bool,
    global: bool,
    project_id: Option<ProjectId>,
    embedding_status: Option<&str>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut hint = build_recall_hint_core(query, semantic, global, embedding_status);

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
fn build_recall_hint_core(
    query: &str,
    semantic: bool,
    global: bool,
    embedding_status: Option<&str>,
) -> String {
    if global {
        format!(
            "No results for '{}' across all projects. The vault may be empty or the query may not match any memories.",
            query
        )
    } else if semantic {
        // Status field (or pretty line) already explains embed cause when != ok.
        let status_explains_cause = embedding_status.is_some_and(|s| s != "ok");
        if status_explains_cause || embedding_status.is_some() {
            // F33: when embedding present, drop redundant model-check clause.
            format!(
                "No results for '{}' (semantic search). Try --global to search across all projects, refine the query, or import more memories.",
                query
            )
        } else {
            format!(
                "No results for '{}' (semantic search). Try --global to search across all projects, or check if the embedding model is running.",
                query
            )
        }
    } else {
        format!(
            "No results for '{}'. Try --semantic for embedding-based search, or --global to search across all projects.",
            query
        )
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
        let hint = build_recall_hint_core("query", false, false, None);
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
        let hint = build_recall_hint_core("query", true, false, None);
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
        let hint = build_recall_hint_core("query", true, false, Some("unreachable"));
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
        let hint = build_recall_hint_core("query", true, false, Some("ok"));
        assert!(!hint.contains("embedding model"), "got: {}", hint);
        assert!(hint.contains("--global"), "got: {}", hint);
    }

    #[test]
    #[allow(non_snake_case)]
    fn build_recall_hint__global_used__notes_all_projects_empty() {
        let hint = build_recall_hint_core("query", false, true, None);
        assert!(
            hint.contains("across all projects"),
            "hint should note global scope; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn recall_hint__no_results_pretty__hint_core_contains_no_results() {
        let hint = build_recall_hint_core("zzzz", false, false, None);
        assert!(
            hint.contains("No results for 'zzzz'"),
            "hint should mention 'zzzz'; got: {}",
            hint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn recall_hint__tty_guard__stdout_is_terminal_check_compiles() {
        let check: bool = std::io::stdout().is_terminal();
        assert!(
            std::any::type_name_of_val(&check).contains("bool"),
            "is_terminal must return a bool"
        );
    }
}
