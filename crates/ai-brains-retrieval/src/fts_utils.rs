/// Sanitize a query string for safe use with SQLite FTS5 or Ledgerful search.
///
/// Extracts alphanumeric/underscore runs and wraps each token in double-quotes
/// so FTS5 treats them as phrase literals rather than operator syntax. This
/// prevents syntax errors from punctuation-heavy natural language.
pub fn sanitize_fts_query(query: &str) -> String {
    let tokens: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"", token))
        .collect();
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_dot_wrapped() {
        assert_eq!(sanitize_fts_query("context.rs"), r#""context" "rs""#);
    }

    #[test]
    fn parentheses_wrapped() {
        assert_eq!(
            sanitize_fts_query("some.method(arg)"),
            r#""some" "method" "arg""#
        );
    }

    #[test]
    fn asterisk_wrapped() {
        assert_eq!(sanitize_fts_query("foo*"), r#""foo""#);
    }

    #[test]
    fn mixed_query() {
        assert_eq!(
            sanitize_fts_query("context.rs brittle hotspot"),
            r#""context" "rs" "brittle" "hotspot""#
        );
    }

    #[test]
    fn empty_query_returns_empty() {
        assert_eq!(sanitize_fts_query(""), "");
    }

    #[test]
    fn internal_double_quotes_escaped() {
        assert_eq!(sanitize_fts_query("say \"hello\""), r#""say" "hello""#);
    }

    #[test]
    fn comma_separated_prompt_is_tokenized() {
        assert_eq!(
            sanitize_fts_query("bridge error: fts5, syntax near comma"),
            r#""bridge" "error" "fts5" "syntax" "near" "comma""#
        );
    }
}
