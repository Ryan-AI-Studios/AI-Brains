//! Hybrid FTS + dense fusion via Reciprocal Rank Fusion (T215).
//!
//! Pure rank-only fusion — never linear-combines raw BM25 with cosine (F3).

use crate::ranking::ScoreKind;
use crate::recall::RecallHit;
use std::collections::{BTreeMap, BTreeSet};

/// Default RRF constant k (Cormack SIGIR 2009; industry default). Env: `AI_BRAINS_RRF_K`.
pub const RRF_K: f64 = 60.0;

/// Default cosine similarity floor for semantic candidates. Env: `AI_BRAINS_SEMANTIC_MIN_SCORE`.
pub const SEMANTIC_MIN_COSINE: f64 = 0.55;

/// Candidate depth before RRF: `max(limit * 3, 15).min(50)` (F9 / M3).
pub fn candidate_depth(limit: usize) -> usize {
    limit.saturating_mul(3).clamp(15, 50)
}

/// Resolve RRF k from env `AI_BRAINS_RRF_K` (invalid/missing → [`RRF_K`]).
pub fn rrf_k() -> f64 {
    parse_positive_f64_env("AI_BRAINS_RRF_K").unwrap_or(RRF_K)
}

/// Resolve semantic cosine floor from env `AI_BRAINS_SEMANTIC_MIN_SCORE`
/// (invalid/missing → [`SEMANTIC_MIN_COSINE`]).
pub fn semantic_min_cosine() -> f64 {
    parse_f64_env("AI_BRAINS_SEMANTIC_MIN_SCORE").unwrap_or(SEMANTIC_MIN_COSINE)
}

/// Effective cosine floor: optional one-shot override, else env/default (F4 / soft F32).
pub fn effective_semantic_min_cosine(override_min: Option<f64>) -> f64 {
    override_min.unwrap_or_else(semantic_min_cosine)
}

/// Drop hits whose cosine `score` is strictly below `floor` (AC2).
///
/// Hits with missing score are dropped. Does not reorder survivors.
pub fn filter_by_cosine_floor(hits: Vec<RecallHit>, floor: f64) -> Vec<RecallHit> {
    hits.into_iter()
        .filter(|h| h.score.is_some_and(|s| s >= floor))
        .collect()
}

fn parse_f64_env(key: &str) -> Option<f64> {
    let raw = std::env::var(key).ok()?;
    let v: f64 = raw.trim().parse().ok()?;
    if v.is_finite() { Some(v) } else { None }
}

fn parse_positive_f64_env(key: &str) -> Option<f64> {
    let v = parse_f64_env(key)?;
    if v > 0.0 { Some(v) } else { None }
}

/// Pure RRF: fuse two ranked lists of [`RecallHit`] by `memory_id` (F5 / F7 / F38).
///
/// - Rank from 1 based on current list order (lists already sorted best-first).
/// - Missing from a list → **no summand** (not `len+1`).
/// - `score = Σ 1/(k+rank)`; `score_kind = HigherIsBetter`.
/// - Source: both → `"hybrid"`; fts-only → `"fts"`; semantic-only → `"semantic"`.
/// - Content / privacy / session / `updated_at`: prefer FTS hit when both present.
/// - Output sorted by RRF score desc, then `memory_id` asc.
pub fn rrf_fuse(fts_hits: &[RecallHit], semantic_hits: &[RecallHit], k: f64) -> Vec<RecallHit> {
    let k = if k > 0.0 && k.is_finite() { k } else { RRF_K };

    let fts_rank = rank_map(fts_hits);
    let sem_rank = rank_map(semantic_hits);
    let fts_by_id = index_by_id(fts_hits);
    let sem_by_id = index_by_id(semantic_hits);

    let mut ids: BTreeSet<String> = BTreeSet::new();
    ids.extend(fts_rank.keys().cloned());
    ids.extend(sem_rank.keys().cloned());

    let mut fused: Vec<RecallHit> = Vec::with_capacity(ids.len());
    for id in ids {
        let mut score = 0.0_f64;
        let in_fts = if let Some(&r) = fts_rank.get(&id) {
            score += 1.0 / (k + r as f64);
            true
        } else {
            false
        };
        let in_sem = if let Some(&r) = sem_rank.get(&id) {
            score += 1.0 / (k + r as f64);
            true
        } else {
            false
        };

        let source = match (in_fts, in_sem) {
            (true, true) => "hybrid",
            (true, false) => "fts",
            (false, true) => "semantic",
            (false, false) => continue, // unreachable for ids union
        };

        // Prefer FTS content/metadata when both present (F7).
        let base = if in_fts {
            fts_by_id.get(&id)
        } else {
            sem_by_id.get(&id)
        };
        let Some(base) = base else {
            continue;
        };

        fused.push(RecallHit {
            memory_id: id,
            content: base.content.clone(),
            source: source.to_string(),
            score: Some(score),
            privacy: base.privacy,
            session_id: base.session_id.clone(),
            updated_at: base.updated_at.clone(),
            is_plan_demoted: false,
            score_kind: ScoreKind::HigherIsBetter,
        });
    }

    fused.sort_by(|a, b| {
        let sa = a.score.unwrap_or(0.0);
        let sb = b.score.unwrap_or(0.0);
        let cmp = sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal);
        if cmp != std::cmp::Ordering::Equal {
            return cmp;
        }
        a.memory_id.cmp(&b.memory_id)
    });

    fused
}

/// 1-based ranks from list order; first occurrence wins on duplicate ids.
fn rank_map(hits: &[RecallHit]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for (i, h) in hits.iter().enumerate() {
        m.entry(h.memory_id.clone()).or_insert(i + 1);
    }
    m
}

fn index_by_id(hits: &[RecallHit]) -> BTreeMap<String, &RecallHit> {
    let mut m = BTreeMap::new();
    for h in hits {
        m.entry(h.memory_id.clone()).or_insert(h);
    }
    m
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // test-only expect/unwrap OK
mod tests {
    use super::*;
    use crate::ranking::{PinKind, RELEVANCE_SCALE, StalenessClass, effective_score, rerank_hits};
    use ai_brains_core::temp_env::TempEnv;

    fn fts_hit(id: &str, content: &str, score: Option<f64>) -> RecallHit {
        RecallHit::fts(id.to_string(), content.to_string(), score, None, None)
    }

    fn sem_hit(id: &str, content: &str, sim: f64) -> RecallHit {
        RecallHit::semantic(
            id.to_string(),
            content.to_string(),
            Some(sim),
            None,
            None,
            None,
        )
    }

    /// AC1: id in both lists ranks above id in one list only (same k).
    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__id_in_both_lists_ranks_above_single_list__ac1() {
        let fts = vec![
            fts_hit("both", "fts-both", Some(-1.0)),
            fts_hit("fts-only", "fts-only-content", Some(-2.0)),
        ];
        let sem = vec![
            sem_hit("sem-only", "sem-only-content", 0.9),
            sem_hit("both", "sem-both", 0.8),
        ];
        let fused = rrf_fuse(&fts, &sem, 60.0);
        assert_eq!(fused.len(), 3);
        // both: 1/(60+1) + 1/(60+2) > either single 1/(60+1) or 1/(60+2)
        assert_eq!(fused[0].memory_id, "both");
        assert_eq!(fused[0].source, "hybrid");
        // fts-only rank1 alone = 1/61; sem-only rank1 alone = 1/61 → memory_id asc: fts-only, sem-only
        let ids: Vec<&str> = fused.iter().map(|h| h.memory_id.as_str()).collect();
        assert_eq!(ids[0], "both");
        assert!(ids.contains(&"fts-only"));
        assert!(ids.contains(&"sem-only"));
        let both_score = fused[0].score.expect("score");
        let single_max = fused[1..]
            .iter()
            .map(|h| h.score.unwrap_or(0.0))
            .fold(0.0_f64, f64::max);
        assert!(
            both_score > single_max,
            "both {both_score} must beat single max {single_max}"
        );
    }

    /// AC2: cosine below floor excluded.
    #[test]
    #[allow(non_snake_case)]
    fn filter_by_cosine_floor__drops_below_threshold__ac2() {
        let hits = vec![
            sem_hit("high", "above", 0.80),
            sem_hit("border", "equal", 0.55),
            sem_hit("low", "below", 0.54),
            sem_hit("noise", "weak", 0.30),
        ];
        let kept = filter_by_cosine_floor(hits, 0.55);
        let ids: Vec<&str> = kept.iter().map(|h| h.memory_id.as_str()).collect();
        assert_eq!(ids, vec!["high", "border"]);
    }

    /// AC5: id in FTS+semantic → source "hybrid" + FTS content preferred.
    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__hybrid_prefers_fts_content__ac5() {
        let fts = vec![fts_hit("m1", "FROM_FTS", Some(-1.0))];
        let sem = vec![sem_hit("m1", "FROM_SEM", 0.9)];
        let fused = rrf_fuse(&fts, &sem, 60.0);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].source, "hybrid");
        assert_eq!(fused[0].content, "FROM_FTS");
        assert_eq!(fused[0].score_kind, ScoreKind::HigherIsBetter);
    }

    /// AC6: all semantic below floor → empty semantic list; fuse is FTS-only (no semantic sources).
    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__all_below_floor_no_semantic_noise__ac6() {
        let fts = vec![
            fts_hit("a", "lexical a", Some(-1.0)),
            fts_hit("b", "lexical b", Some(-2.0)),
        ];
        let weak_sem = vec![
            sem_hit("noise1", "off topic", 0.40),
            sem_hit("noise2", "also weak", 0.44),
        ];
        let filtered = filter_by_cosine_floor(weak_sem, SEMANTIC_MIN_COSINE);
        assert!(filtered.is_empty());
        let fused = rrf_fuse(&fts, &filtered, RRF_K);
        assert_eq!(fused.len(), 2);
        assert!(fused.iter().all(|h| h.source == "fts"));
        assert!(
            fused
                .iter()
                .all(|h| h.score_kind == ScoreKind::HigherIsBetter)
        );
    }

    /// AC14: FTS empty + semantic post-floor → semantic-only; pin re-rank applies.
    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__fts_empty_semantic_only_then_pin_rerank__ac14() {
        let fts: Vec<RecallHit> = Vec::new();
        let sem = vec![
            sem_hit("other", "plain other about topic", 0.80),
            sem_hit("cons", "CONSTRAINT: topic must be enforced", 0.70),
        ];
        let filtered = filter_by_cosine_floor(sem, 0.55);
        assert_eq!(filtered.len(), 2);
        let mut fused = rrf_fuse(&fts, &filtered, RRF_K);
        assert!(fused.iter().all(|h| h.source == "semantic"));
        // Before pin re-rank: rank order is native semantic (other first).
        assert_eq!(fused[0].memory_id, "other");
        rerank_hits(&mut fused);
        // CONSTRAINT kind boost must lift cons above other in hybrid space.
        assert_eq!(fused[0].memory_id, "cons");
        assert_eq!(fused[1].memory_id, "other");
    }

    #[test]
    #[allow(non_snake_case)]
    fn candidate_depth__clamps_to_bounds() {
        assert_eq!(candidate_depth(1), 15); // max(3,15)=15
        assert_eq!(candidate_depth(5), 15); // max(15,15)=15
        assert_eq!(candidate_depth(10), 30); // max(30,15)=30
        assert_eq!(candidate_depth(20), 50); // max(60,15).min(50)=50
        assert_eq!(candidate_depth(100), 50);
    }

    #[test]
    #[allow(non_snake_case)]
    fn semantic_min_cosine__env_override() {
        let _g = TempEnv::set("AI_BRAINS_SEMANTIC_MIN_SCORE", "0.42");
        assert!((semantic_min_cosine() - 0.42).abs() < 1e-12);
    }

    #[test]
    #[allow(non_snake_case)]
    fn semantic_min_cosine__invalid_env__default() {
        let _g = TempEnv::set("AI_BRAINS_SEMANTIC_MIN_SCORE", "not-a-float");
        assert!((semantic_min_cosine() - SEMANTIC_MIN_COSINE).abs() < 1e-12);
    }

    #[test]
    #[allow(non_snake_case)]
    fn rrf_k__env_override() {
        let _g = TempEnv::set("AI_BRAINS_RRF_K", "40");
        assert!((rrf_k() - 40.0).abs() < 1e-12);
    }

    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__missing_list_omits_term_not_len_plus_one() {
        // F38: single-list RRF = 1/(k+rank) only — no phantom second term.
        let fts = vec![fts_hit("solo", "only fts", Some(-1.0))];
        let fused = rrf_fuse(&fts, &[], 60.0);
        assert_eq!(fused.len(), 1);
        let expected = 1.0 / (60.0 + 1.0);
        assert!((fused[0].score.unwrap() - expected).abs() < 1e-12);
    }

    #[test]
    #[allow(non_snake_case)]
    fn higher_is_better_rrf_score_scales_with_relevance_scale() {
        let fts = vec![fts_hit("a", "a", Some(-1.0))];
        let fused = rrf_fuse(&fts, &[], 60.0);
        let s = fused[0].score.unwrap();
        let eff = effective_score(
            Some(s),
            PinKind::Other,
            StalenessClass::Unknown,
            false,
            None,
            ScoreKind::HigherIsBetter,
        );
        assert!((eff - s * RELEVANCE_SCALE).abs() < 1e-9);
    }
}
