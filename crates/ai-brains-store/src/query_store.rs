use crate::connection::VaultConnection;
use crate::errors::{Result, StoreError};
use crate::{MemoryListFilter, MemoryListRow, MemoryListStatus, QueryStore};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use rusqlite::{OptionalExtension, params};
use std::str::FromStr;

/// High limit for legacy `list_forgotten_memories` thin-wrap (tests / non-CLI callers).
/// Production CLI uses bounded `list_memories` via `memory list` / `forget --list-forgotten`.
const LEGACY_LIST_FORGOTTEN_CAP: usize = 1_000_000;

/// Case-insensitive exact token match on first-line `TAGS: a, b` (T216 F12 stage 2).
///
/// Strips one leading USER:/ASSISTANT:/SYSTEM: (capture pin stores
/// `ASSISTANT: TAGS: a, b\nbody` via turn projection).
fn content_has_tag_token(content: &str, tag: &str) -> bool {
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

/// Build parameterized list/count SQL fragments (T216 F15/F16 SOOT).
///
/// Returns `(from_where_sql, params)` without SELECT list or LIMIT/ORDER.
fn memory_list_from_where(
    status: MemoryListStatus,
    project_id: Option<&ProjectId>,
    tag_sql_prefix: bool,
) -> (String, Vec<String>) {
    let status_s = status.as_str().to_string();
    let mut params: Vec<String> = vec![status_s];
    let mut sql = String::from(
        "FROM memory_projection mp \
         LEFT JOIN session_projection sp ON mp.session_id = sp.session_id \
         WHERE mp.status = ?",
    );
    if let Some(pid) = project_id {
        let pid_str = pid.to_string();
        sql.push_str(" AND (sp.project_id = ? OR mp.project_id = ?)");
        params.push(pid_str.clone());
        params.push(pid_str);
    }
    if tag_sql_prefix {
        // Start-anchored TAGS: after optional role prefix from turn projection
        // (`ASSISTANT: TAGS: …`). Never mid-body `%TAGS:%` (F12 / AC10).
        sql.push_str(
            " AND (mp.content LIKE 'TAGS:%' \
              OR mp.content LIKE 'USER: TAGS:%' \
              OR mp.content LIKE 'ASSISTANT: TAGS:%' \
              OR mp.content LIKE 'SYSTEM: TAGS:%')",
        );
    }
    (sql, params)
}

impl QueryStore for VaultConnection {
    fn get_unsummarized_sessions(&self) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT session_id FROM session_projection 
             WHERE status = 'completed' AND summary_memory_id IS NULL",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_session_turns(&self, session_id: &str) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;

        let mut stmt = conn.prepare(
            "SELECT role, content FROM turn_projection
             WHERE session_id = ?
             ORDER BY occurred_at ASC",
        )?;
        let rows = stmt.query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_session_status(&self, session_id: &SessionId) -> Result<Option<String>> {
        let conn = self.lock()?;
        let mut stmt =
            conn.prepare("SELECT status FROM session_projection WHERE session_id = ?")?;
        let status: Option<String> = stmt
            .query_row([session_id.to_string()], |row| row.get(0))
            .optional()?;
        Ok(status)
    }

    fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<(MemoryId, String)>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT f.memory_id, f.content 
             FROM memory_fts f
             JOIN memory_projection p ON f.memory_id = p.memory_id
             WHERE f.content MATCH ? AND p.status != 'forgotten'
             LIMIT ?",
        )?;
        let rows = stmt.query_map([query, &limit.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id_str, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id_str, content) = row?;
            let id = MemoryId::from_str(&id_str)
                .map_err(|e| crate::StoreError::EventReadFailed(e.to_string()))?;
            results.push((id, content));
        }
        Ok(results)
    }

    fn get_memories_by_level(
        &self,
        level: u32,
        limit: Option<usize>,
    ) -> Result<Vec<(MemoryId, String)>> {
        let conn = self.lock()?;
        let sql = if let Some(n) = limit {
            format!(
                "SELECT memory_id, content FROM memory_projection 
                 WHERE level = ? AND status = 'pinned'
                 ORDER BY updated_at DESC LIMIT {}",
                n
            )
        } else {
            "SELECT memory_id, content FROM memory_projection 
             WHERE level = ? AND status = 'pinned'
             ORDER BY updated_at DESC"
                .to_string()
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([level], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id_str, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (id_str, content) = row?;
            let id = MemoryId::from_str(&id_str)
                .map_err(|e| crate::StoreError::EventReadFailed(e.to_string()))?;
            results.push((id, content));
        }
        Ok(results)
    }

    fn get_memory_privacy(&self, memory_id: &MemoryId) -> Result<Option<Privacy>> {
        let conn = self.lock()?;
        let privacy_json: Option<String> = conn
            .query_row(
                "SELECT privacy FROM memory_projection WHERE memory_id = ?",
                [memory_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        match privacy_json {
            Some(json) => {
                let privacy: Privacy = serde_json::from_str(&json)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                Ok(Some(privacy))
            }
            None => Ok(None),
        }
    }

    fn delete_old_turns(&self, cutoff: chrono::DateTime<chrono::Utc>) -> Result<usize> {
        let conn = self.lock()?;
        let count = conn.execute(
            "DELETE FROM turn_projection WHERE last_accessed_at < ?",
            [cutoff.to_rfc3339()],
        )?;
        Ok(count)
    }

    fn list_memories(&self, filter: &MemoryListFilter) -> Result<Vec<MemoryListRow>> {
        let conn = self.lock()?;
        let tag_sql = filter.tag.is_some();
        let (from_where, mut params) =
            memory_list_from_where(filter.status, filter.project_id.as_ref(), tag_sql);
        let limit = filter.limit.max(1);
        params.push(limit.to_string());
        let sql = format!(
            "SELECT mp.memory_id, mp.content, mp.updated_at, \
                    COALESCE(mp.project_id, sp.project_id) AS project_id, \
                    mp.status \
             {from_where} \
             ORDER BY mp.updated_at DESC, mp.memory_id ASC \
             LIMIT ?"
        );
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let memory_id: String = row.get(0)?;
            let content: String = row.get(1)?;
            let updated_at: String = row.get(2)?;
            let project_id: Option<String> = row.get(3)?;
            let status: String = row.get(4)?;
            Ok(MemoryListRow {
                memory_id,
                content,
                updated_at,
                project_id,
                status,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn count_memories(&self, filter: &MemoryListFilter) -> Result<u64> {
        let conn = self.lock()?;
        let tag_sql = filter.tag.is_some();
        let (from_where, params) =
            memory_list_from_where(filter.status, filter.project_id.as_ref(), tag_sql);

        // Two-stage tag total: scan TAGS:% candidates and exact-token filter (F12/F43).
        if let Some(ref tag) = filter.tag {
            let sql = format!("SELECT mp.content {from_where}");
            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
                .iter()
                .map(|p| p as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| {
                let content: String = row.get(0)?;
                Ok(content)
            })?;
            let mut count = 0u64;
            for row in rows {
                let content = row?;
                if content_has_tag_token(&content, tag) {
                    count = count.saturating_add(1);
                }
            }
            return Ok(count);
        }

        let sql = format!("SELECT COUNT(*) {from_where}");
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();
        let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))?;
        Ok(count as u64)
    }

    fn count_memories_by_project(&self) -> Result<Vec<(String, u64, u64)>> {
        let conn = self.lock()?;
        // F38: only projects with pinned>0 OR forgotten>0; exclude null project_id.
        // memory_projection.project_id only (turn-only projects excluded).
        let sql = "
            SELECT project_id,
                   SUM(CASE WHEN status = 'pinned' THEN 1 ELSE 0 END) AS pinned,
                   SUM(CASE WHEN status = 'forgotten' THEN 1 ELSE 0 END) AS forgotten
            FROM memory_projection
            WHERE status IN ('pinned', 'forgotten')
              AND project_id IS NOT NULL
            GROUP BY project_id
            HAVING pinned > 0 OR forgotten > 0
            ORDER BY (pinned + forgotten) DESC, project_id ASC
        ";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            let project_id: String = row.get(0)?;
            let pinned: i64 = row.get(1)?;
            let forgotten: i64 = row.get(2)?;
            Ok((project_id, pinned as u64, forgotten as u64))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn count_forgotten_memories(&self, project_id: Option<&ProjectId>) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = match project_id {
            None => conn.query_row(
                "SELECT COUNT(*) FROM memory_projection WHERE status = 'forgotten'",
                [],
                |row| row.get(0),
            )?,
            Some(pid) => conn.query_row(
                "SELECT COUNT(*) FROM memory_projection
                 WHERE status = 'forgotten' AND project_id = ?",
                params![pid.to_string()],
                |row| row.get(0),
            )?,
        };
        Ok(count as u64)
    }

    fn list_forgotten_memories(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<(String, String)>> {
        // Thin-wrap shared list (F37). High cap for legacy callers; CLI uses bounded list.
        let filter = MemoryListFilter {
            status: MemoryListStatus::Forgotten,
            project_id,
            tag: None,
            limit: LEGACY_LIST_FORGOTTEN_CAP,
        };
        let rows = self.list_memories(&filter)?;
        Ok(rows.into_iter().map(|r| (r.memory_id, r.content)).collect())
    }

    fn resolve_project_id_from_alias(&self, alias: &str) -> Result<Option<ProjectId>> {
        let conn = self.lock()?;
        let res: Option<String> = conn
            .query_row(
                "SELECT project_id FROM project_alias_projection WHERE alias = ?",
                [alias],
                |row| row.get(0),
            )
            .optional()?;

        match res {
            Some(s) => Ok(Some(ProjectId::from_str(&s).map_err(|e| {
                crate::errors::StoreError::EventReadFailed(e.to_string())
            })?)),
            None => Ok(None),
        }
    }

    fn get_max_turn_index(&self, session_id: &SessionId) -> Result<Option<i32>> {
        let conn = self.lock()?;
        let res: Option<i32> = conn
            .query_row(
                "SELECT MAX(turn_index) FROM turn_projection WHERE session_id = ?",
                [session_id.to_string()],
                |row| row.get::<_, Option<i32>>(0),
            )
            .optional()?
            .flatten();
        Ok(res)
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare("SELECT value FROM sync_state WHERE key = ?")?;
        let res: Option<String> = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(res)
    }

    fn get_last_nightly_run(&self) -> Result<Option<String>> {
        self.get_sync_state("last_nightly_run")
    }

    fn store_embedding(&self, memory_id: &str, embedding: &[u8]) -> Result<()> {
        let conn = self.lock()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE memory_projection SET embedding = ?, embedding_generated_at = ? WHERE memory_id = ?",
            params![embedding, now, memory_id],
        )?;
        Ok(())
    }

    fn get_stale_memories(
        &self,
        days_threshold: i32,
        limit: usize,
    ) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT memory_id, content FROM memory_projection
             WHERE embedding IS NOT NULL
               AND (
                 embedding_generated_at IS NULL
                 OR datetime(embedding_generated_at) < datetime('now', '-{} days')
               )
             ORDER BY COALESCE(embedding_generated_at, updated_at) ASC
             LIMIT {}",
            days_threshold, limit
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_memories_without_embeddings(
        &self,
        limit: usize,
        since_days: Option<i32>,
    ) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;
        let sql = if let Some(days) = since_days {
            format!(
                "SELECT memory_id, content FROM memory_projection
                 WHERE embedding IS NULL
                   AND status = 'pinned'
                   AND updated_at > datetime('now', '-{} days')
                 ORDER BY updated_at DESC
                 LIMIT {}",
                days, limit
            )
        } else {
            format!(
                "SELECT memory_id, content FROM memory_projection
                 WHERE embedding IS NULL
                   AND status = 'pinned'
                 ORDER BY updated_at DESC
                 LIMIT {}",
                limit
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((id, content))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_session_memory_ids(&self, session_id: &str) -> Result<Vec<MemoryId>> {
        let conn = self.lock()?;
        let mut stmt =
            conn.prepare("SELECT memory_id FROM memory_projection WHERE session_id = ?")?;
        let rows = stmt.query_map([session_id], |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        })?;
        let mut results = Vec::new();
        for row in rows {
            let id_str = row?;
            let id = MemoryId::from_str(&id_str)
                .map_err(|e| crate::StoreError::EventReadFailed(e.to_string()))?;
            results.push(id);
        }
        Ok(results)
    }

    fn list_projects(&self) -> Result<Vec<(String, String, String, usize)>> {
        let conn = self.lock()?;
        // F13/F41: deterministic tie-break on project_id ASC.
        let sql = "
            SELECT
                p.project_id,
                p.name,
                COALESCE(a.alias, '') as alias,
                COALESCE(mem.memory_count, 0) as memory_count
            FROM project_projection p
            LEFT JOIN (
                SELECT project_id, alias FROM project_alias_projection
            ) a ON p.project_id = a.project_id
            LEFT JOIN (
                SELECT project_id, COUNT(*) as memory_count
                FROM memory_projection
                GROUP BY project_id
            ) mem ON p.project_id = mem.project_id
            ORDER BY memory_count DESC, p.project_id ASC
        ";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            let project_id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let alias: String = row.get(2)?;
            let count: usize = row.get(3)?;
            Ok((project_id, name, alias, count))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn list_projects_detail(&self) -> Result<Vec<crate::ProjectListDetail>> {
        let conn = self.lock()?;
        // F6: path via scalar correlated subquery (ORDER BY path ASC LIMIT 1) —
        // never a plain multi-row JOIN that would duplicate projects.
        // F7/M4: last_activity = COALESCE(MAX(mp.updated_at), p.updated_at).
        // F13: ORDER BY memory_count DESC, project_id ASC.
        let sql = "
            SELECT
                p.project_id,
                p.name,
                COALESCE(a.alias, '') AS alias,
                COALESCE(mem.memory_count, 0) AS memory_count,
                COALESCE(mem.last_activity, p.updated_at) AS last_activity,
                (
                    SELECT normalized_path
                    FROM repository_path_alias_projection r
                    WHERE r.project_id = p.project_id
                    ORDER BY r.normalized_path ASC
                    LIMIT 1
                ) AS path
            FROM project_projection p
            LEFT JOIN project_alias_projection a ON p.project_id = a.project_id
            LEFT JOIN (
                SELECT project_id, COUNT(*) AS memory_count, MAX(updated_at) AS last_activity
                FROM memory_projection
                GROUP BY project_id
            ) mem ON p.project_id = mem.project_id
            ORDER BY memory_count DESC, p.project_id ASC
        ";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            let project_id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let alias: String = row.get(2)?;
            let memory_count: usize = row.get(3)?;
            let last_activity: Option<String> = row.get(4)?;
            let path: Option<String> = row.get(5)?;
            Ok(crate::ProjectListDetail {
                project_id,
                name,
                alias,
                memory_count,
                last_activity: last_activity.unwrap_or_default(),
                path: path.filter(|p| !p.is_empty()),
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_project_by_id(&self, project_id: &ProjectId) -> Result<Option<(String, String)>> {
        let conn = self.lock()?;
        // Single-id SELECT (F32 / T207): same JOIN pattern as list_projects, no full scan.
        let sql = "
            SELECT
                p.name,
                COALESCE(a.alias, '') as alias
            FROM project_projection p
            LEFT JOIN (
                SELECT project_id, alias FROM project_alias_projection
            ) a ON p.project_id = a.project_id
            WHERE p.project_id = ?
            LIMIT 1
        ";
        let row = conn
            .query_row(sql, [project_id.to_string()], |row| {
                let name: String = row.get(0)?;
                let alias: String = row.get(1)?;
                Ok((name, alias))
            })
            .optional()?;
        Ok(row)
    }

    fn count_projects_with_pinned(&self) -> Result<u64> {
        let conn = self.lock()?;
        // F7 / T214: static SQL — no string interpolation of identifiers.
        let count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT project_id) FROM memory_projection
             WHERE status = 'pinned' AND project_id IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    fn count_pinned_memories(&self, project_id: Option<&ProjectId>) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = match project_id {
            None => conn.query_row(
                "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
                [],
                |row| row.get(0),
            )?,
            Some(pid) => conn.query_row(
                "SELECT COUNT(*) FROM memory_projection
                 WHERE status = 'pinned' AND project_id = ?",
                params![pid.to_string()],
                |row| row.get(0),
            )?,
        };
        Ok(count as u64)
    }

    fn count_active_sessions(&self, project_id: Option<&ProjectId>) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = match project_id {
            None => conn.query_row(
                "SELECT COUNT(*) FROM session_projection WHERE status = 'active'",
                [],
                |row| row.get(0),
            )?,
            Some(pid) => conn.query_row(
                "SELECT COUNT(*) FROM session_projection
                 WHERE status = 'active' AND project_id = ?",
                params![pid.to_string()],
                |row| row.get(0),
            )?,
        };
        Ok(count as u64)
    }

    fn memory_exists(&self, memory_id: &str) -> Result<bool> {
        let conn = self.lock()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_projection WHERE memory_id = ?",
            [memory_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
