#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T163 — migration 0026 content envelope schema + side stores + erasure projections.

use ai_brains_core::ids::{ContentKeyId, MemoryId, PrincipalId, TombstoneId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ContentErasedPayload, ContentErasureRequestedPayload, ErasureTicketAcceptedPayload,
    MemoryForgottenPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use tempfile::NamedTempFile;

/// Fixture wrap nonce — not real AEAD (T163 uses opaque bytes only).
const FIXTURE_WRAP_NONCE: [u8; 12] = [0xA5; 12];
/// Fixture wrap / content ciphertext — not real AEAD.
const FIXTURE_CIPHERTEXT: [u8; 48] = [0xBE; 48];
const FIXTURE_CONTENT_NONCE: [u8; 12] = [0xC3; 12];
const CREATED_AT: &str = "2026-07-28T12:00:00Z";
const DESTROYED_AT: &str = "2026-07-28T13:00:00Z";

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

fn table_exists_conn(conn: &VaultConnection, name: &str) -> bool {
    let locked = conn.lock().unwrap();
    locked
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type IN ('table', 'view') AND name = ?
            )",
            [name],
            |row| row.get::<_, i64>(0),
        )
        .map(|v| v == 1)
        .unwrap()
}

fn index_exists(store: &SqliteEventStore, name: &str) -> bool {
    let conn = store.connection().lock().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?",
            [name],
            |r| r.get(0),
        )
        .unwrap();
    count == 1
}

fn table_columns(store: &SqliteEventStore, table: &str) -> Vec<String> {
    let conn = store.connection().lock().unwrap();
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

fn append_erasure_requested(
    store: &SqliteEventStore,
    content_key_id: ContentKeyId,
    requester: PrincipalId,
    reason: &str,
) {
    let payload = Payload::ContentErasureRequested(ContentErasureRequestedPayload {
        content_key_id,
        requester,
        reason: reason.to_string(),
    });
    let envelope = EventBuilder::new(
        AggregateType::System,
        content_key_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(payload)
    .unwrap();
    store.append_event(&envelope).unwrap();
}

fn append_content_erased(
    store: &SqliteEventStore,
    content_key_id: ContentKeyId,
    tombstone_id: TombstoneId,
) {
    let payload = Payload::ContentErased(ContentErasedPayload {
        content_key_id,
        tombstone_id,
    });
    let envelope = EventBuilder::new(
        AggregateType::System,
        content_key_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(payload)
    .unwrap();
    store.append_event(&envelope).unwrap();
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

#[test]
fn migration_0026__fresh_db__creates_envelope_tables() {
    let (_tmp, store) = open_store();
    for table in [
        "content_key_store",
        "encrypted_content_blob",
        "erasure_request_projection",
        "tombstone_projection",
    ] {
        assert!(table_exists(&store, table), "missing table {table}");
    }
    for index in [
        "idx_content_key_store_status",
        "idx_encrypted_content_blob_content_key",
        "idx_encrypted_content_blob_subject",
        "idx_erasure_request_status",
    ] {
        assert!(index_exists(&store, index), "missing index {index}");
    }
    // Prior migrations still present.
    assert!(table_exists(&store, "briefing_cache_projection"));
    assert!(table_exists(&store, "events"));
}

#[test]
fn migration_0026__after_0025__applies_forward() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0025_briefings_query_traces")).unwrap();
    }

    assert!(
        table_exists_conn(&conn, "briefing_cache_projection"),
        "0025 vault must have briefing_cache_projection"
    );
    assert!(
        !table_exists_conn(&conn, "content_key_store"),
        "0025-only vault must not yet have content_key_store"
    );

    {
        let locked = conn.lock().unwrap();
        locked
            .execute(
                "INSERT INTO briefing_cache_projection (
                    cache_key, briefing_type, scope_key, policy_version,
                    source_version_vector, budget, packet_json, generated_at, expires
                 ) VALUES ('seed-key', 'Project', 'scope', '1', 'vv', 0, '{}',
                           '2026-01-01T00:00:00Z', NULL)",
                [],
            )
            .unwrap();
    }

    conn.migrate().unwrap();

    assert!(table_exists_conn(&conn, "content_key_store"));
    assert!(table_exists_conn(&conn, "encrypted_content_blob"));
    assert!(table_exists_conn(&conn, "erasure_request_projection"));
    assert!(table_exists_conn(&conn, "tombstone_projection"));

    let count: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM briefing_cache_projection WHERE cache_key = 'seed-key'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "prior briefing row must survive 0026 migrate");

    let applied: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE name = '0026_content_envelopes_erasure'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 1);
}

// ---------------------------------------------------------------------------
// content_key_store side store
// ---------------------------------------------------------------------------

#[test]
fn content_key_store__insert_active__round_trip() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key_id = ContentKeyId::new().to_string();

    content_envelope::insert_content_key_wrap(
        &conn,
        &key_id,
        ENVELOPE_SCHEMA_VERSION,
        &FIXTURE_WRAP_NONCE,
        &FIXTURE_CIPHERTEXT,
        CREATED_AT,
    )
    .unwrap();

    let row = content_envelope::get_content_key_wrap(&conn, &key_id)
        .unwrap()
        .expect("row must exist");
    assert_eq!(row.content_key_id, key_id);
    assert_eq!(row.wrap_schema_version, ENVELOPE_SCHEMA_VERSION);
    assert_eq!(row.algorithm, ALGORITHM_AES_256_GCM);
    assert_eq!(
        row.wrap_nonce.as_deref(),
        Some(FIXTURE_WRAP_NONCE.as_slice())
    );
    assert_eq!(
        row.wrap_ciphertext.as_deref(),
        Some(FIXTURE_CIPHERTEXT.as_slice())
    );
    assert_eq!(row.status, "active");
    assert_eq!(row.created_at, CREATED_AT);
    assert_eq!(row.destroyed_at, None);

    let missing = content_envelope::get_content_key_wrap(&conn, "no-such-key").unwrap();
    assert!(missing.is_none());
}

#[test]
fn content_key_store__destroy__nulls_wrap_and_sets_status() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key_id = ContentKeyId::new().to_string();

    content_envelope::insert_content_key_wrap(
        &conn,
        &key_id,
        ENVELOPE_SCHEMA_VERSION,
        &FIXTURE_WRAP_NONCE,
        &FIXTURE_CIPHERTEXT,
        CREATED_AT,
    )
    .unwrap();

    content_envelope::destroy_content_key_wrap(&conn, &key_id, DESTROYED_AT).unwrap();

    let row = content_envelope::get_content_key_wrap(&conn, &key_id)
        .unwrap()
        .expect("destroyed row must still exist");
    assert_eq!(row.status, "destroyed");
    assert_eq!(row.wrap_nonce, None);
    assert_eq!(row.wrap_ciphertext, None);
    assert_eq!(row.destroyed_at.as_deref(), Some(DESTROYED_AT));
    assert!(content_envelope::is_content_key_destroyed(&conn, &key_id).unwrap());
}

#[test]
fn content_key_store__destroy_idempotent__second_call_ok() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key_id = ContentKeyId::new().to_string();

    content_envelope::insert_content_key_wrap(
        &conn,
        &key_id,
        ENVELOPE_SCHEMA_VERSION,
        &FIXTURE_WRAP_NONCE,
        &FIXTURE_CIPHERTEXT,
        CREATED_AT,
    )
    .unwrap();

    content_envelope::destroy_content_key_wrap(&conn, &key_id, DESTROYED_AT).unwrap();
    content_envelope::destroy_content_key_wrap(&conn, &key_id, "2026-07-28T14:00:00Z").unwrap();

    let row = content_envelope::get_content_key_wrap(&conn, &key_id)
        .unwrap()
        .expect("row must exist");
    assert_eq!(row.status, "destroyed");
    assert_eq!(row.wrap_nonce, None);
    assert_eq!(row.wrap_ciphertext, None);
    // First destroyed_at preserved (COALESCE).
    assert_eq!(row.destroyed_at.as_deref(), Some(DESTROYED_AT));
}

#[test]
fn content_key_store__check__rejects_active_without_wrap() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let err = conn
        .execute(
            "INSERT INTO content_key_store (
                content_key_id, wrap_schema_version, algorithm,
                wrap_nonce, wrap_ciphertext, status, created_at, destroyed_at
             ) VALUES ('k-active-null', 1, 'AES-256-GCM', NULL, NULL, 'active', ?, NULL)",
            [CREATED_AT],
        )
        .expect_err("active without wrap must fail CHECK");
    let msg = err.to_string();
    assert!(
        msg.contains("CHECK") || msg.contains("constraint"),
        "expected CHECK constraint failure, got: {msg}"
    );
}

#[test]
fn content_key_store__check__rejects_destroyed_with_wrap() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let err = conn
        .execute(
            "INSERT INTO content_key_store (
                content_key_id, wrap_schema_version, algorithm,
                wrap_nonce, wrap_ciphertext, status, created_at, destroyed_at
             ) VALUES ('k-destroyed-wrap', 1, 'AES-256-GCM', X'A5A5A5A5A5A5A5A5A5A5A5A5', X'BE', 'destroyed', ?, ?)",
            [CREATED_AT, DESTROYED_AT],
        )
        .expect_err("destroyed with wrap must fail CHECK");
    let msg = err.to_string();
    assert!(
        msg.contains("CHECK") || msg.contains("constraint"),
        "expected CHECK constraint failure, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// encrypted_content_blob side store
// ---------------------------------------------------------------------------

#[test]
fn encrypted_blob__insert_get__opaque_bytes() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key_id = ContentKeyId::new().to_string();
    let blob_id = ContentKeyId::new().to_string();

    let row = EncryptedBlobRow {
        blob_id: blob_id.clone(),
        content_key_id: key_id.clone(),
        envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
        algorithm: ALGORITHM_AES_256_GCM.to_string(),
        nonce: FIXTURE_CONTENT_NONCE.to_vec(),
        ciphertext: FIXTURE_CIPHERTEXT.to_vec(),
        content_class: Some("raw_turn".to_string()),
        subject_kind: Some("memory".to_string()),
        subject_id: Some(MemoryId::new().to_string()),
        size_bytes: FIXTURE_CIPHERTEXT.len() as i64,
        created_at: CREATED_AT.to_string(),
    };
    content_envelope::insert_encrypted_blob(&conn, &row).unwrap();

    let got = content_envelope::get_encrypted_blob(&conn, &blob_id)
        .unwrap()
        .expect("blob must exist");
    assert_eq!(got.blob_id, blob_id);
    assert_eq!(got.content_key_id, key_id);
    assert_eq!(got.nonce, FIXTURE_CONTENT_NONCE.as_slice());
    assert_eq!(got.ciphertext, FIXTURE_CIPHERTEXT.as_slice());
    assert_eq!(got.size_bytes, FIXTURE_CIPHERTEXT.len() as i64);
    assert_eq!(got.content_class.as_deref(), Some("raw_turn"));
    assert_eq!(got.algorithm, ALGORITHM_AES_256_GCM);

    let missing = content_envelope::get_encrypted_blob(&conn, "no-blob").unwrap();
    assert!(missing.is_none());
}

#[test]
fn encrypted_blob__list_by_content_key__returns_rows() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key_id = ContentKeyId::new().to_string();
    let other_key = ContentKeyId::new().to_string();

    for (blob_suffix, ck) in [("a", &key_id), ("b", &key_id), ("c", &other_key)] {
        let row = EncryptedBlobRow {
            blob_id: format!("blob-{blob_suffix}"),
            content_key_id: ck.clone(),
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            algorithm: ALGORITHM_AES_256_GCM.to_string(),
            nonce: FIXTURE_CONTENT_NONCE.to_vec(),
            ciphertext: FIXTURE_CIPHERTEXT.to_vec(),
            content_class: None,
            subject_kind: None,
            subject_id: None,
            size_bytes: FIXTURE_CIPHERTEXT.len() as i64,
            created_at: CREATED_AT.to_string(),
        };
        content_envelope::insert_encrypted_blob(&conn, &row).unwrap();
    }

    let listed = content_envelope::list_blobs_for_content_key(&conn, &key_id).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].blob_id, "blob-a");
    assert_eq!(listed[1].blob_id, "blob-b");
    for b in &listed {
        assert_eq!(b.content_key_id, key_id);
        assert_eq!(b.ciphertext, FIXTURE_CIPHERTEXT.as_slice());
    }
}

// ---------------------------------------------------------------------------
// Event projections
// ---------------------------------------------------------------------------

#[test]
fn erasure_request_projection__content_erasure_requested__inserts_requested() {
    let (_tmp, store) = open_store();
    let content_key_id = ContentKeyId::new();
    let requester = PrincipalId::new();

    append_erasure_requested(&store, content_key_id, requester, "operator request");

    let conn = store.connection().lock().unwrap();
    let row = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("erasure request row");
    assert_eq!(row.content_key_id, content_key_id.to_string());
    assert_eq!(row.requester, requester.to_string());
    assert_eq!(row.reason, "operator request");
    assert_eq!(row.status, "requested");
    assert_eq!(row.completed_at, None);
    assert_eq!(row.tombstone_id, None);
    assert!(!row.requested_at.is_empty());
}

#[test]
fn tombstone_projection__content_erased__inserts_and_completes_request() {
    let (_tmp, store) = open_store();
    let content_key_id = ContentKeyId::new();
    let requester = PrincipalId::new();
    let tombstone_id = TombstoneId::new();

    append_erasure_requested(&store, content_key_id, requester, "wipe me");
    append_content_erased(&store, content_key_id, tombstone_id);

    let conn = store.connection().lock().unwrap();
    let req = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("request");
    assert_eq!(req.status, "completed");
    assert_eq!(
        req.tombstone_id.as_deref(),
        Some(tombstone_id.to_string().as_str())
    );
    assert!(req.completed_at.is_some());

    let ts = content_envelope::get_tombstone(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("tombstone");
    assert_eq!(ts.tombstone_id, tombstone_id.to_string());
    assert_eq!(ts.content_key_id, content_key_id.to_string());
    assert_eq!(ts.reason_code, "");
    assert!(!ts.erased_at.is_empty());
}

#[test]
fn erasure_request_projection__duplicate_request_after_completed__stays_completed() {
    let (_tmp, store) = open_store();
    let content_key_id = ContentKeyId::new();
    let requester = PrincipalId::new();
    let tombstone_id = TombstoneId::new();

    append_erasure_requested(&store, content_key_id, requester, "first");
    append_content_erased(&store, content_key_id, tombstone_id);

    let conn = store.connection().lock().unwrap();
    let before = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("completed request");
    let completed_at = before.completed_at.clone();
    let tombstone = before.tombstone_id.clone();
    drop(conn);

    // Re-request after completion must not demote (S14).
    append_erasure_requested(
        &store,
        content_key_id,
        PrincipalId::new(),
        "retry after done",
    );

    let conn = store.connection().lock().unwrap();
    let after = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("still present");
    assert_eq!(after.status, "completed");
    assert_eq!(after.completed_at, completed_at);
    assert_eq!(after.tombstone_id, tombstone);
    // Original requester/reason preserved (conditional UPDATE skipped).
    assert_eq!(after.requester, requester.to_string());
    assert_eq!(after.reason, "first");
}

#[test]
fn ticket_accepted__and_memory_forgotten__do_not_write_ce_tables() {
    let (_tmp, store) = open_store();
    let principal = PrincipalId::new();
    let memory_id = MemoryId::new();

    let ticket = EventBuilder::new(
        AggregateType::System,
        principal.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ErasureTicketAccepted(
        ErasureTicketAcceptedPayload {
            request_id: "ticket-1".to_string(),
            requester: principal,
            target_ids: vec![memory_id.to_string()],
            reason: Some("please erase".to_string()),
            scope: Some("Repository:x".to_string()),
        },
    ))
    .unwrap();
    store.append_event(&ticket).unwrap();

    let forget = EventBuilder::new(
        AggregateType::System,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryForgotten(MemoryForgottenPayload {
        memory_id,
    }))
    .unwrap();
    store.append_event(&forget).unwrap();

    let conn = store.connection().lock().unwrap();
    for table in [
        "content_key_store",
        "encrypted_content_blob",
        "erasure_request_projection",
        "tombstone_projection",
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "{table} must stay empty after ticket/soft-forget");
    }
}

// ---------------------------------------------------------------------------
// Rebuild policy
// ---------------------------------------------------------------------------

#[test]
fn rebuild_projections__retains_key_store_and_blobs() {
    let (_tmp, mut store) = open_store();
    let content_key_id = ContentKeyId::new();
    let key_str = content_key_id.to_string();
    let blob_id = ContentKeyId::new().to_string();

    {
        let conn = store.connection().lock().unwrap();
        content_envelope::insert_content_key_wrap(
            &conn,
            &key_str,
            ENVELOPE_SCHEMA_VERSION,
            &FIXTURE_WRAP_NONCE,
            &FIXTURE_CIPHERTEXT,
            CREATED_AT,
        )
        .unwrap();
        content_envelope::insert_encrypted_blob(
            &conn,
            &EncryptedBlobRow {
                blob_id: blob_id.clone(),
                content_key_id: key_str.clone(),
                envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
                algorithm: ALGORITHM_AES_256_GCM.to_string(),
                nonce: FIXTURE_CONTENT_NONCE.to_vec(),
                ciphertext: FIXTURE_CIPHERTEXT.to_vec(),
                content_class: None,
                subject_kind: None,
                subject_id: None,
                size_bytes: FIXTURE_CIPHERTEXT.len() as i64,
                created_at: CREATED_AT.to_string(),
            },
        )
        .unwrap();
        content_envelope::destroy_content_key_wrap(&conn, &key_str, DESTROYED_AT).unwrap();
    }

    // Seed an event so rebuild has something to apply (not empty log).
    append_erasure_requested(&store, content_key_id, PrincipalId::new(), "rebuild seed");
    append_content_erased(&store, content_key_id, TombstoneId::new());

    store.rebuild_projections().unwrap();

    let conn = store.connection().lock().unwrap();
    let wrap = content_envelope::get_content_key_wrap(&conn, &key_str)
        .unwrap()
        .expect("side-store wrap must survive rebuild");
    assert_eq!(wrap.status, "destroyed");
    assert_eq!(wrap.wrap_nonce, None);
    assert_eq!(wrap.destroyed_at.as_deref(), Some(DESTROYED_AT));

    let blob = content_envelope::get_encrypted_blob(&conn, &blob_id)
        .unwrap()
        .expect("side-store blob must survive rebuild");
    assert_eq!(blob.ciphertext, FIXTURE_CIPHERTEXT.as_slice());
}

#[test]
fn rebuild_projections__replays_erasure_and_tombstone() {
    let (_tmp, mut store) = open_store();
    let content_key_id = ContentKeyId::new();
    let requester = PrincipalId::new();
    let tombstone_id = TombstoneId::new();

    append_erasure_requested(&store, content_key_id, requester, "replay me");
    append_content_erased(&store, content_key_id, tombstone_id);

    // Simulate corrupted projection tables (as rebuild will truncate + reapply).
    {
        let conn = store.connection().lock().unwrap();
        conn.execute("DELETE FROM erasure_request_projection", [])
            .unwrap();
        conn.execute("DELETE FROM tombstone_projection", [])
            .unwrap();
    }

    store.rebuild_projections().unwrap();

    let conn = store.connection().lock().unwrap();
    let req = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("rebuilt request");
    assert_eq!(req.status, "completed");
    assert_eq!(req.requester, requester.to_string());
    assert_eq!(
        req.tombstone_id.as_deref(),
        Some(tombstone_id.to_string().as_str())
    );

    let ts = content_envelope::get_tombstone(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("rebuilt tombstone");
    assert_eq!(ts.tombstone_id, tombstone_id.to_string());
}

#[test]
fn rebuild_projections__later_request_does_not_demote_completed() {
    let (_tmp, mut store) = open_store();
    let content_key_id = ContentKeyId::new();
    let requester = PrincipalId::new();
    let tombstone_id = TombstoneId::new();

    // History: Request₁ → Erased → Request₂
    append_erasure_requested(&store, content_key_id, requester, "first");
    append_content_erased(&store, content_key_id, tombstone_id);
    append_erasure_requested(&store, content_key_id, PrincipalId::new(), "second");

    store.rebuild_projections().unwrap();

    let conn = store.connection().lock().unwrap();
    let req = content_envelope::get_erasure_request(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("request after rebuild");
    assert_eq!(
        req.status, "completed",
        "S14: rebuild must not demote completed → requested"
    );
    assert_eq!(
        req.tombstone_id.as_deref(),
        Some(tombstone_id.to_string().as_str())
    );
    assert!(req.completed_at.is_some());
    // Original request fields preserved when re-request skipped.
    assert_eq!(req.requester, requester.to_string());
    assert_eq!(req.reason, "first");

    let ts = content_envelope::get_tombstone(&conn, &content_key_id.to_string())
        .unwrap()
        .expect("tombstone still present");
    assert_eq!(ts.tombstone_id, tombstone_id.to_string());
}

// ---------------------------------------------------------------------------
// Schema honesty
// ---------------------------------------------------------------------------

#[test]
fn schema__no_plaintext_content_columns() {
    let (_tmp, store) = open_store();
    let forbidden = [
        "plaintext",
        "dek",
        "body",
        "content_plain",
        "plain_content",
        "raw_content",
        "secret",
        "data_key",
    ];
    for table in [
        "content_key_store",
        "encrypted_content_blob",
        "erasure_request_projection",
        "tombstone_projection",
    ] {
        let cols = table_columns(&store, table);
        for col in &cols {
            let lower = col.to_ascii_lowercase();
            for bad in forbidden {
                assert!(
                    lower != bad,
                    "table {table} must not have forbidden column {col}"
                );
            }
        }
        // Positive: expected opaque columns present on side stores.
        if table == "content_key_store" {
            assert!(cols.iter().any(|c| c == "wrap_nonce"));
            assert!(cols.iter().any(|c| c == "wrap_ciphertext"));
            assert!(!cols.iter().any(|c| c == "plaintext"));
        }
        if table == "encrypted_content_blob" {
            assert!(cols.iter().any(|c| c == "ciphertext"));
            assert!(cols.iter().any(|c| c == "nonce"));
            assert!(!cols.iter().any(|c| c == "plaintext"));
        }
    }
}
