use crate::command::run_git_timeout;
use crate::errors::Result;
use crate::policy::{GitRunOptions, or_soft_default};
use std::path::Path;

pub(crate) fn read_commit_with_options(
    root: &Path,
    opts: &GitRunOptions,
) -> Result<Option<String>> {
    match run_git_timeout(root, &["rev-parse", "HEAD"], opts.timeout) {
        Ok(commit) => Ok(commit),
        Err(e) => or_soft_default(Err(e), opts.policy, None),
    }
}
