//! Scope grant listing and governed scope-resolution wire DTOs (T158).
//!
//! Domain mapping (T159): control-plane `ResolvedScope` → [`ScopeResolvedResponse`]
//! via `scope_identity_key`, `ScopeConfidence` Display/name, and `is_authoritative`.
//!
//! **E1 empty states**
//! - Unresolved / low confidence: `authoritative: false`, `confidence` set, `evidence` /
//!   `warnings` / `alternatives` always present as arrays (possibly empty) — never omit
//!   fields to imply full repo authority.
//! - Grant list: `grants: []` not null.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const API_VERSION: &str = "1";

fn default_api_version() -> String {
    API_VERSION.to_string()
}

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

// ---------------------------------------------------------------------------
// Resolve scope (daemon protocol — T158)
// ---------------------------------------------------------------------------

/// Request to resolve the active governed scope for a working context.
///
/// Wire surface only — no control-plane / git / store logic here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolveScopeRequest {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// Working directory hint (UTF-8 path string). Optional when signals alone suffice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Optional free-form resolution signals (`explicit_project_id`, path aliases, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signals: Option<BTreeMap<String, String>>,
    /// Explicit repository project id override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explicit_project_id: Option<String>,
    /// When true, prefer Personal scope (never auto-selected otherwise).
    #[serde(default)]
    pub force_personal: bool,
    /// Optional personal user id used when `force_personal` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_user_id: Option<String>,
}

impl Default for ResolveScopeRequest {
    fn default() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            cwd: None,
            signals: None,
            explicit_project_id: None,
            force_personal: false,
            personal_user_id: None,
        }
    }
}

/// One signal that contributed to scope resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeEvidenceDto {
    /// e.g. `explicit_project_id`, `normalized_remote_hash`, `path_alias`, `cwd`
    pub signal: String,
    pub detail: String,
}

/// Full scope-resolution response for CLI/desktop disambiguation (T158 / #20).
///
/// Mirrors control-plane `ResolvedScope` utility without importing control-plane.
///
/// **E1:** arrays default to `[]`; `authoritative: false` for Low/Ambiguous/unresolved.
/// Never treat missing fields as full grant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeResolvedResponse {
    pub api_version: String,
    /// Scope identity key (e.g. `Repository:{uuid}`). May be empty or a nil-sentinel when
    /// unresolved — clients must check `authoritative` / `confidence`.
    pub scope: String,
    /// `High` | `Medium` | `Low` | `Ambiguous` (match control-plane `ScopeConfidence` names).
    pub confidence: String,
    /// Derived like `is_authoritative()` — **false** for Low/Ambiguous/unresolved.
    pub authoritative: bool,
    /// Resolution signals; **E1** default `[]`.
    #[serde(default)]
    pub evidence: Vec<ScopeEvidenceDto>,
    /// Human-readable warnings; **E1** default `[]`.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Other scope identity keys when Ambiguous; **E1** default `[]`.
    #[serde(default)]
    pub alternatives: Vec<String>,
}

impl ScopeResolvedResponse {
    pub fn new(
        scope: impl Into<String>,
        confidence: impl Into<String>,
        authoritative: bool,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            scope: scope.into(),
            confidence: confidence.into(),
            authoritative,
            evidence: Vec::new(),
            warnings: Vec::new(),
            alternatives: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_scope_request__roundtrip() {
        let req = ResolveScopeRequest {
            api_version: API_VERSION.to_string(),
            cwd: Some("C:/dev/AI-Brains".into()),
            signals: Some(BTreeMap::from([("path_alias".into(), "ai-brains".into())])),
            explicit_project_id: Some("00000000-0000-0000-0000-0000000000a1".into()),
            force_personal: false,
            personal_user_id: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: ResolveScopeRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn resolve_scope_request__optional_fields_default() {
        let decoded: ResolveScopeRequest =
            serde_json::from_str(r#"{"api_version":"1"}"#).expect("deserialize");
        assert_eq!(decoded.api_version, "1");
        assert!(decoded.cwd.is_none());
        assert!(decoded.signals.is_none());
        assert!(!decoded.force_personal);
    }

    #[test]
    fn scope_resolved_response__authoritative_false__roundtrip() {
        let resp = ScopeResolvedResponse {
            api_version: API_VERSION.to_string(),
            scope: String::new(),
            confidence: "Low".into(),
            authoritative: false,
            evidence: vec![ScopeEvidenceDto {
                signal: "cwd".into(),
                detail: "heuristic only".into(),
            }],
            warnings: vec!["cwd-only resolution is not authoritative".into()],
            alternatives: Vec::new(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: ScopeResolvedResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, resp);
        assert!(!decoded.authoritative);
    }

    #[test]
    fn scope_resolved_response__alternatives_and_warnings__roundtrip() {
        let resp = ScopeResolvedResponse {
            api_version: API_VERSION.to_string(),
            scope: "Repository:00000000-0000-0000-0000-0000000000a1".into(),
            confidence: "Ambiguous".into(),
            authoritative: false,
            evidence: vec![],
            warnings: vec!["multiple candidates".into()],
            alternatives: vec![
                "Repository:00000000-0000-0000-0000-0000000000a1".into(),
                "Repository:00000000-0000-0000-0000-0000000000a2".into(),
            ],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let decoded: ScopeResolvedResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.alternatives.len(), 2);
        assert_eq!(decoded.warnings, vec!["multiple candidates".to_string()]);
        assert!(!decoded.authoritative);
    }

    #[test]
    fn scope_resolved_response__e1_empty_arrays_present() {
        // Missing evidence/warnings/alternatives keys → empty arrays, not null.
        let v = json!({
            "api_version": "1",
            "scope": "Repository:00000000-0000-0000-0000-0000000000a1",
            "confidence": "High",
            "authoritative": true
        });
        let decoded: ScopeResolvedResponse = serde_json::from_value(v).expect("deserialize");
        assert!(decoded.evidence.is_empty());
        assert!(decoded.warnings.is_empty());
        assert!(decoded.alternatives.is_empty());

        let round = serde_json::to_value(&decoded).expect("serialize");
        assert!(round.get("evidence").unwrap().is_array());
        assert!(round.get("warnings").unwrap().is_array());
        assert!(round.get("alternatives").unwrap().is_array());
    }
}
