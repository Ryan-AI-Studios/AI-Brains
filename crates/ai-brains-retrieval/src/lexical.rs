use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_core::{
    LEXICAL_MATCH_HARD_CAP, contentful_tokens, extract_fts_tokens, match_and, match_or,
    select_or_tokens,
};
use ai_brains_store::VaultConnection;
use rusqlite::params_from_iter;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalMemory {
    pub memory_id: String,
    pub content: String,
    pub score: Option<f64>,
    pub session_id: Option<String>,
    /// Memory projection `updated_at` (RFC3339), when available (T211 recency).
    pub updated_at: Option<String>,
}

/// Options for [`lexical_search`] (T217).
///
/// - `rescue`: when true, run stopword-AND / contentful-OR ladder after empty R0
///   (recall only; default **false** so forget stays strict).
/// - `limit`: SQL `LIMIT` bound; clamped to [`LEXICAL_MATCH_HARD_CAP`] (200).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexicalSearchOptions {
    pub rescue: bool,
    pub limit: usize,
    /// When true, append the T260 GLOB stub exclusion (recall path only).
    /// Default **false** so `forget --match` still finds stubs (F10).
    pub exclude_symbol_stubs: bool,
}

impl Default for LexicalSearchOptions {
    fn default() -> Self {
        Self {
            rescue: false,
            limit: LEXICAL_MATCH_HARD_CAP,
            exclude_symbol_stubs: false,
        }
    }
}

/// Bound for `ORDER BY rank LIMIT ?` on every MATCH.
pub fn match_limit_bound(caller_limit: usize) -> usize {
    caller_limit.min(LEXICAL_MATCH_HARD_CAP)
}

/// Lexical FTS5 search with optional multi-token rescue ladder (T217).
///
/// Pass the **raw** query — MATCH expressions are built internally. Do not
/// pre-sanitize: double-sanitize would break OR rescue expressions.
pub fn lexical_search(
    conn: &VaultConnection,
    raw_query: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
    opts: LexicalSearchOptions,
) -> Result<Vec<RetrievalMemory>> {
    let tokens = extract_fts_tokens(raw_query);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let limit = match_limit_bound(opts.limit);

    // R0: full AND of all extracted tokens
    let r0_expr = match_and(&tokens);
    let mut results = match_query(
        conn,
        &r0_expr,
        project_id,
        session_id,
        limit,
        opts.exclude_symbol_stubs,
    )?;
    if !results.is_empty() {
        return Ok(results);
    }

    // Rescue ladder only when opt-in, empty R0, and ≥3 tokens (D1).
    if !opts.rescue || tokens.len() < 3 {
        return Ok(results);
    }

    let contentful = contentful_tokens(&tokens);
    if contentful.is_empty() {
        return Ok(results);
    }

    // R1: AND of contentful tokens when they differ from full token sequence
    if contentful != tokens {
        let r1_expr = match_and(&contentful);
        tracing::debug!(
            stage = "R1",
            token_count = tokens.len(),
            contentful_count = contentful.len(),
            "FTS multi-token rescue: contentful AND"
        );
        results = match_query(
            conn,
            &r1_expr,
            project_id,
            session_id,
            limit,
            opts.exclude_symbol_stubs,
        )?;
        if !results.is_empty() {
            return Ok(results);
        }
    }

    // R2: OR of selected contentful tokens (cap 8) when ≥2 contentful
    if contentful.len() >= 2 {
        let or_tokens = select_or_tokens(&contentful);
        let r2_expr = match_or(&or_tokens);
        tracing::debug!(
            stage = "R2",
            or_token_count = or_tokens.len(),
            "FTS multi-token rescue: contentful OR"
        );
        results = match_query(
            conn,
            &r2_expr,
            project_id,
            session_id,
            limit,
            opts.exclude_symbol_stubs,
        )?;
    }

    Ok(results)
}

/// Execute a parameterized FTS5 MATCH with project/session scope and SQL LIMIT.
///
/// `match_expr` must already be a safe expression (quoted tokens only). Does
/// **not** re-run `sanitize_fts_query` (F9 — OR rescue must not be double-sanitized).
fn match_query(
    conn: &VaultConnection,
    match_expr: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
    limit: usize,
    exclude_symbol_stubs: bool,
) -> Result<Vec<RetrievalMemory>> {
    if match_expr.is_empty() {
        return Ok(Vec::new());
    }

    let conn = conn.lock()?;

    let mut sql =
        "SELECT mp.memory_id, mp.content, mp.privacy, mp.session_id, fts.rank, mp.updated_at
         FROM memory_fts fts
         JOIN memory_projection mp ON mp.rowid = fts.rowid
         LEFT JOIN session_projection sp ON mp.session_id = sp.session_id
         WHERE memory_fts MATCH ? AND mp.status = 'pinned'
           AND mp.privacy NOT IN ('\"Sealed\"', '\"NeverInject\"', '\"Never Inject\"', '\"Private\"')"
            .to_string();

    let mut params_vec: Vec<rusqlite::types::Value> = vec![match_expr.to_string().into()];

    if let Some(sid) = session_id {
        sql.push_str(" AND mp.session_id = ?");
        params_vec.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(" AND (sp.project_id = ? OR mp.project_id = ?)");
        let pid_str = pid.to_string();
        params_vec.push(pid_str.clone().into());
        params_vec.push(pid_str.into());
    }

    if exclude_symbol_stubs {
        sql.push_str(&crate::symbol_stub::symbol_stub_sql_exclusion("mp.content"));
    }

    sql.push_str(" ORDER BY rank LIMIT ?");
    params_vec.push((limit as i64).into());

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params_vec))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        // Defense-in-depth: SQL already excludes non-injectable privacy (so LIMIT
        // applies to injectable rows). Keep the helper for unknown/legacy labels.
        let privacy: String = row.get(2)?;
        if is_injectable_privacy(&privacy) {
            results.push(RetrievalMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
                score: row.get(4)?,
                session_id: row.get(3)?,
                updated_at: row.get(5)?,
            });
        }
    }

    Ok(results)
}

/// Substring fallback when FTS5 returns no results.
///
/// Only runs for small vaults (<= 10,000 pinned memories in the requested
/// project scope). The LIKE pattern is case-insensitive for ASCII characters
/// but case-sensitive for most Unicode characters because SQLite's default
/// `LIKE` uses ASCII case folding only.
pub fn substring_fallback(
    conn: &VaultConnection,
    query: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
    limit: usize,
    exclude_symbol_stubs: bool,
) -> Result<Vec<RetrievalMemory>> {
    let conn = conn.lock()?;

    // CPU guard: skip substring scan for large projects.
    let count: i64 = project_memory_count(&conn, project_id, session_id)?;
    if count > 10_000 {
        tracing::debug!(
            project_id = ?project_id,
            count,
            "Skipping substring fallback: project has {} memories (>10000 threshold)",
            count
        );
        return Ok(Vec::new());
    }

    if query.is_empty() {
        return Ok(Vec::new());
    }

    let pattern = format!("%{}%", escape_like_pattern(query));

    let mut sql = "SELECT memory_id, content, privacy, session_id, updated_at FROM memory_projection\n         WHERE content LIKE ? ESCAPE '\\' AND status = 'pinned'".to_string();
    let mut params_vec: Vec<rusqlite::types::Value> = vec![pattern.into()];

    if let Some(sid) = session_id {
        sql.push_str(" AND session_id = ?");
        params_vec.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(
            " AND (project_id = ? OR EXISTS (
             SELECT 1 FROM session_projection sp
             WHERE sp.session_id = memory_projection.session_id
             AND sp.project_id = ?))",
        );
        let pid_str = pid.to_string();
        params_vec.push(pid_str.clone().into());
        params_vec.push(pid_str.into());
    }

    if exclude_symbol_stubs {
        sql.push_str(&crate::symbol_stub::symbol_stub_sql_exclusion("content"));
    }

    sql.push_str(" ORDER BY updated_at DESC LIMIT ?");
    params_vec.push((limit as i64).into());

    tracing::debug!(
        project_id = ?project_id,
        query = %query,
        "FTS5 returned 0 results, falling back to substring search for '{}'",
        query
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params_vec))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(2)?;
        if is_injectable_privacy(&privacy) {
            results.push(RetrievalMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
                score: None,
                session_id: row.get(3)?,
                updated_at: row.get(4)?,
            });
        }
    }

    Ok(results)
}

fn project_memory_count(
    conn: &rusqlite::Connection,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<i64> {
    let mut sql = "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'".to_string();
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(sid) = session_id {
        sql.push_str(" AND session_id = ?");
        params_vec.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(
            " AND (project_id = ? OR EXISTS (
             SELECT 1 FROM session_projection sp
             WHERE sp.session_id = memory_projection.session_id
             AND sp.project_id = ?))",
        );
        let pid_str = pid.to_string();
        params_vec.push(pid_str.clone().into());
        params_vec.push(pid_str.into());
    }

    let count: i64 = conn.query_row(&sql, params_from_iter(params_vec), |row| row.get(0))?;
    Ok(count)
}

/// Escape `%` and `_` so they are treated as literals by SQLite LIKE.
fn escape_like_pattern(query: &str) -> String {
    query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
#[allow(non_snake_case)] // TDD names use __ separators
mod unit_tests {
    use super::*;

    #[test]
    fn match_limit_bound__clamps_to_hard_cap() {
        assert_eq!(match_limit_bound(50), 50);
        assert_eq!(match_limit_bound(200), 200);
        assert_eq!(match_limit_bound(500), LEXICAL_MATCH_HARD_CAP);
        assert_eq!(match_limit_bound(0), 0);
    }

    #[test]
    fn lexical_search_options_default__rescue_false_limit_hard_cap() {
        let opts = LexicalSearchOptions::default();
        assert!(!opts.rescue);
        assert_eq!(opts.limit, LEXICAL_MATCH_HARD_CAP);
        assert!(!opts.exclude_symbol_stubs);
    }
}
