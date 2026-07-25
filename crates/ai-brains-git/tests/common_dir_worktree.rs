//! Worktree common-dir discovery (R-GIT3).

#![allow(non_snake_case)]

mod common;

use ai_brains_git::{collect_metadata, discover_common_dir};
use std::path::Path;

fn paths_equal_canonical(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

#[test]
fn discover_common_dir__two_worktrees_same_repo__equal_common_dir()
-> Result<(), Box<dyn std::error::Error>> {
    let root = common::init_repo("common-dir-main")?;
    common::commit_file(&root, "README.md", "hello\n", "initial")?;

    let worktree = common::unique_temp_dir("common-dir-wt");
    // `git worktree add <path>` creates a linked worktree sharing the common git dir.
    common::run_git(
        &root,
        &[
            "worktree",
            "add",
            worktree
                .to_str()
                .ok_or("worktree path is not valid UTF-8")?,
            "-b",
            "wt-branch",
        ],
    )?;

    let main_meta = collect_metadata(&root)?;
    let wt_meta = collect_metadata(&worktree)?;

    let main_common = main_meta
        .common_dir
        .as_ref()
        .ok_or("main worktree missing common_dir")?;
    let wt_common = wt_meta
        .common_dir
        .as_ref()
        .ok_or("linked worktree missing common_dir")?;

    assert!(
        paths_equal_canonical(main_common, wt_common),
        "main common_dir={main_common:?} worktree common_dir={wt_common:?}"
    );

    // discover_common_dir API should agree with metadata field.
    let discovered_main = discover_common_dir(&root)?.ok_or("discover_common_dir main")?;
    let discovered_wt = discover_common_dir(&worktree)?.ok_or("discover_common_dir wt")?;
    assert!(paths_equal_canonical(&discovered_main, &discovered_wt));
    assert!(paths_equal_canonical(&discovered_main, main_common));

    // Cleanup: remove worktree registration then dirs.
    let _ = common::run_git(
        &root,
        &[
            "worktree",
            "remove",
            "--force",
            worktree
                .to_str()
                .ok_or("worktree path is not valid UTF-8")?,
        ],
    );
    let _ = std::fs::remove_dir_all(&worktree);
    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
