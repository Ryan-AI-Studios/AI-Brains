//! Scenario 6 — human_correction_supersedes.

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use serde_json::Value;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_active_conclusion,
    stable_conclusion_id, stable_evidence_id, stable_project,
};
use crate::adapters::{StorePorts, SystemClock};
use crate::conclusions::{activate_conclusion, correct_conclusion};
use crate::errors::Result;

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("human-correction");
    let scope = ScopeRef::Repository(project_id);
    let human_p = human("correction-human");
    let agent_p = agent("correction-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    let old = seed_active_conclusion(
        ports,
        &agent_p,
        scope.clone(),
        "agent inference: use algorithm A",
        "correction:old",
    )?;

    // Parse conclusion id for correct_conclusion.
    let old_id = old
        .parse()
        .map_err(|e| crate::errors::ControlPlaneError::InvalidPayload(format!("id: {e}")))?;

    let successor_id = stable_conclusion_id("correction:successor");
    let successor_ev = stable_evidence_id("correction:successor");

    let policy = ports.production_policy();
    let new_id = correct_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        &human_p,
        old_id,
        "human correction: use algorithm B with evidence".into(),
        vec![successor_ev],
        "human supersedes agent inference",
        Privacy::LocalOnly,
        Some(successor_id),
    )?;

    // Activate successor so it is current authority.
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &SystemClock,
        &policy,
        &agent_p,
        new_id,
        Privacy::LocalOnly,
    )?;

    let mut must_be_absent = BTreeSet::new();
    must_be_absent.insert(old.clone());

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        claim_ids: vec![new_id.to_string()],
        warning_subject_ids: vec![old],
        must_be_absent_claim_ids: must_be_absent,
        require_citations: true,
        ..SeedOutcome::default()
    })
}
