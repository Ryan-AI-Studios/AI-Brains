use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityLevel {
    Full,
    Partial,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCapability {
    pub name: String,
    pub level: CapabilityLevel,
    pub supports_hooks: bool,
    pub supports_wrapper_mode: bool,
    pub notes: String,
}
