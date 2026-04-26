use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Hash mismatch: expected {expected}, found {found}")]
    HashMismatch { expected: String, found: String },

    #[error("Invalid event kind: {0}")]
    InvalidEventKind(String),

    #[error("Unknown version: {0}")]
    UnknownVersion(u32),

    #[error("Upcast failed: {0}")]
    UpcastFailed(String),
}
