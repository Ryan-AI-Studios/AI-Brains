use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightRequest {
    pub client_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResponse {
    pub daemon_version: String,
    pub vault_locked: bool,
    pub system_healthy: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub capabilities: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightContextResponse {
    pub text: String,
    pub word_count: usize,
}
