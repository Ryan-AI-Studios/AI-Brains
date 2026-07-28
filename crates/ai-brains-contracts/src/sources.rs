//! Source registry wire DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

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

/// Inspect a registered source by id (daemon protocol — T158).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InspectSourceRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn inspect_source_request__roundtrip() {
        let req = InspectSourceRequest {
            api_version: API_VERSION.to_string(),
            id: "src-1".into(),
            principal_id: None,
            scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: InspectSourceRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }
}
