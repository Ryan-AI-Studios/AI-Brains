mod common;

use ai_brains_git::collect_metadata;

#[test]
fn remote_url_hash_stable() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("remote-hash")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;
    common::run_git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            "https://example.com/org/repo.git",
        ],
    )?;

    let left = collect_metadata(&root)?;
    let right = collect_metadata(&root)?;

    assert_eq!(left.remote_url_hash, right.remote_url_hash);
    assert_ne!(
        left.remote_url_hash.as_deref(),
        Some("https://example.com/org/repo.git")
    );

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
