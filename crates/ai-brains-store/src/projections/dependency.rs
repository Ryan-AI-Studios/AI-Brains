use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::{OptionalExtension, Transaction};
use time::format_description::well_known::Rfc3339;

pub struct DependencyProjection;

impl Projection for DependencyProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            // SourceVersionRecorded intentionally does **not** insert Pending queue
            // rows. Observation may run with `run_invalidation: false`, which would
            // strand Pending forever (Codex round2 P2-4). Queue audit rows are
            // written as Processed by ConclusionMarkedStale when invalidation runs.
            Payload::SourceVersionRecorded(_) => {}
            Payload::ConclusionProposed(p) => {
                for evidence_id in &p.evidence_ids {
                    let source_version_id: Option<String> = tx
                        .query_row(
                            "SELECT source_version_id FROM evidence_projection WHERE evidence_id = ?",
                            rusqlite::params![evidence_id.to_string()],
                            |row| row.get(0),
                        )
                        .optional()?;

                    tx.execute(
                        "INSERT INTO knowledge_dependency_projection (
                            parent_type, parent_id, evidence_id, source_version_id, recorded_at
                         ) VALUES ('Conclusion', ?, ?, ?, ?)",
                        rusqlite::params![
                            p.conclusion_id.to_string(),
                            evidence_id.to_string(),
                            source_version_id,
                            occurred_at,
                        ],
                    )?;
                }
            }
            Payload::DecisionProposed(p) => {
                if let Some(conclusion_ids) = &p.conclusion_ids {
                    for conclusion_id in conclusion_ids {
                        // Reverse lookup: decision depends on supporting conclusions
                        // via their evidence edges. Materialize parent edges so
                        // invalidation can walk Decision ← Conclusion ← Evidence.
                        // Also record a synthetic dependency row keyed by conclusion
                        // evidence ids already projected under the conclusion.
                        let mut stmt = tx.prepare(
                            "SELECT evidence_id, source_version_id
                             FROM knowledge_dependency_projection
                             WHERE parent_type = 'Conclusion' AND parent_id = ?",
                        )?;
                        let rows =
                            stmt.query_map(rusqlite::params![conclusion_id.to_string()], |row| {
                                Ok((
                                    row.get::<_, Option<String>>(0)?,
                                    row.get::<_, Option<String>>(1)?,
                                ))
                            })?;

                        let mut deps = Vec::new();
                        for row in rows {
                            deps.push(row.map_err(|e| StoreError::EventReadFailed(e.to_string()))?);
                        }
                        drop(stmt);

                        if deps.is_empty() {
                            // No evidence edges yet; still record a parent marker
                            // so reverse lookup of decision→conclusion exists.
                            // Use a null evidence with the conclusion id encoded
                            // is not allowed by CHECK — skip empty dependency.
                            continue;
                        }

                        for (evidence_id, source_version_id) in deps {
                            tx.execute(
                                "INSERT INTO knowledge_dependency_projection (
                                    parent_type, parent_id, evidence_id, source_version_id, recorded_at
                                 ) VALUES ('Decision', ?, ?, ?, ?)",
                                rusqlite::params![
                                    p.decision_id.to_string(),
                                    evidence_id,
                                    source_version_id,
                                    occurred_at,
                                ],
                            )?;
                        }
                    }
                }
            }
            Payload::ConclusionMarkedStale(p) => {
                // Mark any Pending queue rows for this conclusion as Processed (T149-F5).
                // If none exist (unavailable path / direct stale without prior version enqueue),
                // insert an audit row already Processed.
                if let Some(version_id) = &p.changed_source_version_id {
                    let updated = tx.execute(
                        "UPDATE invalidation_queue_projection
                         SET status = 'Processed'
                         WHERE parent_type = 'Conclusion'
                           AND parent_id = ?
                           AND source_version_id = ?
                           AND status = 'Pending'",
                        rusqlite::params![p.conclusion_id.to_string(), version_id.to_string(),],
                    )?;
                    if updated == 0 {
                        tx.execute(
                            "INSERT INTO invalidation_queue_projection (
                                parent_type, parent_id, reason, source_version_id, status, enqueued_at
                             ) VALUES ('Conclusion', ?, 'MarkedStale', ?, 'Processed', ?)",
                            rusqlite::params![
                                p.conclusion_id.to_string(),
                                version_id.to_string(),
                                occurred_at,
                            ],
                        )?;
                    }
                } else if let Some(reason) = &p.unavailable_reason {
                    tx.execute(
                        "INSERT INTO invalidation_queue_projection (
                            parent_type, parent_id, reason, source_version_id, status, enqueued_at
                         ) VALUES ('Conclusion', ?, ?, NULL, 'Processed', ?)",
                        rusqlite::params![p.conclusion_id.to_string(), reason, occurred_at,],
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
