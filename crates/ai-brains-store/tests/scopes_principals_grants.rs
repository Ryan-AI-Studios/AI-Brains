#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T151 Phase B — migration 0024 + workspace/principal/grant projections.

use ai_brains_core::ids::{GrantId, PrincipalId, ProjectId, WorkspaceId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    PrincipalRegisteredPayload, RepositoryJoinedWorkspacePayload, ScopeGrantIssuedPayload,
    ScopeGrantRevokedPayload, WorkspaceRegisteredPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::apply_migrations_through;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
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

#[test]
fn scopes_principals_grants__fresh_vault__0024_tables_exist() {
    let (_tmp, store) = open_store();
    for table in [
        "workspace_projection",
        "workspace_repository_projection",
        "repository_identity_projection",
        "repository_path_alias_projection",
        "principal_projection",
        "scope_grant_projection",
        "policy_decision_log",
    ] {
        assert!(table_exists(&store, table), "missing table {table}");
    }
    assert!(
        index_exists(&store, "idx_scope_grant_active_unique"),
        "partial unique index on active grants required"
    );
    assert!(
        index_exists(&store, "idx_repository_identity_remote_hash"),
        "unique index on normalized remote_url_hash required"
    );
    // Prior migrations still present.
    assert!(table_exists(&store, "conclusion_projection"));
    assert!(table_exists(&store, "events"));
}

#[test]
fn scopes_principals_grants__migrate_twice__idempotent() {
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
            "SELECT COUNT(*) FROM schema_migrations WHERE name = '0024_scopes_principals_grants'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(applied, 1, "migration row must appear exactly once");

    let store = SqliteEventStore::new(conn);
    assert!(table_exists(&store, "scope_grant_projection"));
    assert!(table_exists(&store, "principal_projection"));
}

/// Stop at 0023, seed conclusion + decision rows, apply 0024+, preserve epistemic tables.
#[test]
fn scopes_principals_grants__from_0023__preserves_epistemic_tables() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();

    {
        let mut locked = conn.lock().unwrap();
        apply_migrations_through(&mut locked, Some("0023_epistemic_review")).unwrap();
    }

    assert!(
        table_exists_conn(&conn, "conclusion_projection"),
        "0023 vault must have conclusion_projection"
    );
    assert!(
        !table_exists_conn(&conn, "workspace_projection"),
        "0023-only vault must not yet have workspace_projection"
    );

    {
        let locked = conn.lock().unwrap();
        locked
            .execute(
                "INSERT INTO conclusion_projection (
                    conclusion_id, state, statement, scope, privacy, proposer,
                    valid_from, valid_until, recorded_at, updated_at,
                    supersedes, superseded_by, protected_category, unsupported
                 ) VALUES (
                    'concl-pre-0024', 'Active', 'seed claim', 'Personal:u1', 'LocalOnly',
                    'principal-1', '2026-01-01T00:00:00Z', NULL,
                    '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z',
                    NULL, NULL, NULL, 0
                 )",
                [],
            )
            .unwrap();
        locked
            .execute(
                "INSERT INTO decision_projection (
                    decision_id, state, title, statement, scope, proposer,
                    recorded_at, updated_at
                 ) VALUES (
                    'dec-pre-0024', 'Proposed', 'seed', 'body', 'Personal:u1',
                    'principal-1', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'
                 )",
                [],
            )
            .unwrap();
    }

    // Forward migrate remaining (0024+).
    conn.migrate().unwrap();

    assert!(table_exists_conn(&conn, "workspace_projection"));
    assert!(table_exists_conn(&conn, "principal_projection"));
    assert!(table_exists_conn(&conn, "scope_grant_projection"));
    assert!(table_exists_conn(&conn, "policy_decision_log"));
    assert!(table_exists_conn(&conn, "repository_identity_projection"));

    let locked = conn.lock().unwrap();
    let conclusion_count: i64 = locked
        .query_row("SELECT COUNT(*) FROM conclusion_projection", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        conclusion_count, 1,
        "pre-0024 conclusion row must survive forward migrate"
    );
    let statement: String = locked
        .query_row(
            "SELECT statement FROM conclusion_projection WHERE conclusion_id = ?",
            ["concl-pre-0024"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(statement, "seed claim");

    let decision_count: i64 = locked
        .query_row("SELECT COUNT(*) FROM decision_projection", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        decision_count, 1,
        "pre-0024 decision row must survive forward migrate"
    );
}

#[test]
fn scopes_principals_grants__project_workspace_principal_grant__rows_materialized() {
    let (_tmp, store) = open_store();
    let workspace_id = WorkspaceId::new();
    let project_id = ProjectId::new();
    let principal_id = PrincipalId::new();
    let grant_id = GrantId::new();
    let actor = Actor::System;

    let ws = EventBuilder::new(
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::WorkspaceRegistered(WorkspaceRegisteredPayload {
        workspace_id,
        name: "acme-ws".into(),
    }))
    .unwrap();
    store.append_event(&ws).unwrap();

    let join = EventBuilder::new(
        AggregateType::Workspace,
        workspace_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::RepositoryJoinedWorkspace(
        RepositoryJoinedWorkspacePayload {
            workspace_id,
            project_id,
        },
    ))
    .unwrap();
    store.append_event(&join).unwrap();

    let principal = EventBuilder::new(
        AggregateType::Principal,
        principal_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::PrincipalRegistered(PrincipalRegisteredPayload {
        principal_id,
        kind: "Agent".into(),
        display_name: "codex".into(),
        bound_source_kinds: Vec::new(),
        bound_capabilities: vec![GrantCapability::ProposeConclusion],
    }))
    .unwrap();
    store.append_event(&principal).unwrap();

    let grant = EventBuilder::new(
        AggregateType::Grant,
        grant_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
        grant_id,
        principal_id,
        scope: ScopeRef::Repository(project_id),
        capability: GrantCapability::ProposeConclusion,
        privacy: Privacy::LocalOnly,
    }))
    .unwrap();
    store.append_event(&grant).unwrap();

    let conn = store.connection().lock().unwrap();

    let (name, privacy): (String, String) = conn
        .query_row(
            "SELECT name, privacy FROM workspace_projection WHERE workspace_id = ?",
            [workspace_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(name, "acme-ws");
    assert_eq!(privacy, "CloudOk");

    let membership: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workspace_repository_projection
             WHERE workspace_id = ? AND project_id = ?",
            [workspace_id.to_string(), project_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(membership, 1);

    let (kind, display, caps): (String, String, String) = conn
        .query_row(
            "SELECT kind, display_name, bound_capabilities
             FROM principal_projection WHERE principal_id = ?",
            [principal_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(kind, "Agent");
    assert_eq!(display, "codex");
    assert!(
        caps.contains("ProposeConclusion"),
        "bound_capabilities JSON must include ProposeConclusion, got: {caps}"
    );

    let expected_scope = format!("Repository:{project_id}");
    let (scope_key, capability, privacy, revoked): (String, String, String, Option<String>) = conn
        .query_row(
            "SELECT scope_key, capability, privacy, revoked_at
             FROM scope_grant_projection WHERE grant_id = ?",
            [grant_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(scope_key, expected_scope);
    assert_eq!(capability, "ProposeConclusion");
    assert_eq!(
        privacy, "LocalOnly",
        "ScopeGrantIssued privacy projects as LocalOnly by default"
    );
    assert!(revoked.is_none(), "issued grant must be active");
}

#[test]
fn scopes_principals_grants__revoke__sets_revoked_at() {
    let (_tmp, store) = open_store();
    let principal_id = PrincipalId::new();
    let grant_id = GrantId::new();
    let project_id = ProjectId::new();
    let actor = Actor::System;

    let issued = EventBuilder::new(
        AggregateType::Grant,
        grant_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
        grant_id,
        principal_id,
        scope: ScopeRef::Repository(project_id),
        capability: GrantCapability::ReadEvidence,
        privacy: Privacy::LocalOnly,
    }))
    .unwrap();
    store.append_event(&issued).unwrap();

    let revoked = EventBuilder::new(
        AggregateType::Grant,
        grant_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ScopeGrantRevoked(ScopeGrantRevokedPayload {
        grant_id,
        reason: "rotated".into(),
    }))
    .unwrap();
    store.append_event(&revoked).unwrap();

    let conn = store.connection().lock().unwrap();
    let revoked_at: Option<String> = conn
        .query_row(
            "SELECT revoked_at FROM scope_grant_projection WHERE grant_id = ?",
            [grant_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(revoked_at.is_some(), "revoke must set revoked_at; got NULL");

    let active: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM scope_grant_projection
             WHERE principal_id = ? AND revoked_at IS NULL",
            [principal_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active, 0, "no active grant after revoke");
}

/// Choice: partial unique index **prevents** a second active grant for the same
/// (principal, scope, capability). Upsert is only by grant_id (idempotent replay).
#[test]
fn scopes_principals_grants__duplicate_active_grant__unique_index_prevents() {
    let (_tmp, store) = open_store();
    let principal_id = PrincipalId::new();
    let project_id = ProjectId::new();
    let grant_a = GrantId::new();
    let grant_b = GrantId::new();
    let actor = Actor::System;
    let scope = ScopeRef::Repository(project_id);

    let first = EventBuilder::new(
        AggregateType::Grant,
        grant_a.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
        grant_id: grant_a,
        principal_id,
        scope: scope.clone(),
        capability: GrantCapability::ReadConclusions,
        privacy: Privacy::LocalOnly,
    }))
    .unwrap();
    store.append_event(&first).unwrap();

    let second = EventBuilder::new(
        AggregateType::Grant,
        grant_b.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
        grant_id: grant_b,
        principal_id,
        scope,
        capability: GrantCapability::ReadConclusions,
        privacy: Privacy::LocalOnly,
    }))
    .unwrap();

    let err = store
        .append_event(&second)
        .expect_err("second active grant same principal/scope/capability must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("UNIQUE")
            || msg.contains("unique")
            || msg.contains("Failed to append")
            || msg.contains("constraint"),
        "expected unique-constraint failure, got: {msg}"
    );
}

#[test]
fn scopes_principals_grants__reissue_after_revoke__allowed() {
    let (_tmp, store) = open_store();
    let principal_id = PrincipalId::new();
    let project_id = ProjectId::new();
    let grant_a = GrantId::new();
    let grant_b = GrantId::new();
    let actor = Actor::System;
    let scope = ScopeRef::Repository(project_id);

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Grant,
                grant_a.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
                grant_id: grant_a,
                principal_id,
                scope: scope.clone(),
                capability: GrantCapability::Export,
                privacy: Privacy::LocalOnly,
            }))
            .unwrap(),
        )
        .unwrap();

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Grant,
                grant_a.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::ScopeGrantRevoked(ScopeGrantRevokedPayload {
                grant_id: grant_a,
                reason: "expired".into(),
            }))
            .unwrap(),
        )
        .unwrap();

    // New grant_id, same principal/scope/capability after revoke — partial unique OK.
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Grant,
                grant_b.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
                grant_id: grant_b,
                principal_id,
                scope,
                capability: GrantCapability::Export,
                privacy: Privacy::LocalOnly,
            }))
            .unwrap(),
        )
        .unwrap();

    let conn = store.connection().lock().unwrap();
    let active: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM scope_grant_projection
             WHERE principal_id = ? AND revoked_at IS NULL",
            [principal_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active, 1);
}

#[test]
fn scopes_principals_grants__rebuild_projections__restores_rows() {
    let (_tmp, mut store) = open_store();
    let workspace_id = WorkspaceId::new();
    let principal_id = PrincipalId::new();
    let grant_id = GrantId::new();
    let actor = Actor::System;

    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Workspace,
                workspace_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::WorkspaceRegistered(WorkspaceRegisteredPayload {
                workspace_id,
                name: "rebuild-ws".into(),
            }))
            .unwrap(),
        )
        .unwrap();
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Principal,
                principal_id.as_uuid(),
                actor.clone(),
                Privacy::LocalOnly,
            )
            .build(Payload::PrincipalRegistered(PrincipalRegisteredPayload {
                principal_id,
                kind: "Human".into(),
                display_name: "owner".into(),
                bound_source_kinds: Vec::new(),
                bound_capabilities: Vec::new(),
            }))
            .unwrap(),
        )
        .unwrap();
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Grant,
                grant_id.as_uuid(),
                actor,
                Privacy::LocalOnly,
            )
            .build(Payload::ScopeGrantIssued(ScopeGrantIssuedPayload {
                grant_id,
                principal_id,
                scope: ScopeRef::Workspace(workspace_id),
                capability: GrantCapability::ApproveDecision,
                privacy: Privacy::LocalOnly,
            }))
            .unwrap(),
        )
        .unwrap();

    {
        let conn = store.connection().lock().unwrap();
        conn.execute("DELETE FROM scope_grant_projection", [])
            .unwrap();
        conn.execute("DELETE FROM principal_projection", [])
            .unwrap();
        conn.execute("DELETE FROM workspace_projection", [])
            .unwrap();
    }

    store.rebuild_projections().unwrap();

    let conn = store.connection().lock().unwrap();
    let ws_name: String = conn
        .query_row(
            "SELECT name FROM workspace_projection WHERE workspace_id = ?",
            [workspace_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(ws_name, "rebuild-ws");

    let display: String = conn
        .query_row(
            "SELECT display_name FROM principal_projection WHERE principal_id = ?",
            [principal_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(display, "owner");

    let cap: String = conn
        .query_row(
            "SELECT capability FROM scope_grant_projection WHERE grant_id = ?",
            [grant_id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cap, "ApproveDecision");
}
