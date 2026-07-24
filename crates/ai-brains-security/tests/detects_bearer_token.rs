use ai_brains_security::{SecretKind, scan_text};

#[test]
fn detects_bearer_token() {
    let findings = scan_text("Authorization: Bearer abcdefghijklmnopQRST1234");
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == SecretKind::BearerToken)
    );
}
