//! Scenario 4 — conflicting_scoped_claims.

use std::collections::BTreeMap;

use ai_brains_core::ids::ConflictId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use serde_json::Value;

use super::SeedOutcome;
use super::common::{
    agent, grant_read_write, human, register, resolve_for_project, seed_active_conclusion,
    stable_project, stable_uuid,
};
use crate::adapters::StorePorts;
use crate::conflicts::{OpenClaimConflictRequest, open_claim_conflict};
use crate::errors::Result;
use crate::sources::scope_identity_key;

pub fn seed(ports: &StorePorts, _params: &BTreeMap<String, Value>) -> Result<SeedOutcome> {
    let project_id = stable_project("conflict");
    let scope = ScopeRef::Repository(project_id);
    let key = scope_identity_key(&scope);
    let human_p = human("conflict-human");
    let agent_p = agent("conflict-agent");
    register(ports, &human_p)?;
    register(ports, &agent_p)?;
    grant_read_write(ports, human_p.id, scope.clone())?;
    grant_read_write(ports, agent_p.id, scope.clone())?;

    let a = seed_active_conclusion(
        ports,
        &agent_p,
        scope.clone(),
        "deploy on friday",
        "conflict:claim-a",
    )?;
    let b = seed_active_conclusion(
        ports,
        &agent_p,
        scope,
        "do not deploy friday",
        "conflict:claim-b",
    )?;

    // Open conflict so both are not silent current authority without warning.
    open_claim_conflict(
        &ports.writer,
        &ports.production_policy(),
        &human_p,
        OpenClaimConflictRequest {
            claim_a_kind: "Conclusion".into(),
            claim_a_id: a.clone(),
            claim_b_kind: "Conclusion".into(),
            claim_b_id: b.clone(),
            scope: key,
            explanation: "incompatible deploy guidance".into(),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            conflict_id: Some(ConflictId::from_uuid(stable_uuid("conflict:id"))),
        },
    )?;

    Ok(SeedOutcome {
        principal: agent_p,
        project_id,
        resolve: resolve_for_project(project_id),
        conflict_claim_ids: Some((a.clone(), b.clone())),
        claim_ids: vec![a, b],
        require_citations: true,
        // Floor: briefing may demote both to warnings — allow 0 current claims.
        // Runner uses scenario min_valid from fixture (set to 0 for this scen).
        ..SeedOutcome::default()
    })
}
