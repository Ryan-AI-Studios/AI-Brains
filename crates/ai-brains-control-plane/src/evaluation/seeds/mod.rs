//! Registered seed programs for evaluation scenarios 1–9 (T169 E24/E25).

mod common;
mod conflict_scoped;
mod erasure_ce_wipe;
mod handoff_interrupted;
mod human_correction;
mod path_alias_wsl;
mod personal_and_cross_project;
mod project_briefing_minimal;
mod source_edit_stale;
mod source_unavailable;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ai_brains_core::ids::{ProjectId, UserId};
use ai_brains_core::principal::Principal;
use serde_json::Value;

use crate::adapters::StorePorts;
use crate::errors::{ControlPlaneError, Result};
use crate::scope_resolver::ScopeResolveInput;

pub use common::open_hermetic_ports;

/// Outcome of a seed program — enough for actions + metrics.
#[derive(Debug, Clone)]
pub struct SeedOutcome {
    pub principal: Principal,
    pub project_id: ProjectId,
    pub resolve: ScopeResolveInput,
    pub foreign_claim_ids: BTreeSet<String>,
    pub beta_claim_ids: BTreeSet<String>,
    pub wiped_subject_id: Option<String>,
    pub must_be_absent_claim_ids: BTreeSet<String>,
    pub conflict_claim_ids: Option<(String, String)>,
    pub scope_keys: Option<(String, String)>,
    pub claim_ids: Vec<String>,
    pub warning_subject_ids: Vec<String>,
    pub content_key_id: Option<String>,
    pub require_citations: bool,
    /// When set, briefing is expected denied/empty (min_valid may be 0).
    pub expect_denied: bool,
    /// When true, runner must verify Personal briefing is denied for principal.
    pub require_personal_denial: bool,
    /// Personal user id for personal-denial path (scen 5).
    pub personal_user_id: Option<UserId>,
}

impl Default for SeedOutcome {
    fn default() -> Self {
        Self {
            principal: crate::conclusions::principal(
                ai_brains_core::principal::PrincipalKind::System,
                ai_brains_core::ids::PrincipalId::from_uuid(common::stable_uuid(
                    "seed-outcome:unset-principal",
                )),
                "unset",
            ),
            project_id: ProjectId::from_uuid(common::stable_uuid("seed-outcome:unset-project")),
            resolve: ScopeResolveInput {
                cwd: PathBuf::from("."),
                explicit_project_id: None,
                force_personal: false,
                personal_user_id: None,
                git_metadata: None,
            },
            foreign_claim_ids: BTreeSet::new(),
            beta_claim_ids: BTreeSet::new(),
            wiped_subject_id: None,
            must_be_absent_claim_ids: BTreeSet::new(),
            conflict_claim_ids: None,
            scope_keys: None,
            claim_ids: Vec::new(),
            warning_subject_ids: Vec::new(),
            content_key_id: None,
            require_citations: true,
            expect_denied: false,
            require_personal_denial: false,
            personal_user_id: None,
        }
    }
}

/// Whitelist of registered program names.
pub const SEED_PROGRAMS: &[&str] = crate::evaluation::schema::KNOWN_SEED_PROGRAMS;

pub fn is_known_seed_program(name: &str) -> bool {
    SEED_PROGRAMS.contains(&name)
}

/// Run a registered seed program against an already-open hermetic vault.
pub fn run_seed(
    ports: &StorePorts,
    program: &str,
    params: &BTreeMap<String, Value>,
) -> Result<SeedOutcome> {
    match program {
        "project_briefing_minimal" => project_briefing_minimal::seed(ports, params),
        "handoff_interrupted" => handoff_interrupted::seed(ports, params),
        "source_edit_stale" => source_edit_stale::seed(ports, params),
        "conflict_scoped" => conflict_scoped::seed(ports, params),
        "personal_and_cross_project" => personal_and_cross_project::seed(ports, params),
        "human_correction" => human_correction::seed(ports, params),
        "source_unavailable" => source_unavailable::seed(ports, params),
        "erasure_ce_wipe" => erasure_ce_wipe::seed(ports, params),
        "path_alias_wsl" => path_alias_wsl::seed(ports, params),
        other => Err(ControlPlaneError::InvalidPayload(format!(
            "unknown seed program '{other}'"
        ))),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn seed__unknown_program__err() {
        let (_tmp, ports) = open_hermetic_ports().expect("ports");
        let err = run_seed(&ports, "nope", &BTreeMap::new()).expect_err("unknown");
        assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
    }

    #[test]
    fn seed__hermetic__two_scenarios_isolated_vaults() {
        let (t1, p1) = open_hermetic_ports().expect("p1");
        let (t2, p2) = open_hermetic_ports().expect("p2");
        // Distinct tempfile vaults (E1/E25) — seed programs may use stable ids.
        assert_ne!(t1.path(), t2.path());
        let o1 = run_seed(&p1, "project_briefing_minimal", &BTreeMap::new()).expect("s1");
        let o2 = run_seed(&p2, "project_briefing_minimal", &BTreeMap::new()).expect("s2");
        assert!(!o1.claim_ids.is_empty());
        assert!(!o2.claim_ids.is_empty());
        // Independent vaults: wiping/seeding one must not affect the other path.
        assert!(t1.path().exists());
        assert!(t2.path().exists());
        drop(p1);
        drop(p2);
        drop(t1);
        drop(t2);
    }

    #[test]
    fn seed__stable_ids__two_runs_same_claim_ids() {
        let (_t1, p1) = open_hermetic_ports().expect("p1");
        let (_t2, p2) = open_hermetic_ports().expect("p2");
        let o1 = run_seed(&p1, "project_briefing_minimal", &BTreeMap::new()).expect("s1");
        let o2 = run_seed(&p2, "project_briefing_minimal", &BTreeMap::new()).expect("s2");
        assert_eq!(o1.claim_ids, o2.claim_ids);
    }
}
