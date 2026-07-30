//! Pure governed-memory metric scorers (T169 E9a, E10, E11, E12, E14, E16, E23).

use std::collections::{BTreeMap, BTreeSet};

use ai_brains_contracts::briefings::ProjectBriefingPacket;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Warning kinds that must never appear as current authority (E9a).
const STALE_WARNING_KINDS: &[&str] = &["stale", "disputed", "rejected", "unavailable"];

/// Context for metrics that need seed-side metadata (foreign ids, wipe subject, paths).
#[derive(Debug, Clone, Default)]
pub struct MetricContext {
    /// Claim ids the principal must not see (unauthorized foreign scope).
    pub foreign_claim_ids: BTreeSet<String>,
    /// Project-Beta claim ids (cross-project leakage when briefing as Alpha).
    pub beta_claim_ids: BTreeSet<String>,
    /// Subject id wiped via CE (scen 8).
    pub wiped_subject_id: Option<String>,
    /// Claim ids that must not appear as current authority (stale/superseded/wiped).
    pub must_be_absent_claim_ids: BTreeSet<String>,
    /// Pair of incompatible claims (scen 4); silent merge when both current without warning.
    pub conflict_claim_ids: Option<(String, String)>,
    /// Two resolved scope_key strings (scen 9).
    pub scope_keys: Option<(String, String)>,
    /// When true, uncited claims are scored (default true).
    pub require_citations: bool,
    /// E23 floor from scenario.
    pub min_valid_claims_count: u32,
}

/// Aggregated metric values for one scenario run.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MetricValues {
    pub stale_as_current_count: u64,
    pub unauthorized_scope_leakage_count: u64,
    pub cross_project_leakage_count: u64,
    pub current_claim_count: u64,
    pub uncited_current_claim_count: u64,
    /// `null` when N/A (min_valid=0 and zero claims).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_coverage: Option<f64>,
    pub budget_compliant: bool,
    pub conflict_unmerged: u64,
    pub ce_subject_absent: u64,
    /// Count of `must_be_absent_claim_ids` still present in current authority (0 = pass).
    pub must_be_absent_present_count: u64,
    pub scope_key_stable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl MetricValues {
    /// Look up a metric by name as JSON value (for assert evaluation).
    pub fn get(&self, metric: &str) -> Option<Value> {
        Some(match metric {
            "stale_as_current_count" => Value::from(self.stale_as_current_count),
            "unauthorized_scope_leakage_count" => {
                Value::from(self.unauthorized_scope_leakage_count)
            }
            "cross_project_leakage_count" => Value::from(self.cross_project_leakage_count),
            "current_claim_count" => Value::from(self.current_claim_count),
            "uncited_current_claim_count" => Value::from(self.uncited_current_claim_count),
            "citation_coverage" => match self.citation_coverage {
                Some(v) => number_value(v),
                None => Value::Null,
            },
            "budget_compliant" => Value::Bool(self.budget_compliant),
            "conflict_unmerged" => Value::from(self.conflict_unmerged),
            "ce_subject_absent" => Value::from(self.ce_subject_absent),
            "must_be_absent_present_count" => Value::from(self.must_be_absent_present_count),
            "scope_key_stable" => Value::Bool(self.scope_key_stable),
            "latency_ms" => match self.latency_ms {
                Some(v) => Value::from(v),
                None => Value::Null,
            },
            // Sources-only metric: not scored by CP harness (P1-03). Returning None
            // makes any assert fail closed if schema validation is bypassed.
            "independent_support_false_positive" => return None,
            _ => return None,
        })
    }

    /// Hard metrics map for report (BTreeMap order).
    pub fn hard_map(&self) -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        m.insert(
            "stale_as_current_count".into(),
            Value::from(self.stale_as_current_count),
        );
        m.insert(
            "unauthorized_scope_leakage_count".into(),
            Value::from(self.unauthorized_scope_leakage_count),
        );
        m.insert(
            "cross_project_leakage_count".into(),
            Value::from(self.cross_project_leakage_count),
        );
        m.insert(
            "current_claim_count".into(),
            Value::from(self.current_claim_count),
        );
        m.insert(
            "uncited_current_claim_count".into(),
            Value::from(self.uncited_current_claim_count),
        );
        m.insert(
            "conflict_unmerged".into(),
            Value::from(self.conflict_unmerged),
        );
        m.insert(
            "ce_subject_absent".into(),
            Value::from(self.ce_subject_absent),
        );
        m.insert(
            "must_be_absent_present_count".into(),
            Value::from(self.must_be_absent_present_count),
        );
        m.insert(
            "scope_key_stable".into(),
            Value::Bool(self.scope_key_stable),
        );
        m
    }

    /// Soft metrics map for report (includes latency when set).
    pub fn soft_map(&self) -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        if let Some(c) = self.citation_coverage {
            m.insert("citation_coverage".into(), number_value(c));
        }
        m.insert(
            "budget_compliant".into(),
            Value::Bool(self.budget_compliant),
        );
        if let Some(ms) = self.latency_ms {
            m.insert("latency_ms".into(), Value::from(ms));
        }
        m
    }
}

/// Sources-only circularity metric cannot be scored without sources runner context.
/// Calling this from CP evaluates to an error (never a silent zero).
pub fn independent_support_false_positive_cp_unavailable() -> Result<u64, &'static str> {
    Err(
        "independent_support_false_positive is only valid for runner=sources_tests; \
         control-plane score_packet has no circularity context",
    )
}

fn number_value(v: f64) -> Value {
    match serde_json::Number::from_f64(v) {
        Some(n) => Value::Number(n),
        None => Value::Null,
    }
}

/// E9a: warning kind ∈ {stale, disputed, rejected, unavailable} AND subject in current authority.
pub fn stale_as_current_count(packet: &ProjectBriefingPacket) -> u64 {
    let current: BTreeSet<&str> = packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .map(|c| c.id.as_str())
        .collect();
    let mut count = 0u64;
    for w in &packet.warnings {
        let kind = w.kind.to_ascii_lowercase();
        if !STALE_WARNING_KINDS.contains(&kind.as_str()) {
            continue;
        }
        if let Some(sid) = w.subject_id.as_deref()
            && current.contains(sid)
        {
            count += 1;
        }
    }
    count
}

/// Count of foreign-scope authority claims visible when principal lacks grant.
/// Denied empty packets score 0.
pub fn unauthorized_scope_leakage_count(
    packet: &ProjectBriefingPacket,
    foreign_claim_ids: &BTreeSet<String>,
) -> u64 {
    if packet.denied {
        return 0;
    }
    if foreign_claim_ids.is_empty() {
        return 0;
    }
    packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .filter(|c| foreign_claim_ids.contains(&c.id))
        .count() as u64
}

/// Beta claim ids visible when briefing as Alpha principal.
pub fn cross_project_leakage_count(
    packet: &ProjectBriefingPacket,
    beta_claim_ids: &BTreeSet<String>,
) -> u64 {
    if beta_claim_ids.is_empty() {
        return 0;
    }
    packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .filter(|c| beta_claim_ids.contains(&c.id))
        .count() as u64
}

/// `decisions.len() + conclusions.len()`.
pub fn current_claim_count(packet: &ProjectBriefingPacket) -> u64 {
    (packet.decisions.len() + packet.conclusions.len()) as u64
}

/// Current claims with empty `evidence_handles`.
pub fn uncited_current_claim_count(packet: &ProjectBriefingPacket) -> u64 {
    packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .filter(|c| c.evidence_handles.is_empty())
        .count() as u64
}

/// Cited / max(claims, 1). Returns `None` when min_valid=0 and zero claims (soft-skip N/A).
pub fn citation_coverage(packet: &ProjectBriefingPacket, min_valid_claims: u32) -> Option<f64> {
    let claims = current_claim_count(packet);
    if min_valid_claims == 0 && claims == 0 {
        return None;
    }
    let cited = claims.saturating_sub(uncited_current_claim_count(packet));
    let denom = claims.max(1) as f64;
    Some(cited as f64 / denom)
}

/// Soft budget: used_words <= max_words (prefer used<=max; truncate flags also ok).
pub fn budget_compliant(packet: &ProjectBriefingPacket) -> bool {
    let b = &packet.budget;
    if b.max_words == 0 {
        return true;
    }
    b.used_words <= b.max_words || !b.truncated_sections.is_empty()
}

/// Silent merge of incompatible claims into authority without open_conflict warning.
/// Returns fail count (≥1 when both present without warning).
pub fn conflict_unmerged(packet: &ProjectBriefingPacket, pair: Option<&(String, String)>) -> u64 {
    let Some((a, b)) = pair else {
        return 0;
    };
    let current: BTreeSet<&str> = packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .map(|c| c.id.as_str())
        .collect();
    if !current.contains(a.as_str()) || !current.contains(b.as_str()) {
        return 0;
    }
    let has_conflict_warning = packet.warnings.iter().any(|w| {
        let kind = w.kind.to_ascii_lowercase();
        kind == "open_conflict"
            || kind == "disputed"
            || w.subject_id.as_deref() == Some(a.as_str())
            || w.subject_id.as_deref() == Some(b.as_str())
    });
    if has_conflict_warning { 0 } else { 1 }
}

/// Fail count when wiped subject still appears in authority (0 when absent = pass).
pub fn ce_subject_absent(packet: &ProjectBriefingPacket, wiped_subject_id: Option<&str>) -> u64 {
    let Some(sid) = wiped_subject_id else {
        return 0;
    };
    let present = packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .any(|c| c.id == sid);
    if present { 1 } else { 0 }
}

/// Count of claim ids that must be absent but are still current authority (0 = pass).
pub fn must_be_absent_present_count(
    packet: &ProjectBriefingPacket,
    must_be_absent: &BTreeSet<String>,
) -> u64 {
    if must_be_absent.is_empty() {
        return 0;
    }
    packet
        .decisions
        .iter()
        .chain(packet.conclusions.iter())
        .filter(|c| must_be_absent.contains(&c.id))
        .count() as u64
}

/// Two path spellings resolve to the same scope_key.
pub fn scope_key_stable(keys: Option<&(String, String)>) -> bool {
    match keys {
        Some((a, b)) => !a.is_empty() && a == b,
        None => true,
    }
}

/// Score a full packet + context into [`MetricValues`].
pub fn score_packet(
    packet: Option<&ProjectBriefingPacket>,
    ctx: &MetricContext,
    latency_ms: Option<u64>,
) -> MetricValues {
    let empty = ProjectBriefingPacket::empty_denied(
        "eval-empty".into(),
        ai_brains_contracts::briefings::BriefingScopeDto {
            scope_key: String::new(),
            confidence: "Low".into(),
            warnings: Vec::new(),
            alternatives: Vec::new(),
            authoritative: false,
        },
        "no packet",
    );
    let p = packet.unwrap_or(&empty);
    MetricValues {
        stale_as_current_count: stale_as_current_count(p),
        unauthorized_scope_leakage_count: unauthorized_scope_leakage_count(
            p,
            &ctx.foreign_claim_ids,
        ),
        cross_project_leakage_count: cross_project_leakage_count(p, &ctx.beta_claim_ids),
        current_claim_count: current_claim_count(p),
        uncited_current_claim_count: if ctx.require_citations {
            uncited_current_claim_count(p)
        } else {
            0
        },
        citation_coverage: citation_coverage(p, ctx.min_valid_claims_count),
        budget_compliant: budget_compliant(p),
        conflict_unmerged: conflict_unmerged(p, ctx.conflict_claim_ids.as_ref()),
        ce_subject_absent: ce_subject_absent(p, ctx.wiped_subject_id.as_deref()),
        must_be_absent_present_count: must_be_absent_present_count(
            p,
            &ctx.must_be_absent_claim_ids,
        ),
        scope_key_stable: scope_key_stable(ctx.scope_keys.as_ref()),
        latency_ms,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_contracts::briefings::{
        BriefingClaimDto, BriefingScopeDto, BriefingWarningDto, BudgetReportDto,
        FreshnessSummaryDto, ProjectBriefingPacket,
    };
    use ai_brains_contracts::knowledge::EvidenceHandle;

    fn scope() -> BriefingScopeDto {
        BriefingScopeDto {
            scope_key: "Repository:alpha".into(),
            confidence: "High".into(),
            warnings: Vec::new(),
            alternatives: Vec::new(),
            authoritative: true,
        }
    }

    fn freshness() -> FreshnessSummaryDto {
        FreshnessSummaryDto {
            total_sources: 0,
            fresh_count: 0,
            stale_count: 0,
            unavailable_count: 0,
            worst_state: "Unknown".into(),
        }
    }

    fn budget(used: usize, max: usize) -> BudgetReportDto {
        BudgetReportDto {
            max_words: max,
            used_words: used,
            truncated_sections: Vec::new(),
            more_available: false,
        }
    }

    fn claim(id: &str, cited: bool) -> BriefingClaimDto {
        BriefingClaimDto {
            id: id.into(),
            kind: "Decision".into(),
            statement: "s".into(),
            state: "Approved".into(),
            evidence_handles: if cited {
                vec![EvidenceHandle {
                    evidence_id: "e1".into(),
                    cite_label: None,
                }]
            } else {
                vec![]
            },
            title: None,
        }
    }

    fn packet(
        decisions: Vec<BriefingClaimDto>,
        conclusions: Vec<BriefingClaimDto>,
        warnings: Vec<BriefingWarningDto>,
        denied: bool,
        used: usize,
        max: usize,
    ) -> ProjectBriefingPacket {
        ProjectBriefingPacket {
            api_version: "1".into(),
            briefing_id: "b1".into(),
            kind: "Project".into(),
            scope: scope(),
            handoff: None,
            decisions,
            conclusions,
            constraints: Vec::new(),
            warnings,
            freshness: freshness(),
            ledgerful: None,
            evidence_handles: Vec::new(),
            budget: budget(used, max),
            generated_at: None,
            denied,
            denial_reason: if denied {
                Some("no grant".into())
            } else {
                None
            },
        }
    }

    #[test]
    fn metric_stale_as_current__warning_subject_in_current__counts() {
        let p = packet(
            vec![claim("c1", true)],
            vec![],
            vec![BriefingWarningDto {
                kind: "stale".into(),
                message: "stale".into(),
                subject_id: Some("c1".into()),
                subject_kind: Some("Conclusion".into()),
            }],
            false,
            10,
            100,
        );
        assert!(stale_as_current_count(&p) >= 1);
    }

    #[test]
    fn metric_stale_as_current__stale_only_in_warnings__zero() {
        let p = packet(
            vec![claim("c-live", true)],
            vec![],
            vec![BriefingWarningDto {
                kind: "stale".into(),
                message: "stale".into(),
                subject_id: Some("c-stale".into()),
                subject_kind: Some("Conclusion".into()),
            }],
            false,
            10,
            100,
        );
        assert_eq!(stale_as_current_count(&p), 0);
    }

    #[test]
    fn metric_scope_leakage__foreign_decision_visible__counts() {
        let p = packet(
            vec![claim("foreign-1", true)],
            vec![],
            vec![],
            false,
            1,
            100,
        );
        let mut foreign = BTreeSet::new();
        foreign.insert("foreign-1".into());
        assert!(unauthorized_scope_leakage_count(&p, &foreign) >= 1);
    }

    #[test]
    fn metric_scope_leakage__denied_packet__zero() {
        let p = packet(vec![], vec![], vec![], true, 0, 100);
        let mut foreign = BTreeSet::new();
        foreign.insert("foreign-1".into());
        assert_eq!(unauthorized_scope_leakage_count(&p, &foreign), 0);
    }

    #[test]
    fn metric_cross_project_leakage__beta_visible_to_alpha__counts() {
        let p = packet(vec![claim("beta-d1", true)], vec![], vec![], false, 1, 100);
        let mut beta = BTreeSet::new();
        beta.insert("beta-d1".into());
        assert!(cross_project_leakage_count(&p, &beta) >= 1);
    }

    #[test]
    fn metric_current_claim_count__empty_when_min_one__hard_fail() {
        let p = packet(vec![], vec![], vec![], false, 0, 100);
        let count = current_claim_count(&p);
        assert_eq!(count, 0);
        // E23: empty with min=1 is a hard floor miss (asserted by runner).
        let min_valid = 1u64;
        assert!(count < min_valid);
    }

    #[test]
    fn metric_citation_coverage__all_cited__one() {
        let p = packet(
            vec![claim("d1", true)],
            vec![claim("c1", true)],
            vec![],
            false,
            10,
            100,
        );
        let cov = citation_coverage(&p, 1).expect("coverage");
        assert!((cov - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn metric_citation_coverage__missing_handle__lt_one() {
        let p = packet(
            vec![claim("d1", true), claim("d2", false)],
            vec![],
            vec![],
            false,
            10,
            100,
        );
        let cov = citation_coverage(&p, 1).expect("coverage");
        assert!(cov < 1.0);
    }

    #[test]
    fn metric_budget_compliant__over_budget__false() {
        let p = packet(vec![claim("d1", true)], vec![], vec![], false, 200, 100);
        assert!(!budget_compliant(&p));
    }

    #[test]
    fn metric_conflict_unmerged__silent_merge__counts() {
        let p = packet(
            vec![claim("a", true), claim("b", true)],
            vec![],
            vec![],
            false,
            10,
            100,
        );
        let pair = ("a".into(), "b".into());
        assert!(conflict_unmerged(&p, Some(&pair)) >= 1);
    }

    #[test]
    fn metric_ce_subject_absent__wiped_in_authority__counts() {
        let p = packet(vec![claim("wiped-1", true)], vec![], vec![], false, 1, 100);
        assert!(ce_subject_absent(&p, Some("wiped-1")) >= 1);
        assert_eq!(ce_subject_absent(&p, Some("other")), 0);
    }

    #[test]
    fn metric_must_be_absent__still_in_authority__counts() {
        let p = packet(
            vec![claim("live", true)],
            vec![claim("stale-dep", true)],
            vec![],
            false,
            1,
            100,
        );
        let mut absent = BTreeSet::new();
        absent.insert("stale-dep".into());
        assert!(must_be_absent_present_count(&p, &absent) >= 1);
    }

    #[test]
    fn metric_must_be_absent__not_in_authority__zero() {
        let p = packet(vec![claim("live", true)], vec![], vec![], false, 1, 100);
        let mut absent = BTreeSet::new();
        absent.insert("stale-dep".into());
        assert_eq!(must_be_absent_present_count(&p, &absent), 0);
    }

    #[test]
    fn metric_scope_key_stable__alias_variants__same_key() {
        assert!(scope_key_stable(Some(&(
            "Repository:abc".into(),
            "Repository:abc".into()
        ))));
        assert!(!scope_key_stable(Some(&(
            "Repository:a".into(),
            "Repository:b".into()
        ))));
    }

    #[test]
    fn metric_independent_support__cp_get__none_not_fake_zero() {
        let m = MetricValues::default();
        assert!(
            m.get("independent_support_false_positive").is_none(),
            "CP MetricValues must not silently score independent_support_false_positive as 0"
        );
        assert!(independent_support_false_positive_cp_unavailable().is_err());
    }
}
