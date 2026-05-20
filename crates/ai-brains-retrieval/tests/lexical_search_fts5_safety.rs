mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::lexical_search;

#[test]
fn lexical_search_survives_question_mark() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "What is the main purpose of event sourcing?",
        Privacy::CloudOk,
    )?;

    // A natural-language question containing '?' must not trigger an FTS5 syntax error.
    let results = lexical_search(store.connection(), "What is the main purpose?", None, None)?;
    assert!(
        !results.is_empty(),
        "Query with '?' should return results, not crash"
    );
    Ok(())
}

#[test]
fn lexical_search_survives_double_quotes() -> Result<(), Box<dyn std::error::Error>> {
    let store =
        common::store_with_memory("He said hello to the immutable event log", Privacy::CloudOk)?;

    let results = lexical_search(store.connection(), "He said \"hello\"", None, None)?;
    assert!(
        !results.is_empty(),
        "Query with double quotes should return results, not crash"
    );
    Ok(())
}

#[test]
fn lexical_search_survives_asterisk() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "Find all files matching the wildcard pattern",
        Privacy::CloudOk,
    )?;

    // Query containing '*' must not trigger an FTS5 syntax error.
    // We assert on absence of error, not result count, because sanitization
    // may alter the query enough to change ranking.
    let _results = lexical_search(store.connection(), "files matching *pattern", None, None)?;
    Ok(())
}

#[test]
fn lexical_search_survives_only_special_chars() -> Result<(), Box<dyn std::error::Error>> {
    let store =
        common::store_with_memory("some test content about architecture", Privacy::CloudOk)?;

    // Query that sanitizes down to nothing must return empty gracefully, not crash.
    let results = lexical_search(store.connection(), "? * \"", None, None)?;
    assert_eq!(
        results.len(),
        0,
        "Query with only special chars should return empty"
    );
    Ok(())
}

#[test]
fn lexical_search_survives_bare_fts_operators() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "Difference between AND and OR operators in logic",
        Privacy::CloudOk,
    )?;

    let _results = lexical_search(store.connection(), "AND OR NOT NEAR", None, None)?;
    // These are valid FTS5 operators; the query should not crash.
    // We don't assert result count because behavior depends on tokenizer,
    // but it MUST NOT return an error.
    Ok(())
}
