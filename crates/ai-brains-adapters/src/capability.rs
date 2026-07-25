use ai_brains_core::ids::PrincipalId;
use ai_brains_core::scope::GrantCapability;
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
    /// Governed read capabilities this adapter may exercise (deny-by-default elsewhere).
    #[serde(default)]
    pub governed_reads: Vec<GrantCapability>,
    /// Governed write capabilities this adapter may exercise (e.g. ProposeConclusion).
    #[serde(default)]
    pub governed_writes: Vec<GrantCapability>,
    /// Optional bound principal; None until Connector principal registry wiring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal_binding: Option<PrincipalId>,
}

/// Standard governed reads for full-level harness adapters that capture.
///
/// Full harnesses are intended to bind as [`ai_brains_core::principal::PrincipalKind::Agent`]
/// (not Connector). Connector principals remain observe-only (`ReadEvidence`); Agent may
/// hold Read*/Propose* with grants. Do not strip `ProposeConclusion` from Full
/// `governed_writes` for that reason.
pub(crate) fn full_harness_governed_reads() -> Vec<GrantCapability> {
    vec![
        GrantCapability::ReadEvidence,
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)] // project test naming: feature__condition__expected
    fn adapter_capability__legacy_json_without_governed_fields__deserializes() {
        let legacy = r#"{
            "name": "legacy-adapter",
            "level": "Full",
            "supports_hooks": true,
            "supports_wrapper_mode": false,
            "notes": "pre-T151 shape"
        }"#;
        let capability: AdapterCapability = match serde_json::from_str(legacy) {
            Ok(c) => c,
            Err(e) => panic!("legacy JSON must deserialize with serde defaults: {e}"),
        };
        assert_eq!(capability.name, "legacy-adapter");
        assert_eq!(capability.level, CapabilityLevel::Full);
        assert!(capability.governed_reads.is_empty());
        assert!(capability.governed_writes.is_empty());
        assert!(capability.principal_binding.is_none());
    }
}
