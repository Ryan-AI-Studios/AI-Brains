use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result, TokenizeRequest, TokenizeResponse,
};
use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct LlamaCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct LlamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Serialize)]
struct LlamaTokenizeRequest<'a> {
    content: &'a str,
}

pub struct LlamaCppProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
    completion_timeout: Duration,
    embedding_timeout: Duration,
    tokenize_timeout: Duration,
}

impl LlamaCppProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        let completion_timeout = Duration::from_secs(
            std::env::var("AI_BRAINS_LLM_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120),
        );
        let embedding_timeout = Duration::from_secs(
            std::env::var("AI_BRAINS_EMBEDDING_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        );
        let tokenize_timeout = Duration::from_secs(
            std::env::var("AI_BRAINS_TOKENIZE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        );
        Self::with_timeouts(
            endpoint,
            model,
            completion_timeout,
            embedding_timeout,
            tokenize_timeout,
        )
    }

    pub fn with_timeouts(
        endpoint: String,
        model: String,
        completion_timeout: Duration,
        embedding_timeout: Duration,
        tokenize_timeout: Duration,
    ) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
            completion_timeout,
            embedding_timeout,
            tokenize_timeout,
        }
    }
}

fn map_send_error(e: reqwest::Error) -> ModelError {
    if e.is_timeout() {
        ModelError::Timeout
    } else {
        ModelError::Network(e.to_string())
    }
}

#[async_trait]
impl ModelProvider for LlamaCppProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let system_prompt = request
            .system_prompt
            .as_deref()
            .unwrap_or("You are a helpful assistant.");
        let body = LlamaCompletionRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: &request.prompt,
                },
            ],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
        };

        let res = self
            .client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .json(&body)
            .timeout(self.completion_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::Provider(format!(
                "llama.cpp (completions) returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                ModelError::Provider("Missing choices[0].message.content field".to_string())
            })?
            .to_string();

        Ok(CompletionResponse {
            text,
            model: self.model.clone(),
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let body = LlamaEmbeddingRequest {
            model: &self.model,
            input: &request.text,
        };

        let res = self
            .client
            .post(format!("{}/v1/embeddings", self.endpoint))
            .json(&body)
            .timeout(self.embedding_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::Provider(format!(
                "llama.cpp (embeddings) returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;

        let vector = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| ModelError::Provider("Missing data[0].embedding field".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(EmbeddingResponse { vector })
    }

    async fn tokenize(&self, request: TokenizeRequest) -> Result<TokenizeResponse> {
        let body = LlamaTokenizeRequest {
            content: &request.text,
        };

        let res = self
            .client
            .post(format!("{}/tokenize", self.endpoint))
            .json(&body)
            .timeout(self.tokenize_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::Provider(format!(
                "llama.cpp (tokenize) returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;

        let tokens = json["tokens"]
            .as_array()
            .ok_or_else(|| ModelError::Provider("Missing tokens field".to_string()))?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u32)
            .collect();

        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "llama-cpp"
    }

    fn is_local(&self) -> bool {
        true
    }
}
