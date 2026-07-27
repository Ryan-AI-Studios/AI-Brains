#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use std::collections::HashMap;
use std::sync::Mutex;

use ai_brains_control_plane::{
    ConnectorTrust, DefaultPolicyEvaluator, GrantPrincipalStore, PolicyContext,
    PolicyDecisionEntry, PolicyEvaluator, ProcessingRoute, Result, scope_identity_key,
};
use ai_brains_core::ids::{PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeGrant, ScopeRef};
use ai_brains_core::source::SourceKind;
use uuid::Uuid;

#[derive(Default)]
struct MemGrantStore {
    principals: Mutex<HashMap<PrincipalId, Principal>>,
    /// principal → list of grants
    grants: Mutex<HashMap<PrincipalId, Vec<ScopeGrant>>>,
    log: Mutex<Vec<PolicyDecisionEntry>>,
}

impl MemGrantStore {
    fn register(&self, p: Principal) {
        self.principals.lock().unwrap().insert(p.id, p);
    }

    fn grant(&self, principal: PrincipalId, scope: ScopeRef, capability: GrantCapability) {
        self.grants
            .lock()
            .unwrap()
            .entry(principal)
            .or_default()
            .push(ScopeGrant {
                scope,
                capability,
                privacy: Privacy::LocalOnly,
            });
    }

    fn grant_with_privacy(
        &self,
        principal: PrincipalId,
        scope: ScopeRef,
        capability: GrantCapability,
        privacy: Privacy,
    ) {
        self.grants
            .lock()
            .unwrap()
            .entry(principal)
            .or_default()
            .push(ScopeGrant {
                scope,
                capability,
                privacy,
            });
    }

    fn last_log(&self) -> Option<PolicyDecisionEntry> {
        self.log.lock().unwrap().last().cloned()
    }

    fn logs(&self) -> Vec<PolicyDecisionEntry> {
        self.log.lock().unwrap().clone()
    }
}

impl GrantPrincipalStore for MemGrantStore {
    fn get_principal(&self, id: PrincipalId) -> Result<Option<Principal>> {
        Ok(self.principals.lock().unwrap().get(&id).cloned())
    }

    fn active_grants(&self, principal: PrincipalId, scope: &ScopeRef) -> Result<Vec<ScopeGrant>> {
        let all = self.grants.lock().unwrap();
        let Some(list) = all.get(&principal) else {
            return Ok(Vec::new());
        };
        Ok(list.iter().filter(|g| &g.scope == scope).cloned().collect())
    }

    fn log_policy_decision(&self, entry: PolicyDecisionEntry) -> Result<()> {
        // Invariant: never store claim/statement text in reason_code or fields.
        assert!(
            !entry.reason_code.to_lowercase().contains("statement"),
            "denial log must not contain claim text"
        );
        assert!(
            !entry.reason_code.contains(' '),
            "reason_code should be a machine code, got {}",
            entry.reason_code
        );
        self.log.lock().unwrap().push(entry);
        Ok(())
    }
}

impl GrantPrincipalStore for &MemGrantStore {
    fn get_principal(&self, id: PrincipalId) -> Result<Option<Principal>> {
        (*self).get_principal(id)
    }

    fn active_grants(&self, principal: PrincipalId, scope: &ScopeRef) -> Result<Vec<ScopeGrant>> {
        (*self).active_grants(principal, scope)
    }

    fn log_policy_decision(&self, entry: PolicyDecisionEntry) -> Result<()> {
        (*self).log_policy_decision(entry)
    }
}

fn principal(kind: PrincipalKind, name: &str) -> Principal {
    Principal {
        id: PrincipalId::new(),
        kind,
        display_name: name.into(),
        bound_source_kinds: Vec::new(),
        bound_capabilities: Vec::new(),
    }
}

fn repo_scope() -> ScopeRef {
    ScopeRef::Repository(ProjectId::from_uuid(Uuid::from_u128(1)))
}

fn other_repo() -> ScopeRef {
    ScopeRef::Repository(ProjectId::from_uuid(Uuid::from_u128(2)))
}

fn ctx() -> PolicyContext {
    PolicyContext::unspecified()
}

#[test]
fn policy_matrix__unknown_principal__deny() {
    let store = MemGrantStore::default();
    let eval = DefaultPolicyEvaluator::new(&store);
    let unknown = PrincipalId::new();
    let allowed = eval
        .allow(
            unknown,
            GrantCapability::ReadEvidence,
            &repo_scope(),
            &ctx(),
        )
        .unwrap();
    assert!(!allowed);
    let log = store.last_log().expect("logged");
    assert!(!log.allowed);
    assert_eq!(log.reason_code, "unknown_principal");
    assert_eq!(log.principal_id, unknown);
    assert_eq!(log.scope_key, scope_identity_key(&repo_scope()));
}

#[test]
fn policy_matrix__agent_propose_on_granted_scope__allow() {
    let store = MemGrantStore::default();
    let agent = principal(PrincipalKind::Agent, "agent");
    let scope = repo_scope();
    store.register(agent.clone());
    store.grant(agent.id, scope.clone(), GrantCapability::ProposeConclusion);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        eval.allow(agent.id, GrantCapability::ProposeConclusion, &scope, &ctx())
            .unwrap()
    );
    assert_eq!(store.last_log().unwrap().reason_code, "allowed");
}

#[test]
fn policy_matrix__agent_propose_on_other_scope__deny() {
    let store = MemGrantStore::default();
    let agent = principal(PrincipalKind::Agent, "agent");
    store.register(agent.clone());
    store.grant(agent.id, repo_scope(), GrantCapability::ProposeConclusion);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(
                agent.id,
                GrantCapability::ProposeConclusion,
                &other_repo(),
                &ctx()
            )
            .unwrap()
    );
    assert_eq!(store.last_log().unwrap().reason_code, "missing_grant");
}

#[test]
fn policy_matrix__agent_approve_decision__hard_deny() {
    let store = MemGrantStore::default();
    let agent = principal(PrincipalKind::Agent, "agent");
    let scope = repo_scope();
    store.register(agent.clone());
    // Even with an ApproveDecision grant row, agent hard-denies Approve*.
    store.grant(agent.id, scope.clone(), GrantCapability::ApproveDecision);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(agent.id, GrantCapability::ApproveDecision, &scope, &ctx())
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "agent_cannot_approve"
    );
}

#[test]
fn policy_matrix__human_approve_decision_granted__allow() {
    let store = MemGrantStore::default();
    let human = principal(PrincipalKind::Human, "human");
    let scope = repo_scope();
    store.register(human.clone());
    store.grant(human.id, scope.clone(), GrantCapability::ApproveDecision);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        eval.allow(human.id, GrantCapability::ApproveDecision, &scope, &ctx())
            .unwrap()
    );
}

#[test]
fn policy_matrix__connector_observe_granted_kind_bound__allow() {
    let store = MemGrantStore::default();
    let mut connector = principal(PrincipalKind::Connector, "claude");
    connector.bound_source_kinds = vec![SourceKind::File];
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::File);
    assert!(
        eval.allow(connector.id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap()
    );
}

#[test]
fn policy_matrix__connector_observe_kind_unbound__deny() {
    let store = MemGrantStore::default();
    let mut connector = principal(PrincipalKind::Connector, "claude");
    connector.bound_source_kinds = vec![SourceKind::File];
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    // Unbound kind for this connector.
    ctx.source_kind = Some(SourceKind::GitRepository);
    assert!(
        !eval
            .allow(connector.id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "connector_source_kind_unbound"
    );
}

#[test]
fn policy_matrix__connector_empty_bound_kinds__deny() {
    let store = MemGrantStore::default();
    let connector = principal(PrincipalKind::Connector, "unbound");
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::File);
    assert!(
        !eval
            .allow(connector.id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap()
    );
}

#[test]
fn policy_matrix__system_unlisted_cap_empty_bound__deny() {
    let store = MemGrantStore::default();
    let system = principal(PrincipalKind::System, "nightly");
    // empty bound_capabilities
    store.register(system.clone());
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(
                system.id,
                GrantCapability::ProposeConclusion,
                &repo_scope(),
                &ctx()
            )
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "system_cap_not_bound"
    );
}

#[test]
fn policy_matrix__system_bound_only_no_grant__deny() {
    let store = MemGrantStore::default();
    let mut system = principal(PrincipalKind::System, "nightly");
    system.bound_capabilities = vec![GrantCapability::ReadEvidence];
    store.register(system.clone());
    // Non-empty bound requires grant as well (intersection / least privilege).
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(
                system.id,
                GrantCapability::ReadEvidence,
                &repo_scope(),
                &ctx()
            )
            .unwrap()
    );
    assert_eq!(store.last_log().unwrap().reason_code, "missing_grant");
}

#[test]
fn policy_matrix__system_grant_only_empty_bound__allow() {
    let store = MemGrantStore::default();
    let system = principal(PrincipalKind::System, "nightly");
    // empty bound_capabilities — grant alone is enough
    store.register(system.clone());
    store.grant(system.id, repo_scope(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        eval.allow(
            system.id,
            GrantCapability::ReadEvidence,
            &repo_scope(),
            &ctx()
        )
        .unwrap()
    );
}

#[test]
fn policy_matrix__system_grant_not_in_nonempty_bound__deny() {
    let store = MemGrantStore::default();
    let mut system = principal(PrincipalKind::System, "nightly");
    system.bound_capabilities = vec![GrantCapability::ReadEvidence];
    store.register(system.clone());
    // Grant for a capability outside the bound set → deny.
    store.grant(system.id, repo_scope(), GrantCapability::ProposeConclusion);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(
                system.id,
                GrantCapability::ProposeConclusion,
                &repo_scope(),
                &ctx()
            )
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "system_cap_not_bound"
    );
}

#[test]
fn policy_matrix__system_bound_and_grant__allow() {
    let store = MemGrantStore::default();
    let mut system = principal(PrincipalKind::System, "nightly");
    system.bound_capabilities = vec![GrantCapability::ReadEvidence];
    store.register(system.clone());
    store.grant(system.id, repo_scope(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        eval.allow(
            system.id,
            GrantCapability::ReadEvidence,
            &repo_scope(),
            &ctx()
        )
        .unwrap()
    );
}

#[test]
fn policy_matrix__agent_export_grant__hard_deny() {
    let store = MemGrantStore::default();
    let agent = principal(PrincipalKind::Agent, "agent");
    let scope = repo_scope();
    store.register(agent.clone());
    store.grant(agent.id, scope.clone(), GrantCapability::Export);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(agent.id, GrantCapability::Export, &scope, &ctx())
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "agent_cap_not_read_or_propose"
    );
}

#[test]
fn policy_matrix__human_export_local_only_cloud_route__deny_privacy() {
    let store = MemGrantStore::default();
    let human = principal(PrincipalKind::Human, "owner");
    let scope = ScopeRef::Personal(UserId::from_uuid(Uuid::from_u128(5)));
    store.register(human.clone());
    store.grant(human.id, scope.clone(), GrantCapability::Export);
    let eval = DefaultPolicyEvaluator::new(&store);
    let ctx = PolicyContext {
        privacy: Privacy::LocalOnly,
        connector_trust: Some(ConnectorTrust::LocalOnly),
        route: Some(ProcessingRoute::Cloud),
        source_kind: None,
    };
    assert!(
        !eval
            .allow(human.id, GrantCapability::Export, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "privacy_route_mismatch"
    );
}

#[test]
fn policy_matrix__service_without_grant__deny() {
    let store = MemGrantStore::default();
    let service = principal(PrincipalKind::Service, "svc");
    store.register(service.clone());
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        !eval
            .allow(
                service.id,
                GrantCapability::ReadConclusions,
                &repo_scope(),
                &ctx()
            )
            .unwrap()
    );
}

#[test]
fn policy_matrix__service_with_grant__allow() {
    let store = MemGrantStore::default();
    let service = principal(PrincipalKind::Service, "svc");
    let scope = repo_scope();
    store.register(service.clone());
    store.grant(service.id, scope.clone(), GrantCapability::ReadConclusions);
    let eval = DefaultPolicyEvaluator::new(&store);
    assert!(
        eval.allow(service.id, GrantCapability::ReadConclusions, &scope, &ctx())
            .unwrap()
    );
}

#[test]
fn policy_matrix__denial_log_has_no_claim_text_fields() {
    let store = MemGrantStore::default();
    let agent = principal(PrincipalKind::Agent, "agent");
    store.register(agent.clone());
    let eval = DefaultPolicyEvaluator::new(&store);
    let _ = eval
        .allow(
            agent.id,
            GrantCapability::ProposeConclusion,
            &repo_scope(),
            &ctx(),
        )
        .unwrap();
    for entry in store.logs() {
        // Fields are structured codes only — no free-form claim bodies.
        assert!(
            entry
                .reason_code
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "reason_code must be machine-safe: {}",
            entry.reason_code
        );
        assert!(!entry.scope_key.contains("claim"));
        assert!(!entry.scope_key.contains("statement"));
    }
}

#[test]
fn policy_matrix__connector_propose_with_grant__hard_deny() {
    let store = MemGrantStore::default();
    let mut connector = principal(PrincipalKind::Connector, "claude");
    connector.bound_source_kinds = vec![SourceKind::File];
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(
        connector.id,
        scope.clone(),
        GrantCapability::ProposeConclusion,
    );
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::File);
    assert!(
        !eval
            .allow(
                connector.id,
                GrantCapability::ProposeConclusion,
                &scope,
                &ctx
            )
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "connector_cap_not_observe"
    );
}

#[test]
fn policy_matrix__connector_export_with_grant__hard_deny() {
    let store = MemGrantStore::default();
    let mut connector = principal(PrincipalKind::Connector, "claude");
    connector.bound_source_kinds = vec![SourceKind::File];
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::Export);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::File);
    assert!(
        !eval
            .allow(connector.id, GrantCapability::Export, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "connector_cap_not_observe"
    );
}

#[test]
fn policy_matrix__connector_trust_local_only_cloud_route__deny() {
    let store = MemGrantStore::default();
    let human = principal(PrincipalKind::Human, "owner");
    let scope = repo_scope();
    store.register(human.clone());
    store.grant(human.id, scope.clone(), GrantCapability::Export);
    let eval = DefaultPolicyEvaluator::new(&store);
    // Content privacy may be CloudOk, but connector_trust LocalOnly + Cloud route denies.
    let ctx = PolicyContext {
        privacy: Privacy::CloudOk,
        connector_trust: Some(ConnectorTrust::LocalOnly),
        route: Some(ProcessingRoute::Cloud),
        source_kind: None,
    };
    assert!(
        !eval
            .allow(human.id, GrantCapability::Export, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "connector_trust_route_mismatch"
    );
}

#[test]
fn policy_matrix__grant_privacy_strictest_blocks_cloud() {
    let store = MemGrantStore::default();
    let human = principal(PrincipalKind::Human, "h");
    let scope = repo_scope();
    store.register(human.clone());
    // Content privacy CloudOk but grant is LocalOnly → combined LocalOnly blocks Cloud route.
    store.grant_with_privacy(
        human.id,
        scope.clone(),
        GrantCapability::Export,
        Privacy::LocalOnly,
    );
    let eval = DefaultPolicyEvaluator::new(&store);
    let ctx = PolicyContext {
        privacy: Privacy::CloudOk,
        connector_trust: None,
        route: Some(ProcessingRoute::Cloud),
        source_kind: None,
    };
    assert!(
        !eval
            .allow(human.id, GrantCapability::Export, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "privacy_route_mismatch"
    );
}

// --- T153: connector registry principal binding ↔ policy ---

#[test]
fn connector_registry__principal_id_bound__read_evidence_allowed() {
    use ai_brains_sources::{
        InProcessConnectorRegistry, MOCK_CONNECTOR_ID, MockConnector, principal_id_for_connector,
    };

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(MockConnector::new()))
        .expect("register mock");
    let bound_id = reg
        .get_manifest(MOCK_CONNECTOR_ID)
        .expect("manifest")
        .principal_id
        .expect("principal_id bound");
    assert_eq!(bound_id, principal_id_for_connector(MOCK_CONNECTOR_ID));

    // Kind bindings from the connector manifest (File).
    let kinds = reg
        .get(MOCK_CONNECTOR_ID)
        .expect("connector")
        .manifest()
        .source_kinds
        .clone();
    assert_eq!(kinds, vec![SourceKind::File]);

    let store = MemGrantStore::default();
    let connector = Principal {
        id: bound_id,
        kind: PrincipalKind::Connector,
        display_name: "Mock Connector".into(),
        bound_source_kinds: kinds,
        bound_capabilities: Vec::new(),
    };
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::File);
    // Trust label parity: sources ConnectorTrustLabel::LocalOnly ↔ ConnectorTrust::LocalOnly
    ctx.connector_trust = Some(ConnectorTrust::LocalOnly);
    assert!(
        eval.allow(bound_id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap(),
        "registry-bound principal with grant + bound File kind must allow ReadEvidence"
    );
}

#[test]
fn connector_observe__unbound_source_kind__policy_denied() {
    use ai_brains_sources::{InProcessConnectorRegistry, MOCK_CONNECTOR_ID, MockConnector};

    let mut reg = InProcessConnectorRegistry::new();
    reg.register(Box::new(MockConnector::new()))
        .expect("register mock");
    let bound_id = reg
        .get_manifest(MOCK_CONNECTOR_ID)
        .expect("manifest")
        .principal_id
        .expect("principal_id bound");

    let store = MemGrantStore::default();
    let connector = Principal {
        id: bound_id,
        kind: PrincipalKind::Connector,
        display_name: "Mock Connector".into(),
        // Bound only to File (from mock manifest); GitRepository is unbound.
        bound_source_kinds: vec![SourceKind::File],
        bound_capabilities: Vec::new(),
    };
    let scope = repo_scope();
    store.register(connector.clone());
    store.grant(connector.id, scope.clone(), GrantCapability::ReadEvidence);
    let eval = DefaultPolicyEvaluator::new(&store);
    let mut ctx = ctx();
    ctx.source_kind = Some(SourceKind::GitRepository);
    assert!(
        !eval
            .allow(bound_id, GrantCapability::ReadEvidence, &scope, &ctx)
            .unwrap()
    );
    assert_eq!(
        store.last_log().unwrap().reason_code,
        "connector_source_kind_unbound"
    );
}
