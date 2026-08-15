//! Shared pure display-text helpers (T219 / prep T224).
//!
//! Capture independence: string ops only — no models, vault I/O, or graph.

/// Leading case-sensitive role tokens (T216/T219/T224 SOOT — single list).
const ROLE_PREFIXES: &[&str] = &["USER:", "ASSISTANT:", "SYSTEM:"];

/// True when `line` starts with a case-sensitive role token (no mid-line match).
///
/// Used by preflight turn counting (`is_session_turn_start`) so detection shares
/// the same token list as [`strip_role_prefix`] (T224 AC10).
pub(crate) fn has_leading_role_prefix(line: &str) -> bool {
    ROLE_PREFIXES.iter().any(|p| line.starts_with(p))
}

/// Strip a leading case-sensitive role token (`USER:` / `ASSISTANT:` / `SYSTEM:`)
/// and return the remainder after `trim_start` on the suffix.
///
/// Mid-line and lowercase tokens are left unchanged (T216 SOOT / T219 F7/F39).
/// Borrowing helper — no allocation (M3).
///
/// **Consumers (T224 O4 — single SOOT; do not fork a second prefix list):**
/// - [`crate::commands::memory::preview_line`]
/// - [`crate::commands::memory::content_has_tag`]
/// - preflight pretty display lines
/// - preflight turn-start detection ([`has_leading_role_prefix`])
/// - [`crate::commands::recall::format_pretty_hit_line`]
/// - forget human previews (via `preview_line` / `forget_match_preview`)
pub(crate) fn strip_role_prefix(line: &str) -> &str {
    for prefix in ROLE_PREFIXES {
        if let Some(rest) = line.strip_prefix(prefix) {
            return rest.trim_start();
        }
    }
    line
}

/// Unicode-safe preview/line cap with a single `…` (T216 / T250).
///
/// `max_chars == 0` → empty. Under-budget strings are cloned unchanged.
/// Over-budget: keep `max_chars.saturating_sub(1)` chars + Unicode ellipsis.
pub(crate) fn truncate_preview_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let truncated: String = s.chars().take(keep).collect();
    format!("{truncated}…")
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// AC9: leading strip; mid-line leave; lowercase leave.
    #[test]
    fn strip_role_prefix__leading_case_sensitive__strips_and_leaves_mid_lower() {
        assert_eq!(
            strip_role_prefix("ASSISTANT: DECISION: use SQLCipher"),
            "DECISION: use SQLCipher"
        );
        assert_eq!(strip_role_prefix("USER: hello world"), "hello world");
        assert_eq!(strip_role_prefix("SYSTEM: note"), "note");
        assert_eq!(
            strip_role_prefix("text ASSISTANT: still here"),
            "text ASSISTANT: still here"
        );
        assert_eq!(
            strip_role_prefix("assistant: leave me"),
            "assistant: leave me"
        );
        assert_eq!(strip_role_prefix("no prefix"), "no prefix");
        // Whitespace after prefix is trimmed on the suffix only.
        assert_eq!(strip_role_prefix("ASSISTANT:   body"), "body");
    }

    #[test]
    fn has_leading_role_prefix__leading_only__true_mid_lower_false() {
        assert!(has_leading_role_prefix("ASSISTANT: x"));
        assert!(has_leading_role_prefix("USER: x"));
        assert!(has_leading_role_prefix("SYSTEM: x"));
        assert!(!has_leading_role_prefix("text ASSISTANT: x"));
        assert!(!has_leading_role_prefix("assistant: x"));
        assert!(!has_leading_role_prefix("no prefix"));
    }

    #[test]
    fn truncate_preview_chars__max_zero__empty() {
        assert_eq!(truncate_preview_chars("anything", 0), "");
    }

    #[test]
    fn truncate_preview_chars__under_max__unchanged() {
        assert_eq!(truncate_preview_chars("short", 80), "short");
    }

    #[test]
    fn truncate_preview_chars__emdash_over_max__ellipsis_no_panic() {
        let s = "———————————————";
        let out = truncate_preview_chars(s, 10);
        assert_eq!(out.chars().count(), 10, "got {out:?}");
        assert!(out.ends_with('…'), "got {out:?}");
    }

    #[test]
    fn truncate_preview_chars__cjk_over_max__no_mid_char_slice() {
        let s = "日本語テストプレビュー境界値チェック用の長い行です";
        let out = truncate_preview_chars(s, 10);
        assert_eq!(out.chars().count(), 10, "got {out:?}");
        assert!(out.ends_with('…'), "got {out:?}");
        assert_eq!(out.chars().next(), Some('日'));
    }
}
