//! Client-side multi-device replicate engine (T177 / ADR-0018).
//!
//! Push/pull sealed envelopes through a [`RelayPort`], apply with L8 pre-verify,
//! gap buffer + drain, and ACK cycle tick. **No sockets** — fake relay only.

use crate::errors::StoreError;
use crate::projections::content_envelope;
use crate::projections::replication::{
    self, DeviceIdentityRow, EnvelopeIndexRow, ErasureAckRow, OutboxRow, PeerContentKeyWrapRow,
    ReplicationCursorRow, SignedControlRow,
};
use ai_brains_core::ids::{ContentKeyId, DeviceId, ReplicationEventId};
use ai_brains_crypto::DataKey;
use ai_brains_crypto::content_envelope::{
    ENVELOPE_SCHEMA_VERSION, SealAad, SealedContent, open, seal,
};
use ai_brains_crypto::content_key_store::ContentDek;
use ai_brains_sync::{
    ContentErasureTombstonePayload, ContentTypeCode, ControlPayload, DeviceRevokedPayload,
    EnvelopeId, ErasureAckPayload, OuterEnvelope, PeerDekWrap, REPLICATION_SCHEMA_VERSION,
    RelayBlob, RelayPort, SealedDevicePrivate, SignedEnvelope, SyncError, WrapRecord,
    build_and_sign_control, decode_control_payload, decode_data_body, decode_signed_envelope,
    encode_data_body, encode_signed_envelope, enrollment_package, fingerprint_sha256,
    nil_content_key_id, open_device_private_blob, sign_envelope, unwrap_content_dek,
    verify_envelope, wrap_content_dek_for_recipient,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rusqlite::Connection;
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

/// Default pull batch size per peer.
const DEFAULT_PULL_LIMIT: usize = 256;

// ---------------------------------------------------------------------------
// Errors / outcomes
// ---------------------------------------------------------------------------

/// Engine-facing errors (map store + sync cleanly; no panic paths).
#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Sync(#[from] SyncError),

    #[error(transparent)]
    Store(#[from] StoreError),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("relay blob routing fields mismatch decoded envelope")]
    RoutingMismatch,

    #[error("local device private key wrap missing for {0}")]
    LocalKeyMissing(String),

    #[error("no wrap for local device in data envelope")]
    MissingSelfWrap,

    #[error("cursor blocked for peer {peer} (seq {seq})")]
    CursorBlocked { peer: String, seq: u64 },

    #[error("invalid device public key material: {0}")]
    InvalidKeyMaterial(String),

    #[error("timestamp format failed: {0}")]
    Timestamp(String),
}

pub type EngineResult<T> = std::result::Result<T, EngineError>;

/// Result of applying one relay blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Projected and advanced cursor.
    Applied,
    /// Same event_id already present (idempotent).
    Idempotent,
    /// Out-of-order; buffered metadata; no project.
    Buffered { local_seq: u64 },
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Client sync session: pending push queue + apply pipeline over a shared relay.
pub struct ReplicateEngine<'a, R: RelayPort> {
    conn: &'a Connection,
    relay: Arc<R>,
    data_key: DataKey,
    local_device_id: DeviceId,
    /// In-memory mirror of local envelopes awaiting `push_pending` (same session).
    /// Durable source of truth is `replication_outbox` (survives process restart).
    pending: VecDeque<SignedEnvelope>,
    /// Max envelopes pulled per peer per `pull_all_peers` call (default 256).
    pull_limit: usize,
}

impl<'a, R: RelayPort> ReplicateEngine<'a, R> {
    pub fn new(
        conn: &'a Connection,
        relay: Arc<R>,
        data_key: DataKey,
        local_device_id: DeviceId,
    ) -> Self {
        Self {
            conn,
            relay,
            data_key,
            local_device_id,
            pending: VecDeque::new(),
            pull_limit: DEFAULT_PULL_LIMIT,
        }
    }

    /// Builder-style pull batch size override (e.g. limit=1 cursor-resume tests).
    pub fn with_pull_limit(mut self, limit: usize) -> Self {
        self.pull_limit = limit.max(1);
        self
    }

    /// Set pull batch size (minimum 1).
    pub fn set_pull_limit(&mut self, limit: usize) {
        self.pull_limit = limit.max(1);
    }

    pub fn pull_limit(&self) -> usize {
        self.pull_limit
    }

    pub fn local_device_id(&self) -> DeviceId {
        self.local_device_id
    }

    pub fn relay(&self) -> &Arc<R> {
        &self.relay
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Queue a pre-signed envelope for the next `push_pending` (durable outbox).
    pub fn queue_signed_envelope(&mut self, signed: SignedEnvelope) -> EngineResult<()> {
        self.persist_outbox(&signed)?;
        self.pending.push_back(signed);
        Ok(())
    }

    /// Seal plaintext data for recipients, sign, and queue for push.
    ///
    /// Generates a fresh ContentDek, wraps for each recipient (sorted), AEAD-seals
    /// the body, and signs the outer envelope with the local device key.
    pub fn seal_and_queue_data(
        &mut self,
        plaintext: &[u8],
        content_key_id: ContentKeyId,
        recipient_device_ids: &[DeviceId],
    ) -> EngineResult<SignedEnvelope> {
        if recipient_device_ids.is_empty() {
            return Err(EngineError::Sync(SyncError::InvalidEncoding(
                "data envelope requires at least one recipient wrap".to_string(),
            )));
        }
        let (signing_key, _x_secret) = self.load_local_keys()?;
        let local_seq =
            replication::next_local_seq(self.conn, &self.local_device_id.to_string())? as u64;

        let dek = ContentDek::generate().map_err(|e| EngineError::Crypto(e.to_string()))?;
        let envelope_id = EnvelopeId::new();
        let event_id = ReplicationEventId::new();

        let seal_aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id,
            blob_id: envelope_id.as_uuid(),
        };
        let sealed =
            seal(&dek, plaintext, &seal_aad).map_err(|e| EngineError::Crypto(e.to_string()))?;
        let ciphertext = encode_data_body(&sealed.nonce, &sealed.ciphertext)?;

        let mut wrap_records = Vec::with_capacity(recipient_device_ids.len());
        let mut sorted_recipients = recipient_device_ids.to_vec();
        sorted_recipients.sort_by(|a, b| a.as_uuid().as_bytes().cmp(b.as_uuid().as_bytes()));

        for recipient in &sorted_recipients {
            let peer_row = replication::get_device(self.conn, &recipient.to_string())?
                .ok_or(SyncError::NotEnrolled)?;
            if peer_row.status != "active" && peer_row.status != "local" {
                return Err(EngineError::Sync(SyncError::NotEnrolled));
            }
            let x_pub_bytes: [u8; 32] =
                peer_row.x25519_public.as_slice().try_into().map_err(|_| {
                    EngineError::InvalidKeyMaterial("x25519_public length".to_string())
                })?;
            let x_pub = X25519Public::from(x_pub_bytes);
            let wrap = wrap_content_dek_for_recipient(
                REPLICATION_SCHEMA_VERSION,
                dek.expose_secret(),
                &content_key_id,
                recipient,
                &self.local_device_id,
                &x_pub,
            )?;
            wrap_records.push(WrapRecord {
                recipient_device_id: *recipient,
                eph_x25519_pub: wrap.eph_x25519_pub,
                wrap_nonce: wrap.wrap_nonce,
                wrap_ct: wrap.wrap_ct,
            });
        }

        let outer = OuterEnvelope {
            schema_version: REPLICATION_SCHEMA_VERSION,
            envelope_id,
            device_id: self.local_device_id,
            local_seq,
            content_type_code: ContentTypeCode::DataEvent,
            event_id,
            content_key_id,
            ciphertext,
            wrap_records,
        };
        let signed = sign_envelope(&outer, &signing_key)?;

        // Index locally so next_local_seq advances and we own the envelope.
        let now = now_rfc3339()?;
        self.persist_local_data_index(&signed, &now)?;
        self.persist_outbox(&signed)?;
        self.pending.push_back(signed.clone());
        Ok(signed)
    }

    /// Queue a signed control envelope, persist index + control row, advance local seq.
    pub fn queue_control(&mut self, signed: SignedEnvelope, body: Vec<u8>) -> EngineResult<()> {
        // DB first; only mirror into in-memory pending after durable success so a
        // failed outer transaction cannot leave a ghost pending ACK (Codex R3 P2).
        self.queue_control_on(self.conn, &signed, body)?;
        self.pending.push_back(signed);
        Ok(())
    }

    /// Persist signed control + index + outbox on `conn` only (no in-memory pending).
    ///
    /// Callers that run inside a larger transaction MUST NOT touch `pending` until
    /// that transaction commits; `push_pending` drains durable outbox rows.
    fn queue_control_on(
        &mut self,
        conn: &Connection,
        signed: &SignedEnvelope,
        body: Vec<u8>,
    ) -> EngineResult<()> {
        let now = now_rfc3339()?;
        let outer = &signed.outer;
        let control = SignedControlRow {
            event_id: outer.event_id.to_string(),
            envelope_id: outer.envelope_id.as_uuid().to_string(),
            sender_device_id: outer.device_id.to_string(),
            content_type_code: outer.content_type_code.as_u16() as i64,
            body,
            signature: signed.signature.to_vec(),
            schema_version: outer.schema_version as i64,
            local_seq: outer.local_seq as i64,
            created_at: now.clone(),
        };
        let index = index_from_signed(signed, Some(&now));
        replication::insert_signed_control(conn, &control)?;
        replication::insert_envelope_index(conn, &index)?;
        self.persist_outbox_on(conn, signed)?;
        Ok(())
    }

    /// Put unpushed outbox rows to the relay (durable across process restarts).
    ///
    /// Drains the in-memory pending mirror first (idempotent outbox insert), then
    /// loads all `pushed_at IS NULL` rows for the local device ordered by `local_seq`.
    pub fn push_pending(&mut self) -> EngineResult<usize> {
        // Ensure same-session pending is durable before relay put.
        while let Some(signed) = self.pending.pop_front() {
            self.persist_outbox(&signed)?;
        }

        let local = self.local_device_id.to_string();
        let rows = replication::list_unpushed_outbox(self.conn, &local)?;
        let mut n = 0usize;
        for row in rows {
            let envelope_id = Uuid::parse_str(&row.envelope_id)
                .map_err(|e| EngineError::InvalidKeyMaterial(format!("outbox envelope_id: {e}")))?;
            let sender: DeviceId = row.sender_device_id.parse().map_err(|e| {
                EngineError::InvalidKeyMaterial(format!("outbox sender_device_id: {e}"))
            })?;
            let content_type_code = u16::try_from(row.content_type_code).map_err(|_| {
                EngineError::InvalidKeyMaterial(format!(
                    "outbox content_type_code out of range: {}",
                    row.content_type_code
                ))
            })?;
            let local_seq = u64::try_from(row.local_seq).map_err(|_| {
                EngineError::InvalidKeyMaterial(format!(
                    "outbox local_seq out of range: {}",
                    row.local_seq
                ))
            })?;
            let blob = RelayBlob {
                envelope_id,
                sender_device_id: sender,
                local_seq,
                content_type_code,
                body: row.wire_body,
            };
            self.relay.put(&blob)?;
            let now = now_rfc3339()?;
            replication::mark_outbox_pushed(self.conn, &row.envelope_id, &now)?;
            n += 1;
        }
        Ok(n)
    }

    /// Pull peers from local enrolled-set (F22), apply, drain gaps.
    pub fn pull_all_peers(&mut self) -> EngineResult<usize> {
        let local_str = self.local_device_id.to_string();
        let peers = replication::list_enrolled_devices(self.conn)?;
        let mut applied = 0usize;
        for peer in peers {
            if peer.device_id == local_str {
                continue;
            }
            let peer_id: DeviceId = peer
                .device_id
                .parse()
                .map_err(|e| EngineError::InvalidKeyMaterial(format!("peer device_id: {e}")))?;
            applied += self.pull_peer(&peer_id)?;
        }
        Ok(applied)
    }

    /// `push_pending` then `pull_all_peers` then `tick_ack_cycle` (F11).
    pub fn sync_round(&mut self) -> EngineResult<()> {
        self.push_pending()?;
        self.pull_all_peers()?;
        let now = now_rfc3339()?;
        replication::tick_ack_cycle(self.conn, &now)?;
        Ok(())
    }

    /// Single-envelope apply path (normative §5.3 order).
    pub fn apply_blob(&mut self, blob: &RelayBlob) -> EngineResult<ApplyOutcome> {
        // 1. Decode wire_v1; routing fields must match.
        let signed = decode_signed_envelope(&blob.body)?;
        if signed.outer.envelope_id.as_uuid() != blob.envelope_id
            || signed.outer.device_id != blob.sender_device_id
            || signed.outer.local_seq != blob.local_seq
            || signed.outer.content_type_code.as_u16() != blob.content_type_code
        {
            return Err(EngineError::RoutingMismatch);
        }

        let sender = signed.outer.device_id;
        let sender_str = sender.to_string();
        let local_seq = signed.outer.local_seq;
        let event_id = signed.outer.event_id.to_string();
        let now = now_rfc3339()?;

        // 2. L8 PRE-VERIFY (F9): lookup before Ed25519.
        let identity = replication::get_device(self.conn, &sender_str)?;
        match identity {
            None => return Err(EngineError::Sync(SyncError::NotEnrolled)),
            Some(row) if row.status == "revoked" => {
                return Err(EngineError::Sync(SyncError::DeviceRevoked));
            }
            Some(row) if row.status != "active" && row.status != "local" => {
                return Err(EngineError::Sync(SyncError::NotEnrolled));
            }
            Some(row) => {
                // 3. Verify Ed25519 with enrolled pub.
                let ed_bytes: [u8; 32] =
                    row.ed25519_public.as_slice().try_into().map_err(|_| {
                        EngineError::InvalidKeyMaterial("ed25519_public length".to_string())
                    })?;
                let vk = VerifyingKey::from_bytes(&ed_bytes).map_err(|e| {
                    EngineError::InvalidKeyMaterial(format!("ed25519 verifying key: {e}"))
                })?;
                verify_envelope(&signed, &vk)?;
            }
        }

        // 4. Schema version (F10) — fail closed on unknown.
        if signed.outer.schema_version != REPLICATION_SCHEMA_VERSION {
            return Err(EngineError::Sync(SyncError::SchemaVersionMismatch {
                got: signed.outer.schema_version,
                expected: REPLICATION_SCHEMA_VERSION,
            }));
        }

        // 5. Seq / gap gate.
        let mut cursor = self.ensure_cursor(&sender_str, &now)?;
        if cursor.state == "blocked" {
            return Err(EngineError::CursorBlocked {
                peer: sender_str,
                seq: local_seq,
            });
        }

        let expected = cursor.expected_local_seq as u64;
        if local_seq > expected {
            replication::buffer_gap_seq(
                self.conn,
                &sender_str,
                local_seq as i64,
                &signed.outer.envelope_id.as_uuid().to_string(),
                &now,
            )?;
            let high = cursor.high_water_seq.max(local_seq as i64);
            cursor.high_water_seq = high;
            cursor.state = "sync_gap".to_string();
            cursor.updated_at = now;
            replication::set_cursor(self.conn, &cursor)?;
            return Ok(ApplyOutcome::Buffered { local_seq });
        }

        if local_seq < expected {
            if let Some(existing) =
                replication::get_envelope_by_sender_seq(self.conn, &sender_str, local_seq as i64)?
            {
                if existing.event_id == event_id {
                    // Idempotent re-apply.
                    self.bump_high_water(&mut cursor, local_seq, &now)?;
                    return Ok(ApplyOutcome::Idempotent);
                }
                // F7: different event_id for same (sender, seq) → blocked.
                cursor.state = "blocked".to_string();
                cursor.updated_at = now;
                replication::set_cursor(self.conn, &cursor)?;
                return Err(EngineError::Sync(SyncError::SeqCollision));
            }
            // Behind expected with no index row for that seq — structured reject (L3).
            return Err(EngineError::Sync(SyncError::InvalidEncoding(format!(
                "local_seq {local_seq} < expected {expected} with no matching event_id in index"
            ))));
        }

        // local_seq == expected — check UNIQUE collision before project.
        if let Some(existing) =
            replication::get_envelope_by_sender_seq(self.conn, &sender_str, local_seq as i64)?
        {
            if existing.event_id == event_id {
                self.advance_cursor_after_apply(&mut cursor, local_seq, &now)?;
                let _ = self.drain_gaps(&sender)?;
                return Ok(ApplyOutcome::Idempotent);
            }
            cursor.state = "blocked".to_string();
            cursor.updated_at = now;
            replication::set_cursor(self.conn, &cursor)?;
            return Err(EngineError::Sync(SyncError::SeqCollision));
        }
        if replication::envelope_exists(self.conn, &event_id)? {
            // Same event under different seq — should not happen; index idempotent.
            self.advance_cursor_after_apply(&mut cursor, local_seq, &now)?;
            let _ = self.drain_gaps(&sender)?;
            return Ok(ApplyOutcome::Idempotent);
        }

        // 6a. Data path: open AEAD body before any durable side effects so we can
        // reject smuggled membership control fail-closed (P1) without wrap/index writes.
        let data_plaintext = if signed.outer.content_type_code == ContentTypeCode::DataEvent {
            let plaintext = self.open_data_plaintext(&signed)?;
            reject_smuggled_membership_control(&plaintext)?;
            Some(plaintext)
        } else {
            None
        };

        // 6–8. Project + envelope index + cursor advance in ONE transaction (P2).
        // drain_gaps runs only after successful commit.
        let mut drain_extra: Option<DeviceId> = None;
        {
            let tx = begin_unchecked_tx(self.conn)?;
            match signed.outer.content_type_code {
                ContentTypeCode::DataEvent => {
                    let plaintext = data_plaintext.as_deref().ok_or_else(|| {
                        EngineError::Crypto(
                            "internal: data plaintext missing after open".to_string(),
                        )
                    })?;
                    self.project_data_store(&tx, &signed, &now, plaintext)?;
                }
                ContentTypeCode::DeviceEnrolled
                | ContentTypeCode::DeviceRevoked
                | ContentTypeCode::ContentErasureTombstone
                | ContentTypeCode::ErasureAck
                | ContentTypeCode::GapSkipAudit => {
                    drain_extra = self.project_control_on(&tx, &signed, &now)?;
                }
            }

            let index = index_from_signed(&signed, Some(&now));
            // UNIQUE(sender,seq) different event_id already gated above.
            if let Err(e) = replication::insert_envelope_index(&tx, &index) {
                // Map constraint race to blocked (F7). Roll back project side effects.
                let msg = e.to_string().to_lowercase();
                if msg.contains("unique") {
                    drop(tx);
                    cursor.state = "blocked".to_string();
                    cursor.updated_at = now;
                    replication::set_cursor(self.conn, &cursor)?;
                    return Err(EngineError::Sync(SyncError::SeqCollision));
                }
                return Err(EngineError::Store(e));
            }

            Self::advance_cursor_on(&tx, &mut cursor, local_seq, &now)?;
            commit_tx(tx)?;
        }

        // 9. Drain loop (new transactions; after commit).
        let _ = self.drain_gaps(&sender)?;
        if let Some(peer) = drain_extra {
            let _ = self.drain_gaps(&peer)?;
        }
        Ok(ApplyOutcome::Applied)
    }

    // -----------------------------------------------------------------------
    // Internals
    // -----------------------------------------------------------------------

    fn pull_peer(&mut self, peer: &DeviceId) -> EngineResult<usize> {
        let peer_str = peer.to_string();
        let now = now_rfc3339()?;
        let cursor = self.ensure_cursor(&peer_str, &now)?;
        if cursor.state == "blocked" {
            return Ok(0);
        }
        // Pull from expected-1 so gaps reappear after reorder/delay.
        let after = (cursor.expected_local_seq as u64).saturating_sub(1);
        let blobs = self.relay.pull(peer, after, self.pull_limit)?;
        let mut n = 0usize;
        for blob in &blobs {
            match self.apply_blob(blob) {
                Ok(ApplyOutcome::Applied) => n += 1,
                Ok(ApplyOutcome::Idempotent) | Ok(ApplyOutcome::Buffered { .. }) => {}
                Err(EngineError::Sync(SyncError::SeqCollision))
                | Err(EngineError::CursorBlocked { .. }) => {
                    // Hard stop for this peer.
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        // Drain even if pull returned empty (gap fill via pull_range).
        n += self.drain_gaps(peer)?;
        Ok(n)
    }

    /// Sequential drain: while gap_buffer has `expected`, re-fetch body and apply.
    fn drain_gaps(&mut self, peer: &DeviceId) -> EngineResult<usize> {
        let peer_str = peer.to_string();
        let mut drained = 0usize;
        loop {
            let now = now_rfc3339()?;
            let cursor = match replication::get_cursor(self.conn, &peer_str)? {
                Some(c) if c.state != "blocked" => c,
                _ => break,
            };
            let expected = cursor.expected_local_seq;
            if !replication::gap_buffer_has_seq(self.conn, &peer_str, expected)? {
                // No sequential buffer row — recompute state.
                let gaps = replication::list_gap_buffer(self.conn, &peer_str)?;
                let state = if gaps.is_empty() {
                    "in_sync"
                } else {
                    "sync_gap"
                };
                if cursor.state != state {
                    replication::set_gap_state(self.conn, &peer_str, state, &now)?;
                }
                break;
            }

            // Prefer body from relay pull_range (F8 / option b).
            let blobs = self
                .relay
                .pull_range(peer, expected as u64, expected as u64)?;
            let Some(blob) = blobs.into_iter().next() else {
                // Body not available yet (C5 delay) — remain sync_gap.
                replication::set_gap_state(self.conn, &peer_str, "sync_gap", &now)?;
                break;
            };

            // Remove buffer entry before apply so re-entry is clean.
            replication::delete_gap_seq(self.conn, &peer_str, expected)?;

            match self.apply_blob(&blob)? {
                ApplyOutcome::Applied | ApplyOutcome::Idempotent => {
                    drained += 1;
                }
                ApplyOutcome::Buffered { .. } => {
                    // Should not re-buffer expected; stop to avoid loop.
                    break;
                }
            }
        }
        Ok(drained)
    }

    /// Open DataEvent AEAD body (crypto only; no durable side effects).
    fn open_data_plaintext(&self, signed: &SignedEnvelope) -> EngineResult<Vec<u8>> {
        let local = self.local_device_id;
        let content_key_id = signed.outer.content_key_id;
        let schema = signed.outer.schema_version;
        let sender = signed.outer.device_id;

        let self_wrap = signed
            .outer
            .wrap_records
            .iter()
            .find(|w| w.recipient_device_id == local)
            .ok_or(EngineError::MissingSelfWrap)?;

        let (_signing, x_secret) = self.load_local_keys()?;
        let peer_wrap = PeerDekWrap {
            eph_x25519_pub: self_wrap.eph_x25519_pub,
            wrap_nonce: self_wrap.wrap_nonce,
            wrap_ct: self_wrap.wrap_ct.clone(),
        };
        let dek_bytes = unwrap_content_dek(
            schema,
            &peer_wrap,
            &content_key_id,
            &local,
            &sender,
            &x_secret,
        )?;
        let (nonce, ct_and_tag) = decode_data_body(&signed.outer.ciphertext)?;
        let dek = ContentDek::from_bytes(dek_bytes);
        let seal_aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id,
            blob_id: signed.outer.envelope_id.as_uuid(),
        };
        let sealed = SealedContent {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            nonce,
            ciphertext: ct_and_tag,
        };
        let plaintext =
            open(&dek, &sealed, &seal_aad).map_err(|e| EngineError::Crypto(e.to_string()))?;
        Ok(plaintext.to_vec())
    }

    /// Durable data project: domain append (if Envelope JSON) + wrap upserts.
    ///
    /// Caller must already have opened plaintext and rejected membership smuggling.
    /// Runs inside the caller's transaction (`tx`).
    fn project_data_store(
        &self,
        tx: &rusqlite::Transaction<'_>,
        signed: &SignedEnvelope,
        now: &str,
        plaintext: &[u8],
    ) -> EngineResult<()> {
        let content_key_id = signed.outer.content_key_id;
        let schema = signed.outer.schema_version;
        let sender = signed.outer.device_id;

        // Spec step 5b: append domain event when plaintext is Envelope JSON.
        // Opaque bodies keep wrap storage only — parse failure must NOT fail apply.
        // Membership control payloads are rejected before this method is called.
        if let Ok(domain) = serde_json::from_slice::<ai_brains_events::Envelope>(plaintext) {
            // Defense in depth: refuse membership even if caller skipped pre-check.
            reject_smuggled_membership_payload(&domain.payload)?;
            crate::event_store::append_event_in_tx(tx, &domain)?;
        }

        for w in &signed.outer.wrap_records {
            replication::upsert_peer_content_key_wrap(
                tx,
                &PeerContentKeyWrapRow {
                    content_key_id: content_key_id.to_string(),
                    recipient_device_id: w.recipient_device_id.to_string(),
                    sender_device_id: sender.to_string(),
                    schema_version: i64::from(schema),
                    eph_x25519_public: w.eph_x25519_pub.to_vec(),
                    wrap_nonce: w.wrap_nonce.to_vec(),
                    wrap_ciphertext: w.wrap_ct.clone(),
                    created_at: now.to_string(),
                },
            )?;
        }
        Ok(())
    }

    /// Project control side effects on `conn` (typically an open transaction).
    ///
    /// Returns an optional extra peer whose gap buffer should be drained after
    /// the outer apply transaction commits (GapSkipAudit non-fill-slot path).
    fn project_control_on(
        &mut self,
        conn: &Connection,
        signed: &SignedEnvelope,
        now: &str,
    ) -> EngineResult<Option<DeviceId>> {
        let kind = signed.outer.content_type_code;
        let payload = decode_control_payload(kind, &signed.outer.ciphertext)?;

        let control = SignedControlRow {
            event_id: signed.outer.event_id.to_string(),
            envelope_id: signed.outer.envelope_id.as_uuid().to_string(),
            sender_device_id: signed.outer.device_id.to_string(),
            content_type_code: kind.as_u16() as i64,
            body: signed.outer.ciphertext.clone(),
            signature: signed.signature.to_vec(),
            schema_version: signed.outer.schema_version as i64,
            local_seq: signed.outer.local_seq as i64,
            created_at: now.to_string(),
        };
        replication::insert_signed_control(conn, &control)?;

        let mut drain_extra = None;
        match payload {
            ControlPayload::DeviceEnrolled(p) => {
                if p.schema_version != REPLICATION_SCHEMA_VERSION {
                    return Err(EngineError::Sync(SyncError::SchemaVersionMismatch {
                        got: p.schema_version,
                        expected: REPLICATION_SCHEMA_VERSION,
                    }));
                }
                let device_str = p.device_id.to_string();
                if replication::get_device(conn, &device_str)?.is_none() {
                    let package = enrollment_package(&p.device_id, &p.ed25519_pub, &p.x25519_pub);
                    let fp = fingerprint_sha256(&package);
                    replication::insert_device_identity(
                        conn,
                        &DeviceIdentityRow {
                            device_id: device_str,
                            schema_version: i64::from(p.schema_version),
                            ed25519_public: p.ed25519_pub.to_vec(),
                            x25519_public: p.x25519_pub.to_vec(),
                            display_name: None,
                            status: "active".to_string(),
                            enrolled_at: now.to_string(),
                            revoked_at: None,
                            enrolled_by_device_id: signed.outer.device_id.to_string(),
                            fingerprint_sha256: fp.to_vec(),
                        },
                    )?;
                }
            }
            ControlPayload::DeviceRevoked(p) => {
                let device_str = p.device_id.to_string();
                if let Some(existing) = replication::get_device(conn, &device_str)?
                    && existing.status != "revoked"
                {
                    replication::tombstone_device(conn, &device_str, now, &p.reason_code)?;
                }
            }
            ControlPayload::ContentErasureTombstone(p) => {
                // F21: destroy local CE + peer wraps; upsert pending acks; queue ErasureAck.
                let ck = p.content_key_id.to_string();
                let had_key_row = content_envelope::get_content_key_wrap(conn, &ck)?.is_some();
                content_envelope::destroy_content_key_wrap(conn, &ck, now)?;
                if had_key_row {
                    verify_content_key_destroyed(conn, &ck)?;
                }
                let erasure_id = signed.outer.event_id.to_string();
                let local_str = self.local_device_id.to_string();
                replication::upsert_erasure_ack(
                    conn,
                    &ErasureAckRow {
                        erasure_id: erasure_id.clone(),
                        peer_device_id: signed.outer.device_id.to_string(),
                        content_key_id: ck.clone(),
                        status: "pending".to_string(),
                        sync_cycles_waiting: 0,
                        updated_at: now.to_string(),
                    },
                )?;
                let sender_str = signed.outer.device_id.to_string();
                for peer in replication::list_enrolled_devices(conn)? {
                    if peer.device_id == local_str || peer.device_id == sender_str {
                        continue;
                    }
                    replication::upsert_erasure_ack(
                        conn,
                        &ErasureAckRow {
                            erasure_id: erasure_id.clone(),
                            peer_device_id: peer.device_id,
                            content_key_id: ck.clone(),
                            status: "pending".to_string(),
                            sync_cycles_waiting: 0,
                            updated_at: now.to_string(),
                        },
                    )?;
                }
                self.queue_erasure_ack_on(
                    conn,
                    ReplicationEventId::from_uuid(signed.outer.event_id.as_uuid()),
                    p.content_key_id,
                )?;
            }
            ControlPayload::ErasureAck(p) => {
                // Peer attestation is bound to the authenticated outer signer.
                // Reject spoofed payload peer_device_id (Codex R4).
                if p.peer_device_id != signed.outer.device_id {
                    return Err(EngineError::Sync(SyncError::InvalidEncoding(
                        "ErasureAck peer_device_id must match authenticated sender_device_id"
                            .to_string(),
                    )));
                }
                let status = if p.status == "acked" || p.status == "failed" {
                    p.status.clone()
                } else {
                    "acked".to_string()
                };
                replication::upsert_erasure_ack(
                    conn,
                    &ErasureAckRow {
                        erasure_id: p.erasure_id.to_string(),
                        peer_device_id: signed.outer.device_id.to_string(),
                        content_key_id: p.content_key_id.to_string(),
                        status,
                        sync_cycles_waiting: 0,
                        updated_at: now.to_string(),
                    },
                )?;
            }
            ControlPayload::GapSkipAudit(p) => {
                replication::insert_gap_skip_audit(
                    conn,
                    &signed.outer.event_id.to_string(),
                    &p.peer_device_id.to_string(),
                    p.skipped_seq as i64,
                    &signed.outer.event_id.to_string(),
                    now,
                )?;
                drain_extra =
                    self.apply_gap_skip_on(conn, &p.peer_device_id, p.skipped_seq, signed, now)?;
            }
        }
        Ok(drain_extra)
    }

    /// Advance the target peer past a permanently missing seq when audit is valid.
    ///
    /// When this GapSkipAudit envelope *is* the control occupying `skipped_seq`
    /// on the same peer stream (`sender == peer && local_seq == skipped_seq`),
    /// only clear buffer metadata — the outer apply path advances the cursor and
    /// drains so we do not clobber a multi-seq drain with a stale cursor rewrite.
    ///
    /// Returns `Some(peer)` when the caller should `drain_gaps(peer)` **after**
    /// committing the outer apply transaction (non-fill-slot path).
    fn apply_gap_skip_on(
        &self,
        conn: &Connection,
        peer_device_id: &DeviceId,
        skipped_seq: u64,
        signed: &SignedEnvelope,
        now: &str,
    ) -> EngineResult<Option<DeviceId>> {
        let peer_str = peer_device_id.to_string();
        let Some(mut cursor) = replication::get_cursor(conn, &peer_str)? else {
            return Ok(None);
        };
        if cursor.state == "blocked" {
            return Ok(None);
        }
        if (cursor.expected_local_seq as u64) != skipped_seq {
            return Ok(None);
        }

        replication::delete_gap_seq(conn, &peer_str, skipped_seq as i64)?;

        let fills_skipped_slot =
            signed.outer.device_id == *peer_device_id && signed.outer.local_seq == skipped_seq;
        if fills_skipped_slot {
            // apply_blob will advance this seq and drain sequential buffers.
            return Ok(None);
        }

        cursor.expected_local_seq = (skipped_seq as i64) + 1;
        cursor.high_water_seq = cursor.high_water_seq.max(skipped_seq as i64);
        let gaps = replication::list_gap_buffer(conn, &peer_str)?;
        cursor.state = if gaps.is_empty() {
            "in_sync".to_string()
        } else {
            "sync_gap".to_string()
        };
        cursor.updated_at = now.to_string();
        replication::set_cursor(conn, &cursor)?;
        Ok(Some(*peer_device_id))
    }

    fn queue_erasure_ack_on(
        &mut self,
        conn: &Connection,
        erasure_id: ReplicationEventId,
        content_key_id: ContentKeyId,
    ) -> EngineResult<()> {
        let (signing_key, _) = self.load_local_keys()?;
        let local_seq =
            replication::next_local_seq(conn, &self.local_device_id.to_string())? as u64;
        let payload = ControlPayload::ErasureAck(ErasureAckPayload {
            erasure_id,
            content_key_id,
            peer_device_id: self.local_device_id,
            status: "acked".to_string(),
        });
        let built = build_and_sign_control(
            ContentTypeCode::ErasureAck,
            &payload,
            self.local_device_id,
            local_seq,
            &signing_key,
            content_key_id,
        )?;
        // Outbox only (no pending mirror): caller runs inside apply TX; after
        // commit, push_pending loads unpushed outbox rows.
        self.queue_control_on(conn, &built.signed, built.body)?;
        Ok(())
    }

    fn load_local_keys(&self) -> EngineResult<(SigningKey, StaticSecret)> {
        let device_str = self.local_device_id.to_string();
        let wrap = replication::get_device_private_key_wrap(self.conn, &device_str)?
            .ok_or_else(|| EngineError::LocalKeyMissing(device_str.clone()))?;
        let nonce: [u8; 12] = wrap.wrap_nonce.as_slice().try_into().map_err(|_| {
            EngineError::InvalidKeyMaterial("wrap_nonce must be 12 bytes".to_string())
        })?;
        let sealed = SealedDevicePrivate {
            wrap_schema_version: wrap.wrap_schema_version as u32,
            protection: wrap.protection,
            wrap_nonce: nonce,
            wrap_ciphertext: wrap.wrap_ciphertext,
        };
        let seeds = open_device_private_blob(&self.data_key, &sealed, &self.local_device_id)?;
        let pair = seeds.into_key_pair();
        Ok((pair.signing_key(), pair.x25519_secret()))
    }

    fn persist_local_data_index(&self, signed: &SignedEnvelope, now: &str) -> EngineResult<()> {
        let index = index_from_signed(signed, Some(now));
        replication::insert_envelope_index(self.conn, &index)?;
        // Store our wraps for local bookkeeping.
        for w in &signed.outer.wrap_records {
            replication::upsert_peer_content_key_wrap(
                self.conn,
                &PeerContentKeyWrapRow {
                    content_key_id: signed.outer.content_key_id.to_string(),
                    recipient_device_id: w.recipient_device_id.to_string(),
                    sender_device_id: signed.outer.device_id.to_string(),
                    schema_version: i64::from(signed.outer.schema_version),
                    eph_x25519_public: w.eph_x25519_pub.to_vec(),
                    wrap_nonce: w.wrap_nonce.to_vec(),
                    wrap_ciphertext: w.wrap_ct.clone(),
                    created_at: now.to_string(),
                },
            )?;
        }
        Ok(())
    }

    fn ensure_cursor(&self, peer: &str, now: &str) -> EngineResult<ReplicationCursorRow> {
        if let Some(c) = replication::get_cursor(self.conn, peer)? {
            return Ok(c);
        }
        let row = ReplicationCursorRow {
            peer_device_id: peer.to_string(),
            high_water_seq: 0,
            expected_local_seq: 1,
            state: "in_sync".to_string(),
            updated_at: now.to_string(),
        };
        replication::set_cursor(self.conn, &row)?;
        Ok(row)
    }

    fn advance_cursor_after_apply(
        &self,
        cursor: &mut ReplicationCursorRow,
        applied_seq: u64,
        now: &str,
    ) -> EngineResult<()> {
        Self::advance_cursor_on(self.conn, cursor, applied_seq, now)
    }

    fn advance_cursor_on(
        conn: &Connection,
        cursor: &mut ReplicationCursorRow,
        applied_seq: u64,
        now: &str,
    ) -> EngineResult<()> {
        cursor.expected_local_seq = (applied_seq as i64) + 1;
        cursor.high_water_seq = cursor.high_water_seq.max(applied_seq as i64);
        let gaps = replication::list_gap_buffer(conn, &cursor.peer_device_id)?;
        cursor.state = if gaps.is_empty() {
            "in_sync".to_string()
        } else {
            "sync_gap".to_string()
        };
        cursor.updated_at = now.to_string();
        replication::set_cursor(conn, cursor)?;
        Ok(())
    }

    fn bump_high_water(
        &self,
        cursor: &mut ReplicationCursorRow,
        seq: u64,
        now: &str,
    ) -> EngineResult<()> {
        cursor.high_water_seq = cursor.high_water_seq.max(seq as i64);
        cursor.updated_at = now.to_string();
        replication::set_cursor(self.conn, cursor)?;
        Ok(())
    }

    /// Persist full wire body so `push_pending` works after engine/process restart.
    fn persist_outbox(&self, signed: &SignedEnvelope) -> EngineResult<()> {
        self.persist_outbox_on(self.conn, signed)
    }

    fn persist_outbox_on(&self, conn: &Connection, signed: &SignedEnvelope) -> EngineResult<()> {
        let wire_body = encode_signed_envelope(signed)?;
        let now = now_rfc3339()?;
        let outer = &signed.outer;
        replication::insert_outbox(
            conn,
            &OutboxRow {
                envelope_id: outer.envelope_id.as_uuid().to_string(),
                event_id: outer.event_id.to_string(),
                sender_device_id: outer.device_id.to_string(),
                local_seq: outer.local_seq as i64,
                content_type_code: i64::from(outer.content_type_code.as_u16()),
                wire_body,
                created_at: now,
                pushed_at: None,
            },
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_rfc3339() -> EngineResult<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| EngineError::Timestamp(e.to_string()))
}

fn begin_unchecked_tx(conn: &Connection) -> EngineResult<rusqlite::Transaction<'_>> {
    conn.unchecked_transaction()
        .map_err(|e| EngineError::Store(StoreError::from(e)))
}

fn commit_tx(tx: rusqlite::Transaction<'_>) -> EngineResult<()> {
    tx.commit()
        .map_err(|e| EngineError::Store(StoreError::from(e)))
}

/// Reject domain membership control smuggled through a DataEvent body (P1 / L8).
///
/// Membership MUST only arrive via outer SignedEnvelope control types
/// (`DeviceEnrolled` 0x0010 / `DeviceRevoked` 0x0011) with L8 pre-verify + Ed25519.
fn reject_smuggled_membership_control(plaintext: &[u8]) -> EngineResult<()> {
    if let Ok(domain) = serde_json::from_slice::<ai_brains_events::Envelope>(plaintext) {
        reject_smuggled_membership_payload(&domain.payload)?;
    }
    Ok(())
}

fn reject_smuggled_membership_payload(payload: &ai_brains_events::Payload) -> EngineResult<()> {
    match payload {
        ai_brains_events::Payload::DeviceEnrolled(_)
        | ai_brains_events::Payload::DeviceRevoked(_) => {
            Err(EngineError::Sync(SyncError::InvalidEncoding(
                "membership control must not be sealed as DataEvent; use outer SignedEnvelope \
                 control content types (0x0010 DeviceEnrolled / 0x0011 DeviceRevoked)"
                    .to_string(),
            )))
        }
        _ => Ok(()),
    }
}

/// After destroy: require status `destroyed` when the row still exists.
fn verify_content_key_destroyed(conn: &Connection, content_key_id: &str) -> EngineResult<()> {
    match content_envelope::get_content_key_wrap(conn, content_key_id)? {
        Some(row) if row.status == "destroyed" => Ok(()),
        Some(row) => Err(EngineError::Crypto(format!(
            "content key {content_key_id} not destroyed after tombstone (status={})",
            row.status
        ))),
        None => Err(EngineError::Crypto(format!(
            "content key {content_key_id} row missing after destroy"
        ))),
    }
}

fn index_from_signed(signed: &SignedEnvelope, applied_at: Option<&str>) -> EnvelopeIndexRow {
    let outer = &signed.outer;
    EnvelopeIndexRow {
        envelope_id: outer.envelope_id.as_uuid().to_string(),
        event_id: outer.event_id.to_string(),
        sender_device_id: outer.device_id.to_string(),
        local_seq: outer.local_seq as i64,
        content_type_code: outer.content_type_code.as_u16() as i64,
        content_key_id: Some(outer.content_key_id.to_string()),
        body_len: outer.ciphertext.len() as i64,
        padding_bucket: None,
        applied_at: applied_at.map(|s| s.to_string()),
    }
}

/// Build a DeviceRevoked control, sign, and queue on the engine (local authority).
///
/// Tombstone + signed control + envelope index + outbox commit in **one** TX.
pub fn sign_and_queue_revoke<R: RelayPort>(
    engine: &mut ReplicateEngine<'_, R>,
    target: DeviceId,
    reason_code: &str,
) -> EngineResult<SignedEnvelope> {
    let (signing_key, _) = engine.load_local_keys()?;
    let local_seq =
        replication::next_local_seq(engine.conn, &engine.local_device_id.to_string())? as u64;
    let payload = ControlPayload::DeviceRevoked(DeviceRevokedPayload {
        device_id: target,
        reason_code: reason_code.to_string(),
    });
    let built = build_and_sign_control(
        ContentTypeCode::DeviceRevoked,
        &payload,
        engine.local_device_id,
        local_seq,
        &signing_key,
        nil_content_key_id(),
    )?;
    let now = now_rfc3339()?;
    let tx = begin_unchecked_tx(engine.conn)?;
    replication::tombstone_device(&tx, &target.to_string(), &now, reason_code)?;
    let outer = &built.signed.outer;
    let control = SignedControlRow {
        event_id: outer.event_id.to_string(),
        envelope_id: outer.envelope_id.as_uuid().to_string(),
        sender_device_id: outer.device_id.to_string(),
        content_type_code: outer.content_type_code.as_u16() as i64,
        body: built.body.clone(),
        signature: built.signed.signature.to_vec(),
        schema_version: outer.schema_version as i64,
        local_seq: outer.local_seq as i64,
        created_at: now.clone(),
    };
    let index = index_from_signed(&built.signed, Some(&now));
    replication::insert_signed_control(&tx, &control)?;
    replication::insert_envelope_index(&tx, &index)?;
    engine.persist_outbox_on(&tx, &built.signed)?;
    commit_tx(tx)?;
    engine.pending.push_back(built.signed.clone());
    Ok(built.signed)
}

/// Build a ContentErasureTombstone control, sign, queue (local CE destroy on push path).
///
/// Destroy + pending ACK rows + control/outbox commit in **one** TX.
pub fn sign_and_queue_erasure_tombstone<R: RelayPort>(
    engine: &mut ReplicateEngine<'_, R>,
    content_key_id: ContentKeyId,
    reason_code: &str,
) -> EngineResult<SignedEnvelope> {
    let (signing_key, _) = engine.load_local_keys()?;
    let local_seq =
        replication::next_local_seq(engine.conn, &engine.local_device_id.to_string())? as u64;
    let payload = ControlPayload::ContentErasureTombstone(ContentErasureTombstonePayload {
        content_key_id,
        reason_code: reason_code.to_string(),
    });
    let built = build_and_sign_control(
        ContentTypeCode::ContentErasureTombstone,
        &payload,
        engine.local_device_id,
        local_seq,
        &signing_key,
        content_key_id,
    )?;
    let now = now_rfc3339()?;
    let ck = content_key_id.to_string();
    let tx = begin_unchecked_tx(engine.conn)?;
    let had_key_row = content_envelope::get_content_key_wrap(&tx, &ck)?.is_some();
    content_envelope::destroy_content_key_wrap(&tx, &ck, &now)?;
    // M3: when a content_key_store row existed, confirm destroy landed.
    // Missing-key still queues pending acks + control (peer attestation of apply).
    if had_key_row {
        verify_content_key_destroyed(&tx, &ck)?;
    }
    let erasure_id = built.signed.outer.event_id.to_string();
    let local_str = engine.local_device_id.to_string();
    for peer in replication::list_enrolled_devices(&tx)? {
        if peer.device_id == local_str {
            continue;
        }
        replication::upsert_erasure_ack(
            &tx,
            &ErasureAckRow {
                erasure_id: erasure_id.clone(),
                peer_device_id: peer.device_id,
                content_key_id: content_key_id.to_string(),
                status: "pending".to_string(),
                sync_cycles_waiting: 0,
                updated_at: now.clone(),
            },
        )?;
    }
    let outer = &built.signed.outer;
    let control = SignedControlRow {
        event_id: outer.event_id.to_string(),
        envelope_id: outer.envelope_id.as_uuid().to_string(),
        sender_device_id: outer.device_id.to_string(),
        content_type_code: outer.content_type_code.as_u16() as i64,
        body: built.body.clone(),
        signature: built.signed.signature.to_vec(),
        schema_version: outer.schema_version as i64,
        local_seq: outer.local_seq as i64,
        created_at: now.clone(),
    };
    let index = index_from_signed(&built.signed, Some(&now));
    replication::insert_signed_control(&tx, &control)?;
    replication::insert_envelope_index(&tx, &index)?;
    engine.persist_outbox_on(&tx, &built.signed)?;
    commit_tx(tx)?;
    engine.pending.push_back(built.signed.clone());
    Ok(built.signed)
}

/// Encode a signed envelope as a [`RelayBlob`].
pub fn signed_to_blob(signed: &SignedEnvelope) -> EngineResult<RelayBlob> {
    let body = encode_signed_envelope(signed)?;
    Ok(RelayBlob {
        envelope_id: signed.outer.envelope_id.as_uuid(),
        sender_device_id: signed.outer.device_id,
        local_seq: signed.outer.local_seq,
        content_type_code: signed.outer.content_type_code.as_u16(),
        body,
    })
}
