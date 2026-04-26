use ai_brains_adapters::render_hook_output;
use ai_brains_contracts::hook::HookResponse;
use serde_json::json;

#[test]
fn hook_output_is_json_only() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_hook_output(json!({"ok": true}))?;
    let parsed: HookResponse = serde_json::from_str(&rendered)?;
    assert!(parsed.success);
    assert!(!rendered.contains('\n'));
    Ok(())
}
