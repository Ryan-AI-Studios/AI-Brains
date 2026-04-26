mod common;

use ai_brains_git::{collect_metadata, max_untracked_files};

#[test]
fn untracked_filenames_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("untracked-bounded")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;

    for index in 0..(max_untracked_files() + 5) {
        std::fs::write(root.join(format!("scratch-{index}.txt")), "x\n")?;
    }

    let metadata = collect_metadata(&root)?;

    assert_eq!(metadata.untracked_files.len(), max_untracked_files());
    assert!(metadata
        .untracked_files
        .iter()
        .all(|name| name.starts_with("scratch-")));

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
