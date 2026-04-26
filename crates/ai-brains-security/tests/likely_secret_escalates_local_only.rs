use ai_brains_core::privacy::Privacy;
use ai_brains_security::{escalate_privacy, scan_text};

#[test]
fn likely_secret_escalates_local_only() {
    let findings = scan_text("Authorization: Bearer abcdefghijklmnopQRST1234");
    let privacy = escalate_privacy(Privacy::CloudOk, &findings);
    assert_eq!(privacy, Privacy::LocalOnly);
}
