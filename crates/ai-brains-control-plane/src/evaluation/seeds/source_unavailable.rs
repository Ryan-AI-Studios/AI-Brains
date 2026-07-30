//! Scenario 7 — source_unavailable.

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_core::ids::{ConclusionId, EvidenceId, PrincipalId, SourceId, SourceVersionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::ReviewCriticality;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{
    ConclusionProposedPayload, EvidenceRecordedPayload, SourceRegisteredPayload,
    SourceVersionRecordedPayload,
};
use ai_brains_events::{Actor, AggregateType, Payload};
use serde_json::Value;
use time::OffsetDateTime;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_approved_decision,
    stable_project, stable_uuid,
};
use crate::adapters::{StorePorts, SystemClock};
use crate::conclusions::activate_conclusion;
use crate::errors::Result;
use crate::invalidation::{SourceUnavailableRequest, mark_source_unavailable};
use crate::ports::EventWriter;
use crate::{normalize_path_locator, scope_identity_key};

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("source-unavail");
    let scope = ScopeRef::Repository(project_id);
    let scope_key = scope_identity_key(&scope);
    let human_p = human("unavail-human");
    let agent_p = agent("unavail-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Continue",
        "Unaffected decision remains current",
        "unavail:unaffected-decision",
    )?;

    let source_id = SourceId::from_uuid(stable_uuid("unavail:source"));
    let version_id = SourceVersionId::from_uuid(stable_uuid("unavail:v1"));
    let evidence_id = EvidenceId::from_uuid(stable_uuid("unavail:ev"));
    let conclusion_id = ConclusionId::from_uuid(stable_uuid("unavail:conc"));
    let principal = PrincipalId::from_uuid(stable_uuid("unavail:principal"));
    let ts = OffsetDateTime::from_unix_timestamp(1_700_000_000)
        .map_err(|e| crate::errors::ControlPlaneError::Clock(format!("fixture timestamp: {e}")))?;

    let events = vec![
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::SourceRegistered(SourceRegisteredPayload {
            source_id,
            kind: SourceKind::File,
            display_name: "remote.md".into(),
            locator: Some(normalize_path_locator("/eval/remote.md")),
            scope: Some(scope_key.clone()),
        }))
        .map_err(|e| crate::errors::ControlPlaneError::EventAppend(e.to_string()))?,
        EventBuilder::new(
            AggregateType::Source,
            source_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::SourceVersionRecorded(
            SourceVersionRecordedPayload {
                source_id,
                version_id,
                fingerprint: "v1:remote".into(),
                recorded_at: ts,
            },
        ))
        .map_err(|e| crate::errors::ControlPlaneError::EventAppend(e.to_string()))?,
        EventBuilder::new(
            AggregateType::Evidence,
            evidence_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
            evidence_id,
            source_id,
            source_version_id: Some(version_id),
            fingerprint: Some("v1:remote".into()),
            model_provenance: None,
            summary: "remote evidence".into(),
        }))
        .map_err(|e| crate::errors::ControlPlaneError::EventAppend(e.to_string()))?,
        EventBuilder::new(
            AggregateType::Conclusion,
            conclusion_id.as_uuid(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::ConclusionProposed(ConclusionProposedPayload {
            conclusion_id,
            statement: "depends on remote source".into(),
            evidence_ids: vec![evidence_id],
            proposer: principal,
            valid_from: None,
            valid_until: None,
            scope: scope_key,
            protected_category: None,
            unsupported: false,
            model_provenance: None,
        }))
        .map_err(|e| crate::errors::ControlPlaneError::EventAppend(e.to_string()))?,
    ];
    ports.writer.append_events(&events)?;

    let policy = ports.production_policy();
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        &agent_p,
        conclusion_id,
        Privacy::LocalOnly,
    )?;

    mark_source_unavailable(
        &ports.writer,
        &ports.query,
        &SystemClock,
        SourceUnavailableRequest {
            source_id,
            reason: "network offline".into(),
            opened_by: agent_p.id,
            privacy: Privacy::LocalOnly,
            criticality: ReviewCriticality::Critical,
        },
    )?;

    let dep_id = conclusion_id.to_string();
    let mut must_be_absent = BTreeSet::new();
    must_be_absent.insert(dep_id.clone());

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        claim_ids: vec![dec_id],
        warning_subject_ids: vec![dep_id],
        must_be_absent_claim_ids: must_be_absent,
        require_citations: true,
        ..SeedOutcome::default()
    })
}
