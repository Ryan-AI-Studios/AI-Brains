//! T283 — human `project list` cwd-first permute (JSON/footer keep store order).
//!
//! `promote_cwd_owner` is pure. `list()` probes cwd via
//! `resolve_path_alias_for_location` (`project.rs`) then permutes the human
//! table only.

use ai_brains_store::ProjectListDetail;

/// Promote the cwd path-owner to index 0 when it is present in `rows`.
///
/// `cwd_owner` comes from `resolve_path_alias_for_location` (`project.rs`
/// `:237–248`). Exact `project_id` match. Store order otherwise. JSON and the
/// unaliased footer keep the unpromoted vec.
pub(crate) fn promote_cwd_owner(
    rows: &[ProjectListDetail],
    cwd_owner: Option<&str>,
) -> Vec<ProjectListDetail> {
    let Some(owner) = cwd_owner.filter(|s| !s.is_empty()) else {
        return rows.to_vec();
    };
    let Some(idx) = rows.iter().position(|r| r.project_id == owner) else {
        return rows.to_vec();
    };
    if idx == 0 {
        return rows.to_vec();
    }
    let mut out = Vec::with_capacity(rows.len());
    out.push(rows[idx].clone());
    out.extend(rows[..idx].iter().cloned());
    out.extend(rows[idx + 1..].iter().cloned());
    out
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    fn row(id: &str, count: usize) -> ProjectListDetail {
        ProjectListDetail {
            project_id: id.to_string(),
            name: id.to_string(),
            alias: String::new(),
            memory_count: count,
            last_activity: String::new(),
            path: None,
        }
    }

    fn ids(rows: &[ProjectListDetail]) -> Vec<&str> {
        rows.iter().map(|r| r.project_id.as_str()).collect()
    }

    #[test]
    fn promote_cwd_owner__middle_id__becomes_first() {
        let rows = vec![row("a", 30), row("b", 20), row("c", 10)];

        let last = promote_cwd_owner(&rows, Some("c"));
        assert_eq!(ids(&last), ["c", "a", "b"]);
        assert_eq!(last.len(), 3);
        assert_eq!(last.iter().filter(|r| r.project_id == "c").count(), 1);

        let mid = promote_cwd_owner(&rows, Some("b"));
        assert_eq!(ids(&mid), ["b", "a", "c"]);
        assert_eq!(mid.len(), 3);
        assert_eq!(mid.iter().filter(|r| r.project_id == "b").count(), 1);

        let first = promote_cwd_owner(&rows, Some("a"));
        assert_eq!(ids(&first), ["a", "b", "c"]);
        assert_eq!(first.len(), 3);
        assert_eq!(first.iter().filter(|r| r.project_id == "a").count(), 1);
    }

    #[rstest::rstest]
    #[case(None)]
    #[case(Some(""))]
    #[case(Some("missing"))]
    fn promote_cwd_owner__none_empty_missing__clone(#[case] owner: Option<&str>) {
        let rows = vec![row("a", 30), row("b", 20)];
        assert_eq!(promote_cwd_owner(&rows, owner), rows);

        let empty: Vec<ProjectListDetail> = Vec::new();
        assert_eq!(promote_cwd_owner(&empty, owner), empty);
    }
}
