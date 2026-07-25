#![allow(non_snake_case)]
#![allow(clippy::disallowed_methods)]

use ai_brains_core::ids::PrincipalId;
use ai_brains_core::principal::{Principal, PrincipalKind};
use ai_brains_core::scope::GrantCapability;
use ai_brains_core::source::SourceKind;

#[test]
fn PrincipalKind_Connector__serde_roundtrip__preserves_variant() {
    let kind = PrincipalKind::Connector;
    let json = serde_json::to_string(&kind).expect("serialize");
    assert_eq!(json, "\"Connector\"");
    let back: PrincipalKind = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, PrincipalKind::Connector);
}

#[test]
fn PrincipalKind_System__serde_roundtrip__preserves_variant() {
    let kind = PrincipalKind::System;
    let json = serde_json::to_string(&kind).expect("serialize");
    assert_eq!(json, "\"System\"");
    let back: PrincipalKind = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, PrincipalKind::System);
}

#[test]
fn PrincipalKind_Service__serde_roundtrip__legacy_retained() {
    let kind = PrincipalKind::Service;
    let json = serde_json::to_string(&kind).expect("serialize");
    assert_eq!(json, "\"Service\"");
    let back: PrincipalKind = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, PrincipalKind::Service);
}

#[test]
fn Principal__bindings_default_empty__on_missing_fields() {
    let id = PrincipalId::new();
    let json = format!(r#"{{"id":"{}","kind":"Human","display_name":"Ryan"}}"#, id);
    let principal: Principal = serde_json::from_str(&json).expect("deserialize without bindings");
    assert_eq!(principal.id, id);
    assert_eq!(principal.kind, PrincipalKind::Human);
    assert_eq!(principal.display_name, "Ryan");
    assert!(principal.bound_source_kinds.is_empty());
    assert!(principal.bound_capabilities.is_empty());
}

#[test]
fn Principal__bindings__roundtrip_when_present() {
    let principal = Principal {
        id: PrincipalId::new(),
        kind: PrincipalKind::Connector,
        display_name: "git-connector".into(),
        bound_source_kinds: vec![SourceKind::GitRepository, SourceKind::File],
        bound_capabilities: vec![GrantCapability::ReadEvidence],
    };
    let json = serde_json::to_string(&principal).expect("serialize");
    let back: Principal = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, principal);
}
