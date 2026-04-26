use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Vault is locked or incorrect key provided: {0}")]
    VaultLocked(String),

    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Failed to append event: {0}")]
    EventAppendFailed(String),

    #[error("Failed to read events: {0}")]
    EventReadFailed(String),

    #[error("Immutable event modified: {0}")]
    ImmutableEventModified(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;
