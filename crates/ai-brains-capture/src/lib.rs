mod action_digest;
mod assistant_final;
mod command_handler;
mod errors;
mod git_capture;
mod malformed;
mod metadata;
mod privacy;
mod session_start;
mod session_stop;
mod user_prompt;
pub mod verification_evidence;
pub mod verification_gate;

pub use command_handler::{
    CaptureContext, CaptureOutcome, CaptureService, CaptureSink, MemorySink, SessionStartCommand,
    SessionStopCommand, SessionStopStatus,
};
pub use errors::{CaptureError, Result, VerificationGateRejection};
pub use malformed::parse_ingest_request;
pub use verification_evidence::{
    VerificationEvidence, VerificationEvidenceStatus, build_verification_evidence_events,
    verification_gate_source_id,
};
pub use verification_gate::{
    GateDecision, LedgerfulVerificationBackend, VerificationBackend, VerificationGate,
    VerifyResponse,
};
