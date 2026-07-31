#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T176 — migration 0027 replication schema + store APIs.

use ai_brains_core::ids::DeviceId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, DeviceEnrolledPayload as EventDeviceEnrolledPayload,
    DeviceRevokedPayload as EventDeviceRevokedPayload, Payload,
};
use ai_brains_store::EventStore;
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brains_store::projections::replication::{
    self, BootstrapLocalDeviceInput, DeviceIdentityRow, DevicePrivateKeyRow, EnvelopeIndexRow,
    PeerContentKeyWrapRow, SignedControlRow,
};
use ai_brains_sync::{
    ContentTypeCode, ControlPayload, DeviceEnrolledPayload, DeviceRevokedPayload,
    REPLICATION_SCHEMA_VERSION, build_and_sign_control, generate_device_keys, nil_content_key_id,
    verify_envelope,
};
use tempfile::NamedTempFile;

const CREATED_AT: &str = "2026-07-30T12:00:00Z";
const REVOKED_AT: &str = "2026-07-30T13:00:00Z";

fn open_store() -> (NamedTempFile, SqliteEventStore) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    (temp_file, SqliteEventStore::new(conn))
}

fn table_exists(store: &SqliteEventStore, name: &str) -> bool {
    let conn = store.connection().lock().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [name],
            |r| r.get(0),
        )
        .unwrap();
    count == 1
}

fn sample_identity(device_id: &str, status: &str, enrolled_by: &str) -> DeviceIdentityRow {
    DeviceIdentityRow {
        device_id: device_id.to_string(),
        schema_version: 1,
        ed25519_public: vec![0x11; 32],
        x25519_public: vec![0x22; 32],
        display_name: Some("test".to_string()),
        status: status.to_string(),
        enrolled_at: CREATED_AT.to_string(),
        revoked_at: None,
        enrolled_by_device_id: enrolled_by.to_string(),
        fingerprint_sha256: vec![0x33; 32],
    }
}

#[test]
fn migration_0027__fresh_vault__tables_exist() {
    let (_tmp, store) = open_store();
    for table in [
        "device_identity",
        "device_id_tombstone",
        "device_private_key_store",
        "peer_content_key_wrap",
        "encrypted_envelope_index",
        "signed_replication_control",
        "replication_cursor",
        "replication_gap_buffer",
        "erasure_ack_projection",
        "replication_gap_skip_audit",
        "replication_outbox",
    ] {
        assert!(table_exists(&store, table), "missing table {table}");
    }
    // No content_hash column (R29).
    let conn = store.connection().lock().unwrap();
    let mut stmt = conn
        .prepare("PRAGMA table_info(encrypted_envelope_index)")
        .unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert!(
        !cols.iter().any(|c| c == "content_hash_sha256"),
        "must not have content_hash_sha256"
    );
}

#[test]
fn migration_0027__after_0026__applies_forward() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0026_content_envelopes_erasure")).unwrap();
    }

    assert!(
        !table_exists_conn(&conn, "device_identity"),
        "0026-only vault must not yet have device_identity"
    );
    assert!(
        table_exists_conn(&conn, "content_key_store"),
        "0026 vault must have content_key_store"
    );

    conn.migrate().unwrap();

    for table in [
        "device_identity",
        "device_id_tombstone",
        "device_private_key_store",
        "peer_content_key_wrap",
        "encrypted_envelope_index",
        "signed_replication_control",
        "replication_cursor",
        "replication_gap_buffer",
        "erasure_ack_projection",
        "replication_gap_skip_audit",
        "replication_outbox",
    ] {
        assert!(
            table_exists_conn(&conn, table),
            "after full migrate, missing table {table}"
        );
    }

    let locked = conn.lock().unwrap();
    let applied: i64 = locked
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE name = '0027_replication_state'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(applied, 1, "0027 must be recorded in schema_migrations");
    let applied_outbox: i64 = locked
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE name = '0028_replication_outbox'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        applied_outbox, 1,
        "0028 must be recorded in schema_migrations"
    );
}

fn table_exists_conn(conn: &VaultConnection, name: &str) -> bool {
    let locked = conn.lock().unwrap();
    let count: i64 = locked
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [name],
            |r| r.get(0),
        )
        .unwrap();
    count == 1
}

#[test]
fn device_identity__bad_status__check_constraint_rejects() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let mut bad = sample_identity(
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        "not-a-status",
        "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    );
    bad.status = "zombie".to_string();
    let err = replication::insert_device_identity(&conn, &bad).expect_err("bad status");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("check") || msg.contains("constraint") || msg.contains("status"),
        "expected CHECK failure, got: {err}"
    );
}

#[test]
fn first_device__enrolled_by__self() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let id = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    replication::insert_device_identity(&conn, &sample_identity(id, "local", id)).unwrap();
    let got = replication::get_device(&conn, id).unwrap().unwrap();
    assert_eq!(got.status, "local");
    assert_eq!(got.enrolled_by_device_id, id);
    assert_eq!(got.device_id, got.enrolled_by_device_id);
}

#[test]
fn device_tombstone__re_enroll_same_id__rejected() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let id = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    replication::insert_device_identity(&conn, &sample_identity(id, "local", id)).unwrap();
    replication::tombstone_device(&conn, id, REVOKED_AT, "test-revoke").unwrap();
    let err = replication::insert_device_identity(&conn, &sample_identity(id, "active", id))
        .expect_err("tombstoned re-enroll");
    let msg = err.to_string();
    assert!(
        msg.contains("tombstoned") || msg.contains("permanently"),
        "got: {msg}"
    );
}

#[test]
fn peer_wrap_pk__upsert__replaces() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let row1 = PeerContentKeyWrapRow {
        content_key_id: "ck-1".to_string(),
        recipient_device_id: "dev-r".to_string(),
        sender_device_id: "dev-s1".to_string(),
        schema_version: 1,
        eph_x25519_public: vec![0x01; 32],
        wrap_nonce: vec![0x02; 12],
        wrap_ciphertext: vec![0x03; 48],
        created_at: CREATED_AT.to_string(),
    };
    replication::upsert_peer_content_key_wrap(&conn, &row1).unwrap();
    let mut row2 = row1.clone();
    row2.sender_device_id = "dev-s2".to_string();
    row2.eph_x25519_public = vec![0xFF; 32];
    replication::upsert_peer_content_key_wrap(&conn, &row2).unwrap();
    let got = replication::get_peer_wrap(&conn, "ck-1", "dev-r")
        .unwrap()
        .unwrap();
    assert_eq!(got.sender_device_id, "dev-s2");
    assert_eq!(got.eph_x25519_public, vec![0xFF; 32]);
}

#[test]
fn revoke__deletes_recipient_wraps() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let local = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    let peer = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
    replication::insert_device_identity(&conn, &sample_identity(local, "local", local)).unwrap();
    replication::insert_device_identity(&conn, &sample_identity(peer, "active", local)).unwrap();
    replication::upsert_peer_content_key_wrap(
        &conn,
        &PeerContentKeyWrapRow {
            content_key_id: "ck-x".to_string(),
            recipient_device_id: peer.to_string(),
            sender_device_id: local.to_string(),
            schema_version: 1,
            eph_x25519_public: vec![0x01; 32],
            wrap_nonce: vec![0x02; 12],
            wrap_ciphertext: vec![0x03; 48],
            created_at: CREATED_AT.to_string(),
        },
    )
    .unwrap();
    assert!(
        replication::get_peer_wrap(&conn, "ck-x", peer)
            .unwrap()
            .is_some()
    );
    replication::tombstone_device(&conn, peer, REVOKED_AT, "stolen").unwrap();
    assert!(
        replication::get_peer_wrap(&conn, "ck-x", peer)
            .unwrap()
            .is_none()
    );
}

#[test]
fn envelope_index__duplicate_event_id__idempotent() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let row = EnvelopeIndexRow {
        envelope_id: "env-1".to_string(),
        event_id: "evt-1".to_string(),
        sender_device_id: "dev-1".to_string(),
        local_seq: 1,
        content_type_code: 0x0010,
        content_key_id: Some("00000000-0000-0000-0000-000000000000".to_string()),
        body_len: 82,
        padding_bucket: None,
        applied_at: Some(CREATED_AT.to_string()),
    };
    replication::insert_envelope_index(&conn, &row).unwrap();
    // Same event_id, different envelope_id — idempotent no-op.
    let mut row2 = row.clone();
    row2.envelope_id = "env-2".to_string();
    replication::insert_envelope_index(&conn, &row2).unwrap();
    assert!(replication::envelope_exists(&conn, "evt-1").unwrap());
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM encrypted_envelope_index WHERE event_id = ?",
            ["evt-1"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn bootstrap_local_device__second_call__err() {
    let (_tmp, store) = open_store();
    let mut conn = store.connection().lock().unwrap();
    assert!(!replication::has_active_or_local_device(&conn).unwrap());

    let keys = generate_device_keys().unwrap();
    let device = DeviceId::new();
    let ed = keys.verifying_key().to_bytes();
    let x = keys.x25519_public().to_bytes();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: ed,
            x25519_pub: x,
        }),
        device,
        1,
        &keys.signing_key(),
        ai_brains_sync::nil_content_key_id(),
    )
    .unwrap();
    verify_envelope(&built.signed, &keys.verifying_key()).unwrap();

    let input = BootstrapLocalDeviceInput {
        identity: DeviceIdentityRow {
            device_id: device.to_string(),
            schema_version: 1,
            ed25519_public: ed.to_vec(),
            x25519_public: x.to_vec(),
            display_name: Some("local".to_string()),
            status: "local".to_string(),
            enrolled_at: CREATED_AT.to_string(),
            revoked_at: None,
            enrolled_by_device_id: device.to_string(),
            fingerprint_sha256: vec![0x33; 32],
        },
        private_key: DevicePrivateKeyRow {
            device_id: device.to_string(),
            wrap_schema_version: 1,
            algorithm: "AES-256-GCM".to_string(),
            protection: "datakey".to_string(),
            wrap_nonce: vec![0xAB; 12],
            wrap_ciphertext: vec![0xCD; 80],
            created_at: CREATED_AT.to_string(),
        },
        signed_control: SignedControlRow {
            event_id: built.signed.outer.event_id.to_string(),
            envelope_id: built.signed.outer.envelope_id.as_uuid().to_string(),
            sender_device_id: device.to_string(),
            content_type_code: CONTENT_TYPE_DEVICE_ENROLLED_I64,
            body: built.body.clone(),
            signature: built.signed.signature.to_vec(),
            schema_version: 1,
            local_seq: 1,
            created_at: CREATED_AT.to_string(),
        },
        envelope_index: EnvelopeIndexRow {
            envelope_id: built.signed.outer.envelope_id.as_uuid().to_string(),
            event_id: built.signed.outer.event_id.to_string(),
            sender_device_id: device.to_string(),
            local_seq: 1,
            content_type_code: CONTENT_TYPE_DEVICE_ENROLLED_I64,
            content_key_id: Some("00000000-0000-0000-0000-000000000000".to_string()),
            body_len: built.body.len() as i64,
            padding_bucket: None,
            applied_at: Some(CREATED_AT.to_string()),
        },
    };

    replication::bootstrap_local_device(&mut conn, &input).unwrap();
    assert!(replication::has_active_or_local_device(&conn).unwrap());

    // Stored control is verifiable.
    let stored = replication::get_signed_control(&conn, &input.signed_control.event_id)
        .unwrap()
        .expect("signed control");
    assert_eq!(stored.sender_device_id, device.to_string());
    assert_eq!(stored.signature, built.signed.signature.to_vec());

    // Second bootstrap rejected with structured error (ID-5).
    let err = replication::bootstrap_local_device(&mut conn, &input).expect_err("second");
    let msg = err.to_string();
    assert!(
        msg.contains("BootstrapAlreadyEnrolled"),
        "expected BootstrapAlreadyEnrolled, got: {msg}"
    );
}

const CONTENT_TYPE_DEVICE_ENROLLED_I64: i64 = 0x0010;

#[test]
fn private_key_wrap__roundtrip_row() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let id = "ffffffff-ffff-4fff-8fff-ffffffffffff";
    replication::insert_device_identity(&conn, &sample_identity(id, "local", id)).unwrap();
    replication::put_device_private_key_wrap(
        &conn,
        &DevicePrivateKeyRow {
            device_id: id.to_string(),
            wrap_schema_version: 1,
            algorithm: "AES-256-GCM".to_string(),
            protection: "datakey".to_string(),
            wrap_nonce: vec![0xAB; 12],
            wrap_ciphertext: vec![0xCD; 80],
            created_at: CREATED_AT.to_string(),
        },
    )
    .unwrap();
    let got = replication::get_device_private_key_wrap(&conn, id)
        .unwrap()
        .unwrap();
    assert_eq!(got.protection, "datakey");
    assert_eq!(got.wrap_nonce.len(), 12);
}

#[test]
fn tick_ack_cycle__reaches_unreachable() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    replication::upsert_erasure_ack(
        &conn,
        &replication::ErasureAckRow {
            erasure_id: "er-1".to_string(),
            peer_device_id: "peer-1".to_string(),
            content_key_id: "ck-1".to_string(),
            status: "pending".to_string(),
            sync_cycles_waiting: 0,
            updated_at: CREATED_AT.to_string(),
        },
    )
    .unwrap();
    for i in 0..3 {
        replication::tick_ack_cycle(&conn, &format!("t{i}")).unwrap();
    }
    let pending = replication::list_pending_acks(&conn).unwrap();
    assert!(pending.is_empty());
    let status: String = conn
        .query_row(
            "SELECT status FROM erasure_ack_projection WHERE erasure_id = ?",
            ["er-1"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "unreachable");
}

// ---------------------------------------------------------------------------
// ReplicationProjection (membership SOV → side stores)
// ---------------------------------------------------------------------------

#[test]
fn append_event__device_enrolled__projects_identity_and_signed_control() {
    let (_tmp, store) = open_store();
    let keys = generate_device_keys().unwrap();
    let device = DeviceId::new();
    let ed = keys.verifying_key().to_bytes();
    let x = keys.x25519_public().to_bytes();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: ed,
            x25519_pub: x,
        }),
        device,
        1,
        &keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    verify_envelope(&built.signed, &keys.verifying_key()).unwrap();

    let fp = [0x33u8; 32];
    let event = EventBuilder::new(
        AggregateType::System,
        device.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id: device,
        enrolled_by_device_id: device,
        status: "local".into(),
        fingerprint_sha256: hex::encode(fp),
        ed25519_public: hex::encode(ed),
        x25519_public: hex::encode(x),
        schema_version: REPLICATION_SCHEMA_VERSION,
        replication_event_id: built.signed.outer.event_id,
        local_seq: built.signed.outer.local_seq,
        envelope_id: built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(built.signed.signature),
        body_hex: hex::encode(&built.body),
        content_type_code: built.signed.outer.content_type_code.as_u16(),
    }))
    .unwrap();

    store.append_event(&event).unwrap();

    let conn = store.connection().lock().unwrap();
    let identity = replication::get_device(&conn, &device.to_string())
        .unwrap()
        .expect("device_identity projected");
    assert_eq!(identity.status, "local");
    assert_eq!(identity.ed25519_public, ed.to_vec());
    assert_eq!(identity.fingerprint_sha256, fp.to_vec());

    let control = replication::get_signed_control(&conn, &built.signed.outer.event_id.to_string())
        .unwrap()
        .expect("signed_control projected");
    assert_eq!(control.signature, built.signed.signature.to_vec());
    assert_eq!(control.body, built.body);
    assert!(replication::envelope_exists(&conn, &built.signed.outer.event_id.to_string()).unwrap());
}

#[test]
fn append_event__device_revoked__projects_tombstone_and_control() {
    let (_tmp, store) = open_store();
    let local_keys = generate_device_keys().unwrap();
    let peer_keys = generate_device_keys().unwrap();
    let local = DeviceId::new();
    let peer = DeviceId::new();
    let local_ed = local_keys.verifying_key().to_bytes();
    let local_x = local_keys.x25519_public().to_bytes();
    let peer_ed = peer_keys.verifying_key().to_bytes();
    let peer_x = peer_keys.x25519_public().to_bytes();

    // Enroll local then peer via events (projector path).
    let local_built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: local,
            ed25519_pub: local_ed,
            x25519_pub: local_x,
        }),
        local,
        1,
        &local_keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let local_event = EventBuilder::new(
        AggregateType::System,
        local.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id: local,
        enrolled_by_device_id: local,
        status: "local".into(),
        fingerprint_sha256: hex::encode([0x11u8; 32]),
        ed25519_public: hex::encode(local_ed),
        x25519_public: hex::encode(local_x),
        schema_version: REPLICATION_SCHEMA_VERSION,
        replication_event_id: local_built.signed.outer.event_id,
        local_seq: 1,
        envelope_id: local_built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(local_built.signed.signature),
        body_hex: hex::encode(&local_built.body),
        content_type_code: ContentTypeCode::DeviceEnrolled.as_u16(),
    }))
    .unwrap();
    store.append_event(&local_event).unwrap();

    let peer_built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: peer,
            ed25519_pub: peer_ed,
            x25519_pub: peer_x,
        }),
        local,
        2,
        &local_keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let peer_event = EventBuilder::new(
        AggregateType::System,
        peer.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id: peer,
        enrolled_by_device_id: local,
        status: "active".into(),
        fingerprint_sha256: hex::encode([0x22u8; 32]),
        ed25519_public: hex::encode(peer_ed),
        x25519_public: hex::encode(peer_x),
        schema_version: REPLICATION_SCHEMA_VERSION,
        replication_event_id: peer_built.signed.outer.event_id,
        local_seq: 2,
        envelope_id: peer_built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(peer_built.signed.signature),
        body_hex: hex::encode(&peer_built.body),
        content_type_code: ContentTypeCode::DeviceEnrolled.as_u16(),
    }))
    .unwrap();
    store.append_event(&peer_event).unwrap();

    let revoke_built = build_and_sign_control(
        ContentTypeCode::DeviceRevoked,
        &ControlPayload::DeviceRevoked(DeviceRevokedPayload {
            device_id: peer,
            reason_code: "test-revoke".into(),
        }),
        local,
        3,
        &local_keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();
    let revoke_event = EventBuilder::new(
        AggregateType::System,
        peer.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceRevoked(EventDeviceRevokedPayload {
        device_id: peer,
        revoked_by_device_id: local,
        reason_code: "test-revoke".into(),
        replication_event_id: revoke_built.signed.outer.event_id,
        local_seq: 3,
        envelope_id: revoke_built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(revoke_built.signed.signature),
        body_hex: hex::encode(&revoke_built.body),
        content_type_code: ContentTypeCode::DeviceRevoked.as_u16(),
    }))
    .unwrap();
    store.append_event(&revoke_event).unwrap();

    let conn = store.connection().lock().unwrap();
    let peer_row = replication::get_device(&conn, &peer.to_string())
        .unwrap()
        .expect("peer row retained");
    assert_eq!(peer_row.status, "revoked");

    let tombstoned: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM device_id_tombstone WHERE device_id = ?)",
            [peer.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(tombstoned, "DeviceRevoked must insert tombstone");

    let control =
        replication::get_signed_control(&conn, &revoke_built.signed.outer.event_id.to_string())
            .unwrap()
            .expect("revoke signed control");
    assert_eq!(control.sender_device_id, local.to_string());
    assert_eq!(
        control.content_type_code,
        ContentTypeCode::DeviceRevoked.as_u16() as i64
    );
}

#[test]
fn append_device_enrolled_with_private_key__bad_wrap__rolls_back_event() {
    let (_tmp, store) = open_store();
    let keys = generate_device_keys().unwrap();
    let device = DeviceId::new();
    let ed = keys.verifying_key().to_bytes();
    let x = keys.x25519_public().to_bytes();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: ed,
            x25519_pub: x,
        }),
        device,
        1,
        &keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();

    let event = EventBuilder::new(
        AggregateType::System,
        device.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id: device,
        enrolled_by_device_id: device,
        status: "local".into(),
        fingerprint_sha256: hex::encode([0xAAu8; 32]),
        ed25519_public: hex::encode(ed),
        x25519_public: hex::encode(x),
        schema_version: REPLICATION_SCHEMA_VERSION,
        replication_event_id: built.signed.outer.event_id,
        local_seq: 1,
        envelope_id: built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(built.signed.signature),
        body_hex: hex::encode(&built.body),
        content_type_code: ContentTypeCode::DeviceEnrolled.as_u16(),
    }))
    .unwrap();

    // Empty wrap fails validation after projector would have written identity.
    let bad_key = DevicePrivateKeyRow {
        device_id: device.to_string(),
        wrap_schema_version: 1,
        algorithm: "AES-256-GCM".to_string(),
        protection: "datakey".to_string(),
        wrap_nonce: vec![],
        wrap_ciphertext: vec![],
        created_at: CREATED_AT.to_string(),
    };
    let err = store
        .append_device_enrolled_with_private_key(&event, &bad_key)
        .expect_err("empty wrap must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("wrap_nonce") || msg.contains("non-empty"),
        "expected wrap validation error, got: {msg}"
    );

    let conn = store.connection().lock().unwrap();
    assert!(
        replication::get_device(&conn, &device.to_string())
            .unwrap()
            .is_none(),
        "failed private-key insert must roll back projected identity"
    );
    let event_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE event_type = 'DeviceEnrolled'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        event_count, 0,
        "failed private-key insert must roll back event SOV row"
    );
}

#[test]
fn append_device_enrolled_with_private_key__local__atomic_ok() {
    let (_tmp, store) = open_store();
    let keys = generate_device_keys().unwrap();
    let device = DeviceId::new();
    let ed = keys.verifying_key().to_bytes();
    let x = keys.x25519_public().to_bytes();
    let built = build_and_sign_control(
        ContentTypeCode::DeviceEnrolled,
        &ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: ed,
            x25519_pub: x,
        }),
        device,
        1,
        &keys.signing_key(),
        nil_content_key_id(),
    )
    .unwrap();

    let event = EventBuilder::new(
        AggregateType::System,
        device.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id: device,
        enrolled_by_device_id: device,
        status: "local".into(),
        fingerprint_sha256: hex::encode([0xBBu8; 32]),
        ed25519_public: hex::encode(ed),
        x25519_public: hex::encode(x),
        schema_version: REPLICATION_SCHEMA_VERSION,
        replication_event_id: built.signed.outer.event_id,
        local_seq: 1,
        envelope_id: built.signed.outer.envelope_id.as_uuid(),
        signature_hex: hex::encode(built.signed.signature),
        body_hex: hex::encode(&built.body),
        content_type_code: ContentTypeCode::DeviceEnrolled.as_u16(),
    }))
    .unwrap();

    let private_key = DevicePrivateKeyRow {
        device_id: device.to_string(),
        wrap_schema_version: 1,
        algorithm: "AES-256-GCM".to_string(),
        protection: "datakey".to_string(),
        wrap_nonce: vec![0xAB; 12],
        wrap_ciphertext: vec![0xCD; 80],
        created_at: CREATED_AT.to_string(),
    };
    store
        .append_device_enrolled_with_private_key(&event, &private_key)
        .unwrap();

    let conn = store.connection().lock().unwrap();
    assert!(replication::has_active_or_local_device(&conn).unwrap());
    let wrap = replication::get_device_private_key_wrap(&conn, &device.to_string())
        .unwrap()
        .expect("private key wrap");
    assert_eq!(wrap.wrap_nonce.len(), 12);
}
