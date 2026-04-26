use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub job_id: String,
    pub status: String,
    pub progress_percent: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}
