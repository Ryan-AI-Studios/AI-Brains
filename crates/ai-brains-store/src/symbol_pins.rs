//! Chunked existence check for symbol `MemoryPinned` rows (T373).
//!
//! The nightly path must not call `read_events` once per candidate. `EventStore`
//! stays unchanged; callers pass a [`crate::connection::VaultConnection`].

use crate::connection::VaultConnection;
use crate::errors::Result;
use rusqlite::params_from_iter;
use std::collections::HashSet;
use uuid::Uuid;

/// Host-parameter chunk. Below SQLite's historical 999 default and the
/// post-3.32 default of 32766, so a lowered compile-time limit still works.
pub const SYMBOL_PIN_CHUNK: usize = 500;

const SYMBOL_TAG_CANONICAL: &str = "ledgerful:symbol";
const SYMBOL_TAG_LEGACY: &str = "changeguard:symbol";

/// Half-open ranges covering `len` items in steps of `chunk`.
pub fn chunk_symbol_pin_ids(len: usize, chunk: usize) -> Vec<(usize, usize)> {
    if chunk == 0 || len == 0 {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut start = 0usize;
    while start < len {
        let end = (start + chunk).min(len);
        ranges.push((start, end));
        start = end;
    }
    ranges
}

/// Ids that already have a `MemoryPinned` row tagged as a symbol pin.
///
/// `source_tag` is read in Rust. The SQL does not use `json_extract`.
pub fn symbol_pin_ids_present(conn: &VaultConnection, ids: &[Uuid]) -> Result<HashSet<Uuid>> {
    let mut found = HashSet::new();
    if ids.is_empty() {
        return Ok(found);
    }
    let guard = conn.lock()?;
    for (start, end) in chunk_symbol_pin_ids(ids.len(), SYMBOL_PIN_CHUNK) {
        let slice = &ids[start..end];
        let mut sql = String::from(
            "SELECT aggregate_id, payload_json FROM events \
             WHERE aggregate_type = 'memory' AND event_type = 'MemoryPinned' \
             AND aggregate_id IN (",
        );
        for i in 0..slice.len() {
            if i > 0 {
                sql.push(',');
            }
            sql.push('?');
        }
        sql.push(')');
        let params: Vec<String> = slice.iter().map(|id| id.to_string()).collect();
        let mut stmt = guard.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params.iter()))?;
        while let Some(row) = rows.next()? {
            let aggregate_id: String = row.get(0)?;
            let payload_json: String = row.get(1)?;
            if !payload_is_symbol_pin(&payload_json) {
                continue;
            }
            if let Ok(id) = Uuid::parse_str(&aggregate_id) {
                found.insert(id);
            }
        }
    }
    Ok(found)
}

fn payload_is_symbol_pin(payload_json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload_json) else {
        return false;
    };
    let tag = value.get("source_tag").and_then(|v| v.as_str());
    matches!(tag, Some(SYMBOL_TAG_CANONICAL) | Some(SYMBOL_TAG_LEGACY))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::event_store::{EventStore, SqliteEventStore};
    use ai_brains_core::ids::{MemoryId, ProjectId};
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    fn open_store() -> Result<(NamedTempFile, SqliteEventStore)> {
        let temp_file = NamedTempFile::new()
            .map_err(|e| crate::errors::StoreError::ConnectionFailed(e.to_string()))?;
        let key = SqlCipherKey::from_data_key(&DataKey::generate());
        let conn = VaultConnection::open(temp_file.path(), &key)?;
        conn.migrate()?;
        Ok((temp_file, SqliteEventStore::new(conn)))
    }

    fn pin(store: &SqliteEventStore, id: Uuid, source_tag: Option<&str>) -> Result<()> {
        let memory_id = MemoryId::from_uuid(id);
        let envelope =
            EventBuilder::new(AggregateType::Memory, id, Actor::System, Privacy::LocalOnly)
                .build(Payload::MemoryPinned(MemoryPinnedPayload {
                    memory_id,
                    content: "symbol stub".to_string(),
                    session_id: None,
                    project_id: Some(ProjectId::new()),
                    tx_id: None,
                    rank: None,
                    source_tag: source_tag.map(str::to_string),
                    query_text: None,
                }))
                .map_err(|e| crate::errors::StoreError::EventAppendFailed(e.to_string()))?;
        store.append_event(&envelope)?;
        Ok(())
    }

    #[test]
    fn chunk_symbol_pin_ids__501__500_and_1() {
        let ranges = chunk_symbol_pin_ids(501, SYMBOL_PIN_CHUNK);
        let lengths: Vec<usize> = ranges.iter().map(|(s, e)| e - s).collect();
        assert_eq!(lengths, vec![500, 1]);
    }

    #[test]
    fn symbol_pin_ids_present__legacy_and_canonical__chunked() -> Result<()> {
        let (_file, store) = open_store()?;
        let legacy = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"legacy");
        let canonical = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"canonical");
        let ordinary = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"ordinary");
        pin(&store, legacy, Some(super::SYMBOL_TAG_LEGACY))?;
        pin(&store, canonical, Some(super::SYMBOL_TAG_CANONICAL))?;
        pin(&store, ordinary, Some("note"))?;

        let mut ids = Vec::with_capacity(501);
        ids.push(legacy);
        ids.push(ordinary);
        for n in 0..498 {
            ids.push(Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("pad-{n}").as_bytes(),
            ));
        }
        ids.push(canonical);
        assert_eq!(ids.len(), 501);

        let found = symbol_pin_ids_present(store.connection(), &ids)?;
        assert!(found.contains(&legacy));
        assert!(found.contains(&canonical));
        assert!(!found.contains(&ordinary));
        assert_eq!(found.len(), 2);
        Ok(())
    }
}
