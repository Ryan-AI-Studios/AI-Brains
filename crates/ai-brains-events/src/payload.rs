use ai_brains_core::ids::{
    BriefingId, ConclusionId, ConflictId, ContentKeyId, DecisionId, EvidenceId, GrantId, MemoryId,
    PrincipalId, ProjectId, QueryTraceId, RecipeId, ReviewItemId, SessionId, SourceId,
    SourceVersionId, TombstoneId, TransactionId, WorkspaceId,
};
use ai_brains_core::model_provenance::ModelProvenance;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::review::{ReviewCriticality, ReviewSubjectKind};
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInitializedPayload {
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKitCreatedPayload {
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRegisteredPayload {
    pub project_id: ProjectId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAliasAddedPayload {
    pub project_id: ProjectId,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub session_id: SessionId,
    pub project_id: ProjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPromptRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistantFinalRecordedPayload {
    pub session_id: SessionId,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompletedPayload {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFailedPayload {
    pub session_id: SessionId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryPinnedPayload {
    pub memory_id: MemoryId,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryForgottenPayload {
    pub memory_id: MemoryId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRestoredPayload {
    pub memory_id: MemoryId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummaryCreatedPayload {
    pub session_id: SessionId,
    pub project_id: Option<ProjectId>,
    pub memory_id: MemoryId,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictDetectedPayload {
    pub conflict_id: ConflictId,
    pub memory_ids: Vec<MemoryId>,
    pub session_id: SessionId,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipePromotedPayload {
    pub recipe_id: RecipeId,
    pub name: String,
    pub content: String,
    pub steps: Vec<String>,
    pub source_memory_ids: Vec<MemoryId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySynthesizedPayload {
    pub memory_id: MemoryId,
    pub level: u32,
    pub source_memory_ids: Vec<MemoryId>,
    pub project_id: ProjectId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackMetricPayload {
    pub metric_kind: String,
    pub value: String,
    pub session_id: Option<SessionId>,
    pub project_id: Option<ProjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionRecordedPayload {
    pub session_id: SessionId,
    pub tx_id: Option<TransactionId>,
    pub predicted_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyOutcomeRecordedPayload {
    pub tx_id: TransactionId,
    pub status: String,
    pub affected_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestGateRejectedPayload {
    pub session_id: SessionId,
    /// Human-readable reason for rejection.
    pub reason: String,
    /// Predicted failure probability (0.0 – 1.0) from Ledgerful.
    pub failure_probability: f64,
    /// Whether Ledgerful detected ledger drift.
    pub drift_detected: bool,
    /// Risk level string: "low", "medium", "high", or "critical".
    pub risk_level: String,
    /// Full explanation payload from the verification engine.
    pub explanation: String,
}

impl PartialEq for IngestGateRejectedPayload {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.reason == other.reason
            && self.failure_probability.to_bits() == other.failure_probability.to_bits()
            && self.drift_detected == other.drift_detected
            && self.risk_level == other.risk_level
            && self.explanation == other.explanation
    }
}

impl Eq for IngestGateRejectedPayload {}

/// Legacy decision pin: `decision_id` remains [`MemoryId`] (not governed [`DecisionId`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRecordedPayload {
    pub decision_id: MemoryId,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<TransactionId>,
}

// --- Governed memory payloads (T148) ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRegisteredPayload {
    pub source_id: SourceId,
    pub kind: SourceKind,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    /// Scope identity key (e.g. `Personal:{user_id}`); absent on historical events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceObservedPayload {
    pub source_id: SourceId,
    #[serde(with = "time::serde::rfc3339")]
    pub observed_at: OffsetDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVersionRecordedPayload {
    pub source_id: SourceId,
    pub version_id: SourceVersionId,
    pub fingerprint: String,
    #[serde(with = "time::serde::rfc3339")]
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceUnavailablePayload {
    pub source_id: SourceId,
    pub reason: String,
    #[serde(with = "time::serde::rfc3339")]
    pub marked_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecordedPayload {
    pub evidence_id: EvidenceId,
    pub source_id: SourceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_version_id: Option<SourceVersionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provenance: Option<ModelProvenance>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSupersededPayload {
    pub evidence_id: EvidenceId,
    pub superseded_by: EvidenceId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionProposedPayload {
    pub conclusion_id: ConclusionId,
    pub statement: String,
    pub evidence_ids: Vec<EvidenceId>,
    pub proposer: PrincipalId,
    /// Domain valid-from (≠ recorded `Envelope.occurred_at`). Historical fixtures
    /// omit this; projection defaults to `occurred_at` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_from: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_until: Option<OffsetDateTime>,
    /// Scope identity key (e.g. `Repository:{project_id}`); empty when absent.
    #[serde(default)]
    pub scope: String,
    /// Protected category name (PascalCase) when gated for human approval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_category: Option<String>,
    /// True when proposed without supporting evidence ids.
    #[serde(default)]
    pub unsupported: bool,
    /// Model lineage for synthesis-derived candidates (no CoT). Historical fixtures omit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_provenance: Option<ModelProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionActivatedPayload {
    pub conclusion_id: ConclusionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionConfirmedPayload {
    pub conclusion_id: ConclusionId,
    pub approver: PrincipalId,
    #[serde(with = "time::serde::rfc3339")]
    pub confirmed_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionMarkedStalePayload {
    pub conclusion_id: ConclusionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_source_version_id: Option<SourceVersionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    /// Optional source that triggered staleness (T149 revalidation matching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<SourceId>,
}

impl ConclusionMarkedStalePayload {
    /// Require at least one of version id or non-empty unavailable reason.
    pub fn validate(&self) -> Result<(), crate::errors::EventError> {
        let has_version = self.changed_source_version_id.is_some();
        let has_reason = self
            .unavailable_reason
            .as_ref()
            .is_some_and(|r| !r.trim().is_empty());
        if !has_version && !has_reason {
            return Err(crate::errors::EventError::InvalidPayload(
                "ConclusionMarkedStale requires changed_source_version_id or non-empty unavailable_reason"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub fn try_new(
        conclusion_id: ConclusionId,
        changed_source_version_id: Option<SourceVersionId>,
        unavailable_reason: Option<String>,
    ) -> Result<Self, crate::errors::EventError> {
        Self::try_new_with_source(
            conclusion_id,
            changed_source_version_id,
            unavailable_reason,
            None,
        )
    }

    pub fn try_new_with_source(
        conclusion_id: ConclusionId,
        changed_source_version_id: Option<SourceVersionId>,
        unavailable_reason: Option<String>,
        source_id: Option<SourceId>,
    ) -> Result<Self, crate::errors::EventError> {
        let payload = Self {
            conclusion_id,
            changed_source_version_id,
            unavailable_reason,
            source_id,
        };
        payload.validate()?;
        Ok(payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionDisputedPayload {
    pub conclusion_id: ConclusionId,
    pub disputant: PrincipalId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionSupersededPayload {
    pub conclusion_id: ConclusionId,
    pub superseded_by: ConclusionId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConclusionRejectedPayload {
    pub conclusion_id: ConclusionId,
    pub rejector: PrincipalId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionProposedPayload {
    pub decision_id: DecisionId,
    pub title: String,
    pub statement: String,
    pub proposer: PrincipalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclusion_ids: Option<Vec<ConclusionId>>,
    /// Supporting evidence links (optional; additive schema default None).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ids: Option<Vec<EvidenceId>>,
    /// Domain valid-from (optional; historical fixtures omit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_from: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_until: Option<OffsetDateTime>,
    /// Scope identity key; empty when absent.
    #[serde(default)]
    pub scope: String,
}

/// Locked T148 shape: identity + proposal event + approver + approval time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionApprovedPayload {
    pub decision_id: DecisionId,
    pub proposal_event_id: Uuid,
    pub approver: PrincipalId,
    #[serde(with = "time::serde::rfc3339")]
    pub approved_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionSupersededPayload {
    pub decision_id: DecisionId,
    pub superseded_by: DecisionId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRevokedPayload {
    pub decision_id: DecisionId,
    pub revoker: PrincipalId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRegisteredPayload {
    pub workspace_id: WorkspaceId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryJoinedWorkspacePayload {
    pub workspace_id: WorkspaceId,
    pub project_id: ProjectId,
}

/// Default privacy for historical `ScopeGrantIssued` events that omit the field.
fn default_scope_grant_privacy() -> Privacy {
    Privacy::LocalOnly
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeGrantIssuedPayload {
    pub grant_id: GrantId,
    pub principal_id: PrincipalId,
    pub scope: ScopeRef,
    pub capability: GrantCapability,
    /// Grant privacy used for `strictest_wins` / cloud-route blocking.
    /// Defaults to [`Privacy::LocalOnly`] when absent from historical events.
    #[serde(default = "default_scope_grant_privacy")]
    pub privacy: Privacy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeGrantRevokedPayload {
    pub grant_id: GrantId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrincipalRegisteredPayload {
    pub principal_id: PrincipalId,
    /// Wire representation of principal kind as PascalCase of
    /// [`ai_brains_core::principal::PrincipalKind`] for round-trip:
    /// `Human` | `Agent` | `Connector` | `System` | `Service` | `Other:{label}`.
    /// Legacy free-form values (including historical `Service`) parse via
    /// `parse_principal_kind` into known variants or `Other`.
    pub kind: String,
    pub display_name: String,
    /// Source kinds this principal is bound to; empty when unbound (default for old events).
    #[serde(default)]
    pub bound_source_kinds: Vec<SourceKind>,
    /// Capabilities this principal is bound to; empty when unbound (default for old events).
    #[serde(default)]
    pub bound_capabilities: Vec<GrantCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewItemOpenedPayload {
    pub review_item_id: ReviewItemId,
    pub subject: String,
    pub opened_by: PrincipalId,
    /// Structured subject kind (Decision / Source / …). Defaults for historical rows.
    #[serde(default)]
    pub subject_kind: ReviewSubjectKind,
    /// Subject entity id as string (decision/source/conclusion uuid).
    #[serde(default)]
    pub subject_id: String,
    #[serde(default)]
    pub criticality: ReviewCriticality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_conclusion_id: Option<ConclusionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_decision_id: Option<DecisionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_source_id: Option<SourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewItemResolvedPayload {
    pub review_item_id: ReviewItemId,
    pub resolution: String,
    pub resolved_by: PrincipalId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BriefingGeneratedPayload {
    pub briefing_id: BriefingId,
    pub kind: String,
    pub evidence_ids: Vec<EvidenceId>,
    pub query_trace_id: Option<QueryTraceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTraceRecordedPayload {
    pub query_trace_id: QueryTraceId,
    pub query_text: String,
    pub evidence_ids: Vec<EvidenceId>,
    /// Scope identity key (e.g. `Repository:{id}`). Empty on legacy events.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scope: String,
    /// Principal id that ran the query. Empty on legacy events.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub principal_id: String,
    /// Policy evaluator label (e.g. `DefaultPolicyEvaluator`). Empty on legacy events.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub applied_policy: String,
    /// Serialized ranking components JSON. Empty/`{}` on legacy events.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ranking_json: String,
    /// Optional freshness summary string (or JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness_summary: Option<String>,
    /// Optional open-conflict summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentErasureRequestedPayload {
    pub content_key_id: ContentKeyId,
    pub requester: PrincipalId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentErasedPayload {
    pub content_key_id: ContentKeyId,
    pub tombstone_id: TombstoneId,
}

/// Durable erasure ticket (T159) — acceptance tracking only, not CE wipe.
///
/// Distinct from [`ContentErasureRequestedPayload`] / [`ContentErasedPayload`]
/// which require `ContentKeyId` (P8). No crypto claims.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureTicketAcceptedPayload {
    /// Stable ticket / request id (deterministic from command_id when present).
    pub request_id: String,
    pub requester: PrincipalId,
    /// Target record / aggregate ids from the wire request.
    #[serde(default)]
    pub target_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Open a claim-level conflict (T150; distinct from legacy memory ConflictDetected).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimConflictOpenedPayload {
    pub conflict_id: ConflictId,
    pub claim_a_kind: String,
    pub claim_a_id: String,
    pub claim_b_kind: String,
    pub claim_b_id: String,
    pub scope: String,
    pub explanation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_from: Option<OffsetDateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub valid_until: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimConflictResolvedPayload {
    pub conflict_id: ConflictId,
    pub resolution: String,
    pub resolved_by: PrincipalId,
}

/// Bind or refresh a repository project identity (normalized remote hash + optional ledgerful id).
///
/// Projected into `repository_identity_projection`. When `force` is true and
/// `remote_url_hash` is set, any other project holding that hash is cleared so
/// the unique index allows rebind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryIdentityRegisteredPayload {
    pub project_id: ProjectId,
    /// SHA-256 hex of normalized remote URL; omit/None when only binding ledgerful id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_url_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledgerful_project_id: Option<String>,
    /// When true, rebind: clear other projects that held this remote_url_hash.
    #[serde(default)]
    pub force: bool,
}

/// Register a normalized path alias for a repository project (Windows/WSL forms).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryPathAliasAddedPayload {
    pub project_id: ProjectId,
    pub normalized_path: String,
}

/// Policy matrix allow/deny audit row (T151) — reason codes only, never claim/statement text.
///
/// Projected into `policy_decision_log` so `rebuild_projections` rehydrates audit history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecisionRecordedPayload {
    pub principal_id: PrincipalId,
    pub capability: GrantCapability,
    pub scope_key: String,
    pub allowed: bool,
    pub reason_code: String,
    /// Content/route privacy considered at decision time (not secret content).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy: Option<Privacy>,
}

/// Internally tagged payload (`type` field, PascalCase).
///
/// [`Payload::Unknown`] preserves the full original JSON object so shadow/append
/// re-serialization cannot strip future event fields (T148 R0 option A).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    SystemInitialized(SystemInitializedPayload),
    RecoveryKitCreated(RecoveryKitCreatedPayload),
    ProjectRegistered(ProjectRegisteredPayload),
    ProjectAliasAdded(ProjectAliasAddedPayload),
    SessionStarted(SessionStartedPayload),
    UserPromptRecorded(UserPromptRecordedPayload),
    AssistantFinalRecorded(AssistantFinalRecordedPayload),
    SessionCompleted(SessionCompletedPayload),
    SessionFailed(SessionFailedPayload),
    MemoryPinned(MemoryPinnedPayload),
    MemoryForgotten(MemoryForgottenPayload),
    MemoryRestored(MemoryRestoredPayload),
    SessionSummaryCreated(SessionSummaryCreatedPayload),
    ConflictDetected(ConflictDetectedPayload),
    RecipePromoted(RecipePromotedPayload),
    MemorySynthesized(MemorySynthesizedPayload),
    FeedbackMetric(FeedbackMetricPayload),
    PredictionRecorded(PredictionRecordedPayload),
    VerifyOutcomeRecorded(VerifyOutcomeRecordedPayload),
    DecisionRecorded(DecisionRecordedPayload),
    IngestGateRejected(IngestGateRejectedPayload),
    SourceRegistered(SourceRegisteredPayload),
    SourceObserved(SourceObservedPayload),
    SourceVersionRecorded(SourceVersionRecordedPayload),
    SourceUnavailable(SourceUnavailablePayload),
    EvidenceRecorded(EvidenceRecordedPayload),
    EvidenceSuperseded(EvidenceSupersededPayload),
    ConclusionProposed(ConclusionProposedPayload),
    ConclusionActivated(ConclusionActivatedPayload),
    ConclusionConfirmed(ConclusionConfirmedPayload),
    ConclusionMarkedStale(ConclusionMarkedStalePayload),
    ConclusionDisputed(ConclusionDisputedPayload),
    ConclusionSuperseded(ConclusionSupersededPayload),
    ConclusionRejected(ConclusionRejectedPayload),
    DecisionProposed(DecisionProposedPayload),
    DecisionApproved(DecisionApprovedPayload),
    DecisionSuperseded(DecisionSupersededPayload),
    DecisionRevoked(DecisionRevokedPayload),
    WorkspaceRegistered(WorkspaceRegisteredPayload),
    RepositoryJoinedWorkspace(RepositoryJoinedWorkspacePayload),
    ScopeGrantIssued(ScopeGrantIssuedPayload),
    ScopeGrantRevoked(ScopeGrantRevokedPayload),
    PrincipalRegistered(PrincipalRegisteredPayload),
    ReviewItemOpened(ReviewItemOpenedPayload),
    ReviewItemResolved(ReviewItemResolvedPayload),
    BriefingGenerated(BriefingGeneratedPayload),
    QueryTraceRecorded(QueryTraceRecordedPayload),
    ContentErasureRequested(ContentErasureRequestedPayload),
    ContentErased(ContentErasedPayload),
    ErasureTicketAccepted(ErasureTicketAcceptedPayload),
    ClaimConflictOpened(ClaimConflictOpenedPayload),
    ClaimConflictResolved(ClaimConflictResolvedPayload),
    RepositoryIdentityRegistered(RepositoryIdentityRegisteredPayload),
    RepositoryPathAliasAdded(RepositoryPathAliasAddedPayload),
    PolicyDecisionRecorded(PolicyDecisionRecordedPayload),
    /// Full original JSON object for unrecognized `type` tags.
    Unknown(serde_json::Value),
}

/// Known variants only — used for tagged ser/de without a unit `other` catch-all.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
enum KnownPayload {
    SystemInitialized(SystemInitializedPayload),
    RecoveryKitCreated(RecoveryKitCreatedPayload),
    ProjectRegistered(ProjectRegisteredPayload),
    ProjectAliasAdded(ProjectAliasAddedPayload),
    SessionStarted(SessionStartedPayload),
    UserPromptRecorded(UserPromptRecordedPayload),
    AssistantFinalRecorded(AssistantFinalRecordedPayload),
    SessionCompleted(SessionCompletedPayload),
    SessionFailed(SessionFailedPayload),
    MemoryPinned(MemoryPinnedPayload),
    MemoryForgotten(MemoryForgottenPayload),
    MemoryRestored(MemoryRestoredPayload),
    SessionSummaryCreated(SessionSummaryCreatedPayload),
    ConflictDetected(ConflictDetectedPayload),
    RecipePromoted(RecipePromotedPayload),
    MemorySynthesized(MemorySynthesizedPayload),
    FeedbackMetric(FeedbackMetricPayload),
    PredictionRecorded(PredictionRecordedPayload),
    VerifyOutcomeRecorded(VerifyOutcomeRecordedPayload),
    DecisionRecorded(DecisionRecordedPayload),
    IngestGateRejected(IngestGateRejectedPayload),
    SourceRegistered(SourceRegisteredPayload),
    SourceObserved(SourceObservedPayload),
    SourceVersionRecorded(SourceVersionRecordedPayload),
    SourceUnavailable(SourceUnavailablePayload),
    EvidenceRecorded(EvidenceRecordedPayload),
    EvidenceSuperseded(EvidenceSupersededPayload),
    ConclusionProposed(ConclusionProposedPayload),
    ConclusionActivated(ConclusionActivatedPayload),
    ConclusionConfirmed(ConclusionConfirmedPayload),
    ConclusionMarkedStale(ConclusionMarkedStalePayload),
    ConclusionDisputed(ConclusionDisputedPayload),
    ConclusionSuperseded(ConclusionSupersededPayload),
    ConclusionRejected(ConclusionRejectedPayload),
    DecisionProposed(DecisionProposedPayload),
    DecisionApproved(DecisionApprovedPayload),
    DecisionSuperseded(DecisionSupersededPayload),
    DecisionRevoked(DecisionRevokedPayload),
    WorkspaceRegistered(WorkspaceRegisteredPayload),
    RepositoryJoinedWorkspace(RepositoryJoinedWorkspacePayload),
    ScopeGrantIssued(ScopeGrantIssuedPayload),
    ScopeGrantRevoked(ScopeGrantRevokedPayload),
    PrincipalRegistered(PrincipalRegisteredPayload),
    ReviewItemOpened(ReviewItemOpenedPayload),
    ReviewItemResolved(ReviewItemResolvedPayload),
    BriefingGenerated(BriefingGeneratedPayload),
    QueryTraceRecorded(QueryTraceRecordedPayload),
    ContentErasureRequested(ContentErasureRequestedPayload),
    ContentErased(ContentErasedPayload),
    ErasureTicketAccepted(ErasureTicketAcceptedPayload),
    ClaimConflictOpened(ClaimConflictOpenedPayload),
    ClaimConflictResolved(ClaimConflictResolvedPayload),
    RepositoryIdentityRegistered(RepositoryIdentityRegisteredPayload),
    RepositoryPathAliasAdded(RepositoryPathAliasAddedPayload),
    PolicyDecisionRecorded(PolicyDecisionRecordedPayload),
}

fn is_known_payload_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "SystemInitialized"
            | "RecoveryKitCreated"
            | "ProjectRegistered"
            | "ProjectAliasAdded"
            | "SessionStarted"
            | "UserPromptRecorded"
            | "AssistantFinalRecorded"
            | "SessionCompleted"
            | "SessionFailed"
            | "MemoryPinned"
            | "MemoryForgotten"
            | "MemoryRestored"
            | "SessionSummaryCreated"
            | "ConflictDetected"
            | "RecipePromoted"
            | "MemorySynthesized"
            | "FeedbackMetric"
            | "PredictionRecorded"
            | "VerifyOutcomeRecorded"
            | "DecisionRecorded"
            | "IngestGateRejected"
            | "SourceRegistered"
            | "SourceObserved"
            | "SourceVersionRecorded"
            | "SourceUnavailable"
            | "EvidenceRecorded"
            | "EvidenceSuperseded"
            | "ConclusionProposed"
            | "ConclusionActivated"
            | "ConclusionConfirmed"
            | "ConclusionMarkedStale"
            | "ConclusionDisputed"
            | "ConclusionSuperseded"
            | "ConclusionRejected"
            | "DecisionProposed"
            | "DecisionApproved"
            | "DecisionSuperseded"
            | "DecisionRevoked"
            | "WorkspaceRegistered"
            | "RepositoryJoinedWorkspace"
            | "ScopeGrantIssued"
            | "ScopeGrantRevoked"
            | "PrincipalRegistered"
            | "ReviewItemOpened"
            | "ReviewItemResolved"
            | "BriefingGenerated"
            | "QueryTraceRecorded"
            | "ContentErasureRequested"
            | "ContentErased"
            | "ErasureTicketAccepted"
            | "ClaimConflictOpened"
            | "ClaimConflictResolved"
            | "RepositoryIdentityRegistered"
            | "RepositoryPathAliasAdded"
            | "PolicyDecisionRecorded"
    )
}

impl From<KnownPayload> for Payload {
    fn from(k: KnownPayload) -> Self {
        match k {
            KnownPayload::SystemInitialized(p) => Payload::SystemInitialized(p),
            KnownPayload::RecoveryKitCreated(p) => Payload::RecoveryKitCreated(p),
            KnownPayload::ProjectRegistered(p) => Payload::ProjectRegistered(p),
            KnownPayload::ProjectAliasAdded(p) => Payload::ProjectAliasAdded(p),
            KnownPayload::SessionStarted(p) => Payload::SessionStarted(p),
            KnownPayload::UserPromptRecorded(p) => Payload::UserPromptRecorded(p),
            KnownPayload::AssistantFinalRecorded(p) => Payload::AssistantFinalRecorded(p),
            KnownPayload::SessionCompleted(p) => Payload::SessionCompleted(p),
            KnownPayload::SessionFailed(p) => Payload::SessionFailed(p),
            KnownPayload::MemoryPinned(p) => Payload::MemoryPinned(p),
            KnownPayload::MemoryForgotten(p) => Payload::MemoryForgotten(p),
            KnownPayload::MemoryRestored(p) => Payload::MemoryRestored(p),
            KnownPayload::SessionSummaryCreated(p) => Payload::SessionSummaryCreated(p),
            KnownPayload::ConflictDetected(p) => Payload::ConflictDetected(p),
            KnownPayload::RecipePromoted(p) => Payload::RecipePromoted(p),
            KnownPayload::MemorySynthesized(p) => Payload::MemorySynthesized(p),
            KnownPayload::FeedbackMetric(p) => Payload::FeedbackMetric(p),
            KnownPayload::PredictionRecorded(p) => Payload::PredictionRecorded(p),
            KnownPayload::VerifyOutcomeRecorded(p) => Payload::VerifyOutcomeRecorded(p),
            KnownPayload::DecisionRecorded(p) => Payload::DecisionRecorded(p),
            KnownPayload::IngestGateRejected(p) => Payload::IngestGateRejected(p),
            KnownPayload::SourceRegistered(p) => Payload::SourceRegistered(p),
            KnownPayload::SourceObserved(p) => Payload::SourceObserved(p),
            KnownPayload::SourceVersionRecorded(p) => Payload::SourceVersionRecorded(p),
            KnownPayload::SourceUnavailable(p) => Payload::SourceUnavailable(p),
            KnownPayload::EvidenceRecorded(p) => Payload::EvidenceRecorded(p),
            KnownPayload::EvidenceSuperseded(p) => Payload::EvidenceSuperseded(p),
            KnownPayload::ConclusionProposed(p) => Payload::ConclusionProposed(p),
            KnownPayload::ConclusionActivated(p) => Payload::ConclusionActivated(p),
            KnownPayload::ConclusionConfirmed(p) => Payload::ConclusionConfirmed(p),
            KnownPayload::ConclusionMarkedStale(p) => Payload::ConclusionMarkedStale(p),
            KnownPayload::ConclusionDisputed(p) => Payload::ConclusionDisputed(p),
            KnownPayload::ConclusionSuperseded(p) => Payload::ConclusionSuperseded(p),
            KnownPayload::ConclusionRejected(p) => Payload::ConclusionRejected(p),
            KnownPayload::DecisionProposed(p) => Payload::DecisionProposed(p),
            KnownPayload::DecisionApproved(p) => Payload::DecisionApproved(p),
            KnownPayload::DecisionSuperseded(p) => Payload::DecisionSuperseded(p),
            KnownPayload::DecisionRevoked(p) => Payload::DecisionRevoked(p),
            KnownPayload::WorkspaceRegistered(p) => Payload::WorkspaceRegistered(p),
            KnownPayload::RepositoryJoinedWorkspace(p) => Payload::RepositoryJoinedWorkspace(p),
            KnownPayload::ScopeGrantIssued(p) => Payload::ScopeGrantIssued(p),
            KnownPayload::ScopeGrantRevoked(p) => Payload::ScopeGrantRevoked(p),
            KnownPayload::PrincipalRegistered(p) => Payload::PrincipalRegistered(p),
            KnownPayload::ReviewItemOpened(p) => Payload::ReviewItemOpened(p),
            KnownPayload::ReviewItemResolved(p) => Payload::ReviewItemResolved(p),
            KnownPayload::BriefingGenerated(p) => Payload::BriefingGenerated(p),
            KnownPayload::QueryTraceRecorded(p) => Payload::QueryTraceRecorded(p),
            KnownPayload::ContentErasureRequested(p) => Payload::ContentErasureRequested(p),
            KnownPayload::ContentErased(p) => Payload::ContentErased(p),
            KnownPayload::ErasureTicketAccepted(p) => Payload::ErasureTicketAccepted(p),
            KnownPayload::ClaimConflictOpened(p) => Payload::ClaimConflictOpened(p),
            KnownPayload::ClaimConflictResolved(p) => Payload::ClaimConflictResolved(p),
            KnownPayload::RepositoryIdentityRegistered(p) => {
                Payload::RepositoryIdentityRegistered(p)
            }
            KnownPayload::RepositoryPathAliasAdded(p) => Payload::RepositoryPathAliasAdded(p),
            KnownPayload::PolicyDecisionRecorded(p) => Payload::PolicyDecisionRecorded(p),
        }
    }
}

impl Payload {
    fn to_known(&self) -> Option<KnownPayload> {
        Some(match self {
            Payload::SystemInitialized(p) => KnownPayload::SystemInitialized(p.clone()),
            Payload::RecoveryKitCreated(p) => KnownPayload::RecoveryKitCreated(p.clone()),
            Payload::ProjectRegistered(p) => KnownPayload::ProjectRegistered(p.clone()),
            Payload::ProjectAliasAdded(p) => KnownPayload::ProjectAliasAdded(p.clone()),
            Payload::SessionStarted(p) => KnownPayload::SessionStarted(p.clone()),
            Payload::UserPromptRecorded(p) => KnownPayload::UserPromptRecorded(p.clone()),
            Payload::AssistantFinalRecorded(p) => KnownPayload::AssistantFinalRecorded(p.clone()),
            Payload::SessionCompleted(p) => KnownPayload::SessionCompleted(p.clone()),
            Payload::SessionFailed(p) => KnownPayload::SessionFailed(p.clone()),
            Payload::MemoryPinned(p) => KnownPayload::MemoryPinned(p.clone()),
            Payload::MemoryForgotten(p) => KnownPayload::MemoryForgotten(p.clone()),
            Payload::MemoryRestored(p) => KnownPayload::MemoryRestored(p.clone()),
            Payload::SessionSummaryCreated(p) => KnownPayload::SessionSummaryCreated(p.clone()),
            Payload::ConflictDetected(p) => KnownPayload::ConflictDetected(p.clone()),
            Payload::RecipePromoted(p) => KnownPayload::RecipePromoted(p.clone()),
            Payload::MemorySynthesized(p) => KnownPayload::MemorySynthesized(p.clone()),
            Payload::FeedbackMetric(p) => KnownPayload::FeedbackMetric(p.clone()),
            Payload::PredictionRecorded(p) => KnownPayload::PredictionRecorded(p.clone()),
            Payload::VerifyOutcomeRecorded(p) => KnownPayload::VerifyOutcomeRecorded(p.clone()),
            Payload::DecisionRecorded(p) => KnownPayload::DecisionRecorded(p.clone()),
            Payload::IngestGateRejected(p) => KnownPayload::IngestGateRejected(p.clone()),
            Payload::SourceRegistered(p) => KnownPayload::SourceRegistered(p.clone()),
            Payload::SourceObserved(p) => KnownPayload::SourceObserved(p.clone()),
            Payload::SourceVersionRecorded(p) => KnownPayload::SourceVersionRecorded(p.clone()),
            Payload::SourceUnavailable(p) => KnownPayload::SourceUnavailable(p.clone()),
            Payload::EvidenceRecorded(p) => KnownPayload::EvidenceRecorded(p.clone()),
            Payload::EvidenceSuperseded(p) => KnownPayload::EvidenceSuperseded(p.clone()),
            Payload::ConclusionProposed(p) => KnownPayload::ConclusionProposed(p.clone()),
            Payload::ConclusionActivated(p) => KnownPayload::ConclusionActivated(p.clone()),
            Payload::ConclusionConfirmed(p) => KnownPayload::ConclusionConfirmed(p.clone()),
            Payload::ConclusionMarkedStale(p) => KnownPayload::ConclusionMarkedStale(p.clone()),
            Payload::ConclusionDisputed(p) => KnownPayload::ConclusionDisputed(p.clone()),
            Payload::ConclusionSuperseded(p) => KnownPayload::ConclusionSuperseded(p.clone()),
            Payload::ConclusionRejected(p) => KnownPayload::ConclusionRejected(p.clone()),
            Payload::DecisionProposed(p) => KnownPayload::DecisionProposed(p.clone()),
            Payload::DecisionApproved(p) => KnownPayload::DecisionApproved(p.clone()),
            Payload::DecisionSuperseded(p) => KnownPayload::DecisionSuperseded(p.clone()),
            Payload::DecisionRevoked(p) => KnownPayload::DecisionRevoked(p.clone()),
            Payload::WorkspaceRegistered(p) => KnownPayload::WorkspaceRegistered(p.clone()),
            Payload::RepositoryJoinedWorkspace(p) => {
                KnownPayload::RepositoryJoinedWorkspace(p.clone())
            }
            Payload::ScopeGrantIssued(p) => KnownPayload::ScopeGrantIssued(p.clone()),
            Payload::ScopeGrantRevoked(p) => KnownPayload::ScopeGrantRevoked(p.clone()),
            Payload::PrincipalRegistered(p) => KnownPayload::PrincipalRegistered(p.clone()),
            Payload::ReviewItemOpened(p) => KnownPayload::ReviewItemOpened(p.clone()),
            Payload::ReviewItemResolved(p) => KnownPayload::ReviewItemResolved(p.clone()),
            Payload::BriefingGenerated(p) => KnownPayload::BriefingGenerated(p.clone()),
            Payload::QueryTraceRecorded(p) => KnownPayload::QueryTraceRecorded(p.clone()),
            Payload::ContentErasureRequested(p) => KnownPayload::ContentErasureRequested(p.clone()),
            Payload::ContentErased(p) => KnownPayload::ContentErased(p.clone()),
            Payload::ErasureTicketAccepted(p) => KnownPayload::ErasureTicketAccepted(p.clone()),
            Payload::ClaimConflictOpened(p) => KnownPayload::ClaimConflictOpened(p.clone()),
            Payload::ClaimConflictResolved(p) => KnownPayload::ClaimConflictResolved(p.clone()),
            Payload::RepositoryIdentityRegistered(p) => {
                KnownPayload::RepositoryIdentityRegistered(p.clone())
            }
            Payload::RepositoryPathAliasAdded(p) => {
                KnownPayload::RepositoryPathAliasAdded(p.clone())
            }
            Payload::PolicyDecisionRecorded(p) => KnownPayload::PolicyDecisionRecorded(p.clone()),
            Payload::Unknown(_) => return None,
        })
    }
}

impl Serialize for Payload {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Payload::Unknown(value) => value.serialize(serializer),
            known => {
                let mapped = known.to_known().ok_or_else(|| {
                    serde::ser::Error::custom(
                        "internal: non-Unknown payload missing KnownPayload mapping",
                    )
                })?;
                mapped.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_str = value.get("type").and_then(|t| t.as_str());
        match type_str {
            Some(t) if is_known_payload_type(t) => {
                let known: KnownPayload =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Payload::from(known))
            }
            _ => Ok(Payload::Unknown(value)),
        }
    }
}
