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

/// Source discovery list (T203).
///
/// **E1:** `items: []` not null when empty. Legacy JSON field `sources` still
/// deserializes via serde alias (M1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceListResponse {
    pub api_version: String,
    #[serde(default, alias = "sources")]
    pub items: Vec<SourceDto>,
    #[serde(default)]
    pub more_available: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl SourceListResponse {
    pub fn new(items: Vec<SourceDto>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            items,
            more_available: false,
            warnings: Vec::new(),
        }
    }

    pub fn with_more(mut self, more_available: bool) -> Self {
        self.more_available = more_available;
        self
    }
}

/// List registered sources for a scope (daemon protocol — T203).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListSourcesRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
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

    #[test]
    fn source_list_response__items_field__serializes() {
        let resp = SourceListResponse::new(vec![]);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"items\""), "serialize uses items: {json}");
        assert!(
            !json.contains("\"sources\""),
            "serialize must not emit sources: {json}"
        );
    }

    #[test]
    fn source_list_response__alias_sources__deserializes() {
        let legacy =
            r#"{"api_version":"1","sources":[{"id":"a","kind":"File","display_name":"n"}]}"#;
        let decoded: SourceListResponse =
            serde_json::from_str(legacy).expect("deserialize alias sources");
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].id, "a");
        assert!(!decoded.more_available);
    }

    #[test]
    fn source_list_response__items_key__deserializes() {
        let modern = r#"{"api_version":"1","items":[],"more_available":true,"warnings":["w"]}"#;
        let decoded: SourceListResponse = serde_json::from_str(modern).expect("deserialize items");
        assert!(decoded.items.is_empty());
        assert!(decoded.more_available);
        assert_eq!(decoded.warnings, vec!["w".to_string()]);
    }

    #[test]
    fn list_sources_request__roundtrip() {
        let req = ListSourcesRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            scope: Some("Repository:00000000-0000-0000-0000-0000000000a1".into()),
            limit: Some(10),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: ListSourcesRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }
}
