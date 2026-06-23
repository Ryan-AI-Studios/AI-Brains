mod common;

use ai_brains_core::ids::ProjectId;
use ai_brains_retrieval::substring_fallback;

/// T105: The substring fallback must be skipped when the scoped project has
/// more than 10,000 pinned memories to avoid expensive full-table scans.
#[test]
#[allow(non_snake_case)]
fn recall__substring_fallback_skipped_above_10k_threshold() -> Result<(), Box<dyn std::error::Error>>
{
    let project_id = ProjectId::new();
    let store = common::store_with_project_id(
        "needle in a large haystack",
        ai_brains_core::privacy::Privacy::CloudOk,
        project_id,
    )?;

    // Seed 10,001 pinned memories directly for this project.
    common::insert_pinned_memories_direct(&store, project_id, 10_001)?;

    // FTS5 won't match "needle haystack" as a substring; the fallback should
    // refuse to run because the project exceeds the 10,000 memory guard.
    let hits = substring_fallback(
        store.connection(),
        "needle haystack",
        Some(project_id),
        None,
        5,
    )?;

    assert!(
        hits.is_empty(),
        "substring fallback must be skipped for projects with >10,000 memories"
    );
    Ok(())
}
