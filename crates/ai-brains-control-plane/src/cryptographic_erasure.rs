//! Governed cryptographic erasure (CE) wipe for envelope-backed content (T165 / P8.3).
//!
//! Order (spec §6 / E6):
//! policy → ContentErasureRequested → destroy wrap → purge FTS/embeddings →
//! dependents (SourceId only) → ContentErased + tombstone → (caller commit) →
//! WAL TRUNCATE → verify wrap_absent.
//!
//! **E2:** Never emit `ContentErased` unless `destroy_content_key_wrap` succeeded
//! and verification (`wrap_absent`) passes.
//! **E1:** Refuse when no `content_key_store` row exists.
//! **E9:** Dry-run default-safe; execute needs `confirm && !dry_run`.

use ai_brains_contracts::erasure::{
    ContentEnvelopeWipedResponse, WIPE_HONESTY_ENVELOPE_ONLY, WIPE_HONESTY_NOT_NIST_PURGE,
    WIPE_HONESTY_PRE_ERASE_BACKUP, WIPE_HONESTY_TICKET_NOT_CE, WIPE_WARNING_DEPENDENTS_SKIPPED,
    WIPE_WARNING_WAL_PENDING_PASSIVE, WipePurgedCounts, WipeValidation, WipeVerify,
};
use ai_brains_core::ids::{ContentKeyId, PrincipalId, SourceId, TombstoneId};
use ai_brains_core::principal::Principal;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::ReviewCriticality;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_events::payload::{ContentErasedPayload, ContentErasureRequestedPayload};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::projections::content_envelope::{
    self, BlobSubject, PurgeDerivedCounts, WalCheckpointOutcome,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::command_id::id_from_command;
use crate::errors::{ControlPlaneError, Result};
use crate::invalidation::{SourceUnavailableRequest, mark_source_unavailable};
use crate::ports::{Clock, EventWriter, GovernedQueryStore, PolicyContext, PolicyEvaluator};
use crate::sources::build_event;

/// Re-export NS for wipe tombstone derivation (canonical in `command_id`).
pub use crate::command_id::NS_WIPE_CONTENT_ENVELOPE;

/// Subject kind product freezes as source-like for E15 dependent invalidation.
pub const SUBJECT_KIND_SOURCE: &str = "source";

// ---------------------------------------------------------------------------
// Ports (fault-injectable store side)
// ---------------------------------------------------------------------------

/// Side-store + purge access for CE wipe (testable without full vault mock).
pub trait ContentEnvelopeWipeStore {
    /// `Ok(None)` when no `content_key_store` row (E1).
    fn get_wrap_status(&self, content_key_id: &str) -> Result<Option<ContentKeyStatus>>;

    fn destroy_content_key_wrap(&self, content_key_id: &str, destroyed_at: &str) -> Result<()>;

    fn list_blob_subjects(&self, content_key_id: &str) -> Result<Vec<BlobSubject>>;

    fn blob_count(&self, content_key_id: &str) -> Result<u64>;

    fn get_tombstone_id(&self, content_key_id: &str) -> Result<Option<String>>;

    fn purge_derived_plaintext(&self, subjects: &[BlobSubject]) -> Result<PurgeDerivedCounts>;

    /// Fresh re-query: wrap material absent / status destroyed (E14 verification).
    fn is_wrap_absent(&self, content_key_id: &str) -> Result<bool>;

    /// Store cannot supply wrap for open (destroyed or missing material) — not AEAD.
    fn store_open_refused(&self, content_key_id: &str) -> Result<bool>;

    /// Best-effort: no FTS hits remain for memory subjects after purge.
    fn fts_clear_for_subjects(&self, subjects: &[BlobSubject]) -> Result<bool>;

    fn wal_checkpoint_truncate(&self) -> Result<WalCheckpointOutcome>;
}

/// Snapshot of content key wrap state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentKeyStatus {
    pub status: String,
    pub wrap_material_present: bool,
}

/// Production adapter over [`SqliteEventStore`].
pub struct StoreContentEnvelopeWipe {
    store: SqliteEventStore,
}

impl StoreContentEnvelopeWipe {
    pub fn new(store: SqliteEventStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SqliteEventStore {
        &self.store
    }
}

impl ContentEnvelopeWipeStore for StoreContentEnvelopeWipe {
    fn get_wrap_status(&self, content_key_id: &str) -> Result<Option<ContentKeyStatus>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let row = content_envelope::get_content_key_wrap(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(row.map(|r| ContentKeyStatus {
            wrap_material_present: r.wrap_nonce.is_some() && r.wrap_ciphertext.is_some(),
            status: r.status,
        }))
    }

    fn destroy_content_key_wrap(&self, content_key_id: &str, destroyed_at: &str) -> Result<()> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        content_envelope::destroy_content_key_wrap(&conn, content_key_id, destroyed_at)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))
    }

    fn list_blob_subjects(&self, content_key_id: &str) -> Result<Vec<BlobSubject>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        content_envelope::subjects_for_content_key(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))
    }

    fn blob_count(&self, content_key_id: &str) -> Result<u64> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let blobs = content_envelope::list_blobs_for_content_key(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(blobs.len() as u64)
    }

    fn get_tombstone_id(&self, content_key_id: &str) -> Result<Option<String>> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let ts = content_envelope::get_tombstone(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        Ok(ts.map(|t| t.tombstone_id))
    }

    fn purge_derived_plaintext(&self, subjects: &[BlobSubject]) -> Result<PurgeDerivedCounts> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        content_envelope::purge_derived_plaintext_for_subjects(&conn, subjects)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))
    }

    fn is_wrap_absent(&self, content_key_id: &str) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        // Verification (E14): status destroyed (or missing material).
        let row = content_envelope::get_content_key_wrap(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        match row {
            None => Ok(false), // key gone entirely is unexpected after destroy path
            Some(r) => {
                let destroyed = r.status == "destroyed"
                    || content_envelope::is_content_key_destroyed(&conn, content_key_id)
                        .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
                let no_material = r.wrap_nonce.is_none() && r.wrap_ciphertext.is_none();
                Ok(destroyed && no_material)
            }
        }
    }

    fn store_open_refused(&self, content_key_id: &str) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        let row = content_envelope::get_content_key_wrap(&conn, content_key_id)
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        match row {
            None => Ok(true),
            Some(r) => {
                let active_with_material = r.status == "active"
                    && r.wrap_nonce.as_ref().is_some_and(|n| !n.is_empty())
                    && r.wrap_ciphertext.as_ref().is_some_and(|c| !c.is_empty());
                Ok(!active_with_material)
            }
        }
    }

    fn fts_clear_for_subjects(&self, subjects: &[BlobSubject]) -> Result<bool> {
        let conn = self
            .store
            .connection()
            .lock()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))?;
        for s in subjects {
            if s.kind.eq_ignore_ascii_case("memory") {
                if content_envelope::memory_fts_has_plaintext(&conn, &s.id)
                    .map_err(|e| ControlPlaneError::Query(e.to_string()))?
                {
                    return Ok(false);
                }
            } else if s.kind.eq_ignore_ascii_case("evidence")
                && content_envelope::evidence_fts_has_plaintext(&conn, &s.id)
                    .map_err(|e| ControlPlaneError::Query(e.to_string()))?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn wal_checkpoint_truncate(&self) -> Result<WalCheckpointOutcome> {
        self.store
            .connection()
            .wal_checkpoint_truncate()
            .map_err(|e| ControlPlaneError::Query(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Request / orchestrator
// ---------------------------------------------------------------------------

/// Inputs for [`wipe_content_envelope`].
#[derive(Debug, Clone)]
pub struct WipeContentEnvelopeCommand {
    pub principal: Principal,
    pub content_key_id: ContentKeyId,
    pub scope: ScopeRef,
    pub reason: Option<String>,
    /// Pre-assigned tombstone when `command_id` present (daemon derives via NS).
    pub tombstone_id: Option<TombstoneId>,
    pub dry_run: bool,
    pub confirm: bool,
}

/// Derive deterministic tombstone id from command_id (uuid v5).
pub fn tombstone_id_from_command(command_id: &str) -> TombstoneId {
    TombstoneId::from_uuid(id_from_command(NS_WIPE_CONTENT_ENVELOPE, command_id))
}

/// Governed CE wipe orchestrator.
pub fn wipe_content_envelope<W, Q, C, P, S>(
    writer: &W,
    query: &Q,
    clock: &C,
    policy: &P,
    side: &S,
    cmd: WipeContentEnvelopeCommand,
) -> Result<ContentEnvelopeWipedResponse>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
    P: PolicyEvaluator,
    S: ContentEnvelopeWipeStore,
{
    let key_str = cmd.content_key_id.to_string();

    // 0. Payload gates
    if !cmd.dry_run && !cmd.confirm {
        return Err(ControlPlaneError::InvalidPayload(
            "execute wipe requires confirm=true and dry_run=false".into(),
        ));
    }

    // 1. Policy GrantCapability::Erase (always before work / already-done)
    let policy_ctx = PolicyContext::default_for_privacy(Privacy::LocalOnly);
    if !policy.allow(
        cmd.principal.id,
        GrantCapability::Erase,
        &cmd.scope,
        &policy_ctx,
    )? {
        return Err(ControlPlaneError::PolicyDenied("Erase denied".into()));
    }

    // 2. Resolve key — E1
    let status = side.get_wrap_status(&key_str)?;
    let Some(key_status) = status else {
        return Err(ControlPlaneError::NotEnvelopeBacked(format!(
            "no content_key_store row for {key_str}; CE applies only to envelope-backed content (use soft forget for legacy)"
        )));
    };

    let subjects = side.list_blob_subjects(&key_str)?;
    let blobs_considered = side.blob_count(&key_str)?;
    let tombstone_existing = side.get_tombstone_id(&key_str)?;
    let wrap_already_destroyed =
        key_status.status == "destroyed" || !key_status.wrap_material_present;

    // 3. Already completed (destroyed + tombstone) → idempotent already_erased (E7/S14)
    if wrap_already_destroyed && tombstone_existing.is_some() {
        let wrap_absent = side.is_wrap_absent(&key_str)?;
        if !wrap_absent {
            return Err(ControlPlaneError::Query(
                "tombstone present but wrap verification failed".into(),
            ));
        }
        let fts_clear = side.fts_clear_for_subjects(&subjects)?;
        let store_open_refused = side.store_open_refused(&key_str)?;
        return Ok(ContentEnvelopeWipedResponse {
            api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
            status: "already_erased".into(),
            content_key_id: key_str,
            tombstone_id: tombstone_existing,
            wrap_destroyed: true,
            blobs_considered,
            purged: WipePurgedCounts::default(),
            dependents_marked: 0,
            warnings: honesty_warnings(),
            verify: WipeVerify { wrap_absent: true },
            validation: WipeValidation {
                fts_clear,
                store_open_refused,
                wal_checkpoint: "skipped_already_erased".into(),
            },
        });
    }

    // 4. Dry-run: plan only
    if cmd.dry_run {
        let mut warnings = honesty_warnings();
        if !has_source_linked_subject(query, &subjects)? {
            warnings.push(WIPE_WARNING_DEPENDENTS_SKIPPED.to_string());
        }
        let wrap_absent = wrap_already_destroyed && side.is_wrap_absent(&key_str)?;
        let store_open_refused = side.store_open_refused(&key_str)?;
        return Ok(ContentEnvelopeWipedResponse {
            api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
            status: "dry_run".into(),
            content_key_id: key_str,
            tombstone_id: None,
            wrap_destroyed: false,
            blobs_considered,
            purged: WipePurgedCounts::default(),
            dependents_marked: 0,
            warnings,
            verify: WipeVerify { wrap_absent },
            validation: WipeValidation {
                fts_clear: false,
                store_open_refused,
                wal_checkpoint: "skipped_dry_run".into(),
            },
        });
    }

    // --- Execute path (confirm + !dry_run) ---

    let now = clock.now()?;
    let now_rfc3339 = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| ControlPlaneError::Clock(e.to_string()))?;
    let reason = cmd
        .reason
        .clone()
        .filter(|r| !r.trim().is_empty())
        .unwrap_or_else(|| "content_envelope_wipe".to_string());
    let tombstone_id = cmd.tombstone_id.unwrap_or_default();

    // 5. Append ContentErasureRequested (audit intent even if destroy fails)
    let requested = build_event(
        AggregateType::System,
        cmd.content_key_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::ContentErasureRequested(ContentErasureRequestedPayload {
            content_key_id: cmd.content_key_id,
            requester: cmd.principal.id,
            reason: reason.clone(),
        }),
    )?;
    writer.append_events(&[requested])?;

    // 6. Destroy wrap (E2 gate). Hard fail → no ContentErased. Retriable Query.
    side.destroy_content_key_wrap(&key_str, &now_rfc3339)?;

    // 7. Verification before Erased (E2/E14)
    let wrap_absent = side.is_wrap_absent(&key_str)?;
    if !wrap_absent {
        return Err(ControlPlaneError::Query(format!(
            "destroy_content_key_wrap did not leave wrap absent for {key_str}; ContentErased not emitted"
        )));
    }

    // 8. Purge FTS / embeddings for all subjects (E13)
    let purge = side.purge_derived_plaintext(&subjects)?;

    // 9. Dependents — SourceId-gated only (E15)
    let (dependents_marked, mut warnings) =
        mark_dependents_for_subjects(writer, query, clock, &subjects, cmd.principal.id, &reason)?;
    warnings.splice(0..0, honesty_warnings());

    // 10. Append ContentErased + tombstone (only after destroy + verify)
    let erased = build_event(
        AggregateType::System,
        cmd.content_key_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
        Payload::ContentErased(ContentErasedPayload {
            content_key_id: cmd.content_key_id,
            tombstone_id,
        }),
    )?;
    writer.append_events(&[erased])?;

    // 11. Post-commit WAL TRUNCATE (E16) — best-effort; wipe success if E2 met
    let wal = match side.wal_checkpoint_truncate() {
        Ok(WalCheckpointOutcome::Truncated) => "truncated".to_string(),
        Ok(WalCheckpointOutcome::PendingPassive) => {
            warnings.push(WIPE_WARNING_WAL_PENDING_PASSIVE.to_string());
            "pending_passive".to_string()
        }
        Err(e) => {
            // Non-fatal for wipe claim; surface pending_passive honesty.
            warnings.push(format!(
                "{WIPE_WARNING_WAL_PENDING_PASSIVE} (checkpoint error: {e})"
            ));
            "pending_passive".to_string()
        }
    };

    let fts_clear = side.fts_clear_for_subjects(&subjects)?;
    let store_open_refused = side.store_open_refused(&key_str)?;
    // Re-verify after Erased
    let wrap_absent_final = side.is_wrap_absent(&key_str)?;
    if !wrap_absent_final {
        return Err(ControlPlaneError::Query(
            "post-erase verification failed: wrap still present".into(),
        ));
    }

    Ok(ContentEnvelopeWipedResponse {
        api_version: ai_brains_contracts::erasure::API_VERSION.to_string(),
        status: "wiped".into(),
        content_key_id: key_str,
        tombstone_id: Some(tombstone_id.to_string()),
        wrap_destroyed: true,
        blobs_considered,
        purged: WipePurgedCounts {
            fts_rows: purge.fts_rows,
            embeddings: purge.embeddings,
            projection_rows: purge.projection_rows,
        },
        dependents_marked,
        warnings,
        verify: WipeVerify {
            wrap_absent: wrap_absent_final,
        },
        validation: WipeValidation {
            fts_clear,
            store_open_refused,
            wal_checkpoint: wal,
        },
    })
}

fn honesty_warnings() -> Vec<String> {
    vec![
        WIPE_HONESTY_NOT_NIST_PURGE.to_string(),
        WIPE_HONESTY_PRE_ERASE_BACKUP.to_string(),
        WIPE_HONESTY_TICKET_NOT_CE.to_string(),
        WIPE_HONESTY_ENVELOPE_ONLY.to_string(),
    ]
}

fn has_source_linked_subject<Q>(query: &Q, subjects: &[BlobSubject]) -> Result<bool>
where
    Q: GovernedQueryStore,
{
    for s in subjects {
        if !s.kind.eq_ignore_ascii_case(SUBJECT_KIND_SOURCE) {
            continue;
        }
        if let Ok(sid) = SourceId::from_str(&s.id)
            && query.get_source(sid)?.is_some()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Mark dependents stale only when subject_kind is source-like and SourceId is registered (E15).
fn mark_dependents_for_subjects<W, Q, C>(
    writer: &W,
    query: &Q,
    clock: &C,
    subjects: &[BlobSubject],
    opened_by: PrincipalId,
    reason: &str,
) -> Result<(u64, Vec<String>)>
where
    W: EventWriter,
    Q: GovernedQueryStore,
    C: Clock,
{
    let mut marked: u64 = 0;
    let mut warnings = Vec::new();
    let mut any_source_attempt = false;
    let mut any_source_linked = false;

    for s in subjects {
        if !s.kind.eq_ignore_ascii_case(SUBJECT_KIND_SOURCE) {
            continue;
        }
        any_source_attempt = true;
        let Ok(source_id) = SourceId::from_str(&s.id) else {
            continue;
        };
        if query.get_source(source_id)?.is_none() {
            continue;
        }
        any_source_linked = true;
        let inv = mark_source_unavailable(
            writer,
            query,
            clock,
            SourceUnavailableRequest {
                source_id,
                reason: format!("content_erased: {reason}"),
                opened_by,
                privacy: Privacy::LocalOnly,
                criticality: ReviewCriticality::High,
            },
        )?;
        marked += inv.stale_conclusions.len() as u64;
        marked += inv.review_items_for_decisions.len() as u64;
    }

    if !any_source_linked {
        // Memory-only, empty, or unresolvable subjects → skip + warn (E15).
        let _ = any_source_attempt;
        warnings.push(WIPE_WARNING_DEPENDENTS_SKIPPED.to_string());
    }

    Ok((marked, warnings))
}

/// Helper: parse content_key_id string into [`ContentKeyId`].
pub fn parse_content_key_id(raw: &str) -> Result<ContentKeyId> {
    let u = Uuid::parse_str(raw.trim())
        .map_err(|_| ControlPlaneError::InvalidPayload(format!("invalid content_key_id: {raw}")))?;
    Ok(ContentKeyId::from_uuid(u))
}
