use thiserror::Error;

pub type Result<T> = std::result::Result<T, RetrievalError>;

#[derive(Debug, Error)]
pub enum RetrievalError {
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("store error: {0}")]
    Store(#[from] ai_brains_store::StoreError),
}
