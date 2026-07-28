use serde::{Deserialize, Serialize};

use crate::response::ApiError;

pub const API_VERSION: &str = "1";
pub const POLICY_DENIED_CODE: &str = "POLICY_DENIED";

/// Structured policy denial — never empty success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDenial {
    pub api_version: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl PolicyDenial {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            code: POLICY_DENIED_CODE.to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn to_api_error(&self) -> ApiError {
        let mut err = ApiError::new(self.code.clone(), self.message.clone());
        if let Some(details) = &self.details {
            err = err.with_details(details.clone());
        }
        err
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn policy_denial__to_api_error() {
        let denial = PolicyDenial::new("principal lacks ApproveDecision on repository scope")
            .with_details(json!({
                "capability": "ApproveDecision",
                "scope": "Repository"
            }));
        let err = denial.to_api_error();
        assert_eq!(err.code, POLICY_DENIED_CODE);
        assert_eq!(err.code, "POLICY_DENIED");
        assert!(err.message.contains("ApproveDecision"));
        assert!(err.details.is_some());
        // No prompt/claim body fields required by protocol.
        let details = err.details.as_ref().expect("details");
        assert!(details.get("prompt").is_none());
        assert!(details.get("content").is_none());
    }

    #[test]
    fn policy_denial__roundtrip_fixture_shape() {
        let raw = include_str!("../tests/fixtures/policy_denial.json");
        let denial: PolicyDenial = serde_json::from_str(raw).expect("fixture");
        assert_eq!(denial.code, POLICY_DENIED_CODE);
        let err = denial.to_api_error();
        assert_eq!(err.code, "POLICY_DENIED");
    }
}
