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
