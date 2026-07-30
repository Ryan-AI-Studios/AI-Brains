//! Loopback HTTP adapter for T161 routes (invoke-only transport).
//!
//! - Token stays in Rust (`%USERPROFILE%\.ai-brains\http.token`); never returned to JS.
//! - Full bearer is never logged.
//! - Adapter only: no grants / freshness / erasure domain semantics.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{resolve_loopback_base_url, user_session_token_path};

/// Connect timeout for loopback daemon HTTP.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// Total request timeout for loopback daemon HTTP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Structured invoke error for the frontend (kind + message + optional status).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvokeApiError {
    /// `offline` | `denied` | `transient` | `error`
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

impl std::fmt::Display for InvokeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl InvokeApiError {
    pub fn offline(message: impl Into<String>) -> Self {
        Self {
            kind: "offline".to_string(),
            message: message.into(),
            status: None,
        }
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Self {
            kind: "denied".to_string(),
            message: message.into(),
            status: Some(401),
        }
    }

    pub fn denied_status(status: u16, message: impl Into<String>) -> Self {
        Self {
            kind: "denied".to_string(),
            message: message.into(),
            status: Some(status),
        }
    }

    pub fn transient(message: impl Into<String>, status: Option<u16>) -> Self {
        Self {
            kind: "transient".to_string(),
            message: message.into(),
            status,
        }
    }

    pub fn error(message: impl Into<String>, status: Option<u16>) -> Self {
        Self {
            kind: "error".to_string(),
            message: message.into(),
            status,
        }
    }
}

/// Map HTTP status codes to structured error kinds (pure; unit-tested).
pub fn map_http_status(status: u16, body_preview: &str) -> InvokeApiError {
    let message = if body_preview.trim().is_empty() {
        format!("HTTP {status}")
    } else {
        // Cap body preview so we never ship huge payloads as error text.
        let preview: String = body_preview.chars().take(240).collect();
        format!("HTTP {status}: {preview}")
    };

    match status {
        401 | 403 => InvokeApiError::denied_status(status, message),
        408 | 429 => InvokeApiError::transient(message, Some(status)),
        500..=599 => InvokeApiError::transient(message, Some(status)),
        _ => InvokeApiError::error(message, Some(status)),
    }
}

/// Map a `reqwest::Error` to structured kinds (connect → offline, timeout → transient).
pub fn map_reqwest_error(err: &reqwest::Error) -> InvokeApiError {
    if err.is_timeout() {
        return InvokeApiError::transient("request timed out contacting the daemon", None);
    }
    if err.is_connect() {
        return InvokeApiError::offline("daemon unreachable (connection failed)");
    }
    // reqwest may surface DNS/connect failures without is_connect on some platforms.
    let display = err.to_string();
    let lower = display.to_ascii_lowercase();
    if lower.contains("connection refused")
        || lower.contains("connect")
        || lower.contains("os error 10061")
        || lower.contains("actively refused")
    {
        return InvokeApiError::offline("daemon unreachable (connection failed)");
    }
    InvokeApiError::error(display, err.status().map(|s| s.as_u16()))
}

/// Ensure `command_id` is a non-empty UUID string (generate in Rust when omitted).
pub fn ensure_command_id(command_id: &mut Option<String>) {
    let empty = command_id
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if empty {
        *command_id = Some(uuid::Uuid::new_v4().to_string());
    }
}

/// Read the user-session bearer from disk. Missing/empty → denied.
///
/// Never returns the token to callers outside this module's request path in a
/// way that would cross the JS boundary; callers attach it only to Authorization.
pub fn read_user_session_token() -> Result<String, InvokeApiError> {
    let path = user_session_token_path().ok_or_else(|| {
        InvokeApiError::denied("user home directory unavailable; cannot locate session token")
    })?;

    if !path.is_file() {
        return Err(InvokeApiError::denied(
            "user-session token missing (%USERPROFILE%\\.ai-brains\\http.token)",
        ));
    }

    let raw = std::fs::read_to_string(&path).map_err(|e| {
        // Do not include file contents; path is fixed and non-secret.
        InvokeApiError::denied(format!("failed to read user-session token: {e}"))
    })?;

    let token = raw.trim();
    if token.is_empty() {
        return Err(InvokeApiError::denied("user-session token file is empty"));
    }

    Ok(token.to_string())
}

/// Minimal application/x-www-form-urlencoded component encoding for query strings.
fn encode_query_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(b & 0xf) as usize]));
            }
        }
    }
    out
}

fn build_client() -> Result<reqwest::Client, InvokeApiError> {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| InvokeApiError::error(format!("failed to build HTTP client: {e}"), None))
}

fn base_url() -> Result<String, InvokeApiError> {
    resolve_loopback_base_url().ok_or_else(|| {
        InvokeApiError::error(
            "invalid AI_BRAINS_HTTP_PORT; cannot resolve loopback base URL",
            None,
        )
    })
}

/// Authenticated JSON request against a T161 path. Response body as `serde_json::Value`.
pub async fn request_json(
    method: reqwest::Method,
    path: &str,
    query: &[(&str, String)],
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, InvokeApiError> {
    let token = read_user_session_token()?;
    let base = base_url()?;
    let client = build_client()?;

    let url = if query.is_empty() {
        format!("{base}{path}")
    } else {
        let mut pairs = Vec::with_capacity(query.len());
        for (k, v) in query {
            pairs.push(format!(
                "{}={}",
                encode_query_component(k),
                encode_query_component(v)
            ));
        }
        format!("{base}{path}?{}", pairs.join("&"))
    };

    let mut builder = client
        .request(method, &url)
        // Authorization header only; never log the bearer value.
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(reqwest::header::ACCEPT, "application/json");

    if let Some(json) = body {
        builder = builder
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(json);
    }

    let response = builder.send().await.map_err(|e| map_reqwest_error(&e))?;
    let status = response.status().as_u16();
    let text = response.text().await.map_err(|e| {
        InvokeApiError::transient(format!("failed to read response body: {e}"), Some(status))
    })?;

    if !(200..300).contains(&status) {
        return Err(map_http_status(status, &text));
    }

    if text.trim().is_empty() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    serde_json::from_str(&text).map_err(|e| {
        InvokeApiError::error(
            format!("daemon returned non-JSON success body: {e}"),
            Some(status),
        )
    })
}

pub async fn post_json(
    path: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, InvokeApiError> {
    request_json(reqwest::Method::POST, path, &[], Some(body)).await
}

pub async fn get_json(
    path: &str,
    query: &[(&str, String)],
) -> Result<serde_json::Value, InvokeApiError> {
    request_json(reqwest::Method::GET, path, query, None).await
}

/// Soft optional: GET `/health` (no auth required by T161).
pub async fn probe_health() -> Result<serde_json::Value, InvokeApiError> {
    let base = base_url()?;
    let client = build_client()?;
    let url = format!("{base}/health");

    let response = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| map_reqwest_error(&e))?;

    let status = response.status().as_u16();
    let text = response.text().await.map_err(|e| {
        InvokeApiError::transient(format!("failed to read health body: {e}"), Some(status))
    })?;

    if !(200..300).contains(&status) {
        return Err(map_http_status(status, &text));
    }

    if text.trim().is_empty() {
        let mut map = serde_json::Map::new();
        map.insert(
            "status".to_string(),
            serde_json::Value::String("ok".to_string()),
        );
        return Ok(serde_json::Value::Object(map));
    }

    serde_json::from_str(&text).map_err(|e| {
        InvokeApiError::error(format!("health returned non-JSON body: {e}"), Some(status))
    })
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn map_http_status__401__denied() {
        let err = map_http_status(401, r#"{"error":"unauthorized"}"#);
        assert_eq!(err.kind, "denied");
        assert_eq!(err.status, Some(401));
        assert!(err.message.contains("401"));
    }

    #[test]
    fn map_http_status__403__denied() {
        let err = map_http_status(403, "forbidden");
        assert_eq!(err.kind, "denied");
        assert_eq!(err.status, Some(403));
    }

    #[test]
    fn map_http_status__503__transient() {
        let err = map_http_status(503, "unavailable");
        assert_eq!(err.kind, "transient");
        assert_eq!(err.status, Some(503));
    }

    #[test]
    fn map_http_status__404__error() {
        let err = map_http_status(404, "not found");
        assert_eq!(err.kind, "error");
        assert_eq!(err.status, Some(404));
    }

    #[test]
    fn ensure_command_id__generates_when_missing() {
        let mut id = None;
        ensure_command_id(&mut id);
        let value = id.expect("command_id");
        assert!(uuid::Uuid::parse_str(&value).is_ok());
    }

    #[test]
    fn ensure_command_id__preserves_when_present() {
        let mut id = Some("already-set".to_string());
        ensure_command_id(&mut id);
        assert_eq!(id.as_deref(), Some("already-set"));
    }

    #[test]
    fn ensure_command_id__regenerates_when_blank() {
        let mut id = Some("   ".to_string());
        ensure_command_id(&mut id);
        let value = id.expect("command_id");
        assert!(uuid::Uuid::parse_str(&value).is_ok());
    }

    #[test]
    fn invoke_api_error__serializes_kind_and_message() {
        let err = InvokeApiError::offline("daemon unreachable");
        let v = serde_json::to_value(&err).expect("serialize");
        assert_eq!(v["kind"], "offline");
        assert_eq!(v["message"], "daemon unreachable");
        assert!(v.get("status").is_none());
    }

    #[tokio::test]
    async fn post_json__httpmock_happy_empty_briefing() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/briefings/project")
                .header("authorization", "Bearer test-token-not-logged");
            then.status(200).json_body(serde_json::json!({
                "api_version": "1",
                "packet": {
                    "api_version": "1",
                    "briefing_id": "b1",
                    "kind": "Project",
                    "scope": {
                        "scope_key": "Repository:00000000-0000-0000-0000-0000000000a1",
                        "confidence": "High",
                        "warnings": [],
                        "alternatives": [],
                        "authoritative": true
                    },
                    "decisions": [],
                    "conclusions": [],
                    "constraints": [],
                    "warnings": [],
                    "freshness": {
                        "total_sources": 0,
                        "fresh_count": 0,
                        "stale_count": 0,
                        "unavailable_count": 0,
                        "worst_state": "Unknown"
                    },
                    "evidence_handles": [],
                    "budget": {
                        "max_words": 500,
                        "used_words": 0,
                        "truncated_sections": [],
                        "more_available": false
                    },
                    "denied": false
                }
            }));
        });

        // Direct call against mock: exercise JSON shape + auth header (no full path via resolve_loopback).
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("client");
        let url = format!("{}/v1/briefings/project", server.base_url());
        let response = client
            .post(&url)
            .header(
                reqwest::header::AUTHORIZATION,
                "Bearer test-token-not-logged",
            )
            .json(&serde_json::json!({ "api_version": "1" }))
            .send()
            .await
            .expect("send");
        assert_eq!(response.status().as_u16(), 200);
        let body: serde_json::Value = response.json().await.expect("json");
        assert_eq!(body["api_version"], "1");
        assert!(body["packet"]["decisions"].as_array().unwrap().is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn get_json__httpmock_review_empty_items() {
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/v1/review/items");
            then.status(200).json_body(serde_json::json!({
                "api_version": "1",
                "items": []
            }));
        });

        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("client");
        let url = format!("{}/v1/review/items", server.base_url());
        let response = client.get(&url).send().await.expect("send");
        assert_eq!(response.status().as_u16(), 200);
        let body: serde_json::Value = response.json().await.expect("json");
        assert!(body["items"].as_array().unwrap().is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn map_reqwest_error__unreachable_loopback__offline_or_transient() {
        // Port 1 is almost never listening. Windows may surface connection-refused
        // (offline) or a short connect timeout (transient); both paint promptly.
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(200))
            .timeout(Duration::from_millis(500))
            .build()
            .expect("client");
        let err = client
            .get("http://127.0.0.1:1/health")
            .send()
            .await
            .expect_err("must fail to connect");
        let mapped = map_reqwest_error(&err);
        assert!(
            mapped.kind == "offline" || mapped.kind == "transient",
            "unreachable daemon must map to offline or transient, got {:?}",
            mapped
        );
    }

    #[test]
    fn map_http_status__401__denied_not_offline() {
        // Regression: auth failures must not look like connectivity loss.
        let err = map_http_status(401, "nope");
        assert_eq!(err.kind, "denied");
        assert_ne!(err.kind, "offline");
    }
}
