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

pub use command_handler::{
    CaptureContext, CaptureOutcome, CaptureService, CaptureSink, MemorySink, SessionStartCommand,
    SessionStopCommand, SessionStopStatus,
};
pub use errors::{CaptureError, Result};
pub use malformed::parse_ingest_request;
