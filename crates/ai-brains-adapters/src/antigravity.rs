use crate::capability::{AdapterCapability, CapabilityLevel};

pub fn antigravity_capability() -> AdapterCapability {
    AdapterCapability {
        name: "antigravity".to_string(),
        level: CapabilityLevel::Manual,
        supports_hooks: false,
        supports_wrapper_mode: false,
        notes: "Manual import mode only; degraded capture path.".to_string(),
    }
}

pub fn manual_import_instructions() -> String {
    "Use manual import mode: export the final assistant/user payload as JSON and pass it through the local ingest command.".to_string()
}
