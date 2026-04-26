mod common;

use ai_brains_git::collect_metadata;

#[test]
fn diffstat_does_not_capture_full_diff() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("diffstat-bounded")?;
    common::commit_file(&root, "README.md", "before\n", "initial")?;
    let secret_text = "this content should never appear in diffstat output";
    std::fs::write(root.join("README.md"), format!("before\n{secret_text}\n"))?;

    let metadata = collect_metadata(&root)?;
    let diffstat = metadata
        .diffstat
        .ok_or("diffstat should be present when repo is dirty")?;

    assert!(diffstat.files_changed >= 1);
    assert!(!diffstat.summary.contains(secret_text));

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
