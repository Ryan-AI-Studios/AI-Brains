use crate::MAX_UNTRACKED_FILES;
use crate::command::run_git;
use crate::errors::Result;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepoStatus {
    pub is_dirty: bool,
    pub untracked_files: Vec<String>,
}

pub fn read_status(root: &Path) -> Result<RepoStatus> {
    let output = match run_git(root, &["status", "--porcelain", "--untracked-files=all"]) {
        Ok(Some(output)) => output,
        Ok(None) => return Ok(RepoStatus::default()),
        Err(_) => return Ok(RepoStatus::default()),
    };

    let mut is_dirty = false;
    let mut untracked_files = Vec::new();

    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        is_dirty = true;
        if let Some(path) = line.strip_prefix("?? ")
            && untracked_files.len() < MAX_UNTRACKED_FILES
        {
            untracked_files.push(path.to_string());
        }
    }

    Ok(RepoStatus {
        is_dirty,
        untracked_files,
    })
}
