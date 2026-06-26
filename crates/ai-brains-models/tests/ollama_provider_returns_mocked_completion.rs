#![allow(clippy::disallowed_methods)]

use ai_brains_models::ollama::OllamaProvider;
use ai_brains_models::{CompletionRequest, ModelError, ModelProvider};
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_ollama_completion() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "Hello from Ollama!"
        })))
        .mount(&mock_server)
        .await;

    let provider = OllamaProvider::new(mock_server.uri(), "llama3".to_string());
    let request = CompletionRequest {
        prompt: "Hi".to_string(),
        system_prompt: None,
        max_tokens: None,
        temperature: None,
    };

    let response = provider.complete(request).await?;
    assert_eq!(response.text, "Hello from Ollama!");
    assert_eq!(response.model, "llama3");

    Ok(())
}

#[tokio::test]
async fn ollama_provider_timeout_expires_returns_timeout_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(3)))
        .mount(&mock_server)
        .await;

    let provider = OllamaProvider::with_timeouts(
        mock_server.uri(),
        "llama3".to_string(),
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );

    let start = std::time::Instant::now();
    let result = provider
        .complete(CompletionRequest {
            prompt: "Hi".to_string(),
            system_prompt: None,
            max_tokens: None,
            temperature: None,
        })
        .await;

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "Request should timeout within ~1s, took {:?}",
        elapsed
    );
    assert!(
        matches!(result, Err(ModelError::Timeout)),
        "Expected ModelError::Timeout, got {:?}",
        result
    );
}

#[tokio::test]
async fn ollama_provider_refused_connection_returns_error_quickly() {
    let provider = OllamaProvider::with_timeouts(
        "http://127.0.0.1:1".to_string(),
        "llama3".to_string(),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );

    let start = std::time::Instant::now();
    let result = provider
        .complete(CompletionRequest {
            prompt: "Hi".to_string(),
            system_prompt: None,
            max_tokens: None,
            temperature: None,
        })
        .await;

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(3),
        "Refused connection should fail fast, took {:?}",
        elapsed
    );
    assert!(
        matches!(
            result,
            Err(ModelError::Timeout) | Err(ModelError::Network(_))
        ),
        "Expected Timeout or Network error, got {:?}",
        result
    );
}
