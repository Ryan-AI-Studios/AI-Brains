use crate::capability::{AdapterCapability, CapabilityLevel};

pub fn opencode_capability() -> AdapterCapability {
    AdapterCapability {
        name: "opencode".to_string(),
        level: CapabilityLevel::Partial,
        supports_hooks: false,
        supports_wrapper_mode: true,
        notes: "Wrapper-mode capture with degraded hook support. Intended PrincipalKind::Connector binding when full capture is wired; principal_binding deferred until registry wiring.".to_string(),
        governed_reads: Vec::new(),
        governed_writes: Vec::new(),
        principal_binding: None,
    }
}
