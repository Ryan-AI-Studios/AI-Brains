#![allow(non_snake_case)]

use ai_brains_adapters::parse_claude_stop_payload;

#[test]
fn claude_stop_payload_parsed() -> Result<(), Box<dyn std::error::Error>> {
    let payload: serde_json::Value = serde_json::from_str(
        r#"{"role":"assistant","content":"final answer","stop_reason":"end_turn"}"#,
    )?;

    let event = parse_claude_stop_payload(&payload)?;
    assert_eq!(event.role, "assistant");
    assert_eq!(event.content, "final answer");
    assert_eq!(event.status.as_deref(), Some("end_turn"));
    Ok(())
}

#[test]
fn claude_stop_payload__tool_only__empty_after_filter() -> Result<(), Box<dyn std::error::Error>> {
    let payload: serde_json::Value = serde_json::from_str(
        r#"{"role":"assistant","content":{"type":"tool_use","name":"bash"}}"#,
    )?;
    let event = parse_claude_stop_payload(&payload)?;
    assert!(event.content.is_empty());
    assert!(event.warnings.iter().any(|w| w.contains("message-only")));
    Ok(())
}
