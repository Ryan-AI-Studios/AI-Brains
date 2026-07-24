use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::knowledge::EvidenceHandle;

pub const API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingDto {
    pub id: String,
    pub kind: String,
    /// Evidence handles (ids + optional labels), not prose-only body.
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingResponse {
    pub api_version: String,
    pub briefing: BriefingDto,
}

impl BriefingResponse {
    pub fn new(briefing: BriefingDto) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            briefing,
        }
    }
}
