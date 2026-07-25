use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct ClaimConflictProjection;

impl Projection for ClaimConflictProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ClaimConflictOpened(p) => {
                let valid_from = match p.valid_from {
                    Some(t) => Some(
                        t.format(&Rfc3339)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => None,
                };
                let valid_until = match p.valid_until {
                    Some(t) => Some(
                        t.format(&Rfc3339)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => None,
                };
                tx.execute(
                    "INSERT INTO claim_conflict_projection (
                        conflict_id, claim_a_kind, claim_a_id, claim_b_kind, claim_b_id,
                        status, scope, valid_from, valid_until, explanation, resolution,
                        recorded_at, updated_at
                     ) VALUES (?, ?, ?, ?, ?, 'Open', ?, ?, ?, ?, NULL, ?, ?)
                     ON CONFLICT(conflict_id) DO UPDATE SET
                        claim_a_kind = excluded.claim_a_kind,
                        claim_a_id = excluded.claim_a_id,
                        claim_b_kind = excluded.claim_b_kind,
                        claim_b_id = excluded.claim_b_id,
                        scope = excluded.scope,
                        valid_from = excluded.valid_from,
                        valid_until = excluded.valid_until,
                        explanation = excluded.explanation,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.conflict_id.to_string(),
                        p.claim_a_kind,
                        p.claim_a_id,
                        p.claim_b_kind,
                        p.claim_b_id,
                        p.scope,
                        valid_from,
                        valid_until,
                        p.explanation,
                        occurred_at,
                        occurred_at,
                    ],
                )?;
            }
            Payload::ClaimConflictResolved(p) => {
                tx.execute(
                    "UPDATE claim_conflict_projection
                     SET status = 'Resolved', resolution = ?, updated_at = ?
                     WHERE conflict_id = ?",
                    rusqlite::params![p.resolution, occurred_at, p.conflict_id.to_string(),],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
