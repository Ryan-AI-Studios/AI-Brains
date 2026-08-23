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
    /// Inventory list (T216): status + optional project/tag + limit (caller uses limit+1).
    ///
    /// Parameterized SQL only (`(sql, params)` SOOT). When `filter.tag` is `Some`,
    /// SQL pre-filters `content LIKE 'TAGS:%'` (start-anchored); token match is the
    /// caller's responsibility on list rows (or use [`Self::count_memories`] for totals).
    fn list_memories(&self, filter: &MemoryListFilter) -> Result<Vec<MemoryListRow>>;
    /// Authority-marker inventory (T287): same filter as [`Self::list_memories`]
    /// plus bind-free GLOB extra on `mp.content` (marker **or** TAGS envelope).
    fn list_authority_memories(&self, filter: &MemoryListFilter) -> Result<Vec<MemoryListRow>>;
    /// Total matching rows for a filter (no LIMIT). When `tag` is set, applies
    /// two-stage TAGS: prefix + case-insensitive exact token match (T216 F12).
    fn count_memories(&self, filter: &MemoryListFilter) -> Result<u64>;
    /// Per-project pinned/forgotten counts for global summary (T216 F11/F38).
    ///
    /// Only projects with pinned > 0 OR forgotten > 0; excludes null `project_id`.
    /// Ordered by `(pinned+forgotten) DESC, project_id ASC`.
    fn count_memories_by_project(&self) -> Result<Vec<(String, u64, u64)>>;
    /// Count of forgotten memories, optionally scoped to one project (T216 F42).
    ///
    /// Mirrors [`Self::count_pinned_memories`]: `None` = vault-wide; `Some` filters
    /// `memory_projection.project_id = ?`.
    fn count_forgotten_memories(
        &self,
        project_id: Option<&ai_brains_core::ids::ProjectId>,
    ) -> Result<u64>;
    /// Legacy forgotten list (T216: thin-wraps [`Self::list_memories`] with a high limit).
    /// Production CLI uses bounded `list_memories` / `memory list` instead.
    fn list_forgotten_memories(
        &self,
        project_id: Option<ai_brains_core::ids::ProjectId>,
    ) -> Result<Vec<(String, String)>>;
    fn resolve_project_id_from_alias(
        &self,
        alias: &str,
    ) -> Result<Option<ai_brains_core::ids::ProjectId>>;
    /// All registered path aliases for nightly multi-root Phase 2 (T233).
    ///
    /// Rows from `repository_path_alias_projection`, ordered by
    /// `normalized_path ASC`. Each entry is `(project_id, normalized_path)`.
    fn list_path_aliases(&self) -> Result<Vec<(ai_brains_core::ids::ProjectId, String)>>;
    /// Owner of a normalized path alias, if any (T233 F21 conflict check).
    ///
    /// `normalized_path` must already be normalized via
    /// `ai_brains_path::normalize_for_location_compare`.
    fn find_path_alias_owner(
        &self,
        normalized_path: &str,
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
    /// Count of distinct non-null projects that have at least one pinned memory (T214 F7).
    ///
    /// Used by `preflight --global --summary` vault rollup. Does **not** use
    /// `list_projects` (which over-counts unpinned turns).
    fn count_projects_with_pinned(&self) -> Result<u64>;
    /// Count of pinned memories, optionally scoped to one project (T214 F8).
    ///
    /// `None` = vault-wide; `Some` = filter `memory_projection.project_id = ?`.
    fn count_pinned_memories(
        &self,
        project_id: Option<&ai_brains_core::ids::ProjectId>,
    ) -> Result<u64>;
    /// Count of active sessions, optionally scoped to one project (T214 F5).
    ///
    /// `None` = vault-wide; `Some` = filter `session_projection.project_id = ?`.
    /// Prefer this over loading turns via retrieval `active_sessions`.
    fn count_active_sessions(
        &self,
        project_id: Option<&ai_brains_core::ids::ProjectId>,
    ) -> Result<u64>;
    fn get_session_memory_ids(&self, session_id: &str) -> Result<Vec<MemoryId>>;
    /// Returns true iff a row with this `memory_id` exists in `memory_projection`.
    /// Used by `forget` to validate `--memory-id` before appending a
    /// `MemoryForgotten` event that would otherwise silently no-op.
    fn memory_exists(&self, memory_id: &str) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// T216 — memory inventory list / count types
// ---------------------------------------------------------------------------

/// Status filter for [`QueryStore::list_memories`] / [`QueryStore::count_memories`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryListStatus {
    Pinned,
    Forgotten,
}

impl MemoryListStatus {
    /// Wire / SQL status string (`pinned` | `forgotten`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pinned => "pinned",
            Self::Forgotten => "forgotten",
        }
    }
}

/// One row from a memory inventory list (T216).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryListRow {
    pub memory_id: String,
    pub content: String,
    pub updated_at: String,
    pub project_id: Option<String>,
    pub status: String,
}

/// Filter for memory inventory list/count (T216 F15).
#[derive(Debug, Clone)]
pub struct MemoryListFilter {
    pub status: MemoryListStatus,
    /// `None` = global (no project predicate); `Some` = project-scoped.
    pub project_id: Option<ai_brains_core::ids::ProjectId>,
    /// When `Some`, SQL pre-filter `content LIKE 'TAGS:%'` (start-anchored).
    /// Token match for count is applied in-store; list callers re-filter.
    pub tag: Option<String>,
    /// Query page size (caller typically passes `limit + 1` for more_available).
    /// For tag candidate over-fetch, pass the elevated candidate cap.
    pub limit: usize,
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
