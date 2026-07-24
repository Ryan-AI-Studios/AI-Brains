use serde::{Deserialize, Serialize};

/// Lifecycle status of an evidence item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EvidenceStatus {
    Active,
    Superseded,
    Unavailable,
    Erased,
}
