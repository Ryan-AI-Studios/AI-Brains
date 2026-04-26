mod common;

use ai_brains_git::collect_metadata;

#[test]
fn commit_detected() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("commit-detected")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    let expected = common::run_git(&root, &["rev-parse", "HEAD"])?;

    let metadata = collect_metadata(&root)?;

    assert_eq!(metadata.commit.as_deref(), Some(expected.as_str()));
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
