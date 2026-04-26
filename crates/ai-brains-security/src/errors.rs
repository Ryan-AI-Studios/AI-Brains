use thiserror::Error;

pub type Result<T> = std::result::Result<T, SecurityError>;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("regex initialization failed")]
    PatternInit,
}
