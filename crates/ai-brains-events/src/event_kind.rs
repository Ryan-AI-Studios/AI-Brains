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

    /// Forward-compatible catch-all; holds the original tag string.
    Unknown(String),
}

impl EventKind {
    fn as_str(&self) -> &str {
        match self {
            EventKind::SystemInitialized => "SystemInitialized",
            EventKind::RecoveryKitCreated => "RecoveryKitCreated",
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
            EventKind::Unknown(s) => s.as_str(),
        }
    }

    fn from_str_tag(s: &str) -> Self {
        match s {
            "SystemInitialized" => EventKind::SystemInitialized,
            "RecoveryKitCreated" => EventKind::RecoveryKitCreated,
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
