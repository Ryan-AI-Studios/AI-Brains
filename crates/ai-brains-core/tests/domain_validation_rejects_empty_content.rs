use ai_brains_core::validation::*;

#[test]
fn test_domain_validation_rejects_empty_content() {
    let content = "";
    let res = validate_content(content);
    assert!(res.is_err());
}
