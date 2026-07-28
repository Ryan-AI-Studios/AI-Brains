//! T161 security suite — bind, auth, CORS, body limit, policy, token ACL, dispatch port.
#![allow(clippy::disallowed_methods, non_snake_case)]

use std::sync::Arc;

use ai_brains_api_server::bind::{
    ALLOW_NON_LOOPBACK_ENV, BindError, is_loopback_addr, resolve_bind_addr,
};
use ai_brains_api_server::dispatch::HttpDispatch;
use ai_brains_api_server::dispatch::test_support::MockHttpDispatch;
use ai_brains_api_server::token::{
    USER_TOKEN_FILE_SDDL, generate_token, load_or_create_token, verify_owner_acl_output,
    write_token_file,
};
use ai_brains_api_server::{
    BODY_LIMIT_BYTES, app_state, build_router, token_bytes_equal, tokens_equal,
};
use ai_brains_contracts::policy::POLICY_DENIED_CODE;
use ai_brains_contracts::response::ApiError;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn test_token() -> String {
    "test-bearer-token-with-enough-entropy-0123456789abcdef".to_string()
}

fn mock_ok_dispatch() -> Arc<dyn HttpDispatch> {
    Arc::new(MockHttpDispatch::always(DaemonResponse::Pong))
}

fn app_with_token(token: &str) -> axum::Router {
    let dispatch = mock_ok_dispatch();
    let state = app_state(dispatch, token.to_string());
    build_router(state)
}

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes()
        .to_vec()
}

#[test]
fn http_bind__non_loopback_without_optin__rejected() {
    // Pure: 0.0.0.0 is not loopback.
    let ip: std::net::IpAddr = "0.0.0.0".parse().unwrap();
    assert!(!is_loopback_addr(ip));

    // Force-clear ambient opt-in so the reject path always runs (R1-08).
    // nextest process isolation is the default isolation story.
    let _guard = TempEnv::remove(ALLOW_NON_LOOPBACK_ENV);
    let err = resolve_bind_addr(Some("0.0.0.0:7432"), None).expect_err("must reject");
    match err {
        BindError::NonLoopbackWithoutOptIn { addr } => {
            assert_eq!(addr.ip().to_string(), "0.0.0.0");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn http_bind__loopback_default__accepted() {
    let addr = resolve_bind_addr(None, Some(0)).expect("loopback");
    assert!(is_loopback_addr(addr.ip()));
    assert_eq!(addr.port(), 0);
}

#[tokio::test]
async fn http_auth__missing_bearer__401() {
    let app = app_with_token(&test_token());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/scope/resolve")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"api_version":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let bytes = body_bytes(resp).await;
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // Auth 401 uses DaemonResponse::Error tagged shape (R1-07).
    assert_eq!(v["type"], "error");
    assert_eq!(v["payload"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn http_auth__wrong_token__401() {
    let app = app_with_token(&test_token());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/scope/resolve")
                .header("content-type", "application/json")
                .header("authorization", "Bearer wrong-token-value")
                .body(Body::from(r#"{"api_version":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn http_auth__valid_token__200_on_health_or_smoke() {
    let token = test_token();
    let app = app_with_token(&token);

    // Health is unauthenticated.
    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let hb = body_bytes(health).await;
    let hv: serde_json::Value = serde_json::from_slice(&hb).unwrap();
    assert_eq!(hv["status"], "ok");

    // Authenticated smoke path.
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/scope/resolve")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(r#"{"api_version":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn http_cors__default__no_allow_origin_star() {
    let app = app_with_token(&test_token());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let acao = resp
        .headers()
        .get(axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN);
    if let Some(v) = acao {
        assert_ne!(
            v.to_str().unwrap_or(""),
            "*",
            "must not set Access-Control-Allow-Origin: *"
        );
    }
    // Preferred: header absent entirely.
    assert!(
        acao.is_none(),
        "CORS deny-by-default: no ACAO header expected"
    );
}

#[tokio::test]
async fn http_body__over_limit__413() {
    let token = test_token();
    let app = app_with_token(&token);
    let oversized = vec![b'a'; BODY_LIMIT_BYTES + 64];
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/scope/resolve")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .header("content-length", oversized.len().to_string())
                .body(Body::from(oversized))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "body over 1 MiB must be 413"
    );
}

#[tokio::test]
async fn http_policy_denied__403_policy_denied_code() {
    let token = test_token();
    let dispatch: Arc<dyn HttpDispatch> = Arc::new(MockHttpDispatch::new(|_req| {
        Ok(DaemonResponse::Error(ApiError::new(
            POLICY_DENIED_CODE,
            "grant missing for capability",
        )))
    }));
    let state = app_state(dispatch, token.clone());
    let app = build_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/knowledge/query")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(
                    r#"{"api_version":"1","query":"x","principal_id":"p"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let bytes = body_bytes(resp).await;
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // DaemonResponse::Error wire shape: type + payload.code
    let code = v
        .get("payload")
        .and_then(|p| p.get("code"))
        .or_else(|| v.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    assert_eq!(code, POLICY_DENIED_CODE);
}

#[test]
fn http_token_compare__constant_time_helper_unit() {
    let a = b"same-token-value-0123456789abcdef";
    let b = b"same-token-value-0123456789abcdef";
    let c = b"diff-token-value-0123456789abcdef";
    assert!(token_bytes_equal(a, b));
    assert!(!token_bytes_equal(a, c));
    assert!(!token_bytes_equal(a, b"short"));
    assert!(tokens_equal("abc", "abc"));
    assert!(!tokens_equal("abc", "abd"));
}

#[test]
fn http_token_file__owner_only_sddl__not_sy_ba() {
    // Frozen constant must not be the ProgramData SY+BA SDDL.
    assert_eq!(USER_TOKEN_FILE_SDDL, "D:P(A;;FA;;;OW)");
    assert!(!USER_TOKEN_FILE_SDDL.contains("SY"));
    assert!(!USER_TOKEN_FILE_SDDL.contains("BA"));

    // Reject SY+BA-only icacls output (ProgramData style).
    let program_data_style = r#"
C:\Users\x\.ai-brains\http.token NT AUTHORITY\SYSTEM:(F)
                                 BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    let err = verify_owner_acl_output(program_data_style).expect_err("SY+BA must fail");
    assert!(
        err.contains("SYSTEM") || err.contains("owner-only") || err.contains("Administrators"),
        "err={err}"
    );

    // Accept a typical owner/user full ACE (no Everyone) — pure OW (F).
    let owner_style = r#"
C:\Users\x\.ai-brains\http.token DESKTOP-X\user:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    verify_owner_acl_output(owner_style).expect("owner full should pass");
}

#[test]
fn http_token_acl_verify__sy_plus_owner__must_fail() {
    let sy_owner = r#"
C:\Users\x\.ai-brains\http.token NT AUTHORITY\SYSTEM:(F)
                                 DESKTOP-X\user:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    let err = verify_owner_acl_output(sy_owner).expect_err("SY+Owner must fail");
    assert!(
        err.contains("SYSTEM") || err.contains("owner-only"),
        "err={err}"
    );
}

#[test]
fn http_token_acl_verify__unexpected_everyone_ace__must_fail() {
    let everyone = r#"
C:\Users\x\.ai-brains\http.token DESKTOP-X\user:(F)
                                 Everyone:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    let err = verify_owner_acl_output(everyone).expect_err("Everyone must fail");
    assert!(
        err.contains("Everyone")
            || err.contains("EVERYONE")
            || err.contains("broad")
            || err.contains("owner-only"),
        "err={err}"
    );
}

#[test]
fn http_token_acl_verify__pure_owner_f__must_pass() {
    let owner_only = r#"
C:\Users\x\.ai-brains\http.token DESKTOP-X\user:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    verify_owner_acl_output(owner_only).expect("pure OW (F) must pass");

    let owner_rights = r#"
C:\Users\x\.ai-brains\http.token OWNER RIGHTS:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
    verify_owner_acl_output(owner_rights).expect("OWNER RIGHTS (F) must pass");
}

#[test]
fn http_token_load__reverify_acl_roundtrip() {
    // Load path must re-verify ACL (R1-04) without rejecting a freshly written token.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("http.token");
    let token = generate_token().unwrap();
    write_token_file(&path, token.as_str()).expect("write token");
    let loaded = load_or_create_token(&path).expect("load must re-verify and succeed");
    assert_eq!(loaded.as_str(), token.as_str());
}

#[test]
fn http_token_file__write_and_reload_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("http.token");
    let token = generate_token().unwrap();
    write_token_file(&path, token.as_str()).expect("write token");
    let loaded = std::fs::read_to_string(&path).unwrap();
    assert_eq!(loaded.trim(), token.as_str());
    assert!(token.as_str().len() >= 40, ">=256 bits base64url");
}

#[tokio::test]
async fn http_dispatch_port__mock__returns_daemon_response_shape() {
    let token = test_token();
    let mock = Arc::new(MockHttpDispatch::new(|req| match req {
        DaemonRequest::ResolveScope(_) => Ok(DaemonResponse::Pong),
        other => Ok(DaemonResponse::unsupported(&format!("{other:?}"))),
    }));
    let state = app_state(mock.clone() as Arc<dyn HttpDispatch>, token.clone());
    let app = build_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/scope/resolve")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(r#"{"api_version":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = body_bytes(resp).await;
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["type"], "pong");

    let calls = mock.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert!(matches!(calls[0], DaemonRequest::ResolveScope(_)));
}

#[tokio::test]
async fn http_v1_health__unauthenticated__ok() {
    let app = app_with_token(&test_token());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
