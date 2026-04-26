mod common;

use ai_brains_git::collect_metadata;

#[test]
fn dirty_status_detected() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("dirty-status")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    std::fs::write(root.join("README.md"), "hello\nchanged\n")?;

    let metadata = collect_metadata(&root)?;

    assert!(metadata.is_dirty);
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
