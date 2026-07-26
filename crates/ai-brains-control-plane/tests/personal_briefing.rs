#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T152 Phase D — Personal Continuity Briefing isolation.

use ai_brains_contracts::briefings::AppliedGrantDto;
use ai_brains_control_plane::{
    BudgetConfig, Clock, PersonalBriefingRequest, ProposeConclusionRequest, StoreEventWriter,
    StorePorts, SystemClock, activate_conclusion, build_personal_briefing, confirm_conclusion,
    issue_grant, make_principal, propose_conclusion, register_principal, render_personal_markdown,
    scope_identity_key,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId, UserId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::DataKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;

fn open_ports() -> (tempfile::NamedTempFile, StorePorts) {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
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

fn human() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Human, PrincipalId::new(), "human")
}

fn agent() -> ai_brains_core::principal::Principal {
    make_principal(PrincipalKind::Agent, PrincipalId::new(), "agent")
}

#[test]
fn personal_briefing__with_personal_grant__returns_preferences() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    let grant_id = issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        personal.clone(),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();
    for cap in [
        GrantCapability::ProposeConclusion,
        GrantCapability::ApproveConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            personal.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }

    let policy = ports.production_policy();
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "Prefer PowerShell statement separators".into(),
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
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let scope_key = scope_identity_key(&personal);
    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        |_p| {
            Ok(vec![AppliedGrantDto {
                grant_id: grant_id.to_string(),
                scope_key: scope_key.clone(),
                capability: "ReadConclusions".into(),
                privacy: "LocalOnly".into(),
            }])
        },
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert_eq!(packet.kind, "Personal");
    assert!(!packet.denied);
    assert!(!packet.preferences.is_empty());
    assert!(
        packet
            .preferences
            .iter()
            .any(|p| p.statement.contains("PowerShell"))
    );
    assert!(!packet.grants_applied.is_empty());
    let md = render_personal_markdown(&packet);
    assert!(md.contains("Personal Continuity"));
}

#[test]
fn personal_briefing__without_grant__denied() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let agent_p = agent();
    register_principal(&ports.writer, &clock, &agent_p).unwrap();
    // Grant only Project, not Personal.
    issue_grant(
        &ports.writer,
        &clock,
        agent_p.id,
        ScopeRef::Repository(ProjectId::new()),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &ports.production_policy(),
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: agent_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(packet.denied);
    assert!(packet.preferences.is_empty());
    assert!(packet.warnings.iter().any(|w| w.kind == "denied"));
}

#[test]
fn personal_briefing__project_only_request_path__cannot_return_personal_without_grant() {
    // When only Project scope is granted, Personal briefing must be denied —
    // never silently nest or return Personal preferences.
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let project = ProjectId::new();
    let user = UserId::new();
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        ScopeRef::Repository(project),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        ScopeRef::Repository(project),
        GrantCapability::ProposeConclusion,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Seed a personal-looking conclusion under AllowAll on personal scope (data exists).
    use ai_brains_control_plane::AllowAllPolicy;
    propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: ScopeRef::Personal(user),
            statement: "secret personal preference".into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &ports.production_policy(),
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(packet.denied);
    assert!(
        !packet
            .preferences
            .iter()
            .any(|p| p.statement.contains("secret personal"))
    );
}

#[test]
fn personal_briefing__project_scoped_open_review__not_included() {
    use ai_brains_control_plane::{
        ProposeDecisionRequest, activate_conclusion, approve_decision, propose_decision,
    };
    use ai_brains_core::ids::ReviewItemId;
    use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::ReviewItemOpenedPayload;
    use ai_brains_events::{Actor, AggregateType, Payload};
    use ai_brains_store::event_store::EventStore;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let project = ProjectId::new();
    let personal = ScopeRef::Personal(user);
    let project_scope = ScopeRef::Repository(project);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();

    for (scope, caps) in [
        (
            personal.clone(),
            [
                GrantCapability::ReadConclusions,
                GrantCapability::ProposeConclusion,
                GrantCapability::ApproveConclusion,
            ]
            .as_slice(),
        ),
        (
            project_scope.clone(),
            [
                GrantCapability::ReadConclusions,
                GrantCapability::ReadDecisions,
                GrantCapability::ProposeConclusion,
                GrantCapability::ProposeDecision,
                GrantCapability::ApproveDecision,
            ]
            .as_slice(),
        ),
    ] {
        for cap in caps {
            issue_grant(
                &ports.writer,
                &clock,
                human_p.id,
                scope.clone(),
                *cap,
                Privacy::LocalOnly,
            )
            .unwrap();
        }
    }

    let policy = ports.production_policy();

    // Project decision + open review bound to it.
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: human_p.clone(),
            scope: project_scope.clone(),
            title: "Project decision under review".into(),
            statement: "Should not appear in personal packet".into(),
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

    let review_item_id = ReviewItemId::new();
    let env = EventBuilder::new(
        AggregateType::ReviewItem,
        review_item_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
        review_item_id,
        subject: "project decision needs human review".into(),
        opened_by: human_p.id,
        subject_kind: ReviewSubjectKind::Decision,
        subject_id: dec.decision_id.to_string(),
        criticality: ReviewCriticality::High,
        related_conclusion_id: None,
        related_decision_id: Some(dec.decision_id),
        related_source_id: None,
    }))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[env]).unwrap();

    // Personal preference + personal-related review.
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "Prefer concise responses".into(),
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
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let personal_review_id = ReviewItemId::new();
    let env2 = EventBuilder::new(
        AggregateType::ReviewItem,
        personal_review_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ReviewItemOpened(ReviewItemOpenedPayload {
        review_item_id: personal_review_id,
        subject: format!("personal item for {}", scope_identity_key(&personal)),
        opened_by: human_p.id,
        subject_kind: ReviewSubjectKind::Conclusion,
        subject_id: pref.conclusion_id.to_string(),
        criticality: ReviewCriticality::Low,
        related_conclusion_id: Some(pref.conclusion_id),
        related_decision_id: None,
        related_source_id: None,
    }))
    .unwrap();
    EventStore::append_events(ports.writer.store(), &[env2]).unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(
        !packet
            .open_review_items
            .iter()
            .any(|i| i.id == review_item_id.to_string()),
        "project-scoped open review must not appear in personal packet: {:?}",
        packet.open_review_items
    );
    assert!(
        packet
            .open_review_items
            .iter()
            .any(|i| i.id == personal_review_id.to_string()),
        "personal-related review must appear: {:?}",
        packet.open_review_items
    );
}

/// T152-P1-04: ReadDecisions alone must not populate conclusions preferences.
#[test]
fn personal_briefing__decisions_only_grant__preferences_empty() {
    use ai_brains_control_plane::AllowAllPolicy;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();

    // Seed a personal preference conclusion under AllowAll (data exists).
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &AllowAllPolicy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "secret preference must not appear without ReadConclusions".into(),
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
        &AllowAllPolicy,
        &human_p,
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    // Principal has only ReadDecisions on Personal — not ReadConclusions.
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        personal.clone(),
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &ports.production_policy(),
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(
        !packet.denied,
        "ReadDecisions still admits the personal packet surface"
    );
    assert!(
        packet.preferences.is_empty(),
        "decisions-only grant must not populate conclusions preferences: {:?}",
        packet.preferences
    );
    assert!(
        !packet
            .preferences
            .iter()
            .any(|p| p.statement.contains("secret preference")),
        "preference statement must not leak under decisions-only grant"
    );
}

/// T152-FRESH-P1-02: Active (unconfirmed) personal conclusions are not preferences.
#[test]
fn personal_briefing__active_only__not_in_preferences() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ProposeConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            personal.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    let policy = ports.production_policy();
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "Active but not confirmed preference".into(),
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
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(
        packet.preferences.is_empty(),
        "Active-only conclusions must not appear as personal preferences: {:?}",
        packet.preferences
    );
}

/// T152-FRESH-P1-02: expired Confirmed preferences omitted by valid-time filter.
#[test]
fn personal_briefing__expired_confirmed__omitted() {
    use time::Duration;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ApproveConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            personal.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    let policy = ports.production_policy();
    let now = clock.now().unwrap();

    let expired = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "expired personal preference".into(),
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
    confirm_conclusion(
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
            scope: personal.clone(),
            statement: "current personal preference".into(),
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
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        current.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(
        !packet
            .preferences
            .iter()
            .any(|p| p.id == expired.conclusion_id.to_string()),
        "expired Confirmed preference must be omitted"
    );
    assert!(
        packet
            .preferences
            .iter()
            .any(|p| p.id == current.conclusion_id.to_string()),
        "in-window Confirmed preference must appear"
    );
}

/// T152-FRESH-P2: personal word budget counts open_review_items and stays ≤ max_words.
#[test]
fn personal_briefing__word_budget__includes_open_review_items() {
    use ai_brains_contracts::briefings::{
        BudgetReportDto, ContinuitySummaryDto, PersonalContinuityBriefingPacket,
        PersonalPreferenceDto, PersonalReviewItemDto,
    };
    use ai_brains_control_plane::apply_personal_budget;

    let long_subject = "review item word ".repeat(80);
    let mut packet = PersonalContinuityBriefingPacket {
        api_version: "1".into(),
        briefing_id: uuid::Uuid::nil().to_string(),
        kind: "Personal".into(),
        scope_key: "Personal:00000000-0000-0000-0000-000000000001".into(),
        preferences: vec![PersonalPreferenceDto {
            id: "pref-1".into(),
            statement: "short pref".into(),
            evidence_handles: vec![],
        }],
        continuity: ContinuitySummaryDto {
            summary: "continuity thread summary words here".into(),
            thread_handles: vec![],
        },
        open_review_items: vec![PersonalReviewItemDto {
            id: "rev-1".into(),
            subject: long_subject,
            criticality: "High".into(),
            status: "Open".into(),
        }],
        grants_applied: vec![],
        warnings: vec![],
        budget: BudgetReportDto {
            max_words: 20,
            used_words: 0,
            truncated_sections: vec![],
            more_available: false,
        },
        generated_at: None,
        denied: false,
        denial_reason: None,
    };

    apply_personal_budget(
        &mut packet,
        BudgetConfig {
            max_words: 20,
            ..BudgetConfig::default()
        },
    );

    assert!(
        packet.budget.used_words <= 20,
        "used_words must not exceed max_words: {}",
        packet.budget.used_words
    );
    assert!(
        packet.budget.more_available
            || packet.open_review_items.is_empty()
            || packet.preferences.is_empty()
            || packet.continuity.summary.is_empty(),
        "over-budget packet must truncate or flag more_available"
    );
}

/// T152-FRESH3-P2-01: personal packet lists only Personal-scope grants, never project grants.
#[test]
fn personal_briefing__project_grant_active__grants_applied_only_personal() {
    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let project = ScopeRef::Repository(ProjectId::new());
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();

    let personal_grant = issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        personal.clone(),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();
    for cap in [
        GrantCapability::ProposeConclusion,
        GrantCapability::ApproveConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            personal.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    let project_grant = issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        project.clone(),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();
    // Extra project decision grant must not bleed into personal grants_applied.
    issue_grant(
        &ports.writer,
        &clock,
        human_p.id,
        project,
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let personal_key = scope_identity_key(&personal);
    let grant_store = ports.grant_store();
    let policy = ports.production_policy();

    // Seed a Confirmed personal preference so packet is non-empty.
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "Prefer filtered grants_applied".into(),
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
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        pref.conclusion_id,
        Privacy::LocalOnly,
    )
    .unwrap();

    let packet = build_personal_briefing(
        None::<&StoreEventWriter>,
        &ports.query,
        &clock,
        &policy,
        |p| {
            grant_store.list_applied_grants(
                p.id,
                &personal_key,
                Some(&["ReadConclusions", "ReadDecisions"]),
            )
        },
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(
        !packet.grants_applied.is_empty(),
        "personal read grant must appear in grants_applied"
    );
    assert!(
        packet
            .grants_applied
            .iter()
            .all(|g| g.scope_key == personal_key),
        "grants_applied must only list Personal scope; got {:?}",
        packet.grants_applied
    );
    assert!(
        packet
            .grants_applied
            .iter()
            .any(|g| g.grant_id == personal_grant.to_string()),
        "personal grant id must be listed"
    );
    assert!(
        !packet
            .grants_applied
            .iter()
            .any(|g| g.grant_id == project_grant.to_string()),
        "project grant must not appear in personal packet"
    );
    assert!(
        packet
            .grants_applied
            .iter()
            .all(|g| g.capability == "ReadConclusions" || g.capability == "ReadDecisions"),
        "only relevant read capabilities: {:?}",
        packet.grants_applied
    );
}

/// T152-FRESH3-P1-01: personal BriefingGenerated inherits NeverInject claim privacy + evidence ids.
#[test]
fn personal_briefing__never_inject_preference__envelope_privacy_and_evidence_ids() {
    use ai_brains_events::Payload;
    use ai_brains_store::event_store::EventStore;

    let (_t, ports) = open_ports();
    let clock = SystemClock;
    let user = UserId::new();
    let personal = ScopeRef::Personal(user);
    let human_p = human();
    register_principal(&ports.writer, &clock, &human_p).unwrap();
    for cap in [
        GrantCapability::ReadConclusions,
        GrantCapability::ProposeConclusion,
        GrantCapability::ApproveConclusion,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human_p.id,
            personal.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .unwrap();
    }
    let policy = ports.production_policy();
    let evidence_id = EvidenceId::new();
    let pref = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human_p.clone(),
            scope: personal.clone(),
            statement: "Never inject personal preference".into(),
            evidence_ids: vec![evidence_id],
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
        &human_p,
        pref.conclusion_id,
        Privacy::NeverInject,
    )
    .unwrap();
    confirm_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human_p,
        pref.conclusion_id,
        Privacy::NeverInject,
    )
    .unwrap();

    let personal_key = scope_identity_key(&personal);
    let packet = build_personal_briefing(
        Some(&ports.writer),
        &ports.query,
        &clock,
        &policy,
        |p| {
            ports
                .grant_store()
                .list_applied_grants(p.id, &personal_key, Some(&["ReadConclusions"]))
        },
        PersonalBriefingRequest {
            principal: human_p,
            user_id: user,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: false,
            briefing_id: None,
        },
    )
    .unwrap();

    assert!(!packet.denied);
    assert!(!packet.preferences.is_empty());

    let events = EventStore::read_all_events(ports.writer.store()).unwrap();
    let generated: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.payload, Payload::BriefingGenerated(_)))
        .collect();
    assert_eq!(generated.len(), 1);
    assert!(
        generated[0].privacy >= Privacy::NeverInject,
        "envelope privacy must inherit NeverInject preference; got {:?}",
        generated[0].privacy
    );
    match &generated[0].payload {
        Payload::BriefingGenerated(p) => {
            assert!(
                p.evidence_ids
                    .iter()
                    .any(|e| e.to_string() == evidence_id.to_string()),
                "BriefingGenerated must record preference evidence handles; got {:?}",
                p.evidence_ids
            );
        }
        _ => unreachable!(),
    }
}
