use ai_brains_core::privacy::Privacy;
use ai_brains_security::is_embeddable;

#[test]
fn sealed_content_not_embeddable() {
    assert!(!is_embeddable(Privacy::Sealed));
    assert!(!is_embeddable(Privacy::NeverInject));
    assert!(is_embeddable(Privacy::LocalOnly));
}
