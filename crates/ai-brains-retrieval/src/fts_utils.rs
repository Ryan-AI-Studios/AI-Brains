/// Sanitize a query string for safe use with SQLite FTS5 or ChangeGuard search.
///
/// Wraps each whitespace-delimited token in double-quotes so FTS5 treats them
/// as phrase literals rather than operator syntax. This prevents syntax errors
/// from special characters: `.`, `*`, `(`, `)`, `:`, `^`, `-`.
/// Internal double-quote characters are escaped by doubling them (FTS5 spec §4.2).
pub fn sanitize_fts_query(query: &str) -> String {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|token| {
            let escaped = token.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        })
        .collect();
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_dot_wrapped() {
        assert_eq!(sanitize_fts_query("context.rs"), r#""context.rs""#);
    }

    #[test]
    fn parentheses_wrapped() {
        assert_eq!(
            sanitize_fts_query("some.method(arg)"),
            r#""some.method(arg)""#
        );
    }

    #[test]
    fn asterisk_wrapped() {
        assert_eq!(sanitize_fts_query("foo*"), r#""foo*""#);
    }

    #[test]
    fn mixed_query() {
        assert_eq!(
            sanitize_fts_query("context.rs brittle hotspot"),
            r#""context.rs" "brittle" "hotspot""#
        );
    }

    #[test]
    fn empty_query_returns_empty() {
        assert_eq!(sanitize_fts_query(""), "");
    }

    #[test]
    fn internal_double_quotes_escaped() {
        assert_eq!(sanitize_fts_query("say \"hello\""), r#""say" """hello""""#);
    }
}
