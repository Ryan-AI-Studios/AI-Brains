use crate::finding::{Confidence, Finding};
use ai_brains_core::privacy::Privacy;

pub fn escalate_privacy(current: Privacy, findings: &[Finding]) -> Privacy {
    findings.iter().fold(current, |privacy, finding| {
        let escalated = match finding.confidence {
            Confidence::High => Privacy::Sealed,
            Confidence::Likely => Privacy::LocalOnly,
        };
        privacy.combine(escalated)
    })
}
