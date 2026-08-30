use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;

/// Loaded `turn_projection` window per active session (T330 F8).
/// Session Other preview cap stays 3; `K` counts skipped Other in this window.
pub(crate) const SESSION_TURN_FETCH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    pub session_id: String,
    pub turns: Vec<SessionTurn>,
    /// Owning project when `session_projection.project_id` is set (T264 F10).
    pub project_id: Option<String>,
}

pub fn active_sessions(
    conn: &VaultConnection,
    project_id: Option<ai_brains_core::ids::ProjectId>,
) -> Result<Vec<SessionContext>> {
    let conn = conn.lock()?;

    if let Some(pid) = project_id {
        let pid = pid.to_string();
        let mut stmt = conn.prepare(
            "SELECT session_id, privacy, project_id
             FROM session_projection
             WHERE status = 'active'
             AND project_id = ?
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query(rusqlite::params![pid])?;
        collect_sessions(&conn, rows)
    } else {
        let mut stmt = conn.prepare(
            "SELECT session_id, privacy, project_id
             FROM session_projection
             WHERE status = 'active'
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query([])?;
        collect_sessions(&conn, rows)
    }
}

fn collect_sessions(
    conn: &rusqlite::Connection,
    mut rows: rusqlite::Rows<'_>,
) -> Result<Vec<SessionContext>> {
    let mut active = Vec::new();

    while let Some(row) = rows.next()? {
        let privacy: String = row.get(1)?;
        if is_injectable_privacy(&privacy) {
            let session_id: String = row.get(0)?;
            let project_id: Option<String> = row.get(2)?;

            let turn_sql = format!(
                "SELECT role, content
                 FROM turn_projection
                 WHERE session_id = ?
                 ORDER BY turn_index DESC
                 LIMIT {SESSION_TURN_FETCH}"
            );
            let mut turn_stmt = conn.prepare(&turn_sql)?;
            let mut turn_rows = turn_stmt.query([&session_id])?;
            let mut turns = Vec::new();
            while let Some(turn_row) = turn_rows.next()? {
                turns.push(SessionTurn {
                    role: turn_row.get(0)?,
                    content: turn_row.get(1)?,
                });
            }
            turns.reverse();

            active.push(SessionContext {
                session_id,
                turns,
                project_id,
            });
        }
    }

    Ok(active)
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(non_snake_case)]
    fn session_turn_fetch__is_twenty_and_sql_limit_uses_const() {
        assert_eq!(super::SESSION_TURN_FETCH, 20, "AC13: SESSION_TURN_FETCH");
        let src = include_str!("sessions.rs");
        assert!(
            src.contains("LIMIT {SESSION_TURN_FETCH}"),
            "AC13: SQL LIMIT must interpolate SESSION_TURN_FETCH"
        );
    }
}
