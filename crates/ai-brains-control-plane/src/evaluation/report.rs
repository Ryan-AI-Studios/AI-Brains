//! EvaluateReport JSON shape + deterministic report_hash (T169 E7, E18).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::metrics::MetricValues;

/// Soft failure entry listed on the report (never flips hard pass unless strict-soft).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoftFailure {
    pub scenario_id: String,
    pub metric: String,
    pub message: String,
}

/// Per-scenario result row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioResult {
    pub id: String,
    /// passed | failed | skipped
    pub status: String,
    #[serde(default)]
    pub hard: BTreeMap<String, Value>,
    #[serde(default)]
    pub soft: BTreeMap<String, Value>,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

/// Aggregate totals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Totals {
    pub scenarios_total: u32,
    pub scenarios_passed: u32,
    pub scenarios_failed: u32,
    pub scenarios_skipped: u32,
}

/// Deterministic human-review seed for T170 (E18).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HumanReviewSeed {
    /// Up to 20 claim ids sorted by (scenario_id, claim_id).
    pub claim_ids_sample: Vec<String>,
    /// All warning subject/ids from active scenarios, sorted.
    pub warning_ids_all: Vec<String>,
}

/// Top-level evaluate-report.json.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluateReport {
    pub schema_version: u32,
    pub created_at: String,
    pub report_hash: String,
    pub harness: BTreeMap<String, String>,
    pub hard_gates_passed: bool,
    #[serde(default)]
    pub soft_failures: Vec<SoftFailure>,
    pub totals: Totals,
    pub scenarios: Vec<ScenarioResult>,
    pub human_review_seed: HumanReviewSeed,
    #[serde(default)]
    pub limitations: Vec<String>,
}

/// Build claim_ids_sample ≤20 sorted by (scenario_id, claim_id) and sorted warnings.
pub fn build_human_review_seed(
    claim_pairs: &[(String, String)],
    warning_ids: &[String],
) -> HumanReviewSeed {
    let mut pairs = claim_pairs.to_vec();
    pairs.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let claim_ids_sample: Vec<String> = pairs.into_iter().take(20).map(|(_sid, cid)| cid).collect();
    let mut warning_ids_all = warning_ids.to_vec();
    warning_ids_all.sort();
    warning_ids_all.dedup();
    HumanReviewSeed {
        claim_ids_sample,
        warning_ids_all,
    }
}

/// Canonical limitations block for v1 reports.
pub fn default_limitations() -> Vec<String> {
    vec![
        "synthetic fixtures only; not LoCoMo/LongMemEval".into(),
        "no LLM-as-judge".into(),
        "CE honesty: no NIST Purge claim".into(),
        "v1 seeds: Rust programs only (no T168 shadow vault required)".into(),
    ]
}

/// Finalize report: sort scenarios, set totals, compute report_hash, set created_at if empty.
pub fn finalize_report(
    mut report: EvaluateReport,
    created_at: Option<DateTime<Utc>>,
) -> Result<EvaluateReport, String> {
    report.scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    let total = report.scenarios.len() as u32;
    let passed = report
        .scenarios
        .iter()
        .filter(|s| s.status == "passed")
        .count() as u32;
    let failed = report
        .scenarios
        .iter()
        .filter(|s| s.status == "failed")
        .count() as u32;
    let skipped = report
        .scenarios
        .iter()
        .filter(|s| s.status == "skipped")
        .count() as u32;
    report.totals = Totals {
        scenarios_total: total,
        scenarios_passed: passed,
        scenarios_failed: failed,
        scenarios_skipped: skipped,
    };
    if report.limitations.is_empty() {
        report.limitations = default_limitations();
    }
    if report.harness.is_empty() {
        let mut h = BTreeMap::new();
        h.insert("name".into(), "ai-brains-evaluate-governed".into());
        h.insert("version".into(), "1".into());
        report.harness = h;
    }
    report.created_at = created_at
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    report.report_hash = compute_report_hash(&report)?;
    Ok(report)
}

/// Hex SHA-256 of canonical view with created_at and ALL latency_ms stripped (E7).
pub fn compute_report_hash(report: &EvaluateReport) -> Result<String, String> {
    let view = hash_view(report);
    let bytes = serde_json::to_vec(&view).map_err(|e| format!("report_hash serialize: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let dig = hasher.finalize();
    Ok(dig.iter().map(|b| format!("{b:02x}")).collect())
}

/// Canonical hash view: strip created_at, report_hash, and every soft.latency_ms.
fn hash_view(report: &EvaluateReport) -> Value {
    let mut scenarios: Vec<Value> = report
        .scenarios
        .iter()
        .map(|s| {
            let mut soft = BTreeMap::new();
            for (k, v) in &s.soft {
                if k == "latency_ms" {
                    continue;
                }
                soft.insert(k.clone(), v.clone());
            }
            let mut row = BTreeMap::new();
            row.insert("id".into(), Value::String(s.id.clone()));
            row.insert("status".into(), Value::String(s.status.clone()));
            row.insert(
                "hard".into(),
                Value::Object(s.hard.clone().into_iter().collect()),
            );
            row.insert("soft".into(), Value::Object(soft.into_iter().collect()));
            row.insert(
                "messages".into(),
                Value::Array(s.messages.iter().cloned().map(Value::String).collect()),
            );
            row.insert(
                "skip_reason".into(),
                match &s.skip_reason {
                    Some(r) => Value::String(r.clone()),
                    None => Value::Null,
                },
            );
            Value::Object(row.into_iter().collect())
        })
        .collect();
    scenarios.sort_by(|a, b| {
        let ia = a.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let ib = b.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        ia.cmp(ib)
    });

    let soft_failures: Vec<Value> = report
        .soft_failures
        .iter()
        .map(|f| {
            let mut m = BTreeMap::new();
            m.insert("scenario_id".into(), Value::String(f.scenario_id.clone()));
            m.insert("metric".into(), Value::String(f.metric.clone()));
            m.insert("message".into(), Value::String(f.message.clone()));
            Value::Object(m.into_iter().collect())
        })
        .collect();

    let mut totals = BTreeMap::new();
    totals.insert(
        "scenarios_total".into(),
        Value::from(report.totals.scenarios_total),
    );
    totals.insert(
        "scenarios_passed".into(),
        Value::from(report.totals.scenarios_passed),
    );
    totals.insert(
        "scenarios_failed".into(),
        Value::from(report.totals.scenarios_failed),
    );
    totals.insert(
        "scenarios_skipped".into(),
        Value::from(report.totals.scenarios_skipped),
    );

    let mut human = BTreeMap::new();
    human.insert(
        "claim_ids_sample".into(),
        Value::Array(
            report
                .human_review_seed
                .claim_ids_sample
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );
    human.insert(
        "warning_ids_all".into(),
        Value::Array(
            report
                .human_review_seed
                .warning_ids_all
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );

    let mut root = BTreeMap::new();
    root.insert("schema_version".into(), Value::from(report.schema_version));
    root.insert(
        "harness".into(),
        Value::Object(
            report
                .harness
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        ),
    );
    root.insert(
        "hard_gates_passed".into(),
        Value::Bool(report.hard_gates_passed),
    );
    root.insert("soft_failures".into(), Value::Array(soft_failures));
    root.insert("totals".into(), Value::Object(totals.into_iter().collect()));
    root.insert("scenarios".into(), Value::Array(scenarios));
    root.insert(
        "human_review_seed".into(),
        Value::Object(human.into_iter().collect()),
    );
    root.insert(
        "limitations".into(),
        Value::Array(
            report
                .limitations
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        ),
    );
    Value::Object(root.into_iter().collect())
}

/// Helper to attach metrics to a scenario result row.
pub fn scenario_result_from_metrics(
    id: &str,
    status: &str,
    metrics: &MetricValues,
    messages: Vec<String>,
    skip_reason: Option<String>,
) -> ScenarioResult {
    ScenarioResult {
        id: id.to_string(),
        status: status.to_string(),
        hard: metrics.hard_map(),
        soft: metrics.soft_map(),
        messages,
        skip_reason,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_report(latency: u64, created: &str) -> EvaluateReport {
        let mut soft = BTreeMap::new();
        soft.insert(
            "citation_coverage".into(),
            Value::Number(serde_json::Number::from_f64(1.0).expect("1.0")),
        );
        soft.insert("latency_ms".into(), Value::from(latency));
        let mut hard = BTreeMap::new();
        hard.insert("stale_as_current_count".into(), Value::from(0u64));
        EvaluateReport {
            schema_version: 1,
            created_at: created.into(),
            report_hash: String::new(),
            harness: {
                let mut h = BTreeMap::new();
                h.insert("name".into(), "ai-brains-evaluate-governed".into());
                h.insert("version".into(), "1".into());
                h
            },
            hard_gates_passed: true,
            soft_failures: vec![],
            totals: Totals {
                scenarios_total: 1,
                scenarios_passed: 1,
                scenarios_failed: 0,
                scenarios_skipped: 0,
            },
            scenarios: vec![ScenarioResult {
                id: "cold_start_cited_project".into(),
                status: "passed".into(),
                hard,
                soft,
                messages: vec![],
                skip_reason: None,
            }],
            human_review_seed: HumanReviewSeed {
                claim_ids_sample: vec!["c1".into()],
                warning_ids_all: vec![],
            },
            limitations: default_limitations(),
        }
    }

    #[test]
    fn report_hash__same_input_same_hash() {
        let r = sample_report(12, "2026-07-29T00:00:00Z");
        let h1 = compute_report_hash(&r).unwrap();
        let h2 = compute_report_hash(&r).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn report_hash__excludes_created_at() {
        let r1 = sample_report(12, "2026-07-29T00:00:00Z");
        let r2 = sample_report(12, "2099-01-01T00:00:00Z");
        assert_eq!(
            compute_report_hash(&r1).unwrap(),
            compute_report_hash(&r2).unwrap()
        );
    }

    #[test]
    fn report_hash__latency_variation__same_hash() {
        let r1 = sample_report(12, "2026-07-29T00:00:00Z");
        let r2 = sample_report(9999, "2026-07-29T00:00:00Z");
        assert_eq!(
            compute_report_hash(&r1).unwrap(),
            compute_report_hash(&r2).unwrap()
        );
    }

    #[test]
    fn report_hash__reordered_scenarios_same_hash() {
        let mut r1 = sample_report(1, "2026-07-29T00:00:00Z");
        let mut r2 = r1.clone();
        r1.scenarios.push(ScenarioResult {
            id: "zzz_last".into(),
            status: "skipped".into(),
            hard: BTreeMap::new(),
            soft: BTreeMap::new(),
            messages: vec![],
            skip_reason: Some("deferred".into()),
        });
        r2.scenarios.insert(
            0,
            ScenarioResult {
                id: "zzz_last".into(),
                status: "skipped".into(),
                hard: BTreeMap::new(),
                soft: BTreeMap::new(),
                messages: vec![],
                skip_reason: Some("deferred".into()),
            },
        );
        // hash_view sorts by id
        assert_eq!(
            compute_report_hash(&r1).unwrap(),
            compute_report_hash(&r2).unwrap()
        );
    }

    #[test]
    fn human_review_seed__sample_size_and_sort__deterministic() {
        let mut pairs = Vec::new();
        for i in 0..30 {
            pairs.push(("scen-b".to_string(), format!("claim-{i:02}")));
            pairs.push(("scen-a".to_string(), format!("claim-{i:02}")));
        }
        let seed = build_human_review_seed(&pairs, &["w2".into(), "w1".into(), "w1".into()]);
        assert!(seed.claim_ids_sample.len() <= 20);
        assert_eq!(seed.claim_ids_sample.len(), 20);
        // First pairs after sort are (scen-a, claim-00).. then scen-b
        assert_eq!(seed.claim_ids_sample[0], "claim-00");
        assert_eq!(
            seed.warning_ids_all,
            vec!["w1".to_string(), "w2".to_string()]
        );
        let seed2 = build_human_review_seed(&pairs, &["w2".into(), "w1".into()]);
        assert_eq!(seed, seed2);
    }
}
