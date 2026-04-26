use crate::errors::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory_id: String,
    pub project_id: Option<String>,
    pub content_markdown: String,
}

pub struct FtsSearch<'a> {
    conn: &'a Connection,
}

impl<'a> FtsSearch<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn search(&self, query: &str, project_id: Option<Uuid>) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        // T06 exposes memory content search before memory-to-project linkage is projected.
        // Preserve the API shape by returning a null project_id until that schema lands.
        let _ = project_id;
        let mut stmt = self.conn.prepare(
            "SELECT memory_id, NULL AS project_id, content AS content_markdown
             FROM memory_fts
             WHERE memory_fts MATCH ?
             ORDER BY rank",
        )?;

        let mut rows = stmt.query(params![query])?;

        while let Some(row) = rows.next()? {
            let memory_id: String = row.get(0)?;
            let project_id: Option<String> = row.get(1)?;
            let content_markdown: String = row.get(2)?;

            results.push(SearchResult {
                memory_id,
                project_id,
                content_markdown,
            });
        }

        Ok(results)
    }
}

pub fn search_memory(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let fts = FtsSearch::new(conn);
    fts.search(query, None)
}
