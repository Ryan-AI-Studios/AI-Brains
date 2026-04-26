use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;
use rusqlite::params;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalMemory {
    pub memory_id: String,
    pub content: String,
}

pub fn lexical_search(conn: &VaultConnection, query: &str) -> Result<Vec<RetrievalMemory>> {
    let conn = conn.lock()?;
    let mut stmt = conn.prepare(
        "SELECT mp.memory_id, mp.content, mp.privacy
         FROM memory_fts fts
         JOIN memory_projection mp ON mp.rowid = fts.rowid
         WHERE memory_fts MATCH ? AND mp.status = 'pinned'
         ORDER BY rank",
    )?;

    let mut rows = stmt.query(params![query])?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(2)?;
        if is_injectable_privacy(&privacy) {
            results.push(RetrievalMemory {
                memory_id: row.get(0)?,
                content: row.get(1)?,
            });
        }
    }

    Ok(results)
}
