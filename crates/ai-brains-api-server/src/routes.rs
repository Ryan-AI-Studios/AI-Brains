//! Thin REST adapters: path + JSON → `DaemonRequest` → `HttpDispatch` → HTTP.

use std::sync::Arc;

use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use ai_brains_contracts::briefings::{
    InspectEvidenceRequest, PersonalBriefingRequest, ProjectBriefingRequest, QueryKnowledgeRequest,
};
use ai_brains_contracts::erasure::RequestErasureRequest;
use ai_brains_contracts::knowledge::{ProposeConclusionRequest, ProposeDecisionRequest};
use ai_brains_contracts::review::{ListReviewItemsRequest, ResolveReviewItemRequest};
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_contracts::sources::InspectSourceRequest;
use ai_brains_daemon_api::DaemonRequest;

use crate::auth::{AuthConfig, Authenticated};
use crate::dispatch::HttpDispatch;
use crate::error::{ApiHttpError, map_daemon_response};

/// Default maximum request body size (1 MiB).
pub const BODY_LIMIT_BYTES: usize = 1024 * 1024;

/// Shared router state (dispatch is type-erased so handlers stay non-generic).
#[derive(Clone)]
pub struct AppState {
    pub dispatch: Arc<dyn HttpDispatch>,
    pub auth: AuthConfig,
}

impl FromRef<AppState> for AuthConfig {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

/// Build the `/v1` router with auth, body limit, and trace layers.
///
/// CORS: **no** permissive layer — deny by default (no `Access-Control-Allow-Origin: *`).
pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/scope/resolve", post(resolve_scope))
        .route("/v1/briefings/project", post(project_briefing))
        .route("/v1/briefings/personal", post(personal_briefing))
        .route("/v1/knowledge/query", post(query_knowledge))
        .route("/v1/evidence/inspect", post(inspect_evidence))
        .route("/v1/sources/inspect", post(inspect_source))
        .route("/v1/conclusions/propose", post(propose_conclusion))
        .route("/v1/decisions/propose", post(propose_decision))
        .route("/v1/review/items", get(list_review_items))
        .route("/v1/review/items/{id}/resolve", post(resolve_review_item))
        .route("/v1/erasure/request", post(request_erasure));

    Router::new()
        .route("/health", get(health))
        .route("/v1/health", get(health))
        .merge(protected)
        .layer(RequestBodyLimitLayer::new(BODY_LIMIT_BYTES))
        // Trace without logging Authorization values — handlers never log the token.
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthBody { status: "ok" }))
}

async fn dispatch_request(
    state: &AppState,
    request: DaemonRequest,
) -> Result<Response, ApiHttpError> {
    let resp = state.dispatch.dispatch(request).await?;
    Ok(map_daemon_response(resp))
}

/// If body `command_id` is empty/missing, fill from `X-Command-Id` (R1-06).
fn fill_command_id_from_header(command_id: &mut Option<String>, headers: &HeaderMap) {
    let body_empty = command_id
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if !body_empty {
        return;
    }
    let Some(raw) = headers
        .get("x-command-id")
        .or_else(|| headers.get("X-Command-Id"))
        .and_then(|v| v.to_str().ok())
    else {
        return;
    };
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        *command_id = Some(trimmed.to_string());
    }
}

async fn resolve_scope(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<ResolveScopeRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::ResolveScope(body)).await
}

async fn project_briefing(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<ProjectBriefingRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::ProjectBriefing(body)).await
}

async fn personal_briefing(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<PersonalBriefingRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::PersonalBriefing(body)).await
}

async fn query_knowledge(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<QueryKnowledgeRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::QueryKnowledge(body)).await
}

async fn inspect_evidence(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<InspectEvidenceRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::InspectEvidence(body)).await
}

async fn inspect_source(
    State(state): State<AppState>,
    _auth: Authenticated,
    Json(body): Json<InspectSourceRequest>,
) -> Result<Response, ApiHttpError> {
    dispatch_request(&state, DaemonRequest::InspectSource(body)).await
}

async fn propose_conclusion(
    State(state): State<AppState>,
    _auth: Authenticated,
    headers: HeaderMap,
    Json(mut body): Json<ProposeConclusionRequest>,
) -> Result<Response, ApiHttpError> {
    fill_command_id_from_header(&mut body.command_id, &headers);
    dispatch_request(&state, DaemonRequest::ProposeConclusion(body)).await
}

async fn propose_decision(
    State(state): State<AppState>,
    _auth: Authenticated,
    headers: HeaderMap,
    Json(mut body): Json<ProposeDecisionRequest>,
) -> Result<Response, ApiHttpError> {
    fill_command_id_from_header(&mut body.command_id, &headers);
    dispatch_request(&state, DaemonRequest::ProposeDecision(body)).await
}

/// Query string for GET `/v1/review/items` (mirrors contracts fields).
#[derive(Debug, Default, Deserialize)]
struct ListReviewQuery {
    #[serde(default)]
    principal_id: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

async fn list_review_items(
    State(state): State<AppState>,
    _auth: Authenticated,
    Query(q): Query<ListReviewQuery>,
) -> Result<Response, ApiHttpError> {
    let body = ListReviewItemsRequest {
        api_version: ai_brains_contracts::review::API_VERSION.to_string(),
        principal_id: q.principal_id,
        scope: q.scope,
        status: q.status,
    };
    dispatch_request(&state, DaemonRequest::ListReviewItems(body)).await
}

async fn resolve_review_item(
    State(state): State<AppState>,
    _auth: Authenticated,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(mut body): Json<ResolveReviewItemRequest>,
) -> Result<Response, ApiHttpError> {
    // Path id is authoritative for REST.
    body.id = id;
    fill_command_id_from_header(&mut body.command_id, &headers);
    dispatch_request(&state, DaemonRequest::ResolveReviewItem(body)).await
}

async fn request_erasure(
    State(state): State<AppState>,
    _auth: Authenticated,
    headers: HeaderMap,
    Json(mut body): Json<RequestErasureRequest>,
) -> Result<Response, ApiHttpError> {
    fill_command_id_from_header(&mut body.command_id, &headers);
    dispatch_request(&state, DaemonRequest::RequestErasure(body)).await
}
