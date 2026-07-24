use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Empty content is not allowed")]
    EmptyContent,

    #[error("Invalid status transition from {from:?} to {to:?}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Approval required for transition: {transition}")]
    ApprovalRequired { transition: String },

    #[error("Source revalidation required for transition from {from} to {to}")]
    RevalidationRequired { from: String, to: String },

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),
}

pub type Result<T> = std::result::Result<T, Error>;
