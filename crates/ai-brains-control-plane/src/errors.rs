use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControlPlaneError {
    #[error("event append failed: {0}")]
    EventAppend(String),

    #[error("query failed: {0}")]
    Query(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("approval required: {0}")]
    ApprovalRequired(String),

    #[error("unsupported conclusion cannot be confirmed: {0}")]
    UnsupportedCannotConfirm(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid transition: {0}")]
    InvalidTransition(String),

    #[error("fingerprint failed: {0}")]
    Fingerprint(String),

    #[error("clock error: {0}")]
    Clock(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    /// Repository identity would create a second project for the same normalized remote.
    #[error("repository identity conflict: {0}")]
    IdentityConflict(String),

    /// CE wipe refused: no live `content_key_store` row (E1 — not envelope-backed).
    #[error("not envelope-backed: {0}")]
    NotEnvelopeBacked(String),
}

pub type Result<T> = std::result::Result<T, ControlPlaneError>;
