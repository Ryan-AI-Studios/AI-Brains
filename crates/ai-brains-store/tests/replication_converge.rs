#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T177 — TwinVaults harness + convergence scenario matrix (C1–C15).

use ai_brains_core::ids::{ContentKeyId, DeviceId, MemoryId, ReplicationEventId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, DeviceRevokedPayload as EventDeviceRevokedPayload, MemoryPinnedPayload,
    Payload,
};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::projections::content_envelope;
use ai_brains_store::projections::replication::{
    self, BootstrapLocalDeviceInput, DeviceIdentityRow, DevicePrivateKeyRow, EnvelopeIndexRow,
    SignedControlRow,
};
use ai_brains_store::replication_engine::{
    EngineError, ReplicateEngine, sign_and_queue_erasure_tombstone, sign_and_queue_revoke,
    signed_to_blob,
};
use ai_brains_sync::{
    AdversarialRelay, ContentTypeCode, ControlPayload, DeviceEnrolledPayload, DevicePrivateSeeds,
    EnvelopeId, ErasureAckPayload, GapSkipAuditPayload, MemoryFakeRelay,
    REPLICATION_SCHEMA_VERSION, RelayPort, SyncError, build_and_sign_control, enrollment_package,
    fingerprint_sha256, generate_device_keys, nil_content_key_id, seal_device_private_blob,
    sign_envelope, verify_envelope,
};
use ed25519_dalek::SigningKey;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Build a domain MemoryPinned Envelope serialized as JSON (production seal body).
fn domain_pin_plaintext(content: &str) -> (Uuid, Vec<u8>) {
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

fn domain_event_count(vault: &TestVault, event_id: Uuid) -> i64 {
    let c = vault.conn.lock().unwrap();
    c.query_row(
        "SELECT COUNT(*) FROM events WHERE event_id = ?",
        [event_id.to_string()],
        |r| r.get(0),
    )
    .unwrap()
}

const CREATED_AT: &str = "2026-07-31T12:00:00Z";

// ---------------------------------------------------------------------------
// Test vault
// ---------------------------------------------------------------------------

struct TestVault {
    _tmp: TempDir,
    data_key: DataKey,
    conn: VaultConnection,
    device_id: DeviceId,
    /// Keep signing seed available for adversarial / collision tests.
    signing_key: SigningKey,
}

impl TestVault {
    fn bootstrap() -> Self {
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

    fn enrollment_package_bytes(&self) -> Vec<u8> {
        let c = self.conn.lock().unwrap();
        let row = replication::get_device(&c, &self.device_id.to_string())
            .unwrap()
            .unwrap();
        let ed: [u8; 32] = row.ed25519_public.as_slice().try_into().unwrap();
        let x: [u8; 32] = row.x25519_public.as_slice().try_into().unwrap();
        enrollment_package(&self.device_id, &ed, &x)
    }

    /// OOB peer enroll: identity only (no shared envelope) so L8 pre-verify works.
    fn enroll_peer_oob(&self, peer_package: &[u8]) {
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
    fn shared_event_ids(&self) -> HashSet<String> {
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

    fn all_event_ids(&self) -> HashSet<String> {
        let c = self.conn.lock().unwrap();
        replication::list_envelope_event_ids(&c)
            .unwrap()
            .into_iter()
            .collect()
    }

    fn with_engine<R: RelayPort, T>(
        &self,
        relay: Arc<R>,
        f: impl FnOnce(&mut ReplicateEngine<'_, R>) -> T,
    ) -> T {
        let guard = self.conn.lock().unwrap();
        let mut engine = ReplicateEngine::new(&guard, relay, self.data_key.clone(), self.device_id);
        f(&mut engine)
    }

    fn cursor_state(&self, peer: &DeviceId) -> Option<String> {
        let c = self.conn.lock().unwrap();
        replication::get_cursor(&c, &peer.to_string())
            .unwrap()
            .map(|r| r.state)
    }
}

// ---------------------------------------------------------------------------
// TwinVaults
// ---------------------------------------------------------------------------

struct TwinVaults {
    a: TestVault,
    b: TestVault,
    relay: Arc<MemoryFakeRelay>,
}

impl TwinVaults {
    fn new_enrolled_pair() -> Self {
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

    fn sync_round_both(&self) {
        self.a
            .with_engine(self.relay.clone(), |e| e.sync_round().unwrap());
        self.b
            .with_engine(self.relay.clone(), |e| e.sync_round().unwrap());
    }

    /// Seal on `side` and push in the **same** engine session (pending is in-memory).
    fn seal_and_push_a(
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

    fn seal_and_push_b(
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
    fn assert_converged(&self) {
        let sa = self.a.shared_event_ids();
        let sb = self.b.shared_event_ids();
        assert_eq!(sa, sb, "convergence failed\n  A={sa:?}\n  B={sb:?}");
    }
}

// ---------------------------------------------------------------------------
// Engine apply unit scenarios (C7, C8, C13, C15, gap)
// ---------------------------------------------------------------------------

#[test]
fn engine_apply__unknown_device__reject_preverify() {
    let vault = TestVault::bootstrap();
    let relay = Arc::new(MemoryFakeRelay::new());
    let unknown = generate_device_keys().unwrap();
    let unknown_id = DeviceId::new();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: unknown_id,
            ed25519_pub: unknown.verifying_key().to_bytes(),
            x25519_pub: unknown.x25519_public().to_bytes(),
        }),
        unknown_id,
        1,
        &unknown.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let blob = signed_to_blob(&built.signed).unwrap();

    let err = vault.with_engine(relay, |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::NotEnrolled) => {}
        other => panic!("expected NotEnrolled, got {other}"),
    }
}

#[test]
fn engine_apply__revoked_device__reject_preverify() {
    let vault = TestVault::bootstrap();
    let peer_keys = generate_device_keys().unwrap();
    let peer_id = DeviceId::new();
    let ed = peer_keys.verifying_key().to_bytes();
    let x = peer_keys.x25519_public().to_bytes();
    let package = enrollment_package(&peer_id, &ed, &x);
    let fp = fingerprint_sha256(&package);

    {
        let c = vault.conn.lock().unwrap();
        replication::insert_device_identity(
            &c,
            &DeviceIdentityRow {
                device_id: peer_id.to_string(),
                schema_version: 1,
                ed25519_public: ed.to_vec(),
                x25519_public: x.to_vec(),
                display_name: None,
                status: "active".to_string(),
                enrolled_at: CREATED_AT.to_string(),
                revoked_at: None,
                enrolled_by_device_id: vault.device_id.to_string(),
                fingerprint_sha256: fp.to_vec(),
            },
        )
        .unwrap();
        replication::tombstone_device(&c, &peer_id.to_string(), CREATED_AT, "test").unwrap();
    }

    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: peer_id,
            ed25519_pub: ed,
            x25519_pub: x,
        }),
        peer_id,
        2,
        &peer_keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let blob = signed_to_blob(&built.signed).unwrap();
    let relay = Arc::new(MemoryFakeRelay::new());
    let err = vault.with_engine(relay, |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::DeviceRevoked) => {}
        other => panic!("expected DeviceRevoked, got {other}"),
    }
}

#[test]
fn engine_apply__enroll_bad_signer__reject() {
    // Unknown signer trying to enroll someone → NotEnrolled pre-verify.
    let vault = TestVault::bootstrap();
    let bad = generate_device_keys().unwrap();
    let bad_id = DeviceId::new();
    let victim = DeviceId::new();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: victim,
            ed25519_pub: bad.verifying_key().to_bytes(),
            x25519_pub: bad.x25519_public().to_bytes(),
        }),
        bad_id,
        1,
        &bad.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let blob = signed_to_blob(&built.signed).unwrap();
    let relay = Arc::new(MemoryFakeRelay::new());
    let err = vault.with_engine(relay, |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::NotEnrolled) => {}
        other => panic!("expected NotEnrolled for bad signer, got {other}"),
    }
}

#[test]
fn engine_apply__schema_version_unknown__reject() {
    let twins = TwinVaults::new_enrolled_pair();
    // Craft control with bad schema_version signed by A (enrolled on B).
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: DeviceId::new(),
            ed25519_pub: [9u8; 32],
            x25519_pub: [8u8; 32],
        }),
        twins.a.device_id,
        2,
        &twins.a.signing_key,
        nil_content_key_id(),
    )
    .unwrap();
    // Mutate schema_version after sign — signature will fail first.
    // Instead build OuterEnvelope with schema 99 and sign.
    let mut outer = built.signed.outer.clone();
    outer.schema_version = 99;
    outer.local_seq = 2;
    outer.envelope_id = EnvelopeId::new();
    outer.event_id = ReplicationEventId::new();
    let signed = sign_envelope(&outer, &twins.a.signing_key).unwrap();
    let blob = signed_to_blob(&signed).unwrap();

    let err = twins
        .b
        .with_engine(twins.relay.clone(), |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::SchemaVersionMismatch {
            got: 99,
            expected: 1,
        }) => {}
        other => panic!("expected SchemaVersionMismatch, got {other}"),
    }
}

#[test]
fn engine_apply__seq_collision_diff_event__blocked() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    // A seals data seq=2 and pushes in-session.
    let signed1 = twins.seal_and_push_a(b"one", ck);
    assert_eq!(signed1.outer.local_seq, 2);
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert!(
        twins
            .b
            .all_event_ids()
            .contains(&signed1.outer.event_id.to_string())
    );

    // Craft different event_id at same (A, seq=2).
    let mut outer = signed1.outer.clone();
    outer.event_id = ReplicationEventId::new();
    outer.envelope_id = EnvelopeId::new();
    // Re-use ciphertext/wraps; re-sign.
    let forged = sign_envelope(&outer, &twins.a.signing_key).unwrap();
    let blob = signed_to_blob(&forged).unwrap();

    let err = twins
        .b
        .with_engine(twins.relay.clone(), |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::SeqCollision) => {}
        other => panic!("expected SeqCollision, got {other}"),
    }
    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("blocked")
    );
}

// ---------------------------------------------------------------------------
// Seed peer cursor at expected=2 after OOB (local bootstrap seq=1 is not replicated)
// ---------------------------------------------------------------------------

impl TwinVaults {
    fn seed_peer_cursors(&self) {
        // After mutual OOB, both vaults know peers but peer bootstrap envelopes are not
        // on the relay. First shared envelope will be local_seq>=2. Seed expected=2.
        for (vault, peer) in [(&self.a, self.b.device_id), (&self.b, self.a.device_id)] {
            let c = vault.conn.lock().unwrap();
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
}

fn twins_ready() -> TwinVaults {
    let t = TwinVaults::new_enrolled_pair();
    t.seed_peer_cursors();
    t
}

#[test]
fn gap_drain__discontiguous_missing__ordered_apply() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    // Seal three envelopes in one session (indexes advance); put selectively.
    let (s2, s3, s4) = twins.a.with_engine(twins.relay.clone(), |e| {
        let s2 = e
            .seal_and_queue_data(b"s2", ck, &[twins.b.device_id])
            .unwrap();
        let s3 = e
            .seal_and_queue_data(b"s3", ck, &[twins.b.device_id])
            .unwrap();
        let s4 = e
            .seal_and_queue_data(b"s4", ck, &[twins.b.device_id])
            .unwrap();
        // Leave pending un-pushed — selective put below via signed_to_blob.
        let _ = e.pending_len();
        (s2, s3, s4)
    });
    assert_eq!(
        (s2.outer.local_seq, s3.outer.local_seq, s4.outer.local_seq),
        (2, 3, 4)
    );

    // Put 4 then 3 (missing 2) — discontiguous.
    twins.relay.put(&signed_to_blob(&s4).unwrap()).unwrap();
    twins.relay.put(&signed_to_blob(&s3).unwrap()).unwrap();

    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("sync_gap")
    );
    // 3 and 4 buffered, not applied yet.
    assert!(
        !twins
            .b
            .shared_event_ids()
            .contains(&s3.outer.event_id.to_string())
    );
    assert!(
        !twins
            .b
            .shared_event_ids()
            .contains(&s4.outer.event_id.to_string())
    );

    // Fill gap with seq 2.
    twins.relay.put(&signed_to_blob(&s2).unwrap()).unwrap();
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());

    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("in_sync")
    );
    let b_ids = twins.b.shared_event_ids();
    assert!(b_ids.contains(&s2.outer.event_id.to_string()));
    assert!(b_ids.contains(&s3.outer.event_id.to_string()));
    assert!(b_ids.contains(&s4.outer.event_id.to_string()));
}

// ---------------------------------------------------------------------------
// Convergence matrix
// ---------------------------------------------------------------------------

#[test]
fn converge__happy_path_wrap__event_id_match() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    // Production seal body: real domain Envelope JSON (spec step 5b).
    let (domain_event_id, plaintext) = domain_pin_plaintext("hello-peer");
    let signed = twins.seal_and_push_a(&plaintext, ck);
    twins.sync_round_both();

    assert!(
        twins
            .b
            .shared_event_ids()
            .contains(&signed.outer.event_id.to_string())
    );
    twins.assert_converged();

    // B has peer wrap for self and can re-unwrap (project_data already opened DEK).
    let c = twins.b.conn.lock().unwrap();
    let wrap =
        replication::get_peer_wrap(&c, &ck.to_string(), &twins.b.device_id.to_string()).unwrap();
    assert!(wrap.is_some(), "B must store peer wrap for local recipient");
    drop(c);

    // Domain event must land on vault B after open + append (not only envelope index).
    assert_eq!(
        domain_event_count(&twins.b, domain_event_id),
        1,
        "B must append domain Envelope after open"
    );
}

#[test]
fn project_data__domain_envelope__appended_on_peer() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let (domain_event_id, plaintext) = domain_pin_plaintext("domain-on-peer");
    let _signed = twins.seal_and_push_a(&plaintext, ck);
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());

    assert_eq!(domain_event_count(&twins.b, domain_event_id), 1);
    // Opaque (non-Envelope) body still applies wraps without failing.
    let ck2 = ContentKeyId::new();
    let opaque = twins.seal_and_push_a(b"not-json-envelope", ck2);
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert!(
        twins
            .b
            .shared_event_ids()
            .contains(&opaque.outer.event_id.to_string())
    );
}

/// P1: DeviceRevoked (membership) must not be accepted via DataEvent body.
///
/// An enrolled peer seals a domain Envelope whose payload is DeviceRevoked as a
/// DataEvent. Peer apply must refuse append; target device stays non-revoked;
/// no wrap/index durable side effects from the smuggled envelope.
#[test]
fn project_data__smuggled_device_revoked__reject() {
    let twins = twins_ready();
    let target = twins.b.device_id;
    let by = twins.a.device_id;

    // Domain Envelope with DeviceRevoked payload (would project tombstone if appended).
    let domain = EventBuilder::new(
        AggregateType::System,
        target.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceRevoked(EventDeviceRevokedPayload {
        device_id: target,
        revoked_by_device_id: by,
        reason_code: "smuggled-via-data".into(),
        replication_event_id: ReplicationEventId::new(),
        local_seq: 99,
        envelope_id: Uuid::new_v4(),
        signature_hex: "00".repeat(64),
        body_hex: "deadbeef".into(),
        content_type_code: ContentTypeCode::DeviceRevoked.as_u16(),
    }))
    .expect("build smuggled DeviceRevoked domain envelope");
    let domain_event_id = domain.event_id;
    let plaintext = serde_json::to_vec(&domain).expect("serialize smuggled envelope");

    let ck = ContentKeyId::new();
    let signed = twins.seal_and_push_a(&plaintext, ck);
    let blob = signed_to_blob(&signed).expect("blob");

    let err = twins
        .b
        .with_engine(twins.relay.clone(), |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::InvalidEncoding(msg)) => {
            assert!(
                msg.contains("membership") || msg.contains("DataEvent") || msg.contains("0x0011"),
                "expected membership smuggle rejection, got: {msg}"
            );
        }
        other => panic!("expected InvalidEncoding membership reject, got {other}"),
    }

    // Device B must remain enrolled/active (not revoked via data path).
    {
        let c = twins.b.conn.lock().unwrap();
        let row = replication::get_device(&c, &target.to_string())
            .unwrap()
            .expect("B identity present");
        assert_ne!(
            row.status, "revoked",
            "smuggled DeviceRevoked must not tombstone via data path"
        );
    }
    // Domain event must not land; smuggled envelope must not be indexed on B.
    assert_eq!(
        domain_event_count(&twins.b, domain_event_id),
        0,
        "smuggled membership must not append domain event"
    );
    assert!(
        !twins
            .b
            .shared_event_ids()
            .contains(&signed.outer.event_id.to_string()),
        "rejected data envelope must not enter envelope index"
    );
    // No peer wrap stored for this content key on B (fail closed before wraps).
    {
        let c = twins.b.conn.lock().unwrap();
        let wrap = replication::get_peer_wrap(&c, &ck.to_string(), &twins.b.device_id.to_string())
            .unwrap();
        assert!(
            wrap.is_none(),
            "reject must not store wraps for smuggled membership data"
        );
    }
}

#[test]
fn gap_skip_audit__missing_seq__advances_and_drains() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let (s2, s3, s4) = twins.a.with_engine(twins.relay.clone(), |e| {
        let s2 = e
            .seal_and_queue_data(b"s2", ck, &[twins.b.device_id])
            .unwrap();
        let s3 = e
            .seal_and_queue_data(b"s3", ck, &[twins.b.device_id])
            .unwrap();
        let s4 = e
            .seal_and_queue_data(b"s4", ck, &[twins.b.device_id])
            .unwrap();
        (s2, s3, s4)
    });
    assert_eq!(
        (s2.outer.local_seq, s3.outer.local_seq, s4.outer.local_seq),
        (2, 3, 4)
    );

    // Permanent loss of seq 2: put 3 then 4 only → sync_gap at expected=2.
    twins.relay.put(&signed_to_blob(&s3).unwrap()).unwrap();
    twins.relay.put(&signed_to_blob(&s4).unwrap()).unwrap();
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("sync_gap")
    );
    {
        let c = twins.b.conn.lock().unwrap();
        let cur = replication::get_cursor(&c, &twins.a.device_id.to_string())
            .unwrap()
            .expect("cursor");
        assert_eq!(cur.expected_local_seq, 2);
    }

    // GapSkipAudit occupies the missing seq slot (local_seq == skipped_seq == 2).
    let audit = build_and_sign_control(
        ContentTypeCode::GapSkipAudit,
        &ControlPayload::GapSkipAudit(GapSkipAuditPayload {
            peer_device_id: twins.a.device_id,
            skipped_seq: 2,
            reason: "permanent-loss-test".to_string(),
        }),
        twins.a.device_id,
        2,
        &twins.a.signing_key,
        nil_content_key_id(),
    )
    .unwrap();
    let blob = signed_to_blob(&audit.signed).unwrap();
    twins.b.with_engine(twins.relay.clone(), |e| {
        e.apply_blob(&blob).unwrap();
    });

    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("in_sync"),
        "skip + drain of buffered 3/4 must recover in_sync"
    );
    {
        let c = twins.b.conn.lock().unwrap();
        let cur = replication::get_cursor(&c, &twins.a.device_id.to_string())
            .unwrap()
            .expect("cursor");
        assert_eq!(
            cur.expected_local_seq, 5,
            "after skip(2)+apply(3)+apply(4) expected becomes 5"
        );
    }
    let b_ids = twins.b.shared_event_ids();
    assert!(b_ids.contains(&s3.outer.event_id.to_string()));
    assert!(b_ids.contains(&s4.outer.event_id.to_string()));
    assert!(b_ids.contains(&audit.signed.outer.event_id.to_string()));
    // Missing data envelope never applied.
    assert!(!b_ids.contains(&s2.outer.event_id.to_string()));
}

#[test]
fn converge__offline_diverge__event_id_union() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let a_ev = twins.seal_and_push_a(b"from-a", ck);
    let b_ev = twins.seal_and_push_b(b"from-b", ck);
    // Offline diverge before mutual pull: each only has own event.
    assert!(
        twins
            .a
            .shared_event_ids()
            .contains(&a_ev.outer.event_id.to_string())
    );
    assert!(
        !twins
            .a
            .shared_event_ids()
            .contains(&b_ev.outer.event_id.to_string())
    );

    twins
        .a
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    twins.assert_converged();
    let ids = twins.a.shared_event_ids();
    assert!(ids.contains(&a_ev.outer.event_id.to_string()));
    assert!(ids.contains(&b_ev.outer.event_id.to_string()));
}

#[test]
fn converge__duplicate_push__single_apply() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let signed = twins.a.with_engine(twins.relay.clone(), |e| {
        e.seal_and_queue_data(b"dup", ck, &[twins.b.device_id])
            .unwrap()
    });
    let blob = signed_to_blob(&signed).unwrap();
    twins.relay.put(&blob).unwrap();
    twins.relay.put(&blob).unwrap(); // idempotent put
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());

    let c = twins.b.conn.lock().unwrap();
    let count: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM encrypted_envelope_index WHERE event_id = ?",
            [signed.outer.event_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn converge__reorder_pull__gap_then_fill() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let (s2, s3) = twins.a.with_engine(twins.relay.clone(), |e| {
        let s2 = e
            .seal_and_queue_data(b"r2", ck, &[twins.b.device_id])
            .unwrap();
        let s3 = e
            .seal_and_queue_data(b"r3", ck, &[twins.b.device_id])
            .unwrap();
        (s2, s3)
    });
    // Deliver 3 before 2.
    twins.relay.put(&signed_to_blob(&s3).unwrap()).unwrap();
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("sync_gap")
    );
    twins.relay.put(&signed_to_blob(&s2).unwrap()).unwrap();
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(
        twins.b.cursor_state(&twins.a.device_id).as_deref(),
        Some("in_sync")
    );
    let ids = twins.b.shared_event_ids();
    assert!(ids.contains(&s2.outer.event_id.to_string()));
    assert!(ids.contains(&s3.outer.event_id.to_string()));
}

#[test]
fn converge__delay_seq__repush_restore() {
    let a = TestVault::bootstrap();
    let b = TestVault::bootstrap();
    a.enroll_peer_oob(&b.enrollment_package_bytes());
    b.enroll_peer_oob(&a.enrollment_package_bytes());
    // Seed cursors.
    for (vault, peer) in [(&a, b.device_id), (&b, a.device_id)] {
        let c = vault.conn.lock().unwrap();
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

    let inner = MemoryFakeRelay::new();
    let adv = Arc::new(AdversarialRelay::new(inner));
    let ck = ContentKeyId::new();

    let (s2, s3) = a.with_engine(adv.clone(), |e| {
        let s2 = e.seal_and_queue_data(b"d2", ck, &[b.device_id]).unwrap();
        let s3 = e.seal_and_queue_data(b"d3", ck, &[b.device_id]).unwrap();
        e.push_pending().unwrap();
        (s2, s3)
    });

    // Delay middle seq (2) — C5: delay not delete. (delay after put hides from pull)
    adv.delay_seq(&a.device_id, 2).unwrap();

    b.with_engine(adv.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(b.cursor_state(&a.device_id).as_deref(), Some("sync_gap"));
    assert!(
        !b.shared_event_ids()
            .contains(&s2.outer.event_id.to_string())
    );

    // Restore: clear delay + sender re-push (idempotent).
    adv.clear_delay(&a.device_id, 2).unwrap();
    adv.put(&signed_to_blob(&s2).unwrap()).unwrap();

    b.with_engine(adv.clone(), |e| e.pull_all_peers().unwrap());
    assert_eq!(b.cursor_state(&a.device_id).as_deref(), Some("in_sync"));
    assert!(
        b.shared_event_ids()
            .contains(&s2.outer.event_id.to_string())
    );
    assert!(
        b.shared_event_ids()
            .contains(&s3.outer.event_id.to_string())
    );
}

#[test]
fn converge__erasure_tombstone__ack_acked() {
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    // Insert a local CE wrap on both so destroy is meaningful.
    for v in [&twins.a, &twins.b] {
        let c = v.conn.lock().unwrap();
        content_envelope::insert_content_key_wrap(
            &c,
            &ck.to_string(),
            1,
            &[1u8; 12],
            &[2u8; 48],
            CREATED_AT,
        )
        .unwrap();
    }

    // A pushes tombstone → B syncs (applies CE destroy + queues ErasureAck) →
    // B pushes ACK → A pulls and applies.
    let tomb = twins.a.with_engine(twins.relay.clone(), |e| {
        let t = sign_and_queue_erasure_tombstone(e, ck, "user-erase").unwrap();
        e.push_pending().unwrap();
        t
    });
    twins.b.with_engine(twins.relay.clone(), |e| {
        e.pull_all_peers().unwrap();
        e.push_pending().unwrap(); // ErasureAck queued on tombstone apply
    });
    twins
        .a
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());

    // 1) B content key destroyed.
    {
        let c = twins.b.conn.lock().unwrap();
        let row = content_envelope::get_content_key_wrap(&c, &ck.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(row.status, "destroyed");
    }
    // 2) On A: ErasureAck from peer B is exactly "acked".
    {
        let c = twins.a.conn.lock().unwrap();
        let ack = replication::get_erasure_ack(
            &c,
            &tomb.outer.event_id.to_string(),
            &twins.b.device_id.to_string(),
        )
        .unwrap()
        .expect("A must have erasure_ack row for peer B after applying ErasureAck");
        assert_eq!(
            ack.status, "acked",
            "A must record B's ErasureAck as acked exactly"
        );
    }
    // 3) B still has a local row toward tombstone issuer A (pending or acked).
    {
        let c = twins.b.conn.lock().unwrap();
        let b_ack = replication::get_erasure_ack(
            &c,
            &tomb.outer.event_id.to_string(),
            &twins.a.device_id.to_string(),
        )
        .unwrap()
        .expect("B must keep local ack row toward tombstone issuer A");
        assert!(
            b_ack.status == "pending" || b_ack.status == "acked",
            "B local ack status unexpected: {}",
            b_ack.status
        );
    }
}

#[test]
fn engine_apply__erasure_ack_peer_mismatch__reject() {
    // Codex R4: ErasureAck.peer_device_id must equal authenticated outer sender.
    let vault = TestVault::bootstrap();
    let peer_keys = generate_device_keys().unwrap();
    let peer_id = DeviceId::new();
    let spoofed_peer = DeviceId::new();
    let ed = peer_keys.verifying_key().to_bytes();
    let x = peer_keys.x25519_public().to_bytes();
    let package = enrollment_package(&peer_id, &ed, &x);
    let fp = fingerprint_sha256(&package);
    {
        let c = vault.conn.lock().unwrap();
        replication::insert_device_identity(
            &c,
            &DeviceIdentityRow {
                device_id: peer_id.to_string(),
                schema_version: 1,
                ed25519_public: ed.to_vec(),
                x25519_public: x.to_vec(),
                display_name: None,
                status: "active".to_string(),
                enrolled_at: CREATED_AT.to_string(),
                revoked_at: None,
                enrolled_by_device_id: vault.device_id.to_string(),
                fingerprint_sha256: fp.to_vec(),
            },
        )
        .unwrap();
        // Seed expected cursor for peer seq 1.
        replication::set_cursor(
            &c,
            &replication::ReplicationCursorRow {
                peer_device_id: peer_id.to_string(),
                high_water_seq: 0,
                expected_local_seq: 1,
                state: "in_sync".to_string(),
                updated_at: CREATED_AT.to_string(),
            },
        )
        .unwrap();
    }

    let erasure_id = ReplicationEventId::new();
    let ck = ContentKeyId::new();
    let built = build_and_sign_control(
        ContentTypeCode::ErasureAck,
        &ControlPayload::ErasureAck(ErasureAckPayload {
            erasure_id,
            content_key_id: ck,
            // Spoof: claim another peer attested, while outer signer is peer_id.
            peer_device_id: spoofed_peer,
            status: "acked".to_string(),
        }),
        peer_id,
        1,
        &peer_keys.signing_key(),
        ck,
    )
    .unwrap();
    let blob = signed_to_blob(&built.signed).unwrap();
    let relay = Arc::new(MemoryFakeRelay::new());
    let err = vault.with_engine(relay, |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::InvalidEncoding(ref m))
            if m.contains("peer_device_id must match") => {}
        other => panic!("expected peer_device_id mismatch InvalidEncoding, got {other}"),
    }
    // Must not record spoofed peer as acked.
    let c = vault.conn.lock().unwrap();
    let spoofed =
        replication::get_erasure_ack(&c, &erasure_id.to_string(), &spoofed_peer.to_string())
            .unwrap();
    assert!(spoofed.is_none(), "spoofed peer must not be acked");
}

#[test]
fn converge__cursor_resume_limit_1__full_after_rounds() {
    // M1: pull_limit=1 requires multiple sync rounds to fully converge.
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let sealed = twins.a.with_engine(twins.relay.clone(), |e| {
        let mut out = Vec::new();
        for i in 0..3 {
            let plaintext = format!("limit1-{i}");
            let s = e
                .seal_and_queue_data(plaintext.as_bytes(), ck, &[twins.b.device_id])
                .unwrap();
            out.push(s);
        }
        e.push_pending().unwrap();
        out
    });
    assert!(sealed.len() >= 3);

    let mut converged = false;
    for _ in 0..16 {
        twins.a.with_engine(twins.relay.clone(), |e| {
            e.set_pull_limit(1);
            e.sync_round().unwrap();
        });
        twins.b.with_engine(twins.relay.clone(), |e| {
            e.set_pull_limit(1);
            e.sync_round().unwrap();
        });
        if twins.a.shared_event_ids() == twins.b.shared_event_ids() {
            let b_ids = twins.b.shared_event_ids();
            if sealed
                .iter()
                .all(|s| b_ids.contains(&s.outer.event_id.to_string()))
            {
                converged = true;
                break;
            }
        }
    }
    assert!(
        converged,
        "expected full convergence under pull_limit=1 within 16 rounds"
    );
    twins.assert_converged();
}

#[test]
fn outbox__engine_restart__push_still_sends() {
    // M2: durable outbox survives engine drop (process-restart simulation).
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let signed = twins.a.with_engine(twins.relay.clone(), |e| {
        e.seal_and_queue_data(b"durable-outbox", ck, &[twins.b.device_id])
            .unwrap()
    });
    // New engine session: in-memory pending is empty; outbox still has the row.
    let n = twins.a.with_engine(twins.relay.clone(), |e| {
        assert_eq!(e.pending_len(), 0, "fresh engine has empty memory pending");
        e.push_pending().unwrap()
    });
    assert_eq!(n, 1, "restart push must put the sealed envelope once");
    // Second push is empty (already marked pushed_at).
    let n2 = twins
        .a
        .with_engine(twins.relay.clone(), |e| e.push_pending().unwrap());
    assert_eq!(n2, 0);
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    assert!(
        twins
            .b
            .shared_event_ids()
            .contains(&signed.outer.event_id.to_string()),
        "B must apply the envelope that survived engine restart"
    );
}

#[test]
fn engine_apply__local_seq_behind_no_index__reject() {
    // L3: local_seq < expected with no index row → structured InvalidEncoding.
    let twins = twins_ready();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: twins.a.device_id,
            ed25519_pub: twins.a.signing_key.verifying_key().to_bytes(),
            x25519_pub: {
                // Re-read enrolled x25519 for A from B's identity table.
                let c = twins.b.conn.lock().unwrap();
                let row = replication::get_device(&c, &twins.a.device_id.to_string())
                    .unwrap()
                    .unwrap();
                row.x25519_public.as_slice().try_into().unwrap()
            },
        }),
        twins.a.device_id,
        1, // behind seeded expected=2
        &twins.a.signing_key,
        nil_content_key_id(),
    )
    .unwrap();
    let blob = signed_to_blob(&built.signed).unwrap();
    let err = twins
        .b
        .with_engine(twins.relay.clone(), |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::InvalidEncoding(msg)) => {
            assert!(
                msg.contains("local_seq") && msg.contains("expected"),
                "error should mention local_seq/expected: {msg}"
            );
        }
        other => panic!("expected InvalidEncoding for stale seq without index, got {other}"),
    }
}

#[test]
fn converge__concurrent_events__both_event_ids_present() {
    // C9 fallback: both event_ids present (no silent LWW drop).
    let twins = twins_ready();
    let ck = ContentKeyId::new();
    let e1 = twins.seal_and_push_a(b"claim-true", ck);
    let e2 = twins.seal_and_push_b(b"claim-false", ck);
    twins
        .a
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    twins
        .b
        .with_engine(twins.relay.clone(), |e| e.pull_all_peers().unwrap());
    let ids = twins.a.shared_event_ids();
    assert!(ids.contains(&e1.outer.event_id.to_string()));
    assert!(ids.contains(&e2.outer.event_id.to_string()));
    twins.assert_converged();
}

#[test]
fn ack_tick__three_cycles_no_ack__unreachable() {
    let vault = TestVault::bootstrap();
    {
        let c = vault.conn.lock().unwrap();
        replication::upsert_erasure_ack(
            &c,
            &replication::ErasureAckRow {
                erasure_id: "er-timeout".to_string(),
                peer_device_id: "peer-x".to_string(),
                content_key_id: "ck-x".to_string(),
                status: "pending".to_string(),
                sync_cycles_waiting: 0,
                updated_at: CREATED_AT.to_string(),
            },
        )
        .unwrap();
    }
    let relay = Arc::new(MemoryFakeRelay::new());
    for _ in 0..3 {
        vault.with_engine(relay.clone(), |e| e.sync_round().unwrap());
    }
    let c = vault.conn.lock().unwrap();
    let status: String = c
        .query_row(
            "SELECT status FROM erasure_ack_projection WHERE erasure_id = ?",
            ["er-timeout"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "unreachable");
}

#[test]
fn converge__revoke_then_reject__preverify() {
    let twins = twins_ready();
    // A revokes B locally and pushes DeviceRevoked.
    twins.a.with_engine(twins.relay.clone(), |e| {
        sign_and_queue_revoke(e, twins.b.device_id, "lost").unwrap();
        e.push_pending().unwrap();
    });
    // B still enrolled on its own vault; craft envelope from B to A after A revoked B.
    // First A should have revoked B in identity.
    {
        let c = twins.a.conn.lock().unwrap();
        let row = replication::get_device(&c, &twins.b.device_id.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(row.status, "revoked");
    }

    let ck = ContentKeyId::new();
    // B still has A enrolled; seal and push to relay.
    let signed = twins.b.with_engine(twins.relay.clone(), |e| {
        let s = e
            .seal_and_queue_data(b"after-revoke", ck, &[twins.a.device_id])
            .unwrap();
        e.push_pending().unwrap();
        s
    });
    let blob = signed_to_blob(&signed).unwrap();
    let err = twins
        .a
        .with_engine(twins.relay.clone(), |e| e.apply_blob(&blob).unwrap_err());
    match err {
        EngineError::Sync(SyncError::DeviceRevoked) => {}
        other => panic!("expected DeviceRevoked pre-verify, got {other}"),
    }
}
