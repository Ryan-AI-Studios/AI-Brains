#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T166 — class-based retention plan/apply tests.

use ai_brains_contracts::retention::{
    CLASS_MEMORY_LEGACY, CLASS_RAW_TURN, MECHANISM_HELD, MECHANISM_SKIP,
    RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY,
};
use ai_brains_control_plane::{
    AllowAllPolicy, MAX_RETENTION_HORIZON_DAYS, RetentionApplyCommand, RetentionConfig,
    StoreContentEnvelopeWipe, StorePorts, SystemClock, apply_retention,
    apply_retention_projections, cascade_memory_ids_for_keys, execute_retention_projection_deletes,
    finalize_retention_apply, make_principal, nightly_ce_enabled, parse_positive_horizon_days,
    plan_retention, prepare_retention_apply,
};
use ai_brains_core::ids::{ContentKeyId, PrincipalId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_crypto::DataKey;
use ai_brains_events::EventKind;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use ai_brains_store::projections::content_envelope::{
    self, ALGORITHM_AES_256_GCM, ENVELOPE_SCHEMA_VERSION, EncryptedBlobRow,
};
use chrono::{Duration, Utc};
use rstest::rstest;
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
fn retention_plan__pinned_memories__memory_legacy_held_inventory() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let body = "t270-pin-body-must-not-appear";
    insert_memory(
        store,
        "aaaaaaaa-aaaa-aaaa-aaaa-000000000001",
        "pinned",
        body,
    );
    insert_memory(
        store,
        "aaaaaaaa-aaaa-aaaa-aaaa-000000000002",
        "pinned",
        body,
    );

    let report = plan_retention(store, &config()).unwrap();
    let legacy = report
        .classes
        .iter()
        .find(|c| c.class == CLASS_MEMORY_LEGACY)
        .expect("memory_legacy bucket");
    assert_eq!(legacy.mechanism, MECHANISM_HELD);
    assert_eq!(legacy.candidate_count, 2);
    assert_eq!(report.totals.would_held, 2);
    assert_eq!(report.totals.would_ce_wipe, 0);
    assert_eq!(report.totals.would_projection_delete, 0);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w == RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY),
        "inventory honesty const missing: {:?}",
        report.warnings
    );
    assert!(
        legacy
            .notes
            .iter()
            .any(|n| n == "inventory overlay; none_auto; pinned held (R11); other skip"),
        "F31 notes missing: {:?}",
        legacy.notes
    );
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains(body), "pin body leaked: {json}");
}

#[rstest]
#[case("forgotten")]
#[case("active")]
fn retention_plan__non_pinned_status__memory_legacy_skip(#[case] status: &str) {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    insert_memory(
        store,
        "bbbbbbbb-bbbb-bbbb-bbbb-000000000001",
        status,
        "t270-skip-body-must-not-appear",
    );
    insert_memory(
        store,
        "bbbbbbbb-bbbb-bbbb-bbbb-000000000002",
        status,
        "t270-skip-body-must-not-appear",
    );

    let report = plan_retention(store, &config()).unwrap();
    let legacy = report
        .classes
        .iter()
        .find(|c| c.class == CLASS_MEMORY_LEGACY)
        .expect("memory_legacy bucket");
    assert_eq!(legacy.mechanism, MECHANISM_SKIP);
    assert!(
        report.totals.would_skip >= 2,
        "would_skip expected >= 2; totals={:?}",
        report.totals
    );
    assert_eq!(report.totals.would_held, 0);
    assert_eq!(report.totals.would_ce_wipe, 0);
    assert_eq!(report.totals.would_projection_delete, 0);
    assert!(
        !legacy.sample_ids.is_empty(),
        "forgotten/active-only samples must be non-empty: {legacy:?}"
    );
    assert!(
        legacy
            .notes
            .iter()
            .any(|n| n == "inventory overlay; none_auto; pinned held (R11); other skip"),
        "F31 notes missing: {:?}",
        legacy.notes
    );
}

#[test]
fn retention_plan__mixed_pinned_and_other__one_bucket_split_totals() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    insert_memory(store, "cccccccc-cccc-cccc-cccc-000000000001", "pinned", "a");
    insert_memory(store, "cccccccc-cccc-cccc-cccc-000000000002", "pinned", "b");
    insert_memory(store, "cccccccc-cccc-cccc-cccc-000000000003", "pinned", "c");
    insert_memory(store, "cccccccc-cccc-cccc-cccc-000000000004", "active", "d");
    insert_memory(
        store,
        "cccccccc-cccc-cccc-cccc-000000000005",
        "forgotten",
        "e",
    );

    let report = plan_retention(store, &config()).unwrap();
    let buckets: Vec<_> = report
        .classes
        .iter()
        .filter(|c| c.class == CLASS_MEMORY_LEGACY)
        .collect();
    assert_eq!(buckets.len(), 1, "one memory_legacy bucket: {report:?}");
    assert_eq!(buckets[0].mechanism, MECHANISM_HELD);
    assert_eq!(buckets[0].candidate_count, 5);
    assert_eq!(report.totals.candidates, 5);
    assert_eq!(report.totals.would_held, 3);
    assert_eq!(report.totals.would_skip, 2);
}

#[test]
fn retention_plan__raw_turn_and_pinned__classes_sorted_memory_legacy_before_raw_turn() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);
    insert_memory(
        store,
        "dddddddd-dddd-dddd-dddd-000000000001",
        "pinned",
        "t270-sort",
    );

    let report = plan_retention(store, &config()).unwrap();
    let names: Vec<&str> = report.classes.iter().map(|c| c.class.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "classes must be sorted by class: {names:?}");
    let legacy_idx = names
        .iter()
        .position(|c| *c == CLASS_MEMORY_LEGACY)
        .expect("memory_legacy present");
    let turn_idx = names
        .iter()
        .position(|c| *c == CLASS_RAW_TURN)
        .expect("raw_turn present");
    assert!(
        legacy_idx < turn_idx,
        "memory_legacy must sort before raw_turn: {names:?}"
    );
}

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
fn retention_plan__linked_turn_held_or_skip_envelope__no_projection_delete() {
    // F-002 / R13: any known turn↔envelope join suppresses stream-A projection_delete
    // even when stream-B mechanism is held or within-horizon skip (not only ce_wipe).
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old_turn = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old_turn);

    let key = ContentKeyId::new();
    let recent = Utc::now().to_rfc3339();
    insert_active_key(store, &key, &recent);
    let turn_subject = format!("{sid}:0");
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        Some("turn"),
        Some(&turn_subject),
        &recent,
    );

    let report = plan_retention(store, &config()).unwrap();
    let raw_proj = report
        .classes
        .iter()
        .find(|c| c.class == "raw_turn" && c.mechanism == "projection_delete");
    assert!(
        raw_proj.is_none() || raw_proj.unwrap().candidate_count == 0,
        "R13: linked turn must not projection_delete when join known: {report:?}"
    );
}

#[test]
fn retention_plan__multi_subject_mixed_pin__held() {
    // F-003 / R11: any pinned memory subject holds the whole content_key.
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let pinned_id = Uuid::new_v4().to_string();
    let unpinned_id = Uuid::new_v4().to_string();
    insert_memory(store, &pinned_id, "pinned", "pinned-body");
    insert_memory(store, &unpinned_id, "active", "unpinned-body");
    let key = ContentKeyId::new();
    let old = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        Some("memory"),
        Some(&pinned_id),
        &old,
    );
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        Some("memory"),
        Some(&unpinned_id),
        &old,
    );

    let report = plan_retention(store, &config()).unwrap();
    assert!(
        report.totals.would_held >= 1,
        "mixed pin must hold key: {report:?}"
    );
    assert_eq!(
        report.totals.would_ce_wipe, 0,
        "must not ce_wipe when any subject pinned: {report:?}"
    );
}

#[test]
fn retention_apply__projections_only__defers_ce_no_local_destroy() {
    // F-001: production path uses prepare (no local wipe); wrap stays active.
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

    let outcome =
        prepare_retention_apply(store, &ports.writer, &config(), "ret-proj-1", true, false)
            .unwrap();
    assert!(
        !outcome.pending_ce_keys.is_empty(),
        "CE keys must be deferred: {outcome:?}"
    );
    assert_eq!(outcome.pending_ce_keys, vec![key.to_string()]);

    let conn = store.connection().lock().unwrap();
    let wrap = content_envelope::get_content_key_wrap(&conn, &key.to_string())
        .unwrap()
        .expect("key row");
    assert_eq!(wrap.status, "active", "prepare path must not destroy wrap");
    assert!(wrap.wrap_nonce.is_some());
}

#[test]
fn retention_apply__projections_append_audit_before_ce() {
    // Codex R2 P1 / R12: planned RetentionApplied is appended (pre-mutation) when CE pending.
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

    let outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-audit-pre-ce",
        true,
        false,
    )
    .unwrap();
    assert!(
        !outcome.pending_ce_keys.is_empty(),
        "CE still deferred: {outcome:?}"
    );
    assert!(
        outcome
            .report
            .warnings
            .iter()
            .any(|w| w.contains("ce_pending=") && w.contains("pre-mutation")),
        "pre-mutation CE warning expected: {:?}",
        outcome.report.warnings
    );

    let events = store.read_all_events().unwrap();
    let audits: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == EventKind::RetentionApplied)
        .collect();
    assert_eq!(
        audits.len(),
        1,
        "R12: exactly one pre-mutation RetentionApplied before finalize"
    );
}

#[test]
fn retention_apply__pinned_inventory__held_in_report_no_delete() {
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    insert_memory(
        store,
        "aaaaaaaa-aaaa-aaaa-aaaa-0000000000aa",
        "pinned",
        "apply-overlay-body",
    );
    insert_memory(
        store,
        "aaaaaaaa-aaaa-aaaa-aaaa-0000000000bb",
        "forgotten",
        "apply-overlay-skip",
    );

    let outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-t270-overlay",
        true,
        false,
    )
    .unwrap();
    let report = &outcome.report;
    let legacy = report
        .classes
        .iter()
        .find(|c| c.class == CLASS_MEMORY_LEGACY)
        .expect("memory_legacy on apply prepare");
    assert_eq!(legacy.mechanism, MECHANISM_HELD);
    assert_eq!(legacy.candidate_count, 2);
    assert_eq!(report.totals.would_held, 1);
    assert_eq!(report.totals.would_skip, 1);
    assert_eq!(report.totals.would_ce_wipe, 0);
    assert_eq!(report.totals.would_projection_delete, 0);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w == RETENTION_HONESTY_MEMORY_LEGACY_INVENTORY),
        "apply prepare must carry inventory honesty: {:?}",
        report.warnings
    );
    assert!(
        legacy
            .notes
            .iter()
            .any(|n| n == "inventory overlay; none_auto; pinned held (R11); other skip"),
        "F31 notes missing on apply: {:?}",
        legacy.notes
    );
    assert!(
        outcome.pending_ce_keys.is_empty(),
        "inventory must not enqueue CE: {outcome:?}"
    );
    assert!(
        outcome.turns_to_delete.is_empty(),
        "inventory must not enqueue projection deletes: {outcome:?}"
    );

    let conn = store.connection().lock().unwrap();
    let pinned: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let forgotten: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_projection WHERE status = 'forgotten'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pinned, 1, "apply prepare must not forget pins");
    assert_eq!(forgotten, 1, "apply prepare must not delete forgotten rows");
    let json = serde_json::to_string(report).unwrap();
    assert!(!json.contains("apply-overlay-body"));
}

#[test]
fn retention_apply__prepare__leaves_turns_present() {
    // CE-first split: prepare audits but does not delete projections.
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);

    let outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-prep-turns",
        true,
        false,
    )
    .unwrap();
    assert!(
        outcome.report.totals.would_projection_delete >= 1,
        "expected projection candidate: {outcome:?}"
    );
    assert!(
        !outcome.turns_to_delete.is_empty(),
        "turns must be deferred for execute: {outcome:?}"
    );

    let events = store.read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::RetentionApplied),
        "R12: RetentionApplied must exist after prepare (before deletes)"
    );

    let conn = store.connection().lock().unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 1, "prepare must not delete projections");
}

#[test]
fn retention_apply__execute__removes_deferred_turns() {
    // prepare leaves turns; execute removes them.
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);

    let mut outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-exec-turns",
        true,
        false,
    )
    .unwrap();
    {
        let conn = store.connection().lock().unwrap();
        let turns: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
                [&sid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(turns, 1, "still present after prepare");
    }

    execute_retention_projection_deletes(store, &mut outcome).unwrap();

    let conn = store.connection().lock().unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 0, "execute removes deferred projection deletes");
}

#[test]
fn retention_apply__projections_audit_before_projection_delete() {
    // Convenience apply_retention_projections = prepare + execute (projection-only).
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old);

    let outcome = apply_retention_projections(
        store,
        &ports.writer,
        &config(),
        "ret-audit-pre-proj",
        true,
        false,
    )
    .unwrap();
    assert!(
        outcome.report.totals.would_projection_delete >= 1,
        "expected projection candidate: {outcome:?}"
    );

    let events = store.read_all_events().unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventKind::RetentionApplied),
        "R12: RetentionApplied must exist after apply (appended before deletes)"
    );

    let conn = store.connection().lock().unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 0, "projection deleted after pre-mutation audit");
}

#[test]
fn retention_apply__prepare__ce_and_projection__defers_both() {
    // CE-first: prepare with both CE + projection candidates deletes neither.
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let sid = Uuid::new_v4().to_string();
    let old_turn = (Utc::now() - Duration::days(120)).to_rfc3339();
    insert_turn(store, &sid, 0, &old_turn);
    let key = ContentKeyId::new();
    let old_ce = (Utc::now() - Duration::days(30)).to_rfc3339();
    insert_active_key(store, &key, &old_ce);
    insert_blob(
        store,
        &key,
        &Uuid::new_v4().to_string(),
        Some("secret"),
        None,
        None,
        &old_ce,
    );

    let outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-ce-first-prep",
        true,
        false,
    )
    .unwrap();
    assert!(
        !outcome.pending_ce_keys.is_empty(),
        "CE deferred: {outcome:?}"
    );
    assert!(
        !outcome.turns_to_delete.is_empty(),
        "projections deferred: {outcome:?}"
    );

    let conn = store.connection().lock().unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turn_projection WHERE session_id = ?",
            [&sid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 1, "prepare must not delete turns before CE");
    let wrap = content_envelope::get_content_key_wrap(&conn, &key.to_string())
        .unwrap()
        .expect("key row");
    assert_eq!(wrap.status, "active", "prepare must not wipe CE");
}

#[test]
fn finalize_retention_apply__appends_final_audit_and_cascades() {
    // Codex R1 P3: finalize appends RetentionApplied (second audit after pre-CE).
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let parent = Uuid::new_v4().to_string();
    let child = Uuid::new_v4().to_string();
    insert_memory(store, &parent, "active", "parent-summary");
    insert_memory(store, &child, "active", "child-body");
    insert_hierarchy(store, &parent, &child);

    // Pre-CE audit (as production prepare path does — no projection deletes yet).
    let outcome = prepare_retention_apply(
        store,
        &ports.writer,
        &config(),
        "ret-finalize-1",
        true,
        false,
    )
    .unwrap();
    let pre_ce_audits = store
        .read_all_events()
        .unwrap()
        .iter()
        .filter(|e| e.event_type == EventKind::RetentionApplied)
        .count();
    assert!(pre_ce_audits >= 1, "pre-CE audit required before finalize");

    // Simulate successful CE subjects only (failed keys excluded by CLI).
    let mut report = outcome.report;
    finalize_retention_apply(
        store,
        &ports.writer,
        "ret-finalize-1",
        std::slice::from_ref(&child),
        &mut report,
    )
    .unwrap();

    assert!(
        report.cascade.parents_marked_for_resynthesis >= 1,
        "cascade should mark parent: {report:?}"
    );
    let audits = store
        .read_all_events()
        .unwrap()
        .iter()
        .filter(|e| e.event_type == EventKind::RetentionApplied)
        .count();
    assert!(
        audits >= 2,
        "pre-CE + finalize RetentionApplied expected, got {audits}"
    );

    let conn = store.connection().lock().unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM memory_projection WHERE memory_id = ?",
            [&parent],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "stale", "parent must be marked stale");
}

#[test]
fn finalize_retention_apply__failed_ce_keys_do_not_cascade() {
    // Codex R1 P2: cascade empty when no successful CE keys.
    let (_tmp, ports) = open_ports();
    let store = store_of(&ports);
    let parent = Uuid::new_v4().to_string();
    let child = Uuid::new_v4().to_string();
    insert_memory(store, &parent, "active", "parent-summary");
    insert_memory(store, &child, "active", "child-body");
    insert_hierarchy(store, &parent, &child);

    let mut report = plan_retention(store, &config()).unwrap();
    report.mode = "apply".into();
    report
        .errors
        .push("ce_wipe abcd1234…: daemon unavailable".into());
    // Empty cascade list = failed CE subjects filtered out.
    finalize_retention_apply(store, &ports.writer, "ret-no-cascade", &[], &mut report).unwrap();

    assert_eq!(report.cascade.parents_marked_for_resynthesis, 0);
    let conn = store.connection().lock().unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM memory_projection WHERE memory_id = ?",
            [&parent],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "active", "failed CE must not stale parent");
}

#[test]
fn cascade_memory_ids_for_keys__only_successful_keys() {
    // Codex R1 P2: map subjects per key; filter to successful CE only.
    let mut by_key = std::collections::BTreeMap::new();
    by_key.insert("key-ok".into(), vec!["mem-a".into(), "mem-b".into()]);
    by_key.insert("key-fail".into(), vec!["mem-c".into()]);

    let ids = cascade_memory_ids_for_keys(&by_key, ["key-ok"]);
    assert_eq!(ids, vec!["mem-a".to_string(), "mem-b".to_string()]);

    let none = cascade_memory_ids_for_keys(&by_key, ["key-fail-not-present"]);
    assert!(none.is_empty());
}

#[test]
fn parse_positive_horizon_days__rejects_non_positive_and_huge() {
    assert_eq!(parse_positive_horizon_days("90").unwrap(), 90);
    assert!(parse_positive_horizon_days("0").is_err());
    assert!(parse_positive_horizon_days("-7").is_err());
    assert!(parse_positive_horizon_days("not-a-number").is_err());
    assert!(parse_positive_horizon_days(&(MAX_RETENTION_HORIZON_DAYS + 1).to_string()).is_err());
    assert_eq!(
        parse_positive_horizon_days(&MAX_RETENTION_HORIZON_DAYS.to_string()).unwrap(),
        MAX_RETENTION_HORIZON_DAYS
    );
}

#[test]
fn retention_config_from_env__negative_falls_back_to_default() {
    // Codex R1 P1: negative env must not produce future cutoffs / panic.
    let _g1 = TempEnv::set("AI_BRAINS_RETENTION_SECRET_DAYS", "-1");
    let _g2 = TempEnv::set("AI_BRAINS_RETENTION_RAW_TURN_DAYS", "0");
    let _g3 = TempEnv::set("AI_BRAINS_RETENTION_EVIDENCE_DAYS", "999999999");
    let _g4 = TempEnv::set("AI_BRAINS_RETENTION_QUERY_TRACE_DAYS", "45");

    let cfg = RetentionConfig::from_env();
    let defaults = RetentionConfig::default();
    assert_eq!(
        cfg.secret_days, defaults.secret_days,
        "negative must fall back"
    );
    assert_eq!(
        cfg.raw_turn_days, defaults.raw_turn_days,
        "zero must fall back"
    );
    assert_eq!(
        cfg.evidence_days, defaults.evidence_days,
        "huge must fall back"
    );
    assert_eq!(cfg.query_trace_days, 45, "valid override accepted");
    assert!(cfg.secret_days > 0);
    // Cutoff construction must not panic for sanitized config.
    let now = Utc::now();
    let _ = now - Duration::days(cfg.secret_days);
    let _ = now - Duration::days(cfg.raw_turn_days);
}

#[test]
fn nightly_default__no_ce_without_opt_in() {
    let mut cfg = RetentionConfig::default();
    assert!(!nightly_ce_enabled(&cfg));
    cfg.apply_ce_on_nightly = true;
    assert!(nightly_ce_enabled(&cfg));
}
