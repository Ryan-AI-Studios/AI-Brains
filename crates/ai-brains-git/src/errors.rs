use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git command failed for {command}: {message}")]
    CommandFailed { command: String, message: String },
    #[error("utf-8 decode failed: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path is not absolute: {0}")]
    NonAbsolutePath(PathBuf),
    #[error("failed to parse diffstat output: {0}")]
    DiffstatParse(String),
}
