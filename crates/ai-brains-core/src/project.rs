use crate::ids::ProjectId;
use crate::privacy::Privacy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub privacy: Privacy,
}
