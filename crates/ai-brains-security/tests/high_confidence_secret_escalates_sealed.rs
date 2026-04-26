use ai_brains_core::privacy::Privacy;
use ai_brains_security::{escalate_privacy, scan_text};

#[test]
fn high_confidence_secret_escalates_sealed() {
    let text = "-----BEGIN PRIVATE KEY-----\nabc123secretmaterial\n-----END PRIVATE KEY-----";
    let findings = scan_text(text);
    let privacy = escalate_privacy(Privacy::CloudOk, &findings);
    assert_eq!(privacy, Privacy::Sealed);
}
