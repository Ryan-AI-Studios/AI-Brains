//! Deterministic governed-memory fixture loader for store integration tests.
//!
//! Loads fixed NDJSON envelopes (not EventBuilder) into a tempfile vault and
//! exports selected projection rows for golden comparison.

use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::Envelope;
use ai_brains_store::StoreError;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Absolute path to a file under workspace `fixtures/governed-memory/`.
pub fn governed_fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/governed-memory")
        .join(file_name)
}

/// Zero key matching CLI init defaults for reproducible fixture vaults.
pub fn fixture_zero_sql_key() -> SqlCipherKey {
    SqlCipherKey::from_raw(
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
    )
}

/// Load NDJSON envelopes from `legacy-v1-events.ndjson` (or given path).
pub fn load_envelopes_from_ndjson(
    path: &Path,
) -> Result<Vec<Envelope>, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let mut envelopes = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let env: Envelope = serde_json::from_str(trimmed).map_err(|e| {
            format!(
                "failed to parse envelope at {}:{}: {e}",
                path.display(),
                line_no + 1
            )
        })?;
        envelopes.push(env);
    }
    Ok(envelopes)
}

/// Open a fresh tempfile vault, migrate through current migrations, append
/// envelopes as-is (fixed event_id / payload_hash).
pub struct LoadedFixture {
    pub _temp: TempDir,
    pub vault_path: PathBuf,
    pub store: SqliteEventStore,
    pub envelopes: Vec<Envelope>,
}

impl LoadedFixture {
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from_path(&governed_fixture_path("legacy-v1-events.ndjson"))
    }

    pub fn load_from_path(ndjson_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let envelopes = load_envelopes_from_ndjson(ndjson_path)?;
        let temp = tempfile::tempdir()?;
        let vault_path = temp.path().join("fixture-vault.db");
        let key = fixture_zero_sql_key();
        let conn = VaultConnection::open(&vault_path, &key)?;
        conn.migrate()?;
        let store = SqliteEventStore::new(conn);
        for env in &envelopes {
            store.append_event(env)?;
        }
        Ok(Self {
            _temp: temp,
            vault_path,
            store,
            envelopes,
        })
    }

    /// Re-append the first fixture envelope (same event_id) for immutability checks.
    pub fn append_duplicate_first(&self) -> Result<(), StoreError> {
        let first = self
            .envelopes
            .first()
            .ok_or_else(|| StoreError::ConfigError("fixture has no envelopes".to_string()))?;
        self.store.append_event(first)
    }

    /// Export selected projection rows as stable JSON (excludes embedding blobs;
    /// memory_id omitted because turn projection assigns random MemoryIds).
    pub fn export_selected_projections(&self) -> Result<Value, Box<dyn std::error::Error>> {
        export_selected_projections(&self.store)
    }
}

/// Query selected tables into a deterministic JSON snapshot.
pub fn export_selected_projections(
    store: &SqliteEventStore,
) -> Result<Value, Box<dyn std::error::Error>> {
    let conn = store.connection().lock()?;

    let mut projects = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT project_id, name, tx_id, created_at, updated_at
             FROM project_projection
             ORDER BY project_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "project_id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "tx_id": row.get::<_, Option<String>>(2)?,
                "created_at": row.get::<_, String>(3)?,
                "updated_at": row.get::<_, String>(4)?,
            }))
        })?;
        for r in rows {
            projects.push(r?);
        }
    }

    let mut sessions = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT session_id, project_id, status, privacy, tx_id, summary_memory_id,
                    summarized_at, created_at, updated_at
             FROM session_projection
             ORDER BY session_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "session_id": row.get::<_, String>(0)?,
                "project_id": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "privacy": row.get::<_, String>(3)?,
                "tx_id": row.get::<_, Option<String>>(4)?,
                "summary_memory_id": row.get::<_, Option<String>>(5)?,
                "summarized_at": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "updated_at": row.get::<_, String>(8)?,
            }))
        })?;
        for r in rows {
            sessions.push(r?);
        }
    }

    let mut turns = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT session_id, project_id, turn_index, role, content, tx_id, occurred_at
             FROM turn_projection
             ORDER BY session_id, turn_index",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "session_id": row.get::<_, String>(0)?,
                "project_id": row.get::<_, Option<String>>(1)?,
                "turn_index": row.get::<_, i64>(2)?,
                "role": row.get::<_, String>(3)?,
                "content": row.get::<_, String>(4)?,
                "tx_id": row.get::<_, Option<String>>(5)?,
                "occurred_at": row.get::<_, String>(6)?,
            }))
        })?;
        for r in rows {
            turns.push(r?);
        }
    }

    // Exclude memory_id: turn projection uses MemoryId::new() (non-deterministic).
    // Sort by content + session_id + created_at for stable ordering.
    let mut memories = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT content, privacy, status, level, session_id, project_id, tx_id,
                    created_at, updated_at
             FROM memory_projection
             ORDER BY content, session_id, created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(json!({
                "content": row.get::<_, String>(0)?,
                "privacy": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "level": row.get::<_, i64>(3)?,
                "session_id": row.get::<_, Option<String>>(4)?,
                "project_id": row.get::<_, Option<String>>(5)?,
                "tx_id": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "updated_at": row.get::<_, String>(8)?,
            }))
        })?;
        for r in rows {
            memories.push(r?);
        }
    }

    let fts_count: i64 = conn.query_row("SELECT COUNT(*) FROM memory_fts", [], |row| row.get(0))?;

    Ok(json!({
        "project_projection": projects,
        "session_projection": sessions,
        "turn_projection": turns,
        "memory_projection": memories,
        "memory_fts_count": fts_count,
    }))
}
