use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDto {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceListResponse {
    pub api_version: String,
    pub sources: Vec<SourceDto>,
}

impl SourceListResponse {
    pub fn new(sources: Vec<SourceDto>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            sources,
        }
    }
}
