use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct ConflictProjection;

impl Projection for ConflictProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        if let Payload::ConflictDetected(p) = &envelope.payload {
            tx.execute(
                "INSERT INTO conflict_projection (conflict_id, session_id, explanation, created_at)
                 VALUES (?, ?, ?, ?)",
                rusqlite::params![
                    p.conflict_id.to_string(),
                    p.session_id.to_string(),
                    p.explanation,
                    occurred_at
                ],
            )?;
        }

        Ok(())
    }
}
