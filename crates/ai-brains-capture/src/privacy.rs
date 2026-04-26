use ai_brains_core::privacy::Privacy;
use ai_brains_security::{escalate_privacy, scan_text};

pub fn effective_privacy(content: &str, base: Privacy) -> Privacy {
    let findings = scan_text(content);
    escalate_privacy(base, &findings)
}
