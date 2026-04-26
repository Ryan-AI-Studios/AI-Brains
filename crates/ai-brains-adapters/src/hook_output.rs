use ai_brains_contracts::hook::HookResponse;
use serde_json::Value;

pub fn render_hook_output(result: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&HookResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}
