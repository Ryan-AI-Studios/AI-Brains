use crate::antigravity::antigravity_capability;
use crate::capability::AdapterCapability;
use crate::claude::claude_capability;
use crate::codex::codex_capability;
use crate::gemini::gemini_capability;
use crate::opencode::opencode_capability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    Claude,
    Gemini,
    Codex,
    OpenCode,
    Antigravity,
}

pub fn adapter_capability(kind: AdapterKind) -> AdapterCapability {
    match kind {
        AdapterKind::Claude => claude_capability(),
        AdapterKind::Gemini => gemini_capability(),
        AdapterKind::Codex => codex_capability(),
        AdapterKind::OpenCode => opencode_capability(),
        AdapterKind::Antigravity => antigravity_capability(),
    }
}
