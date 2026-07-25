use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::Transaction;
use time::format_description::well_known::Rfc3339;

pub struct EvidenceProjection;

impl Projection for EvidenceProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let privacy_json = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::EvidenceRecorded(p) => {
                let model_provenance_json = match &p.model_provenance {
                    Some(mp) => Some(
                        serde_json::to_string(mp)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    ),
                    None => None,
                };
                // Ensure a source_projection row exists for well-known / orphan
                // source ids (e.g. verification-gate evidence without re-emitting
                // SourceRegistered on every capture).
                tx.execute(
                    "INSERT INTO source_projection (
                        source_id, scope, kind, display_name, locator, status,
                        recorded_at, updated_at
                     ) VALUES (?, '', 'Other', 'Referenced source', NULL, 'Active', ?, ?)
                     ON CONFLICT(source_id) DO NOTHING",
                    rusqlite::params![p.source_id.to_string(), occurred_at, occurred_at],
                )?;
                tx.execute(
                    "INSERT INTO evidence_projection (
                        evidence_id, source_id, source_version_id, status, summary,
                        privacy, model_provenance_json, fingerprint, recorded_at
                     ) VALUES (?, ?, ?, 'Active', ?, ?, ?, ?, ?)
                     ON CONFLICT(evidence_id) DO UPDATE SET
                        source_id = excluded.source_id,
                        source_version_id = excluded.source_version_id,
                        summary = excluded.summary,
                        privacy = excluded.privacy,
                        model_provenance_json = excluded.model_provenance_json,
                        fingerprint = excluded.fingerprint,
                        recorded_at = excluded.recorded_at",
                    rusqlite::params![
                        p.evidence_id.to_string(),
                        p.source_id.to_string(),
                        p.source_version_id.as_ref().map(|id| id.to_string()),
                        p.summary,
                        privacy_json,
                        model_provenance_json,
                        p.fingerprint,
                        occurred_at,
                    ],
                )?;
            }
            Payload::EvidenceSuperseded(p) => {
                tx.execute(
                    "UPDATE evidence_projection
                     SET status = 'Superseded'
                     WHERE evidence_id = ?",
                    rusqlite::params![p.evidence_id.to_string()],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
