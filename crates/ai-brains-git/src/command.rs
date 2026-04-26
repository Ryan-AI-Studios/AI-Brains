use crate::errors::{GitError, Result};
use std::path::Path;
use std::process::Command;

pub fn run_git(path: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git").args(args).current_dir(path).output()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let trimmed = stdout.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(GitError::CommandFailed {
            command: format!("git {}", args.join(" ")),
            message: stderr.trim().to_string(),
        })
    }
}
