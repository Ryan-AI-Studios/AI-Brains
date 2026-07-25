use crate::command::run_git;
use crate::errors::Result;
use std::path::{Path, PathBuf};

pub fn discover_root(path: &Path) -> Result<Option<PathBuf>> {
    match run_git(path, &["rev-parse", "--show-toplevel"]) {
        Ok(Some(root)) => Ok(Some(PathBuf::from(root))),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

/// Locate the shared git directory (`git rev-parse --git-common-dir`).
///
/// Linked worktrees report the main repository’s git dir. Relative results are
/// resolved against `path` (the command working directory) and canonicalized
/// when the filesystem allows.
pub fn discover_common_dir(path: &Path) -> Result<Option<PathBuf>> {
    match run_git(path, &["rev-parse", "--git-common-dir"]) {
        Ok(Some(dir)) => Ok(Some(resolve_git_path(path, &dir))),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

fn resolve_git_path(base: &Path, reported: &str) -> PathBuf {
    let candidate = {
        let p = PathBuf::from(reported);
        if p.is_absolute() { p } else { base.join(p) }
    };

    match candidate.canonicalize() {
        Ok(abs) => abs,
        Err(_) => candidate,
    }
}
