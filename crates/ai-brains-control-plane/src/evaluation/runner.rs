//! Hermetic scenario runner → EvaluateReport (T169 E1, E2, E22, E23, E25).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use ai_brains_contracts::briefings::ProjectBriefingPacket;
use chrono::Utc;
use serde_json::Value;

use super::metrics::{MetricContext, MetricValues, score_packet};
use super::report::{
    EvaluateReport, SoftFailure, build_human_review_seed, default_limitations, finalize_report,
    scenario_result_from_metrics,
};
use super::schema::{
    AssertOp, AssertSpec, Scenario, ScenarioAction, ScenarioStatus, load_scenario_path,
};
use super::seeds::{SeedOutcome, open_hermetic_ports, run_seed};
use ai_brains_core::privacy::Privacy;

use crate::adapters::{StorePorts, SystemClock};
use crate::briefings::{
    BudgetConfig, PersonalBriefingRequest, ProjectBriefingRequest, build_personal_briefing,
    build_project_briefing,
};
use crate::errors::{ControlPlaneError, Result};

/// Options for a full evaluate run.
#[derive(Debug, Clone, Default)]
pub struct EvaluateOptions {
    /// Soft metric failures → hard_gates_passed false / exit 7 when CLI maps.
    pub strict_soft: bool,
    /// Deferred scenarios count as fail.
    pub require_all_active: bool,
    /// Filter to these scenario ids (empty = all).
    pub scenario_filter: Vec<String>,
}

/// Outcome of evaluate_scenarios (report + exit-code hint).
#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub report: EvaluateReport,
    /// 0 hard pass; 7 hard/strict-soft fail; 6 invalid; 1 internal — set by CLI mapping too.
    pub suggested_exit: i32,
}

/// Suggested exit codes (E22).
pub const EXIT_OK: i32 = 0;
pub const EXIT_INTERNAL: i32 = 1;
pub const EXIT_INVALID_PAYLOAD: i32 = 6;
pub const EXIT_HARD_GATE_FAILED: i32 = 7;

pub fn exit_code_for_outcome(outcome: &RunOutcome) -> i32 {
    outcome.suggested_exit
}

/// Load all `*.json` scenarios from a fixtures directory.
pub fn load_scenarios_dir(dir: &Path) -> Result<Vec<Scenario>> {
    if !dir.is_dir() {
        return Err(ControlPlaneError::InvalidPayload(format!(
            "fixtures dir not found: {}",
            dir.display()
        )));
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| {
            ControlPlaneError::InvalidPayload(format!("read fixtures dir {}: {e}", dir.display()))
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("json"))
        })
        .collect();
    paths.sort();
    let mut scenarios = Vec::with_capacity(paths.len());
    for path in paths {
        scenarios.push(load_scenario_path(&path)?);
    }
    scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(scenarios)
}

/// Run all (filtered) scenarios hermetically and aggregate a report.
pub fn evaluate_scenarios(scenarios: &[Scenario], opts: &EvaluateOptions) -> Result<RunOutcome> {
    let mut results = Vec::new();
    let mut soft_failures = Vec::new();
    let mut claim_pairs: Vec<(String, String)> = Vec::new();
    let mut warning_ids: Vec<String> = Vec::new();
    let mut any_hard_fail = false;
    let mut any_soft_fail = false;

    let filtered: Vec<&Scenario> = if opts.scenario_filter.is_empty() {
        scenarios.iter().collect()
    } else {
        scenarios
            .iter()
            .filter(|s| opts.scenario_filter.iter().any(|f| f == &s.id))
            .collect()
    };

    for scenario in filtered {
        if scenario.status == ScenarioStatus::Deferred {
            if opts.require_all_active {
                any_hard_fail = true;
                results.push(scenario_result_from_metrics(
                    &scenario.id,
                    "failed",
                    &MetricValues::default(),
                    vec![format!(
                        "require-all-active: deferred scenario ({})",
                        scenario.defer_reason.as_deref().unwrap_or("no reason")
                    )],
                    scenario.defer_reason.clone(),
                ));
            } else {
                results.push(scenario_result_from_metrics(
                    &scenario.id,
                    "skipped",
                    &MetricValues::default(),
                    vec![],
                    Some(
                        scenario
                            .defer_reason
                            .clone()
                            .unwrap_or_else(|| "deferred".into()),
                    ),
                ));
            }
            continue;
        }

        // External runner (scenario 10): skipped in CP harness with explicit reason.
        if scenario.runner.as_deref() == Some("sources_tests") {
            results.push(scenario_result_from_metrics(
                &scenario.id,
                "skipped",
                &MetricValues::default(),
                vec![],
                Some("runner=sources_tests".into()),
            ));
            continue;
        }

        match run_one_scenario(scenario) {
            Ok(one) => {
                for c in &one.claim_ids {
                    claim_pairs.push((scenario.id.clone(), c.clone()));
                }
                warning_ids.extend(one.warning_ids);
                if !one.hard_ok {
                    any_hard_fail = true;
                }
                if !one.soft_ok {
                    any_soft_fail = true;
                    for sf in one.soft_failure_entries {
                        soft_failures.push(sf);
                    }
                }
                results.push(one.result);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    let hard_gates_passed = !(any_hard_fail || (opts.strict_soft && any_soft_fail));

    let report = EvaluateReport {
        schema_version: 1,
        created_at: String::new(),
        report_hash: String::new(),
        harness: BTreeMap::from([
            ("name".into(), "ai-brains-evaluate-governed".into()),
            ("version".into(), "1".into()),
        ]),
        hard_gates_passed,
        soft_failures,
        totals: Default::default(),
        scenarios: results,
        human_review_seed: build_human_review_seed(&claim_pairs, &warning_ids),
        limitations: default_limitations(),
    };

    let report = finalize_report(report, Some(Utc::now()))
        .map_err(|e| ControlPlaneError::Query(format!("finalize report: {e}")))?;

    let suggested_exit = if hard_gates_passed {
        EXIT_OK
    } else {
        EXIT_HARD_GATE_FAILED
    };

    Ok(RunOutcome {
        report,
        suggested_exit,
    })
}

struct OneResult {
    result: super::report::ScenarioResult,
    hard_ok: bool,
    soft_ok: bool,
    soft_failure_entries: Vec<SoftFailure>,
    claim_ids: Vec<String>,
    warning_ids: Vec<String>,
}

fn run_one_scenario(scenario: &Scenario) -> Result<OneResult> {
    let seed_spec = scenario.seed.as_ref().ok_or_else(|| {
        ControlPlaneError::InvalidPayload(format!("scenario '{}' active without seed", scenario.id))
    })?;

    // Hermetic vault per scenario (E1/E25).
    let (_tmp, ports) = open_hermetic_ports()?;
    let outcome = run_seed(&ports, &seed_spec.program, &seed_spec.params)?;

    let start = Instant::now();
    let packet = run_actions(&ports, &outcome, &scenario.actions)?;
    let latency_ms = start.elapsed().as_millis() as u64;

    let mut warning_ids: Vec<String> = outcome.warning_subject_ids.clone();
    if let Some(ref p) = packet {
        for w in &p.warnings {
            if let Some(sid) = &w.subject_id {
                warning_ids.push(sid.clone());
            }
        }
    }

    let ctx = MetricContext {
        foreign_claim_ids: outcome.foreign_claim_ids.clone(),
        beta_claim_ids: outcome.beta_claim_ids.clone(),
        wiped_subject_id: outcome.wiped_subject_id.clone(),
        must_be_absent_claim_ids: outcome.must_be_absent_claim_ids.clone(),
        conflict_claim_ids: outcome.conflict_claim_ids.clone(),
        scope_keys: outcome.scope_keys.clone(),
        require_citations: outcome.require_citations,
        min_valid_claims_count: scenario.min_valid_claims_count,
    };

    let metrics = score_packet(packet.as_ref(), &ctx, Some(latency_ms));

    let mut messages = Vec::new();
    let mut hard_ok = true;
    let mut soft_ok = true;
    let mut soft_failure_entries = Vec::new();

    // Personal grant denial path (scen 5): principal without Personal grant must
    // receive a denied empty personal briefing.
    if outcome.require_personal_denial {
        match verify_personal_denial(&ports, &outcome) {
            Ok(()) => {}
            Err(msg) => {
                hard_ok = false;
                messages.push(msg);
            }
        }
    }

    // E23 anti zero-recall: always hard-check min_valid_claims_count.
    if metrics.current_claim_count < u64::from(scenario.min_valid_claims_count) {
        hard_ok = false;
        messages.push(format!(
            "E23 min_valid_claims_count: got {} need >= {}",
            metrics.current_claim_count, scenario.min_valid_claims_count
        ));
    }

    for a in &scenario.asserts.hard {
        if !eval_assert(a, &metrics) {
            hard_ok = false;
            messages.push(format!(
                "hard assert failed: {} {:?} {:?}",
                a.metric, a.op, a.value
            ));
        }
    }
    for a in &scenario.asserts.soft {
        if a.metric == "citation_coverage" && metrics.citation_coverage.is_none() {
            // N/A soft-skip
            continue;
        }
        if !eval_assert(a, &metrics) {
            soft_ok = false;
            let msg = format!("soft assert failed: {} {:?} {:?}", a.metric, a.op, a.value);
            messages.push(msg.clone());
            soft_failure_entries.push(SoftFailure {
                scenario_id: scenario.id.clone(),
                metric: a.metric.clone(),
                message: msg,
            });
        }
    }

    // Path-alias scenario: enforce scope_key_stable when keys present.
    if outcome.scope_keys.is_some() && !metrics.scope_key_stable {
        hard_ok = false;
        messages.push("scope_key_stable hard fail".into());
    }

    let status = if hard_ok { "passed" } else { "failed" };
    let mut claim_ids = outcome.claim_ids.clone();
    if let Some(ref p) = packet {
        for c in p.decisions.iter().chain(p.conclusions.iter()) {
            if !claim_ids.contains(&c.id) {
                claim_ids.push(c.id.clone());
            }
        }
    }

    Ok(OneResult {
        result: scenario_result_from_metrics(&scenario.id, status, &metrics, messages, None),
        hard_ok,
        soft_ok,
        soft_failure_entries,
        claim_ids,
        warning_ids,
    })
}

fn run_actions(
    ports: &StorePorts,
    outcome: &SeedOutcome,
    actions: &[ScenarioAction],
) -> Result<Option<ProjectBriefingPacket>> {
    if actions.is_empty() {
        // Default: build briefing when not path-only.
        if outcome.scope_keys.is_some() && outcome.expect_denied {
            return Ok(None);
        }
        return Ok(Some(build_briefing(ports, outcome)?));
    }

    let mut last_packet = None;
    for action in actions {
        match action.op.as_str() {
            "build_project_briefing" => {
                last_packet = Some(build_briefing(ports, outcome)?);
            }
            "resolve_scope" => {
                // Keys already resolved in seed; nothing else required.
            }
            "wipe_content_envelope" => {
                // Wipe already performed in erasure seed.
            }
            "noop" => {}
            other => {
                return Err(ControlPlaneError::InvalidPayload(format!(
                    "unknown action op '{other}'"
                )));
            }
        }
    }
    Ok(last_packet)
}

fn build_briefing(ports: &StorePorts, outcome: &SeedOutcome) -> Result<ProjectBriefingPacket> {
    let policy = ports.production_policy();
    let identity = ports.identity_store();
    build_project_briefing(
        Some(&ports.writer),
        &ports.query,
        &SystemClock,
        &policy,
        &identity,
        ProjectBriefingRequest {
            principal: outcome.principal.clone(),
            resolve: outcome.resolve.clone(),
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
            ledgerful: None,
        },
    )
}

/// Personal briefing for principal without Personal grant must be denied/empty.
fn verify_personal_denial(
    ports: &StorePorts,
    outcome: &SeedOutcome,
) -> std::result::Result<(), String> {
    let user_id = outcome.personal_user_id.ok_or_else(|| {
        "require_personal_denial set but personal_user_id missing from seed".to_string()
    })?;
    let policy = ports.production_policy();
    // Alpha principal has no Personal grant; empty applied-grants list is correct.
    let packet = build_personal_briefing(
        Some(&ports.writer),
        &ports.query,
        &SystemClock,
        &policy,
        |_p| Ok(vec![]),
        PersonalBriefingRequest {
            principal: outcome.principal.clone(),
            user_id,
            budget: BudgetConfig::default(),
            privacy: Privacy::LocalOnly,
            dry_run: true,
            briefing_id: None,
        },
    )
    .map_err(|e| format!("personal denial briefing failed: {e}"))?;

    if !packet.denied {
        return Err(
            "personal denial hard fail: expected denied personal briefing without Personal grant"
                .into(),
        );
    }
    if !packet.preferences.is_empty() {
        return Err(format!(
            "personal denial hard fail: expected empty preferences, got {}",
            packet.preferences.len()
        ));
    }
    Ok(())
}

fn eval_assert(spec: &AssertSpec, metrics: &MetricValues) -> bool {
    let Some(actual) = metrics.get(&spec.metric) else {
        return false;
    };
    compare(&actual, spec.op, &spec.value)
}

fn compare(actual: &Value, op: AssertOp, expected: &Value) -> bool {
    // Boolean
    if let (Some(a), Some(e)) = (actual.as_bool(), expected.as_bool()) {
        return match op {
            AssertOp::Eq => a == e,
            AssertOp::Ne => a != e,
            _ => false,
        };
    }
    // Integer-like
    if let (Some(a), Some(e)) = (as_f64(actual), as_f64(expected)) {
        return match op {
            AssertOp::Eq => (a - e).abs() < 1e-9,
            AssertOp::Ne => (a - e).abs() >= 1e-9,
            AssertOp::Gte => a >= e,
            AssertOp::Lte => a <= e,
            AssertOp::Gt => a > e,
            AssertOp::Lt => a < e,
        };
    }
    // Fallback string equality
    match op {
        AssertOp::Eq => actual == expected,
        AssertOp::Ne => actual != expected,
        _ => false,
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|i| i as f64))
        .or_else(|| v.as_u64().map(|u| u as f64))
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::evaluation::schema::{AssertsSpec, ScenarioSeed};

    fn active_scenario(id: &str, program: &str, min_claims: u32) -> Scenario {
        Scenario {
            schema_version: 1,
            id: id.into(),
            title: id.into(),
            tags: vec![],
            status: ScenarioStatus::Active,
            defer_reason: None,
            owner: None,
            seed: Some(ScenarioSeed {
                program: program.into(),
                params: BTreeMap::new(),
            }),
            runner: None,
            min_valid_claims_count: min_claims,
            actions: vec![ScenarioAction {
                op: "build_project_briefing".into(),
                dry_run: Some(true),
                path: None,
                extra: BTreeMap::new(),
            }],
            asserts: AssertsSpec {
                hard: vec![
                    AssertSpec {
                        metric: "stale_as_current_count".into(),
                        op: AssertOp::Eq,
                        value: Value::from(0),
                    },
                    AssertSpec {
                        metric: "unauthorized_scope_leakage_count".into(),
                        op: AssertOp::Eq,
                        value: Value::from(0),
                    },
                ],
                soft: vec![],
            },
        }
    }

    #[test]
    fn runner__soft_fail_default__hard_gates_passed_true() {
        let mut s = active_scenario("cold", "project_briefing_minimal", 1);
        s.asserts.soft.push(AssertSpec {
            metric: "citation_coverage".into(),
            op: AssertOp::Gte,
            value: Value::Number(serde_json::Number::from_f64(2.0).expect("2.0")),
        });
        let out = evaluate_scenarios(&[s], &EvaluateOptions::default()).expect("run");
        assert!(out.report.hard_gates_passed);
        assert!(!out.report.soft_failures.is_empty());
        assert_eq!(out.suggested_exit, EXIT_OK);
    }

    #[test]
    fn runner__strict_soft__fails_quality_gate() {
        let mut s = active_scenario("cold", "project_briefing_minimal", 1);
        s.asserts.soft.push(AssertSpec {
            metric: "citation_coverage".into(),
            op: AssertOp::Gte,
            value: Value::Number(serde_json::Number::from_f64(2.0).expect("2.0")),
        });
        let opts = EvaluateOptions {
            strict_soft: true,
            ..Default::default()
        };
        let out = evaluate_scenarios(&[s], &opts).expect("run");
        assert!(!out.report.hard_gates_passed);
        assert_eq!(out.suggested_exit, EXIT_HARD_GATE_FAILED);
    }

    #[test]
    fn runner__deferred_scenario__skipped_not_omitted() {
        let deferred = Scenario {
            schema_version: 1,
            id: "future_thing".into(),
            title: "deferred".into(),
            tags: vec![],
            status: ScenarioStatus::Deferred,
            defer_reason: Some("not yet implemented".into()),
            owner: Some("T170".into()),
            seed: None,
            runner: None,
            min_valid_claims_count: 0,
            actions: vec![],
            asserts: AssertsSpec::default(),
        };
        let out = evaluate_scenarios(&[deferred], &EvaluateOptions::default()).expect("run");
        assert_eq!(out.report.scenarios.len(), 1);
        assert_eq!(out.report.scenarios[0].status, "skipped");
        assert!(out.report.hard_gates_passed);
    }

    #[test]
    fn runner__require_all_active__deferred_fails() {
        let deferred = Scenario {
            schema_version: 1,
            id: "future_thing".into(),
            title: "deferred".into(),
            tags: vec![],
            status: ScenarioStatus::Deferred,
            defer_reason: Some("later".into()),
            owner: Some("T170".into()),
            seed: None,
            runner: None,
            min_valid_claims_count: 0,
            actions: vec![],
            asserts: AssertsSpec::default(),
        };
        let opts = EvaluateOptions {
            require_all_active: true,
            ..Default::default()
        };
        let out = evaluate_scenarios(&[deferred], &opts).expect("run");
        assert!(!out.report.hard_gates_passed);
        assert_eq!(out.suggested_exit, EXIT_HARD_GATE_FAILED);
    }

    #[test]
    fn runner__zero_recall_when_min_one__hard_fail() {
        // path_alias seeds no authority claims; min=1 forces E23 hard fail.
        let s = active_scenario("path", "path_alias_wsl", 1);
        let out = evaluate_scenarios(&[s], &EvaluateOptions::default()).expect("run");
        assert!(!out.report.hard_gates_passed);
        assert_eq!(out.suggested_exit, EXIT_HARD_GATE_FAILED);
    }

    #[test]
    fn runner__hard_fail__exit_7() {
        let s = active_scenario("path", "path_alias_wsl", 1);
        let out = evaluate_scenarios(&[s], &EvaluateOptions::default()).expect("run");
        assert_eq!(out.suggested_exit, EXIT_HARD_GATE_FAILED);
    }

    #[test]
    fn runner__hermetic__two_scenarios_isolated_vaults() {
        let a = active_scenario("a", "project_briefing_minimal", 1);
        let b = active_scenario("b", "project_briefing_minimal", 1);
        let out = evaluate_scenarios(&[a, b], &EvaluateOptions::default()).expect("run");
        assert!(out.report.hard_gates_passed, "{:?}", out.report.scenarios);
        assert_eq!(out.report.scenarios.len(), 2);
    }

    #[test]
    fn report_hash__two_evaluate_runs__same_hash() {
        // Cold-start alone is enough for e2e determinism of claim ids + hash.
        let s = active_scenario("cold", "project_briefing_minimal", 1);
        let out1 = evaluate_scenarios(std::slice::from_ref(&s), &EvaluateOptions::default())
            .expect("run1");
        let out2 =
            evaluate_scenarios(&[s], &EvaluateOptions::default()).expect("run2");
        assert!(
            out1.report.hard_gates_passed,
            "run1: {:?}",
            out1.report.scenarios
        );
        assert!(
            out2.report.hard_gates_passed,
            "run2: {:?}",
            out2.report.scenarios
        );
        assert_eq!(
            out1.report.report_hash, out2.report.report_hash,
            "report_hash must be stable across two evaluate runs (E7); seed ids={:?} vs {:?}",
            out1.report.human_review_seed.claim_ids_sample,
            out2.report.human_review_seed.claim_ids_sample
        );
        assert!(!out1.report.report_hash.is_empty());
    }
}
