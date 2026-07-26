#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T152 Phase B — migration 0025 + briefing cache / query trace projections.

use ai_brains_core::ids::{BriefingId, EvidenceId, QueryTraceId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{BriefingGeneratedPayload, QueryTraceRecordedPayload};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use ai_brains_store::projections::briefing::briefing_cache_key;
use tempfile::NamedTempFile;

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

#[test]
fn briefings_query_traces__fresh_vault__0025_tables_exist() {
    let (_tmp, store) = open_store();
    for table in [
        "briefing_cache_projection",
        "query_trace_projection",
        "retrieval_feedback_projection",
    ] {
        assert!(table_exists(&store, table), "missing table {table}");
    }
    // Prior migrations still present.
    assert!(table_exists(&store, "scope_grant_projection"));
    assert!(table_exists(&store, "conclusion_projection"));
    assert!(table_exists(&store, "events"));
}

#[test]
fn briefings_query_traces__migrate_twice__idempotent() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    conn.migrate().unwrap();

    let applied: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE name = '0025_briefings_query_traces'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 1, "migration row must appear exactly once");

    let store = SqliteEventStore::new(conn);
    assert!(table_exists(&store, "briefing_cache_projection"));
    assert!(table_exists(&store, "query_trace_projection"));
}

/// Stop at 0024, seed a grant table row, apply 0025+, preserve prior tables.
#[test]
fn briefings_query_traces__from_0024__preserves_prior_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0024_scopes_principals_grants")).unwrap();
    }

    assert!(
        table_exists_conn(&conn, "scope_grant_projection"),
        "0024 vault must have scope_grant_projection"
    );
    assert!(
        !table_exists_conn(&conn, "briefing_cache_projection"),
        "0024-only vault must not yet have briefing_cache_projection"
    );

    {
        let locked = conn.lock().unwrap();
        locked
            .execute(
                "INSERT INTO principal_projection (
                    principal_id, kind, display_name, bound_source_kinds, bound_capabilities,
                    recorded_at, updated_at
                 ) VALUES ('p1', 'Human', 'tester', '[]', '[]',
                           '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
            .unwrap();
    }

    conn.migrate().unwrap();

    assert!(table_exists_conn(&conn, "briefing_cache_projection"));
    assert!(table_exists_conn(&conn, "query_trace_projection"));
    assert!(table_exists_conn(&conn, "retrieval_feedback_projection"));

    let count: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM principal_projection WHERE principal_id = 'p1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "prior principal row must survive 0025 migrate");
}

#[test]
fn briefing_cache_key__includes_version_vector_and_budget() {
    let k1 = briefing_cache_key("Project", "Repository:a", "pol-1", "vv-1", 1500);
    let k2 = briefing_cache_key("Project", "Repository:a", "pol-1", "vv-2", 1500);
    let k3 = briefing_cache_key("Project", "Repository:a", "pol-1", "vv-1", 800);
    assert_ne!(k1, k2, "version vector advance must change cache key");
    assert_ne!(k1, k3, "budget change must change cache key");
    assert_eq!(
        k1,
        briefing_cache_key("Project", "Repository:a", "pol-1", "vv-1", 1500)
    );
}

#[test]
fn briefing_cache__version_vector_advance__cache_miss() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();

    let key_v1 = briefing_cache_key("Project", "Repository:scope-a", "1", "vv-1", 100);
    let key_v2 = briefing_cache_key("Project", "Repository:scope-a", "1", "vv-2", 100);

    conn.execute(
        "INSERT INTO briefing_cache_projection (
            cache_key, briefing_type, scope_key, policy_version, source_version_vector,
            budget, packet_json, generated_at, expires
         ) VALUES (?, 'Project', 'Repository:scope-a', '1', 'vv-1', 100, '{}',
                   '2026-01-01T00:00:00Z', NULL)",
        [&key_v1],
    )
    .unwrap();

    let hit: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM briefing_cache_projection WHERE cache_key = ?",
            [&key_v1],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hit, 1);

    let miss: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM briefing_cache_projection WHERE cache_key = ?",
            [&key_v2],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(miss, 0, "advanced version vector must miss cache");
}

#[test]
fn briefing_generated_and_query_trace__project_into_tables() {
    let (_tmp, mut store) = open_store();
    let briefing_id = BriefingId::new();
    let trace_id = QueryTraceId::new();
    let evidence_id = EvidenceId::new();

    let briefing_env = EventBuilder::new(
        AggregateType::Briefing,
        briefing_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::BriefingGenerated(BriefingGeneratedPayload {
        briefing_id,
        kind: "Project".into(),
        evidence_ids: vec![evidence_id],
        query_trace_id: Some(trace_id),
    }))
    .unwrap();

    let trace_env = EventBuilder::new(
        AggregateType::QueryTrace,
        trace_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::QueryTraceRecorded(QueryTraceRecordedPayload {
        query_trace_id: trace_id,
        query_text: "what is the briefing authority order?".into(),
        evidence_ids: vec![evidence_id],
        scope: "Repository:test".into(),
        principal_id: "principal-test".into(),
        applied_policy: "DefaultPolicyEvaluator".into(),
        ranking_json: r#"{"order":["policy","authority"]}"#.into(),
        freshness_summary: Some("fresh=1".into()),
        conflict_summary: None,
    }))
    .unwrap();

    EventStore::append_events(&store, &[briefing_env, trace_env]).unwrap();

    {
        let conn = store.connection().lock().unwrap();
        let cache_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM briefing_cache_projection WHERE briefing_type = 'Project'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cache_count, 1);

        let trace_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM query_trace_projection WHERE trace_id = ?",
                [trace_id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(trace_count, 1);

        let query: String = conn
            .query_row(
                "SELECT query FROM query_trace_projection WHERE trace_id = ?",
                [trace_id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert!(query.contains("authority"));
    }

    // Rebuild must rehydrate from events after truncate.
    store.rebuild_projections().unwrap();
    {
        let conn = store.connection().lock().unwrap();
        let cache_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM briefing_cache_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(cache_count, 1, "rebuild must rehydrate briefing cache");
        let trace_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM query_trace_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(trace_count, 1, "rebuild must rehydrate query traces");
    }
}

#[test]
fn briefing_cache__cache_key_uniqueness() {
    let (_tmp, store) = open_store();
    let conn = store.connection().lock().unwrap();
    let key = briefing_cache_key("Project", "Repository:x", "1", "vv", 10);
    conn.execute(
        "INSERT INTO briefing_cache_projection (
            cache_key, briefing_type, scope_key, policy_version, source_version_vector,
            budget, packet_json, generated_at, expires
         ) VALUES (?, 'Project', 'Repository:x', '1', 'vv', 10, '{}',
                   '2026-01-01T00:00:00Z', NULL)",
        [&key],
    )
    .unwrap();
    let err = conn
        .execute(
            "INSERT INTO briefing_cache_projection (
                cache_key, briefing_type, scope_key, policy_version, source_version_vector,
                budget, packet_json, generated_at, expires
             ) VALUES (?, 'Project', 'Repository:x', '1', 'vv', 10, '{\"x\":1}',
                       '2026-01-01T00:00:01Z', NULL)",
            [&key],
        )
        .expect_err("duplicate cache_key must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("UNIQUE") || msg.contains("unique"),
        "expected UNIQUE constraint, got: {msg}"
    );
}
