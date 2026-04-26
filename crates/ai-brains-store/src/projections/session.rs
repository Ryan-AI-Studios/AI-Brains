use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct SessionProjection;

impl Projection for SessionProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let privacy = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::SessionStarted(p) => {
                tx.execute(
                    "INSERT INTO session_projection (session_id, project_id, status, privacy, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(session_id) DO UPDATE SET
                        status = excluded.status,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.session_id.to_string(),
                        p.project_id.to_string(),
                        "active",
                        privacy,
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::SessionCompleted(p) => {
                tx.execute(
                    "UPDATE session_projection SET status = ?, updated_at = ? WHERE session_id = ?",
                    rusqlite::params!["completed", occurred_at, p.session_id.to_string()],
                )?;
            }
            Payload::SessionFailed(p) => {
                tx.execute(
                    "UPDATE session_projection SET status = ?, updated_at = ? WHERE session_id = ?",
                    rusqlite::params!["failed", occurred_at, p.session_id.to_string()],
                )?;
            }
            Payload::SessionSummaryCreated(p) => {
                tx.execute(
                    "UPDATE session_projection 
                     SET summary_memory_id = ?, summarized_at = ?, updated_at = ? 
                     WHERE session_id = ?",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        occurred_at,
                        occurred_at,
                        p.session_id.to_string()
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
