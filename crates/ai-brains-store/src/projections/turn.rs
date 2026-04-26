use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct TurnProjection;

impl Projection for TurnProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        let (session_id, role, content) = match &envelope.payload {
            Payload::UserPromptRecorded(p) => (p.session_id.to_string(), "user", p.content.clone()),
            Payload::AssistantFinalRecorded(p) => {
                (p.session_id.to_string(), "assistant", p.content.clone())
            }
            _ => return Ok(()),
        };

        let turn_index: i64 = tx.query_row(
            "SELECT COALESCE(MAX(turn_index), -1) + 1 FROM turn_projection WHERE session_id = ?",
            [&session_id],
            |row| row.get(0),
        )?;

        tx.execute(
            "INSERT INTO turn_projection (session_id, turn_index, role, content, occurred_at)
             VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![session_id, turn_index, role, content, occurred_at],
        )?;

        Ok(())
    }
}
