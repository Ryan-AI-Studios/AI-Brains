use ai_brains_core::ids::SessionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub memory_id: String,
    pub content: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResponse {
    pub results: Vec<RecallResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__serializes_with_session_id() {
        let resp = RecallResponse {
            results: vec![],
            session_id: Some("test-session".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("test-session"));
    }

    #[test]
    #[allow(clippy::disallowed_methods)]
    #[allow(non_snake_case)]
    fn recall_response__omits_none_session_id() {
        let resp = RecallResponse {
            results: vec![],
            session_id: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("session_id"));
    }
}
