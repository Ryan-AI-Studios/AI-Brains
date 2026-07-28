use crate::endpoint::{classify_endpoint, endpoint_is_local};
use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result, TokenizeRequest, TokenizeResponse,
};
use ai_brains_core::model_provenance::EndpointClass;
use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OllamaCompletionRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a Option<String>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Serialize)]
struct OllamaTokenizeRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

pub struct OllamaProvider {
    endpoint: String,
    model: String,
    /// Classified once at construct from `endpoint` (not hardcoded).
    endpoint_class: EndpointClass,
    /// Derived from `endpoint_class` — loopback only is local for privacy.
    is_local: bool,
    client: reqwest::Client,
    completion_timeout: Duration,
    embedding_timeout: Duration,
    tokenize_timeout: Duration,
}

impl OllamaProvider {
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
        let endpoint_class = classify_endpoint(&endpoint);
        let is_local = endpoint_is_local(&endpoint);
        Self {
            endpoint,
            model,
            endpoint_class,
            is_local,
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
impl ModelProvider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let body = OllamaCompletionRequest {
            model: &self.model,
            prompt: &request.prompt,
            system: &request.system_prompt,
            stream: false,
            options: OllamaOptions {
                num_predict: request.max_tokens,
                temperature: request.temperature,
            },
        };

        let res = self
            .client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&body)
            .timeout(self.completion_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            return Err(ModelError::Provider(format!(
                "Ollama returned {}",
                res.status()
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;
        let text = json["response"]
            .as_str()
            .ok_or_else(|| ModelError::Provider("Missing response field".to_string()))?
            .to_string();

        Ok(CompletionResponse {
            text,
            model: self.model.clone(),
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let body = OllamaEmbeddingRequest {
            model: &self.model,
            prompt: &request.text,
        };

        let res = self
            .client
            .post(format!("{}/api/embeddings", self.endpoint))
            .json(&body)
            .timeout(self.embedding_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            return Err(ModelError::Provider(format!(
                "Ollama returned {}",
                res.status()
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;
        let vector = json["embedding"]
            .as_array()
            .ok_or_else(|| ModelError::Provider("Missing embedding field".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(EmbeddingResponse { vector })
    }

    async fn tokenize(&self, request: TokenizeRequest) -> Result<TokenizeResponse> {
        let body = OllamaTokenizeRequest {
            model: &self.model,
            prompt: &request.text,
        };

        let res = self
            .client
            .post(format!("{}/api/tokenize", self.endpoint))
            .json(&body)
            .timeout(self.tokenize_timeout)
            .send()
            .await
            .map_err(map_send_error)?;

        if !res.status().is_success() {
            return Err(ModelError::Provider(format!(
                "Ollama (tokenize) returned {}",
                res.status()
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
        "ollama"
    }

    fn is_local(&self) -> bool {
        self.is_local
    }

    fn endpoint_class(&self) -> EndpointClass {
        self.endpoint_class
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn ollama_provider__remote_endpoint__is_local_false() {
        let p = OllamaProvider::new("https://gpu-box.example.com:11434".into(), "qwen".into());
        assert!(!p.is_local());
        assert_eq!(p.endpoint_class(), EndpointClass::CloudApi);
    }

    #[test]
    fn ollama_provider__loopback_endpoint__is_local_true() {
        let p = OllamaProvider::new("http://127.0.0.1:11434".into(), "qwen".into());
        assert!(p.is_local());
        assert_eq!(p.endpoint_class(), EndpointClass::LocalLoopback);

        let p2 = OllamaProvider::new("http://localhost:11434".into(), "qwen".into());
        assert!(p2.is_local());
    }
}
