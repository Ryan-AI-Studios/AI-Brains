use crate::capability::{AdapterCapability, CapabilityLevel, full_harness_governed_reads};
use crate::neutral_event::NeutralEvent;
use ai_brains_core::scope::GrantCapability;
use serde_json::Value;

pub fn claude_capability() -> AdapterCapability {
    AdapterCapability {
        name: "claude".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Parses stop payloads and supports user-level hook configuration. Full harnesses bind as PrincipalKind::Agent (not Connector) so ProposeConclusion is in-matrix; principal_binding deferred until registry wiring. Connector observe-only remains ReadEvidence.".to_string(),
        governed_reads: full_harness_governed_reads(),
        governed_writes: vec![GrantCapability::ProposeConclusion],
        principal_binding: None,
    }
}

pub fn parse_claude_stop_payload(value: &Value) -> crate::Result<NeutralEvent> {
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("assistant")
        .to_string();
    let content = value
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let status = value
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(NeutralEvent {
        role,
        content,
        status,
        warnings: Vec::new(),
    })
}
