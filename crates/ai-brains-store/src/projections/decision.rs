use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct DecisionProjection;

impl Projection for DecisionProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::DecisionProposed(p) => {
                let valid_from = match p.valid_from {
                    Some(t) => Some(
                        t.format(&Rfc3339)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => Some(occurred_at.clone()),
                };
                let valid_until = match p.valid_until {
                    Some(t) => Some(
                        t.format(&Rfc3339)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => None,
                };
                tx.execute(
                    "INSERT INTO decision_projection (
                        decision_id, state, title, statement, scope, proposer,
                        approver, proposal_event_id, valid_from, valid_until,
                        recorded_at, updated_at, superseded_by
                     ) VALUES (?, 'Proposed', ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, NULL)
                     ON CONFLICT(decision_id) DO UPDATE SET
                        title = excluded.title,
                        statement = excluded.statement,
                        scope = excluded.scope,
                        proposer = excluded.proposer,
                        proposal_event_id = excluded.proposal_event_id,
                        valid_from = excluded.valid_from,
                        valid_until = excluded.valid_until,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.decision_id.to_string(),
                        p.title,
                        p.statement,
                        p.scope,
                        p.proposer.to_string(),
                        envelope.event_id.to_string(),
                        valid_from,
                        valid_until,
                        occurred_at,
                        occurred_at,
                    ],
                )?;
                if let Some(conclusion_ids) = &p.conclusion_ids {
                    for cid in conclusion_ids {
                        tx.execute(
                            "INSERT INTO decision_support_projection (
                                decision_id, conclusion_id, evidence_id
                             ) VALUES (?, ?, '')
                             ON CONFLICT(decision_id, conclusion_id, evidence_id) DO NOTHING",
                            rusqlite::params![p.decision_id.to_string(), cid.to_string()],
                        )?;
                    }
                }
                if let Some(evidence_ids) = &p.evidence_ids {
                    for eid in evidence_ids {
                        tx.execute(
                            "INSERT INTO decision_support_projection (
                                decision_id, conclusion_id, evidence_id
                             ) VALUES (?, '', ?)
                             ON CONFLICT(decision_id, conclusion_id, evidence_id) DO NOTHING",
                            rusqlite::params![p.decision_id.to_string(), eid.to_string()],
                        )?;
                    }
                }
            }
            Payload::DecisionApproved(p) => {
                tx.execute(
                    "UPDATE decision_projection
                     SET state = 'Approved', approver = ?, proposal_event_id = COALESCE(proposal_event_id, ?),
                         updated_at = ?
                     WHERE decision_id = ?",
                    rusqlite::params![
                        p.approver.to_string(),
                        p.proposal_event_id.to_string(),
                        occurred_at,
                        p.decision_id.to_string(),
                    ],
                )?;
            }
            Payload::DecisionSuperseded(p) => {
                tx.execute(
                    "UPDATE decision_projection
                     SET state = 'Superseded', superseded_by = ?, updated_at = ?
                     WHERE decision_id = ?",
                    rusqlite::params![
                        p.superseded_by.to_string(),
                        occurred_at,
                        p.decision_id.to_string()
                    ],
                )?;
            }
            Payload::DecisionRevoked(p) => {
                tx.execute(
                    "UPDATE decision_projection
                     SET state = 'Revoked', updated_at = ?
                     WHERE decision_id = ?",
                    rusqlite::params![occurred_at, p.decision_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
