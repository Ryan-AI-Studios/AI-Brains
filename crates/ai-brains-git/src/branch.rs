use crate::command::run_git;
use crate::errors::Result;
use std::path::Path;

pub fn read_branch(root: &Path) -> Result<Option<String>> {
    match run_git(root, &["branch", "--show-current"]) {
        Ok(branch) => Ok(branch),
        Err(_) => Ok(None),
    }
}
