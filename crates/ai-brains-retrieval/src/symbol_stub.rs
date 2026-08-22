//! T70 code-symbol stub detector, SQL exclude fragment, and content dedupe (T260).
//!
//! Detector is source of truth (F19). SQL `GLOB` is a case-sensitive subset
//! (digit required after the locator colon). Do not use `LIKE`.

use crate::ranking::strip_assistant_prefix;
use crate::recall::RecallHit;
use std::collections::BTreeSet;

/// Closed T70 `symbol_kind` list (case-sensitive). Includes `Unknown` (missing JSON kind).
pub const SYMBOL_KINDS: &[&str] = &[
    "Module",
    "Struct",
    "Function",
    "Fn",
    "Enum",
    "Trait",
    "Type",
    "Const",
    "Static",
    "Impl",
    "Macro",
    "Field",
    "Variant",
    "Union",
    "Method",
    "Interface",
    "Class",
    "Unknown",
];

/// True for live T70 `symbol_content` (`{kind} {qualified} ({path}:{line})`).
///
/// Fast-rejects unless the trimmed string ends with `)`. Strips one leading
/// `ASSISTANT: `. First token must be a [`SYMBOL_KINDS`] member. Suffix after
/// the last ` (` must be `*:digits)`.
pub fn is_symbol_stub_content(content: &str) -> bool {
    let trimmed = content.trim();
    if !trimmed.ends_with(')') {
        return false;
    }
    let stripped = strip_assistant_prefix(trimmed).trim();
    let Some((kind, rest)) = stripped.split_once(' ') else {
        return false;
    };
    if !SYMBOL_KINDS.contains(&kind) {
        return false;
    }
    let Some(idx) = rest.rfind(" (") else {
        return false;
    };
    let locator = &rest[idx + 2..];
    let Some(inner) = locator.strip_suffix(')') else {
        return false;
    };
    let Some((_, digits)) = inner.rsplit_once(':') else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Bind-free `AND NOT (col GLOB …)` list from [`SYMBOL_KINDS`] (F19).
///
/// `column` must be a SQL identifier (`content` or `mp.content`). Never pass
/// user input.
pub fn symbol_stub_sql_exclusion(column: &str) -> String {
    debug_assert!(
        is_safe_sql_ident(column),
        "symbol_stub_sql_exclusion column must be a SQL identifier"
    );
    let mut parts = Vec::with_capacity(SYMBOL_KINDS.len().saturating_mul(2));
    for kind in SYMBOL_KINDS {
        parts.push(format!("{column} GLOB '{kind} * (*:[0-9]*)'"));
        parts.push(format!("{column} GLOB 'ASSISTANT: {kind} * (*:[0-9]*)'"));
    }
    format!(" AND NOT ({})", parts.join(" OR "))
}

/// Collapse identical stub `content` to the first row (already sorted).
///
/// Non-stub rows are never collapsed. F3: call after [`crate::ranking::rerank_hits`].
pub fn dedupe_symbol_stubs(hits: &mut Vec<RecallHit>) {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    hits.retain(|h| {
        if !is_symbol_stub_content(&h.content) {
            return true;
        }
        seen.insert(h.content.clone())
    });
}

/// Drop detector-positive rows (memory retain; F7).
pub fn retain_non_symbol_stubs(hits: &mut Vec<RecallHit>) {
    hits.retain(|h| !is_symbol_stub_content(&h.content));
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

    #[test]
    fn is_symbol_stub_content__t70_module_format__true() {
        assert!(is_symbol_stub_content(
            "Module sqlite_backend (crates/ai-brains-graph/src/lib.rs:7)"
        ));
        assert!(is_symbol_stub_content(
            "Struct Project (crates/ai-brains-core/src/project.rs:6)"
        ));
        assert!(is_symbol_stub_content(
            "Function capture_metadata (crates/ai-brains-capture/src/git_capture.rs:4)"
        ));
        assert!(is_symbol_stub_content(
            "Enum CaptureError (crates/ai-brains-capture/src/errors.rs:28)"
        ));
    }

    #[test]
    fn is_symbol_stub_content__assistant_prefix__true() {
        assert!(is_symbol_stub_content("ASSISTANT: Module foo (a.rs:1)"));
    }

    #[test]
    fn is_symbol_stub_content__fn_kind__true() {
        assert!(is_symbol_stub_content("Fn do_thing (src/lib.rs:10)"));
    }

    #[test]
    fn is_symbol_stub_content__decision_quoting_module__false() {
        assert!(!is_symbol_stub_content(
            "DECISION: we mention Module sqlite_backend in the body"
        ));
        assert!(!is_symbol_stub_content(
            "CONSTRAINT: never return Module sqlite_backend as the answer"
        ));
        assert!(!is_symbol_stub_content("just a chat turn about Module"));
        assert!(!is_symbol_stub_content(""));
        assert!(!is_symbol_stub_content("   "));
    }

    #[test]
    fn is_symbol_stub_content__function_without_locator__false() {
        assert!(!is_symbol_stub_content("Function capture_metadata"));
        assert!(!is_symbol_stub_content("Module foo (draft: notes)"));
        assert!(!is_symbol_stub_content("module foo (src/foo.rs:1)"));
    }

    #[test]
    fn symbol_stub_sql_exclusion__glob_digit_class_not_like() {
        let sql = symbol_stub_sql_exclusion("mp.content");
        assert!(sql.contains("GLOB"), "F19: must use GLOB; got {sql}");
        assert!(
            !sql.to_ascii_uppercase().contains("LIKE"),
            "F19: must not emit LIKE; got {sql}"
        );
        assert!(
            sql.contains("[0-9]"),
            "F19: digit class required; got {sql}"
        );
        assert!(sql.contains("ASSISTANT: Module"));
        for kind in SYMBOL_KINDS {
            assert!(
                sql.contains(&format!("'{kind} * (*:[0-9]*)'")),
                "missing kind {kind} in {sql}"
            );
        }
    }

    #[test]
    fn dedupe_symbol_stubs__identical_content_distinct_ids__one() {
        let mut hits = vec![
            hit("id-a", "Module foo (src/foo.rs:1)"),
            hit("id-b", "Module foo (src/foo.rs:1)"),
            hit("id-c", "Module foo (src/foo.rs:1)"),
            hit("id-d", "DECISION: we chose foo for the bar path"),
        ];
        dedupe_symbol_stubs(&mut hits);
        let stubs: Vec<_> = hits
            .iter()
            .filter(|h| h.content == "Module foo (src/foo.rs:1)")
            .collect();
        assert_eq!(stubs.len(), 1, "AC6: identical stubs collapse to one");
        assert_eq!(
            stubs[0].memory_id, "id-a",
            "keep first in already-sorted order"
        );
        assert!(hits.iter().any(|h| h.memory_id == "id-d"));
    }

    #[test]
    fn retain_non_symbol_stubs__bridge_source_stub__dropped() {
        let mut hits = vec![
            RecallHit::bridge(
                "br-stub".into(),
                "Module sqlite_backend (crates/ai-brains-graph/src/lib.rs:7)".into(),
                Some(12.0),
                "bridge".into(),
                None,
                None,
            ),
            RecallHit::bridge(
                "br-dec".into(),
                "DECISION: we chose the sqlite graph backend".into(),
                Some(11.0),
                "bridge".into(),
                None,
                None,
            ),
        ];
        retain_non_symbol_stubs(&mut hits);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].memory_id, "br-dec");
    }
}
