//! Word-budget helpers for preflight / governed markdown assembly.
//!
//! T219 F1/F2/F2b: `trim_to_word_budget` preserves newlines while limiting to
//! `max_words` (via whitespace tokens). Over-budget outputs append a trailing
//! `…` sentinel on its own line; the sentinel is display chrome and is **not**
//! included in content word counts (`content_word_count`).

/// Count whitespace-separated tokens (unchanged semantics).
pub fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

/// Content word count excluding a trailing F2b truncation sentinel (`…` / `...`).
///
/// Prefer this for budget-window `PreflightContext.word_count` so
/// `content_word_count(text) <= max_words` always holds after trim (F32 / AC10).
pub fn content_word_count(input: &str) -> usize {
    let without = strip_trailing_truncation_sentinel(input);
    word_count(without)
}

/// Strip a trailing F2b sentinel if present (own-line or end-of-string).
fn strip_trailing_truncation_sentinel(input: &str) -> &str {
    let s = input.trim_end_matches(['\r', '\n']);
    if let Some(rest) = s.strip_suffix('…') {
        return rest.trim_end_matches(['\r', '\n']);
    }
    if let Some(rest) = s.strip_suffix("...") {
        return rest.trim_end_matches(['\r', '\n']);
    }
    input
}

/// Trim `input` to at most `max_words` whitespace tokens while preserving newlines
/// and blank-line structure (T219 F1/F2). When truncation occurs, append a trailing
/// `…` on its own line (F2b); the sentinel is not counted toward the budget.
pub fn trim_to_word_budget(input: &str, max_words: usize) -> String {
    if max_words == 0 {
        return String::new();
    }
    let total_words = word_count(input);
    let mut out = String::new();
    let mut used = 0usize;
    let mut first_line = true;
    for line in input.split('\n') {
        let line = line.trim_end_matches('\r');
        if used >= max_words {
            break;
        }
        if !first_line {
            out.push('\n');
        }
        first_line = false;
        if line.is_empty() {
            // Preserve blank lines (paragraph structure) while budget remains.
            continue;
        }
        let mut parts: Vec<&str> = Vec::new();
        for tok in line.split_whitespace() {
            if used >= max_words {
                break;
            }
            parts.push(tok);
            used += 1;
        }
        out.push_str(&parts.join(" "));
    }
    if total_words > max_words && !out.ends_with('…') && !out.ends_with("...") {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('…');
    }
    out
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    /// AC1: multi-line under budget preserves `\n` / blank structure (not space-joined).
    #[test]
    fn trim_to_word_budget__multiline_under_budget__preserves_newlines() {
        let input = "a\n\nb c";
        let out = trim_to_word_budget(input, 10);
        assert_eq!(out, "a\n\nb c");
        assert!(out.contains('\n'));
        assert_ne!(out, "a b c", "must not space-join into single line");
        assert!(!out.ends_with('…'), "under-budget must not append sentinel");
    }

    /// AC2: over-budget multi-line truncates with structure, not single-line flatten.
    #[test]
    fn trim_to_word_budget__multiline_over_budget__keeps_structure_until_cut() {
        let input = "one two\n\nthree four five six";
        let out = trim_to_word_budget(input, 3);
        assert!(
            out.contains('\n'),
            "must retain newline structure, got {out:?}"
        );
        // First line fully kept; second line starts then cuts mid-line at word 3.
        assert!(out.starts_with("one two\n"), "got {out:?}");
        assert_eq!(content_word_count(&out), 3);
        assert!(out.ends_with('…'), "F2b sentinel required when truncated");
        assert_ne!(out, "one two three", "must not space-join into single line");
    }

    /// AC15: over-budget ends with `…`; under-budget does not.
    #[test]
    fn trim_to_word_budget__over_budget__appends_ellipsis_sentinel() {
        let over = trim_to_word_budget("alpha beta gamma delta", 2);
        assert!(over.ends_with('…'), "got {over:?}");
        assert_eq!(content_word_count(&over), 2);

        let under = trim_to_word_budget("alpha beta", 10);
        assert!(!under.ends_with('…'));
        assert!(!under.ends_with("..."));
        assert_eq!(under, "alpha beta");
    }

    /// AC16 / F32: content words == max and result contains newline when input did.
    #[test]
    fn trim_to_word_budget__invariant__content_words_eq_max_with_newline() {
        let result = trim_to_word_budget("a b c\n\nd e f", 3);
        assert!(
            result.contains('\n'),
            "must preserve newline, got {result:?}"
        );
        assert_eq!(content_word_count(&result), 3);
        // raw word_count may include sentinel chrome; content_word_count excludes it
        assert!(result.ends_with('…'));
    }

    /// AC17: CRLF input yields `\n` structure without stray `\r` tokens.
    #[test]
    fn trim_to_word_budget__crlf__strips_cr_preserves_structure() {
        let input = "line one\r\n\r\nline two three";
        let out = trim_to_word_budget(input, 10);
        assert!(!out.contains('\r'), "must not leave CR tokens, got {out:?}");
        assert!(out.contains('\n'));
        assert_eq!(out, "line one\n\nline two three");
    }

    #[test]
    fn trim_to_word_budget__max_zero__empty() {
        assert_eq!(trim_to_word_budget("a b c\n\nd", 0), "");
        assert_eq!(content_word_count(""), 0);
    }

    #[test]
    fn trim_to_word_budget__under_budget_no_sentinel() {
        let out = trim_to_word_budget("hello world\n\nmore text here", 100);
        assert_eq!(out, "hello world\n\nmore text here");
        assert!(!out.ends_with('…'));
        assert_eq!(word_count(&out), 5);
    }

    #[test]
    fn content_word_count__excludes_trailing_sentinel() {
        assert_eq!(content_word_count("a b c\n…"), 3);
        assert_eq!(content_word_count("a b c\n..."), 3);
        assert_eq!(content_word_count("a b c"), 3);
    }

    #[test]
    fn word_count__split_whitespace_semantics() {
        assert_eq!(word_count("  a   b\nc  "), 3);
        assert_eq!(word_count(""), 0);
    }
}
