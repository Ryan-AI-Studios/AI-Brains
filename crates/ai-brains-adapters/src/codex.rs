use crate::capability::{AdapterCapability, CapabilityLevel, full_harness_governed_reads};
use ai_brains_core::scope::GrantCapability;

pub fn codex_capability() -> AdapterCapability {
    AdapterCapability {
        name: "codex".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Supports wrapper and structured ingest integration. Full harnesses bind as PrincipalKind::Agent (not Connector) so ProposeConclusion is in-matrix; principal_binding deferred until registry wiring. Connector observe-only remains ReadEvidence.".to_string(),
        governed_reads: full_harness_governed_reads(),
        governed_writes: vec![GrantCapability::ProposeConclusion],
        principal_binding: None,
    }
}
