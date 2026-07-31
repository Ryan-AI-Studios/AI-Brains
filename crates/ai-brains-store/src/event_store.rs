use crate::SyncStateStore;
use crate::connection::VaultConnection;
use crate::errors::{Result, StoreError};
use crate::projections;
use crate::projections::replication::{
    DevicePrivateKeyRow, has_active_or_local_device, put_device_private_key_wrap,
};
use ai_brains_events::Envelope;
use ai_brains_events::Payload;
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

pub trait EventStore: Send + Sync {
    fn append_event(&self, envelope: &Envelope) -> Result<()>;
    /// Append multiple events in a single transaction (all-or-nothing).
    fn append_events(&self, envelopes: &[Envelope]) -> Result<()>;
    fn read_events(&self, aggregate_id: Uuid) -> Result<Vec<Envelope>>;
    fn read_all_events(&self) -> Result<Vec<Envelope>>;
    fn get_sync_state(&self, key: &str) -> Result<Option<String>>;
    fn set_sync_state(&self, key: &str, value: &str) -> Result<()>;
    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> Result<Option<ai_brains_core::privacy::Privacy>>;
}

pub struct SqliteEventStore {
    pub conn: VaultConnection,
}

impl SqliteEventStore {
    pub fn new(conn: VaultConnection) -> Self {
        Self { conn }
    }

    pub fn connection(&self) -> &VaultConnection {
        &self.conn
    }

    /// Append a `DeviceEnrolled` event and store the local private-key wrap in one
    /// IMMEDIATE transaction (bootstrap path).
    ///
    /// Order: R27 check (when `status = local`) → event row + projector
    /// (identity/control/index) → private key wrap. Failure of any step rolls back
    /// the event so SOV and side stores cannot diverge. Private key material is
    /// never written into the event payload.
    pub fn append_device_enrolled_with_private_key(
        &self,
        envelope: &Envelope,
        private_key: &DevicePrivateKeyRow,
    ) -> Result<()> {
        validate_envelope_payload(envelope)?;

        let Payload::DeviceEnrolled(ref enrolled) = envelope.payload else {
            return Err(StoreError::EventAppendFailed(
                "append_device_enrolled_with_private_key requires DeviceEnrolled payload"
                    .to_string(),
            ));
        };

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        // R27: reject concurrent second local bootstrap inside the write lock.
        if enrolled.status == "local" && has_active_or_local_device(&tx)? {
            return Err(StoreError::ConfigError(
                "BootstrapAlreadyEnrolled: an active or local device already exists".to_string(),
            ));
        }

        insert_event_row(&tx, envelope)?;
        put_device_private_key_wrap(&tx, private_key)?;

        tx.commit()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
        Ok(())
    }
}

impl SyncStateStore for SqliteEventStore {
    fn set_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock()?;
        conn.execute(
            "INSERT INTO sync_state (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn validate_envelope_payload(envelope: &Envelope) -> Result<()> {
    if let Payload::ConclusionMarkedStale(ref p) = envelope.payload {
        p.validate()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
    }
    Ok(())
}

fn insert_event_row(tx: &rusqlite::Transaction<'_>, envelope: &Envelope) -> Result<()> {
    let actor_json = serde_json::to_string(&envelope.actor)
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
    let payload_json = serde_json::to_string(&envelope.payload)
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
    let occurred_at = envelope
        .occurred_at
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| StoreError::EventAppendFailed(format!("Failed to format date: {}", e)))?;

    let aggregate_type_str = serde_json::to_string(&envelope.aggregate_type)
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?
        .trim_matches('"')
        .to_string();

    let event_type_str = serde_json::to_string(&envelope.event_type)
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?
        .trim_matches('"')
        .to_string();

    tx.execute(
        "INSERT INTO events (
            event_id, schema_version, aggregate_type, aggregate_id, event_type,
            occurred_at, actor_json, causation_id, correlation_id, privacy,
            payload_json, payload_hash
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            envelope.event_id.to_string(),
            envelope.schema_version,
            aggregate_type_str,
            envelope.aggregate_id.to_string(),
            event_type_str,
            occurred_at,
            actor_json,
            envelope.causation_id.map(|u| u.to_string()),
            envelope.correlation_id.map(|u| u.to_string()),
            serde_json::to_string(&envelope.privacy)
                .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?,
            payload_json,
            envelope.payload_hash,
        ],
    )
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("events are immutable") {
            StoreError::ImmutableEventModified(msg)
        } else if msg.to_lowercase().contains("unique") {
            StoreError::EventAppendFailed(format!("unique_event_id:{msg}"))
        } else {
            StoreError::EventAppendFailed(msg)
        }
    })?;

    projections::apply_all(tx, envelope)?;
    Ok(())
}

/// Insert a domain event + projectors on an existing transaction (no commit).
///
/// Duplicate `event_id` (UNIQUE) is **idempotent Ok** so re-apply of the same
/// sealed data envelope is safe. Callers that need a full commit should use
/// [`append_event_on_connection`] or commit the outer transaction themselves
/// (e.g. replication `apply_blob` single-TX path).
pub fn append_event_in_tx(tx: &rusqlite::Transaction<'_>, envelope: &Envelope) -> Result<()> {
    validate_envelope_payload(envelope)?;

    let already = tx
        .query_row(
            "SELECT 1 FROM events WHERE event_id = ?1",
            params![envelope.event_id.to_string()],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?
        .unwrap_or(false);
    if already {
        return Ok(());
    }

    match insert_event_row(tx, envelope) {
        Ok(()) => Ok(()),
        Err(StoreError::EventAppendFailed(msg)) if msg.starts_with("unique_event_id:") => {
            // Race-safe idempotent path; outer TX may still commit other work.
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Append a domain event on a raw `&Connection` (standalone replication helper).
///
/// Begins a transaction, inserts the event row + runs projectors, commits.
/// Duplicate `event_id` (UNIQUE) is **idempotent Ok**. Does not require
/// [`VaultConnection`] / mutex — the caller already holds the connection.
///
/// Prefer [`append_event_in_tx`] when the caller already owns a transaction
/// (nested `BEGIN` is not supported).
pub fn append_event_on_connection(conn: &Connection, envelope: &Envelope) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
    match append_event_in_tx(&tx, envelope) {
        Ok(()) => {
            tx.commit()
                .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;
            Ok(())
        }
        Err(e) => {
            // Drop rolls back any partial insert from this TX.
            drop(tx);
            Err(e)
        }
    }
}

impl EventStore for SqliteEventStore {
    fn append_event(&self, envelope: &Envelope) -> Result<()> {
        validate_envelope_payload(envelope)?;

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        insert_event_row(&tx, envelope)?;

        tx.commit()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        Ok(())
    }

    fn append_events(&self, envelopes: &[Envelope]) -> Result<()> {
        if envelopes.is_empty() {
            return Ok(());
        }

        for envelope in envelopes {
            validate_envelope_payload(envelope)?;
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        let tx = conn
            .transaction()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        for envelope in envelopes {
            insert_event_row(&tx, envelope)?;
        }

        tx.commit()
            .map_err(|e| StoreError::EventAppendFailed(e.to_string()))?;

        Ok(())
    }

    fn read_events(&self, aggregate_id: Uuid) -> Result<Vec<Envelope>> {
        self.read_events_internal(Some(aggregate_id))
    }

    fn read_all_events(&self) -> Result<Vec<Envelope>> {
        self.read_events_internal(None)
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare("SELECT value FROM sync_state WHERE key = ?")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn set_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock()?;
        conn.execute(
            "INSERT INTO sync_state (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn get_session_privacy(
        &self,
        session_id: &str,
    ) -> Result<Option<ai_brains_core::privacy::Privacy>> {
        let conn = self.conn.lock()?;
        let mut stmt =
            conn.prepare("SELECT privacy FROM session_projection WHERE session_id = ?")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let p_str: String = row.get(0)?;
            let p: ai_brains_core::privacy::Privacy = serde_json::from_str(&p_str)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }
}

impl SqliteEventStore {
    fn read_events_internal(&self, aggregate_id: Option<Uuid>) -> Result<Vec<Envelope>> {
        let (query, params) = match aggregate_id {
            Some(id) => (
                "SELECT 
                    event_id, schema_version, aggregate_type, aggregate_id, event_type,
                    occurred_at, actor_json, causation_id, correlation_id, privacy,
                    payload_json, payload_hash
                FROM events 
                WHERE aggregate_id = ?
                ORDER BY occurred_at ASC, event_id ASC",
                vec![id.to_string()],
            ),
            None => (
                "SELECT 
                    event_id, schema_version, aggregate_type, aggregate_id, event_type,
                    occurred_at, actor_json, causation_id, correlation_id, privacy,
                    payload_json, payload_hash
                FROM events 
                ORDER BY occurred_at ASC, event_id ASC",
                vec![],
            ),
        };

        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params))?;
        let mut events = Vec::new();

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
            let causation_id = causation_id_str
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let correlation_id_str: Option<String> = row.get(8)?;
            let correlation_id = correlation_id_str
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let privacy_json: String = row.get(9)?;
            let privacy = serde_json::from_str(&privacy_json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let payload_json: String = row.get(10)?;
            let payload = serde_json::from_str(&payload_json)
                .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

            let payload_hash: String = row.get(11)?;

            events.push(Envelope {
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

        Ok(events)
    }
}
