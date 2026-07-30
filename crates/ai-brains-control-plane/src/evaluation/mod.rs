//! Governed-memory evaluation harness (T169 / P9.3).
//!
//! Pure metrics, versioned scenario schema, seed programs (1–9), and a hermetic
//! runner that never mutates a live vault. Scenario 10 (circularity) lives in
//! `ai-brains-sources` tests — this module does not re-export those helpers as
//! a seed program.

pub mod metrics;
pub mod report;
pub mod runner;
pub mod schema;
pub mod seeds;

pub use metrics::{
    MetricContext, MetricValues, budget_compliant, ce_subject_absent, citation_coverage,
    conflict_unmerged, cross_project_leakage_count, current_claim_count, scope_key_stable,
    score_packet, stale_as_current_count, unauthorized_scope_leakage_count,
    uncited_current_claim_count,
};
pub use report::{
    EvaluateReport, HumanReviewSeed, ScenarioResult, SoftFailure, Totals, compute_report_hash,
    finalize_report,
};
pub use runner::{
    EvaluateOptions, RunOutcome, evaluate_scenarios, exit_code_for_outcome, load_scenarios_dir,
};
pub use schema::{
    AssertOp, AssertSpec, AssertsSpec, Scenario, ScenarioAction, ScenarioSeed, ScenarioStatus,
    load_scenario_json, load_scenario_path, validate_scenario,
};
pub use seeds::{SEED_PROGRAMS, SeedOutcome, is_known_seed_program, run_seed};
