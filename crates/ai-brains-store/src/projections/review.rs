use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct ReviewProjection;

impl Projection for ReviewProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ReviewItemOpened(p) => {
                let subject_kind = serde_json::to_string(&p.subject_kind)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?
                    .trim_matches('"')
                    .to_string();
                let criticality = serde_json::to_string(&p.criticality)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?
                    .trim_matches('"')
                    .to_string();
                tx.execute(
                    "INSERT INTO review_item_projection (
                        review_item_id, subject_kind, subject_id, criticality, status,
                        opened_by, subject, resolution, resolved_by,
                        related_conclusion_id, related_decision_id, related_source_id,
                        recorded_at, updated_at
                     ) VALUES (?, ?, ?, ?, 'Open', ?, ?, NULL, NULL, ?, ?, ?, ?, ?)
                     ON CONFLICT(review_item_id) DO UPDATE SET
                        subject_kind = excluded.subject_kind,
                        subject_id = excluded.subject_id,
                        criticality = excluded.criticality,
                        subject = excluded.subject,
                        related_conclusion_id = excluded.related_conclusion_id,
                        related_decision_id = excluded.related_decision_id,
                        related_source_id = excluded.related_source_id,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.review_item_id.to_string(),
                        subject_kind,
                        p.subject_id,
                        criticality,
                        p.opened_by.to_string(),
                        p.subject,
                        p.related_conclusion_id.as_ref().map(|id| id.to_string()),
                        p.related_decision_id.as_ref().map(|id| id.to_string()),
                        p.related_source_id.as_ref().map(|id| id.to_string()),
                        occurred_at,
                        occurred_at,
                    ],
                )?;
            }
            Payload::ReviewItemResolved(p) => {
                tx.execute(
                    "UPDATE review_item_projection
                     SET status = 'Resolved', resolution = ?, resolved_by = ?, updated_at = ?
                     WHERE review_item_id = ?",
                    rusqlite::params![
                        p.resolution,
                        p.resolved_by.to_string(),
                        occurred_at,
                        p.review_item_id.to_string(),
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
