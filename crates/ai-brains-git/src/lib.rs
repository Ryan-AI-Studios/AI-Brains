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

pub use diffstat::DiffStat;
pub use errors::{GitError, Result};

const MAX_UNTRACKED_FILES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitMetadata {
    pub root: Option<PathBuf>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub remote_url_hash: Option<String>,
    pub is_dirty: bool,
    pub untracked_files: Vec<String>,
    pub diffstat: Option<DiffStat>,
}

impl GitMetadata {
    pub fn is_repository(&self) -> bool {
        self.root.is_some()
    }
}

pub fn collect_metadata(path: &Path) -> Result<GitMetadata> {
    let Some(root) = discover::discover_root(path)? else {
        return Ok(GitMetadata::default());
    };

    let status = status::read_status(&root)?;
    Ok(GitMetadata {
        root: Some(root.clone()),
        branch: branch::read_branch(&root)?,
        commit: commit::read_commit(&root)?,
        remote_url_hash: remote::read_remote_url_hash(&root)?,
        is_dirty: status.is_dirty,
        untracked_files: status.untracked_files,
        diffstat: diffstat::read_diffstat(&root)?,
    })
}

pub fn max_untracked_files() -> usize {
    MAX_UNTRACKED_FILES
}
