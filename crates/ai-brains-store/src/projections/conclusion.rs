use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct ConclusionProjection;

impl Projection for ConclusionProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let privacy_json = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ConclusionProposed(p) => {
                let valid_from = p
                    .valid_from
                    .unwrap_or(envelope.occurred_at)
                    .format(&Rfc3339)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                let valid_until = match p.valid_until {
                    Some(t) => Some(
                        t.format(&Rfc3339)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => None,
                };
                let unsupported = if p.unsupported || p.evidence_ids.is_empty() {
                    1i64
                } else {
                    0i64
                };
                tx.execute(
                    "INSERT INTO conclusion_projection (
                        conclusion_id, state, statement, scope, privacy, proposer,
                        valid_from, valid_until, recorded_at, updated_at,
                        supersedes, superseded_by, protected_category, unsupported
                     ) VALUES (?, 'Candidate', ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, ?, ?)
                     ON CONFLICT(conclusion_id) DO UPDATE SET
                        statement = excluded.statement,
                        scope = excluded.scope,
                        privacy = excluded.privacy,
                        proposer = excluded.proposer,
                        valid_from = excluded.valid_from,
                        valid_until = excluded.valid_until,
                        updated_at = excluded.updated_at,
                        protected_category = excluded.protected_category,
                        unsupported = excluded.unsupported",
                    rusqlite::params![
                        p.conclusion_id.to_string(),
                        p.statement,
                        p.scope,
                        privacy_json,
                        p.proposer.to_string(),
                        valid_from,
                        valid_until,
                        occurred_at,
                        occurred_at,
                        p.protected_category,
                        unsupported,
                    ],
                )?;
                for evidence_id in &p.evidence_ids {
                    tx.execute(
                        "INSERT INTO conclusion_evidence_projection (conclusion_id, evidence_id)
                         VALUES (?, ?)
                         ON CONFLICT(conclusion_id, evidence_id) DO NOTHING",
                        rusqlite::params![p.conclusion_id.to_string(), evidence_id.to_string()],
                    )?;
                }
            }
            Payload::ConclusionActivated(p) => {
                update_state(tx, &p.conclusion_id.to_string(), "Active", &occurred_at)?;
            }
            Payload::ConclusionConfirmed(p) => {
                update_state(tx, &p.conclusion_id.to_string(), "Confirmed", &occurred_at)?;
            }
            Payload::ConclusionMarkedStale(p) => {
                update_state(tx, &p.conclusion_id.to_string(), "Stale", &occurred_at)?;
            }
            Payload::ConclusionDisputed(p) => {
                update_state(tx, &p.conclusion_id.to_string(), "Disputed", &occurred_at)?;
            }
            Payload::ConclusionRejected(p) => {
                update_state(tx, &p.conclusion_id.to_string(), "Rejected", &occurred_at)?;
            }
            Payload::ConclusionSuperseded(p) => {
                tx.execute(
                    "UPDATE conclusion_projection
                     SET state = 'Superseded', superseded_by = ?, updated_at = ?
                     WHERE conclusion_id = ?",
                    rusqlite::params![
                        p.superseded_by.to_string(),
                        occurred_at,
                        p.conclusion_id.to_string()
                    ],
                )?;
                // Link successor's supersedes if successor row exists.
                tx.execute(
                    "UPDATE conclusion_projection
                     SET supersedes = ?, updated_at = ?
                     WHERE conclusion_id = ?",
                    rusqlite::params![
                        p.conclusion_id.to_string(),
                        occurred_at,
                        p.superseded_by.to_string()
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn update_state(
    tx: &Transaction,
    conclusion_id: &str,
    state: &str,
    updated_at: &str,
) -> Result<()> {
    tx.execute(
        "UPDATE conclusion_projection SET state = ?, updated_at = ? WHERE conclusion_id = ?",
        rusqlite::params![state, updated_at, conclusion_id],
    )?;
    Ok(())
}
