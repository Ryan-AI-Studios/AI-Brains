use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::neutral_event::NeutralEvent;
use serde_json::Value;

pub fn claude_capability() -> AdapterCapability {
    AdapterCapability {
        name: "claude".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Parses stop payloads and supports user-level hook configuration.".to_string(),
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
