use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Event kind tag stored on the envelope.
///
/// Known kinds serialize as PascalCase strings. Unknown future tags are preserved
/// in [`EventKind::Unknown`] so re-serialization does not rewrite them as `"Unknown"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    // System
    SystemInitialized,
    RecoveryKitCreated,
    /// Vault DataKey rotation audit (T189 / ADR-0020).
    DataKeyRotated,

    // Project
    ProjectRegistered,
    ProjectAliasAdded,

    // Session
    SessionStarted,
    UserPromptRecorded,
    AssistantFinalRecorded,
    SessionCompleted,
    SessionFailed,
    SessionSummaryCreated,

    // Memory
    MemoryPinned,
    MemoryForgotten,
    MemoryRestored,
    PrivacyEscalated,

    // Background
    NightlyJobStarted,
    ConflictDetected,
    RecipePromoted,
    MemorySynthesized,
    FeedbackMetric,
    PredictionRecorded,
    VerifyOutcomeRecorded,
    DecisionRecorded,

    // Verification gating (T43)
    IngestGateRejected,

    // Governed memory (T148) — additive facts at schema v1
    SourceRegistered,
    SourceObserved,
    SourceVersionRecorded,
    SourceUnavailable,
    EvidenceRecorded,
    EvidenceSuperseded,
    ConclusionProposed,
    ConclusionActivated,
    ConclusionConfirmed,
    ConclusionMarkedStale,
    ConclusionDisputed,
    ConclusionSuperseded,
    ConclusionRejected,
    DecisionProposed,
    DecisionApproved,
    DecisionSuperseded,
    DecisionRevoked,
    WorkspaceRegistered,
    RepositoryJoinedWorkspace,
    ScopeGrantIssued,
    ScopeGrantRevoked,
    PrincipalRegistered,
    ReviewItemOpened,
    ReviewItemResolved,
    BriefingGenerated,
    QueryTraceRecorded,
    ContentErasureRequested,
    ContentErased,
    /// Durable erasure accept ticket (T159); not CE wipe.
    ErasureTicketAccepted,
    /// Class-based retention apply audit (T166); not CE by itself.
    RetentionApplied,
    /// Legacy → governed classification import apply audit (T167).
    LegacyImportApplied,

    // Claim conflicts (T150) — distinct from legacy ConflictDetected (memory)
    ClaimConflictOpened,
    ClaimConflictResolved,

    // Repository identity / path aliases (T151 / T254) — rehydrate on rebuild_projections
    RepositoryIdentityRegistered,
    RepositoryPathAliasAdded,
    RepositoryPathAliasRemoved,

    // Policy audit (T151) — rehydrate policy_decision_log on rebuild_projections
    PolicyDecisionRecorded,

    // Multi-device membership controls (T176 / ADR-0018) — canonical event-log SOV
    DeviceEnrolled,
    DeviceRevoked,

    /// Forward-compatible catch-all; holds the original tag string.
    Unknown(String),
}

impl EventKind {
    fn as_str(&self) -> &str {
        match self {
            EventKind::SystemInitialized => "SystemInitialized",
            EventKind::RecoveryKitCreated => "RecoveryKitCreated",
            EventKind::DataKeyRotated => "DataKeyRotated",
            EventKind::ProjectRegistered => "ProjectRegistered",
            EventKind::ProjectAliasAdded => "ProjectAliasAdded",
            EventKind::SessionStarted => "SessionStarted",
            EventKind::UserPromptRecorded => "UserPromptRecorded",
            EventKind::AssistantFinalRecorded => "AssistantFinalRecorded",
            EventKind::SessionCompleted => "SessionCompleted",
            EventKind::SessionFailed => "SessionFailed",
            EventKind::SessionSummaryCreated => "SessionSummaryCreated",
            EventKind::MemoryPinned => "MemoryPinned",
            EventKind::MemoryForgotten => "MemoryForgotten",
            EventKind::MemoryRestored => "MemoryRestored",
            EventKind::PrivacyEscalated => "PrivacyEscalated",
            EventKind::NightlyJobStarted => "NightlyJobStarted",
            EventKind::ConflictDetected => "ConflictDetected",
            EventKind::RecipePromoted => "RecipePromoted",
            EventKind::MemorySynthesized => "MemorySynthesized",
            EventKind::FeedbackMetric => "FeedbackMetric",
            EventKind::PredictionRecorded => "PredictionRecorded",
            EventKind::VerifyOutcomeRecorded => "VerifyOutcomeRecorded",
            EventKind::DecisionRecorded => "DecisionRecorded",
            EventKind::IngestGateRejected => "IngestGateRejected",
            EventKind::SourceRegistered => "SourceRegistered",
            EventKind::SourceObserved => "SourceObserved",
            EventKind::SourceVersionRecorded => "SourceVersionRecorded",
            EventKind::SourceUnavailable => "SourceUnavailable",
            EventKind::EvidenceRecorded => "EvidenceRecorded",
            EventKind::EvidenceSuperseded => "EvidenceSuperseded",
            EventKind::ConclusionProposed => "ConclusionProposed",
            EventKind::ConclusionActivated => "ConclusionActivated",
            EventKind::ConclusionConfirmed => "ConclusionConfirmed",
            EventKind::ConclusionMarkedStale => "ConclusionMarkedStale",
            EventKind::ConclusionDisputed => "ConclusionDisputed",
            EventKind::ConclusionSuperseded => "ConclusionSuperseded",
            EventKind::ConclusionRejected => "ConclusionRejected",
            EventKind::DecisionProposed => "DecisionProposed",
            EventKind::DecisionApproved => "DecisionApproved",
            EventKind::DecisionSuperseded => "DecisionSuperseded",
            EventKind::DecisionRevoked => "DecisionRevoked",
            EventKind::WorkspaceRegistered => "WorkspaceRegistered",
            EventKind::RepositoryJoinedWorkspace => "RepositoryJoinedWorkspace",
            EventKind::ScopeGrantIssued => "ScopeGrantIssued",
            EventKind::ScopeGrantRevoked => "ScopeGrantRevoked",
            EventKind::PrincipalRegistered => "PrincipalRegistered",
            EventKind::ReviewItemOpened => "ReviewItemOpened",
            EventKind::ReviewItemResolved => "ReviewItemResolved",
            EventKind::BriefingGenerated => "BriefingGenerated",
            EventKind::QueryTraceRecorded => "QueryTraceRecorded",
            EventKind::ContentErasureRequested => "ContentErasureRequested",
            EventKind::ContentErased => "ContentErased",
            EventKind::ErasureTicketAccepted => "ErasureTicketAccepted",
            EventKind::RetentionApplied => "RetentionApplied",
            EventKind::LegacyImportApplied => "LegacyImportApplied",
            EventKind::ClaimConflictOpened => "ClaimConflictOpened",
            EventKind::ClaimConflictResolved => "ClaimConflictResolved",
            EventKind::RepositoryIdentityRegistered => "RepositoryIdentityRegistered",
            EventKind::RepositoryPathAliasAdded => "RepositoryPathAliasAdded",
            EventKind::RepositoryPathAliasRemoved => "RepositoryPathAliasRemoved",
            EventKind::PolicyDecisionRecorded => "PolicyDecisionRecorded",
            EventKind::DeviceEnrolled => "DeviceEnrolled",
            EventKind::DeviceRevoked => "DeviceRevoked",
            EventKind::Unknown(s) => s.as_str(),
        }
    }

    fn from_str_tag(s: &str) -> Self {
        match s {
            "SystemInitialized" => EventKind::SystemInitialized,
            "RecoveryKitCreated" => EventKind::RecoveryKitCreated,
            "DataKeyRotated" => EventKind::DataKeyRotated,
            "ProjectRegistered" => EventKind::ProjectRegistered,
            "ProjectAliasAdded" => EventKind::ProjectAliasAdded,
            "SessionStarted" => EventKind::SessionStarted,
            "UserPromptRecorded" => EventKind::UserPromptRecorded,
            "AssistantFinalRecorded" => EventKind::AssistantFinalRecorded,
            "SessionCompleted" => EventKind::SessionCompleted,
            "SessionFailed" => EventKind::SessionFailed,
            "SessionSummaryCreated" => EventKind::SessionSummaryCreated,
            "MemoryPinned" => EventKind::MemoryPinned,
            "MemoryForgotten" => EventKind::MemoryForgotten,
            "MemoryRestored" => EventKind::MemoryRestored,
            "PrivacyEscalated" => EventKind::PrivacyEscalated,
            "NightlyJobStarted" => EventKind::NightlyJobStarted,
            "ConflictDetected" => EventKind::ConflictDetected,
            "RecipePromoted" => EventKind::RecipePromoted,
            "MemorySynthesized" => EventKind::MemorySynthesized,
            "FeedbackMetric" => EventKind::FeedbackMetric,
            "PredictionRecorded" => EventKind::PredictionRecorded,
            "VerifyOutcomeRecorded" => EventKind::VerifyOutcomeRecorded,
            "DecisionRecorded" => EventKind::DecisionRecorded,
            "IngestGateRejected" => EventKind::IngestGateRejected,
            "SourceRegistered" => EventKind::SourceRegistered,
            "SourceObserved" => EventKind::SourceObserved,
            "SourceVersionRecorded" => EventKind::SourceVersionRecorded,
            "SourceUnavailable" => EventKind::SourceUnavailable,
            "EvidenceRecorded" => EventKind::EvidenceRecorded,
            "EvidenceSuperseded" => EventKind::EvidenceSuperseded,
            "ConclusionProposed" => EventKind::ConclusionProposed,
            "ConclusionActivated" => EventKind::ConclusionActivated,
            "ConclusionConfirmed" => EventKind::ConclusionConfirmed,
            "ConclusionMarkedStale" => EventKind::ConclusionMarkedStale,
            "ConclusionDisputed" => EventKind::ConclusionDisputed,
            "ConclusionSuperseded" => EventKind::ConclusionSuperseded,
            "ConclusionRejected" => EventKind::ConclusionRejected,
            "DecisionProposed" => EventKind::DecisionProposed,
            "DecisionApproved" => EventKind::DecisionApproved,
            "DecisionSuperseded" => EventKind::DecisionSuperseded,
            "DecisionRevoked" => EventKind::DecisionRevoked,
            "WorkspaceRegistered" => EventKind::WorkspaceRegistered,
            "RepositoryJoinedWorkspace" => EventKind::RepositoryJoinedWorkspace,
            "ScopeGrantIssued" => EventKind::ScopeGrantIssued,
            "ScopeGrantRevoked" => EventKind::ScopeGrantRevoked,
            "PrincipalRegistered" => EventKind::PrincipalRegistered,
            "ReviewItemOpened" => EventKind::ReviewItemOpened,
            "ReviewItemResolved" => EventKind::ReviewItemResolved,
            "BriefingGenerated" => EventKind::BriefingGenerated,
            "QueryTraceRecorded" => EventKind::QueryTraceRecorded,
            "ContentErasureRequested" => EventKind::ContentErasureRequested,
            "ContentErased" => EventKind::ContentErased,
            "ErasureTicketAccepted" => EventKind::ErasureTicketAccepted,
            "RetentionApplied" => EventKind::RetentionApplied,
            "LegacyImportApplied" => EventKind::LegacyImportApplied,
            "ClaimConflictOpened" => EventKind::ClaimConflictOpened,
            "ClaimConflictResolved" => EventKind::ClaimConflictResolved,
            "RepositoryIdentityRegistered" => EventKind::RepositoryIdentityRegistered,
            "RepositoryPathAliasAdded" => EventKind::RepositoryPathAliasAdded,
            "RepositoryPathAliasRemoved" => EventKind::RepositoryPathAliasRemoved,
            "PolicyDecisionRecorded" => EventKind::PolicyDecisionRecorded,
            "DeviceEnrolled" => EventKind::DeviceEnrolled,
            "DeviceRevoked" => EventKind::DeviceRevoked,
            other => EventKind::Unknown(other.to_string()),
        }
    }
}

impl Serialize for EventKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(EventKind::from_str_tag(&s))
    }
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Single source of truth: envelope `event_type` is always derived from payload.
///
/// Exhaustive on known variants. [`Payload::Unknown`] extracts the JSON `type`
/// tag when present so re-serialization preserves the future kind string.
impl From<&crate::payload::Payload> for EventKind {
    fn from(payload: &crate::payload::Payload) -> Self {
        use crate::payload::Payload;
        match payload {
            Payload::SystemInitialized(_) => EventKind::SystemInitialized,
            Payload::RecoveryKitCreated(_) => EventKind::RecoveryKitCreated,
            Payload::DataKeyRotated(_) => EventKind::DataKeyRotated,
            Payload::ProjectRegistered(_) => EventKind::ProjectRegistered,
            Payload::ProjectAliasAdded(_) => EventKind::ProjectAliasAdded,
            Payload::SessionStarted(_) => EventKind::SessionStarted,
            Payload::UserPromptRecorded(_) => EventKind::UserPromptRecorded,
            Payload::AssistantFinalRecorded(_) => EventKind::AssistantFinalRecorded,
            Payload::SessionCompleted(_) => EventKind::SessionCompleted,
            Payload::SessionFailed(_) => EventKind::SessionFailed,
            Payload::MemoryPinned(_) => EventKind::MemoryPinned,
            Payload::MemoryForgotten(_) => EventKind::MemoryForgotten,
            Payload::MemoryRestored(_) => EventKind::MemoryRestored,
            Payload::SessionSummaryCreated(_) => EventKind::SessionSummaryCreated,
            Payload::ConflictDetected(_) => EventKind::ConflictDetected,
            Payload::RecipePromoted(_) => EventKind::RecipePromoted,
            Payload::MemorySynthesized(_) => EventKind::MemorySynthesized,
            Payload::FeedbackMetric(_) => EventKind::FeedbackMetric,
            Payload::PredictionRecorded(_) => EventKind::PredictionRecorded,
            Payload::VerifyOutcomeRecorded(_) => EventKind::VerifyOutcomeRecorded,
            Payload::DecisionRecorded(_) => EventKind::DecisionRecorded,
            Payload::IngestGateRejected(_) => EventKind::IngestGateRejected,
            Payload::SourceRegistered(_) => EventKind::SourceRegistered,
            Payload::SourceObserved(_) => EventKind::SourceObserved,
            Payload::SourceVersionRecorded(_) => EventKind::SourceVersionRecorded,
            Payload::SourceUnavailable(_) => EventKind::SourceUnavailable,
            Payload::EvidenceRecorded(_) => EventKind::EvidenceRecorded,
            Payload::EvidenceSuperseded(_) => EventKind::EvidenceSuperseded,
            Payload::ConclusionProposed(_) => EventKind::ConclusionProposed,
            Payload::ConclusionActivated(_) => EventKind::ConclusionActivated,
            Payload::ConclusionConfirmed(_) => EventKind::ConclusionConfirmed,
            Payload::ConclusionMarkedStale(_) => EventKind::ConclusionMarkedStale,
            Payload::ConclusionDisputed(_) => EventKind::ConclusionDisputed,
            Payload::ConclusionSuperseded(_) => EventKind::ConclusionSuperseded,
            Payload::ConclusionRejected(_) => EventKind::ConclusionRejected,
            Payload::DecisionProposed(_) => EventKind::DecisionProposed,
            Payload::DecisionApproved(_) => EventKind::DecisionApproved,
            Payload::DecisionSuperseded(_) => EventKind::DecisionSuperseded,
            Payload::DecisionRevoked(_) => EventKind::DecisionRevoked,
            Payload::WorkspaceRegistered(_) => EventKind::WorkspaceRegistered,
            Payload::RepositoryJoinedWorkspace(_) => EventKind::RepositoryJoinedWorkspace,
            Payload::ScopeGrantIssued(_) => EventKind::ScopeGrantIssued,
            Payload::ScopeGrantRevoked(_) => EventKind::ScopeGrantRevoked,
            Payload::PrincipalRegistered(_) => EventKind::PrincipalRegistered,
            Payload::ReviewItemOpened(_) => EventKind::ReviewItemOpened,
            Payload::ReviewItemResolved(_) => EventKind::ReviewItemResolved,
            Payload::BriefingGenerated(_) => EventKind::BriefingGenerated,
            Payload::QueryTraceRecorded(_) => EventKind::QueryTraceRecorded,
            Payload::ContentErasureRequested(_) => EventKind::ContentErasureRequested,
            Payload::ContentErased(_) => EventKind::ContentErased,
            Payload::ErasureTicketAccepted(_) => EventKind::ErasureTicketAccepted,
            Payload::RetentionApplied(_) => EventKind::RetentionApplied,
            Payload::LegacyImportApplied(_) => EventKind::LegacyImportApplied,
            Payload::ClaimConflictOpened(_) => EventKind::ClaimConflictOpened,
            Payload::ClaimConflictResolved(_) => EventKind::ClaimConflictResolved,
            Payload::RepositoryIdentityRegistered(_) => EventKind::RepositoryIdentityRegistered,
            Payload::RepositoryPathAliasAdded(_) => EventKind::RepositoryPathAliasAdded,
            Payload::RepositoryPathAliasRemoved(_) => EventKind::RepositoryPathAliasRemoved,
            Payload::PolicyDecisionRecorded(_) => EventKind::PolicyDecisionRecorded,
            Payload::DeviceEnrolled(_) => EventKind::DeviceEnrolled,
            Payload::DeviceRevoked(_) => EventKind::DeviceRevoked,
            Payload::Unknown(value) => {
                let tag = value
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown");
                EventKind::from_str_tag(tag)
            }
        }
    }
}
