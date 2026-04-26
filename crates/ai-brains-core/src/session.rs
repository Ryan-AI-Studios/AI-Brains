use crate::ids::{DeviceId, HarnessId, ProjectId, SessionId, UserId};
use crate::privacy::Privacy;
use crate::status::SessionStatus;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub harness_id: HarnessId,
    pub status: SessionStatus,
    pub privacy: Privacy,
    pub created_at: OffsetDateTime,
}
