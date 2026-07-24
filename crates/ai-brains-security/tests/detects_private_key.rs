use ai_brains_security::{SecretKind, scan_text};

#[test]
fn detects_private_key() {
    let text = "-----BEGIN PRIVATE KEY-----\nabc123secretmaterial\n-----END PRIVATE KEY-----";
    let findings = scan_text(text);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == SecretKind::PrivateKey)
    );
}
