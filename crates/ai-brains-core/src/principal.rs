use crate::ids::PrincipalId;
use serde::{Deserialize, Serialize};

/// Kind of principal that may act on governed memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PrincipalKind {
    Human,
    Service,
    Agent,
    Other(String),
}

/// Shell identity for event/DTO layers (not a full IAM model).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    pub id: PrincipalId,
    pub kind: PrincipalKind,
    pub display_name: String,
}
