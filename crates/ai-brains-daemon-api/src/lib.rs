//! Named-pipe daemon request/response protocol (line-delimited JSON).
//!
//! **Design locks (T158 / T159)**
//! - Serde: `tag = "type"`, `content = "payload"`, `rename_all = "snake_case"`.
//! - Dependencies: **serde + ai-brains-contracts only** — no domain logic.
//! - Unknown `type` fails deserialize (fail-closed; no `#[serde(other)]`).
//! - Legacy `ping` / `ingest` / `sync` / `shutdown` wire remains deserializable.
//! - Governed protocol surface is complete; handlers live in `ai-brainsd`
//!   (T159). This crate remains DTO-only.
//! - [`DaemonResponse::unsupported`] is a generic helper for callers that need
//!   an explicit [`UNSUPPORTED_OPERATION`] reply — not a T159 residual.

use ai_brains_contracts::bridge::BridgeRecord;
use ai_brains_contracts::briefings::{
    HandlePreviewDto, InspectEvidenceRequest, PersonalBriefingRequest, PersonalBriefingResponse,
    ProgressiveQueryResponse, ProjectBriefingRequest, ProjectBriefingResponse,
    QueryKnowledgeRequest,
};
use ai_brains_contracts::erasure::{
    ContentEnvelopeWipedResponse, ErasureAcceptedResponse, RequestErasureRequest,
    WipeContentEnvelopeRequest,
};
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

/// Stable error code when a recognized request cannot be handled by this path.
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
    /// Governed cryptographic erase for envelope-backed content (T165).
    WipeContentEnvelope(WipeContentEnvelopeRequest),
}

/// Outbound daemon reply (or multi-line Sync query framing outside this enum).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonResponse {
    Pong,
    Ingest(IngestResponse),
    Sync {
        success: bool,
    },
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
    /// CE wipe result (T165); never claims NIST Purge.
    ContentEnvelopeWiped(ContentEnvelopeWipedResponse),
}

impl DaemonResponse {
    /// Build a structured protocol error.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error(ApiError::new(code, message))
    }

    /// Build an [`UNSUPPORTED_OPERATION`] error for an unrecognized or unhandled op.
    pub fn unsupported(operation: &str) -> Self {
        Self::Error(ApiError::new(
            UNSUPPORTED_OPERATION,
            format!("{operation} is not supported on this path"),
        ))
    }
}
