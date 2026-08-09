//! FTS5 query helpers shared by retrieval and control-plane discovery reads.
//!
//! Token split aligns with the default `unicode61` tokenizer (non-alphanumeric
//! separators **including** `_`). Match expressions use only quoted phrase
//! literals plus literal ` OR ` so operators cannot be injected from user input.

/// Hard cap for SQL `LIMIT` on every lexical MATCH (T217 D13).
pub const LEXICAL_MATCH_HARD_CAP: usize = 200;

/// Literal English stopwords for contentful filtering (T217 §4.1).
/// Case-insensitive. Negators (`not`/`no`/`never`/…) are intentionally absent.
const ENGLISH_STOPWORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
    "can", "need", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into",
    "through", "during", "before", "after", "above", "below", "between", "out", "off", "over",
    "under", "again", "further", "then", "once", "here", "there", "when", "where", "why", "how",
    "all", "each", "few", "more", "most", "other", "some", "such", "only", "own", "same", "so",
    "than", "too", "very", "just", "now", "what", "which", "who", "whom", "this", "that", "these",
    "those", "am", "i", "me", "my", "we", "our", "you", "your", "he", "she", "it", "they", "them",
    "their", "and", "or", "but", "if", "because", "until", "while", "about",
];

/// Extract FTS tokens by splitting on non-alphanumeric characters (including `_`).
///
/// Aligns with SQLite FTS5 `unicode61` indexing so `ai_brains_core` becomes
/// three tokens rather than one underscore-joined phrase.
pub fn extract_fts_tokens(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

/// True when `token` is in the fixed English stopword set (case-insensitive).
///
/// Negators (`not`, `no`, `never`, …) are **not** stopwords.
pub fn is_english_stopword(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    ENGLISH_STOPWORDS.contains(&lower.as_str())
}

/// Contentful tokens: not stopwords, length ≥ 2, first-seen order, deduped
/// case-insensitively while preserving the first spelling.
pub fn contentful_tokens(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen_lower = std::collections::HashSet::new();
    for token in tokens {
        if token.len() < 2 || is_english_stopword(token) {
            continue;
        }
        let key = token.to_ascii_lowercase();
        if seen_lower.insert(key) {
            out.push(token.clone());
        }
    }
    out
}

/// Build an FTS5 AND expression: quoted tokens joined by space (implicit AND).
pub fn match_and(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build an FTS5 OR expression: quoted tokens joined by ` OR `.
pub fn match_or(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// Select up to 8 contentful tokens for OR rescue: longer first, then lexical asc.
pub fn select_or_tokens(contentful: &[String]) -> Vec<String> {
    let mut selected = contentful.to_vec();
    selected.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()))
            .then_with(|| a.cmp(b))
    });
    selected.truncate(8);
    selected
}

/// Sanitize a query string for safe use with SQLite FTS5 or Ledgerful search.
///
/// Extracts alphanumeric runs (splitting `_`) and wraps each token in double-quotes
/// so FTS5 treats them as phrase literals rather than operator syntax. Equivalent to
/// `match_and(extract_fts_tokens(query))`.
pub fn sanitize_fts_query(query: &str) -> String {
    match_and(&extract_fts_tokens(query))
}

/// Whether an empty-recall hint should append “try fewer keywords” (T217 D7).
///
/// True when raw token count ≥ 3 and at least one contentful token remains.
/// All-stopword multi-token queries return false.
pub fn should_suggest_fewer_keywords(query: &str) -> bool {
    let tokens = extract_fts_tokens(query);
    tokens.len() >= 3 && !contentful_tokens(&tokens).is_empty()
}

#[cfg(test)]
#[allow(non_snake_case)] // TDD names use __ separators
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

    // --- T217 pure helpers ---

    #[test]
    fn extract_fts_tokens__underscore_split__three_tokens() {
        // AC16 / D14: unicode61 alignment
        assert_eq!(
            extract_fts_tokens("ai_brains_core"),
            vec!["ai", "brains", "core"]
        );
        assert_eq!(
            sanitize_fts_query("ai_brains_core"),
            r#""ai" "brains" "core""#
        );
    }

    #[test]
    fn extract_fts_tokens__mixed_separators__splits() {
        assert_eq!(
            extract_fts_tokens("what did we decide about forget list"),
            vec!["what", "did", "we", "decide", "about", "forget", "list"]
        );
    }

    #[test]
    fn is_english_stopword__common_stopwords__true() {
        assert!(is_english_stopword("what"));
        assert!(is_english_stopword("DID"));
        assert!(is_english_stopword("About"));
        assert!(is_english_stopword("the"));
        assert!(is_english_stopword("we"));
    }

    #[test]
    fn is_english_stopword__negators__false() {
        // AC6b / F22: keep negators contentful
        assert!(!is_english_stopword("not"));
        assert!(!is_english_stopword("no"));
        assert!(!is_english_stopword("never"));
        assert!(!is_english_stopword("nor"));
        assert!(!is_english_stopword("neither"));
        assert!(!is_english_stopword("none"));
        assert!(!is_english_stopword("nobody"));
        assert!(!is_english_stopword("nothing"));
        assert!(!is_english_stopword("nowhere"));
    }

    #[test]
    fn is_english_stopword__content_words__false() {
        assert!(!is_english_stopword("forget"));
        assert!(!is_english_stopword("list"));
        assert!(!is_english_stopword("decide"));
    }

    #[test]
    fn contentful_tokens__stopwords_dropped_negators_kept() {
        // AC6 / AC6b
        let tokens = extract_fts_tokens("what not to forget");
        let c = contentful_tokens(&tokens);
        assert!(c.iter().any(|t| t.eq_ignore_ascii_case("not")));
        assert!(c.iter().any(|t| t.eq_ignore_ascii_case("forget")));
        assert!(!c.iter().any(|t| t.eq_ignore_ascii_case("what")));
        assert!(!c.iter().any(|t| t.eq_ignore_ascii_case("to")));
    }

    #[test]
    fn contentful_tokens__min_length_two__drops_single_char() {
        let tokens = vec!["a".into(), "t".into(), "ok".into(), "s".into()];
        assert_eq!(contentful_tokens(&tokens), vec!["ok"]);
    }

    #[test]
    fn contentful_tokens__dedupe_preserves_first_seen_order() {
        // AC17
        let tokens = vec!["forget".into(), "list".into(), "forget".into()];
        assert_eq!(contentful_tokens(&tokens), vec!["forget", "list"]);
    }

    #[test]
    fn contentful_tokens__case_insensitive_dedupe() {
        let tokens = vec!["Forget".into(), "list".into(), "forget".into()];
        assert_eq!(contentful_tokens(&tokens), vec!["Forget", "list"]);
    }

    #[test]
    fn match_and__quoted_space_joined() {
        let tokens = vec!["forget".into(), "list".into()];
        assert_eq!(match_and(&tokens), r#""forget" "list""#);
    }

    #[test]
    fn match_or__quoted_or_joined() {
        // AC5
        let tokens = vec!["forget".into(), "list".into()];
        assert_eq!(match_or(&tokens), r#""forget" OR "list""#);
    }

    #[test]
    fn match_or__user_or_literal_is_tokenized_not_operator() {
        // AC5: user "foo OR bar" becomes quoted tokens, not bare OR operator injection
        let tokens = extract_fts_tokens("foo OR bar");
        let expr = match_and(&tokens);
        assert_eq!(expr, r#""foo" "OR" "bar""#);
        assert!(!expr.contains(" OR "));
    }

    #[test]
    fn select_or_tokens__cap_eight_length_desc_then_lexical() {
        let contentful: Vec<String> = vec![
            "aa".into(),
            "bbb".into(),
            "cccc".into(),
            "dd".into(),
            "eee".into(),
            "ffff".into(),
            "gg".into(),
            "hhh".into(),
            "iiii".into(),
            "jj".into(),
        ];
        let selected = select_or_tokens(&contentful);
        assert_eq!(selected.len(), 8);
        // Longest first (len 4 lexical asc): cccc, ffff, iiii; then len 3; then len 2
        assert_eq!(
            selected,
            vec!["cccc", "ffff", "iiii", "bbb", "eee", "hhh", "aa", "dd"]
        );
    }

    #[test]
    fn select_or_tokens__same_length__lexical_asc() {
        let contentful = vec!["zebra".into(), "apple".into(), "mango".into()];
        let selected = select_or_tokens(&contentful);
        assert_eq!(selected, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn should_suggest_fewer_keywords__multi_token_contentful__true() {
        assert!(should_suggest_fewer_keywords(
            "what did we decide about forget list"
        ));
    }

    #[test]
    fn should_suggest_fewer_keywords__all_stopword__false() {
        // AC7b gate
        assert!(!should_suggest_fewer_keywords("what did we do about this"));
    }

    #[test]
    fn should_suggest_fewer_keywords__two_token__false() {
        assert!(!should_suggest_fewer_keywords("forget list"));
    }

    #[test]
    fn sanitize_fts_query__equals_match_and_extract() {
        let q = "what_about forget-list?";
        assert_eq!(sanitize_fts_query(q), match_and(&extract_fts_tokens(q)));
    }
}
