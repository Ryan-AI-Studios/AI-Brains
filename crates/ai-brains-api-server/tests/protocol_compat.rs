//! T180 P-HTTP protocol compatibility: api_version honesty, DTO goldens, unknown route.
#![allow(clippy::disallowed_methods, non_snake_case)]

use std::sync::Arc;

use ai_brains_api_server::dispatch::HttpDispatch;
use ai_brains_api_server::dispatch::test_support::MockHttpDispatch;
use ai_brains_api_server::{app_state, build_router};
use ai_brains_contracts::briefings::QueryKnowledgeRequest;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use zeroize::Zeroizing;

const TOKEN: &str = "t180-http-compat-token-0123456789abcdef01";

fn app_echo() -> (axum::Router, Arc<MockHttpDispatch>) {
    let mock = Arc::new(MockHttpDispatch::new(|_req| Ok(DaemonResponse::Pong)));
    let dispatch: Arc<dyn HttpDispatch> = mock.clone();
    let state = app_state(dispatch, Zeroizing::new(TOKEN.to_string()));
    (build_router(state), mock)
}

async fn oneshot_post(app: axum::Router, path: &str, body: &str) -> (StatusCode, Vec<u8>) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, bytes)
}

/// Inject additive unknown fields into a JSON object body (F29 pattern for HTTP DTOs).
fn with_extra_fields(base: &str) -> String {
    let mut val: serde_json::Value = serde_json::from_str(base).expect("base JSON");
    if let Some(obj) = val.as_object_mut() {
        obj.insert("_test_unknown_string".into(), "unknown_value".into());
        obj.insert("_test_unknown_number".into(), serde_json::json!(42));
        obj.insert(
            "_test_unknown_object".into(),
            serde_json::json!({"nested": true}),
        );
    }
    serde_json::to_string(&val).expect("re-ser")
}

// ---------------------------------------------------------------------------
// T180-H-dto-goldens
// ---------------------------------------------------------------------------

#[test]
fn t180_h_dto_goldens__query_knowledge_fixture__deserializes() {
    // T180-H-dto-goldens
    let raw = include_str!("fixtures/query_knowledge_v1.json");
    let decoded: QueryKnowledgeRequest = serde_json::from_str(raw).expect("query fixture");
    assert_eq!(decoded.api_version, "1");
    assert_eq!(decoded.query, "budget decision");
    assert_eq!(decoded.limit, Some(5));
}

#[test]
fn t180_h_dto_goldens__resolve_scope_fixture__deserializes() {
    let raw = include_str!("fixtures/resolve_scope_v1.json");
    let decoded: ResolveScopeRequest = serde_json::from_str(raw).expect("scope fixture");
    assert_eq!(decoded.api_version, "1");
    assert_eq!(decoded.cwd.as_deref(), Some("C:/dev/AI-Brains"));
}

// ---------------------------------------------------------------------------
// T180-H-api-version-1 / T180-H-api-version-unenforced
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t180_h_api_version_1__query_knowledge__accepted() {
    // T180-H-api-version-1
    let (app, dispatch) = app_echo();
    let (status, _) = oneshot_post(
        app,
        "/v1/knowledge/query",
        r#"{"api_version":"1","query":"hello"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::QueryKnowledge(req) => assert_eq!(req.api_version, "1"),
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn t180_h_api_version_unenforced__version_2__also_accepted() {
    // T180-H-api-version-unenforced — honesty: update when enforcement lands.
    let (app, dispatch) = app_echo();
    let (status, _) = oneshot_post(
        app,
        "/v1/knowledge/query",
        r#"{"api_version":"2","query":"hello"}"#,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "api_version 2 must still be accepted today (not enforced)"
    );
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::QueryKnowledge(req) => assert_eq!(req.api_version, "2"),
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn t180_h_api_version_unenforced__version_2__resolve_scope() {
    let (app, dispatch) = app_echo();
    let (status, _) = oneshot_post(
        app,
        "/v1/scope/resolve",
        r#"{"api_version":"2","cwd":"C:/tmp","force_personal":false}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::ResolveScope(req) => assert_eq!(req.api_version, "2"),
        other => panic!("unexpected {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// T180-H-additive-extra-field
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t180_h_additive_extra_field__query_body__accepted() {
    // T180-H-additive-extra-field
    let (app, dispatch) = app_echo();
    let body = with_extra_fields(r#"{"api_version":"1","query":"hello"}"#);
    let (status, _) = oneshot_post(app, "/v1/knowledge/query", &body).await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    assert!(matches!(calls[0], DaemonRequest::QueryKnowledge(_)));
}

#[test]
fn t180_h_additive_extra_field__dto_level__resolve_scope() {
    let base = include_str!("fixtures/resolve_scope_v1.json");
    let mut val: serde_json::Value = serde_json::from_str(base).expect("parse");
    if let Some(obj) = val.as_object_mut() {
        obj.insert("_test_unknown_string".into(), "x".into());
        obj.insert("_test_unknown_number".into(), serde_json::json!(1));
    }
    let _: ResolveScopeRequest =
        serde_json::from_value(val).expect("public HTTP DTO must tolerate additive fields");
}

// ---------------------------------------------------------------------------
// T180-H-unknown-route
// ---------------------------------------------------------------------------

#[tokio::test]
async fn t180_h_unknown_route__returns_404() {
    // T180-H-unknown-route — defined shape today: bare Axum 404 (no custom JSON envelope).
    // Documented in Docs/PROTOCOL-COMPAT.md §9.5.
    let (app, _) = app_echo();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/does-not-exist")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    // No product-defined error envelope for unmapped routes — body may be empty or default.
}
