use crate::ids::{MemoryId, SessionId};
use crate::privacy::Privacy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    pub session_id: SessionId,
    pub content: String,
    pub privacy: Privacy,
}
