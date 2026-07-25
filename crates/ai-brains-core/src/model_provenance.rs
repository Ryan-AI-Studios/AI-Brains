use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Model/provider/workflow version fields for model-derived evidence and conclusions.
///
/// Additive optional fields (schema v1): never store chain-of-thought.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProvenance {
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_version: Option<String>,
    /// Deployment class when known: `"local"` or `"cloud"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,
    /// Input evidence/memory ids as strings (no CoT or raw tool logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_ids: Option<Vec<String>>,
    /// SHA-256 hex of model output text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
}
