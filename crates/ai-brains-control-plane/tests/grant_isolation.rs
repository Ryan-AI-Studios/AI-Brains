#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_control_plane::{
    ControlPlaneError, PolicyEvaluator, ProposeConclusionRequest, RemoteIdentityKey,
    ScopeIdentityStore, StorePorts, SystemClock, issue_grant, join_repository, make_principal,
    propose_conclusion, rebind_path_alias, register_path_alias, register_principal,
    register_workspace, revoke_grant, scope_identity_key, upsert_repository_identity,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId, UserId, WorkspaceId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_events::Payload;
use ai_brains_git::hash_remote_url;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use tempfile::NamedTempFile;

fn open_ports() -> (NamedTempFile, StorePorts) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let key = DataKey::generate();
    let sql_key = ai_brains_crypto::SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db_path, &sql_key).unwrap();
    conn.migrate().unwrap();
    let store = SqliteEventStore::new(conn);
    (temp_file, StorePorts::from_store(store))
}

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent-a")
}

#[test]
fn grant_isolation__agent_granted_project_a_only__propose_b_denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;

    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let hash_a = hash_remote_url("https://github.com/org/project-a.git").unwrap();
    let hash_b = hash_remote_url("https://github.com/org/project-b.git").unwrap();
    assert_ne!(hash_a, hash_b);

    let identity = ports.identity_store();
    // Register identities (normalized hashes); second same-hash without force would conflict.
    upsert_repository_identity(
        &ports.writer,
        &identity,
        project_a,
        RemoteIdentityKey::NormalizedHash(hash_a.clone()),
        false,
    )
    .unwrap();
    upsert_repository_identity(
        &ports.writer,
        &identity,
        project_b,
        RemoteIdentityKey::NormalizedHash(hash_b),
        false,
    )
    .unwrap();

    // Same hash → different project without force must error (no dual identity).
    let err = upsert_repository_identity(
        &ports.writer,
        &identity,
        ProjectId::new(),
        RemoteIdentityKey::NormalizedHash(hash_a),
        false,
    )
    .unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::IdentityConflict(_)),
        "expected IdentityConflict, got {err:?}"
    );

    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();

    // Grant agent only project A ProposeConclusion.
    let grant_id = issue_grant(
        &ports.writer,
        &clock,
        agent_p.id,
        ScopeRef::Repository(project_a),
        GrantCapability::ProposeConclusion,
        Privacy::LocalOnly,
    )
    .unwrap();
    assert!(!grant_id.to_string().is_empty());

    // Production path only — never AllowAllPolicy for isolation checks.
    let policy = ports.production_policy();
    let scope_a = ScopeRef::Repository(project_a);
    let scope_b = ScopeRef::Repository(project_b);

    // Propose on A OK.
    let res_a = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope_a.clone(),
            statement: "claim on A".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .expect("propose A must succeed");
    assert!(!res_a.unsupported);

    // Propose on B denied.
    let err_b = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope_b.clone(),
            statement: "claim on B must not leak".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap_err();
    assert!(
        matches!(err_b, ControlPlaneError::PolicyDenied(_)),
        "expected PolicyDenied, got {err_b:?}"
    );

    // Policy log for deny has reason codes only — never statement text.
    let grant_store = ports.grant_store();
    let logs = grant_store.list_policy_decisions(agent_p.id, 10).unwrap();
    assert!(logs.iter().any(|e| !e.allowed));
    for e in &logs {
        assert!(
            !e.reason_code.contains("claim"),
            "log must not contain claim text: {}",
            e.reason_code
        );
        assert!(
            !e.reason_code.contains("leak"),
            "log must not contain statement text"
        );
    }

    // No Personal leakage: active grants for agent do not include Personal scopes.
    let keys = grant_store
        .list_active_grant_scope_keys(agent_p.id)
        .unwrap();
    assert!(
        keys.iter().all(|k| !k.starts_with("Personal:")),
        "agent must not have Personal grants: {keys:?}"
    );
    assert!(keys.contains(&scope_identity_key(&scope_a)));
    assert!(!keys.contains(&scope_identity_key(&scope_b)));
}

#[test]
fn grant_isolation__no_personal_scope_without_grant() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();

    let personal = ScopeRef::Personal(UserId::new());
    let policy = ports.production_policy();
    let allowed = policy
        .allow(
            agent_p.id,
            GrantCapability::ProposeConclusion,
            &personal,
            &ai_brains_control_plane::PolicyContext::unspecified(),
        )
        .unwrap();
    assert!(!allowed, "Personal must not be accessible without grant");

    let keys = ports
        .grant_store()
        .list_active_grant_scope_keys(agent_p.id)
        .unwrap();
    assert!(
        keys.iter().all(|k| !k.starts_with("Personal:")),
        "no Personal leakage in grant listing"
    );
}

#[test]
fn grant_isolation__workspace_register_and_join() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let ws = WorkspaceId::new();
    let project = ProjectId::new();
    register_workspace(&ports.writer, &clock, ws, "acme").unwrap();
    join_repository(&ports.writer, &clock, ws, project).unwrap();

    let store = ports.store();
    let conn = store.connection().lock().unwrap();
    let name: String = conn
        .query_row(
            "SELECT name FROM workspace_projection WHERE workspace_id = ?",
            [ws.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "acme");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workspace_repository_projection
             WHERE workspace_id = ? AND project_id = ?",
            [ws.to_string(), project.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn grant_isolation__revoke_removes_access() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let grant_id = issue_grant(
        &ports.writer,
        &clock,
        agent_p.id,
        scope.clone(),
        GrantCapability::ProposeConclusion,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.production_policy();
    assert!(
        policy
            .allow(
                agent_p.id,
                GrantCapability::ProposeConclusion,
                &scope,
                &ai_brains_control_plane::PolicyContext::unspecified(),
            )
            .unwrap()
    );

    revoke_grant(&ports.writer, &clock, grant_id, "test revoke").unwrap();

    // Fresh evaluator against same vault.
    let policy2 = ports.policy_evaluator();
    assert!(
        !policy2
            .allow(
                agent_p.id,
                GrantCapability::ProposeConclusion,
                &scope,
                &ai_brains_control_plane::PolicyContext::unspecified(),
            )
            .unwrap()
    );
}

#[test]
fn grant_isolation__path_alias_registration() {
    let (_t, ports) = open_ports();
    let project = ProjectId::new();
    register_path_alias(&ports.writer, r"C:\Dev\AliasProj", project).unwrap();
    register_path_alias(&ports.writer, "/mnt/c/Dev/AliasProj", project).unwrap();

    let id_store = ports.identity_store();
    let win = ai_brains_path::normalize_for_location_compare(r"C:\Dev\AliasProj");
    let found = id_store.find_by_path_alias(&win).unwrap();
    assert_eq!(found, Some(project));
}

#[test]
fn rebind_path_alias__from_eq_to__invalid_payload() {
    let (_t, ports) = open_ports();
    let from = ProjectId::new();
    let err = rebind_path_alias(&ports.writer, r"C:\Dev\Same", from, from).unwrap_err();
    assert!(
        matches!(err, ControlPlaneError::InvalidPayload(_)),
        "AC18: from==to is InvalidPayload; got {err:?}"
    );
    let events = EventStore::read_all_events(&ports.store()).unwrap();
    assert!(
        events.iter().all(|e| {
            !matches!(
                e.payload,
                Payload::RepositoryPathAliasRemoved(_) | Payload::RepositoryPathAliasAdded(_)
            )
        }),
        "AC18: no path events appended"
    );
}

#[test]
fn rebind_path_alias__appends_removed_then_added() {
    let (_t, ports) = open_ports();
    let from = ProjectId::new();
    let to = ProjectId::new();
    assert_ne!(from, to);
    register_path_alias(&ports.writer, r"C:\Dev\MoveMe", from).unwrap();
    rebind_path_alias(&ports.writer, r"C:\Dev\MoveMe", from, to).unwrap();

    let events = EventStore::read_all_events(&ports.store()).unwrap();
    let path_events: Vec<&Payload> = events
        .iter()
        .filter_map(|e| match &e.payload {
            p @ (Payload::RepositoryPathAliasRemoved(_) | Payload::RepositoryPathAliasAdded(_)) => {
                Some(p)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        path_events.len(),
        3,
        "register Added + rebind Removed+Added"
    );
    match path_events[1] {
        Payload::RepositoryPathAliasRemoved(p) => {
            assert_eq!(p.project_id, from);
        }
        other => panic!("second path event must be Removed; got {other:?}"),
    }
    match path_events[2] {
        Payload::RepositoryPathAliasAdded(p) => {
            assert_eq!(p.project_id, to);
        }
        other => panic!("third path event must be Added; got {other:?}"),
    }

    let id_store = ports.identity_store();
    let key = ai_brains_path::normalize_for_location_compare(r"C:\Dev\MoveMe");
    assert_eq!(id_store.find_by_path_alias(&key).unwrap(), Some(to));
}

#[test]
fn grant_isolation__upsert_from_raw_url_uses_normalized_hash() {
    let (_t, ports) = open_ports();
    let project = ProjectId::new();
    let identity = ports.identity_store();
    upsert_repository_identity(
        &ports.writer,
        &identity,
        project,
        RemoteIdentityKey::RawUrl("https://GitHub.com/Org/Repo.git".into()),
        false,
    )
    .unwrap();

    let expected = hash_remote_url("git@github.com:Org/Repo").unwrap();
    let id_store = ports.identity_store();
    assert_eq!(
        id_store.find_by_remote_hash(&expected).unwrap(),
        Some(project)
    );
}

#[test]
fn grant_isolation__force_rebind_moves_remote_hash_to_new_project() {
    let (_t, ports) = open_ports();
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let hash = hash_remote_url("https://github.com/org/rebind.git").unwrap();
    let identity = ports.identity_store();

    upsert_repository_identity(
        &ports.writer,
        &identity,
        project_a,
        RemoteIdentityKey::NormalizedHash(hash.clone()),
        false,
    )
    .unwrap();
    assert_eq!(
        identity.find_by_remote_hash(&hash).unwrap(),
        Some(project_a)
    );

    upsert_repository_identity(
        &ports.writer,
        &identity,
        project_b,
        RemoteIdentityKey::NormalizedHash(hash.clone()),
        true,
    )
    .unwrap();
    assert_eq!(
        identity.find_by_remote_hash(&hash).unwrap(),
        Some(project_b)
    );
}

#[test]
fn grant_isolation__empty_remote_url__rejected() {
    let (_t, ports) = open_ports();
    let identity = ports.identity_store();
    let err = upsert_repository_identity(
        &ports.writer,
        &identity,
        ProjectId::new(),
        RemoteIdentityKey::RawUrl("   ".into()),
        false,
    )
    .unwrap_err();
    assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
}

/// Event-path grant privacy: LocalOnly grant + Cloud route denies Export for human.
#[test]
fn grant_isolation__issue_grant_local_only_privacy__cloud_export_denied() {
    use ai_brains_control_plane::{
        ConnectorTrust, GrantPrincipalStore, PolicyContext, ProcessingRoute, make_principal,
        register_principal,
    };
    use ai_brains_core::principal::PrincipalKind;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "owner");
    register_principal(&ports.writer, &clock, &human).unwrap();
    let scope = ScopeRef::Repository(ProjectId::new());

    issue_grant(
        &ports.writer,
        &clock,
        human.id,
        scope.clone(),
        GrantCapability::Export,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Projection stored LocalOnly privacy.
    let grant_store = ports.grant_store();
    let grants = grant_store.active_grants(human.id, &scope).unwrap();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].privacy, Privacy::LocalOnly);

    let policy = ports.production_policy();
    let ctx = PolicyContext {
        privacy: Privacy::CloudOk,
        connector_trust: Some(ConnectorTrust::CloudOk),
        route: Some(ProcessingRoute::Cloud),
        source_kind: None,
    };
    assert!(
        !policy
            .allow(human.id, GrantCapability::Export, &scope, &ctx)
            .unwrap(),
        "LocalOnly grant privacy must block Cloud route Export"
    );
    let logs = grant_store.list_policy_decisions(human.id, 5).unwrap();
    assert!(
        logs.iter()
            .any(|e| !e.allowed && e.reason_code == "privacy_route_mismatch"),
        "expected privacy_route_mismatch, got {logs:?}"
    );
}

/// Principal kind PascalCase labels round-trip event → projection → GrantPrincipalStore.
#[test]
fn grant_isolation__principal_kind_pascalcase__roundtrip_all_kinds() {
    use ai_brains_control_plane::{GrantPrincipalStore, make_principal, register_principal};
    use ai_brains_core::principal::{Principal, PrincipalKind};

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let kinds = [
        PrincipalKind::Human,
        PrincipalKind::Agent,
        PrincipalKind::Connector,
        PrincipalKind::System,
        PrincipalKind::Service,
        PrincipalKind::Other("custom".into()),
    ];
    let mut registered: Vec<Principal> = Vec::new();
    for kind in kinds {
        let name = format!("{kind:?}");
        let p = make_principal(kind.clone(), PrincipalId::new(), &name);
        register_principal(&ports.writer, &clock, &p).unwrap();
        registered.push(p);
    }

    let store = ports.grant_store();
    for expected in &registered {
        let loaded = store
            .get_principal(expected.id)
            .unwrap()
            .expect("principal must project");
        assert_eq!(
            loaded.kind, expected.kind,
            "kind round-trip failed for {:?}",
            expected.kind
        );
        assert_eq!(loaded.display_name, expected.display_name);
    }
}

#[test]
fn grant_isolation__identity_and_alias_survive_rebuild() {
    let (_t, ports) = open_ports();
    let project = ProjectId::new();
    let hash = hash_remote_url("https://github.com/org/rebuild-me.git").unwrap();
    let identity = ports.identity_store();
    upsert_repository_identity(
        &ports.writer,
        &identity,
        project,
        RemoteIdentityKey::NormalizedHash(hash.clone()),
        false,
    )
    .unwrap();
    register_path_alias(&ports.writer, r"C:\Dev\RebuildMe", project).unwrap();

    let mut store = ports.store();
    store.rebuild_projections().unwrap();

    let id_store = ports.identity_store();
    assert_eq!(id_store.find_by_remote_hash(&hash).unwrap(), Some(project));
    let path = ai_brains_path::normalize_for_location_compare(r"C:\Dev\RebuildMe");
    assert_eq!(id_store.find_by_path_alias(&path).unwrap(), Some(project));
}

/// Policy audit is event-sourced: allow/deny produce events; rebuild restores log rows.
#[test]
fn grant_isolation__policy_decision_log__survives_rebuild() {
    use ai_brains_control_plane::{PolicyContext, make_principal, register_principal};
    use ai_brains_core::principal::PrincipalKind;
    use ai_brains_store::event_store::EventStore;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "auditor");
    register_principal(&ports.writer, &clock, &human).unwrap();
    let scope = ScopeRef::Repository(ProjectId::new());

    issue_grant(
        &ports.writer,
        &clock,
        human.id,
        scope.clone(),
        GrantCapability::ReadEvidence,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.production_policy();
    let ctx = PolicyContext::unspecified();

    // Allow path (granted ReadEvidence).
    assert!(
        policy
            .allow(human.id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap()
    );
    // Deny path (no Export grant).
    assert!(
        !policy
            .allow(human.id, GrantCapability::Export, &scope, &ctx)
            .unwrap()
    );

    let grant_store = ports.grant_store();
    let before = grant_store.list_policy_decisions(human.id, 20).unwrap();
    assert!(
        before
            .iter()
            .any(|e| e.allowed && e.reason_code == "allowed"),
        "expected allow log, got {before:?}"
    );
    assert!(
        before
            .iter()
            .any(|e| !e.allowed && e.reason_code == "missing_grant"),
        "expected deny log with reason_code, got {before:?}"
    );
    for e in &before {
        assert!(
            !e.reason_code.contains("claim") && !e.reason_code.contains("statement"),
            "audit must not contain claim/statement text: {}",
            e.reason_code
        );
    }

    // Events must exist in the log (not only projection side-writes).
    let store = ports.store();
    let events = EventStore::read_all_events(&store).unwrap();
    let policy_events: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e.payload,
                ai_brains_events::Payload::PolicyDecisionRecorded(_)
            )
        })
        .collect();
    assert!(
        policy_events.len() >= 2,
        "expected ≥2 PolicyDecisionRecorded events, got {}",
        policy_events.len()
    );

    let mut store = ports.store();
    store.rebuild_projections().unwrap();

    let after = ports
        .grant_store()
        .list_policy_decisions(human.id, 20)
        .unwrap();
    assert_eq!(
        after.len(),
        before.len(),
        "rebuild must restore policy_decision_log rows (before={before:?}, after={after:?})"
    );
    assert!(
        after
            .iter()
            .any(|e| e.allowed && e.reason_code == "allowed"),
        "allow row missing after rebuild: {after:?}"
    );
    assert!(
        after
            .iter()
            .any(|e| !e.allowed && e.reason_code == "missing_grant"),
        "deny row missing after rebuild: {after:?}"
    );
}
