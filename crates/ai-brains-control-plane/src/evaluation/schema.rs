//! Scenario JSON schema v1 + typed seed-param whitelist (T169 E6).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::{ControlPlaneError, Result};

/// Known seed program names (scenarios 1–9). Scenario 10 uses `runner: sources_tests`.
pub const KNOWN_SEED_PROGRAMS: &[&str] = &[
    "project_briefing_minimal",
    "handoff_interrupted",
    "source_edit_stale",
    "conflict_scoped",
    "personal_and_cross_project",
    "human_correction",
    "source_unavailable",
    "erasure_ce_wipe",
    "path_alias_wsl",
];

/// Known metric names allowed in asserts.
pub const KNOWN_METRICS: &[&str] = &[
    "stale_as_current_count",
    "unauthorized_scope_leakage_count",
    "cross_project_leakage_count",
    "current_claim_count",
    "uncited_current_claim_count",
    "citation_coverage",
    "budget_compliant",
    "latency_ms",
    "conflict_unmerged",
    "independent_support_false_positive",
    "ce_subject_absent",
    "scope_key_stable",
];

/// Params keys allowed for every seed program (all optional; empty object is fine).
const UNIVERSAL_PARAM_KEYS: &[&str] = &["label", "project_label"];

/// Extra allowed param keys per program (empty = only universal).
fn program_param_keys(program: &str) -> &'static [&'static str] {
    match program {
        "path_alias_wsl" => &["win_path", "wsl_path"],
        "erasure_ce_wipe" => &["wipe_reason"],
        "personal_and_cross_project" => &["alpha_label", "beta_label"],
        _ => &[],
    }
}

/// Scenario lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioStatus {
    #[default]
    Active,
    Deferred,
}

/// Comparison operator for a metric assert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertOp {
    Eq,
    Gte,
    Lte,
    Lt,
    Gt,
    Ne,
}

/// One hard or soft metric assert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssertSpec {
    pub metric: String,
    pub op: AssertOp,
    pub value: Value,
}

/// Hard + soft assert groups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AssertsSpec {
    #[serde(default)]
    pub hard: Vec<AssertSpec>,
    #[serde(default)]
    pub soft: Vec<AssertSpec>,
}

/// Seed program reference with typed params object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioSeed {
    pub program: String,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

/// Action the runner executes after seed (v1: build_project_briefing, resolve_scope, wipe).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioAction {
    pub op: String,
    #[serde(default)]
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Versioned scenario fixture (schema_version = 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: ScenarioStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<ScenarioSeed>,
    /// External runner label (e.g. `sources_tests` for scenario 10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<String>,
    /// E23 anti zero-recall floor. Default 1 when omitted for CP-runnable seeds.
    #[serde(default = "default_min_valid_claims")]
    pub min_valid_claims_count: u32,
    #[serde(default)]
    pub actions: Vec<ScenarioAction>,
    #[serde(default)]
    pub asserts: AssertsSpec,
}

fn default_min_valid_claims() -> u32 {
    1
}

/// Load and validate a scenario from a JSON string.
pub fn load_scenario_json(raw: &str) -> Result<Scenario> {
    let scenario: Scenario = serde_json::from_str(raw).map_err(|e| {
        ControlPlaneError::InvalidPayload(format!("scenario JSON parse failed: {e}"))
    })?;
    validate_scenario(&scenario)?;
    Ok(scenario)
}

/// Load and validate a scenario from a filesystem path.
pub fn load_scenario_path(path: &Path) -> Result<Scenario> {
    let raw = fs::read_to_string(path).map_err(|e| {
        ControlPlaneError::InvalidPayload(format!(
            "failed to read scenario {}: {e}",
            path.display()
        ))
    })?;
    load_scenario_json(&raw)
}

/// Validate schema_version, seed program whitelist, metrics, and typed params.
pub fn validate_scenario(scenario: &Scenario) -> Result<()> {
    if scenario.schema_version != 1 {
        return Err(ControlPlaneError::InvalidPayload(format!(
            "unknown scenario schema_version {}; only 1 is supported",
            scenario.schema_version
        )));
    }
    if scenario.id.trim().is_empty() {
        return Err(ControlPlaneError::InvalidPayload(
            "scenario id must be non-empty".into(),
        ));
    }

    if scenario.status == ScenarioStatus::Deferred {
        // Deferred may lack seed; still validate asserts if present.
        validate_asserts(&scenario.asserts)?;
        return Ok(());
    }

    // Active scenarios: either CP seed program or external runner descriptor.
    if let Some(runner) = scenario.runner.as_deref() {
        if runner != "sources_tests" {
            return Err(ControlPlaneError::InvalidPayload(format!(
                "unknown scenario runner '{runner}'"
            )));
        }
        // External-runner scenarios need no CP seed.
        validate_asserts(&scenario.asserts)?;
        return Ok(());
    }

    let seed = scenario.seed.as_ref().ok_or_else(|| {
        ControlPlaneError::InvalidPayload(format!(
            "scenario '{}' status=active requires seed.program or runner",
            scenario.id
        ))
    })?;

    if !KNOWN_SEED_PROGRAMS.contains(&seed.program.as_str()) {
        return Err(ControlPlaneError::InvalidPayload(format!(
            "unknown seed program '{}'",
            seed.program
        )));
    }

    validate_seed_params(&seed.program, &seed.params)?;
    validate_actions(&scenario.actions)?;
    validate_asserts(&scenario.asserts)?;
    Ok(())
}

fn validate_seed_params(program: &str, params: &BTreeMap<String, Value>) -> Result<()> {
    let extra = program_param_keys(program);
    for (key, value) in params {
        let allowed = UNIVERSAL_PARAM_KEYS.contains(&key.as_str()) || extra.contains(&key.as_str());
        if !allowed {
            return Err(ControlPlaneError::InvalidPayload(format!(
                "unknown seed param '{key}' for program '{program}'"
            )));
        }
        // Typed whitelist: only string/bool/number/null for known keys.
        match value {
            Value::String(_) | Value::Bool(_) | Value::Number(_) | Value::Null => {}
            Value::Array(_) | Value::Object(_) => {
                return Err(ControlPlaneError::InvalidPayload(format!(
                    "seed param '{key}' has unsupported type for program '{program}'"
                )));
            }
        }
    }
    Ok(())
}

fn validate_actions(actions: &[ScenarioAction]) -> Result<()> {
    const KNOWN_OPS: &[&str] = &[
        "build_project_briefing",
        "resolve_scope",
        "wipe_content_envelope",
        "noop",
    ];
    for action in actions {
        if !KNOWN_OPS.contains(&action.op.as_str()) {
            return Err(ControlPlaneError::InvalidPayload(format!(
                "unknown action op '{}'",
                action.op
            )));
        }
    }
    Ok(())
}

fn validate_asserts(asserts: &AssertsSpec) -> Result<()> {
    for a in asserts.hard.iter().chain(asserts.soft.iter()) {
        if !KNOWN_METRICS.contains(&a.metric.as_str()) {
            return Err(ControlPlaneError::InvalidPayload(format!(
                "unknown assert metric '{}'",
                a.metric
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    fn minimal_active_json(program: &str) -> String {
        format!(
            r#"{{
              "schema_version": 1,
              "id": "cold_start_cited_project",
              "title": "test",
              "status": "active",
              "seed": {{ "program": "{program}", "params": {{}} }},
              "min_valid_claims_count": 1,
              "actions": [{{ "op": "build_project_briefing", "dry_run": true }}],
              "asserts": {{
                "hard": [
                  {{ "metric": "stale_as_current_count", "op": "eq", "value": 0 }}
                ],
                "soft": []
              }}
            }}"#
        )
    }

    #[test]
    fn scenario_schema__unknown_version__invalid_payload() {
        let raw = r#"{
          "schema_version": 99,
          "id": "x",
          "title": "t",
          "status": "active",
          "seed": { "program": "project_briefing_minimal", "params": {} },
          "actions": [],
          "asserts": { "hard": [], "soft": [] }
        }"#;
        let err = load_scenario_json(raw).expect_err("must reject unknown version");
        assert!(
            matches!(err, ControlPlaneError::InvalidPayload(_)),
            "got {err:?}"
        );
        assert!(err.to_string().contains("schema_version"));
    }

    #[test]
    fn scenario_schema__unknown_seed_program__invalid_payload() {
        let raw = minimal_active_json("not_a_real_program");
        let err = load_scenario_json(&raw).expect_err("must reject unknown program");
        assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
        assert!(err.to_string().contains("unknown seed program"));
    }

    #[test]
    fn scenario_schema__unknown_metric__invalid_payload() {
        let raw = r#"{
          "schema_version": 1,
          "id": "x",
          "title": "t",
          "status": "active",
          "seed": { "program": "project_briefing_minimal", "params": {} },
          "actions": [{ "op": "build_project_briefing", "dry_run": true }],
          "asserts": {
            "hard": [{ "metric": "not_a_metric", "op": "eq", "value": 0 }],
            "soft": []
          }
        }"#;
        let err = load_scenario_json(raw).expect_err("must reject unknown metric");
        assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
        assert!(err.to_string().contains("unknown assert metric"));
    }

    #[test]
    fn scenario_schema__bad_seed_params__invalid_payload() {
        let raw = r#"{
          "schema_version": 1,
          "id": "x",
          "title": "t",
          "status": "active",
          "seed": {
            "program": "project_briefing_minimal",
            "params": { "evil_key": "nope" }
          },
          "actions": [],
          "asserts": { "hard": [], "soft": [] }
        }"#;
        let err = load_scenario_json(raw).expect_err("must reject unknown param");
        assert!(matches!(err, ControlPlaneError::InvalidPayload(_)));
        assert!(err.to_string().contains("unknown seed param"));
    }

    #[test]
    fn scenario_schema__valid_minimal__ok() {
        let s = load_scenario_json(&minimal_active_json("project_briefing_minimal"))
            .expect("valid scenario");
        assert_eq!(s.id, "cold_start_cited_project");
        assert_eq!(s.min_valid_claims_count, 1);
    }

    #[test]
    fn scenario_schema__sources_runner__ok_without_seed() {
        let raw = r#"{
          "schema_version": 1,
          "id": "circular_external_writeback",
          "title": "circularity",
          "status": "active",
          "runner": "sources_tests",
          "min_valid_claims_count": 0,
          "actions": [],
          "asserts": { "hard": [], "soft": [] }
        }"#;
        let s = load_scenario_json(raw).expect("catalog-only scen 10");
        assert_eq!(s.runner.as_deref(), Some("sources_tests"));
    }
}
