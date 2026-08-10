//! Shared pure display-text helpers (T219 / prep T224).
//!
//! Capture independence: string ops only — no models, vault I/O, or graph.

/// Strip a leading case-sensitive role token (`USER:` / `ASSISTANT:` / `SYSTEM:`)
/// and return the remainder after `trim_start` on the suffix.
///
/// Mid-line and lowercase tokens are left unchanged (T216 SOOT / T219 F7/F39).
/// Borrowing helper — no allocation (M3).
pub(crate) fn strip_role_prefix(line: &str) -> &str {
    for prefix in ["USER:", "ASSISTANT:", "SYSTEM:"] {
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
}
