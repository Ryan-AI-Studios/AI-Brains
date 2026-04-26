use crate::command::run_git;
use crate::errors::Result;
use std::path::Path;

pub fn read_commit(root: &Path) -> Result<Option<String>> {
    match run_git(root, &["rev-parse", "HEAD"]) {
        Ok(commit) => Ok(commit),
        Err(_) => Ok(None),
    }
}
