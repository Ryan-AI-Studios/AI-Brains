use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControlPlaneError {
    #[error("event append failed: {0}")]
    EventAppend(String),

    #[error("query failed: {0}")]
    Query(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("fingerprint failed: {0}")]
    Fingerprint(String),

    #[error("clock error: {0}")]
    Clock(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),
}

pub type Result<T> = std::result::Result<T, ControlPlaneError>;
