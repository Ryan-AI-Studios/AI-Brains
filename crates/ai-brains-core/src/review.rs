use serde::{Deserialize, Serialize};

/// Severity for review-queue items opened by invalidation or policy (T149 R-REV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewCriticality {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// What entity a review item primarily targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReviewSubjectKind {
    Decision,
    Conclusion,
    Source,
    Evidence,
    #[default]
    Other,
}
