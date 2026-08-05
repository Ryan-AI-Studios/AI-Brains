use crate::QueryStore;
use crate::connection::VaultConnection;
use crate::errors::{Result, StoreError};
use ai_brains_core::ids::{MemoryId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use rusqlite::{OptionalExtension, params};
use std::str::FromStr;

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

    fn list_forgotten_memories(
        &self,
        project_id: Option<ProjectId>,
    ) -> Result<Vec<(String, String)>> {
        let conn = self.lock()?;
        let (sql, params): (String, Vec<String>) = if let Some(pid) = project_id {
            let pid_str = pid.to_string();
            (
                "SELECT mp.memory_id, mp.content FROM memory_projection mp \
                 LEFT JOIN session_projection sp ON mp.session_id = sp.session_id \
                 WHERE mp.status = 'forgotten' AND (sp.project_id = ? OR mp.project_id = ?) \
                 ORDER BY mp.updated_at DESC"
                    .into(),
                vec![pid_str.clone(), pid_str],
            )
        } else {
            (
                "SELECT memory_id, content FROM memory_projection \
                 WHERE status = 'forgotten' ORDER BY updated_at DESC"
                    .into(),
                vec![],
            )
        };

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
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
