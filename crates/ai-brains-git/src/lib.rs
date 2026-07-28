mod branch;
mod command;
mod commit;
mod diffstat;
mod discover;
mod errors;
mod policy;
mod remote;
mod status;

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

pub use command::{
    DEFAULT_GIT_TIMEOUT, DEFAULT_GIT_TIMEOUT_MS, ENV_GCM_INTERACTIVE, ENV_GIT_ASKPASS,
    ENV_GIT_TERMINAL_PROMPT, ENV_SSH_ASKPASS_REQUIRE, apply_git_automation_env,
    automation_env_pairs, git_askpass_noop_program, git_command, run_git, run_git_timeout,
};
pub use diffstat::DiffStat;
pub use discover::{
    discover_common_dir, discover_common_dir_with_options, discover_root,
    discover_root_with_options,
};
pub use errors::{GitError, Result};
pub use policy::{
    GitRunOptions, SoftFailPolicy, discover_error_is_soft, is_not_a_git_repository_error,
};
pub use remote::{
    RemoteHashResult, hash_remote_url, normalize_remote_url, read_remote_selection,
    read_remote_url_hash,
};

const MAX_UNTRACKED_FILES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitMetadata {
    pub root: Option<PathBuf>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub remote_url_hash: Option<String>,
    /// Configured remote names (evidence when hash is ambiguous / multi-remote).
    pub remote_names: Vec<String>,
    /// Shared git directory (`rev-parse --git-common-dir`); same across worktrees.
    pub common_dir: Option<PathBuf>,
    pub is_dirty: bool,
    pub untracked_files: Vec<String>,
    pub diffstat: Option<DiffStat>,
}

impl GitMetadata {
    pub fn is_repository(&self) -> bool {
        self.root.is_some()
    }
}

/// Collect repository metadata using soft-fail policy and the default timeout
/// ([`DEFAULT_GIT_TIMEOUT`], 5000 ms).
///
/// Soft-fail (legacy / scope resolver): genuine non-repo paths and most CLI
/// failures degrade to empty metadata. [`GitError::Timeout`] during discover
/// still propagates. Prefer [`collect_metadata_strict`] for connector paths that
/// must surface hard failures as `Err`.
///
/// Every underlying `git` spawn applies non-interactive env guards and the
/// configured deadline. See the `command` module docs for hang-prevention
/// design (env guards primary; kill is direct-child only).
pub fn collect_metadata(path: &Path) -> Result<GitMetadata> {
    collect_metadata_with_options(path, &GitRunOptions::soft())
}

/// Soft-fail collect with an explicit per-spawn timeout.
pub fn collect_metadata_with_timeout(path: &Path, timeout: Duration) -> Result<GitMetadata> {
    collect_metadata_with_options(path, &GitRunOptions::soft_with_timeout(timeout))
}

/// Strict collect using the default timeout.
///
/// Used by the Git connector: Timeout / Io / non-not-a-repo CommandFailed
/// propagate as `Err`. Genuine not-a-repository outcomes remain
/// `Ok(GitMetadata::default())` (`!is_repository()`).
pub fn collect_metadata_strict(path: &Path) -> Result<GitMetadata> {
    collect_metadata_with_options(path, &GitRunOptions::strict())
}

/// Strict collect with an explicit per-spawn timeout (Git connector options).
pub fn collect_metadata_strict_with_timeout(path: &Path, timeout: Duration) -> Result<GitMetadata> {
    collect_metadata_with_options(path, &GitRunOptions::strict_with_timeout(timeout))
}

/// Collect metadata with explicit [`GitRunOptions`] (timeout + soft-fail policy).
pub fn collect_metadata_with_options(path: &Path, opts: &GitRunOptions) -> Result<GitMetadata> {
    let Some(root) = discover::discover_root_with_options(path, opts)? else {
        return Ok(GitMetadata::default());
    };

    let status = status::read_status_with_options(&root, opts)?;
    let remote = remote::read_remote_selection_with_options(&root, opts)?;
    Ok(GitMetadata {
        root: Some(root.clone()),
        branch: branch::read_branch_with_options(&root, opts)?,
        commit: commit::read_commit_with_options(&root, opts)?,
        remote_url_hash: remote.hash,
        remote_names: remote.remote_names,
        common_dir: discover::discover_common_dir_with_options(&root, opts)?,
        is_dirty: status.is_dirty,
        untracked_files: status.untracked_files,
        diffstat: diffstat::read_diffstat_with_options(&root, opts)?,
    })
}

pub fn max_untracked_files() -> usize {
    MAX_UNTRACKED_FILES
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn collect_metadata__uses_default_timeout_constant() {
        // Production collect path uses `run_git` → `DEFAULT_GIT_TIMEOUT` (5s)
        // on every discover/status/remote/… spawn.
        assert_eq!(DEFAULT_GIT_TIMEOUT_MS, 5_000);
        assert_eq!(DEFAULT_GIT_TIMEOUT.as_millis(), 5_000);
    }

    #[test]
    fn collect_metadata_strict__uses_strict_policy() {
        let opts = GitRunOptions::strict();
        assert_eq!(opts.policy, SoftFailPolicy::Strict);
        assert_eq!(opts.timeout, DEFAULT_GIT_TIMEOUT);

        let soft = GitRunOptions::soft();
        assert_eq!(soft.policy, SoftFailPolicy::Soft);

        let custom = Duration::from_millis(1234);
        assert_eq!(GitRunOptions::strict_with_timeout(custom).timeout, custom);
        assert_eq!(
            GitRunOptions::strict_with_timeout(custom).policy,
            SoftFailPolicy::Strict
        );
    }
}
