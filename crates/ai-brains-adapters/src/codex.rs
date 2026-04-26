use crate::capability::{AdapterCapability, CapabilityLevel};

pub fn codex_capability() -> AdapterCapability {
    AdapterCapability {
        name: "codex".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Supports wrapper and structured ingest integration.".to_string(),
    }
}
