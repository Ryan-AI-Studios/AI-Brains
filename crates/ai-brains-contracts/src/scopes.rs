use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeGrantDto {
    pub grant_id: String,
    pub principal_id: String,
    pub scope: String,
    pub capability: String,
    pub privacy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeGrantsResponse {
    pub api_version: String,
    #[serde(default)]
    pub grants: Vec<ScopeGrantDto>,
}

impl ScopeGrantsResponse {
    pub fn new(grants: Vec<ScopeGrantDto>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            grants,
        }
    }
}
