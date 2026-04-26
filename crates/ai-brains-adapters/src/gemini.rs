use crate::capability::{AdapterCapability, CapabilityLevel};

pub fn gemini_capability() -> AdapterCapability {
    AdapterCapability {
        name: "gemini".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Supports hook integration and degraded parsing fallback.".to_string(),
    }
}
