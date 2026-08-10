//! Hybrid FTS + dense fusion via Reciprocal Rank Fusion (T215 / T218).
//!
//! Pure rank-only fusion — never linear-combines raw BM25 with cosine (F3).

use crate::ranking::ScoreKind;
use crate::recall::RecallHit;
use std::collections::{BTreeMap, BTreeSet};

/// Default RRF constant k (Cormack SIGIR 2009; industry default). Env: `AI_BRAINS_RRF_K`.
pub const RRF_K: f64 = 60.0;

/// Default cosine similarity floor for the hybrid semantic arm. Env: `AI_BRAINS_SEMANTIC_MIN_SCORE`.
pub const SEMANTIC_MIN_COSINE: f64 = 0.55;

/// Stricter cosine floor when **no FTS arm** (semantic-only / substring-only gate).
/// Env: `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE` (T218 F2).
pub const SEMANTIC_ONLY_MIN_COSINE: f64 = 0.60;

/// Candidate depth before RRF: `max(limit * 3, 15).min(50)` (F9 / M3).
pub fn candidate_depth(limit: usize) -> usize {
    limit.saturating_mul(3).clamp(15, 50)
}

/// Resolve RRF k from env `AI_BRAINS_RRF_K` (invalid/missing → [`RRF_K`]).
pub fn rrf_k() -> f64 {
    parse_positive_f64_env("AI_BRAINS_RRF_K").unwrap_or(RRF_K)
}

/// Resolve hybrid-arm cosine floor from env `AI_BRAINS_SEMANTIC_MIN_SCORE`
/// (invalid/missing → [`SEMANTIC_MIN_COSINE`]).
pub fn semantic_min_cosine() -> f64 {
    parse_f64_env("AI_BRAINS_SEMANTIC_MIN_SCORE").unwrap_or(SEMANTIC_MIN_COSINE)
}

/// Resolve semantic-only cosine floor from env `AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`
/// (invalid/missing → [`SEMANTIC_ONLY_MIN_COSINE`]; T218 F2).
pub fn semantic_only_min_cosine() -> f64 {
    parse_f64_env("AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE").unwrap_or(SEMANTIC_ONLY_MIN_COSINE)
}

/// Effective hybrid-arm cosine floor: optional one-shot override **replaces**
/// env/default (T215 F4 / T218 F2b — not `max()`).
pub fn effective_semantic_min_cosine(override_min: Option<f64>) -> f64 {
    override_min.unwrap_or_else(semantic_min_cosine)
}

/// Effective semantic-only floor: optional one-shot override **replaces**
/// env/default (T218 F2b / F39 — not `max()`).
pub fn effective_semantic_only_min_cosine(override_min: Option<f64>) -> f64 {
    override_min.unwrap_or_else(semantic_only_min_cosine)
}

/// True when the pre-RRF local arm has any hit with `source == "fts"` (T218 F37).
///
/// Substring-only and empty local → `false` (strict floor applies). True FTS
/// (including T217 rescue ranks) → `true` (hybrid arm floor only).
pub fn has_fts_arm(hits: &[RecallHit]) -> bool {
    hits.iter().any(|h| h.source == "fts")
}

/// Drop hits whose cosine is strictly below `floor`.
///
/// Prefers [`RecallHit::cosine`], else falls back to `score` (pre-fuse semantic
/// path where score still holds cosine). Hits with neither are dropped.
/// Does not reorder survivors.
pub fn filter_by_cosine_floor(hits: Vec<RecallHit>, floor: f64) -> Vec<RecallHit> {
    hits.into_iter()
        .filter(|h| h.cosine.or(h.score).is_some_and(|s| s >= floor))
        .collect()
}

/// Apply dual floor for pure-unit / blend helpers (T218 AC1–AC3 / AC18).
///
/// When `has_fts_arm(local_hits)`, returns `semantic_hits` unchanged (already
/// past the hybrid-arm 0.55 floor). When no FTS arm, filters with
/// [`effective_semantic_only_min_cosine`].
pub fn apply_dual_semantic_floor(
    local_hits: &[RecallHit],
    semantic_hits: Vec<RecallHit>,
    min_score_override: Option<f64>,
) -> Vec<RecallHit> {
    if has_fts_arm(local_hits) {
        return semantic_hits;
    }
    let floor = effective_semantic_only_min_cosine(min_score_override);
    filter_by_cosine_floor(semantic_hits, floor)
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
/// - **Cosine** (T218 F4): when the semantic arm has the id, preserve its
///   pre-fuse `cosine` (even when content prefers FTS).
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

        // T218 F4: preserve pre-fuse cosine from the semantic arm when present.
        let cosine = sem_by_id
            .get(&id)
            .and_then(|h| h.cosine.or(h.score))
            .or(base.cosine);

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
            cosine,
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

    // -----------------------------------------------------------------------
    // T218 dual floor + gate + cosine preserve
    // -----------------------------------------------------------------------

    /// AC1: no FTS + cosines in [0.55, 0.60) → no semantic after dual floor.
    #[test]
    #[allow(non_snake_case)]
    fn dual_floor__no_fts_arm__weak_cosine_dropped__ac1() {
        let local: Vec<RecallHit> = Vec::new();
        let sem = vec![
            sem_hit("weak-a", "noise a", 0.55),
            sem_hit("weak-b", "noise b", 0.59),
        ];
        // Precondition: both pass hybrid-arm 0.55.
        let post_hybrid = filter_by_cosine_floor(sem, SEMANTIC_MIN_COSINE);
        assert_eq!(post_hybrid.len(), 2);
        let after = apply_dual_semantic_floor(&local, post_hybrid, None);
        assert!(
            after.is_empty(),
            "semantic-only floor 0.60 must drop [0.55,0.60); got {:?}",
            after
                .iter()
                .map(|h| h.memory_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    /// AC2: no FTS + cosine ≥ 0.60 → retained.
    #[test]
    #[allow(non_snake_case)]
    fn dual_floor__no_fts_arm__strong_cosine_retained__ac2() {
        let local: Vec<RecallHit> = Vec::new();
        let sem = vec![
            sem_hit("strong", "on topic", 0.72),
            sem_hit("border", "border", SEMANTIC_ONLY_MIN_COSINE),
            sem_hit("weak", "noise", 0.59),
        ];
        let after = apply_dual_semantic_floor(&local, sem, None);
        let ids: Vec<&str> = after.iter().map(|h| h.memory_id.as_str()).collect();
        assert_eq!(ids, vec!["strong", "border"]);
    }

    /// AC3: FTS present + cosine in [0.55, 0.60) still eligible for RRF.
    #[test]
    #[allow(non_snake_case)]
    fn dual_floor__fts_arm_present__weak_still_eligible__ac3() {
        let local = vec![fts_hit("lex", "lexical hit", Some(-2.0))];
        let weak = sem_hit("weak-sem", "weak neighbor", 0.57);
        let sem = vec![weak.clone()];
        let after = apply_dual_semantic_floor(&local, sem, None);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].memory_id, "weak-sem");
        // And it participates in RRF.
        let fused = rrf_fuse(&local, &after, RRF_K);
        assert!(fused.iter().any(|h| h.memory_id == "weak-sem"));
        assert!(fused.iter().any(|h| h.memory_id == "lex"));
    }

    /// AC16: --min-score override replaces both gate defaults (not max).
    #[test]
    #[allow(non_snake_case)]
    fn dual_floor__min_score_override_replaces_both_gates__ac16() {
        // Override 0.57: re-admits [0.55,0.60) residual that default dual would drop.
        assert!((effective_semantic_min_cosine(Some(0.57)) - 0.57).abs() < 1e-12);
        assert!((effective_semantic_only_min_cosine(Some(0.57)) - 0.57).abs() < 1e-12);
        // Not max(0.57, 0.60)=0.60.
        assert!((effective_semantic_only_min_cosine(Some(0.57)) - 0.60).abs() > 1e-9);

        let local: Vec<RecallHit> = Vec::new();
        let residual = vec![sem_hit("residual", "almost", 0.575)];
        let kept = apply_dual_semantic_floor(&local, residual.clone(), Some(0.57));
        assert_eq!(kept.len(), 1);

        // Override 0.58: drops 0.575.
        let dropped = apply_dual_semantic_floor(&local, residual, Some(0.58));
        assert!(dropped.is_empty());

        // Hybrid-arm replace: 0.50 override admits what default 0.55 would drop.
        assert!((effective_semantic_min_cosine(Some(0.50)) - 0.50).abs() < 1e-12);
    }

    /// AC18: substring-only local arm applies SEMANTIC_ONLY_MIN; weak dropped;
    /// substring still merges outside RRF.
    #[test]
    #[allow(non_snake_case)]
    fn dual_floor__substring_only_applies_strict_floor_and_merges__ac18() {
        let local = vec![RecallHit::substring(
            "sub-1".into(),
            "substring match content".into(),
            None,
            None,
        )];
        assert!(
            !has_fts_arm(&local),
            "substring-only must not count as FTS arm"
        );

        let weak = vec![sem_hit("noise", "off topic", 0.56)];
        let after_floor = apply_dual_semantic_floor(&local, weak, None);
        assert!(
            after_floor.is_empty(),
            "weak semantic must drop under semantic-only floor"
        );

        // F41: RRF FTS list empty; fuse semantic-empty → empty; substring merges after.
        let fts_only: Vec<RecallHit> = local
            .iter()
            .filter(|h| h.source == "fts")
            .cloned()
            .collect();
        assert!(fts_only.is_empty());
        let fused = rrf_fuse(&fts_only, &after_floor, RRF_K);
        assert!(fused.is_empty());

        let substring_rest: Vec<RecallHit> = local
            .iter()
            .filter(|h| h.source != "fts")
            .cloned()
            .collect();
        let mut seen = std::collections::HashSet::new();
        let mut blended = Vec::new();
        for hit in fused {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        for hit in substring_rest {
            if seen.insert(hit.memory_id.clone()) {
                blended.push(hit);
            }
        }
        assert_eq!(blended.len(), 1);
        assert_eq!(blended[0].source, "substring");
        assert_eq!(blended[0].memory_id, "sub-1");
    }

    /// T218 F4: rrf_fuse preserves cosine from semantic arm (even when content prefers FTS).
    #[test]
    #[allow(non_snake_case)]
    fn rrf_fuse__preserves_cosine_from_semantic_arm__f4() {
        let fts = vec![fts_hit("m1", "FROM_FTS", Some(-1.0))];
        let sem = sem_hit("m1", "FROM_SEM", 0.81);
        assert_eq!(sem.cosine, Some(0.81));
        // score will be overwritten by RRF; cosine must survive.
        let fused = rrf_fuse(&fts, &[sem], 60.0);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].content, "FROM_FTS");
        assert_eq!(fused[0].source, "hybrid");
        assert_eq!(fused[0].cosine, Some(0.81));
        // score is RRF rank contribution, not cosine.
        let expected_rrf = 1.0 / 61.0 + 1.0 / 61.0;
        assert!((fused[0].score.unwrap() - expected_rrf).abs() < 1e-12);
        assert!(fused[0].score.unwrap() < 0.1);
    }

    #[test]
    #[allow(non_snake_case)]
    fn has_fts_arm__fts_true_substring_false_empty_false() {
        assert!(!has_fts_arm(&[]));
        assert!(!has_fts_arm(&[RecallHit::substring(
            "s".into(),
            "c".into(),
            None,
            None
        )]));
        assert!(has_fts_arm(&[fts_hit("f", "c", Some(-1.0))]));
    }

    #[test]
    #[allow(non_snake_case)]
    fn semantic_only_min_cosine__env_override() {
        let _g = TempEnv::set("AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE", "0.63");
        assert!((semantic_only_min_cosine() - 0.63).abs() < 1e-12);
    }

    #[test]
    #[allow(non_snake_case)]
    fn semantic_only_min_cosine__invalid_env__default() {
        let _g = TempEnv::set("AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE", "nope");
        assert!((semantic_only_min_cosine() - SEMANTIC_ONLY_MIN_COSINE).abs() < 1e-12);
    }
}
