//! T229 F2 / AC6 — soft probe_health on LlamaCppProvider (wiremock).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_models::llama_cpp::{LlamaCppProvider, ProbeStatus};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// AC6: HTTP 200 on `/health` → Ok.
#[tokio::test]
async fn probe_health__health_200__ok() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let provider = LlamaCppProvider::new(mock_server.uri(), "test-model".to_string());
    let status = provider.probe_health(Duration::from_secs(2)).await;
    assert_eq!(status, ProbeStatus::Ok);
}

/// AC6: `/health` 404 then `/v1/models` 200 → Ok (llama.cpp fallback path).
#[tokio::test]
async fn probe_health__health_404_models_200__ok() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": []
        })))
        .mount(&mock_server)
        .await;

    let provider = LlamaCppProvider::new(mock_server.uri(), "test-model".to_string());
    let status = provider.probe_health(Duration::from_secs(2)).await;
    assert_eq!(status, ProbeStatus::Ok);
}

/// AC6: closed loopback is non-Ok (hermetic: no DNS / no external network).
///
/// Windows often surfaces closed ports as **Timeout** rather than refuse; Linux
/// typically **Down**. Pure refuse→Down / other→Error mapping is unit-tested in
/// `llama_cpp.rs` (`classify_probe_transport_signals`).
#[tokio::test]
async fn probe_health__closed_loopback__not_ok() {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback for refuse fixture");
    let port = listener.local_addr().expect("local_addr").port();
    drop(listener);

    let provider =
        LlamaCppProvider::new(format!("http://127.0.0.1:{port}"), "test-model".to_string());
    let status = provider.probe_health(Duration::from_secs(2)).await;
    assert!(
        matches!(status, ProbeStatus::Down | ProbeStatus::Timeout),
        "closed loopback must be Down or Timeout (never Ok/Error), got {status:?}"
    );
}

/// AC6: delayed response past probe timeout → Timeout.
#[tokio::test]
async fn probe_health__slow_response__timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(3)))
        .mount(&mock_server)
        .await;

    let provider = LlamaCppProvider::new(mock_server.uri(), "test-model".to_string());
    let start = std::time::Instant::now();
    let status = provider.probe_health(Duration::from_millis(500)).await;
    let elapsed = start.elapsed();

    assert_eq!(status, ProbeStatus::Timeout);
    assert!(
        elapsed < Duration::from_secs(2),
        "probe should honor short timeout, took {elapsed:?}"
    );
}

/// Non-200/non-404 on `/health` → Error (no silent Ok).
#[tokio::test]
async fn probe_health__health_500__error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let provider = LlamaCppProvider::new(mock_server.uri(), "test-model".to_string());
    let status = provider.probe_health(Duration::from_secs(2)).await;
    assert_eq!(status, ProbeStatus::Error);
}
