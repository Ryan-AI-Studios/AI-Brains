#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_core::model_provenance::reason;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_models::mock::MockProvider;
use ai_brains_models::registry::{ALLOW_CLOUD_EXTRACTION_ENV, ProviderRegistry};
use ai_brains_models::{CompletionResponse, ModelError};

fn cloud_mock(label: &str) -> MockProvider {
    let mut mock = MockProvider::new(vec![CompletionResponse {
        text: format!("{label} response"),
        model: "mock".to_string(),
    }]);
    mock.is_local = false;
    mock
}

fn local_mock(label: &str) -> MockProvider {
    MockProvider::new(vec![CompletionResponse {
        text: format!("{label} response"),
        model: "mock".to_string(),
    }])
}

fn privacy_violation_code(err: ModelError) -> String {
    match err {
        ModelError::PrivacyViolation(code) => code,
        other => panic!("expected PrivacyViolation, got {other:?}"),
    }
}

#[test]
fn test_registry_blocks_cloud_for_local_only() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_err());
}

#[test]
fn test_registry_allows_local_for_local_only() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(local_mock("local")));
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_ok());
    assert!(result.expect("local provider").is_local());
}

#[test]
fn provider_registry__sealed__skips_non_local_provider() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    registry.register(Box::new(local_mock("local")));
    let selected = registry
        .select_provider(&Privacy::Sealed)
        .expect("local must be selected for Sealed");
    assert!(selected.is_local());
    assert_eq!(selected.name(), "mock");
}

#[test]
fn provider_registry__never_inject__skips_non_local_provider() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    registry.register(Box::new(local_mock("local")));
    let selected = registry
        .select_provider(&Privacy::NeverInject)
        .expect("local must be selected for NeverInject");
    assert!(selected.is_local());
}

#[test]
fn provider_registry__local_only__skips_non_local_provider() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    registry.register(Box::new(local_mock("local")));
    let selected = registry
        .select_provider(&Privacy::LocalOnly)
        .expect("local must be selected for LocalOnly");
    assert!(selected.is_local());
}

#[test]
fn provider_registry__only_remote_registered_sealed__privacy_violation() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    let err = match registry.select_provider(&Privacy::Sealed) {
        Err(e) => e,
        Ok(_) => panic!("Sealed + only remote must deny"),
    };
    let code = privacy_violation_code(err);
    assert_eq!(
        code,
        reason::PRIVACY_ROUTE_MISMATCH,
        "Sealed + only remote must surface privacy_route_mismatch"
    );
    // No Privacy Debug dump
    assert!(!code.contains("Sealed"));
    assert!(!code.contains("Privacy"));
}

#[test]
fn provider_registry__allow_cloud_flag_off__denies_remote_even_cloudok() {
    let _guard = TempEnv::remove(ALLOW_CLOUD_EXTRACTION_ENV);
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud")));
    let err = match registry.select_provider(&Privacy::CloudOk) {
        Err(e) => e,
        Ok(_) => panic!("CloudOk + remote + flag off must deny"),
    };
    let code = privacy_violation_code(err);
    assert_eq!(code, reason::CLOUD_EXTRACTION_DISABLED);
}

#[test]
fn provider_registry__local_and_remote_both_allowed__prefers_local() {
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "1");
    let mut registry = ProviderRegistry::new();
    // Local registered first
    registry.register(Box::new(local_mock("local-a")));
    registry.register(Box::new(cloud_mock("cloud-b")));
    let selected = registry
        .select_provider(&Privacy::CloudOk)
        .expect("must select a provider");
    assert!(selected.is_local(), "must prefer local when both allowed");
}

#[test]
fn provider_registry__registration_order_remote_first__still_prefers_local() {
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "true");
    let mut registry = ProviderRegistry::new();
    // Remote registered first — local-first must still win
    registry.register(Box::new(cloud_mock("cloud-first")));
    registry.register(Box::new(local_mock("local-second")));
    let selected = registry
        .select_provider(&Privacy::CloudOk)
        .expect("must select a provider");
    assert!(
        selected.is_local(),
        "local-first must beat registration order"
    );
}

#[test]
fn provider_registry__allow_cloud_flag_on__cloudok_selects_remote_when_only_remote() {
    let _guard = TempEnv::set(ALLOW_CLOUD_EXTRACTION_ENV, "yes");
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(cloud_mock("cloud-only")));
    let selected = registry
        .select_provider(&Privacy::CloudOk)
        .expect("CloudOk + flag on + remote only must allow");
    assert!(!selected.is_local());
}
