use crate::errors::{Result, StoreError};
use crate::event_store::SqliteEventStore;
use crate::projections;
use ai_brains_events::Envelope;
use uuid::Uuid;

impl SqliteEventStore {
    pub fn rebuild_projections(&mut self) -> Result<()> {
        let mut envelopes = Vec::new();
        {
            let conn = self.conn.lock()?;
            // 1. Fetch all events first to avoid borrowing the transaction while iterating
            let mut stmt = conn.prepare(
                "SELECT 
                event_id, schema_version, aggregate_type, aggregate_id, event_type,
                occurred_at, actor_json, causation_id, correlation_id, privacy,
                payload_json, payload_hash
            FROM events 
            ORDER BY occurred_at ASC",
            )?;

            let mut rows = stmt.query([])?;

            while let Some(row) = rows.next()? {
                let event_id_str: String = row.get(0)?;
                let event_id = Uuid::parse_str(&event_id_str)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let schema_version: u32 = row.get(1)?;

                let aggregate_type_str: String = row.get(2)?;
                let aggregate_type = serde_json::from_str(&format!("\"{}\"", aggregate_type_str))
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let aggregate_id_str: String = row.get(3)?;
                let aggregate_id = Uuid::parse_str(&aggregate_id_str)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let event_type_str: String = row.get(4)?;
                let event_type = serde_json::from_str(&format!("\"{}\"", event_type_str))
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let occurred_at_str: String = row.get(5)?;
                let occurred_at = time::OffsetDateTime::parse(
                    &occurred_at_str,
                    &time::format_description::well_known::Rfc3339,
                )
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let actor_json: String = row.get(6)?;
                let actor = serde_json::from_str(&actor_json)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let causation_id_str: Option<String> = row.get(7)?;
                let causation_id = if let Some(s) = causation_id_str.as_ref() {
                    Some(
                        Uuid::parse_str(s)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    )
                } else {
                    None
                };

                let correlation_id_str: Option<String> = row.get(8)?;
                let correlation_id = if let Some(s) = correlation_id_str.as_ref() {
                    Some(
                        Uuid::parse_str(s)
                            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?,
                    )
                } else {
                    None
                };

                let privacy_json: String = row.get(9)?;
                let privacy = serde_json::from_str(&privacy_json)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let payload_json: String = row.get(10)?;
                let payload = serde_json::from_str(&payload_json)
                    .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

                let payload_hash: String = row.get(11)?;

                envelopes.push(Envelope {
                    event_id,
                    schema_version,
                    aggregate_type,
                    aggregate_id,
                    event_type,
                    occurred_at,
                    actor,
                    causation_id,
                    correlation_id,
                    privacy,
                    payload,
                    payload_hash,
                });
            }
        }

        let mut conn = self.conn.lock()?;
        let tx = conn
            .transaction()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        // 2. Truncate projections
        tx.execute("DELETE FROM memory_projection", [])?;
        tx.execute("DELETE FROM turn_projection", [])?;
        tx.execute("DELETE FROM session_projection", [])?;
        tx.execute("DELETE FROM project_projection", [])?;
        tx.execute("DELETE FROM memory_fts", [])?;

        // 3. Apply all events
        for envelope in envelopes {
            projections::apply_all(&tx, &envelope)?;
        }

        tx.commit()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        Ok(())
    }
}
