//! Named-pipe daemon request/response protocol (line-delimited JSON).
//!
//! **Design locks (T158)**
//! - Serde: `tag = "type"`, `content = "payload"`, `rename_all = "snake_case"`.
//! - Dependencies: **serde + ai-brains-contracts only** — no domain logic.
//! - Unknown `type` fails deserialize (fail-closed; no `#[serde(other)]`).
//! - Legacy `ping` / `ingest` / `sync` / `shutdown` wire remains deserializable.
//! - Full governed handlers land in T159; daemon stubs return
//!   [`UNSUPPORTED_OPERATION`] via shared dispatch.

use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::briefings::{
    HandlePreviewDto, InspectEvidenceRequest, PersonalBriefingRequest, PersonalBriefingResponse,
    ProgressiveQueryResponse, ProjectBriefingRequest, ProjectBriefingResponse,
    QueryKnowledgeRequest,
};
use ai_brains_contracts::erasure::{ErasureAcceptedResponse, RequestErasureRequest};
use ai_brains_contracts::ingest::{IngestRequest, IngestResponse};
use ai_brains_contracts::knowledge::{
    ConclusionProposedResponse, DecisionProposedResponse, ProposeConclusionRequest,
    ProposeDecisionRequest,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_contracts::review::{
    ListReviewItemsRequest, ResolveReviewItemRequest, ReviewQueueResponse, ReviewResolvedResponse,
};
use ai_brains_contracts::scopes::{ResolveScopeRequest, ScopeResolvedResponse};
use ai_brains_contracts::sources::{InspectSourceRequest, SourceDto};
use serde::{Deserialize, Serialize};

/// Stable error code when a request variant is recognized but not yet handled (T159).
pub const UNSUPPORTED_OPERATION: &str = "UNSUPPORTED_OPERATION";

/// Inbound daemon command (CLI / agent / desktop → `ai-brainsd`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonRequest {
    Ping,
    Ingest(IngestRequest),
    Sync(BridgeRecord),
    Shutdown,
    ResolveScope(ResolveScopeRequest),
    ProjectBriefing(ProjectBriefingRequest),
    PersonalBriefing(PersonalBriefingRequest),
    QueryKnowledge(QueryKnowledgeRequest),
    InspectEvidence(InspectEvidenceRequest),
    InspectSource(InspectSourceRequest),
    ProposeConclusion(ProposeConclusionRequest),
    ProposeDecision(ProposeDecisionRequest),
    ListReviewItems(ListReviewItemsRequest),
    ResolveReviewItem(ResolveReviewItemRequest),
    RequestErasure(RequestErasureRequest),
}

/// Outbound daemon reply (or multi-line Sync query framing outside this enum).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonResponse {
    Pong,
    Ingest(IngestResponse),
    Sync { success: bool },
    Error(ApiError),
    ScopeResolved(ScopeResolvedResponse),
    ProjectBriefing(ProjectBriefingResponse),
    PersonalBriefing(PersonalBriefingResponse),
    QueryKnowledge(ProgressiveQueryResponse),
    EvidencePreview(HandlePreviewDto),
    Source(SourceDto),
    ConclusionProposed(ConclusionProposedResponse),
    DecisionProposed(DecisionProposedResponse),
    ReviewList(ReviewQueueResponse),
    ReviewResolved(ReviewResolvedResponse),
    ErasureAccepted(ErasureAcceptedResponse),
}

impl DaemonResponse {
    /// Build a structured protocol error.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error(ApiError::new(code, message))
    }

    /// Stub response for variants not yet implemented (T159).
    pub fn unsupported(operation: &str) -> Self {
        Self::Error(ApiError::new(
            UNSUPPORTED_OPERATION,
            format!("{operation} is not implemented yet (T159)"),
        ))
    }
}
