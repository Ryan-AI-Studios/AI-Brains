use crate::errors::Result;
use crate::errors::StoreError;
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::{OptionalExtension, Transaction};
use time::format_description::well_known::Rfc3339;

pub struct MemoryProjection;

impl Projection for MemoryProjection {
    fn apply(&self, tx: &Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
        let privacy_json = serde_json::to_string(&envelope.privacy)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::MemoryPinned(p) => {
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, tx_id, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        session_id = COALESCE(excluded.session_id, memory_projection.session_id),
                        project_id = COALESCE(excluded.project_id, memory_projection.project_id),
                        status = excluded.status,
                        tx_id = COALESCE(excluded.tx_id, memory_projection.tx_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.session_id.as_ref().map(|s| s.to_string()),
                        p.project_id.as_ref().map(|s| s.to_string()),
                        p.content,
                        privacy_json,
                        "pinned",
                        0, // Level 0 for pinned memories
                        p.tx_id.as_ref().map(|t| t.to_string()),
                        occurred_at,
                        occurred_at
                    ],
                )?;

                // If pinned to a session, ensure session privacy is escalated
                if let Some(session_id) = &p.session_id {
                    self.escalate_session_privacy(
                        tx,
                        &session_id.to_string(),
                        envelope.privacy,
                        &occurred_at,
                    )?;
                }
            }
            Payload::SessionSummaryCreated(p) => {
                let project_id: Option<String> = if let Some(pid) = &p.project_id {
                    Some(pid.to_string())
                } else {
                    tx.query_row(
                        "SELECT project_id FROM session_projection WHERE session_id = ?",
                        rusqlite::params![p.session_id.to_string()],
                        |row| row.get(0),
                    )
                    .optional()?
                };

                tx.execute(
                    "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        session_id = COALESCE(excluded.session_id, memory_projection.session_id),
                        project_id = COALESCE(excluded.project_id, memory_projection.project_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.session_id.to_string(),
                        project_id,
                        p.summary,
                        privacy_json,
                        "pinned",
                        0, // Level 0 for session summaries
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            Payload::MemorySynthesized(p) => {
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, project_id, content, privacy, status, level, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        level = excluded.level,
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.memory_id.to_string(),
                        p.project_id.to_string(),
                        p.content,
                        privacy_json,
                        "pinned",
                        p.level,
                        occurred_at,
                        occurred_at
                    ],
                )?;

                // Track hierarchy
                for source_id in &p.source_memory_ids {
                    tx.execute(
                        "INSERT INTO memory_hierarchy (parent_memory_id, child_memory_id) VALUES (?, ?)
                         ON CONFLICT DO NOTHING",
                        rusqlite::params![p.memory_id.to_string(), source_id.to_string()],
                    )?;
                }
            }
            Payload::MemoryForgotten(p) => {
                tx.execute(
                    "UPDATE memory_projection SET status = ?, updated_at = ? WHERE memory_id = ?",
                    rusqlite::params!["forgotten", occurred_at, p.memory_id.to_string()],
                )?;
            }
            Payload::DecisionRecorded(p) => {
                let content = MemoryProjection::format_madr_markdown(
                    &p.title,
                    &p.context,
                    &p.decision,
                    &p.consequences,
                );
                tx.execute(
                    "INSERT INTO memory_projection (memory_id, session_id, project_id, content, privacy, status, level, tx_id, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(memory_id) DO UPDATE SET
                        content = excluded.content,
                        session_id = COALESCE(excluded.session_id, memory_projection.session_id),
                        project_id = COALESCE(excluded.project_id, memory_projection.project_id),
                        updated_at = excluded.updated_at",
                    rusqlite::params![
                        p.decision_id.to_string(),
                        p.session_id.as_ref().map(|s| s.to_string()),
                        p.project_id.as_ref().map(|s| s.to_string()),
                        content,
                        privacy_json,
                        "pinned",
                        0,
                        p.tx_id.as_ref().map(|t| t.to_string()),
                        occurred_at,
                        occurred_at
                    ],
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl MemoryProjection {
    fn format_madr_markdown(
        title: &str,
        context: &str,
        decision: &str,
        consequences: &str,
    ) -> String {
        format!(
            "# {}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}",
            title, context, decision, consequences
        )
    }

    fn escalate_session_privacy(
        &self,
        tx: &Transaction,
        session_id: &str,
        new_privacy: ai_brains_core::privacy::Privacy,
        occurred_at: &str,
    ) -> Result<()> {
        use rusqlite::OptionalExtension;

        let current_privacy_json: Option<String> = tx
            .query_row(
                "SELECT privacy FROM session_projection WHERE session_id = ?",
                rusqlite::params![session_id],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(json) = current_privacy_json {
            let current_privacy: ai_brains_core::privacy::Privacy = serde_json::from_str(&json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            let combined = current_privacy.combine(new_privacy);
            if combined != current_privacy {
                let combined_json = serde_json::to_string(&combined)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
                tx.execute(
                    "UPDATE session_projection SET privacy = ?, updated_at = ? WHERE session_id = ?",
                    rusqlite::params![combined_json, occurred_at, session_id],
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryProjection;

    #[test]
    fn format_madr_markdown_produces_valid_madr() {
        let result = MemoryProjection::format_madr_markdown(
            "ADR-001: Use Rust for Core Engine",
            "We evaluated Rust, Go, and Python for the core engine.",
            "We selected Rust for its safety guarantees and zero-cost abstractions.",
            "Increased compile times but eliminated entire classes of runtime bugs.",
        );

        assert!(result.starts_with("# ADR-001: Use Rust for Core Engine"));
        assert!(result.contains("## Context\nWe evaluated Rust, Go, and Python"));
        assert!(result.contains("## Decision\nWe selected Rust"));
        assert!(result.contains("## Consequences\nIncreased compile times"));
    }

    #[test]
    fn format_madr_markdown_handles_multiline_fields() {
        let result = MemoryProjection::format_madr_markdown(
            "ADR: Multiline",
            "Context line 1\nContext line 2",
            "Decision line 1\nDecision line 2",
            "Consequence line 1\nConsequence line 2",
        );

        assert!(result.contains("Context line 1\nContext line 2"));
        assert!(result.contains("Decision line 1\nDecision line 2"));
        assert!(result.contains("Consequence line 1\nConsequence line 2"));
    }
}
