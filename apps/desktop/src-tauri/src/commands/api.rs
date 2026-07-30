//! Tauri commands that proxy T161 HTTP routes (adapter only).
//!
//! Request bodies are hand-synced wire shapes (serde_json). Responses pass through
//! as `serde_json::Value` so the desktop binary does not depend on `ai-brains-contracts`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::http_client::{
    InvokeApiError, ensure_command_id, get_json, post_json, probe_health as http_probe_health,
};

fn api_version_one() -> String {
    "1".to_string()
}

// ---------------------------------------------------------------------------
// Request args (invoke payloads from JS)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectBriefingArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_words: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_briefing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalBriefingArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_words: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governed_briefing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryKnowledgeArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectEvidenceArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectSourceArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListReviewItemsArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveReviewItemArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    pub id: String,
    /// e.g. approved | dismissed | deferred
    pub resolution: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolveScopeArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signals: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explicit_project_id: Option<String>,
    #[serde(default)]
    pub force_personal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personal_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestErasureArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipeContentEnvelopeArgs {
    #[serde(default = "api_version_one")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    pub content_key_id: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    /// Default true (dry-run safe).
    #[serde(default = "default_true")]
    pub dry_run: bool,
    /// Must be true with dry_run false to execute.
    #[serde(default)]
    pub confirm: bool,
}

fn default_true() -> bool {
    true
}

fn to_value<T: Serialize>(value: &T) -> Result<Value, InvokeApiError> {
    serde_json::to_value(value)
        .map_err(|e| InvokeApiError::error(format!("failed to serialize request: {e}"), None))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn project_briefing(args: ProjectBriefingArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/briefings/project", &to_value(&args)?).await
}

#[tauri::command]
pub async fn personal_briefing(args: PersonalBriefingArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/briefings/personal", &to_value(&args)?).await
}

#[tauri::command]
pub async fn query_knowledge(args: QueryKnowledgeArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/knowledge/query", &to_value(&args)?).await
}

#[tauri::command]
pub async fn inspect_evidence(args: InspectEvidenceArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/evidence/inspect", &to_value(&args)?).await
}

#[tauri::command]
pub async fn inspect_source(args: InspectSourceArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/sources/inspect", &to_value(&args)?).await
}

#[tauri::command]
pub async fn list_review_items(args: ListReviewItemsArgs) -> Result<Value, InvokeApiError> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if let Some(p) = args.principal_id {
        query.push(("principal_id", p));
    }
    if let Some(s) = args.scope {
        query.push(("scope", s));
    }
    if let Some(st) = args.status {
        query.push(("status", st));
    }
    get_json("/v1/review/items", &query).await
}

#[tauri::command]
pub async fn resolve_review_item(mut args: ResolveReviewItemArgs) -> Result<Value, InvokeApiError> {
    ensure_command_id(&mut args.command_id);
    let id = args.id.clone();
    let path = format!("/v1/review/items/{id}/resolve");
    post_json(&path, &to_value(&args)?).await
}

#[tauri::command]
pub async fn resolve_scope(args: ResolveScopeArgs) -> Result<Value, InvokeApiError> {
    post_json("/v1/scope/resolve", &to_value(&args)?).await
}

#[tauri::command]
pub async fn request_erasure(mut args: RequestErasureArgs) -> Result<Value, InvokeApiError> {
    ensure_command_id(&mut args.command_id);
    post_json("/v1/erasure/request", &to_value(&args)?).await
}

#[tauri::command]
pub async fn wipe_content_envelope(
    mut args: WipeContentEnvelopeArgs,
) -> Result<Value, InvokeApiError> {
    ensure_command_id(&mut args.command_id);
    post_json("/v1/erasure/wipe", &to_value(&args)?).await
}

/// Soft optional: loopback `/health` probe (no bearer).
#[tauri::command]
pub async fn probe_health() -> Result<Value, InvokeApiError> {
    http_probe_health().await
}

/// Build request JSON for resolve_review_item (pure shape helper for tests).
#[cfg(test)]
pub fn resolve_review_item_request_json(mut args: ResolveReviewItemArgs) -> Value {
    ensure_command_id(&mut args.command_id);
    serde_json::to_value(&args).unwrap_or(Value::Null)
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn resolve_review_item_request_json__includes_command_id_and_resolution() {
        let args = ResolveReviewItemArgs {
            api_version: "1".into(),
            id: "item-1".into(),
            resolution: "approved".into(),
            principal_id: None,
            note: Some("ok".into()),
            scope: Some("Repository:a".into()),
            command_id: None,
        };
        let v = resolve_review_item_request_json(args);
        assert_eq!(v["id"], "item-1");
        assert_eq!(v["resolution"], "approved");
        assert_eq!(v["api_version"], "1");
        let cid = v["command_id"].as_str().expect("command_id string");
        assert!(uuid::Uuid::parse_str(cid).is_ok());
    }

    #[test]
    fn wipe_args__default_dry_run_true() {
        let raw = r#"{"content_key_id":"k1","scope":"Personal:u"}"#;
        let args: WipeContentEnvelopeArgs = serde_json::from_str(raw).expect("deserialize");
        assert!(args.dry_run);
        assert!(!args.confirm);
    }

    #[test]
    fn request_erasure_args__ids_default_empty_array() {
        let raw = r#"{"api_version":"1"}"#;
        let args: RequestErasureArgs = serde_json::from_str(raw).expect("deserialize");
        assert!(args.ids.is_empty());
    }
}
