//! Session-chrome detector, authority GLOB, prefer-fill, and first-line collapse (T274).
//!
//! Detector is source of truth (F8). SQL `GLOB` is a case-sensitive prefix subset.

use crate::ranking::{PinKind, classify_pin_kind, first_contentful_line, strip_assistant_prefix};
use crate::recall::RecallHit;
use std::collections::BTreeSet;

/// Composite penalty applied in [`crate::ranking::rerank_hits`] when the detector
/// is true (same scale as [`crate::ranking::SYMBOL_PENALTY`]).
pub const SESSION_CHROME_PENALTY: f64 = 16.0;

/// True for closed-list harness session dumps (first contentful line).
pub fn is_session_chrome(content: &str) -> bool {
    let line = first_contentful_line(content);
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("## objective") {
        return true;
    }
    if lower.starts_with("# track plan review") {
        return true;
    }
    if lower.starts_with("### track") && lower.contains("review") {
        return true;
    }
    if lower.starts_with("# ai-brains onboarding") {
        return true;
    }
    if lower.starts_with("# ai-brains session onboarding") {
        return true;
    }
    if lower.starts_with("# review of track") {
        return true;
    }
    if lower.starts_with("```json") {
        return true;
    }
    let head: String = strip_assistant_prefix(content)
        .trim()
        .chars()
        .take(500)
        .collect();
    line.starts_with('{') && head.contains("\"decisions\":")
}

/// Leading DECISION / CONSTRAINT / INVARIANT (F3 maps INVARIANT → Constraint).
pub fn is_authority_pin_content(content: &str) -> bool {
    matches!(
        classify_pin_kind(content),
        PinKind::Decision | PinKind::Constraint
    )
}

/// Bind-free `AND (col GLOB …)` for decision-class authority prefixes (F36).
///
/// `column` must be a SQL identifier (`content` / `mp.content` / `m.content`).
/// HOTSPOT is **not** included (recall pass 1 is DECISION/CONSTRAINT/INVARIANT).
pub fn authority_glob_sql(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "authority_glob_sql column must be a SQL identifier"
    );
    let prefixes = [
        "DECISION:",
        "CONSTRAINT:",
        "INVARIANT:",
        "ASSISTANT: DECISION:",
        "ASSISTANT: CONSTRAINT:",
        "ASSISTANT: INVARIANT:",
    ];
    let parts: Vec<String> = prefixes
        .iter()
        .map(|p| format!("{column} GLOB '{p}*'"))
        .collect();
    format!(" AND ({})", parts.join(" OR "))
}

/// Bind-free `AND (col GLOB 'TAGS:*' OR col GLOB 'ASSISTANT: TAGS:*')` (T285 F7).
///
/// `column` must be a SQL identifier (`content` / `mp.content` / `m.content`).
pub fn tags_envelope_sql(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "tags_envelope_sql column must be a SQL identifier"
    );
    format!(" AND ({column} GLOB 'TAGS:*' OR {column} GLOB 'ASSISTANT: TAGS:*')")
}

/// Index pass-1 GLOB: [`authority_glob_sql`] plus leading HOTSPOT (F11).
pub fn index_marker_glob_sql(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "index_marker_glob_sql column must be a SQL identifier"
    );
    let auth = authority_glob_sql(column);
    // authority_glob_sql returns ` AND (…)` — extend the group with HOTSPOT prefixes.
    let inner = auth
        .strip_prefix(" AND (")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(auth.as_str());
    format!(" AND ({inner} OR {column} GLOB 'HOTSPOT:*' OR {column} GLOB 'ASSISTANT: HOTSPOT:*')")
}

/// Index pass-1 GLOB: marker+HOTSPOT **or** TAGS envelope (T286 F2).
///
/// Same inner-join shape as lexical Prefer (`AND (marker OR tags)`). Does **not**
/// stack two `AND (` clauses. `index_marker_glob_sql` stays marker-only.
pub fn index_pass1_glob_sql(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "index_pass1_glob_sql column must be a SQL identifier"
    );
    let marker = index_marker_glob_sql(column);
    let tags = tags_envelope_sql(column);
    let marker_inner = marker
        .strip_prefix(" AND (")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(marker.as_str());
    let tags_inner = tags
        .strip_prefix(" AND (")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(tags.as_str());
    format!(" AND ({marker_inner} OR {tags_inner})")
}

/// Safety GLOB: leading CONSTRAINT / INVARIANT / HOTSPOT only (T279 F1).
///
/// Does **not** include `DECISION:` — that belongs to Index (`index_marker_glob_sql`).
pub fn safety_marker_glob_sql(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "safety_marker_glob_sql column must be a SQL identifier"
    );
    let prefixes = [
        "CONSTRAINT:",
        "INVARIANT:",
        "HOTSPOT:",
        "ASSISTANT: CONSTRAINT:",
        "ASSISTANT: INVARIANT:",
        "ASSISTANT: HOTSPOT:",
    ];
    let parts: Vec<String> = prefixes
        .iter()
        .map(|p| format!("{column} GLOB '{p}*'"))
        .collect();
    format!(" AND ({})", parts.join(" OR "))
}

/// `AND col NOT IN (?,?,…)` with `n` placeholders. `n == 0` → omit (F35).
pub fn bound_not_in_sql(column: &str, n: usize) -> Option<String> {
    debug_assert!(
        is_safe_sql_ident(column),
        "bound_not_in_sql column must be a SQL identifier"
    );
    if n == 0 {
        return None;
    }
    let placeholders = vec!["?"; n].join(",");
    Some(format!(" AND {column} NOT IN ({placeholders})"))
}

/// Collapse detector-chrome rows that share the same first contentful line.
///
/// Non-chrome rows never collapse (two `DECISION:` pins stay two). Call after
/// [`crate::ranking::rerank_hits`].
pub fn dedupe_session_chrome(hits: &mut Vec<RecallHit>) {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    hits.retain(|h| {
        if !is_session_chrome(&h.content) {
            return true;
        }
        let key = first_contentful_line(&h.content).to_ascii_lowercase();
        seen.insert(key)
    });
}

/// T285 F36: chrome-shaped parents must not seed graph neighbors.
///
/// True for authority pins (after envelope). False for session chrome.
pub fn parent_seeds_graph_neighbors(content: &str) -> bool {
    !is_session_chrome(content)
}

/// Authority hits first (relative order preserved), then others, cap `depth` (F9).
pub fn prefer_authority_hits(hits: Vec<RecallHit>, depth: usize) -> Vec<RecallHit> {
    let mut auth = Vec::new();
    let mut other = Vec::new();
    for h in hits {
        if is_authority_pin_content(&h.content) {
            auth.push(h);
        } else {
            other.push(h);
        }
    }
    auth.extend(other);
    auth.truncate(depth);
    auth
}

fn is_safe_sql_ident(column: &str) -> bool {
    let mut parts = column.split('.');
    let all_ok = parts.all(|p| {
        let mut chars = p.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            _ => false,
        }
    });
    all_ok && !column.is_empty() && !column.starts_with('.') && !column.ends_with('.')
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::ranking::ScoreKind;
    use crate::recall::RecallHit;

    fn hit(id: &str, content: &str) -> RecallHit {
        RecallHit {
            memory_id: id.to_string(),
            content: content.to_string(),
            source: "fts".to_string(),
            score: Some(-1.0),
            privacy: None,
            session_id: None,
            updated_at: None,
            is_plan_demoted: false,
            score_kind: ScoreKind::Bm25LowerBetter,
            cosine: None,
            project_id: None,
        }
    }

    use rstest::rstest;

    #[rstest]
    #[case("## Objective\nbody", true)]
    #[case("# Track Plan Review: T274", true)]
    #[case("### Track 248 Review", true)]
    #[case("# AI-Brains Onboarding", true)]
    #[case("```json\n{\"a\":1}\n```", true)]
    #[case("{\"decisions\": [\"x\"]}", true)]
    #[case("# AI-Brains Session Onboarding Complete", true)]
    #[case("# Review of Track 254: ranking hole", true)]
    #[case("DECISION: we chose X", false)]
    #[case("CONSTRAINT: must be safe", false)]
    #[case("# Heading without chrome prefixes", false)]
    fn is_session_chrome__live_and_closed_prefixes__ac2(
        #[case] content: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(
            is_session_chrome(content),
            expected,
            "AC2 content={content:?}"
        );
    }

    #[test]
    fn parent_seeds_graph_neighbors__chrome_false_authority_true__ac6() {
        assert!(
            !parent_seeds_graph_neighbors("# Review of Track 254: ranking hole"),
            "AC6: live review chrome must not seed"
        );
        assert!(
            !parent_seeds_graph_neighbors("# AI-Brains Session Onboarding Complete"),
            "AC6: live onboarding chrome must not seed"
        );
        assert!(
            !parent_seeds_graph_neighbors("## Objective\nbody"),
            "AC6: T274 chrome must not seed"
        );
        assert!(
            !parent_seeds_graph_neighbors("ASSISTANT: TAGS: x\n## Objective\nbody"),
            "AC6: TAGS envelope then ## Objective is still chrome"
        );
        assert!(
            parent_seeds_graph_neighbors("DECISION: we chose X"),
            "AC6: authority pin may seed"
        );
        assert!(
            parent_seeds_graph_neighbors("ASSISTANT: TAGS: t\nDECISION: we chose X"),
            "AC6: tagged DECISION may seed"
        );
    }

    #[test]
    fn is_session_chrome__closed_prefixes__true() {
        assert!(is_session_chrome("## Objective\nbody"));
        assert!(is_session_chrome("# Track Plan Review: T274"));
        assert!(is_session_chrome("### Track 248 Review"));
        assert!(is_session_chrome("# AI-Brains Onboarding"));
        assert!(is_session_chrome("```json\n{\"a\":1}\n```"));
        assert!(is_session_chrome("{\"decisions\": [\"x\"]}"));
    }

    #[test]
    fn is_session_chrome__authority_and_chat__false() {
        assert!(!is_session_chrome("DECISION: we chose X"));
        assert!(!is_session_chrome("CONSTRAINT: must be safe"));
        assert!(!is_session_chrome("just a chat turn about ranking"));
        assert!(!is_session_chrome("# Heading without chrome prefixes"));
    }

    #[test]
    fn safety_marker_glob_sql__includes_constraint_not_decision() {
        let sql = safety_marker_glob_sql("m.content");
        assert!(
            sql.contains("CONSTRAINT:*"),
            "AC1: CONSTRAINT:* GLOB; got {sql}"
        );
        assert!(
            sql.contains("INVARIANT:*"),
            "AC1: INVARIANT:* GLOB; got {sql}"
        );
        assert!(sql.contains("HOTSPOT:*"), "AC1: HOTSPOT:* GLOB; got {sql}");
        assert!(
            sql.contains("ASSISTANT: CONSTRAINT:*"),
            "AC1: ASSISTANT: CONSTRAINT:* GLOB; got {sql}"
        );
        assert!(
            !sql.contains("DECISION:"),
            "AC1: Safety GLOB must not include DECISION:; got {sql}"
        );
        assert!(sql.contains("GLOB"), "AC1: must use GLOB; got {sql}");
        assert!(
            !sql.to_ascii_uppercase().contains("LIKE"),
            "AC1: must not emit LIKE; got {sql}"
        );
    }

    #[test]
    fn index_pass1_glob_sql__tags_or_marker__single_and_group() {
        let sql = index_pass1_glob_sql("m.content");
        assert!(sql.contains("GLOB 'TAGS:*'"), "AC4: TAGS:* GLOB; got {sql}");
        assert!(
            sql.contains("GLOB 'ASSISTANT: TAGS:*'"),
            "AC4: ASSISTANT: TAGS:* GLOB; got {sql}"
        );
        assert!(
            sql.contains("GLOB 'DECISION:*'"),
            "AC4: DECISION:* GLOB; got {sql}"
        );
        assert!(
            sql.contains("GLOB 'HOTSPOT:*'"),
            "AC4: HOTSPOT:* GLOB; got {sql}"
        );
        assert_eq!(
            sql.matches(" AND (").count(),
            1,
            "AC4: single AND ( grouping, not stacked ANDs; got {sql}"
        );
        assert!(sql.contains(" OR "), "AC4: OR join; got {sql}");
        let src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/session_chrome.rs"
        ));
        let start = src
            .find("pub fn index_pass1_glob_sql")
            .expect("index_pass1_glob_sql present");
        let rest = &src[start..];
        let end = rest.find("\npub fn ").unwrap_or(rest.len());
        let body = &rest[..end];
        assert!(
            body.contains("debug_assert!(") && body.contains("is_safe_sql_ident(column)"),
            "AC4: helper must debug_assert is_safe_sql_ident; body={body}"
        );
    }

    #[test]
    fn authority_glob_sql__glob_not_like() {
        let sql = authority_glob_sql("mp.content");
        assert!(sql.contains("GLOB"), "F36: must use GLOB; got {sql}");
        assert!(
            !sql.to_ascii_uppercase().contains("LIKE"),
            "F36: must not emit LIKE; got {sql}"
        );
        assert!(sql.contains("DECISION:*"));
        assert!(sql.contains("CONSTRAINT:*"));
        assert!(sql.contains("INVARIANT:*"));
        assert!(sql.contains("ASSISTANT: DECISION:*"));
        assert!(
            !sql.contains("HOTSPOT"),
            "F7: HOTSPOT is not recall pass-1; got {sql}"
        );
    }

    #[test]
    fn bound_not_in_sql__placeholders_no_literals() {
        assert!(bound_not_in_sql("mp.memory_id", 0).is_none());
        let sql = bound_not_in_sql("mp.memory_id", 3).expect("n=3");
        assert!(
            sql.contains("NOT IN (?,?,?)"),
            "F35: only ? placeholders; got {sql}"
        );
        assert!(!sql.contains('-'), "F35: no UUID literals; got {sql}");
    }

    #[test]
    fn dedupe_session_chrome__identical_first_line__one_chrome_two_pins() {
        let mut hits = vec![
            hit("c1", "## Objective\nfirst dump"),
            hit("c2", "## Objective\nsecond dump"),
            hit("p1", "DECISION: first pin body"),
            hit("p2", "DECISION: second pin body"),
        ];
        dedupe_session_chrome(&mut hits);
        let chrome: Vec<_> = hits
            .iter()
            .filter(|h| h.content.starts_with("## Objective"))
            .collect();
        assert_eq!(
            chrome.len(),
            1,
            "AC5: identical chrome first line collapses"
        );
        assert_eq!(chrome[0].memory_id, "c1");
        assert_eq!(
            hits.iter()
                .filter(|h| h.content.starts_with("DECISION:"))
                .count(),
            2,
            "AC5: distinct DECISION pins never collapse"
        );
    }

    #[test]
    fn prefer_authority_hits__authority_first_then_cap() {
        let shuffled = vec![
            hit("c1", "## Objective\ndump one"),
            hit("p1", "DECISION: pin one"),
            hit("c2", "## Objective\ndump two"),
            hit("p2", "CONSTRAINT: pin two"),
        ];
        let out = prefer_authority_hits(shuffled, 3);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].memory_id, "p1");
        assert_eq!(out[1].memory_id, "p2");
        assert_eq!(out[2].memory_id, "c1");
    }
}
