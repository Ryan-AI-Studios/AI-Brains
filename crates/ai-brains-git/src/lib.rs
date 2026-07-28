mod branch;
mod command;
mod commit;
mod diffstat;
mod discover;
mod errors;
mod remote;
mod status;

use std::path::Path;
use std::path::PathBuf;

pub use command::{
    DEFAULT_GIT_TIMEOUT, DEFAULT_GIT_TIMEOUT_MS, ENV_GCM_INTERACTIVE, ENV_GIT_ASKPASS,
    ENV_GIT_TERMINAL_PROMPT, ENV_SSH_ASKPASS_REQUIRE, apply_git_automation_env,
    automation_env_pairs, git_askpass_noop_program, git_command, run_git, run_git_timeout,
};
pub use diffstat::DiffStat;
pub use discover::discover_common_dir;
pub use errors::{GitError, Result};
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

/// Collect repository metadata using the default git command timeout
/// ([`DEFAULT_GIT_TIMEOUT`], 5000 ms).
///
/// Every underlying `git` spawn (via [`run_git`]) applies non-interactive env
/// guards and the default deadline. See the `command` module docs for
/// hang-prevention design (env guards primary; kill is direct-child only).
pub fn collect_metadata(path: &Path) -> Result<GitMetadata> {
    let Some(root) = discover::discover_root(path)? else {
        return Ok(GitMetadata::default());
    };

    let status = status::read_status(&root)?;
    let remote = remote::read_remote_selection(&root)?;
    Ok(GitMetadata {
        root: Some(root.clone()),
        branch: branch::read_branch(&root)?,
        commit: commit::read_commit(&root)?,
        remote_url_hash: remote.hash,
        remote_names: remote.remote_names,
        common_dir: discover::discover_common_dir(&root)?,
        is_dirty: status.is_dirty,
        untracked_files: status.untracked_files,
        diffstat: diffstat::read_diffstat(&root)?,
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
}
