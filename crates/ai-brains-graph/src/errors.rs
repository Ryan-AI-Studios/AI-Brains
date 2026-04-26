use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Projection error: {0}")]
    ProjectionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Store error: {0}")]
    StoreError(String),
}

pub type Result<T> = std::result::Result<T, GraphError>;
