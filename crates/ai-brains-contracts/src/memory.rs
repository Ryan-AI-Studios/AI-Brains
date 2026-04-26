use ai_brains_core::ids::{MemoryId, SessionId, TurnId};
use ai_brains_core::privacy::Privacy;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDto {
    pub id: MemoryId,
    pub session_id: SessionId,
    pub turn_id: Option<TurnId>,
    pub content: String,
    pub privacy: Privacy,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoryRequest {
    pub content: String,
    pub privacy: Privacy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<TurnId>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}
