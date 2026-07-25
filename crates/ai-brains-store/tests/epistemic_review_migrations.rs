#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::ids::{
    ConclusionId, ConflictId, DecisionId, EvidenceId, PrincipalId, ReviewItemId,
};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ClaimConflictOpenedPayload, ConclusionProposedPayload, DecisionProposedPayload,
    ReviewItemOpenedPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use tempfile::NamedTempFile;
use time::OffsetDateTime;

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

fn column_exists(store: &SqliteEventStore, table: &str, column: &str) -> bool {
    let conn = store.connection().lock().unwrap();
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap();
    let rows = stmt.query_map([], |r| r.get::<_, String>(1)).unwrap();
    for row in rows {
        if row.unwrap() == column {
            return true;
        }
    }
    false
}

#[test]
fn epistemic_review_migrations__fresh_migrate__tables_and_valid_time_columns() {
    let (_tmp, store) = open_store();
    for table in [
        "conclusion_projection",
        "conclusion_evidence_projection",
        "decision_projection",
        "decision_support_projection",
        "review_item_projection",
        "claim_conflict_projection",
    ] {
        assert!(table_exists(&store, table), "missing table {table}");
    }
    assert!(column_exists(&store, "conclusion_projection", "valid_from"));
    assert!(column_exists(
        &store,
        "conclusion_projection",
        "valid_until"
    ));
    assert!(column_exists(&store, "decision_projection", "valid_from"));
    // Legacy conflict table still present.
    assert!(table_exists(&store, "conflict_projection"));
}

#[test]
fn epistemic_review_migrations__migrate_twice__idempotent() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    conn.migrate().unwrap(); // second pass
    let store = SqliteEventStore::new(conn);
    assert!(table_exists(&store, "conclusion_projection"));
}

#[test]
fn conclusion_projection__explicit_valid_from__stored_not_occurred_at() {
    let (_tmp, store) = open_store();
    let conclusion_id = ConclusionId::new();
    let valid_from = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();
    let occurred = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    assert_ne!(valid_from, occurred);

    let mut env = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id,
        statement: "domain claim".into(),
        evidence_ids: vec![EvidenceId::new()],
        proposer: PrincipalId::new(),
        valid_from: Some(valid_from),
        valid_until: None,
        scope: "Repository:abc".into(),
        protected_category: None,
        unsupported: false,
        model_provenance: None,
    }))
    .unwrap();
    env.occurred_at = occurred;
    store.append_event(&env).unwrap();

    let conn = store.connection().lock().unwrap();
    let (vf, vu, stmt): (String, Option<String>, String) = conn
        .query_row(
            "SELECT valid_from, valid_until, statement FROM conclusion_projection WHERE conclusion_id = ?",
            [conclusion_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert!(vf.starts_with("2020-"), "valid_from={vf}");
    assert!(vu.is_none());
    assert_eq!(stmt, "domain claim");
    // recorded_at should reflect occurred_at (~2023), distinct from valid_from
    let recorded: String = conn
        .query_row(
            "SELECT recorded_at FROM conclusion_projection WHERE conclusion_id = ?",
            [conclusion_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(recorded.starts_with("2023-"), "recorded_at={recorded}");
}

#[test]
fn conclusion_projection__omit_valid_from__defaults_to_occurred_at() {
    let (_tmp, store) = open_store();
    let conclusion_id = ConclusionId::new();
    let occurred = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    let mut env = EventBuilder::new(
        AggregateType::Conclusion,
        conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionProposed(ConclusionProposedPayload {
        conclusion_id,
        statement: "default valid".into(),
        evidence_ids: vec![],
        proposer: PrincipalId::new(),
        valid_from: None,
        valid_until: None,
        scope: String::new(),
        protected_category: None,
        unsupported: true,
        model_provenance: None,
    }))
    .unwrap();
    env.occurred_at = occurred;
    store.append_event(&env).unwrap();

    let conn = store.connection().lock().unwrap();
    let (vf, unsupported): (String, i64) = conn
        .query_row(
            "SELECT valid_from, unsupported FROM conclusion_projection WHERE conclusion_id = ?",
            [conclusion_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(
        vf.starts_with("2023-"),
        "valid_from should default to occurred_at={vf}"
    );
    assert_eq!(unsupported, 1);
}

#[test]
fn review_item_projection__opened__row_materialized() {
    let (_tmp, store) = open_store();
    let review_item_id = ReviewItemId::new();
    let env = EventBuilder::new(
        AggregateType::ReviewItem,
        review_item_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
        review_item_id,
        subject: "source changed".into(),
        opened_by: PrincipalId::new(),
        subject_kind: ReviewSubjectKind::Decision,
        subject_id: "dec-1".into(),
        criticality: ReviewCriticality::High,
        related_conclusion_id: None,
        related_decision_id: None,
        related_source_id: None,
    }))
    .unwrap();
    store.append_event(&env).unwrap();

    let conn = store.connection().lock().unwrap();
    let (status, crit): (String, String) = conn
        .query_row(
            "SELECT status, criticality FROM review_item_projection WHERE review_item_id = ?",
            [review_item_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(status, "Open");
    assert_eq!(crit, "High");
}

/// R1: stop at 0022, seed a source_projection row, apply 0023+, preserve prior + create epistemic tables.
#[test]
fn epistemic_review_migrations__from_0022__preserves_rows_and_creates_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0022_graph_governed_kinds")).unwrap();
    }

    assert!(
        table_exists_conn(&conn, "source_projection"),
        "0022 vault must already have source_projection"
    );
    assert!(
        !table_exists_conn(&conn, "conclusion_projection"),
        "0022-only vault must not yet have conclusion_projection"
    );

    {
        let locked = conn.lock().unwrap();
        locked
            .execute(
                "INSERT INTO source_projection (
                    source_id, scope, kind, display_name, locator, status, recorded_at, updated_at
                 ) VALUES (?, 'Personal:u1', 'File', 'seed', '/tmp/seed.md', 'Active',
                           '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                ["src-pre-0023"],
            )
            .unwrap();
        let count: i64 = locked
            .query_row("SELECT COUNT(*) FROM source_projection", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    // Forward migrate remaining (0023+).
    conn.migrate().unwrap();

    assert!(table_exists_conn(&conn, "conclusion_projection"));
    assert!(table_exists_conn(&conn, "decision_projection"));
    assert!(table_exists_conn(&conn, "review_item_projection"));
    assert!(table_exists_conn(&conn, "claim_conflict_projection"));

    let locked = conn.lock().unwrap();
    let source_count: i64 = locked
        .query_row("SELECT COUNT(*) FROM source_projection", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        source_count, 1,
        "pre-0023 source row must survive forward migrate"
    );
    let name: String = locked
        .query_row(
            "SELECT display_name FROM source_projection WHERE source_id = ?",
            ["src-pre-0023"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "seed");

    // valid_from column present on conclusion_projection after 0023.
    let mut stmt = locked
        .prepare("PRAGMA table_info(conclusion_projection)")
        .unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|c| c.unwrap())
        .collect();
    assert!(
        cols.iter().any(|c| c == "valid_from"),
        "conclusion_projection must have valid_from after 0023; cols={cols:?}"
    );
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

/// R2: rebuild_projections restores conclusion/decision/review/claim_conflict rows.
#[test]
fn rebuild_projections__restores_epistemic_lifecycle_rows() {
    let (_tmp, mut store) = open_store();
    let conclusion_id = ConclusionId::new();
    let decision_id = DecisionId::new();
    let review_item_id = ReviewItemId::new();
    let conflict_id = ConflictId::new();
    let principal = PrincipalId::new();
    let actor = Actor::System;
    let valid_from = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();

    for envelope in [
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "rebuild claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            proposer: principal,
            valid_from: Some(valid_from),
            valid_until: None,
            scope: "Repository:rebuild".into(),
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Decision,
            decision_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::DecisionProposed(DecisionProposedPayload {
            decision_id,
            title: "rebuild decision".into(),
            statement: "we decide".into(),
            proposer: principal,
            conclusion_ids: Some(vec![conclusion_id]),
            evidence_ids: None,
            valid_from: Some(valid_from),
            valid_until: None,
            scope: "Repository:rebuild".into(),
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::ReviewItem,
            review_item_id.as_uuid(),
            actor.clone(),
            Privacy::LocalOnly,
        )
        .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
            review_item_id,
            subject: "rebuild review".into(),
            opened_by: principal,
            subject_kind: ReviewSubjectKind::Conclusion,
            subject_id: conclusion_id.to_string(),
            criticality: ReviewCriticality::Medium,
            related_conclusion_id: Some(conclusion_id),
            related_decision_id: None,
            related_source_id: None,
        }))
        .unwrap(),
        EventBuilder::new(
            AggregateType::Conflict,
            conflict_id.as_uuid(),
            actor,
            Privacy::LocalOnly,
        )
        .build(Payload::ClaimConflictOpened(ClaimConflictOpenedPayload {
            conflict_id,
            claim_a_kind: "Conclusion".into(),
            claim_a_id: conclusion_id.to_string(),
            claim_b_kind: "Decision".into(),
            claim_b_id: decision_id.to_string(),
            scope: "Repository:rebuild".into(),
            explanation: "test open conflict".into(),
            valid_from: Some(valid_from),
            valid_until: None,
        }))
        .unwrap(),
    ] {
        store.append_event(&envelope).unwrap();
    }

    let before = {
        let conn = store.connection().lock().unwrap();
        let c: i64 = conn
            .query_row("SELECT COUNT(*) FROM conclusion_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let d: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_projection", [], |r| r.get(0))
            .unwrap();
        let r: i64 = conn
            .query_row("SELECT COUNT(*) FROM review_item_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let k: i64 = conn
            .query_row("SELECT COUNT(*) FROM claim_conflict_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        (c, d, r, k)
    };
    assert_eq!(before, (1, 1, 1, 1));

    {
        let conn = store.connection().lock().unwrap();
        conn.execute("DELETE FROM claim_conflict_projection", [])
            .unwrap();
        conn.execute("DELETE FROM decision_support_projection", [])
            .unwrap();
        conn.execute("DELETE FROM conclusion_evidence_projection", [])
            .unwrap();
        conn.execute("DELETE FROM review_item_projection", [])
            .unwrap();
        conn.execute("DELETE FROM decision_projection", []).unwrap();
        conn.execute("DELETE FROM conclusion_projection", [])
            .unwrap();
    }

    store.rebuild_projections().unwrap();

    let after = {
        let conn = store.connection().lock().unwrap();
        let c: i64 = conn
            .query_row("SELECT COUNT(*) FROM conclusion_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let d: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_projection", [], |r| r.get(0))
            .unwrap();
        let r: i64 = conn
            .query_row("SELECT COUNT(*) FROM review_item_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let k: i64 = conn
            .query_row("SELECT COUNT(*) FROM claim_conflict_projection", [], |r| {
                r.get(0)
            })
            .unwrap();
        let (stmt, vf): (String, String) = conn
            .query_row(
                "SELECT statement, valid_from FROM conclusion_projection WHERE conclusion_id = ?",
                [conclusion_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let title: String = conn
            .query_row(
                "SELECT title FROM decision_projection WHERE decision_id = ?",
                [decision_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM claim_conflict_projection WHERE conflict_id = ?",
                [conflict_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        (c, d, r, k, stmt, vf, title, status)
    };

    assert_eq!(after.0, 1);
    assert_eq!(after.1, 1);
    assert_eq!(after.2, 1);
    assert_eq!(after.3, 1);
    assert_eq!(after.4, "rebuild claim");
    assert!(
        after.5.starts_with("2020-"),
        "valid_from restored={:?}",
        after.5
    );
    assert_eq!(after.6, "rebuild decision");
    assert_eq!(after.7, "Open");
}
