use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeutralEvent {
    pub role: String,
    pub content: String,
    pub status: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}
