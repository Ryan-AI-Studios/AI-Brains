use ai_brains_core::privacy::*;

#[test]
fn test_privacy_strictest_wins() {
    assert_eq!(
        Privacy::LocalOnly.combine(Privacy::CloudOk),
        Privacy::LocalOnly
    );
    assert_eq!(Privacy::Sealed.combine(Privacy::LocalOnly), Privacy::Sealed);
    assert_eq!(
        Privacy::NeverInject.combine(Privacy::CloudOk),
        Privacy::NeverInject
    );
    assert_eq!(
        Privacy::NeverInject.combine(Privacy::LocalOnly),
        Privacy::NeverInject
    );
    assert_eq!(
        Privacy::Sealed.combine(Privacy::NeverInject),
        Privacy::Sealed
    );
}
