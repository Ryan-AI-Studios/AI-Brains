mod common;

use ai_brains_core::privacy::Privacy;
use ai_brains_retrieval::{RecallOptions, recall};

/// T105: When FTS5 returns zero results, recall should fall back to a
/// substring LIKE scan and return the matching memory with source="substring".
#[test]
#[allow(non_snake_case)]
fn recall__fts5_empty__substring_fallback_finds_match() -> Result<(), Box<dyn std::error::Error>> {
    let store =
        common::store_with_memory("hello world from the retrieval subsystem", Privacy::CloudOk)?;

    // "llo worl" is a true substring of "hello world" but does not match
    // any FTS5 token, so FTS5 returns empty. The substring fallback should find it.
    let hits = recall(
        store.connection(),
        None,
        "llo worl",
        5,
        RecallOptions {
            project_id: None,
            session_id: None,
            semantic: false,
            graph_boost: 0.0,
            graph_hop_depth: 0,
            quiet: true,
            no_bridge: true,
        },
    )?;

    assert_eq!(hits.len(), 1, "substring fallback should find one memory");
    assert_eq!(hits[0].source, "substring");
    assert!(
        hits[0].content.contains("hello world"),
        "fallback result should contain the original content"
    );
    Ok(())
}
