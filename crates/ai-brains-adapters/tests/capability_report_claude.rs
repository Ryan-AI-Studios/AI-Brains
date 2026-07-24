use ai_brains_adapters::{AdapterKind, CapabilityLevel, adapter_capability};

#[test]
fn capability_report_claude() {
    let capability = adapter_capability(AdapterKind::Claude);
    assert_eq!(capability.name, "claude");
    assert_eq!(capability.level, CapabilityLevel::Full);
    assert!(capability.supports_hooks);
}
