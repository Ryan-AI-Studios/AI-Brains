use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;
use rusqlite::params_from_iter;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalMemory {
    pub memory_id: String,
    pub content: String,
    pub score: Option<f64>,
}

pub fn lexical_search(
    conn: &VaultConnection,
    query: &str,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<RetrievalMemory>> {
    let conn = conn.lock()?;

    let sanitized = sanitize_for_fts5(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    let mut sql = "SELECT mp.memory_id, mp.content, mp.privacy, fts.rank
         FROM memory_fts fts
         JOIN memory_projection mp ON mp.rowid = fts.rowid
         LEFT JOIN session_projection sp ON mp.session_id = sp.session_id
         WHERE memory_fts MATCH ? AND mp.status = 'pinned'"
        .to_string();

    let mut params_vec: Vec<rusqlite::types::Value> = vec![sanitized.into()];

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

    sql.push_str(" ORDER BY rank");

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params_vec))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(2)?;
        if is_injectable_privacy(&privacy) {
            results.push(RetrievalMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
                score: row.get(3)?,
            });
        }
    }

    Ok(results)
}

/// Defensive sanitization for SQLite FTS5 MATCH expressions.
///
/// Downstream consumers (e.g. ChangeGuard `bridge query`) forward raw
/// natural-language questions that may contain characters which break FTS5
/// syntax (`?`, `"`, `*`, `(`, `)`) or bare operator keywords (`AND`, `OR`,
/// `NOT`, `NEAR`). This function removes those hazards while preserving
/// alphanumeric tokens so that lexical recall never panics or returns a
/// database syntax error.
fn sanitize_for_fts5(query: &str) -> String {
    // Replace punctuation known to confuse the FTS5 query parser.
    let normalized: String =
        query.replace(['?', '"', '*', '(', ')', '.', '-', ':'], " ");

    // Filter out bare FTS5 operator keywords so they are not interpreted as
    // boolean operators.
    let mut result = String::new();
    for token in normalized.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if lower == "and" || lower == "or" || lower == "not" || lower == "near" {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(token);
    }

    result
}
