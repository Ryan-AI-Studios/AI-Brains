mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{LexicalSearchOptions, lexical_search};

#[test]
fn lexical_search_returns_memory() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::store_with_memory(
        "architectural nuance about event sourcing and immutability",
        Privacy::CloudOk,
    )?;

    let results = lexical_search(
        store.connection(),
        "architectural nuance",
        None,
        None,
        LexicalSearchOptions::default(),
    )?;
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("event sourcing"));
    Ok(())
}
