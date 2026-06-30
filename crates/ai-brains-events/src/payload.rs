use ai_brains_core::ids::{ConflictId, MemoryId, ProjectId, RecipeId, SessionId, TransactionId};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
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

    /// Used for unknown future events to prevent deserialization failure
    #[serde(other)]
    Unknown,
}
