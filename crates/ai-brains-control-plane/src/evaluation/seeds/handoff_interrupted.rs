//! Scenario 2 — interrupted_task_resumption / handoff.

use std::collections::BTreeMap;

use ai_brains_core::scope::ScopeRef;
use serde_json::Value;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_active_conclusion,
    seed_approved_decision, stable_project,
};
use crate::adapters::StorePorts;
use crate::errors::Result;

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("handoff");
    let scope = ScopeRef::Repository(project_id);
    let human_p = human("handoff-human");
    let agent_p = agent("handoff-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    // Open-work signals as current authority (handoff section may be empty in v1).
    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Resume handoff",
        "Continue interrupted migration task from checkpoint",
        "handoff:decision",
    )?;
    let open_work = seed_active_conclusion(
        ports,
        &agent_p,
        scope,
        "Open work: finish evaluate harness wiring and scenario fixtures",
        "handoff:open-work",
    )?;

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        claim_ids: vec![dec_id, open_work],
        require_citations: true,
        ..SeedOutcome::default()
    })
}
