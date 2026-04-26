use ai_brains_security::{redact_text, scan_text};

#[test]
fn redaction_preserves_readability() {
    let input = "Connect with Authorization: Bearer abcdefghijklmnopQRST1234 before retrying.";
    let findings = scan_text(input);
    let redacted = redact_text(input, &findings);

    assert!(redacted.contains("Connect with"));
    assert!(redacted.contains("before retrying."));
    assert!(!redacted.contains("abcdefghijklmnopQRST1234"));
}
