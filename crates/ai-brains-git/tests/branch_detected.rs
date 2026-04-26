mod common;

use ai_brains_git::collect_metadata;

#[test]
fn branch_detected() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("branch-detected")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    common::run_git(&root, &["checkout", "-b", "feature/identity"])?;

    let metadata = collect_metadata(&root)?;

    assert_eq!(metadata.branch.as_deref(), Some("feature/identity"));
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
