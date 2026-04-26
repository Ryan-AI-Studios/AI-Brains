mod common;

use ai_brains_git::collect_metadata;

#[test]
fn git_root_discovered() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("root-discovered")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    let nested = root.join("src").join("nested");
    std::fs::create_dir_all(&nested)?;

    let metadata = collect_metadata(&nested)?;

    assert_eq!(metadata.root.as_deref(), Some(root.as_path()));
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
