#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use ai_brains_models::ModelProvider;
use ai_brains_models::llama_cpp::LlamaCppProvider;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider(uri: String) -> LlamaCppProvider {
    LlamaCppProvider::with_timeouts(
        uri,
        "test-model".to_string(),
        Duration::from_secs(2),
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
}

#[tokio::test]
async fn llama_cpp_embed_batch__two_inputs__one_post_two_vectors() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"index": 0, "embedding": [0.1, 0.2]},
                {"index": 1, "embedding": [0.3, 0.4]}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let out = provider(mock_server.uri())
        .embed_batch(vec!["alpha".to_string(), "beta".to_string()])
        .await
        .expect("embed_batch two inputs");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].vector, vec![0.1_f32, 0.2_f32]);
    assert_eq!(out[1].vector, vec![0.3_f32, 0.4_f32]);

    let reqs = mock_server
        .received_requests()
        .await
        .expect("received_requests");
    assert_eq!(reqs.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).expect("request body json");
    assert!(
        body["input"].is_array(),
        "two-input body must serialize input as a JSON array, got {body}"
    );
    assert_eq!(body["input"].as_array().map(|a| a.len()), Some(2));
}

#[tokio::test]
async fn llama_cpp_embed_batch__one_input__json_array_not_string() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"index": 0, "embedding": [0.5, 0.6]}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let out = provider(mock_server.uri())
        .embed_batch(vec!["solo".to_string()])
        .await
        .expect("embed_batch one input");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].vector, vec![0.5_f32, 0.6_f32]);

    let reqs = mock_server
        .received_requests()
        .await
        .expect("received_requests");
    assert_eq!(reqs.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&reqs[0].body).expect("request body json");
    assert!(
        body["input"].is_array(),
        "len-1 body must be a JSON array, got {body}"
    );
    assert!(
        !body["input"].is_string(),
        "len-1 input must not be a JSON string"
    );
    let arr = body["input"].as_array().expect("input array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], "solo");
}
