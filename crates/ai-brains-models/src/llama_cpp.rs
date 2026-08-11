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

/// Soft liveness result for local router / llama.cpp endpoints (T229 F2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStatus {
    Ok,
    Down,
    Timeout,
    Error,
}

impl ProbeStatus {
    /// Human label for status lines (`probe=ok|down|timeout|error`).
    pub fn as_label(self) -> &'static str {
        match self {
            ProbeStatus::Ok => "ok",
            ProbeStatus::Down => "down",
            ProbeStatus::Timeout => "timeout",
            ProbeStatus::Error => "error",
        }
    }
}

pub struct LlamaCppProvider {
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

    /// Soft liveness probe. `timeout` is independent of completion/embedding timeouts
    /// (do **not** use the 120s LLM timeout — callers pass ~2s).
    ///
    /// Sequence: `GET {endpoint}/health` → if 404, `GET {endpoint}/v1/models`.
    /// Map: 200→[`Ok`](ProbeStatus::Ok), connect fail→[`Down`](ProbeStatus::Down),
    /// timeout→[`Timeout`](ProbeStatus::Timeout), other→[`Error`](ProbeStatus::Error).
    pub async fn probe_health(&self, timeout: Duration) -> ProbeStatus {
        match self.probe_get("/health", timeout).await {
            ProbeHttpOutcome::Ok => ProbeStatus::Ok,
            ProbeHttpOutcome::NotFound => match self.probe_get("/v1/models", timeout).await {
                ProbeHttpOutcome::Ok => ProbeStatus::Ok,
                ProbeHttpOutcome::NotFound => ProbeStatus::Error,
                ProbeHttpOutcome::Down => ProbeStatus::Down,
                ProbeHttpOutcome::Timeout => ProbeStatus::Timeout,
                ProbeHttpOutcome::Error => ProbeStatus::Error,
            },
            ProbeHttpOutcome::Down => ProbeStatus::Down,
            ProbeHttpOutcome::Timeout => ProbeStatus::Timeout,
            ProbeHttpOutcome::Error => ProbeStatus::Error,
        }
    }

    async fn probe_get(&self, path: &str, timeout: Duration) -> ProbeHttpOutcome {
        let base = self.endpoint.trim_end_matches('/');
        let url = format!("{base}{path}");
        // Probe uses a short-lived client with **connect_timeout** = probe budget.
        // The shared LLM client cannot set connect_timeout per request; without it,
        // Windows often surfaces closed loopback as Timeout instead of Down (AC6).
        // Request timeout stays independent of the 120s completion timeout (F2).
        let client = match reqwest::Client::builder()
            .connect_timeout(timeout)
            .timeout(timeout)
            .build()
        {
            Ok(c) => c,
            Err(_) => return ProbeHttpOutcome::Error,
        };
        match client.get(&url).send().await {
            Ok(res) => {
                let status = res.status();
                if status.as_u16() == 200 {
                    ProbeHttpOutcome::Ok
                } else if status.as_u16() == 404 {
                    ProbeHttpOutcome::NotFound
                } else {
                    ProbeHttpOutcome::Error
                }
            }
            Err(e) => classify_probe_transport_error(&e),
        }
    }
}

/// Internal HTTP outcome for probe path routing (404 → fallback path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProbeHttpOutcome {
    Ok,
    NotFound,
    Down,
    Timeout,
    Error,
}

/// Map reqwest transport errors to probe outcomes (T229 F2 / AC6).
fn classify_probe_transport_error(e: &reqwest::Error) -> ProbeHttpOutcome {
    classify_probe_transport_signals(e.is_connect(), e.is_timeout(), &e.to_string())
}

/// Pure signal classification for probe transport (unit-testable, hermetic).
///
/// - **Down** — connect/DNS/refuse (including Windows WSAECONNREFUSED messages).
/// - **Timeout** — peer delayed past the probe budget without a connect-class signal.
/// - **Error** — other transport failures (TLS/protocol/builder/etc.); never map to Down.
pub(crate) fn classify_probe_transport_signals(
    is_connect: bool,
    is_timeout: bool,
    message: &str,
) -> ProbeHttpOutcome {
    let msg = message.to_ascii_lowercase();
    let connect_like = is_connect
        || msg.contains("connection refused")
        || msg.contains("actively refused")
        || msg.contains("os error 10061")
        || msg.contains("failed to connect")
        || msg.contains("dns")
        || msg.contains("name or service not known")
        || msg.contains("no such host")
        || msg.contains("nodename nor servname");
    if connect_like {
        ProbeHttpOutcome::Down
    } else if is_timeout {
        ProbeHttpOutcome::Timeout
    } else {
        ProbeHttpOutcome::Error
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
    fn llama_cpp_provider__remote_endpoint__is_local_false() {
        let p = LlamaCppProvider::new("https://gpu-box.example.com:8080".into(), "model".into());
        assert!(!p.is_local());
        assert_eq!(p.endpoint_class(), EndpointClass::CloudApi);
    }

    #[test]
    fn llama_cpp_provider__loopback_endpoint__is_local_true() {
        let p = LlamaCppProvider::new("http://127.0.0.1:8080".into(), "model".into());
        assert!(p.is_local());
        assert_eq!(p.endpoint_class(), EndpointClass::LocalLoopback);
    }

    #[test]
    fn probe_status__as_label__maps_all_variants() {
        assert_eq!(ProbeStatus::Ok.as_label(), "ok");
        assert_eq!(ProbeStatus::Down.as_label(), "down");
        assert_eq!(ProbeStatus::Timeout.as_label(), "timeout");
        assert_eq!(ProbeStatus::Error.as_label(), "error");
    }

    /// F2: connect/refuse messages → Down; timeout flag → Timeout; other → Error (not Down).
    #[test]
    fn classify_probe_transport_signals__mapping() {
        assert_eq!(
            classify_probe_transport_signals(true, false, "anything"),
            ProbeHttpOutcome::Down
        );
        assert_eq!(
            classify_probe_transport_signals(false, false, "connection refused"),
            ProbeHttpOutcome::Down
        );
        assert_eq!(
            classify_probe_transport_signals(false, false, "actively refused by host"),
            ProbeHttpOutcome::Down
        );
        assert_eq!(
            classify_probe_transport_signals(false, false, "os error 10061"),
            ProbeHttpOutcome::Down
        );
        assert_eq!(
            classify_probe_transport_signals(false, true, "operation timed out"),
            ProbeHttpOutcome::Timeout
        );
        // Non-connect, non-timeout transport (TLS/protocol) must be Error, not Down.
        assert_eq!(
            classify_probe_transport_signals(false, false, "tls handshake failure"),
            ProbeHttpOutcome::Error
        );
        assert_eq!(
            classify_probe_transport_signals(false, false, "invalid certificate"),
            ProbeHttpOutcome::Error
        );
    }
}
