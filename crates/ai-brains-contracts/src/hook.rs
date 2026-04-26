use serde::{Deserialize, Serialize};

/// Represents a hook execution request sent to external integrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRequest {
    pub hook_type: String,
    pub payload: serde_json::Value,
}

/// Represents the strict response format expected from a hook.
///
/// This struct is designed to facilitate strict parsing of hook outputs.
/// Hooks often output noise to stdout (e.g., debugging info, logs).
/// The actual JSON contract should be identifiable and parseable despite this noise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl HookResponse {
    /// Attempts to find and parse a HookResponse from a string that may contain noise.
    /// This is useful when capturing stdout from a script that might print other things.
    pub fn from_stdout(stdout: &str) -> Option<Self> {
        // Look for the first occurrence of '{' and the last occurrence of '}'
        let start = stdout.find('{')?;
        let end = stdout.rfind('}')?;

        if start > end {
            return None;
        }

        let json_part = &stdout[start..=end];
        serde_json::from_str(json_part).ok()
    }
}
