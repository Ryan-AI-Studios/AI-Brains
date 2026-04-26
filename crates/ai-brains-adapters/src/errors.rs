use thiserror::Error;

pub type Result<T> = std::result::Result<T, AdapterError>;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
}
