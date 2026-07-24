#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeGrant, ScopeRef, strictest_wins};

#[test]
fn conflicting_grants__stricter_privacy_and_narrower_capability_win() {
    let scope = ScopeRef::Repository(ProjectId::new());
    let broad = ScopeGrant {
        scope: scope.clone(),
        capability: GrantCapability::Erase,
        privacy: Privacy::CloudOk,
    };
    let narrow = ScopeGrant {
        scope: scope.clone(),
        capability: GrantCapability::ReadEvidence,
        privacy: Privacy::Sealed,
    };

    let combined = strictest_wins(&broad, &narrow);
    assert_eq!(combined.capability, GrantCapability::ReadEvidence);
    assert_eq!(combined.privacy, Privacy::Sealed);
    assert_eq!(combined.scope, scope);
}

#[test]
fn same_capability__stricter_privacy_wins() {
    let scope = ScopeRef::Repository(ProjectId::new());
    let a = ScopeGrant {
        scope: scope.clone(),
        capability: GrantCapability::ReadDecisions,
        privacy: Privacy::LocalOnly,
    };
    let b = ScopeGrant {
        scope,
        capability: GrantCapability::ReadDecisions,
        privacy: Privacy::NeverInject,
    };
    let combined = strictest_wins(&a, &b);
    assert_eq!(combined.privacy, Privacy::NeverInject);
    assert_eq!(combined.capability, GrantCapability::ReadDecisions);
}
