//! Shared TwinVaults harness for T177 convergence + T178 security tests (F23).
//!
//! Extracted from `replication_converge.rs` so both integration test binaries
//! share one enrollment / seal / apply surface without duplication.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{ContentKeyId, DeviceId, MemoryId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, MemoryPinnedPayload, Payload};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::projections::replication::{
    self, BootstrapLocalDeviceInput, DeviceIdentityRow, DevicePrivateKeyRow, EnvelopeIndexRow,
    SignedControlRow,
};
use ai_brains_store::replication_engine::{EngineError, ReplicateEngine};
use ai_brains_sync::{
    ContentTypeCode, ControlPayload, DeviceEnrolledPayload, DevicePrivateSeeds, MemoryFakeRelay,
    REPLICATION_SCHEMA_VERSION, RelayBlob, RelayPort, build_and_sign_control, enrollment_package,
    fingerprint_sha256, generate_device_keys, nil_content_key_id, seal_device_private_blob,
    verify_envelope,
};
use ed25519_dalek::SigningKey;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Fixed timestamp used by harness bootstrap / OOB rows.
pub const CREATED_AT: &str = "2026-07-31T12:00:00Z";

/// Build a domain MemoryPinned Envelope serialized as JSON (production seal body).
pub fn domain_pin_plaintext(content: &str) -> (Uuid, Vec<u8>) {
    let memory_id = MemoryId::new();
    let envelope = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.to_string(),
        session_id: None,
        project_id: None,
        tx_id: None,
        rank: None,
        source_tag: Some("t177-sync".into()),
        query_text: None,
    }))
    .expect("build MemoryPinned envelope");
    let event_id = envelope.event_id;
    let bytes = serde_json::to_vec(&envelope).expect("serialize domain Envelope");
    (event_id, bytes)
}

/// Count domain `events` rows for `event_id` in a vault.
pub fn domain_event_count(vault: &TestVault, event_id: Uuid) -> i64 {
    let c = vault.conn.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM events WHERE event_id = ?",
        [event_id.to_string()],
        |r| r.get(0),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Security snapshot (F19)
// ---------------------------------------------------------------------------

/// Security-relevant vault state for side-effect isolation on reject paths (F19).
///
/// Expanded under IR1-M7: envelope index detail, peer wraps, signed controls,
/// gap buffer, tombstones, and domain event count — all ordered for PartialEq.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecuritySnapshot {
    pub event_ids: Vec<String>,
    /// (event_id, sender_device_id, local_seq, content_type_code)
    pub envelope_index: Vec<(String, String, i64, i64)>,
    pub peer_cursors: Vec<(String, i64, i64, String)>,
    pub device_identity: Vec<(String, String, Option<String>)>,
    pub erasure_acks: Vec<(String, String, String)>,
    /// (content_key_id, recipient_device_id) pairs from peer_content_key_wrap
    pub peer_content_key_wraps: Vec<(String, String)>,
    pub signed_control_event_ids: Vec<String>,
    /// (peer_device_id, local_seq) gap buffer rows
    pub gap_buffer: Vec<(String, i64)>,
    pub device_id_tombstones: Vec<String>,
    pub domain_events_count: i64,
}

impl TestVault {
    /// Capture security-relevant rows (envelope index, wraps, controls, cursors, …).
    pub fn capture_security_snapshot(&self) -> SecuritySnapshot {
        let c = self.conn.lock().unwrap();

        let mut event_ids = replication::list_envelope_event_ids(&c).unwrap();
        event_ids.sort();

        let mut envelope_index = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT event_id, sender_device_id, local_seq, content_type_code
                     FROM encrypted_envelope_index
                     ORDER BY event_id, sender_device_id, local_seq",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .unwrap();
            for r in rows {
                envelope_index.push(r.unwrap());
            }
        }

        let mut peer_cursors = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT peer_device_id, high_water_seq, expected_local_seq, state
                     FROM replication_cursor
                     ORDER BY peer_device_id",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .unwrap();
            for r in rows {
                peer_cursors.push(r.unwrap());
            }
        }

        let mut device_identity = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT device_id, status, revoked_at
                     FROM device_identity
                     ORDER BY device_id",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                })
                .unwrap();
            for r in rows {
                device_identity.push(r.unwrap());
            }
        }

        let mut erasure_acks = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT erasure_id, peer_device_id, status
                     FROM erasure_ack_projection
                     ORDER BY erasure_id, peer_device_id",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .unwrap();
            for r in rows {
                erasure_acks.push(r.unwrap());
            }
        }

        let mut peer_content_key_wraps = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT content_key_id, recipient_device_id
                     FROM peer_content_key_wrap
                     ORDER BY content_key_id, recipient_device_id",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .unwrap();
            for r in rows {
                peer_content_key_wraps.push(r.unwrap());
            }
        }

        let mut signed_control_event_ids = Vec::new();
        {
            let mut stmt = c
                .prepare("SELECT event_id FROM signed_replication_control ORDER BY event_id")
                .unwrap();
            let rows = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();
            for r in rows {
                signed_control_event_ids.push(r.unwrap());
            }
        }

        let mut gap_buffer = Vec::new();
        {
            let mut stmt = c
                .prepare(
                    "SELECT peer_device_id, local_seq
                     FROM replication_gap_buffer
                     ORDER BY peer_device_id, local_seq",
                )
                .unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })
                .unwrap();
            for r in rows {
                gap_buffer.push(r.unwrap());
            }
        }

        let mut device_id_tombstones = Vec::new();
        {
            let mut stmt = c
                .prepare("SELECT device_id FROM device_id_tombstone ORDER BY device_id")
                .unwrap();
            let rows = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();
            for r in rows {
                device_id_tombstones.push(r.unwrap());
            }
        }

        let domain_events_count: i64 = c
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();

        SecuritySnapshot {
            event_ids,
            envelope_index,
            peer_cursors,
            device_identity,
            erasure_acks,
            peer_content_key_wraps,
            signed_control_event_ids,
            gap_buffer,
            device_id_tombstones,
            domain_events_count,
        }
    }
}

/// Apply an adversarial blob; assert reject **and** zero security-relevant side effects (F19).
pub fn assert_rejected_no_side_effect<R: RelayPort>(
    vault: &TestVault,
    relay: Arc<R>,
    blob: &RelayBlob,
) -> EngineError {
    let before = vault.capture_security_snapshot();
    let err = vault.with_engine(relay, |e| e.apply_blob(blob).unwrap_err());
    let after = vault.capture_security_snapshot();
    assert_eq!(
        before, after,
        "adversarial rejection must leave zero security-relevant side-effects\nbefore={before:?}\nafter={after:?}\nerr={err}"
    );
    err
}

// ---------------------------------------------------------------------------
// Test vault
// ---------------------------------------------------------------------------

pub struct TestVault {
    pub _tmp: TempDir,
    pub data_key: DataKey,
    pub conn: VaultConnection,
    pub device_id: DeviceId,
    /// Keep signing seed available for adversarial / collision tests.
    pub signing_key: SigningKey,
}

impl TestVault {
    pub fn bootstrap() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("vault.db");
        let data_key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&data_key);
        let conn = VaultConnection::open(&path, &sql_key).unwrap();
        conn.migrate().unwrap();

        let keys = generate_device_keys().unwrap();
        let device_id = DeviceId::new();
        let ed_pub = keys.verifying_key().to_bytes();
        let x_pub = keys.x25519_public().to_bytes();
        let package = enrollment_package(&device_id, &ed_pub, &x_pub);
        let fp = fingerprint_sha256(&package);

        let built = build_and_sign_control(
            ContentTypeCode::DeviceEnrolled,
            &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
                schema_version: REPLICATION_SCHEMA_VERSION,
                device_id,
                ed25519_pub: ed_pub,
                x25519_pub: x_pub,
            }),
            device_id,
            1,
            &keys.signing_key(),
            nil_content_key_id(),
        )
        .unwrap();
        verify_envelope(&built.signed, &keys.verifying_key()).unwrap();

        let seeds = DevicePrivateSeeds::from_key_pair(&keys);
        let sealed = seal_device_private_blob(&data_key, &seeds, &device_id).unwrap();
        let private_key = DevicePrivateKeyRow {
            device_id: device_id.to_string(),
            wrap_schema_version: sealed.wrap_schema_version as i64,
            algorithm: "AES-256-GCM".to_string(),
            protection: sealed.protection,
            wrap_nonce: sealed.wrap_nonce.to_vec(),
            wrap_ciphertext: sealed.wrap_ciphertext,
            created_at: CREATED_AT.to_string(),
        };

        let input = BootstrapLocalDeviceInput {
            identity: DeviceIdentityRow {
                device_id: device_id.to_string(),
                schema_version: i64::from(REPLICATION_SCHEMA_VERSION),
                ed25519_public: ed_pub.to_vec(),
                x25519_public: x_pub.to_vec(),
                display_name: Some("local".to_string()),
                status: "local".to_string(),
                enrolled_at: CREATED_AT.to_string(),
                revoked_at: None,
                enrolled_by_device_id: device_id.to_string(),
                fingerprint_sha256: fp.to_vec(),
            },
            private_key,
            signed_control: SignedControlRow {
                event_id: built.signed.outer.event_id.to_string(),
                envelope_id: built.signed.outer.envelope_id.as_uuid().to_string(),
                sender_device_id: device_id.to_string(),
                content_type_code: ContentTypeCode::DeviceEnrolled.as_u16() as i64,
                body: built.body,
                signature: built.signed.signature.to_vec(),
                schema_version: i64::from(REPLICATION_SCHEMA_VERSION),
                local_seq: 1,
                created_at: CREATED_AT.to_string(),
            },
            envelope_index: EnvelopeIndexRow {
                envelope_id: built.signed.outer.envelope_id.as_uuid().to_string(),
                event_id: built.signed.outer.event_id.to_string(),
                sender_device_id: device_id.to_string(),
                local_seq: 1,
                content_type_code: ContentTypeCode::DeviceEnrolled.as_u16() as i64,
                content_key_id: Some(Uuid::nil().to_string()),
                body_len: built.signed.outer.ciphertext.len() as i64,
                padding_bucket: None,
                applied_at: Some(CREATED_AT.to_string()),
            },
        };

        {
            let mut c = conn.lock().unwrap();
            replication::bootstrap_local_device(&mut c, &input).unwrap();
        }

        let signing_key = keys.signing_key();
        Self {
            _tmp: tmp,
            data_key,
            conn,
            device_id,
            signing_key,
        }
    }

    pub fn enrollment_package_bytes(&self) -> Vec<u8> {
        let c = self.conn.lock().unwrap();
        let row = replication::get_device(&c, &self.device_id.to_string())
            .unwrap()
            .unwrap();
        let ed: [u8; 32] = row.ed25519_public.as_slice().try_into().unwrap();
        let x: [u8; 32] = row.x25519_public.as_slice().try_into().unwrap();
        enrollment_package(&self.device_id, &ed, &x)
    }

    /// OOB peer enroll: identity only (no shared envelope) so L8 pre-verify works.
    pub fn enroll_peer_oob(&self, peer_package: &[u8]) {
        let parsed = ai_brains_sync::parse_enrollment_package(peer_package).unwrap();
        let fp = fingerprint_sha256(peer_package);
        let c = self.conn.lock().unwrap();
        if replication::get_device(&c, &parsed.device_id.to_string())
            .unwrap()
            .is_some()
        {
            return;
        }
        replication::insert_device_identity(
            &c,
            &DeviceIdentityRow {
                device_id: parsed.device_id.to_string(),
                schema_version: i64::from(parsed.schema_version),
                ed25519_public: parsed.ed25519_pub.to_vec(),
                x25519_public: parsed.x25519_pub.to_vec(),
                display_name: None,
                status: "active".to_string(),
                enrolled_at: CREATED_AT.to_string(),
                revoked_at: None,
                enrolled_by_device_id: self.device_id.to_string(),
                fingerprint_sha256: fp.to_vec(),
            },
        )
        .unwrap();
    }

    /// Event ids excluding this vault's own bootstrap DeviceEnrolled (local-only).
    pub fn shared_event_ids(&self) -> HashSet<String> {
        let c = self.conn.lock().unwrap();
        let local = self.device_id.to_string();
        let mut out = HashSet::new();
        let mut stmt = c
            .prepare(
                "SELECT event_id, sender_device_id, local_seq, content_type_code
                 FROM encrypted_envelope_index",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .unwrap();
        for r in rows {
            let (eid, sender, seq, ctype) = r.unwrap();
            // Skip local bootstrap (sender=local, seq=1, DeviceEnrolled).
            if sender == local
                && seq == 1
                && ctype == ContentTypeCode::DeviceEnrolled.as_u16() as i64
            {
                continue;
            }
            out.insert(eid);
        }
        out
    }

    pub fn all_event_ids(&self) -> HashSet<String> {
        let c = self.conn.lock().unwrap();
        replication::list_envelope_event_ids(&c)
            .unwrap()
            .into_iter()
            .collect()
    }

    pub fn with_engine<R: RelayPort, T>(
        &self,
        relay: Arc<R>,
        f: impl FnOnce(&mut ReplicateEngine<'_, R>) -> T,
    ) -> T {
        let guard = self.conn.lock().unwrap();
        let mut engine = ReplicateEngine::new(&guard, relay, self.data_key.clone(), self.device_id);
        f(&mut engine)
    }

    pub fn cursor_state(&self, peer: &DeviceId) -> Option<String> {
        let c = self.conn.lock().unwrap();
        replication::get_cursor(&c, &peer.to_string())
            .unwrap()
            .map(|r| r.state)
    }

    /// Seed peer cursor at high_water=1 / expected=2 (post-OOB first shared seq).
    pub fn seed_cursor_for_peer(&self, peer: &DeviceId) {
        let c = self.conn.lock().unwrap();
        replication::set_cursor(
            &c,
            &replication::ReplicationCursorRow {
                peer_device_id: peer.to_string(),
                high_water_seq: 1,
                expected_local_seq: 2,
                state: "in_sync".to_string(),
                updated_at: CREATED_AT.to_string(),
            },
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// TwinVaults
// ---------------------------------------------------------------------------

pub struct TwinVaults {
    pub a: TestVault,
    pub b: TestVault,
    pub relay: Arc<MemoryFakeRelay>,
}

impl TwinVaults {
    pub fn new_enrolled_pair() -> Self {
        let a = TestVault::bootstrap();
        let b = TestVault::bootstrap();
        let pkg_a = a.enrollment_package_bytes();
        let pkg_b = b.enrollment_package_bytes();
        a.enroll_peer_oob(&pkg_b);
        b.enroll_peer_oob(&pkg_a);
        Self {
            a,
            b,
            relay: Arc::new(MemoryFakeRelay::new()),
        }
    }

    pub fn sync_round_both(&self) {
        self.a
            .with_engine(self.relay.clone(), |e| e.sync_round().unwrap());
        self.b
            .with_engine(self.relay.clone(), |e| e.sync_round().unwrap());
    }

    /// Seal on A and push in the **same** engine session (pending is in-memory).
    pub fn seal_and_push_a(
        &self,
        plaintext: &[u8],
        content_key_id: ContentKeyId,
    ) -> ai_brains_sync::SignedEnvelope {
        self.a.with_engine(self.relay.clone(), |e| {
            let s = e
                .seal_and_queue_data(plaintext, content_key_id, &[self.b.device_id])
                .unwrap();
            e.push_pending().unwrap();
            s
        })
    }

    pub fn seal_and_push_b(
        &self,
        plaintext: &[u8],
        content_key_id: ContentKeyId,
    ) -> ai_brains_sync::SignedEnvelope {
        self.b.with_engine(self.relay.clone(), |e| {
            let s = e
                .seal_and_queue_data(plaintext, content_key_id, &[self.a.device_id])
                .unwrap();
            e.push_pending().unwrap();
            s
        })
    }

    /// F4: shared (non-bootstrap-local) event_id sets equal.
    pub fn assert_converged(&self) {
        let sa = self.a.shared_event_ids();
        let sb = self.b.shared_event_ids();
        assert_eq!(sa, sb, "convergence failed\n  A={sa:?}\n  B={sb:?}");
    }

    /// After mutual OOB, seed expected=2 on both sides (bootstrap seq=1 not on relay).
    pub fn seed_peer_cursors(&self) {
        for (vault, peer) in [(&self.a, self.b.device_id), (&self.b, self.a.device_id)] {
            vault.seed_cursor_for_peer(&peer);
        }
    }
}

/// Enrolled pair with peer cursors ready for first shared envelope at seq≥2.
pub fn twins_ready() -> TwinVaults {
    let t = TwinVaults::new_enrolled_pair();
    t.seed_peer_cursors();
    t
}

// ---------------------------------------------------------------------------
// TripleVaults (F23 optional 3-device residual)
// ---------------------------------------------------------------------------

/// Three mutually OOB-enrolled vaults sharing one MemoryFakeRelay.
pub struct TripleVaults {
    pub a: TestVault,
    pub b: TestVault,
    pub c: TestVault,
    pub relay: Arc<MemoryFakeRelay>,
}

impl TripleVaults {
    pub fn new_enrolled() -> Self {
        let a = TestVault::bootstrap();
        let b = TestVault::bootstrap();
        let c = TestVault::bootstrap();
        let pkg_a = a.enrollment_package_bytes();
        let pkg_b = b.enrollment_package_bytes();
        let pkg_c = c.enrollment_package_bytes();
        a.enroll_peer_oob(&pkg_b);
        a.enroll_peer_oob(&pkg_c);
        b.enroll_peer_oob(&pkg_a);
        b.enroll_peer_oob(&pkg_c);
        c.enroll_peer_oob(&pkg_a);
        c.enroll_peer_oob(&pkg_b);
        Self {
            a,
            b,
            c,
            relay: Arc::new(MemoryFakeRelay::new()),
        }
    }

    pub fn seed_peer_cursors(&self) {
        self.a.seed_cursor_for_peer(&self.b.device_id);
        self.a.seed_cursor_for_peer(&self.c.device_id);
        self.b.seed_cursor_for_peer(&self.a.device_id);
        self.b.seed_cursor_for_peer(&self.c.device_id);
        self.c.seed_cursor_for_peer(&self.a.device_id);
        self.c.seed_cursor_for_peer(&self.b.device_id);
    }
}

/// Three-device residual helper (F23).
pub fn triple_enrolled() -> TripleVaults {
    let t = TripleVaults::new_enrolled();
    t.seed_peer_cursors();
    t
}
