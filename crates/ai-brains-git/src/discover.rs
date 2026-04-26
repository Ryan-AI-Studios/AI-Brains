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
