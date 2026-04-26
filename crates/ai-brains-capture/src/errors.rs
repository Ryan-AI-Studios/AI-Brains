use thiserror::Error;

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("unsupported role: {0}")]
    UnsupportedRole(String),
    #[error("user prompt content cannot be empty")]
    EmptyPrompt,
    #[error("assistant final content cannot be empty unless status-only")]
    EmptyFinal,
    #[error("session stop reason is required for failed status")]
    MissingFailureReason,
    #[error("event build failed: {0}")]
    Event(#[from] ai_brains_events::EventError),
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("git metadata failed: {0}")]
    Git(#[from] ai_brains_git::GitError),
}
