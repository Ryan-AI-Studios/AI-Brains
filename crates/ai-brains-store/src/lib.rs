pub mod config;
pub mod connection;
pub mod encrypt;
pub mod errors;
pub mod event_store;
pub mod fts;
pub mod header;
pub mod migrations;
pub mod pragmas;
pub mod projections;
pub mod query_store;
pub mod replay;
pub mod replication_engine;
pub mod rotate;
pub mod sqlcipher_log_policy;
pub mod transaction;

pub use connection::{ALLOW_ZERO_KEY_ENV, VaultConnection};
pub use encrypt::{EncryptOptions, encrypt_plaintext_vault};
pub use errors::{Result, StoreError};
pub use event_store::{EventStore, SqliteEventStore};
pub use fts::{FtsSearch, SearchResult};
pub use header::{SQLITE_PLAIN_HEADER, is_plain_sqlite_header, legacy_plaintext_migrate_hint};
pub use migrations::apply_migrations_through;
pub use projections::content_envelope::{list_active_content_key_wraps, update_content_key_wrap};
pub use replication_engine::{
    ApplyOutcome, EngineError, EngineResult, ReplicateEngine, sign_and_queue_erasure_tombstone,
    sign_and_queue_revoke, signed_to_blob,
};
pub use rotate::{
    RotateDataKeyOptions, RotateDataKeyResult, RotateDryRunPlan, RotateMethod, atomic_replace_file,
    plan_rotate_datakey, rotate_datakey,
};
pub use transaction::Transaction;

use ai_brains_core::ids::{MemoryId, SessionId};
use ai_brains_core::privacy::Privacy;

pub trait QueryStore: std::marker::Send + std::marker::Sync {
    fn get_unsummarized_sessions(&self) -> Result<Vec<String>>;
    fn get_session_turns(&self, session_id: &str) -> Result<Vec<(String, String)>>;
    fn get_session_status(&self, session_id: &SessionId) -> Result<Option<String>>;
    fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<(MemoryId, String)>>;
    fn get_memories_by_level(
        &self,
        level: u32,
        limit: Option<usize>,
    ) -> Result<Vec<(MemoryId, String)>>;
    /// Privacy flag for a memory in `memory_projection`, if present.
    fn get_memory_privacy(&self, memory_id: &MemoryId) -> Result<Option<Privacy>>;
    fn delete_old_turns(&self, cutoff: chrono::DateTime<chrono::Utc>) -> Result<usize>;
    fn list_forgotten_memories(
        &self,
        project_id: Option<ai_brains_core::ids::ProjectId>,
    ) -> Result<Vec<(String, String)>>;
    fn resolve_project_id_from_alias(
        &self,
        alias: &str,
    ) -> Result<Option<ai_brains_core::ids::ProjectId>>;
    fn get_max_turn_index(&self, session_id: &SessionId) -> Result<Option<i32>>;
    fn get_sync_state(&self, key: &str) -> Result<Option<String>>;
    fn get_last_nightly_run(&self) -> Result<Option<String>>;
    fn store_embedding(&self, memory_id: &str, embedding: &[u8]) -> Result<()>;
    fn get_memories_without_embeddings(
        &self,
        limit: usize,
        since_days: Option<i32>,
    ) -> Result<Vec<(String, String)>>;
    fn get_stale_memories(
        &self,
        days_threshold: i32,
        limit: usize,
    ) -> Result<Vec<(String, String)>>;
    fn list_projects(&self) -> Result<Vec<(String, String, String, usize)>>; // UUID, name, alias, memory_count
    /// Extended project list for UI/JSON (label-first list command).
    ///
    /// `last_activity` is last **memory-projection mutation** (pin/forget/ingest/turn
    /// upsert), falling back to project `updated_at` when the project has no memories.
    /// `path` is the lexicographically first registered `normalized_path` for the
    /// project, or `None` when no path alias exists (never invented).
    fn list_projects_detail(&self) -> Result<Vec<ProjectListDetail>>;
    /// Look up a single project by id. Returns `(name, alias)` when present.
    /// Alias is empty string when none is set. Does not load the full project list.
    fn get_project_by_id(
        &self,
        project_id: &ai_brains_core::ids::ProjectId,
    ) -> Result<Option<(String, String)>>;
    fn get_session_memory_ids(&self, session_id: &str) -> Result<Vec<MemoryId>>;
    /// Returns true iff a row with this `memory_id` exists in `memory_projection`.
    /// Used by `forget` to validate `--memory-id` before appending a
    /// `MemoryForgotten` event that would otherwise silently no-op.
    fn memory_exists(&self, memory_id: &str) -> Result<bool>;
}

/// Row from [`QueryStore::list_projects_detail`] (T212).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectListDetail {
    pub project_id: String,
    pub name: String,
    /// Empty string when no alias is set.
    pub alias: String,
    pub memory_count: usize,
    /// RFC 3339 timestamp string from SQL, or empty if missing.
    pub last_activity: String,
    /// Lexicographically first registered repo path, if any.
    pub path: Option<String>,
}

pub trait SyncStateStore: std::marker::Send + std::marker::Sync {
    fn set_sync_state(&self, key: &str, value: &str) -> Result<()>;
}
