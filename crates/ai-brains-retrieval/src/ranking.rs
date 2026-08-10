//! Pin-type authority + recency re-ranking for blended recall hits (T211).
//!
//! # F40 — single post-blend ranking entry point
//!
//! [`rerank_hits`] is the **only** final ranking step after blend/graph and
//! before truncate. Semantic / hybrid embedding relevance (**T215**) must
//! extend this function rather than introducing a second final sort.

use crate::recall::RecallHit;

// ---------------------------------------------------------------------------
// F9 — boost magnitudes (frozen)
// ---------------------------------------------------------------------------

/// CONSTRAINT: kind boost.
pub const KIND_CONSTRAINT: f64 = 4.0;
/// DECISION: kind boost.
pub const KIND_DECISION: f64 = 2.0;
/// HOTSPOT: kind boost.
pub const KIND_HOTSPOT: f64 = 0.5;
/// Other / unmarked content kind boost.
pub const KIND_OTHER: f64 = 0.0;
/// Shipped Decision boost.
pub const SHIPPED_BOOST: f64 = 1.0;
/// Plan Decision penalty (subtracted).
pub const PLAN_PENALTY: f64 = 3.0;
/// Extra sibling-Plan penalty when a Shipped Decision shares a track token.
pub const SIBLING_PLAN_PENALTY: f64 = 2.0;
/// Recency age clamp (days).
pub const RECENCY_MAX_DAYS: f64 = 365.0;
/// Recency scale: boost = scale * (1 - d/365).
pub const RECENCY_SCALE: f64 = 1.0;

/// Scale for cosine / RRF scores into pin-boost composite space (T215 F8).
///
/// Rank-1 alone under RRF k=60 is ≈ 1/61 ≈ 0.0164 → ≈ 8.2 after scale.
/// Bridge Tantivy scores do **not** use this scale ([`ScoreKind::BridgeHigherIsBetter`]).
pub const RELEVANCE_SCALE: f64 = 500.0;

/// How [`RecallHit::score`](crate::recall::RecallHit::score) should enter the
/// composite effective score (T215 F6).
///
/// Set at every construction site; graph inherits the parent's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScoreKind {
    /// FTS5 BM25 rank: more-negative is better. `base = -score` (T211 F33).
    #[default]
    Bm25LowerBetter,
    /// Cosine / RRF fused: higher is better. `base = score * RELEVANCE_SCALE`.
    HigherIsBetter,
    /// Bridge Tantivy relevance: higher is better. `base = score` (unscaled; M1).
    BridgeHigherIsBetter,
}

/// Marker-derived pin kind (F4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinKind {
    Constraint,
    Decision,
    Hotspot,
    Other,
}

/// Staleness class for Decision pins only (F5). Non-Decision → [`Unknown`](StalenessClass::Unknown).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StalenessClass {
    Plan,
    Shipped,
    Unknown,
}

/// Strip leading `ASSISTANT: ` once (F4/F42 preflight parity).
pub fn strip_assistant_prefix(content: &str) -> &str {
    content.strip_prefix("ASSISTANT: ").unwrap_or(content)
}

/// Classify pin kind from content markers (case-insensitive, first match wins).
///
/// Leftmost marker among CONSTRAINT / DECISION / HOTSPOT wins. Leading
/// `ASSISTANT: ` is stripped once before scan.
pub fn classify_pin_kind(content: &str) -> PinKind {
    let stripped = strip_assistant_prefix(content);
    let lower = stripped.to_ascii_lowercase();

    let mut best: Option<(usize, PinKind)> = None;
    for (needle, kind) in [
        ("constraint:", PinKind::Constraint),
        ("decision:", PinKind::Decision),
        ("hotspot:", PinKind::Hotspot),
    ] {
        if let Some(pos) = lower.find(needle) {
            match best {
                None => best = Some((pos, kind)),
                Some((best_pos, _)) if pos < best_pos => best = Some((pos, kind)),
                _ => {}
            }
        }
    }
    best.map(|(_, k)| k).unwrap_or(PinKind::Other)
}

/// Classify Decision staleness from ASCII-lower content heuristics (F5).
///
/// Plan markers take precedence when both Plan and Shipped phrases appear
/// (demotion-honest). Non-Decision kinds always return [`Unknown`](StalenessClass::Unknown).
pub fn classify_staleness(content: &str, kind: PinKind) -> StalenessClass {
    if kind != PinKind::Decision {
        return StalenessClass::Unknown;
    }
    let lower = content.to_ascii_lowercase();

    const PLAN_MARKERS: &[&str] = &[
        "plan-only",
        "placeholder",
        "expanded",
        "until go",
        "not dod",
        "planning",
    ];
    if PLAN_MARKERS.iter().any(|m| lower.contains(m)) {
        return StalenessClass::Plan;
    }

    const SHIPPED_MARKERS: &[&str] = &[
        "shipped",
        "complete",
        "closed by",
        "pr #",
        "squash-merged",
        "verified_fixed",
    ];
    if SHIPPED_MARKERS.iter().any(|m| lower.contains(m)) {
        return StalenessClass::Shipped;
    }

    StalenessClass::Unknown
}

/// Manual track-token scan (F6): word-boundary `T`/`t` + ≥1 ASCII digits.
///
/// No `regex` crate — keeps F18 zero-new-crates for retrieval. Tokens are
/// normalized to uppercase `T` + digits, sorted and deduped.
pub fn extract_track_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        let is_t = c == b'T' || c == b't';
        if is_t {
            let boundary_before = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
            if boundary_before {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > i + 1 {
                    // ASCII digits are single-byte UTF-8; slice is valid.
                    let digits = &text[i + 1..j];
                    tokens.push(format!("T{digits}"));
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

fn kind_boost(kind: PinKind) -> f64 {
    match kind {
        PinKind::Constraint => KIND_CONSTRAINT,
        PinKind::Decision => KIND_DECISION,
        PinKind::Hotspot => KIND_HOTSPOT,
        PinKind::Other => KIND_OTHER,
    }
}

fn recency_boost(updated_at: Option<&str>) -> f64 {
    let Some(s) = updated_at else {
        return 0.0;
    };
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) else {
        return 0.0;
    };
    let updated = dt.with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    let age_days = now.signed_duration_since(updated).num_days();
    // Future timestamps → age 0 (full recency boost); clamp to [0, 365].
    let d = (age_days.max(0) as f64).clamp(0.0, RECENCY_MAX_DAYS);
    RECENCY_SCALE * (1.0 - d / RECENCY_MAX_DAYS)
}

/// Composite effective score (F8/F9/F33 + T215 F6 ScoreKind).
///
/// # Score polarity ([`ScoreKind`])
///
/// - [`ScoreKind::Bm25LowerBetter`]: FTS5 `rank` is more-negative = better.
///   `base = -score` so stronger matches rank higher (T211 F33).
/// - [`ScoreKind::HigherIsBetter`]: cosine / RRF. `base = score * RELEVANCE_SCALE`.
/// - [`ScoreKind::BridgeHigherIsBetter`]: Tantivy relevance. `base = score`
///   (no scale, no negate) so large bridge scores stay authority-class (M1).
///
/// Unscored hits (`None`) use base `0.0` (do not leapfrog strong FTS).
pub fn effective_score(
    base: Option<f64>,
    kind: PinKind,
    staleness: StalenessClass,
    sibling_demoted: bool,
    updated_at: Option<&str>,
    score_kind: ScoreKind,
) -> f64 {
    let base_v = match (base, score_kind) {
        (None, _) => 0.0,
        (Some(s), ScoreKind::Bm25LowerBetter) => -s,
        (Some(s), ScoreKind::HigherIsBetter) => s * RELEVANCE_SCALE,
        (Some(s), ScoreKind::BridgeHigherIsBetter) => s,
    };
    let mut v = base_v + kind_boost(kind);
    if kind == PinKind::Decision {
        match staleness {
            StalenessClass::Shipped => v += SHIPPED_BOOST,
            StalenessClass::Plan => {
                v -= PLAN_PENALTY;
                if sibling_demoted {
                    v -= SIBLING_PLAN_PENALTY;
                }
            }
            StalenessClass::Unknown => {}
        }
    }
    v += recency_boost(updated_at);
    v
}

/// Re-rank blended hits in place with pin-type + recency composite (F8).
///
/// Sets [`RecallHit::is_plan_demoted`] for Plan-class Decisions. Sort:
/// effective desc → `updated_at` desc (missing last) → `memory_id` asc.
///
/// **F40:** single post-blend entry point; T215 extends here.
pub fn rerank_hits(hits: &mut Vec<RecallHit>) {
    if hits.is_empty() {
        return;
    }

    let meta: Vec<(PinKind, StalenessClass, Vec<String>)> = hits
        .iter()
        .map(|h| {
            let kind = classify_pin_kind(&h.content);
            let staleness = classify_staleness(&h.content, kind);
            let tokens = if kind == PinKind::Decision {
                extract_track_tokens(&h.content)
            } else {
                Vec::new()
            };
            (kind, staleness, tokens)
        })
        .collect();

    // Track tokens present on any Shipped Decision in this set (F6 sibling).
    let mut shipped_tokens = std::collections::BTreeSet::new();
    for (kind, staleness, tokens) in &meta {
        if *kind == PinKind::Decision && *staleness == StalenessClass::Shipped {
            for t in tokens {
                shipped_tokens.insert(t.clone());
            }
        }
    }

    struct Ranked {
        effective: f64,
        updated_at: Option<String>,
        memory_id: String,
        hit: RecallHit,
    }

    let mut ranked: Vec<Ranked> = hits
        .drain(..)
        .enumerate()
        .map(|(i, mut hit)| {
            let (kind, staleness, ref tokens) = meta[i];
            let is_plan = kind == PinKind::Decision && staleness == StalenessClass::Plan;
            let sibling = is_plan && tokens.iter().any(|t| shipped_tokens.contains(t));
            hit.is_plan_demoted = is_plan;
            let effective = effective_score(
                hit.score,
                kind,
                staleness,
                sibling,
                hit.updated_at.as_deref(),
                hit.score_kind,
            );
            Ranked {
                effective,
                updated_at: hit.updated_at.clone(),
                memory_id: hit.memory_id.clone(),
                hit,
            }
        })
        .collect();

    ranked.sort_by(|a, b| {
        let eff = b
            .effective
            .partial_cmp(&a.effective)
            .unwrap_or(std::cmp::Ordering::Equal);
        if eff != std::cmp::Ordering::Equal {
            return eff;
        }
        match (&a.updated_at, &b.updated_at) {
            (Some(ua), Some(ub)) => {
                let c = ub.cmp(ua);
                if c != std::cmp::Ordering::Equal {
                    return c;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        a.memory_id.cmp(&b.memory_id)
    });

    *hits = ranked.into_iter().map(|r| r.hit).collect();
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // test-only expect/unwrap OK
mod tests {
    use super::*;

    fn hit(id: &str, content: &str, score: Option<f64>, updated_at: Option<&str>) -> RecallHit {
        hit_kind(
            id,
            content,
            score,
            updated_at,
            "fts",
            ScoreKind::Bm25LowerBetter,
        )
    }

    fn hit_kind(
        id: &str,
        content: &str,
        score: Option<f64>,
        updated_at: Option<&str>,
        source: &str,
        score_kind: ScoreKind,
    ) -> RecallHit {
        RecallHit {
            memory_id: id.to_string(),
            content: content.to_string(),
            source: source.to_string(),
            score,
            privacy: None,
            session_id: None,
            updated_at: updated_at.map(str::to_string),
            is_plan_demoted: false,
            score_kind,
            cosine: None,
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_pin_kind__constraint_decision_hotspot_other() {
        assert_eq!(
            classify_pin_kind("CONSTRAINT: must be safe"),
            PinKind::Constraint
        );
        assert_eq!(
            classify_pin_kind("DECISION: do the thing"),
            PinKind::Decision
        );
        assert_eq!(classify_pin_kind("HOTSPOT: src/foo.rs"), PinKind::Hotspot);
        assert_eq!(classify_pin_kind("just a chat turn"), PinKind::Other);
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_pin_kind__strips_assistant_prefix() {
        assert_eq!(
            classify_pin_kind("ASSISTANT: DECISION: plan-only until go"),
            PinKind::Decision
        );
        assert_eq!(
            classify_pin_kind("ASSISTANT: CONSTRAINT: rule"),
            PinKind::Constraint
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_pin_kind__case_insensitive() {
        assert_eq!(
            classify_pin_kind("decision: lowercase marker"),
            PinKind::Decision
        );
        assert_eq!(classify_pin_kind("Constraint: mixed"), PinKind::Constraint);
    }

    #[test]
    #[allow(non_snake_case)]
    fn classify_staleness__plan_and_shipped() {
        let plan = "DECISION: plan-only T999 expanded until go";
        assert_eq!(
            classify_staleness(plan, PinKind::Decision),
            StalenessClass::Plan
        );
        let shipped = "DECISION: shipped T999 PR #1 complete";
        assert_eq!(
            classify_staleness(shipped, PinKind::Decision),
            StalenessClass::Shipped
        );
        assert_eq!(
            classify_staleness("DECISION: we chose X", PinKind::Decision),
            StalenessClass::Unknown
        );
        assert_eq!(
            classify_staleness("CONSTRAINT: plan-only text", PinKind::Constraint),
            StalenessClass::Unknown
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn extract_track_tokens__word_boundary_digits() {
        assert_eq!(
            extract_track_tokens("see T999 and t12 in text"),
            vec!["T12".to_string(), "T999".to_string()]
        );
        // "XT999" — X is alphanumeric so no word boundary before T.
        assert_eq!(
            extract_track_tokens("not a XT999 false"),
            Vec::<String>::new()
        );
        assert_eq!(
            extract_track_tokens("T1,T2"),
            vec!["T1".to_string(), "T2".to_string()]
        );
        assert_eq!(extract_track_tokens("T"), Vec::<String>::new());
    }

    /// AC1: Shipped ranks above Plan for same track + keyword.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__plan_below_shipped_same_track__ac1() {
        let mut hits = vec![
            hit(
                "mem-plan",
                "DECISION: plan-only T999 expanded ranking test keyword_rank_token until go",
                Some(-1.0),
                None,
            ),
            hit(
                "mem-ship",
                "DECISION: shipped T999 keyword_rank_token PR #1 complete",
                Some(-1.0),
                None,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "mem-ship");
        assert_eq!(hits[1].memory_id, "mem-plan");
        assert!(hits[1].is_plan_demoted);
        assert!(!hits[0].is_plan_demoted);
    }

    /// AC1b / AC11: Shipped raw FTS −0.5 outranks Plan raw FTS −3.0 under F9
    /// after BM25-preserving negation (shipped base 0.5+2+1=3.5; plan base 3+2−3=2).
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__shipped_worse_fts_beats_plan_better_fts__ac1b() {
        let shipped_eff = effective_score(
            Some(-0.5),
            PinKind::Decision,
            StalenessClass::Shipped,
            false,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        let plan_eff = effective_score(
            Some(-3.0),
            PinKind::Decision,
            StalenessClass::Plan,
            false,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        assert!(
            shipped_eff > plan_eff,
            "Shipped effective {shipped_eff} must beat Plan {plan_eff}"
        );
        // Document expected magnitudes with BM25 negation.
        assert!(
            (shipped_eff - 3.5).abs() < 1e-9,
            "shipped_eff={shipped_eff}"
        );
        assert!((plan_eff - 2.0).abs() < 1e-9, "plan_eff={plan_eff}");

        let mut hits = vec![
            hit(
                "plan",
                "DECISION: plan-only T1 expanded until go",
                Some(-3.0),
                None,
            ),
            hit(
                "ship",
                "DECISION: shipped T1 PR #1 complete",
                Some(-0.5),
                None,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "ship");
        assert_eq!(hits[1].memory_id, "plan");
    }

    /// R1/R3: same-kind Other — more-negative FTS rank (better BM25) ranks first.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__same_kind_better_bm25_first__f33() {
        let mut hits = vec![
            hit("weak", "plain other keyword", Some(-0.5), None),
            hit("strong", "plain other keyword", Some(-10.0), None),
        ];
        rerank_hits(&mut hits);
        assert_eq!(
            hits[0].memory_id, "strong",
            "better BM25 (more negative rank) must rank first within same kind"
        );
        assert_eq!(hits[1].memory_id, "weak");
    }

    /// AC2: CONSTRAINT outranks plain Other for same base.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__constraint_outranks_other__ac2() {
        let mut hits = vec![
            hit(
                "other",
                "plain chat about keyword_rank_token",
                Some(-1.0),
                None,
            ),
            hit(
                "cons",
                "CONSTRAINT: keyword_rank_token must be safe",
                Some(-1.0),
                None,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "cons");
        assert_eq!(hits[1].memory_id, "other");
    }

    /// AC4: sibling demotion — Plan shares track with Shipped → Plan after Shipped.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__sibling_plan_demotion__ac4() {
        let mut hits = vec![
            hit(
                "plan",
                "DECISION: plan-only T42 expanded until go",
                Some(0.0),
                None,
            ),
            hit(
                "ship",
                "DECISION: shipped T42 complete PR #2",
                Some(0.0),
                None,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "ship");
        assert_eq!(hits[1].memory_id, "plan");
        assert!(hits[1].is_plan_demoted);

        let with_sib = effective_score(
            Some(0.0),
            PinKind::Decision,
            StalenessClass::Plan,
            true,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        let without = effective_score(
            Some(0.0),
            PinKind::Decision,
            StalenessClass::Plan,
            false,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        assert!(with_sib < without);
        assert!((with_sib - (0.0 + 2.0 - 3.0 - 2.0)).abs() < 1e-9);
    }

    /// AC5: equal composite → memory_id ascending.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__equal_scores_memory_id_asc__ac5() {
        let mut hits = vec![
            hit("mem-z", "plain other z", None, None),
            hit("mem-a", "plain other a", None, None),
            hit("mem-m", "plain other m", None, None),
        ];
        rerank_hits(&mut hits);
        let ids: Vec<&str> = hits.iter().map(|h| h.memory_id.as_str()).collect();
        assert_eq!(ids, vec!["mem-a", "mem-m", "mem-z"]);
    }

    #[test]
    #[allow(non_snake_case)]
    fn effective_score__none_base_is_zero() {
        let e = effective_score(
            None,
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        assert!((e - 0.0).abs() < 1e-9);
    }

    #[test]
    #[allow(non_snake_case)]
    fn recency_boost__recent_higher_than_old() {
        let recent = chrono::Utc::now().to_rfc3339();
        let old = (chrono::Utc::now() - chrono::Duration::days(300)).to_rfc3339();
        let r = effective_score(
            Some(0.0),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            Some(&recent),
            ScoreKind::Bm25LowerBetter,
        );
        let o = effective_score(
            Some(0.0),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            Some(&old),
            ScoreKind::Bm25LowerBetter,
        );
        assert!(r > o, "recent {r} should beat old {o}");
    }

    /// AC3: HigherIsBetter is not negated; strong semantic beats weak same-kind.
    #[test]
    #[allow(non_snake_case)]
    fn effective_score__higher_is_better_not_negated__ac3() {
        let strong = effective_score(
            Some(0.9),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        let weak = effective_score(
            Some(0.2),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        assert!(
            strong > weak,
            "strong cosine-scaled {strong} must beat weak {weak}"
        );
        // 0.9 * 500 = 450; 0.2 * 500 = 100
        assert!((strong - 450.0).abs() < 1e-9, "strong={strong}");
        assert!((weak - 100.0).abs() < 1e-9, "weak={weak}");

        let mut hits = vec![
            hit_kind(
                "weak",
                "plain other weak",
                Some(0.2),
                None,
                "semantic",
                ScoreKind::HigherIsBetter,
            ),
            hit_kind(
                "strong",
                "plain other strong",
                Some(0.9),
                None,
                "semantic",
                ScoreKind::HigherIsBetter,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "strong");
        assert_eq!(hits[1].memory_id, "weak");
    }

    /// AC9: Plan demotion still works on hybrid-scored hits.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__plan_demotion_on_hybrid_scores__ac9() {
        // Same RRF-scale base; plan DECISION must rank below shipped.
        let mut hits = vec![
            hit_kind(
                "mem-plan",
                "DECISION: plan-only T999 expanded ranking keyword until go",
                Some(0.02),
                None,
                "hybrid",
                ScoreKind::HigherIsBetter,
            ),
            hit_kind(
                "mem-ship",
                "DECISION: shipped T999 keyword PR #1 complete",
                Some(0.02),
                None,
                "hybrid",
                ScoreKind::HigherIsBetter,
            ),
        ];
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "mem-ship");
        assert_eq!(hits[1].memory_id, "mem-plan");
        assert!(hits[1].is_plan_demoted);
        assert!(!hits[0].is_plan_demoted);
    }

    /// AC15: Within hybrid score space, CONSTRAINT near-tie outranks Other after RELEVANCE_SCALE.
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__constraint_outranks_other_in_hybrid_space__ac15() {
        // Near-tie RRF scores: CONSTRAINT kind boost (+4) must still win.
        let mut hits = vec![
            hit_kind(
                "other",
                "plain chat about keyword_rank_token",
                Some(0.0164),
                None,
                "hybrid",
                ScoreKind::HigherIsBetter,
            ),
            hit_kind(
                "cons",
                "CONSTRAINT: keyword_rank_token must be safe",
                Some(0.0160),
                None,
                "hybrid",
                ScoreKind::HigherIsBetter,
            ),
        ];
        // other base ≈ 8.2; cons base ≈ 8.0 + 4 = 12.0 → cons wins.
        rerank_hits(&mut hits);
        assert_eq!(hits[0].memory_id, "cons");
        assert_eq!(hits[1].memory_id, "other");
    }

    /// AC16: Graph hit inherits parent score_kind (constructor + effective_score path).
    #[test]
    #[allow(non_snake_case)]
    fn graph_hit__inherits_parent_score_kind__ac16() {
        let fts_parent =
            RecallHit::fts("p-fts".into(), "parent fts".into(), Some(-2.0), None, None);
        assert_eq!(fts_parent.score_kind, ScoreKind::Bm25LowerBetter);
        let g_fts = RecallHit::graph(
            "g-fts".into(),
            "graph from fts".into(),
            Some(-1.5),
            None,
            None,
            fts_parent.score_kind,
        );
        assert_eq!(g_fts.score_kind, ScoreKind::Bm25LowerBetter);

        let hybrid_parent = hit_kind(
            "p-hyb",
            "parent hybrid",
            Some(0.03),
            None,
            "hybrid",
            ScoreKind::HigherIsBetter,
        );
        let g_hyb = RecallHit::graph(
            "g-hyb".into(),
            "graph from hybrid".into(),
            Some(0.04),
            None,
            None,
            hybrid_parent.score_kind,
        );
        assert_eq!(g_hyb.score_kind, ScoreKind::HigherIsBetter);

        // Effective path respects inherited kind (not negated for HigherIsBetter).
        let eff_hyb = effective_score(
            g_hyb.score,
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            g_hyb.score_kind,
        );
        assert!(
            (eff_hyb - 0.04 * RELEVANCE_SCALE).abs() < 1e-9,
            "eff_hyb={eff_hyb}"
        );
        let eff_fts = effective_score(
            g_fts.score,
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            g_fts.score_kind,
        );
        assert!((eff_fts - 1.5).abs() < 1e-9, "eff_fts={eff_fts}");
    }

    /// AC17: Bridge positive relevance outranks weak FTS after rerank (M1 polarity).
    #[test]
    #[allow(non_snake_case)]
    fn rerank_hits__bridge_positive_outranks_weak_fts__ac17() {
        let mut hits = vec![
            hit_kind(
                "fts-weak",
                "plain other weak fts",
                Some(-0.5),
                None,
                "fts",
                ScoreKind::Bm25LowerBetter,
            ),
            hit_kind(
                "bridge-strong",
                "plain other bridge authority",
                Some(18.3),
                None,
                "bridge",
                ScoreKind::BridgeHigherIsBetter,
            ),
        ];
        // Bridge base 18.3 >> FTS base 0.5 after polarity.
        rerank_hits(&mut hits);
        assert_eq!(
            hits[0].memory_id, "bridge-strong",
            "bridge must not be demoted by BM25 negation"
        );
        assert_eq!(hits[1].memory_id, "fts-weak");

        // If bridge were wrongly Bm25LowerBetter, base = -18.3 would lose to 0.5.
        let wrong = effective_score(
            Some(18.3),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::Bm25LowerBetter,
        );
        let right = effective_score(
            Some(18.3),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::BridgeHigherIsBetter,
        );
        assert!(right > 0.0 && wrong < 0.0);
        assert!(right > wrong);
    }
}
