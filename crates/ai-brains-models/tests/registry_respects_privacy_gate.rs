use ai_brains_core::privacy::Privacy;
use ai_brains_models::mock::MockProvider;
use ai_brains_models::registry::ProviderRegistry;

#[test]
fn test_registry_blocks_cloud_for_local_only() {
    let mut registry = ProviderRegistry::new();
    
    // Register a cloud-only provider
    registry.register(Box::new(MockProvider {
        response: "cloud response".to_string(),
        is_local: false,
    }));
    
    // Try to select for LocalOnly
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Privacy violation"));
    }
}

#[test]
fn test_registry_allows_local_for_local_only() {
    let mut registry = ProviderRegistry::new();
    
    // Register a local provider
    registry.register(Box::new(MockProvider {
        response: "local response".to_string(),
        is_local: true,
    }));
    
    // Select for LocalOnly
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_ok());
    assert!(result.unwrap().is_local());
}
