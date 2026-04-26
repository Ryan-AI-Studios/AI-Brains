use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

impl SessionStatus {
    pub fn transition(&self, to: SessionStatus) -> Result<SessionStatus> {
        match (self, to) {
            (SessionStatus::Active, SessionStatus::Paused) => Ok(to),
            (SessionStatus::Active, SessionStatus::Completed) => Ok(to),
            (SessionStatus::Paused, SessionStatus::Active) => Ok(to),
            (SessionStatus::Paused, SessionStatus::Completed) => Ok(to),
            (SessionStatus::Completed, SessionStatus::Archived) => Ok(to),
            _ => Err(Error::InvalidStatusTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", to),
            }),
        }
    }
}
