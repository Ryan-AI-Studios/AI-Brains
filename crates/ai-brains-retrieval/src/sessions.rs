use crate::errors::Result;
use crate::privacy_filter::is_injectable_privacy;
use ai_brains_store::VaultConnection;

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

            let mut turn_stmt = conn.prepare(
                "SELECT role, content
                 FROM turn_projection
                 WHERE session_id = ?
                 ORDER BY turn_index DESC
                 LIMIT 5",
            )?;
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
