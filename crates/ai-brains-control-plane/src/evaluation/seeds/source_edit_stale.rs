//! Scenario 3 — source_edit_stales_conclusion.

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_core::ids::{ConclusionId, EvidenceId, PrincipalId, SourceId, SourceVersionId};
use ai_brains_core::privacy::Privacy;
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
use crate::adapters::{Sha256FingerprinterPort, StorePorts, SystemClock};
use crate::conclusions::activate_conclusion;
use crate::errors::Result;
use crate::ports::EventWriter;
use crate::sources::{ObserveSourceRequest, SourceContent, observe_source, scope_identity_key};
use crate::normalize_path_locator;

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("source-edit");
    let scope = ScopeRef::Repository(project_id);
    let scope_key = scope_identity_key(&scope);
    let human_p = human("source-edit-human");
    let agent_p = agent("source-edit-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    // Stable live decision so min_valid_claims can still pass after staling conclusion.
    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Keep shipping",
        "Unaffected decision remains current",
        "source-edit:unaffected-decision",
    )?;

    let source_id = SourceId::from_uuid(stable_uuid("source-edit:source"));
    let version_id = SourceVersionId::from_uuid(stable_uuid("source-edit:v1"));
    let evidence_id = EvidenceId::from_uuid(stable_uuid("source-edit:ev"));
    let conclusion_id = ConclusionId::from_uuid(stable_uuid("source-edit:conc"));
    let principal = PrincipalId::from_uuid(stable_uuid("source-edit:principal"));
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
            display_name: "eval-source.md".into(),
            locator: Some(normalize_path_locator("/eval/source-edit.md")),
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
                fingerprint: "v1:eval-source".into(),
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
            fingerprint: Some("v1:eval-source".into()),
            model_provenance: None,
            summary: "initial evidence".into(),
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
            statement: "depends on eval-source".into(),
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

    // Change source → invalidate dependent conclusion (production policy only; F-006).
    let fp = Sha256FingerprinterPort::new();
    let req = ObserveSourceRequest {
        principal: agent_p.id,
        scope: scope.clone(),
        kind: SourceKind::File,
        display_name: "eval-source.md".into(),
        locator: Some("/eval/source-edit.md".into()),
        content: SourceContent::Bytes(b"eval source content changed substantially\n".to_vec()),
        privacy: Privacy::LocalOnly,
        run_invalidation: true,
    };
    observe_source(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &fp,
        &policy,
        req,
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
