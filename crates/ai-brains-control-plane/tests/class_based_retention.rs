#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T166 — class-based retention plan/apply tests.

use ai_brains_control_plane::{
    AllowAllPolicy, RetentionApplyCommand, RetentionConfig, StoreContentEnvelopeWipe, StorePorts,
    SystemClock, apply_retention, make_principal, nightly_ce_enabled, plan_retention,
};
use ai_brains_core::ids::{ContentKeyId, PrincipalId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::scope::ScopeRef;
use ai_brains_crypto::DataKey;
use ai_brains_events::EventKind;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use chrono::{Duration, Utc};
use tempfile::NamedTempFile;
use uuid::Uuid;

const CREATED_AT: &str = "2020-01-01T00:00:00Z";

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

fn store_of(ports: &StorePorts) -> &SqliteEventStore {
    ports.writer.store()
}

fn principal() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "retention-op")
}

fn scope() -> ScopeRef {
    ScopeRef::Personal(UserId::new())
}

fn config() -> RetentionConfig {
    RetentionConfig::default()
}

fn insert_turn(store: &SqliteEventStore, session_id: &str, turn_index: i64, last_accessed: &str) {
    let conn = store.connection().lock().unwrap();
    // session_projection FK
    let _ = conn.execute(
        "INSERT OR IGNORE INTO project_projection (project_id, name, created_at, updated_at)
         VALUES ('00000000-0000-0000-0000-0000000000aa', 't', ?, ?)",
        rusqlite::params![CREATED_AT, CREATED_AT],
    );
    let _ = conn.execute(
        "INSERT OR IGNORE INTO session_projection (
            session_id, project_id, status, privacy, created_at, updated_at
         ) VALUES (?, '00000000-0000-0000-0000-0000000000aa', 'active', '\"LocalOnly\"', ?, ?)",
        rusqlite::params![session_id, CREATED_AT, CREATED_AT],
    );
    conn.execute(
        "INSERT INTO turn_projection (
            session_id, turn_index, role, content, occurred_at, last_accessed_at
         ) VALUES (?, ?, 'user', 'turn-body-must-not-appear-in-report', ?, ?)",
        rusqlite::params![session_id, turn_index, last_accessed, last_accessed],
    )
    .unwrap();
}

fn insert_active_key(store: &SqliteEventStore, content_key_id: &ContentKeyId, created_at: &str) {
    let conn = store.connection().lock().unwrap();
    content_envelope::insert_content_key_wrap(
        &conn,
        &content_key_id.to_string(),
        1,
        &[0xAAu8; 12],
        &[0xBBu8; 48],
        created_at,
    )
    .unwrap();
}

fn insert_blob(
    store: &SqliteEventStore,
    content_key_id: &ContentKeyId,
    blob_id: &str,
    content_class: Option<&str>,
    subject_kind: Option<&str>,
    subject_id: Option<&str>,
    created_at: &str,
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
            content_class: content_class.map(str::to_string),
            subject_kind: subject_kind.map(str::to_string),
            subject_id: subject_id.map(str::to_string),
            size_bytes: ct.len() as i64,
            created_at: created_at.to_string(),
        },
    )
    .unwrap();
}

fn insert_memory(store: &SqliteEventStore, memory_id: &str, status: &str, content: &str) {
    let conn = store.connection().lock().unwrap();
    conn.execute(
        "INSERT INTO memory_projection (
            memory_id, content, privacy, status, level, created_at, updated_at
         ) VALUES (?, ?, '\"LocalOnly\"', ?, 0, ?, ?)",
        rusqlite::params![memory_id, content, status, CREATED_AT, CREATED_AT],
    )
    .unwrap();
}

fn insert_hierarchy(store: &SqliteEventStore, parent: &str, child: &str) {
    let conn = store.connection().lock().unwrap();
    conn.execute(
        "INSERT INTO memory_hierarchy (parent_memory_id, child_memory_id) VALUES (?, ?)",
        rusqlite::params![parent, child],
    )
    .unwrap();
}

fn insert_decision(store: &SqliteEventStore, decision_id: &str, state: &str, updated_at: &str) {
    let conn = store.connection().lock().unwrap();
    conn.execute(
        "INSERT INTO decision_projection (
            decision_id, state, title, statement, scope, proposer,
            recorded_at, updated_at
         ) VALUES (?, ?, 't', 's', '', 'p', ?, ?)",
        rusqlite::params![decision_id, state, updated_at, updated_at],
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// Plan tests
// ---------------------------------------------------------------------------

#[test]
fn retention_plan__empty_vault__zero_counts() {
    let (_tmp, ports) = open_ports();
    let report = plan_retention(store_of(&ports), &config()).unwrap();
    assert_eq!(report.api_version, "1");
    assert_eq!(report.mode, "dry_run");
    assert_eq!(report.totals.candidates, 0);
    assert_eq!(report.totals.would_ce_wipe, 0);
    assert_eq!(report.totals.would_projection_delete, 0);
    assert!(report.classes.is_empty() || report.totals.candidates == 0);
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("turn-body"));
}

#[test]
fn retention_plan__raw_turns_past_horizon__projection_delete() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);
    insert_turn(store, &sid, 1, &old);

    let report = plan_retention(store, &config()).unwrap();
    assert!(report.totals.would_projection_delete >= 2);
    let raw = report
        .classes
        .iter()
        .find(|c| c.class == "raw_turn")
        .expect("raw_turn class");
    assert_eq!(raw.mechanism, "projection_delete");
    assert!(raw.candidate_count >= 2);
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        !json.contains("turn-body-must-not-appear"),
        "R4 no plaintext: {json}"
    );
}

#[test]
fn retention_plan__envelope_secret_class__ce_wipe() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    assert!(report.totals.would_ce_wipe >= 1);
    let secret = report
        .classes
        .iter()
        .find(|c| c.class == "secret")
        .expect("secret");
    assert_eq!(secret.mechanism, "ce_wipe");
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("ciphertext"));
    assert!(!json.contains("plaintext"));
}

#[test]
fn retention_plan__unknown_class__skip() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    insert_active_key(store, &key, CREATED_AT);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("mystery_blob_class"),
        None,
        None,
        CREATED_AT,
    );

    let report = plan_retention(store, &config()).unwrap();
    let u = report
        .classes
        .iter()
        .find(|c| c.class == "unclassified")
        .expect("unclassified");
    assert_eq!(u.mechanism, "skip");
    assert!(report.totals.would_skip >= 1);
}

#[test]
fn retention_plan__approved_decision_active__skip() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let did = Uuid::new_v4().to_string();
    insert_decision(store, &did, "Approved", CREATED_AT);

    let report = plan_retention(store, &config()).unwrap();
    // Active approved must not be a dispose candidate
    let dispose = report
        .classes
        .iter()
        .filter(|c| c.class == "decision_approved")
        .any(|c| c.mechanism == "projection_delete" || c.mechanism == "ce_wipe");
    assert!(!dispose, "R6: active approved must not be age-wiped");
}

#[test]
fn retention_plan__pinned_memory__held() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let mid = Uuid::new_v4().to_string();
    insert_memory(store, &mid, "pinned", "pinned-body-secret");
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        Some("memory"),
        Some(&mid),
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    assert!(report.totals.would_held >= 1);
    let held = report.classes.iter().any(|c| c.mechanism == "held");
    assert!(held, "R11 pinned must be held: {report:?}");
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("pinned-body-secret"));
}

#[test]
fn retention_plan__no_double_count_same_content_key() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    // Two blobs, same key + class
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    let secret = report
        .classes
        .iter()
        .find(|c| c.class == "secret")
        .expect("secret");
    assert_eq!(
        secret.candidate_count, 1,
        "R13: one content_key_id → one candidate"
    );
    assert_eq!(report.totals.would_ce_wipe, 1);
}

#[test]
fn retention_plan__orphaned_envelope__listed() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(14)).to_rfc3339();
    insert_active_key(store, &key, &old);
    // no blobs

    let report = plan_retention(store, &config()).unwrap();
    let orphan = report
        .classes
        .iter()
        .find(|c| c.class == "orphaned_envelope")
        .expect("orphaned_envelope");
    assert_eq!(orphan.mechanism, "ce_wipe");
    assert!(orphan.candidate_count >= 1);
}

#[test]
fn retention_report__contains_honesty_warnings() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    let joined = report.warnings.join(" ");
    assert!(
        joined.contains("not cryptographic erasure") || joined.contains("legacy"),
        "{joined}"
    );
    assert!(
        joined.to_ascii_lowercase().contains("purge")
            || joined.to_ascii_lowercase().contains("nist"),
        "{joined}"
    );
    assert!(
        joined.to_ascii_lowercase().contains("backup"),
        "CE candidates must warn backups: {joined}"
    );
    assert!(
        joined.contains("stream_a_and_stream_b"),
        "stream independence: {joined}"
    );
}

#[test]
fn classification__blob_content_class_stream_b_only() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    // Recent turn (not past horizon)
    let recent = Utc::now().to_rfc3339();
    insert_turn(store, &sid, 0, &recent);
    // Old envelope labeled raw_turn — must NOT drive stream A delete of the turn
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(200)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("raw_turn"),
        Some("turn"),
        Some(&format!("{sid}:0")),
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    // Turn is recent → not in raw_turn projection_delete from age
    let raw_proj = report
        .classes
        .iter()
        .find(|c| c.class == "raw_turn" && c.mechanism == "projection_delete");
    assert!(
        raw_proj.is_none() || raw_proj.unwrap().candidate_count == 0,
        "blob raw_turn must not force unlinked/recent turn projection delete"
    );
}

// ---------------------------------------------------------------------------
// Apply tests
// ---------------------------------------------------------------------------

#[test]
fn retention_apply__without_confirm__refused() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    let err = apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        RetentionApplyCommand {
            principal: principal(),
            scope: scope(),
            command_id: "ret-1".into(),
            confirm: false,
            dry_run: true,
        },
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("confirm") || msg.contains("dry_run"), "{msg}");
}

#[test]
fn retention_apply__raw_turns__deletes_projection_only() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);

    // Append a user prompt event so event log is non-empty
    let before_events = {
        let conn = store.connection().lock().unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        n
    };

    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    let report = apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        RetentionApplyCommand {
            principal: principal(),
            scope: scope(),
            command_id: "ret-turns-1".into(),
            confirm: true,
            dry_run: false,
        },
    )
    .unwrap();
    assert_eq!(report.mode, "apply");

    let conn = store.connection().lock().unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 0, "projection deleted");
    let after_events: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap();
    // Event log must not lose prior rows; RetentionApplied adds at least one
    assert!(after_events >= before_events);
}

#[test]
fn retention_apply__envelope__calls_wipe_not_parallel_ce() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );

    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        RetentionApplyCommand {
            principal: principal(),
            scope: scope(),
            command_id: "ret-ce-1".into(),
            confirm: true,
            dry_run: false,
        },
    )
    .unwrap();

    let conn = store.connection().lock().unwrap();
    let wrap = content_envelope::get_content_key_wrap(&conn, &key.to_string())
        .unwrap()
        .expect("key row");
    assert_eq!(wrap.status, "destroyed");
    assert!(wrap.wrap_nonce.is_none());
    assert!(wrap.wrap_ciphertext.is_none());
}

#[test]
fn retention_apply__appends_retention_applied_event() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        RetentionApplyCommand {
            principal: principal(),
            scope: scope(),
            command_id: "ret-audit-1".into(),
            confirm: true,
            dry_run: false,
        },
    )
    .unwrap();

    let events = store.read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::RetentionApplied),
        "R12 expected RetentionApplied"
    );
}

#[test]
fn retention_apply__hierarchy_parent_marked_for_resynthesis() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let parent = Uuid::new_v4().to_string();
    let child = Uuid::new_v4().to_string();
    insert_memory(store, &parent, "pinned", "parent-summary");
    insert_memory(store, &child, "active", "child-body");
    insert_hierarchy(store, &parent, &child);

    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        Some("memory"),
        Some(&child),
        &old,
    );

    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    let report = apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        RetentionApplyCommand {
            principal: principal(),
            scope: scope(),
            command_id: "ret-cascade-1".into(),
            confirm: true,
            dry_run: false,
        },
    )
    .unwrap();
    assert!(
        report.cascade.parents_marked_for_resynthesis >= 1,
        "R15 cascade: {report:?}"
    );

    let conn = store.connection().lock().unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM memory_projection WHERE memory_id = ?",
            [&parent],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "stale");
}

#[test]
fn retention_apply__idempotent_second_run() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old,
    );

    let side = StoreContentEnvelopeWipe::new(SqliteEventStore::new(store.connection().clone()));
    let cmd = || RetentionApplyCommand {
        principal: principal(),
        scope: scope(),
        command_id: "ret-idemp-1".into(),
        confirm: true,
        dry_run: false,
    };
    apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        cmd(),
    )
    .unwrap();
    // Second run: already erased; should not fail hard (or report zero CE work)
    let report = apply_retention(
        store,
        &ports.writer,
        &ports.query,
        &SystemClock,
        &AllowAllPolicy,
        &side,
        &config(),
        cmd(),
    )
    .unwrap();
    // Key already destroyed → not an active secret candidate; zero CE is fine
    assert_eq!(report.mode, "apply");
}

#[test]
fn nightly_default__no_ce_without_opt_in() {
    let mut cfg = RetentionConfig::default();
    assert!(!nightly_ce_enabled(&cfg));
    cfg.apply_ce_on_nightly = true;
    assert!(nightly_ce_enabled(&cfg));
}
