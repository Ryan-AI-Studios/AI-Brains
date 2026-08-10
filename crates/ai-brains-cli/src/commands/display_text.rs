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
}
