//! Review queue wire DTOs (list / resolve).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItemDto {
    pub id: String,
    pub subject: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opened_at: Option<DateTime<Utc>>,
}

/// Review queue listing.
///
/// **E1:** `items: []` not null when empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQueueResponse {
    pub api_version: String,
    #[serde(default)]
    pub items: Vec<ReviewItemDto>,
}

impl ReviewQueueResponse {
    pub fn new(items: Vec<ReviewItemDto>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            items,
        }
    }
}

/// List open (or filtered) review items.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListReviewItemsRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Optional status filter (e.g. `Open`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl Default for ListReviewItemsRequest {
    fn default() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            principal_id: None,
            scope: None,
            status: None,
        }
    }
}

/// Resolve (approve / dismiss / defer) a review item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolveReviewItemRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    pub id: String,
    /// e.g. `approved`, `dismissed`, `deferred`
    pub resolution: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Governing scope identity key for grant lookup (required by control-plane when resolving).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client command / idempotency key for spool + replay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
}

/// Outcome of resolving a review item.
///
/// **E1:** `warnings: []` when none.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewResolvedResponse {
    pub api_version: String,
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl ReviewResolvedResponse {
    pub fn new(id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            id: id.into(),
            status: status.into(),
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn list_review_items_request__roundtrip() {
        let req = ListReviewItemsRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            scope: None,
            status: Some("Open".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: ListReviewItemsRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn review_queue_response__e1_empty_items() {
        let resp = ReviewQueueResponse::new(Vec::new());
        let v = serde_json::to_value(&resp).expect("serialize");
        assert!(v.get("items").unwrap().is_array());
        assert_eq!(v.get("items").unwrap().as_array().unwrap().len(), 0);
    }
}
