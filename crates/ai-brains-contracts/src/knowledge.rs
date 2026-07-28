//! Knowledge item and governed propose DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

/// Evidence handle — id + optional cite label (not prose-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

// ---------------------------------------------------------------------------
// Propose conclusion / decision (daemon protocol — T158)
// ---------------------------------------------------------------------------

/// Propose a conclusion (wire request; handlers in T159).
///
/// No secret / full sealed-prompt bodies. Evidence referenced by id only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposeConclusionRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// Scope identity key.
    pub scope: String,
    pub statement: String,
    /// Supporting evidence ids (empty array allowed only for deserialize; handlers may reject).
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
}

/// Result of accepting a conclusion proposal into the control plane.
///
/// **E1:** `warnings: []` when none; never null.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConclusionProposedResponse {
    pub api_version: String,
    pub conclusion_id: String,
    /// e.g. `proposed`, `unsupported`
    pub status: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl ConclusionProposedResponse {
    pub fn new(conclusion_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            conclusion_id: conclusion_id.into(),
            status: status.into(),
            warnings: Vec::new(),
        }
    }
}

/// Propose a decision (wire request; handlers in T159).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposeDecisionRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub statement: String,
    #[serde(default)]
    pub conclusion_ids: Vec<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
}

/// Result of accepting a decision proposal.
///
/// **E1:** `warnings: []` when none.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionProposedResponse {
    pub api_version: String,
    pub decision_id: String,
    pub status: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl DecisionProposedResponse {
    pub fn new(decision_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            decision_id: decision_id.into(),
            status: status.into(),
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn propose_conclusion_request__roundtrip() {
        let req = ProposeConclusionRequest {
            api_version: API_VERSION.to_string(),
            principal_id: Some("p1".into()),
            scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
            statement: "Briefings are deterministic".into(),
            evidence_ids: vec!["e1".into(), "e2".into()],
            privacy: Some("LocalOnly".into()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: ProposeConclusionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }
}
