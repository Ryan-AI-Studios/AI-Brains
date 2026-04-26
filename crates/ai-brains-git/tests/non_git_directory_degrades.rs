mod common;

use ai_brains_git::collect_metadata;

#[test]
fn non_git_directory_degrades() -> Result<(), Box<dyn std::error::Error>> {
    let dir = common::unique_temp_dir("non-git");
    std::fs::create_dir_all(&dir)?;

    let metadata = collect_metadata(&dir)?;

    assert!(!metadata.is_repository());
    assert!(metadata.branch.is_none());
    assert!(metadata.commit.is_none());
    assert!(!metadata.is_dirty);
    assert!(metadata.untracked_files.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
