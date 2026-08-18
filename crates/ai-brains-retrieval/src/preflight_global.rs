//! T264 — `--global` preflight isolation: round-robin caps + project tags.
//!
//! Pure helpers only. Callers in `preflight.rs` apply these after recency
//! ORDER BY. Project-scoped preflight must not call them.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::str::FromStr;

use ai_brains_core::ids::ProjectId;

/// First-seen project order, then the i-th item from each bucket (`i = 0..per_project`),
/// stopping at `max_total`. Recency inside a bucket is the input order.
pub fn take_round_robin<T, K, F>(
    items: impl IntoIterator<Item = T>,
    key_of: F,
    per_project: usize,
    max_total: usize,
) -> Vec<T>
where
    K: Eq + Hash + Clone,
    F: Fn(&T) -> K,
{
    if per_project == 0 || max_total == 0 {
        return Vec::new();
    }
    let mut order: Vec<K> = Vec::new();
    let mut buckets: HashMap<K, VecDeque<T>> = HashMap::new();
    for item in items {
        let key = key_of(&item);
        if !buckets.contains_key(&key) {
            order.push(key.clone());
        }
        buckets.entry(key).or_default().push_back(item);
    }
    let mut out = Vec::new();
    for _round in 0..per_project {
        for key in &order {
            if out.len() >= max_total {
                return out;
            }
            if let Some(bucket) = buckets.get_mut(key)
                && let Some(item) = bucket.pop_front()
            {
                out.push(item);
            }
        }
    }
    out
}

/// Stable tag token: `[` + 8 lowercase hex + `]` or literal `[unknown]`.
/// Display-only. Caps and span use [`project_key`] (full UUID).
pub fn project_tag(project_id: Option<&str>) -> String {
    match project_key(project_id).as_str() {
        "unknown" => "[unknown]".to_string(),
        id => format!("[{}]", &id[..8]),
    }
}

/// Bucket key for round-robin + span: canonical full UUID, or `"unknown"`.
pub fn project_key(project_id: Option<&str>) -> String {
    match project_id {
        Some(raw) => match ProjectId::from_str(raw.trim()) {
            Ok(id) => id.to_string().to_ascii_lowercase(),
            Err(_) => "unknown".to_string(),
        },
        None => "unknown".to_string(),
    }
}

/// Prefix the first line only (F2 / F30). Continuation lines are untouched.
pub fn prefix_first_line(text: &str, project_id: Option<&str>) -> String {
    let tag = project_tag(project_id);
    match text.split_once('\n') {
        Some((first, rest)) => format!("{tag} {first}\n{rest}"),
        None => format!("{tag} {text}"),
    }
}

/// Distinct non-`unknown` project keys that contributed at least one emitted item.
pub fn span_count<I, S>(ids: I) -> u32
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = HashSet::new();
    for id in ids {
        let key = project_key(Some(id.as_ref()));
        if key != "unknown" {
            seen.insert(key);
        }
    }
    seen.len() as u32
}

pub const GLOBAL_SAFETY_PER_PROJECT: usize = 2;
pub const GLOBAL_SAFETY_MAX: usize = 8;
pub const GLOBAL_SAFETY_FETCH: usize = 40;
pub const GLOBAL_INDEX_PER_PROJECT: usize = 3;
pub const GLOBAL_INDEX_MAX: usize = 15;
pub const GLOBAL_INDEX_FETCH: usize = 80;
pub const GLOBAL_RECENT_PER_PROJECT: usize = 1;
pub const GLOBAL_RECENT_MAX: usize = 3;
pub const GLOBAL_SESSION_PER_PROJECT: usize = 1;
pub const GLOBAL_SESSION_MAX: usize = 40;

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn take_round_robin__leftover_then_other__interleaves_per_project() {
        // AC1: 5 leftover + 5 other, leftover-first recency; per_project=2, max=8.
        let leftover = "7d97a456-f2f4-43ea-1f11-aaaaaaaaaaaa";
        let other = "3581317d-601e-44f7-ab84-fde90aa12d3c";
        let mut items = Vec::new();
        for i in 0..5 {
            items.push((leftover, format!("L{i}")));
        }
        for i in 0..5 {
            items.push((other, format!("O{i}")));
        }
        let out = take_round_robin(items, |it| it.0, 2, 8);
        let labels: Vec<&str> = out.iter().map(|it| it.1.as_str()).collect();
        assert_eq!(
            labels,
            ["L0", "O0", "L1", "O1"],
            "interleave by round; 2 leftover + 2 other; never 5 leftover; got {labels:?}"
        );
        let leftover_n = out.iter().filter(|it| it.0 == leftover).count();
        assert_eq!(leftover_n, 2);
        let other_n = out.iter().filter(|it| it.0 == other).count();
        assert_eq!(other_n, 2);
    }

    #[test]
    fn take_round_robin__empty_and_unknown__respects_max() {
        // AC2
        let empty: Vec<(&str, u8)> = Vec::new();
        let out = take_round_robin(empty, |it| it.0, 2, 8);
        assert!(out.is_empty(), "empty input → empty; got {out:?}");

        let unknown: Vec<(Option<&str>, u8)> = (0..10).map(|i| (None, i)).collect();
        let out = take_round_robin(unknown, |it| project_key(it.0), 5, 3);
        assert_eq!(
            out.len(),
            3,
            "all-unknown still respects max_total; got {out:?}"
        );

        let a = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let b = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let c = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let mixed = vec![(a, 0), (a, 1), (b, 2), (b, 3), (c, 4), (c, 5)];
        let out = take_round_robin(mixed, |it| it.0, 1, 8);
        assert_eq!(out.len(), 3, "per_project=1 emits at most one per id");
        let keys: Vec<&str> = out.iter().map(|it| it.0).collect();
        assert_eq!(keys, [a, b, c]);
    }

    #[test]
    fn take_round_robin__shared_prefix_uuids__are_distinct_projects() {
        let a = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let b = "aaaaaaaa-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let items = vec![(a, 0), (a, 1), (b, 2), (b, 3)];
        let out = take_round_robin(items, |it| project_key(Some(it.0)), 1, 8);
        assert_eq!(
            out.len(),
            2,
            "full UUID identity; shared 8-hex is not one bucket"
        );
        assert_eq!(out[0].0, a);
        assert_eq!(out[1].0, b);
        assert_eq!(project_tag(Some(a)), project_tag(Some(b)));
        assert_eq!(project_tag(Some("deadbeef-not-a-uuid")), "[unknown]");
        assert_eq!(span_count([a, b]), 2);
    }
}
