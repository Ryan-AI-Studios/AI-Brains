//! Content-envelope side stores + erasure/tombstone projections (T163 / P8.1).
//!
//! Side stores (`content_key_store`, `encrypted_content_blob`) are written by
//! command paths (T164/T165), not by this projection. Event projections track
//! `ContentErasureRequested` / `ContentErased` only. No crypto seal/open here —
//! wrap and ciphertext columns hold opaque fixture or sealed bytes.
//!
//! **S14:** `ContentErasureRequested` must never demote a completed erasure row
//! back to `requested` (conditional UPSERT; do not copy ReviewProjection's
//! open-always pattern).

use crate::errors::{Result, StoreError};
use crate::projections::Projection;
use ai_brains_events::{Envelope, Payload};
use rusqlite::{Connection, OptionalExtension, params};
use std::fmt;
use time::format_description::well_known::Rfc3339;

/// Envelope / wrap schema version locked by ADR-0016 / T163 (S2).
pub const ENVELOPE_SCHEMA_VERSION: i64 = 1;

/// Forensic algorithm label (not a runtime cipher switch).
pub const ALGORITHM_AES_256_GCM: &str = "AES-256-GCM";

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

/// Row from `content_key_store`. Destroyed keys return wrap fields as `None`.
///
/// Debug redacts wrap material (lengths only) so accidental `{:?}` logging
/// does not expose ciphertext (spec §6.2 / ADR-0016 §5).
#[derive(Clone, PartialEq, Eq)]
pub struct ContentKeyWrapRow {
    pub content_key_id: String,
    pub wrap_schema_version: i64,
    pub algorithm: String,
    pub wrap_nonce: Option<Vec<u8>>,
    pub wrap_ciphertext: Option<Vec<u8>>,
    pub status: String,
    pub created_at: String,
    pub destroyed_at: Option<String>,
}

impl fmt::Debug for ContentKeyWrapRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentKeyWrapRow")
            .field("content_key_id", &self.content_key_id)
            .field("wrap_schema_version", &self.wrap_schema_version)
            .field("algorithm", &self.algorithm)
            .field(
                "wrap_nonce",
                &self
                    .wrap_nonce
                    .as_ref()
                    .map(|b| format!("<redacted len={}>", b.len())),
            )
            .field(
                "wrap_ciphertext",
                &self
                    .wrap_ciphertext
                    .as_ref()
                    .map(|b| format!("<redacted len={}>", b.len())),
            )
            .field("status", &self.status)
            .field("created_at", &self.created_at)
            .field("destroyed_at", &self.destroyed_at)
            .finish()
    }
}

/// Opaque ciphertext blob row from `encrypted_content_blob`.
///
/// Debug redacts nonce/ciphertext (lengths only).
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedBlobRow {
    pub blob_id: String,
    pub content_key_id: String,
    pub envelope_schema_version: i64,
    pub algorithm: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub content_class: Option<String>,
    pub subject_kind: Option<String>,
    pub subject_id: Option<String>,
    pub size_bytes: i64,
    pub created_at: String,
}

impl fmt::Debug for EncryptedBlobRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedBlobRow")
            .field("blob_id", &self.blob_id)
            .field("content_key_id", &self.content_key_id)
            .field("envelope_schema_version", &self.envelope_schema_version)
            .field("algorithm", &self.algorithm)
            .field("nonce", &format!("<redacted len={}>", self.nonce.len()))
            .field(
                "ciphertext",
                &format!("<redacted len={}>", self.ciphertext.len()),
            )
            .field("content_class", &self.content_class)
            .field("subject_kind", &self.subject_kind)
            .field("subject_id", &self.subject_id)
            .field("size_bytes", &self.size_bytes)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Row from `erasure_request_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureRequestRow {
    pub content_key_id: String,
    pub requester: String,
    pub reason: String,
    pub status: String,
    pub requested_at: String,
    pub completed_at: Option<String>,
    pub tombstone_id: Option<String>,
}

/// Minimal tombstone row from `tombstone_projection`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TombstoneRow {
    pub tombstone_id: String,
    pub content_key_id: String,
    pub erased_at: String,
    pub reason_code: String,
}

// ---------------------------------------------------------------------------
// Side-store APIs (no crypto; opaque bytes only)
// ---------------------------------------------------------------------------

/// Insert an active content-key wrap. Wrap nonce and ciphertext must be non-empty
/// (app-level reject; SQL CHECK also enforces non-NULL wraps when `status = 'active'`).
pub fn insert_content_key_wrap(
    conn: &Connection,
    content_key_id: &str,
    wrap_schema_version: i64,
    wrap_nonce: &[u8],
    wrap_ciphertext: &[u8],
    created_at: &str,
) -> Result<()> {
    if wrap_nonce.is_empty() {
        return Err(StoreError::ConfigError(
            "content key wrap_nonce must be non-empty".to_string(),
        ));
    }
    if wrap_ciphertext.is_empty() {
        return Err(StoreError::ConfigError(
            "content key wrap_ciphertext must be non-empty".to_string(),
        ));
    }
    conn.execute(
        "INSERT INTO content_key_store (
            content_key_id, wrap_schema_version, algorithm,
            wrap_nonce, wrap_ciphertext, status, created_at, destroyed_at
         ) VALUES (?, ?, ?, ?, ?, 'active', ?, NULL)",
        params![
            content_key_id,
            wrap_schema_version,
            ALGORITHM_AES_256_GCM,
            wrap_nonce,
            wrap_ciphertext,
            created_at,
        ],
    )?;
    Ok(())
}

/// Fetch a content-key wrap row. Missing → `Ok(None)`. Destroyed → `Some` with
/// wrap fields `None` and `status = "destroyed"`.
pub fn get_content_key_wrap(
    conn: &Connection,
    content_key_id: &str,
) -> Result<Option<ContentKeyWrapRow>> {
    let mut stmt = conn.prepare(
        "SELECT content_key_id, wrap_schema_version, algorithm,
                wrap_nonce, wrap_ciphertext, status, created_at, destroyed_at
         FROM content_key_store
         WHERE content_key_id = ?",
    )?;
    let row = stmt
        .query_row(params![content_key_id], |row| {
            Ok(ContentKeyWrapRow {
                content_key_id: row.get(0)?,
                wrap_schema_version: row.get(1)?,
                algorithm: row.get(2)?,
                wrap_nonce: row.get(3)?,
                wrap_ciphertext: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                destroyed_at: row.get(7)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// Destroy a content-key wrap: NULL wrap columns, `status = 'destroyed'`, set
/// `destroyed_at`. Idempotent if already destroyed (preserves first
/// `destroyed_at`). Missing key is a no-op (`Ok(())`).
pub fn destroy_content_key_wrap(
    conn: &Connection,
    content_key_id: &str,
    destroyed_at: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE content_key_store
         SET wrap_nonce = NULL,
             wrap_ciphertext = NULL,
             status = 'destroyed',
             destroyed_at = COALESCE(destroyed_at, ?)
         WHERE content_key_id = ?",
        params![destroyed_at, content_key_id],
    )?;
    Ok(())
}

/// Insert an opaque encrypted content blob. `size_bytes` must equal
/// `ciphertext.len()` at insert time. Logical FK (spec §5.2): `content_key_id`
/// must already exist in `content_key_store`.
pub fn insert_encrypted_blob(conn: &Connection, row: &EncryptedBlobRow) -> Result<()> {
    let ciphertext_len = row.ciphertext.len() as i64;
    if row.size_bytes != ciphertext_len {
        return Err(StoreError::ConfigError(format!(
            "encrypted blob size_bytes ({}) must equal ciphertext.len() ({})",
            row.size_bytes, ciphertext_len
        )));
    }
    let key_exists: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM content_key_store WHERE content_key_id = ?
         )",
        params![row.content_key_id],
        |r| r.get(0),
    )?;
    if !key_exists {
        return Err(StoreError::ConfigError(format!(
            "content_key_id does not exist in content_key_store: {}",
            row.content_key_id
        )));
    }
    conn.execute(
        "INSERT INTO encrypted_content_blob (
            blob_id, content_key_id, envelope_schema_version, algorithm,
            nonce, ciphertext, content_class, subject_kind, subject_id,
            size_bytes, created_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            row.blob_id,
            row.content_key_id,
            row.envelope_schema_version,
            row.algorithm,
            row.nonce,
            row.ciphertext,
            row.content_class,
            row.subject_kind,
            row.subject_id,
            row.size_bytes,
            row.created_at,
        ],
    )?;
    Ok(())
}

/// Fetch an encrypted blob by id. Missing → `Ok(None)`. Returns opaque
/// ciphertext bytes; does not decrypt.
pub fn get_encrypted_blob(conn: &Connection, blob_id: &str) -> Result<Option<EncryptedBlobRow>> {
    let mut stmt = conn.prepare(
        "SELECT blob_id, content_key_id, envelope_schema_version, algorithm,
                nonce, ciphertext, content_class, subject_kind, subject_id,
                size_bytes, created_at
         FROM encrypted_content_blob
         WHERE blob_id = ?",
    )?;
    let row = stmt
        .query_row(params![blob_id], |row| {
            Ok(EncryptedBlobRow {
                blob_id: row.get(0)?,
                content_key_id: row.get(1)?,
                envelope_schema_version: row.get(2)?,
                algorithm: row.get(3)?,
                nonce: row.get(4)?,
                ciphertext: row.get(5)?,
                content_class: row.get(6)?,
                subject_kind: row.get(7)?,
                subject_id: row.get(8)?,
                size_bytes: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// List all blobs for a content key (ordered by `created_at`, then `blob_id`).
pub fn list_blobs_for_content_key(
    conn: &Connection,
    content_key_id: &str,
) -> Result<Vec<EncryptedBlobRow>> {
    let mut stmt = conn.prepare(
        "SELECT blob_id, content_key_id, envelope_schema_version, algorithm,
                nonce, ciphertext, content_class, subject_kind, subject_id,
                size_bytes, created_at
         FROM encrypted_content_blob
         WHERE content_key_id = ?
         ORDER BY created_at ASC, blob_id ASC",
    )?;
    let rows = stmt.query_map(params![content_key_id], |row| {
        Ok(EncryptedBlobRow {
            blob_id: row.get(0)?,
            content_key_id: row.get(1)?,
            envelope_schema_version: row.get(2)?,
            algorithm: row.get(3)?,
            nonce: row.get(4)?,
            ciphertext: row.get(5)?,
            content_class: row.get(6)?,
            subject_kind: row.get(7)?,
            subject_id: row.get(8)?,
            size_bytes: row.get(9)?,
            created_at: row.get(10)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// True iff a content key row exists with `status = 'destroyed'`.
pub fn is_content_key_destroyed(conn: &Connection, content_key_id: &str) -> Result<bool> {
    let destroyed: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM content_key_store
            WHERE content_key_id = ? AND status = 'destroyed'
         )",
        params![content_key_id],
        |row| row.get(0),
    )?;
    Ok(destroyed)
}

/// Fetch tombstone by `content_key_id`. Missing → `Ok(None)`.
pub fn get_tombstone(conn: &Connection, content_key_id: &str) -> Result<Option<TombstoneRow>> {
    let mut stmt = conn.prepare(
        "SELECT tombstone_id, content_key_id, erased_at, reason_code
         FROM tombstone_projection
         WHERE content_key_id = ?",
    )?;
    let row = stmt
        .query_row(params![content_key_id], |row| {
            Ok(TombstoneRow {
                tombstone_id: row.get(0)?,
                content_key_id: row.get(1)?,
                erased_at: row.get(2)?,
                reason_code: row.get(3)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// Fetch tombstone by primary key `tombstone_id`. Missing → `Ok(None)`.
pub fn get_tombstone_by_id(conn: &Connection, tombstone_id: &str) -> Result<Option<TombstoneRow>> {
    let mut stmt = conn.prepare(
        "SELECT tombstone_id, content_key_id, erased_at, reason_code
         FROM tombstone_projection
         WHERE tombstone_id = ?",
    )?;
    let row = stmt
        .query_row(params![tombstone_id], |row| {
            Ok(TombstoneRow {
                tombstone_id: row.get(0)?,
                content_key_id: row.get(1)?,
                erased_at: row.get(2)?,
                reason_code: row.get(3)?,
            })
        })
        .optional()?;
    Ok(row)
}

/// Fetch erasure request by `content_key_id`. Missing → `Ok(None)`.
pub fn get_erasure_request(
    conn: &Connection,
    content_key_id: &str,
) -> Result<Option<ErasureRequestRow>> {
    let mut stmt = conn.prepare(
        "SELECT content_key_id, requester, reason, status,
                requested_at, completed_at, tombstone_id
         FROM erasure_request_projection
         WHERE content_key_id = ?",
    )?;
    let row = stmt
        .query_row(params![content_key_id], |row| {
            Ok(ErasureRequestRow {
                content_key_id: row.get(0)?,
                requester: row.get(1)?,
                reason: row.get(2)?,
                status: row.get(3)?,
                requested_at: row.get(4)?,
                completed_at: row.get(5)?,
                tombstone_id: row.get(6)?,
            })
        })
        .optional()?;
    Ok(row)
}

// ---------------------------------------------------------------------------
// Derived-plaintext purge + WAL (T165 CE)
// ---------------------------------------------------------------------------

/// Counts from scoped FTS / embedding purge for wipe subjects (E13).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeDerivedCounts {
    pub fts_rows: u64,
    pub embeddings: u64,
    pub projection_rows: u64,
}

/// Subject reference from an encrypted blob (kind + id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlobSubject {
    pub kind: String,
    pub id: String,
}

/// Collect unique subjects from all blobs for a content key (E13).
pub fn subjects_for_content_key(
    conn: &Connection,
    content_key_id: &str,
) -> Result<Vec<BlobSubject>> {
    let blobs = list_blobs_for_content_key(conn, content_key_id)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for b in blobs {
        let (Some(kind), Some(id)) = (b.subject_kind, b.subject_id) else {
            continue;
        };
        if kind.trim().is_empty() || id.trim().is_empty() {
            continue;
        }
        let key = (kind.clone(), id.clone());
        if seen.insert(key) {
            out.push(BlobSubject { kind, id });
        }
    }
    Ok(out)
}

/// Purge FTS-backed / projection plaintext for CE-linked subjects.
///
/// Scoped by the provided subject set — never vault-wide.
///
/// - **memory**: clear `memory_projection.content` + embeddings (FTS au trigger).
/// - **evidence**: clear `evidence_projection.summary` (FTS au trigger).
/// - **source**: no source-level FTS/summary table; E15 marks unavailable separately.
///
/// Unknown kinds are ignored (no graph pruning invented here).
pub fn purge_derived_plaintext_for_subjects(
    conn: &Connection,
    subjects: &[BlobSubject],
) -> Result<PurgeDerivedCounts> {
    let mut counts = PurgeDerivedCounts::default();
    for subj in subjects {
        if subj.kind.eq_ignore_ascii_case("memory") {
            purge_memory_subject(conn, subj.id.as_str(), &mut counts)?;
        } else if subj.kind.eq_ignore_ascii_case("evidence") {
            purge_evidence_subject(conn, subj.id.as_str(), &mut counts)?;
        }
        // source: no FTS/summary plaintext index; dependents via E15 only.
    }
    Ok(counts)
}

/// Collect subjects for a content key and purge derived plaintext (E13 / rebuild).
pub fn purge_derived_plaintext_for_content_key(
    conn: &Connection,
    content_key_id: &str,
) -> Result<PurgeDerivedCounts> {
    let subjects = subjects_for_content_key(conn, content_key_id)?;
    purge_derived_plaintext_for_subjects(conn, &subjects)
}

/// Re-apply derived-plaintext purge for every destroyed content key and every
/// tombstone (rebuild durability — side stores retained; event replay can restore
/// projection plaintext before `ContentErased` re-purges).
pub fn reapply_purge_for_erased_content_keys(conn: &Connection) -> Result<()> {
    let mut keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT content_key_id FROM content_key_store WHERE status = 'destroyed'")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for r in rows {
            keys.insert(r?);
        }
    }
    {
        let mut stmt = conn.prepare("SELECT content_key_id FROM tombstone_projection")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for r in rows {
            keys.insert(r?);
        }
    }
    for key in keys {
        let _ = purge_derived_plaintext_for_content_key(conn, &key)?;
    }
    Ok(())
}

fn purge_memory_subject(
    conn: &Connection,
    memory_id: &str,
    counts: &mut PurgeDerivedCounts,
) -> Result<()> {
    // Clear embedding first (count only when embedding was present).
    let emb_changed = conn.execute(
        "UPDATE memory_projection
         SET embedding = NULL,
             embedding_generated_at = NULL
         WHERE memory_id = ?
           AND embedding IS NOT NULL",
        params![memory_id],
    )?;
    counts.embeddings += emb_changed as u64;

    // Clear projection content so FTS au trigger drops searchable plaintext.
    // Status is left intact (not soft-forget); content is CE residual purge.
    let had_plaintext: bool = conn.query_row(
        "SELECT EXISTS(
                SELECT 1 FROM memory_projection
                WHERE memory_id = ?
                  AND content IS NOT NULL
                  AND TRIM(content) != ''
             )",
        params![memory_id],
        |row| row.get(0),
    )?;

    let proj_changed = conn.execute(
        "UPDATE memory_projection
         SET content = ''
         WHERE memory_id = ?
           AND content IS NOT NULL
           AND TRIM(content) != ''",
        params![memory_id],
    )?;
    counts.projection_rows += proj_changed as u64;
    if had_plaintext || proj_changed > 0 {
        counts.fts_rows += 1;
    }

    // Defensive: if FTS still has non-empty content for this memory_id
    // (trigger missing / external content drift), force FTS delete by rowid.
    if memory_fts_has_plaintext(conn, memory_id)? {
        let fts_rows: Vec<(i64, String)> = {
            let mut stmt = conn.prepare(
                "SELECT p.rowid, COALESCE(p.content, '')
                 FROM memory_projection p
                 WHERE p.memory_id = ?",
            )?;
            let mapped = stmt.query_map(params![memory_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut rows = Vec::new();
            for r in mapped {
                rows.push(r?);
            }
            rows
        };
        for (rowid, content) in &fts_rows {
            conn.execute(
                "INSERT INTO memory_fts(memory_fts, rowid, content, memory_id)
                 VALUES('delete', ?, ?, ?)",
                params![rowid, content, memory_id],
            )?;
        }
    }
    Ok(())
}

fn purge_evidence_subject(
    conn: &Connection,
    evidence_id: &str,
    counts: &mut PurgeDerivedCounts,
) -> Result<()> {
    let had_plaintext: bool = conn.query_row(
        "SELECT EXISTS(
                SELECT 1 FROM evidence_projection
                WHERE evidence_id = ?
                  AND summary IS NOT NULL
                  AND TRIM(summary) != ''
             )",
        params![evidence_id],
        |row| row.get(0),
    )?;

    // summary is NOT NULL — empty string clears searchable FTS via au trigger.
    let proj_changed = conn.execute(
        "UPDATE evidence_projection
         SET summary = ''
         WHERE evidence_id = ?
           AND TRIM(summary) != ''",
        params![evidence_id],
    )?;
    counts.projection_rows += proj_changed as u64;
    if had_plaintext || proj_changed > 0 {
        counts.fts_rows += 1;
    }

    if evidence_fts_has_plaintext(conn, evidence_id)? {
        let fts_rows: Vec<(i64, String)> = {
            let mut stmt = conn.prepare(
                "SELECT p.rowid, COALESCE(p.summary, '')
                 FROM evidence_projection p
                 WHERE p.evidence_id = ?",
            )?;
            let mapped = stmt.query_map(params![evidence_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut rows = Vec::new();
            for r in mapped {
                rows.push(r?);
            }
            rows
        };
        for (rowid, summary) in &fts_rows {
            conn.execute(
                "INSERT INTO evidence_fts(evidence_fts, rowid, summary, evidence_id)
                 VALUES('delete', ?, ?, ?)",
                params![rowid, summary, evidence_id],
            )?;
        }
    }
    Ok(())
}

/// True iff FTS still holds non-empty content for the memory (validation layer).
pub fn memory_fts_has_plaintext(conn: &Connection, memory_id: &str) -> Result<bool> {
    // Prefer projection content (source of truth for external-content FTS).
    let proj: Option<String> = conn
        .query_row(
            "SELECT content FROM memory_projection WHERE memory_id = ?",
            params![memory_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(c) = proj
        && !c.trim().is_empty()
    {
        return Ok(true);
    }
    // Also check FTS shadow table for non-empty content rows.
    let fts_nonempty: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_fts
             WHERE memory_id = ? AND content IS NOT NULL AND TRIM(content) != ''",
        params![memory_id],
        |row| row.get(0),
    )?;
    Ok(fts_nonempty > 0)
}

/// True iff evidence projection/FTS still holds non-empty summary plaintext.
pub fn evidence_fts_has_plaintext(conn: &Connection, evidence_id: &str) -> Result<bool> {
    let proj: Option<String> = conn
        .query_row(
            "SELECT summary FROM evidence_projection WHERE evidence_id = ?",
            params![evidence_id],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(c) = proj
        && !c.trim().is_empty()
    {
        return Ok(true);
    }
    let fts_nonempty: i64 = conn.query_row(
        "SELECT COUNT(*) FROM evidence_fts
             WHERE evidence_id = ? AND summary IS NOT NULL AND TRIM(summary) != ''",
        params![evidence_id],
        |row| row.get(0),
    )?;
    Ok(fts_nonempty > 0)
}

/// Back-compat name used by tests / callers: non-empty FTS/projection plaintext.
pub fn memory_fts_has_hits(conn: &Connection, memory_id: &str) -> Result<bool> {
    memory_fts_has_plaintext(conn, memory_id)
}

/// Outcome of post-wipe `PRAGMA wal_checkpoint(TRUNCATE)` (E16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalCheckpointOutcome {
    /// Truncate checkpoint completed (busy == 0).
    Truncated,
    /// BUSY after one retry; routine PASSIVE/autocheckpoint will finish later.
    PendingPassive,
}

/// Run `PRAGMA wal_checkpoint(TRUNCATE)` with one busy retry (E16).
///
/// Handles both forms of busy:
/// 1. Result row `(busy, log, checkpointed)` with `busy != 0`
/// 2. rusqlite [`Error`] when SQLite returns `SQLITE_BUSY` as an error
///
/// One retry; still busy → [`WalCheckpointOutcome::PendingPassive`] (**not** an
/// error). Does not claim media sanitization or free-page zeroization.
pub fn wal_checkpoint_truncate(conn: &Connection) -> Result<WalCheckpointOutcome> {
    match run_wal_checkpoint_truncate(conn) {
        Ok(0) => Ok(WalCheckpointOutcome::Truncated),
        Ok(_) => {
            // busy column != 0: one retry
            match run_wal_checkpoint_truncate(conn) {
                Ok(0) => Ok(WalCheckpointOutcome::Truncated),
                Ok(_) => Ok(WalCheckpointOutcome::PendingPassive),
                Err(e) if is_sqlite_busy_error(&e) => Ok(WalCheckpointOutcome::PendingPassive),
                Err(e) => Err(e),
            }
        }
        Err(e) if is_sqlite_busy_error(&e) => {
            // SQLITE_BUSY as Error (not only busy column): one retry
            match run_wal_checkpoint_truncate(conn) {
                Ok(0) => Ok(WalCheckpointOutcome::Truncated),
                Ok(_) => Ok(WalCheckpointOutcome::PendingPassive),
                Err(e2) if is_sqlite_busy_error(&e2) => Ok(WalCheckpointOutcome::PendingPassive),
                Err(e2) => Err(e2),
            }
        }
        Err(e) => Err(e),
    }
}

/// Returns the `busy` column from `PRAGMA wal_checkpoint(TRUNCATE)`.
fn run_wal_checkpoint_truncate(conn: &Connection) -> Result<i64> {
    conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
        .map_err(StoreError::DatabaseError)
}

/// True when rusqlite reports SQLITE_BUSY / DatabaseBusy.
fn is_sqlite_busy_error(err: &StoreError) -> bool {
    match err {
        StoreError::DatabaseError(rusqlite::Error::SqliteFailure(code, _)) => {
            matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) || code.extended_code == 5 // SQLITE_BUSY
                || code.extended_code == 6 // SQLITE_LOCKED
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

/// Applies `ContentErasureRequested` / `ContentErased` to event projections.
/// Does **not** write side stores.
pub struct ContentEnvelopeProjection;

impl Projection for ContentEnvelopeProjection {
    fn apply(&self, tx: &rusqlite::Transaction, envelope: &Envelope) -> Result<()> {
        let occurred_at = envelope
            .occurred_at
            .format(&Rfc3339)
            .map_err(|e| StoreError::EventReadFailed(e.to_string()))?;

        match &envelope.payload {
            Payload::ContentErasureRequested(p) => {
                // S14: never demote completed → requested.
                // ON CONFLICT DO UPDATE only when status is not completed and
                // tombstone_id is still NULL.
                tx.execute(
                    "INSERT INTO erasure_request_projection (
                        content_key_id, requester, reason, status,
                        requested_at, completed_at, tombstone_id
                     ) VALUES (?, ?, ?, 'requested', ?, NULL, NULL)
                     ON CONFLICT(content_key_id) DO UPDATE SET
                        requester = excluded.requester,
                        reason = excluded.reason,
                        requested_at = excluded.requested_at
                     WHERE erasure_request_projection.status != 'completed'
                       AND erasure_request_projection.tombstone_id IS NULL",
                    params![
                        p.content_key_id.to_string(),
                        p.requester.to_string(),
                        p.reason,
                        occurred_at,
                    ],
                )?;
            }
            Payload::ContentErased(p) => {
                // Complete request (UPSERT) + insert minimal tombstone.
                // Orphan ContentErased (no prior request): insert completed row
                // with empty requester/reason; requested_at = completed_at.
                let key_str = p.content_key_id.to_string();
                tx.execute(
                    "INSERT INTO erasure_request_projection (
                        content_key_id, requester, reason, status,
                        requested_at, completed_at, tombstone_id
                     ) VALUES (?, '', '', 'completed', ?, ?, ?)
                     ON CONFLICT(content_key_id) DO UPDATE SET
                        status = 'completed',
                        completed_at = excluded.completed_at,
                        tombstone_id = excluded.tombstone_id",
                    params![
                        key_str.as_str(),
                        occurred_at,
                        occurred_at,
                        p.tombstone_id.to_string(),
                    ],
                )?;
                tx.execute(
                    "INSERT INTO tombstone_projection (
                        tombstone_id, content_key_id, erased_at, reason_code
                     ) VALUES (?, ?, ?, '')
                     ON CONFLICT(tombstone_id) DO UPDATE SET
                        content_key_id = excluded.content_key_id,
                        erased_at = excluded.erased_at,
                        reason_code = excluded.reason_code",
                    params![p.tombstone_id.to_string(), key_str.as_str(), occurred_at,],
                )?;
                // Rebuild durability (T165 Codex R1): re-purge derived plaintext for
                // subjects linked via retained encrypted_content_blob side store.
                // Event replay restores memory/evidence projection text; CE must
                // clear it again when ContentErased is applied.
                let _ = purge_derived_plaintext_for_content_key(tx, &key_str)?;
            }
            // Ticket path and soft forget must not write CE tables (S7/S8).
            _ => {}
        }
        Ok(())
    }
}
