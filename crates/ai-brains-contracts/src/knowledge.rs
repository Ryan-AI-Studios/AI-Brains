use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

/// Evidence handle — id + optional cite label (not prose-only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHandle {
    pub evidence_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cite_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItemDto {
    pub id: String,
    pub kind: String,
    pub statement: String,
    pub state: String,
    #[serde(default)]
    pub evidence_handles: Vec<EvidenceHandle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItemResponse {
    pub api_version: String,
    pub item: KnowledgeItemDto,
}

impl KnowledgeItemResponse {
    pub fn new(item: KnowledgeItemDto) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            item,
        }
    }
}
