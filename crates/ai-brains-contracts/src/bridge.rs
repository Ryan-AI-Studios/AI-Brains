use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BridgePayload {
    Hotspot {
        #[serde(rename = "type")]
        type_field: String,
        path: String,
        score: f64,
        reason: String,
        #[serde(default)]
        temporal_coupling: f64,
        #[serde(default)]
        failure_risk_probability: f64,
    },
    LedgerDelta {
        #[serde(rename = "type")]
        type_field: String,
        tx_id: String,
        intent: String,
        files_changed: usize,
    },
    Insight {
        #[serde(rename = "type")]
        type_field: String,
        memory_id: String,
        relevance: f64,
        content: String,
    },
    VerifyOutcome {
        #[serde(rename = "type")]
        type_field: String,
        #[serde(flatten)]
        outcome: BridgeVerifyOutcome,
    },
    Query {
        #[serde(rename = "type")]
        type_field: String,
        text: String,
    },
    Madr {
        #[serde(rename = "type")]
        type_field: String,
        title: String,
        context: String,
        decision: String,
        consequences: String,
    },
    RiskAlert {
        #[serde(rename = "type")]
        type_field: String,
        coupled_file_a: String,
        coupled_file_b: String,
        coupling_score: f64,
        affected_symbols: Vec<String>,
        suggested_remediation: String,
        risk_level: String,
    },
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeVerifyOutcome {
    pub success: bool,
    pub command: String,
    pub error_snippet: Option<String>,
}

/// Bridge interchange record. Uses flexible string types for cross-repo compatibility.
/// Conversion to typed IDs happens at the ingest boundary, not at deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRecord {
    pub bridge_version: String,
    pub direction: BridgeDirection,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parent_hash: Option<String>,
    pub project_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub tx_id: Option<String>,
    pub record_kind: String,
    pub payload: BridgePayload,
    pub privacy: Privacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BridgeDirection {
    Inbound,
    Outbound,
}

impl BridgeRecord {
    pub fn formatted_payload(&self) -> String {
        serde_json::to_string_pretty(&self.payload).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn payload_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.payload).unwrap_or(serde_json::Value::Null)
    }
}
