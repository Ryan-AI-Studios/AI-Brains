use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Error)]
pub enum GitError {
    /// Non-zero git process exit.
    ///
    /// `exit_code` is `None` when the process was terminated by a signal (Unix)
    /// or the exit status is otherwise unavailable. Callers that soft-map
    /// specific codes (e.g. `git config --get` exit 1 = missing key) must
    /// treat only the matching code as soft; other codes / `None` stay hard
    /// under [`crate::SoftFailPolicy::Strict`].
    #[error("git command failed for {command}: {message}")]
    CommandFailed {
        command: String,
        message: String,
        /// Process exit code when available (`None` if signal-killed / unknown).
        exit_code: Option<i32>,
    },
    #[error("git command timed out for {command} after {elapsed_ms}ms")]
    Timeout { command: String, elapsed_ms: u64 },
    #[error("utf-8 decode failed: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path is not absolute: {0}")]
    NonAbsolutePath(PathBuf),
    #[error("failed to parse diffstat output: {0}")]
    DiffstatParse(String),
}
