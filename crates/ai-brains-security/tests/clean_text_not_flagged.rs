use ai_brains_security::scan_text;

#[test]
fn clean_text_not_flagged() {
    let findings = scan_text("Normal engineering note without credentials or secrets.");
    assert!(findings.is_empty());
}
