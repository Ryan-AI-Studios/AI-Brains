//! Scenario 1 — cold_start_cited_project.

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
    let project_id = stable_project("cold-start");
    let scope = ScopeRef::Repository(project_id);
    let human_p = human("cold-start-human");
    let agent_p = agent("cold-start-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    let dec_id = seed_approved_decision(
        ports,
        &human_p,
        scope.clone(),
        "Ship briefings",
        "Use deterministic project briefings for cold-start",
        "cold-start:decision",
    )?;
    let conc_id = seed_active_conclusion(
        ports,
        &agent_p,
        scope,
        "Authority order is policy-first with evidence handles",
        "cold-start:conclusion",
    )?;

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        claim_ids: vec![dec_id, conc_id],
        require_citations: true,
        expect_denied: false,
        ..SeedOutcome::default()
    })
}
