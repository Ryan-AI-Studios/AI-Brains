use ai_brains_core::ids::{DeviceId, HarnessId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum Actor {
    User(UserId),
    Device(DeviceId),
    Harness(HarnessId),
    System,
}
