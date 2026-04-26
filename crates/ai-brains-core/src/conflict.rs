use crate::ids::{ConflictId, MemoryId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: ConflictId,
    pub memory_id: MemoryId,
    pub description: String,
}
