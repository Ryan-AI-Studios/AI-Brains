//! Loopback HTTP adapter for T161 routes (invoke-only transport).
//!
//! - Token stays in Rust (`%USERPROFILE%\.ai-brains\http.token`); never returned to JS.
//! - Full bearer is never logged.
//! - Adapter only: no grants / freshness / erasure domain semantics.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use zeroize::Zeroizing;

use super::{resolve_loopback_base_url, user_session_token_path};

#[cfg(test)]
use std::sync::Mutex;

/// Test-only base URL override so httpmock ports do not race on env vars.
#[cfg(test)]
static BASE_URL_OVERRIDE_FOR_TESTS: Mutex<Option<String>> = Mutex::new(None);

/// Serialize adapter HTTP tests that mutate token path / base URL overrides.
/// Tokio mutex so the guard can be held across `.await` in async tests.
#[cfg(test)]
static ADAPTER_HTTP_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
/// Returns `Zeroizing<String>` so the secret is zeroed on drop. Never returned
/// to JS — callers attach it only to the Authorization header within Rust.
///
/// Best-effort zeroize: file contents are read into `Zeroizing`, trimmed into a
/// second `Zeroizing`, and the raw buffer is dropped before return. The
/// Authorization header is a short-lived `String` owned by reqwest for the
/// request lifetime only.
pub fn read_user_session_token() -> Result<Zeroizing<String>, InvokeApiError> {
    let path = user_session_token_path().ok_or_else(|| {
        InvokeApiError::denied("user home directory unavailable; cannot locate session token")
    })?;

    if !path.is_file() {
        return Err(InvokeApiError::denied(
            "user-session token missing (%USERPROFILE%\\.ai-brains\\http.token)",
        ));
    }

    // Read into Zeroizing so the full file buffer is wiped on drop.
    let raw = Zeroizing::new(std::fs::read_to_string(&path).map_err(|e| {
        // Do not include file contents; path is non-secret.
        InvokeApiError::denied(format!("failed to read user-session token: {e}"))
    })?);

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(InvokeApiError::denied("user-session token file is empty"));
    }

    // Copy only the trimmed token into a fresh Zeroizing; `raw` drops next.
    Ok(Zeroizing::new(trimmed.to_owned()))
}

/// Percent-encode a single URL path segment (RFC 3986 unreserved left alone).
///
/// Differs from query form-encoding: space becomes `%20` (not `+`).
pub fn encode_path_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(b & 0xf) as usize]));
            }
        }
    }
    out
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
    #[cfg(test)]
    {
        if let Ok(guard) = BASE_URL_OVERRIDE_FOR_TESTS.lock()
            && let Some(ref url) = *guard
        {
            return Ok(url.clone());
        }
    }

    resolve_loopback_base_url().ok_or_else(|| {
        InvokeApiError::error(
            "invalid AI_BRAINS_HTTP_PORT; cannot resolve loopback base URL",
            None,
        )
    })
}

/// Point the adapter at an httpmock (or other) base URL for the current process.
///
/// Cleared by passing `None`. Callers must serialize concurrent adapter tests
/// (see `ADAPTER_HTTP_TEST_LOCK` in the test module).
#[cfg(test)]
pub fn set_base_url_override_for_tests(url: Option<String>) {
    if let Ok(mut guard) = BASE_URL_OVERRIDE_FOR_TESTS.lock() {
        *guard = url;
    }
}

/// Build a sensitive `Authorization: Bearer …` header without a long-lived plain `String`.
///
/// Bytes are assembled in a [`Zeroizing`] buffer, then copied into a
/// [`reqwest::header::HeaderValue`] marked sensitive (redacted in Debug). The
/// Zeroizing buffer is zeroed on drop; HeaderValue ownership lasts for the
/// request lifetime (reqwest API constraint).
fn authorization_header_value(token: &str) -> Result<reqwest::header::HeaderValue, InvokeApiError> {
    let mut buf = Zeroizing::new(Vec::with_capacity(7 + token.len()));
    buf.extend_from_slice(b"Bearer ");
    buf.extend_from_slice(token.as_bytes());
    let mut value = reqwest::header::HeaderValue::from_bytes(buf.as_slice()).map_err(|e| {
        InvokeApiError::error(
            format!("invalid user-session token for Authorization header: {e}"),
            None,
        )
    })?;
    value.set_sensitive(true);
    Ok(value)
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

    // Build Authorization without `format!` into a long-lived plain String:
    // Zeroizing buffer → sensitive HeaderValue (redacted in Debug; never logged).
    let auth_header = authorization_header_value(token.as_str())?;

    let mut builder = client
        .request(method, &url)
        .header(reqwest::header::AUTHORIZATION, auth_header)
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

    /// Holds the process-wide adapter test lock + restores overrides on drop.
    struct AdapterHttpTestGuard {
        _lock: tokio::sync::MutexGuard<'static, ()>,
        _dir: tempfile::TempDir,
    }

    impl AdapterHttpTestGuard {
        async fn lock_with_token(token: Option<&str>) -> Self {
            let lock = ADAPTER_HTTP_TEST_LOCK.lock().await;
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("http.token");
            match token {
                Some(t) => {
                    std::fs::write(&path, t).expect("write token");
                    crate::commands::set_token_path_override_for_tests(Some(path));
                }
                None => {
                    // Point at a non-existent file under the temp dir.
                    crate::commands::set_token_path_override_for_tests(Some(path));
                }
            }
            Self {
                _lock: lock,
                _dir: dir,
            }
        }

        fn point_at(server: &httpmock::MockServer) {
            set_base_url_override_for_tests(Some(server.base_url()));
        }
    }

    impl Drop for AdapterHttpTestGuard {
        fn drop(&mut self) {
            set_base_url_override_for_tests(None);
            crate::commands::set_token_path_override_for_tests(None);
        }
    }

    #[tokio::test]
    async fn post_json__httpmock_token_file__success_body() {
        let server = httpmock::MockServer::start();
        let _guard = AdapterHttpTestGuard::lock_with_token(Some("test-token-not-logged")).await;
        AdapterHttpTestGuard::point_at(&server);

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

        let body = post_json(
            "/v1/briefings/project",
            &serde_json::json!({ "api_version": "1" }),
        )
        .await
        .expect("post_json should succeed");
        assert_eq!(body["api_version"], "1");
        assert!(
            body["packet"]["decisions"]
                .as_array()
                .expect("decisions array")
                .is_empty()
        );
        mock.assert();
    }

    #[tokio::test]
    async fn get_json__httpmock_token_file__success_body() {
        let server = httpmock::MockServer::start();
        let _guard = AdapterHttpTestGuard::lock_with_token(Some("test-token-not-logged")).await;
        AdapterHttpTestGuard::point_at(&server);

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/v1/review/items")
                .header("authorization", "Bearer test-token-not-logged");
            then.status(200).json_body(serde_json::json!({
                "api_version": "1",
                "items": []
            }));
        });

        let body = get_json("/v1/review/items", &[])
            .await
            .expect("get_json should succeed");
        assert!(
            body["items"]
                .as_array()
                .expect("items array")
                .is_empty()
        );
        mock.assert();
    }

    #[tokio::test]
    async fn post_json__missing_token__denied() {
        let server = httpmock::MockServer::start();
        let _guard = AdapterHttpTestGuard::lock_with_token(None).await;
        AdapterHttpTestGuard::point_at(&server);

        let err = post_json(
            "/v1/briefings/project",
            &serde_json::json!({ "api_version": "1" }),
        )
        .await
        .expect_err("missing token must deny");
        assert_eq!(err.kind, "denied");
        assert!(
            err.message.to_ascii_lowercase().contains("token"),
            "expected token-related message, got {}",
            err.message
        );
    }

    #[tokio::test]
    async fn post_json__httpmock_401__denied_kind() {
        let server = httpmock::MockServer::start();
        let _guard = AdapterHttpTestGuard::lock_with_token(Some("bad-or-expired-token")).await;
        AdapterHttpTestGuard::point_at(&server);

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/v1/briefings/project")
                .header("authorization", "Bearer bad-or-expired-token");
            then.status(401).body(r#"{"error":"unauthorized"}"#);
        });

        let err = post_json(
            "/v1/briefings/project",
            &serde_json::json!({ "api_version": "1" }),
        )
        .await
        .expect_err("401 must map to denied");
        assert_eq!(err.kind, "denied");
        assert_eq!(err.status, Some(401));
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

    #[test]
    fn encode_path_segment__leaves_unreserved() {
        assert_eq!(encode_path_segment("item-1_A.z~"), "item-1_A.z~");
    }

    #[test]
    fn encode_path_segment__encodes_slash_and_space() {
        assert_eq!(encode_path_segment("a/b c"), "a%2Fb%20c");
    }

    #[test]
    fn encode_path_segment__encodes_plus_and_percent() {
        assert_eq!(encode_path_segment("a+b%"), "a%2Bb%25");
    }
}
