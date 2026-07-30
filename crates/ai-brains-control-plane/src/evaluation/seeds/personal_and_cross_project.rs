//! Scenario 5 — personal_and_cross_project_denied.

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_core::scope::ScopeRef;
use serde_json::Value;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_approved_decision,
    stable_project,
};
use crate::adapters::StorePorts;
use crate::errors::Result;

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let alpha = stable_project("alpha");
    let beta = stable_project("beta");
    let scope_a = ScopeRef::Repository(alpha);
    let scope_b = ScopeRef::Repository(beta);

    let human_p = human("iso-human");
    let alpha_agent = agent("iso-alpha");
    let beta_agent = agent("iso-beta");
    register(ports, &human_p)?;
    register(ports, &alpha_agent)?;
    register(ports, &beta_agent)?;

    // Alpha agent: only Alpha grants. No Personal. No Beta.
    grant_read_write(ports, alpha_agent.id, scope_a.clone())?;
    // Beta agent + human on Beta so we can seed Beta authority.
    grant_read_write(ports, beta_agent.id, scope_b.clone())?;
    grant_read_write(ports, human_p.id, scope_b.clone())?;
    grant_read_write(ports, human_p.id, scope_a.clone())?;

    let _alpha_dec = seed_approved_decision(
        ports,
        &human_p,
        scope_a,
        "Alpha only",
        "Alpha project decision",
    )?;
    let beta_dec = seed_approved_decision(
        ports,
        &human_p,
        scope_b,
        "Beta secret",
        "Beta project decision must not leak to Alpha",
    )?;

    let mut beta_claim_ids = BTreeSet::new();
    beta_claim_ids.insert(beta_dec.clone());
    let mut foreign = BTreeSet::new();
    foreign.insert(beta_dec.clone());

    Ok(SeedOutcome {
        // Brief as Alpha principal (no Beta / Personal grant).
        principal: alpha_agent,
        project_id: alpha,
        resolve: resolve_for_project(alpha),
        foreign_claim_ids: foreign,
        beta_claim_ids,
        claim_ids: vec![],
        require_citations: false,
        // Denied/empty path for foreign; Alpha may still have its own claims if
        // human seeded under shared grants — Alpha agent has Alpha grant so may see Alpha.
        // Fixture min_valid should allow Alpha claims if present; leakage must be 0.
        expect_denied: false,
        ..SeedOutcome::default()
    })
}
