//! T264 — peel / upgrade `--global` project tags on the pretty path.
//!
//! Retrieval writes `[8hex]` / `[unknown]` on item first lines. Human/pretty
//! peels the **leading** tag, strips chrome on the remainder, then reattaches
//! an upgraded label (`display_label` + `truncate_chars(32)` + `]` → `·`).
//! Body-internal `[8hex]` is never rewritten (F4).

use super::project::{display_label, truncate_chars};

/// Lookup for T264 pretty tag upgrade: 8-hex prefix → (project_id, name, alias).
type ProjectTagLookup<'a> = dyn Fn(&str) -> Option<(String, String, String)> + 'a;

/// Unique full project id whose first 8 hex chars match `tag8`.
/// Collision or miss → `None` so pretty keeps the raw `[8hex]` tag.
pub(crate) fn unique_project_id_for_tag<'a>(
    ids: impl IntoIterator<Item = &'a str>,
    tag8: &str,
) -> Option<String> {
    let mut hits: Vec<&str> = ids
        .into_iter()
        .filter(|id| id.len() >= 8 && id[..8].eq_ignore_ascii_case(tag8))
        .collect();
    hits.sort_unstable();
    if hits.len() == 1 {
        Some(hits[0].to_string())
    } else {
        None
    }
}

/// Peel a leading `[` + 8 ASCII hex + `]` or `[unknown]`. One shot — remainder
/// is not re-scanned even if it still contains a tag token.
pub(crate) fn peel_global_tag(line: &str) -> (Option<String>, &str) {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix('[') else {
        return (None, line);
    };
    let Some(end) = rest.find(']') else {
        return (None, line);
    };
    let inner = &rest[..end];
    let is_hex8 = inner.len() == 8 && inner.bytes().all(|b| b.is_ascii_hexdigit());
    if !is_hex8 && inner != "unknown" {
        return (None, line);
    }
    let after = &rest[end + 1..];
    let remainder = after.strip_prefix(' ').unwrap_or(after);
    (Some(format!("[{inner}]")), remainder)
}

/// Upgrade a peeled tag token. `resolved` is `(project_id, name, alias)`.
/// Missing project / `[unknown]` stay as the original token (F3 fallback).
pub(crate) fn upgrade_global_tag(tag: &str, resolved: Option<(&str, &str, &str)>) -> String {
    if tag == "[unknown]" {
        return tag.to_string();
    }
    let Some((project_id, name, alias)) = resolved else {
        return tag.to_string();
    };
    let label = display_label(name, alias, project_id);
    let sanitized = truncate_chars(&label, 32).replace(']', "·");
    format!("[{sanitized}]")
}

/// Peel leading tag, upgrade it, reattach. Body after the leading tag is intact.
#[cfg(test)]
pub(crate) fn upgrade_leading_tag_on_line(
    line: &str,
    resolved: Option<(&str, &str, &str)>,
) -> String {
    let (tag, rest) = peel_global_tag(line);
    match tag {
        Some(t) => {
            let upgraded = upgrade_global_tag(&t, resolved);
            if rest.is_empty() {
                upgraded
            } else {
                format!("{upgraded} {rest}")
            }
        }
        None => line.to_string(),
    }
}

/// Upgrade the `[8hex]` / `[unknown]` slot on `--- Session: uuid [tag] ---`.
/// Does not scan the rest of the line (F4 leading/slot only).
pub(crate) fn upgrade_session_header_tag(
    header: &str,
    resolved: Option<(&str, &str, &str)>,
) -> String {
    let t = header.trim();
    let Some(without_close) = t.strip_suffix("---") else {
        return t.to_string();
    };
    let Some(tag_at) = without_close.rfind('[') else {
        return t.to_string();
    };
    let from_tag = &without_close[tag_at..];
    let Some(end) = from_tag.find(']') else {
        return t.to_string();
    };
    let token = &from_tag[..=end];
    let inner = &token[1..token.len() - 1];
    let is_hex8 = inner.len() == 8 && inner.bytes().all(|b| b.is_ascii_hexdigit());
    if !is_hex8 && inner != "unknown" {
        return t.to_string();
    }
    let upgraded = upgrade_global_tag(token, resolved);
    format!("{}{} ---", &without_close[..tag_at], upgraded)
}

/// Resolve a peeled `[8hex]` via `lookup(inner)` then upgrade.
pub(crate) fn upgrade_tag_with_lookup(tag: &str, lookup: Option<&ProjectTagLookup<'_>>) -> String {
    let inner = tag
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or("");
    if inner == "unknown" || inner.is_empty() {
        return upgrade_global_tag(tag, None);
    }
    let resolved = lookup.and_then(|f| f(inner));
    let refs = resolved
        .as_ref()
        .map(|(id, n, a)| (id.as_str(), n.as_str(), a.as_str()));
    upgrade_global_tag(tag, refs)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn peel_global_tag__tagged_timestamp_role__chrome_still_strips() {
        // AC3 (peel half). Chrome-on-remainder is locked in `preflight.rs`
        // so this module does not import `preflight` (avoids a module cycle).
        let input = "[3581317d] (just now) ASSISTANT: DECISION: x see [3581317d]";
        let (tag, rest) = peel_global_tag(input);
        assert_eq!(
            tag.as_deref(),
            Some("[3581317d]"),
            "leading tag peeled; got {tag:?}"
        );
        assert_eq!(
            rest, "(just now) ASSISTANT: DECISION: x see [3581317d]",
            "remainder keeps timestamp/role + body-internal tag"
        );
        let (again, rest2) = peel_global_tag(rest);
        assert!(
            again.is_none(),
            "remainder that still contains [3581317d] is not peeled again; got {again:?}"
        );
        assert_eq!(rest2, rest);
    }

    #[test]
    fn upgrade_global_tag__alias_missing_and_bracket() {
        // AC4
        let pid = "3581317d-601e-44f7-ab84-fde90aa12d3c";
        assert_eq!(
            upgrade_global_tag("[3581317d]", Some((pid, "Acme Corp", "acme"))),
            "[acme]",
            "alias wins"
        );
        assert_eq!(
            upgrade_global_tag("[3581317d]", None),
            "[3581317d]",
            "missing project keeps 8-char"
        );
        assert_eq!(
            upgrade_global_tag("[unknown]", None),
            "[unknown]",
            "unknown stays unknown"
        );
        let bracket_name = "foo]bar]baz-extra-long-project-name-here";
        let upgraded = upgrade_global_tag("[3581317d]", Some((pid, bracket_name, "")));
        assert!(
            upgraded.starts_with('[') && upgraded.ends_with(']'),
            "wrapped; got {upgraded}"
        );
        assert!(
            !upgraded[1..upgraded.len() - 1].contains(']'),
            "] sanitized to ·; got {upgraded}"
        );
        assert!(upgraded.contains('·'), "F24 ] → ·; got {upgraded}");

        let line = upgrade_leading_tag_on_line(
            "[3581317d] see [aaaaaaaa] later",
            Some((pid, "Acme Corp", "acme")),
        );
        assert_eq!(
            line, "[acme] see [aaaaaaaa] later",
            "upgrade leading only; body-internal [8hex] stays"
        );
    }

    #[test]
    fn unique_project_id_for_tag__collision__returns_none() {
        let a = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let b = "aaaaaaaa-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let c = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        assert_eq!(
            unique_project_id_for_tag([a, b, c], "aaaaaaaa"),
            None,
            "shared 8-hex prefix must not pick an arbitrary project"
        );
        assert_eq!(
            unique_project_id_for_tag([a, b, c], "cccccccc").as_deref(),
            Some(c)
        );
        assert_eq!(unique_project_id_for_tag([a], "ffffffff"), None);
    }
}
