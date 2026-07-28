//! Erasure request wire surface (T158). Crypto / CE enforcement is P8.

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

/// Request erasure of governed records (ids + reason only — no crypto claims).
///
/// Handlers may accept the request into a queue; content-envelope wipe is out of scope here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestErasureRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Target record / aggregate ids.
    #[serde(default)]
    pub ids: Vec<String>,
    /// Human-readable reason (no secrets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client command / idempotency key. When set, daemon spools and derives
    /// a deterministic ticket `request_id` (uuid v5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
}

/// Acknowledgement that an erasure request was accepted (not that wipe completed).
///
/// **E1:** `warnings: []` when none; `status` is a queue/accept state, not a crypto proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErasureAcceptedResponse {
    pub api_version: String,
    /// Request / ticket id for tracking.
    pub request_id: String,
    /// e.g. `accepted`, `queued`
    pub status: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl ErasureAcceptedResponse {
    pub fn new(request_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            request_id: request_id.into(),
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
    fn request_erasure_request__roundtrip() {
        let req = RequestErasureRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            ids: vec!["agg-1".into()],
            reason: Some("user request".into()),
            scope: Some("Personal:00000000-0000-0000-0000-0000000000u1".into()),
            command_id: Some("erase-cmd-1".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: RequestErasureRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn request_erasure_request__command_id_optional() {
        let decoded: RequestErasureRequest =
            serde_json::from_str(r#"{"api_version":"1","ids":["a"]}"#).expect("deserialize");
        assert!(decoded.command_id.is_none());
    }
}
