use ai_brains_security::{SecretKind, scan_text};

#[test]
fn detects_connection_string() {
    let findings = scan_text("Server=db.internal;Database=prod;User Id=sa;Password=supersecret;");
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == SecretKind::ConnectionString)
    );
}
