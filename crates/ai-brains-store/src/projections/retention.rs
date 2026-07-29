//! Class-based retention scan + projection dispose helpers (T166 / P8.4).
//!
//! Stream **A** = projection identities; stream **B** = content_key_id envelopes.
//! No second CE path here — CE is control-plane `wipe_content_envelope` only (R2).
//! Projection DELETE is never CE (R3). Event log is never rewritten (R10).
//!
//! **R14:** decision revoke/supersede cooldown and review closed age use
//! `updated_at` while those states remain **terminal** (no later projection writes).
//! Do not invent dedicated `revoked_at` / `closed_at` columns in v1.

use crate::errors::Result;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Row types (ids only — no plaintext bodies)
// ---------------------------------------------------------------------------

/// Turn projection identity (stream A).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TurnKey {
    pub session_id: String,
    pub turn_index: i64,
}

impl TurnKey {
    pub fn identity(&self) -> String {
        format!("{}:{}", self.session_id, self.turn_index)
    }
}

/// Active content key with optional blob class labels (stream B).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeKeyScan {
    pub content_key_id: String,
    pub status: String,
    pub created_at: String,
    /// Distinct non-null content_class values on blobs under this key.
    pub content_classes: Vec<String>,
    pub blob_count: u64,
    /// Memory subject ids linked by blobs (for pin holds / cascade).
    pub memory_subject_ids: Vec<String>,
    /// Turn subject join keys (`subject_kind=turn`, subject_id) when present.
    pub turn_subject_ids: Vec<String>,
    /// Oldest blob created_at when blobs exist; else key created_at.
    pub age_anchor: String,
}

/// Query trace past horizon.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryTraceKey {
    pub trace_id: String,
    pub recorded_at: String,
}

/// Closed review item past horizon.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewTraceKey {
    pub review_item_id: String,
    pub status: String,
    pub updated_at: String,
}

/// Terminal decision eligible for dispose after cooldown (R6/R14).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionDisposeKey {
    pub decision_id: String,
    pub state: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Stream A scans
// ---------------------------------------------------------------------------

/// Turns whose `last_accessed_at` (fallback `occurred_at`) is strictly before cutoff.
pub fn list_old_turns(conn: &Connection, cutoff_rfc3339: &str) -> Result<Vec<TurnKey>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, turn_index
         FROM turn_projection
         WHERE COALESCE(last_accessed_at, occurred_at) < ?
         ORDER BY session_id ASC, turn_index ASC",
    )?;
    let rows = stmt.query_map(params![cutoff_rfc3339], |row| {
        Ok(TurnKey {
            session_id: row.get(0)?,
            turn_index: row.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Delete specific turn projection rows by identity. Returns rows deleted.
pub fn delete_turns(conn: &Connection, keys: &[TurnKey]) -> Result<usize> {
    let mut total = 0usize;
    for k in keys {
        let n = conn.execute(
            "DELETE FROM turn_projection WHERE session_id = ? AND turn_index = ?",
            params![k.session_id, k.turn_index],
        )?;
        total = total.saturating_add(n);
    }
    Ok(total)
}

/// Query traces with `recorded_at` strictly before cutoff.
pub fn list_old_query_traces(
    conn: &Connection,
    cutoff_rfc3339: &str,
) -> Result<Vec<QueryTraceKey>> {
    let mut stmt = conn.prepare(
        "SELECT trace_id, recorded_at
         FROM query_trace_projection
         WHERE recorded_at < ?
         ORDER BY trace_id ASC",
    )?;
    let rows = stmt.query_map(params![cutoff_rfc3339], |row| {
        Ok(QueryTraceKey {
            trace_id: row.get(0)?,
            recorded_at: row.get(1)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn delete_query_traces(conn: &Connection, ids: &[String]) -> Result<usize> {
    let mut total = 0usize;
    for id in ids {
        // Feedback rows reference query_trace_projection.
        conn.execute(
            "DELETE FROM retrieval_feedback_projection WHERE trace_id = ?",
            params![id],
        )?;
        let n = conn.execute(
            "DELETE FROM query_trace_projection WHERE trace_id = ?",
            params![id],
        )?;
        total = total.saturating_add(n);
    }
    Ok(total)
}

/// Closed (Resolved) review items with terminal `updated_at` before cutoff (R14).
pub fn list_old_closed_reviews(
    conn: &Connection,
    cutoff_rfc3339: &str,
) -> Result<Vec<ReviewTraceKey>> {
    let mut stmt = conn.prepare(
        "SELECT review_item_id, status, updated_at
         FROM review_item_projection
         WHERE status = 'Resolved' AND updated_at < ?
         ORDER BY review_item_id ASC",
    )?;
    let rows = stmt.query_map(params![cutoff_rfc3339], |row| {
        Ok(ReviewTraceKey {
            review_item_id: row.get(0)?,
            status: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn delete_review_items(conn: &Connection, ids: &[String]) -> Result<usize> {
    let mut total = 0usize;
    for id in ids {
        let n = conn.execute(
            "DELETE FROM review_item_projection WHERE review_item_id = ?",
            params![id],
        )?;
        total = total.saturating_add(n);
    }
    Ok(total)
}

/// Decisions in terminal Revoked/Superseded with `updated_at` before cooldown cutoff (R14).
///
/// Active Approved decisions are **not** returned (R6 — no age auto-wipe).
pub fn list_disposable_decisions(
    conn: &Connection,
    cooldown_cutoff_rfc3339: &str,
) -> Result<Vec<DecisionDisposeKey>> {
    let mut stmt = conn.prepare(
        "SELECT decision_id, state, updated_at
         FROM decision_projection
         WHERE state IN ('Revoked', 'Superseded') AND updated_at < ?
         ORDER BY decision_id ASC",
    )?;
    let rows = stmt.query_map(params![cooldown_cutoff_rfc3339], |row| {
        Ok(DecisionDisposeKey {
            decision_id: row.get(0)?,
            state: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Count of active Approved decisions (for plan skip visibility tests).
pub fn count_approved_active_decisions(conn: &Connection) -> Result<u64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM decision_projection WHERE state = 'Approved'",
        [],
        |r| r.get(0),
    )?;
    Ok(n as u64)
}

pub fn delete_decisions(conn: &Connection, ids: &[String]) -> Result<usize> {
    let mut total = 0usize;
    for id in ids {
        conn.execute(
            "DELETE FROM decision_support_projection WHERE decision_id = ?",
            params![id],
        )?;
        let n = conn.execute(
            "DELETE FROM decision_projection WHERE decision_id = ?",
            params![id],
        )?;
        total = total.saturating_add(n);
    }
    Ok(total)
}

/// Memory ids with status `pinned` (R11 holds).
pub fn list_pinned_memory_ids(conn: &Connection) -> Result<BTreeSet<String>> {
    let mut stmt = conn.prepare(
        "SELECT memory_id FROM memory_projection WHERE status = 'pinned' ORDER BY memory_id",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut out = BTreeSet::new();
    for r in rows {
        out.insert(r?);
    }
    Ok(out)
}

/// Memory status if present.
pub fn memory_status(conn: &Connection, memory_id: &str) -> Result<Option<String>> {
    let s: Option<String> = conn
        .query_row(
            "SELECT status FROM memory_projection WHERE memory_id = ?",
            params![memory_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(s)
}

// ---------------------------------------------------------------------------
// Stream B scans
// ---------------------------------------------------------------------------

/// Scan active (or all non-destroyed) content keys with blob class labels.
pub fn list_envelope_keys(conn: &Connection) -> Result<Vec<EnvelopeKeyScan>> {
    let mut stmt = conn.prepare(
        "SELECT content_key_id, status, created_at
         FROM content_key_store
         WHERE status = 'active'
         ORDER BY content_key_id ASC",
    )?;
    let key_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for kr in key_rows {
        let (content_key_id, status, created_at) = kr?;
        let blob_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM encrypted_content_blob WHERE content_key_id = ?",
            params![&content_key_id],
            |r| r.get(0),
        )?;
        let mut class_stmt = conn.prepare(
            "SELECT DISTINCT content_class FROM encrypted_content_blob
             WHERE content_key_id = ? AND content_class IS NOT NULL
             ORDER BY content_class ASC",
        )?;
        let class_rows =
            class_stmt.query_map(params![&content_key_id], |row| row.get::<_, String>(0))?;
        let mut content_classes = Vec::new();
        for c in class_rows {
            content_classes.push(c?);
        }
        let mut mem_stmt = conn.prepare(
            "SELECT DISTINCT subject_id FROM encrypted_content_blob
             WHERE content_key_id = ?
               AND subject_kind IS NOT NULL
               AND lower(subject_kind) = 'memory'
               AND subject_id IS NOT NULL
             ORDER BY subject_id ASC",
        )?;
        let mem_rows =
            mem_stmt.query_map(params![&content_key_id], |row| row.get::<_, String>(0))?;
        let mut memory_subject_ids = Vec::new();
        for m in mem_rows {
            memory_subject_ids.push(m?);
        }
        let mut turn_stmt = conn.prepare(
            "SELECT DISTINCT subject_id FROM encrypted_content_blob
             WHERE content_key_id = ?
               AND subject_kind IS NOT NULL
               AND lower(subject_kind) = 'turn'
               AND subject_id IS NOT NULL
             ORDER BY subject_id ASC",
        )?;
        let turn_rows =
            turn_stmt.query_map(params![&content_key_id], |row| row.get::<_, String>(0))?;
        let mut turn_subject_ids = Vec::new();
        for t in turn_rows {
            turn_subject_ids.push(t?);
        }
        let age_anchor: String = if blob_count > 0 {
            conn.query_row(
                "SELECT MIN(created_at) FROM encrypted_content_blob WHERE content_key_id = ?",
                params![&content_key_id],
                |r| r.get(0),
            )?
        } else {
            created_at.clone()
        };
        out.push(EnvelopeKeyScan {
            content_key_id,
            status,
            created_at,
            content_classes,
            blob_count: blob_count as u64,
            memory_subject_ids,
            turn_subject_ids,
            age_anchor,
        });
    }
    Ok(out)
}

/// Active wraps with zero blobs and `created_at` before orphan cutoff (R16, 7d default).
pub fn list_orphaned_envelopes(
    conn: &Connection,
    orphan_cutoff_rfc3339: &str,
) -> Result<Vec<EnvelopeKeyScan>> {
    let all = list_envelope_keys(conn)?;
    Ok(all
        .into_iter()
        .filter(|k| k.blob_count == 0 && k.created_at.as_str() < orphan_cutoff_rfc3339)
        .collect())
}

// ---------------------------------------------------------------------------
// Hierarchy cascade (R15)
// ---------------------------------------------------------------------------

/// Parent memory ids that list `child_memory_id` as a child in `memory_hierarchy`.
pub fn parents_of_child(conn: &Connection, child_memory_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT parent_memory_id FROM memory_hierarchy
         WHERE child_memory_id = ?
         ORDER BY parent_memory_id ASC",
    )?;
    let rows = stmt.query_map(params![child_memory_id], |row| row.get::<_, String>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Mark parent memories for resynthesis by setting `status = 'stale'` when currently
/// `pinned` or `active` (existing free-form status pattern — R15).
///
/// Returns number of parent rows updated.
pub fn mark_parents_for_resynthesis(
    conn: &Connection,
    child_memory_ids: &[String],
    updated_at: &str,
) -> Result<u64> {
    let mut parents: BTreeSet<String> = BTreeSet::new();
    for child in child_memory_ids {
        for p in parents_of_child(conn, child)? {
            parents.insert(p);
        }
    }
    let mut marked = 0u64;
    for parent in parents {
        let n = conn.execute(
            "UPDATE memory_projection
             SET status = 'stale', updated_at = ?
             WHERE memory_id = ?
               AND status IN ('pinned', 'active')",
            params![updated_at, parent],
        )?;
        marked = marked.saturating_add(n as u64);
    }
    Ok(marked)
}

/// Count hierarchy parents that would be marked (dry-run estimate).
pub fn count_parents_for_resynthesis(
    conn: &Connection,
    child_memory_ids: &[String],
) -> Result<u64> {
    let mut parents: BTreeSet<String> = BTreeSet::new();
    for child in child_memory_ids {
        for p in parents_of_child(conn, child)? {
            parents.insert(p);
        }
    }
    let mut count = 0u64;
    for parent in parents {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM memory_projection
                WHERE memory_id = ? AND status IN ('pinned', 'active')
             )",
            params![parent],
            |r| r.get(0),
        )?;
        if exists {
            count = count.saturating_add(1);
        }
    }
    Ok(count)
}

/// Group helper used by plan aggregation tests.
pub fn group_count_by_class(classes: &[String]) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for c in classes {
        *m.entry(c.clone()).or_insert(0) += 1;
    }
    m
}
