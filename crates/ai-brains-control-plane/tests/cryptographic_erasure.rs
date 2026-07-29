#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T165 — governed cryptographic erasure orchestrator tests.

use ai_brains_control_plane::{
    AllowAllPolicy, ContentEnvelopeWipeStore, ContentKeyStatus, ControlPlaneError, DenyAllPolicy,
    StoreContentEnvelopeWipe, StorePorts, SystemClock, WipeContentEnvelopeCommand, make_principal,
    parse_scope_key, wipe_content_envelope,
};
use ai_brains_core::ids::{ContentKeyId, PrincipalId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_events::{EventKind, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, BlobSubject, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
    PurgeDerivedCounts, WalCheckpointOutcome,
};
use std::cell::Cell;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use uuid::Uuid;

const CREATED_AT: &str = "2026-07-28T12:00:00Z";
const FIXTURE_PLAINTEXT: &str = "T165-fixture-secret-plaintext-xyzzy";

fn open_ports() -> (NamedTempFile, StorePorts) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    (
        temp_file,
        StorePorts::from_store(SqliteEventStore::new(conn)),
    )
}

fn principal() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "wipe-op")
}

fn scope() -> ScopeRef {
    ScopeRef::Personal(UserId::new())
}

fn insert_active_key(store: &SqliteEventStore, content_key_id: &ContentKeyId) {
    let conn = store.connection().lock().unwrap();
    content_envelope::insert_content_key_wrap(
        &conn,
        &content_key_id.to_string(),
        1,
        &[0xAAu8; 12],
        &[0xBBu8; 48],
        CREATED_AT,
    )
    .unwrap();
}

fn insert_blob(
    store: &SqliteEventStore,
    content_key_id: &ContentKeyId,
    blob_id: &str,
    subject_kind: Option<&str>,
    subject_id: Option<&str>,
) {
    let conn = store.connection().lock().unwrap();
    let ct = vec![0xCCu8; 32];
    content_envelope::insert_encrypted_blob(
        &conn,
        &EncryptedBlobRow {
            blob_id: blob_id.to_string(),
            content_key_id: content_key_id.to_string(),
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            algorithm: ALGORITHM_AES_256_GCM.to_string(),
            nonce: vec![0xDDu8; 12],
            ciphertext: ct.clone(),
            content_class: None,
            subject_kind: subject_kind.map(str::to_string),
            subject_id: subject_id.map(str::to_string),
            size_bytes: ct.len() as i64,
            created_at: CREATED_AT.to_string(),
        },
    )
    .unwrap();
}

fn insert_memory_with_fts_and_embedding(store: &SqliteEventStore, memory_id: &str, content: &str) {
    let conn = store.connection().lock().unwrap();
    let embedding: Vec<u8> = (0u8..16).collect();
    conn.execute(
        "INSERT INTO memory_projection (
            memory_id, content, privacy, status, level, created_at, updated_at, embedding, embedding_generated_at
         ) VALUES (?, ?, '\"LocalOnly\"', 'active', 0, ?, ?, ?, ?)",
        rusqlite::params![
            memory_id,
            content,
            CREATED_AT,
            CREATED_AT,
            embedding,
            CREATED_AT,
        ],
    )
    .unwrap();
    // FTS trigger should populate memory_fts on insert; if not, insert explicitly.
    let fts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_fts WHERE memory_id = ?",
            [memory_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if fts_count == 0 {
        let rowid: i64 = conn
            .query_row(
                "SELECT rowid FROM memory_projection WHERE memory_id = ?",
                [memory_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO memory_fts(rowid, content, memory_id) VALUES (?, ?, ?)",
            rusqlite::params![rowid, content, memory_id],
        )
        .unwrap();
    }
}

fn cmd(content_key_id: ContentKeyId, dry_run: bool, confirm: bool) -> WipeContentEnvelopeCommand {
    WipeContentEnvelopeCommand {
        principal: principal(),
        content_key_id,
        scope: scope(),
        reason: Some("test wipe".into()),
        tombstone_id: None,
        dry_run,
        confirm,
    }
}

// ---------------------------------------------------------------------------
// Fault-injectable wrapper
// ---------------------------------------------------------------------------

struct FailDestroyStore {
    inner: StoreContentEnvelopeWipe,
    fail_destroy: Cell<bool>,
    destroy_calls: Cell<u32>,
}

impl ContentEnvelopeWipeStore for FailDestroyStore {
    fn get_wrap_status(
        &self,
        content_key_id: &str,
    ) -> Result<Option<ContentKeyStatus>, ControlPlaneError> {
        self.inner.get_wrap_status(content_key_id)
    }
    fn destroy_content_key_wrap(
        &self,
        content_key_id: &str,
        destroyed_at: &str,
    ) -> Result<(), ControlPlaneError> {
        self.destroy_calls.set(self.destroy_calls.get() + 1);
        if self.fail_destroy.get() {
            return Err(ControlPlaneError::Query("injected destroy failure".into()));
        }
        self.inner
            .destroy_content_key_wrap(content_key_id, destroyed_at)
    }
    fn list_blob_subjects(
        &self,
        content_key_id: &str,
    ) -> Result<Vec<BlobSubject>, ControlPlaneError> {
        self.inner.list_blob_subjects(content_key_id)
    }
    fn blob_count(&self, content_key_id: &str) -> Result<u64, ControlPlaneError> {
        self.inner.blob_count(content_key_id)
    }
    fn get_tombstone_id(&self, content_key_id: &str) -> Result<Option<String>, ControlPlaneError> {
        self.inner.get_tombstone_id(content_key_id)
    }
    fn purge_derived_plaintext(
        &self,
        subjects: &[BlobSubject],
    ) -> Result<PurgeDerivedCounts, ControlPlaneError> {
        self.inner.purge_derived_plaintext(subjects)
    }
    fn is_wrap_absent(&self, content_key_id: &str) -> Result<bool, ControlPlaneError> {
        self.inner.is_wrap_absent(content_key_id)
    }
    fn store_open_refused(&self, content_key_id: &str) -> Result<bool, ControlPlaneError> {
        self.inner.store_open_refused(content_key_id)
    }
    fn fts_clear_for_subjects(&self, subjects: &[BlobSubject]) -> Result<bool, ControlPlaneError> {
        self.inner.fts_clear_for_subjects(subjects)
    }
    fn wal_checkpoint_truncate(&self) -> Result<WalCheckpointOutcome, ControlPlaneError> {
        self.inner.wal_checkpoint_truncate()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn wipe__dry_run__no_wrap_destroy() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        None,
        None,
    );
    let side = StoreContentEnvelopeWipe::new(ports.store());

    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, true, false),
    )
    .unwrap();

    assert_eq!(resp.status, "dry_run");
    assert!(!resp.wrap_destroyed);
    assert_eq!(resp.validation.wal_checkpoint, "skipped_dry_run");
    assert!(
        resp.warnings
            .iter()
            .any(|w| w.contains("NIST") || w.contains("Purge"))
    );

    {
        let conn = ports.writer.store().connection().lock().unwrap();
        assert!(!content_envelope::is_content_key_destroyed(&conn, &key.to_string()).unwrap());
    }
    let events = ports.writer.store().read_all_events().unwrap();
    assert!(
        !events
            .iter()
            .any(|e| e.event_type == EventKind::ContentErasureRequested
                || e.event_type == EventKind::ContentErased)
    );
}

#[test]
fn wipe__execute__destroys_wrap_and_emits_events() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        None,
        None,
    );
    let side = StoreContentEnvelopeWipe::new(ports.store());

    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();

    assert_eq!(resp.status, "wiped");
    assert!(resp.wrap_destroyed);
    assert!(resp.verify.wrap_absent);
    assert!(resp.tombstone_id.is_some());
    assert!(resp.validation.store_open_refused);
    assert!(
        resp.validation.wal_checkpoint == "truncated"
            || resp.validation.wal_checkpoint == "pending_passive"
    );

    {
        let conn = ports.writer.store().connection().lock().unwrap();
        assert!(content_envelope::is_content_key_destroyed(&conn, &key.to_string()).unwrap());
        assert!(
            content_envelope::get_tombstone(&conn, &key.to_string())
                .unwrap()
                .is_some()
        );
    }

    let events = ports.writer.store().read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::ContentErasureRequested)
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::ContentErased)
    );
}

#[test]
fn wipe__execute__never_erased_without_destroy() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = FailDestroyStore {
        inner: StoreContentEnvelopeWipe::new(ports.store()),
        fail_destroy: Cell::new(true),
        destroy_calls: Cell::new(0),
    };

    let err = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::Query(_)));
    assert_eq!(side.destroy_calls.get(), 1);

    let events = ports.writer.store().read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::ContentErasureRequested),
        "Requested should be durable for audit"
    );
    assert!(
        !events
            .iter()
            .any(|e| e.event_type == EventKind::ContentErased),
        "E2: never ContentErased without destroy"
    );
}

#[test]
fn wipe__verify__wrap_absent_after_wipe() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert!(resp.verify.wrap_absent);
    assert!(side.is_wrap_absent(&key.to_string()).unwrap());
}

#[test]
fn wipe__validation__store_cannot_load_wrap() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert!(resp.validation.store_open_refused);
    // Not a fake AEAD open_fails signal — wrap material simply absent.
    let conn = ports.writer.store().connection().lock().unwrap();
    let row = content_envelope::get_content_key_wrap(&conn, &key.to_string())
        .unwrap()
        .unwrap();
    assert!(row.wrap_nonce.is_none());
    assert!(row.wrap_ciphertext.is_none());
    assert_eq!(row.status, "destroyed");
}

#[test]
fn wipe__already_erased__idempotent() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());

    let first = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(first.status, "wiped");

    let second = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(second.status, "already_erased");
    assert!(second.wrap_destroyed);
    assert_eq!(second.tombstone_id, first.tombstone_id);
    assert_eq!(second.validation.wal_checkpoint, "skipped_already_erased");
}

#[test]
fn wipe__legacy_memory_id__not_envelope__refused() {
    let (_t, ports) = open_ports();
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let key = ContentKeyId::new(); // no content_key_store row
    let err = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::NotEnvelopeBacked(_)));
    let events = ports.writer.store().read_all_events().unwrap();
    assert!(events.is_empty());
}

#[test]
fn wipe__policy_denied__no_events() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let err = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &DenyAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::PolicyDenied(_)));
    let events = ports.writer.store().read_all_events().unwrap();
    assert!(events.is_empty());
}

#[test]
fn wipe__resume_after_destroy_before_erased() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    // Simulate crash after destroy, before ContentErased.
    {
        let conn = ports.writer.store().connection().lock().unwrap();
        content_envelope::destroy_content_key_wrap(&conn, &key.to_string(), CREATED_AT).unwrap();
        assert!(content_envelope::is_content_key_destroyed(&conn, &key.to_string()).unwrap());
        assert!(
            content_envelope::get_tombstone(&conn, &key.to_string())
                .unwrap()
                .is_none()
        );
    }
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(resp.status, "wiped");
    assert!(resp.verify.wrap_absent);
    let conn = ports.writer.store().connection().lock().unwrap();
    assert!(
        content_envelope::get_tombstone(&conn, &key.to_string())
            .unwrap()
            .is_some()
    );
    // Wrap must not be resurrected.
    assert!(content_envelope::is_content_key_destroyed(&conn, &key.to_string()).unwrap());
}

#[test]
fn wipe__fts__plaintext_absent_after_purge() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    let memory_id = Uuid::new_v4().to_string();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("memory"),
        Some(&memory_id),
    );
    insert_memory_with_fts_and_embedding(ports.writer.store(), &memory_id, FIXTURE_PLAINTEXT);

    {
        let conn = ports.writer.store().connection().lock().unwrap();
        assert!(content_envelope::memory_fts_has_hits(&conn, &memory_id).unwrap());
    }

    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(resp.status, "wiped");
    assert!(resp.validation.fts_clear);
    assert!(resp.purged.fts_rows >= 1);

    let conn = ports.writer.store().connection().lock().unwrap();
    assert!(!content_envelope::memory_fts_has_hits(&conn, &memory_id).unwrap());
}

#[test]
fn wipe__embedding__cleared_for_subject() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    let memory_id = Uuid::new_v4().to_string();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("memory"),
        Some(&memory_id),
    );
    insert_memory_with_fts_and_embedding(ports.writer.store(), &memory_id, FIXTURE_PLAINTEXT);

    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert!(resp.purged.embeddings >= 1);

    let conn = ports.writer.store().connection().lock().unwrap();
    let emb: Option<Vec<u8>> = conn
        .query_row(
            "SELECT embedding FROM memory_projection WHERE memory_id = ?",
            [&memory_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(emb.is_none());
}

#[test]
fn wipe__multi_blob__all_subjects_purged() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    let mem_a = Uuid::new_v4().to_string();
    let mem_b = Uuid::new_v4().to_string();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("memory"),
        Some(&mem_a),
    );
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("memory"),
        Some(&mem_b),
    );
    insert_memory_with_fts_and_embedding(ports.writer.store(), &mem_a, "subject-a-secret");
    insert_memory_with_fts_and_embedding(ports.writer.store(), &mem_b, "subject-b-secret");

    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(resp.blobs_considered, 2);
    assert!(resp.validation.fts_clear);

    let conn = ports.writer.store().connection().lock().unwrap();
    assert!(!content_envelope::memory_fts_has_hits(&conn, &mem_a).unwrap());
    assert!(!content_envelope::memory_fts_has_hits(&conn, &mem_b).unwrap());
}

#[test]
fn wipe__wal_checkpoint__truncate_attempted() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert!(
        resp.validation.wal_checkpoint == "truncated"
            || resp.validation.wal_checkpoint == "pending_passive",
        "wal_checkpoint={}",
        resp.validation.wal_checkpoint
    );
}

#[test]
fn wipe__dependents__skipped_without_source_link() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    let memory_id = Uuid::new_v4().to_string();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("memory"),
        Some(&memory_id),
    );
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, true),
    )
    .unwrap();
    assert_eq!(resp.dependents_marked, 0);
    assert!(
        resp.warnings
            .iter()
            .any(|w| w.contains("dependents_skipped_no_source_link"))
    );
}

#[test]
fn wipe__dependents__marked_stale_when_source_linked() {
    use ai_brains_control_plane::{
        ObserveSourceRequest, Sha256FingerprinterPort, SourceContent, observe_source,
    };
    use ai_brains_core::source::SourceKind;

    let (_t, ports) = open_ports();
    let scope = ScopeRef::Personal(UserId::new());
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "human");
    let fp = Sha256FingerprinterPort::new();

    // Observe a source so it is registered.
    let obs = observe_source(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &fp,
        &AllowAllPolicy,
        ObserveSourceRequest {
            principal: human.id,
            scope: scope.clone(),
            kind: SourceKind::File,
            locator: Some("file:///tmp/t165-src.md".into()),
            display_name: "t165-src".into(),
            content: SourceContent::Bytes(b"source body for CE".to_vec()),
            privacy: Privacy::LocalOnly,
            run_invalidation: false,
        },
    )
    .unwrap();
    let source_id = obs.source_id;

    // Wipe with subject_kind=source and registered SourceId — mark_source_unavailable runs
    // even with zero dependents (dependents_marked may be 0 but no skip warning).
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    insert_blob(
        ports.writer.store(),
        &key,
        &Uuid::new_v4().to_string(),
        Some("source"),
        Some(&source_id.to_string()),
    );

    let side = StoreContentEnvelopeWipe::new(ports.store());
    let resp = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        WipeContentEnvelopeCommand {
            principal: human,
            content_key_id: key,
            scope,
            reason: Some("ce source".into()),
            tombstone_id: None,
            dry_run: false,
            confirm: true,
        },
    )
    .unwrap();
    assert_eq!(resp.status, "wiped");
    // Source-linked path was taken: no dependents_skipped warning when source registered.
    let skipped = resp
        .warnings
        .iter()
        .any(|w| w.contains("dependents_skipped_no_source_link"));
    assert!(
        !skipped,
        "registered SourceId subject must not emit skip warning: {:?}",
        resp.warnings
    );

    // SourceUnavailable should have been appended.
    let events = ports.writer.store().read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(&e.payload, Payload::SourceUnavailable(_))),
        "expected SourceUnavailable for linked source"
    );
}

#[test]
fn wipe__execute_without_confirm__invalid_payload() {
    let (_t, ports) = open_ports();
    let key = ContentKeyId::new();
    insert_active_key(ports.writer.store(), &key);
    let side = StoreContentEnvelopeWipe::new(ports.store());
    let err = wipe_content_envelope(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        cmd(key, false, false),
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
}

#[test]
fn wipe__parse_scope_used_in_daemon_shape() {
    // Smoke: scope parse used by daemon path remains available.
    let s = parse_scope_key(&format!("Personal:{}", UserId::new())).unwrap();
    assert!(matches!(s, ScopeRef::Personal(_)));
}

// Silence unused import if Arc/Mutex kept for future fault injectors.
#[allow(dead_code)]
fn _keep_arc() {
    let _ = Arc::new(Mutex::new(0u32));
}
