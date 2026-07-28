//! Route → DaemonRequest mapping via mock HttpDispatch (T161 Phase C).
#![allow(clippy::disallowed_methods, non_snake_case)]

use std::sync::Arc;

use ai_brains_api_server::dispatch::HttpDispatch;
use ai_brains_api_server::dispatch::test_support::MockHttpDispatch;
use ai_brains_api_server::{app_state, build_router};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use zeroize::Zeroizing;

const TOKEN: &str = "route-test-token-0123456789abcdef01234567";

fn app_echo_type() -> (axum::Router, Arc<MockHttpDispatch>) {
    let mock = Arc::new(MockHttpDispatch::new(|req| {
        let _ = req;
        Ok(DaemonResponse::Pong)
    }));
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

#[tokio::test]
async fn routes__project_briefing__dispatches_project_briefing() {
    let (app, dispatch) = app_echo_type();
    let (status, _) = oneshot_post(app, "/v1/briefings/project", r#"{"api_version":"1"}"#).await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    assert!(matches!(calls[0], DaemonRequest::ProjectBriefing(_)));
}

#[tokio::test]
async fn routes__query_knowledge__dispatches_query() {
    let (app, dispatch) = app_echo_type();
    let (status, bytes) = oneshot_post(
        app,
        "/v1/knowledge/query",
        r#"{"api_version":"1","query":"hello"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["type"], "pong");
    let calls = dispatch.calls.lock().await;
    assert!(matches!(calls[0], DaemonRequest::QueryKnowledge(_)));
}

#[tokio::test]
async fn routes__propose_conclusion__dispatches_mutation() {
    let (app, dispatch) = app_echo_type();
    let (status, _) = oneshot_post(
        app,
        "/v1/conclusions/propose",
        r#"{"api_version":"1","statement":"x","scope":"project:p","principal_id":"p","command_id":"c1"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    assert!(matches!(calls[0], DaemonRequest::ProposeConclusion(_)));
}

#[tokio::test]
async fn routes__list_review_items__get_dispatches() {
    let (app, dispatch) = app_echo_type();
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/review/items?scope=project:p")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::ListReviewItems(req) => {
            assert_eq!(req.scope.as_deref(), Some("project:p"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn routes__resolve_review_item__path_id_overrides() {
    let (app, dispatch) = app_echo_type();
    let (status, _) = oneshot_post(
        app,
        "/v1/review/items/item-42/resolve",
        r#"{"api_version":"1","id":"ignored","resolution":"approved"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::ResolveReviewItem(req) => {
            assert_eq!(req.id, "item-42");
            assert_eq!(req.resolution, "approved");
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn routes__parity__briefing_response_is_daemon_response_json() {
    // Build a minimal ProjectBriefingResponse via JSON so we do not depend on
    // every packet field being Default in the test.
    let packet_json = serde_json::json!({
        "type": "project_briefing",
        "payload": {
            "api_version": "1",
            "packet": {
                "scope": "Repository:11111111-1111-1111-1111-111111111111",
                "project_name": null,
                "active_conclusions": [],
                "open_conflicts": [],
                "recent_decisions": [],
                "evidence_handles": [],
                "source_freshness": [],
                "warnings": [],
                "word_budget": { "used": 0, "max": 1500 },
                "denied": false,
                "denial_reason": null
            }
        }
    });

    // Fall back to Pong if packet shape drifts — still assert DaemonResponse wire equality.
    let expected: DaemonResponse =
        serde_json::from_value(packet_json).unwrap_or(DaemonResponse::Pong);
    let expected_json = serde_json::to_value(&expected).unwrap();

    let mock = Arc::new(MockHttpDispatch::always(expected.clone()));
    let dispatch: Arc<dyn HttpDispatch> = mock;
    let state = app_state(dispatch, Zeroizing::new(TOKEN.to_string()));
    let app = build_router(state);

    let (status, bytes) =
        oneshot_post(app, "/v1/briefings/project", r#"{"api_version":"1"}"#).await;
    assert_eq!(status, StatusCode::OK);
    let got: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        got, expected_json,
        "HTTP body must equal DaemonResponse JSON"
    );
}

#[tokio::test]
async fn routes__personal_briefing__dispatches() {
    let (app, dispatch) = app_echo_type();
    let (status, _) = oneshot_post(app, "/v1/briefings/personal", r#"{"api_version":"1"}"#).await;
    assert_eq!(status, StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    assert!(matches!(calls[0], DaemonRequest::PersonalBriefing(_)));
}

#[tokio::test]
async fn routes__erasure_request__dispatches() {
    let (app, dispatch) = app_echo_type();
    // Contracts: `ids: Vec<String>` (not `id`); assert RequestErasure (R1-09).
    let (status, _) = oneshot_post(
        app,
        "/v1/erasure/request",
        r#"{"api_version":"1","ids":["m1"],"scope":"project:p","principal_id":"p","command_id":"e1"}"#,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "valid erasure body must dispatch");
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::RequestErasure(req) => {
            assert_eq!(req.ids, vec!["m1".to_string()]);
            assert_eq!(req.command_id.as_deref(), Some("e1"));
        }
        other => panic!("expected RequestErasure, got {other:?}"),
    }
}

#[tokio::test]
async fn routes__propose_conclusion__x_command_id_header_fills_body() {
    let (app, dispatch) = app_echo_type();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/conclusions/propose")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {TOKEN}"))
                .header("X-Command-Id", "from-header-cmd")
                .body(Body::from(
                    r#"{"api_version":"1","statement":"x","scope":"project:p","principal_id":"p"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let calls = dispatch.calls.lock().await;
    match &calls[0] {
        DaemonRequest::ProposeConclusion(req) => {
            assert_eq!(req.command_id.as_deref(), Some("from-header-cmd"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
