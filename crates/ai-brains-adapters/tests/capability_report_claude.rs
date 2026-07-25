use ai_brains_adapters::{AdapterKind, CapabilityLevel, adapter_capability};
use ai_brains_core::scope::GrantCapability;

#[test]
fn capability_report_claude() {
    let capability = adapter_capability(AdapterKind::Claude);
    assert_eq!(capability.name, "claude");
    assert_eq!(capability.level, CapabilityLevel::Full);
    assert!(capability.supports_hooks);
    assert!(
        !capability.governed_reads.is_empty(),
        "full harness adapters must declare non-empty governed_reads"
    );
    assert!(
        capability
            .governed_reads
            .contains(&GrantCapability::ReadEvidence)
    );
    assert!(
        capability
            .governed_reads
            .contains(&GrantCapability::ReadConclusions)
    );
    assert!(
        capability
            .governed_reads
            .contains(&GrantCapability::ReadDecisions)
    );
    assert!(capability.principal_binding.is_none());
}

#[test]
fn full_harness_adapters_declare_governed_reads() {
    for kind in [AdapterKind::Claude, AdapterKind::Gemini, AdapterKind::Codex] {
        let capability = adapter_capability(kind);
        assert_eq!(capability.level, CapabilityLevel::Full);
        assert!(
            !capability.governed_reads.is_empty(),
            "{kind:?} full harness must declare non-empty governed_reads"
        );
        assert!(capability.principal_binding.is_none());
    }
}
