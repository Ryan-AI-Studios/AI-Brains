//! T276 — merge preferred-project lexical hits ahead of unscoped `--global` hits.
//!
//! Prefer-fill is **not** a leftover exclude (T264 F11). AC3 leftover-in-candidates
//! is **this** merge output (pre-`rerank_hits`). A leftover-free post-rerank top-5
//! is not a drop regression (F41).

use crate::recall::RecallHit;
use std::collections::HashSet;

/// Preferred hits first, then unscoped global, `memory_id` once (preferred wins).
///
/// When `preferred.len() >= depth`, truncate preferred to `depth` and **do not**
/// scan global (F39). That absence is the depth cap, not a leftover SQL filter.
pub fn merge_preferred_then_global(
    preferred: Vec<RecallHit>,
    global: Vec<RecallHit>,
    depth: usize,
) -> Vec<RecallHit> {
    if depth == 0 {
        return Vec::new();
    }

    let skip_global = preferred.len() >= depth;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(depth);

    for hit in preferred {
        if seen.insert(hit.memory_id.clone()) {
            out.push(hit);
            if out.len() >= depth {
                return out;
            }
        }
    }

    if skip_global {
        return out;
    }

    for hit in global {
        if seen.insert(hit.memory_id.clone()) {
            out.push(hit);
            if out.len() >= depth {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::recall::RecallHit;

    fn hit(id: &str) -> RecallHit {
        RecallHit::fts(
            id.to_string(),
            format!("content {id}"),
            Some(-1.0),
            None,
            None,
        )
    }

    fn ids(hits: &[RecallHit]) -> Vec<&str> {
        hits.iter().map(|h| h.memory_id.as_str()).collect()
    }

    #[test]
    fn merge_preferred_then_global__preferred_first_no_dupes() {
        let preferred: Vec<RecallHit> = (0..3).map(|i| hit(&format!("p{i}"))).collect();
        let mut global: Vec<RecallHit> = (0..15).map(|i| hit(&format!("g{i}"))).collect();
        global.push(hit("p1"));
        let merged = merge_preferred_then_global(preferred, global, 15);
        assert!(
            merged.len() <= 15,
            "AC1: len must be ≤ depth; got {}",
            merged.len()
        );
        assert_eq!(ids(&merged)[..3], ["p0", "p1", "p2"]);
        assert_eq!(
            merged.iter().filter(|h| h.memory_id == "p1").count(),
            1,
            "AC1: overlapping preferred id once"
        );
    }

    #[test]
    fn merge_preferred_then_global__overlap_id__once() {
        let preferred = vec![hit("shared"), hit("p-only")];
        let global = vec![hit("shared"), hit("g-only")];
        let merged = merge_preferred_then_global(preferred, global, 15);
        assert_eq!(ids(&merged), ["shared", "p-only", "g-only"]);
    }

    #[test]
    fn merge_preferred_then_global__preferred_fills_depth__skips_global() {
        let preferred: Vec<RecallHit> = (0..15).map(|i| hit(&format!("p{i}"))).collect();
        let global: Vec<RecallHit> = (0..5).map(|i| hit(&format!("g{i}"))).collect();
        let merged = merge_preferred_then_global(preferred, global, 15);
        assert_eq!(merged.len(), 15);
        assert!(
            merged.iter().all(|h| h.memory_id.starts_with('p')),
            "F39: preferred-full must skip global; ids={:?}",
            ids(&merged)
        );
    }

    #[test]
    fn merge_preferred_then_global__preferred_none__identity() {
        let global: Vec<RecallHit> = (0..4).map(|i| hit(&format!("g{i}"))).collect();
        let merged = merge_preferred_then_global(Vec::new(), global.clone(), 15);
        assert_eq!(ids(&merged), ids(&global));
    }
}
