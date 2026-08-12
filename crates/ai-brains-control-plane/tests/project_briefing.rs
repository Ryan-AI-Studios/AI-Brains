#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T152 Phase C — deterministic Project Briefing service.

use ai_brains_control_plane::{
    AllowAllPolicy, BudgetConfig, Clock, OpenClaimConflictRequest, ProjectBriefingRequest,
    StoreEventWriter, StorePorts, SystemClock, activate_conclusion, approve_decision,
    build_project_briefing, issue_grant, make_principal, open_claim_conflict, propose_conclusion,
    propose_decision, register_principal, render_project_markdown, revoke_grant,
    scope_identity_key, try_mark_stale_payload,
};
use ai_brains_control_plane::{
    ProposeConclusionRequest, ProposeDecisionRequest, ScopeResolveInput,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload};
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::EventStore;
use std::path::PathBuf;
use tempfile::NamedTempFile;

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

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent")
}
fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

fn grant_reads(ports: &StorePorts, principal: PrincipalId, scope: ScopeRef) {
    let clock = SystemClock;
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
        GrantCapability::ApproveConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            principal,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
}

#[test]
fn project_briefing__scope_a__includes_decision_and_active_excludes_stale() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let scope_a = ScopeRef::Repository(project_a);
    let scope_b = ScopeRef::Repository(project_b);
    let key_a = scope_identity_key(&scope_a);

    let human_p = human();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    grant_reads(&ports, human_p.id, scope_a.clone());
    grant_reads(&ports, agent_p.id, scope_a.clone());
    // Agent has grant only on A, not B.
    grant_reads(&ports, human_p.id, scope_b.clone());

    let policy = ports.production_policy();

    // Approved decision on A.
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: human_p.clone(),
            scope: scope_a.clone(),
            title: "Ship briefings".into(),
            statement: "Use deterministic briefings".into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![EvidenceId::new()]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Active conclusion on A.
    let active = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope_a.clone(),
            statement: "Authority order is policy-first".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &agent_p,
        active.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Stale conclusion on A (propose + activate + mark stale).
    let stale = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope_a.clone(),
            statement: "Old claim that became stale".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &agent_p,
        stale.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let stale_payload =
        try_mark_stale_payload(stale.conclusion_id, None, Some("source changed".into())).unwrap();
    let env = EventBuilder::new(
        AggregateType::Conclusion,
        stale.conclusion_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ConclusionMarkedStale(stale_payload))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[env]).unwrap();

    // Open conflict on A.
    open_claim_conflict(
        &ports.writer,
        &policy,
        &human_p,
        OpenClaimConflictRequest {
            claim_a_kind: "Conclusion".into(),
            claim_a_id: active.conclusion_id.to_string(),
            claim_b_kind: "Conclusion".into(),
            claim_b_id: stale.conclusion_id.to_string(),
            scope: key_a.clone(),
            explanation: "overlapping claims".into(),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            conflict_id: None,
        },
    )
    .unwrap();

    let identity = ports.identity_store();
    let packet = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: agent_p.clone(),
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project_a),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert_eq!(packet.kind, "Project");
    assert!(packet.scope.authoritative);
    assert!(!packet.denied);
    assert_eq!(packet.decisions.len(), 1);
    assert_eq!(packet.decisions[0].state, "Approved");
    assert!(!packet.decisions[0].evidence_handles.is_empty());
    assert!(
        packet
            .conclusions
            .iter()
            .any(|c| c.id == active.conclusion_id.to_string() && c.state == "Active")
    );
    assert!(
        packet
            .conclusions
            .iter()
            .all(|c| c.state != "Stale" && c.state != "Disputed" && c.state != "Rejected")
    );
    assert!(
        packet.warnings.iter().any(|w| w.kind == "stale"
            && w.subject_id.as_deref() == Some(&stale.conclusion_id.to_string()))
    );
    assert!(packet.warnings.iter().any(|w| w.kind == "open_conflict"));

    let md = render_project_markdown(&packet);
    assert!(md.contains("Decisions"));
    assert!(!md.contains("Old claim that became stale") || md.contains("Warnings"));
}

#[test]
fn project_briefing__scope_b_without_grant__denied_empty() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project_a = ProjectId::new();
    let project_b = ProjectId::new();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    grant_reads(&ports, agent_p.id, ScopeRef::Repository(project_a));

    // Seed an approved decision on B under AllowAll so data exists but agent has no grant.
    {
        let human_p = human();
        register_principal(&ports.writer, &clock, &human_p).unwrap();
        let dec = propose_decision(
            &ports.writer,
            &ports.query,
            &clock,
            &AllowAllPolicy,
            ProposeDecisionRequest {
                principal: human_p.clone(),
                scope: ScopeRef::Repository(project_b),
                title: "Secret B".into(),
                statement: "must not leak to agent".into(),
                conclusion_ids: None,
                evidence_ids: Some(vec![EvidenceId::new()]),
                privacy: Privacy::LocalOnly,
                valid_from: None,
                valid_until: None,
                decision_id: None,
            },
        )
        .unwrap();
        approve_decision(
            &ports.writer,
            &ports.query,
            &clock,
            &AllowAllPolicy,
            &human_p,
            dec.decision_id,
            Privacy::LocalOnly,
        )
        .unwrap();
    }

    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: agent_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project_b),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(packet.denied || packet.decisions.is_empty());
    assert!(packet.decisions.is_empty());
    assert!(packet.conclusions.is_empty());
    assert!(packet.denied || packet.warnings.iter().any(|w| w.kind == "denied"));
    assert!(
        !packet
            .decisions
            .iter()
            .any(|d| d.statement.contains("must not leak"))
    );
}

#[test]
fn project_briefing__low_confidence_scope__no_high_authority_injection() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());

    // Seed high-authority content under AllowAll.
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeDecisionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            title: "Hidden".into(),
            statement: "should not inject on low confidence".into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![EvidenceId::new()]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        &human_p,
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Low confidence: no explicit project id, cwd only (nil sentinel / non-authoritative).
    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: None,
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(
        !packet.scope.authoritative
            || packet.scope.confidence == "Low"
            || packet.scope.confidence == "Ambiguous"
    );
    assert!(
        packet.decisions.is_empty(),
        "low-confidence must not inject high-authority decisions"
    );
    assert!(packet.conclusions.is_empty());
    assert!(
        packet
            .warnings
            .iter()
            .any(|w| w.kind == "low_confidence" || w.kind == "other")
    );
}

#[test]
fn project_briefing__budget_truncation__marks_dropped_sections() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();

    for i in 0..5 {
        let d = propose_decision(
            &ports.writer,
            &ports.query,
            &clock,
            &policy,
            ProposeDecisionRequest {
                principal: human_p.clone(),
                scope: scope.clone(),
                title: format!("D{i}"),
                statement: format!("decision statement number {i} with extra words for budget"),
                conclusion_ids: None,
                evidence_ids: Some(vec![EvidenceId::new()]),
                privacy: Privacy::LocalOnly,
                valid_from: None,
                valid_until: None,
                decision_id: None,
            },
        )
        .unwrap();
        approve_decision(
            &ports.writer,
            &ports.query,
            &clock,
            &policy,
            &human_p,
            d.decision_id,
            Privacy::LocalOnly,
        )
        .unwrap();
    }

    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig {
                max_words: 1500,
                max_decisions: 2,
                max_conclusions: 2,
                max_constraints: 4,
                max_warnings: 8,
            },
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(packet.decisions.len() <= 2);
    assert!(
        packet
            .budget
            .truncated_sections
            .iter()
            .any(|s| s == "decisions")
            || packet.budget.more_available
    );
}

#[test]
fn project_briefing__never_nests_personal() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, ScopeRef::Repository(project));
    // Also seed personal conclusion — must not appear in project packet.
    let personal = ScopeRef::Personal(UserId::new());
    grant_reads(&ports, human_p.id, personal.clone());
    propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &ports.production_policy(),
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal,
            statement: "personal preference secret".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();

    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &ports.production_policy(),
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    let json = serde_json::to_value(&packet).unwrap();
    assert!(json.get("personal").is_none());
    assert!(json.get("preferences").is_none());
    assert!(
        !packet
            .conclusions
            .iter()
            .any(|c| c.statement.contains("personal preference"))
    );
}

#[test]
fn project_briefing__cache_hit_then_miss_on_version_vector_advance() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();
    let identity = ports.identity_store();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "Cached authority claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let req = ProjectBriefingRequest {
        principal: human_p.clone(),
        resolve: ScopeResolveInput {
            cwd: PathBuf::from("."),
            explicit_project_id: Some(project),
            force_personal: false,
            personal_user_id: None,
            git_metadata: None,
        },
        budget: BudgetConfig::default(),
        privacy: Privacy::LocalOnly,
        dry_run: false, // enable cache write/read
        briefing_id: None,
        ledgerful: None,
    };

    let first = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req.clone(),
    )
    .unwrap();
    assert!(!first.conclusions.is_empty());
    let first_id = first.briefing_id.clone();

    // Second call with same version vector should hit cache (same packet_json).
    let second = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req.clone(),
    )
    .unwrap();
    assert_eq!(
        second.briefing_id, first_id,
        "cache hit must return same cached briefing_id"
    );
    assert_eq!(second.conclusions.len(), first.conclusions.len());

    // Advance version vector by adding another conclusion.
    let conc2 = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "New conclusion advances version vector".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        conc2.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let third = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req,
    )
    .unwrap();
    assert_ne!(
        third.briefing_id, first_id,
        "version vector advance must miss cache and rebuild"
    );
    assert!(
        third.conclusions.len() >= first.conclusions.len(),
        "rebuild after advance should include new conclusions"
    );
}

/// T152-R2-01: grant revoke must not serve a prior authorized cache hit.
#[test]
fn project_briefing__grant_then_revoke__cache_miss_or_empty_denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();

    // Issue grants and retain ids so we can revoke them.
    let mut grant_ids = Vec::new();
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
        GrantCapability::ApproveConclusion,
    ] {
        let gid = issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
        grant_ids.push(gid);
    }

    let policy = ports.production_policy();
    let identity = ports.identity_store();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "Authority that must not survive revoke via cache".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let req = ProjectBriefingRequest {
        principal: human_p.clone(),
        resolve: ScopeResolveInput {
            cwd: PathBuf::from("."),
            explicit_project_id: Some(project),
            force_personal: false,
            personal_user_id: None,
            git_metadata: None,
        },
        budget: BudgetConfig::default(),
        privacy: Privacy::LocalOnly,
        dry_run: false,
        briefing_id: None,
        ledgerful: None,
    };

    let authorized = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req.clone(),
    )
    .unwrap();
    assert!(
        !authorized.denied && !authorized.conclusions.is_empty(),
        "precondition: authorized build must put conclusions into cache"
    );
    let authorized_id = authorized.briefing_id.clone();
    let authorized_statement = authorized.conclusions[0].statement.clone();

    // Revoke all grants for this principal at scope.
    for gid in grant_ids {
        revoke_grant(&ports.writer, &clock, gid, "t152-r2-01 revoke").unwrap();
    }

    let after_revoke = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req,
    )
    .unwrap();

    assert_ne!(
        after_revoke.briefing_id, authorized_id,
        "grant revoke must miss cache (new briefing_id), not serve prior authorized packet"
    );
    assert!(
        after_revoke.denied || after_revoke.conclusions.is_empty(),
        "after revoke must not expose prior conclusions as authorized; denied={} conclusions={}",
        after_revoke.denied,
        after_revoke.conclusions.len()
    );
    assert!(
        !after_revoke
            .conclusions
            .iter()
            .any(|c| c.statement == authorized_statement),
        "prior authorized conclusion statement must not appear after grant revoke"
    );
    assert!(
        after_revoke.decisions.is_empty(),
        "after revoke decisions section must be empty"
    );
}

/// T152-P1-01: High-confidence cache must not be served when resolution is non-authoritative
/// (Ambiguous) for the same scope identity key.
#[test]
fn project_briefing__high_cache_then_non_authoritative__does_not_serve_authority() {
    use ai_brains_control_plane::{
        Result as CpResult, ScopeConfidence, ScopeIdentityStore, is_authoritative, resolve_scope,
    };
    use ai_brains_git::{GitMetadata, hash_remote_url};
    use std::collections::HashMap;

    /// Identity store that can force Ambiguous for a fixed pair of projects.
    #[derive(Default)]
    struct AmbiguousIdentity {
        by_remote: HashMap<String, ProjectId>,
        by_path: HashMap<String, ProjectId>,
    }
    impl ScopeIdentityStore for AmbiguousIdentity {
        fn find_by_remote_hash(&self, hash: &str) -> CpResult<Option<ProjectId>> {
            Ok(self.by_remote.get(hash).copied())
        }
        fn find_by_path_alias(&self, normalized_path: &str) -> CpResult<Option<ProjectId>> {
            Ok(self.by_path.get(normalized_path).copied())
        }
        fn find_by_common_dir_alias(&self, path: &str) -> CpResult<Option<ProjectId>> {
            self.find_by_path_alias(path)
        }
        fn find_by_ledgerful_id(&self, _id: &str) -> CpResult<Option<ProjectId>> {
            Ok(None)
        }
    }

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    // Stable ordering: primary Ambiguous pick is the lower UUID string.
    let project_a = ProjectId::from_uuid(uuid::Uuid::from_u128(10));
    let project_b = ProjectId::from_uuid(uuid::Uuid::from_u128(11));
    let scope_a = ScopeRef::Repository(project_a);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope_a.clone());
    let policy = ports.production_policy();

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope_a.clone(),
            statement: "High-authority claim that must not leak via cache on Ambiguous".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // 1) High-confidence build caches the high-authority packet for project A.
    let identity = ports.identity_store();
    let high = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p.clone(),
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project_a),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: false,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();
    assert!(high.scope.authoritative);
    assert!(!high.conclusions.is_empty());
    let high_statement = high.conclusions[0].statement.clone();

    // 2) Non-authoritative Ambiguous resolve that still picks project A as primary
    // (same cache key scope segment).
    let hash_a = hash_remote_url("https://example.com/a.git").unwrap();
    let common_for_b = r"C:\other\b\.git";
    let mut amb_identity = AmbiguousIdentity::default();
    amb_identity.by_remote.insert(hash_a.clone(), project_a);
    amb_identity.by_path.insert(
        ai_brains_path::normalize_for_location_compare(common_for_b),
        project_b,
    );
    let meta = GitMetadata {
        root: Some(PathBuf::from(r"C:\work\a")),
        remote_url_hash: Some(hash_a),
        common_dir: Some(PathBuf::from(common_for_b)),
        ..GitMetadata::default()
    };
    let amb_input = ScopeResolveInput {
        cwd: PathBuf::from(r"C:\work\a"),
        explicit_project_id: None,
        force_personal: false,
        personal_user_id: None,
        git_metadata: Some(meta),
    };
    let resolved = resolve_scope(&amb_input, &amb_identity).unwrap();
    assert_eq!(resolved.confidence, ScopeConfidence::Ambiguous);
    assert!(!is_authoritative(&resolved));
    assert_eq!(resolved.scope, ScopeRef::Repository(project_a));

    let after = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &amb_identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: amb_input,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: false,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(
        !after.scope.authoritative,
        "Ambiguous resolution must not be authoritative"
    );
    assert!(
        after.decisions.is_empty() && after.conclusions.is_empty(),
        "non-authoritative path must withhold high-authority claims"
    );
    assert!(
        !after
            .conclusions
            .iter()
            .any(|c| c.statement == high_statement),
        "must not serve cached high-authority statement under non-authoritative resolve"
    );
}

/// T152-P1-02: future and expired claims excluded from project current authority.
#[test]
fn project_briefing__future_and_expired_claims__excluded() {
    use time::Duration;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();
    let now = clock.now().unwrap();

    let future = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "future project claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now + Duration::days(7)),
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        future.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let expired = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "expired project claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now - Duration::days(30)),
            valid_until: Some(now - Duration::hours(1)),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        expired.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let current = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "current project claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(now - Duration::hours(1)),
            valid_until: Some(now + Duration::days(7)),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        current.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(
        !packet
            .conclusions
            .iter()
            .any(|c| c.id == future.conclusion_id.to_string()),
        "future conclusion must not be current authority"
    );
    assert!(
        !packet
            .conclusions
            .iter()
            .any(|c| c.id == expired.conclusion_id.to_string()),
        "expired conclusion must not be current authority"
    );
    assert!(
        packet
            .conclusions
            .iter()
            .any(|c| c.id == current.conclusion_id.to_string()),
        "in-window conclusion must appear"
    );
    assert!(
        packet
            .warnings
            .iter()
            .any(|w| w.kind == "out_of_valid_time"),
        "out-of-window claims should surface as warnings"
    );
}

/// T152-P2-02: word budget must bring used_words under max_words even with long warnings.
#[test]
fn project_briefing__word_budget_over_limit__truncates_warnings() {
    use ai_brains_contracts::briefings::{
        BriefingScopeDto, BriefingWarningDto, BudgetReportDto, FreshnessSummaryDto,
        ProjectBriefingPacket,
    };
    use ai_brains_control_plane::apply_budget;

    let long = "word ".repeat(200);
    let mut packet = ProjectBriefingPacket {
        api_version: "1".into(),
        briefing_id: uuid::Uuid::nil().to_string(),
        kind: "Project".into(),
        scope: BriefingScopeDto {
            scope_key: "Repository:00000000-0000-0000-0000-000000000001".into(),
            confidence: "High".into(),
            warnings: vec![],
            alternatives: vec![],
            authoritative: true,
        },
        handoff: None,
        decisions: vec![],
        conclusions: vec![],
        constraints: vec![],
        warnings: vec![
            BriefingWarningDto {
                kind: "other".into(),
                message: long.clone(),
                subject_id: None,
                subject_kind: None,
            },
            BriefingWarningDto {
                kind: "other".into(),
                message: long,
                subject_id: None,
                subject_kind: None,
            },
        ],
        freshness: FreshnessSummaryDto {
            total_sources: 0,
            fresh_count: 0,
            stale_count: 0,
            unavailable_count: 0,
            worst_state: "Unknown".into(),
        },
        ledgerful: None,
        evidence_handles: vec![],
        budget: BudgetReportDto {
            max_words: 50,
            used_words: 0,
            truncated_sections: vec![],
            more_available: false,
        },
        generated_at: None,
        denied: false,
        denial_reason: None,
        denial_hint: None,
    };
    apply_budget(
        &mut packet,
        BudgetConfig {
            max_words: 50,
            max_decisions: 32,
            max_conclusions: 32,
            max_constraints: 16,
            max_warnings: 24,
        },
    );
    assert!(
        packet.budget.used_words <= packet.budget.max_words,
        "used_words={} must be <= max_words={}",
        packet.budget.used_words,
        packet.budget.max_words
    );
    assert!(packet.budget.more_available);
    assert!(
        packet
            .budget
            .truncated_sections
            .iter()
            .any(|s| s == "warnings")
    );
}

/// T152-FRESH-P1-01: cache hit re-filters valid-time after the clock advances.
#[test]
fn project_briefing__cache_hit__refilters_expired_valid_time() {
    use ai_brains_control_plane::{Clock, ControlPlaneError, Result as CpResult};
    use std::sync::Mutex;
    use time::Duration;

    struct MutableClock {
        now: Mutex<time::OffsetDateTime>,
    }
    impl Clock for MutableClock {
        fn now(&self) -> CpResult<time::OffsetDateTime> {
            self.now
                .lock()
                .map(|g| *g)
                .map_err(|e| ControlPlaneError::Clock(e.to_string()))
        }
    }

    let (_t, ports) = open_ports();
    let t0 = SystemClock.now().unwrap();
    let clock = MutableClock {
        now: Mutex::new(t0),
    };
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());
    let policy = ports.production_policy();
    let identity = ports.identity_store();

    // Claim valid until t0 + 2 hours.
    let expiring = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "short-lived claim for cache valid-time".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: Some(t0 - Duration::hours(1)),
            valid_until: Some(t0 + Duration::hours(2)),
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        expiring.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let req = ProjectBriefingRequest {
        principal: human_p.clone(),
        resolve: ScopeResolveInput {
            cwd: PathBuf::from("."),
            explicit_project_id: Some(project),
            force_personal: false,
            personal_user_id: None,
            git_metadata: None,
        },
        budget: BudgetConfig::default(),
        privacy: Privacy::LocalOnly,
        dry_run: false,
        briefing_id: None,
        ledgerful: None,
    };

    let first = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req.clone(),
    )
    .unwrap();
    assert!(
        first
            .conclusions
            .iter()
            .any(|c| c.id == expiring.conclusion_id.to_string()),
        "precondition: claim must be current when cached"
    );

    // Advance past valid_until without changing epistemic version vector content
    // (state still Active; only clock moves).
    *clock.now.lock().unwrap() = t0 + Duration::hours(3);

    let second = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        req,
    )
    .unwrap();

    assert!(
        !second
            .conclusions
            .iter()
            .any(|c| c.id == expiring.conclusion_id.to_string()),
        "cache hit must drop expired claim after time advance: {:?}",
        second.conclusions
    );
    assert!(
        second.warnings.iter().any(|w| w.kind == "out_of_valid_time"
            && w.subject_id.as_deref() == Some(&expiring.conclusion_id.to_string())),
        "expired cached claim should surface as out_of_valid_time warning"
    );
}

/// T152-FRESH-P1-03: decisions-only principal must not see conclusion conflict text.
#[test]
fn project_briefing__decisions_only__no_conclusion_conflict_warning() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let key = scope_identity_key(&scope);
    let human_p = human();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();

    // Human seeds data + opens a conclusion-vs-conclusion conflict.
    grant_reads(&ports, human_p.id, scope.clone());
    // Agent: ReadDecisions only (no ReadConclusions).
    issue_grant(
        &ports.writer,
        &clock,
        agent_p.id,
        scope.clone(),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.production_policy();
    let c1 = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "secret conclusion A text must not leak".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        c1.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    let c2 = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            statement: "secret conclusion B text must not leak".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        c2.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let conflict_explanation =
        "overlapping secret conclusion A text must not leak vs B".to_string();
    open_claim_conflict(
        &ports.writer,
        &policy,
        &human_p,
        OpenClaimConflictRequest {
            claim_a_kind: "Conclusion".into(),
            claim_a_id: c1.conclusion_id.to_string(),
            claim_b_kind: "Conclusion".into(),
            claim_b_id: c2.conclusion_id.to_string(),
            scope: key,
            explanation: conflict_explanation.clone(),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            conflict_id: None,
        },
    )
    .unwrap();

    // Also seed an approved decision so the agent has something to read.
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: human_p.clone(),
            scope: scope.clone(),
            title: "Ship".into(),
            statement: "Ship the briefing".into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![EvidenceId::new()]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .unwrap();
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: agent_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(!packet.decisions.is_empty());
    assert!(packet.conclusions.is_empty());
    assert!(
        !packet.warnings.iter().any(|w| w.kind == "open_conflict"),
        "decisions-only principal must not receive conclusion-only conflict warnings: {:?}",
        packet.warnings
    );
    assert!(
        !packet
            .warnings
            .iter()
            .any(|w| w.message.contains("secret conclusion")),
        "conflict explanation must not leak conclusion statement text: {:?}",
        packet.warnings
    );
}

/// T152-FRESH3-P1-01: NeverInject/Sealed claim forces BriefingGenerated envelope privacy.
#[test]
fn project_briefing__never_inject_claim__envelope_privacy_at_least_as_strict() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let scope = ScopeRef::Repository(project);
    let human_p = human();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    grant_reads(&ports, human_p.id, scope.clone());
    grant_reads(&ports, agent_p.id, scope.clone());
    let policy = ports.production_policy();
    let identity = ports.identity_store();

    let sealed = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope.clone(),
            statement: "Never inject this sealed authority claim".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::NeverInject,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &agent_p,
        sealed.conclusion_id,
        Privacy::NeverInject,
    )
    .unwrap();

    // Stale Sealed claim whose statement appears only in warnings must also raise privacy.
    let stale = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: agent_p.clone(),
            scope: scope.clone(),
            statement: "Stale sealed warning text must tighten envelope".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::Sealed,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &agent_p,
        stale.conclusion_id,
        Privacy::Sealed,
    )
    .unwrap();
    let mark =
        try_mark_stale_payload(stale.conclusion_id, None, Some("test stale".into())).unwrap();
    let stale_env = EventBuilder::new(
        AggregateType::Conclusion,
        stale.conclusion_id.as_uuid(),
        Actor::System,
        Privacy::Sealed,
    )
    .build(Payload::ConclusionMarkedStale(mark))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[stale_env]).unwrap();

    let packet = build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            // Weaker request privacy must not win over included claims.
            privacy: Privacy::LocalOnly,
            dry_run: false,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(
        packet
            .conclusions
            .iter()
            .any(|c| c.id == sealed.conclusion_id.to_string()),
        "NeverInject current claim must be in conclusions"
    );
    assert!(
        packet.warnings.iter().any(|w| w.kind == "stale"
            && w.subject_id.as_deref() == Some(&stale.conclusion_id.to_string())),
        "Sealed stale claim must appear in warnings"
    );

    let events = EventStore::read_all_events(ports.writer.store()).unwrap();
    let generated: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.payload, Payload::BriefingGenerated(_)))
        .collect();
    assert_eq!(generated.len(), 1, "exactly one BriefingGenerated event");
    let env = generated[0];
    assert!(
        env.privacy >= Privacy::NeverInject,
        "envelope privacy must be at least NeverInject from current claim; got {:?}",
        env.privacy
    );
    assert_eq!(
        env.privacy,
        Privacy::Sealed,
        "Sealed warning claim must force strictest envelope privacy; got {:?}",
        env.privacy
    );
}

// ---------------------------------------------------------------------------
// T202 AC6 — denied paths seed kind=denied (no double warning)
// ---------------------------------------------------------------------------

#[test]
fn project_briefing__personal_scope_refuse__warnings_kind_denied_exactly_once() {
    // Personal-scope refuse (~181–188) returns bare empty_denied; helper must seed kind.
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    // Grant on Personal so policy is not the deny reason — force_personal path refuses Project packet.
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        ScopeRef::Personal(user),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: human_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: None,
                force_personal: true,
                personal_user_id: Some(user),
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(packet.denied, "Personal scope must deny Project packet");
    assert!(
        packet
            .denial_reason
            .as_deref()
            .is_some_and(|r| r.contains("Personal") || r.contains("personal")),
        "denial_reason should mention Personal; got {:?}",
        packet.denial_reason
    );
    let denied_count = packet
        .warnings
        .iter()
        .filter(|w| w.kind == "denied")
        .count();
    assert_eq!(
        denied_count, 1,
        "exactly one kind=denied warning (helper seed, no double); got {:?}",
        packet.warnings
    );
}

#[test]
fn project_briefing__grant_deny__warnings_kind_denied_exactly_once() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    // No read grants → full grant deny path via empty_denied.

    let policy = ports.production_policy();
    let identity = ports.identity_store();
    let packet = build_project_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: agent_p,
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: Some(project),
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
    .unwrap();

    assert!(packet.denied);
    let denied_count = packet
        .warnings
        .iter()
        .filter(|w| w.kind == "denied")
        .count();
    assert_eq!(
        denied_count, 1,
        "grant-deny must not double-push denied after helper seed; got {:?}",
        packet.warnings
    );
}

// ---------------------------------------------------------------------------
// T202 AC7 — markdown Denied one-liner with non-empty reason
// ---------------------------------------------------------------------------

#[test]
fn project_briefing__denied_markdown__contains_denied_oneliner_and_reason() {
    let reason = "ReadDecisions/ReadConclusions denied for principal at scope";
    let packet = ai_brains_contracts::briefings::ProjectBriefingPacket::empty_denied(
        "briefing-ac7".into(),
        ai_brains_contracts::briefings::BriefingScopeDto {
            scope_key: "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
            confidence: "High".into(),
            warnings: vec![],
            alternatives: vec![],
            authoritative: true,
        },
        reason,
    );
    let md = render_project_markdown(&packet);
    assert!(
        md.contains("**Denied:**"),
        "markdown must include Denied one-liner; got:\n{md}"
    );
    assert!(
        md.contains(reason),
        "markdown must include non-empty denial reason; got:\n{md}"
    );
    assert!(
        !packet.denial_reason.as_deref().unwrap_or("").is_empty(),
        "denial_reason must be non-empty"
    );
}
