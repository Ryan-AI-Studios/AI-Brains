use crate::ids::HarnessId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Harness {
    pub id: HarnessId,
    pub name: String,
}
