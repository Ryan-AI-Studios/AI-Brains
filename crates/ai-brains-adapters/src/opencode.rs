use crate::capability::{AdapterCapability, CapabilityLevel};

pub fn opencode_capability() -> AdapterCapability {
    AdapterCapability {
        name: "opencode".to_string(),
        level: CapabilityLevel::Partial,
        supports_hooks: false,
        supports_wrapper_mode: true,
        notes: "Wrapper-mode capture with degraded hook support.".to_string(),
    }
}
