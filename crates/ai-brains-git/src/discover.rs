use crate::command::run_git_timeout;
use crate::errors::Result;
use crate::policy::{GitRunOptions, SoftFailPolicy, discover_error_is_soft};
use std::path::{Path, PathBuf};

/// Discover the work-tree root using soft-fail policy (legacy / scope resolver).
///
/// Hard failures that soft-map (non-timeout under Soft) become `Ok(None)`.
/// [`GitError::Timeout`] propagates even under Soft for truthful unavailability.
pub fn discover_root(path: &Path) -> Result<Option<PathBuf>> {
    discover_root_with_options(path, &GitRunOptions::default())
}

/// Discover the work-tree root with explicit timeout and soft-fail policy.
pub fn discover_root_with_options(path: &Path, opts: &GitRunOptions) -> Result<Option<PathBuf>> {
    match run_git_timeout(path, &["rev-parse", "--show-toplevel"], opts.timeout) {
        Ok(Some(root)) => Ok(Some(PathBuf::from(root))),
        Ok(None) => Ok(None),
        Err(e) if discover_error_is_soft(&e, opts.policy) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Locate the shared git directory (`git rev-parse --git-common-dir`).
///
/// Linked worktrees report the main repository’s git dir. Relative results are
/// resolved against `path` (the command working directory) and canonicalized
/// when the filesystem allows.
///
/// Uses soft-fail policy (legacy API). Prefer
/// [`discover_common_dir_with_options`] under strict collect.
pub fn discover_common_dir(path: &Path) -> Result<Option<PathBuf>> {
    discover_common_dir_with_options(path, &GitRunOptions::default())
}

/// [`discover_common_dir`] with explicit timeout and soft-fail policy.
pub fn discover_common_dir_with_options(
    path: &Path,
    opts: &GitRunOptions,
) -> Result<Option<PathBuf>> {
    match run_git_timeout(path, &["rev-parse", "--git-common-dir"], opts.timeout) {
        Ok(Some(dir)) => Ok(Some(resolve_git_path(path, &dir))),
        Ok(None) => Ok(None),
        Err(e) => match opts.policy {
            SoftFailPolicy::Soft => Ok(None),
            SoftFailPolicy::Strict => Err(e),
        },
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

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::errors::GitError;
    use crate::policy::{SoftFailPolicy, discover_error_is_soft, is_not_a_git_repository_error};

    #[test]
    fn discover_root_strict__timeout_error__propagates() {
        // Classification used by discover_root_with_options: Timeout is never soft.
        let err = GitError::Timeout {
            command: "git rev-parse --show-toplevel".into(),
            elapsed_ms: 1,
        };
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Strict));
        assert!(!discover_error_is_soft(&err, SoftFailPolicy::Soft));
        assert!(!is_not_a_git_repository_error(&err));
    }
}
