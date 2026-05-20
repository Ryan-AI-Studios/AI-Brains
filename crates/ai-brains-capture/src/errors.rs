use thiserror::Error;

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Structured payload carried by [`CaptureError::VerificationGateRejected`].
#[derive(Debug, Clone)]
pub struct VerificationGateRejection {
    pub session_id: ai_brains_core::ids::SessionId,
    pub failure_probability: f64,
    pub drift_detected: bool,
    pub risk_level: String,
    pub explanation: String,
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("unsupported role: {0}")]
    UnsupportedRole(String),
    #[error("user prompt content cannot be empty")]
    EmptyPrompt,
    #[error("assistant final content cannot be empty unless status-only")]
    EmptyFinal,
    #[error("session stop reason is required for failed status")]
    MissingFailureReason,
    #[error(
        "verification gate rejected ingest: {explanation} (failure_prob={failure_probability:.2}, drift={drift_detected}, risk={risk_level})",
        failure_probability = .0.failure_probability,
        drift_detected = .0.drift_detected,
        risk_level = .0.risk_level,
        explanation = .0.explanation
    )]
    VerificationGateRejected(VerificationGateRejection),
    #[error("event build failed: {0}")]
    Event(#[from] ai_brains_events::EventError),
    #[error("json parse failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("git metadata failed: {0}")]
    Git(#[from] ai_brains_git::GitError),
    #[error("validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", "))]
    ValidationErrors(Vec<ValidationError>),
}
