//! Preflight DTOs.
//!
//! `PreflightContextResponse` is the CLI compact full-preflight JSON envelope
//! (`text`, `word_count`, additive `sections`). Daemon/HTTP do not serialize this type.

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

/// One `---` header block inside compact preflight JSON `text` (T265).
/// Wire `id` is a string (closed set this schema; unknown future ids still deserialize).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreflightSection {
    pub id: String,
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightContextResponse {
    pub text: String,
    pub word_count: usize,
    /// Always present (T265). E1 empty is `[]`, never null, never omitted.
    /// N−1 2-key JSON deserializes as empty via `default`.
    #[serde(default)]
    pub sections: Vec<PreflightSection>,
}
