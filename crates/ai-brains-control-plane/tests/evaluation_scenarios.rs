#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

//! T169 scenario integration: hard gates for CP-runnable scenarios 1–9.

use std::path::PathBuf;

use ai_brains_control_plane::evaluation::{
    EvaluateOptions, evaluate_scenarios, load_scenarios_dir,
};

fn fixtures_dir() -> PathBuf {
    // Crate tests run with CARGO_MANIFEST_DIR = crates/ai-brains-control-plane
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/governed-memory/scenarios")
}

fn run_one(id: &str) {
    let all = load_scenarios_dir(&fixtures_dir()).expect("load fixtures");
    let scen: Vec<_> = all.into_iter().filter(|s| s.id == id).collect();
    assert_eq!(scen.len(), 1, "scenario {id} missing from fixtures");
    let out = evaluate_scenarios(&scen, &EvaluateOptions::default()).expect("evaluate");
    assert!(
        out.report.hard_gates_passed,
        "scenario {id} hard gates failed: {:?}",
        out.report.scenarios
    );
    assert_eq!(out.report.scenarios[0].status, "passed");
}

#[test]
fn scenario__cold_start_cited_project__hard_gates_pass() {
    run_one("cold_start_cited_project");
}

#[test]
fn scenario__interrupted_task_resumption__hard_gates_pass() {
    run_one("interrupted_task_resumption");
}

#[test]
fn scenario__source_edit_stales_conclusion__hard_gates_pass() {
    run_one("source_edit_stales_conclusion");
}

#[test]
fn scenario__conflicting_scoped_claims__hard_gates_pass() {
    run_one("conflicting_scoped_claims");
}

#[test]
fn scenario__personal_and_cross_project_denied__hard_gates_pass() {
    run_one("personal_and_cross_project_denied");
}

#[test]
fn scenario__human_correction_supersedes__hard_gates_pass() {
    run_one("human_correction_supersedes");
}

#[test]
fn scenario__source_unavailable__hard_gates_pass() {
    run_one("source_unavailable");
}

#[test]
fn scenario__erased_evidence_removes_derived__hard_gates_pass() {
    run_one("erased_evidence_removes_derived");
}

#[test]
fn scenario__windows_wsl_repo_alias__scope_key_stable() {
    run_one("windows_wsl_repo_alias");
}

#[test]
fn scenario__circular_external_writeback__skipped_in_cp_runner() {
    let all = load_scenarios_dir(&fixtures_dir()).expect("load");
    let scen: Vec<_> = all
        .into_iter()
        .filter(|s| s.id == "circular_external_writeback")
        .collect();
    let out = evaluate_scenarios(&scen, &EvaluateOptions::default()).expect("eval");
    assert_eq!(out.report.scenarios[0].status, "skipped");
    assert_eq!(
        out.report.scenarios[0].skip_reason.as_deref(),
        Some("runner=sources_tests")
    );
    assert!(out.report.hard_gates_passed);
}

#[test]
fn evaluation__all_active_cp_scenarios__hard_gates_pass() {
    let all = load_scenarios_dir(&fixtures_dir()).expect("load");
    assert!(all.len() >= 10, "expected 10 scenarios, got {}", all.len());
    let out = evaluate_scenarios(&all, &EvaluateOptions::default()).expect("eval all");
    // Scenario 10 skipped; 1–9 must pass hard.
    for s in &out.report.scenarios {
        if s.id == "circular_external_writeback" {
            assert_eq!(s.status, "skipped");
            continue;
        }
        assert_eq!(
            s.status, "passed",
            "scenario {} failed: {:?}",
            s.id, s.messages
        );
    }
    assert!(out.report.hard_gates_passed);
}
