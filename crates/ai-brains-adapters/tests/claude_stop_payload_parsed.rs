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
