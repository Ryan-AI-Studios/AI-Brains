#![allow(clippy::disallowed_methods)]

use ai_brains_models::llama_cpp::LlamaCppProvider;
use ai_brains_models::{CompletionRequest, ModelError, ModelProvider};
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

#[tokio::test]
async fn llama_cpp_provider_timeout_expires_returns_timeout_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(3)))
        .mount(&mock_server)
        .await;

    let provider = LlamaCppProvider::with_timeouts(
        mock_server.uri(),
        "test-model".to_string(),
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );

    let start = std::time::Instant::now();
    let result = provider
        .complete(CompletionRequest {
            prompt: "test".to_string(),
            system_prompt: None,
            max_tokens: Some(10),
            temperature: None,
        })
        .await;

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "Request should timeout within ~1s, took {:?}",
        elapsed
    );
    assert!(result.is_err());
    assert!(
        matches!(result, Err(ModelError::Timeout)),
        "Expected ModelError::Timeout, got {:?}",
        result
    );
}

#[tokio::test]
async fn llama_cpp_provider_refused_connection_returns_error_quickly() {
    let provider = LlamaCppProvider::with_timeouts(
        "http://127.0.0.1:1".to_string(),
        "test-model".to_string(),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );

    let start = std::time::Instant::now();
    let result = provider
        .complete(CompletionRequest {
            prompt: "test".to_string(),
            system_prompt: None,
            max_tokens: Some(10),
            temperature: None,
        })
        .await;

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(3),
        "Refused connection should fail fast, took {:?}",
        elapsed
    );
    assert!(result.is_err());
    assert!(
        matches!(
            result,
            Err(ModelError::Timeout) | Err(ModelError::Network(_))
        ),
        "Expected Timeout or Network error, got {:?}",
        result
    );
}
