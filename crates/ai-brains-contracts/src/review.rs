use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItemDto {
    pub id: String,
    pub subject: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opened_at: Option<DateTime<Utc>>,
}

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
