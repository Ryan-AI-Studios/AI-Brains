use crate::errors::{CaptureError, Result};
use ai_brains_contracts::ingest::IngestRequest;

pub fn parse_ingest_request(json: &str) -> Result<IngestRequest> {
    serde_json::from_str(json).map_err(CaptureError::from)
}
