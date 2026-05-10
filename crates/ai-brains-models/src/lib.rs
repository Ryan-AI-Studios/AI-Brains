use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Privacy violation: {0}")]
    PrivacyViolation(String),
    #[error("Timeout")]
    Timeout,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizeResponse {
    pub tokens: Vec<u32>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;
    async fn tokenize(&self, request: TokenizeRequest) -> Result<TokenizeResponse>;
    fn name(&self) -> &str;
    fn is_local(&self) -> bool;
}

pub fn estimate_tokens(text: &str) -> usize {
    // Heuristic: 1 token per 3.5 characters (conservative)
    // Most models are ~4 chars/token, so 3.5 gives us a safety margin.
    (text.len() as f32 / 3.5).ceil() as usize
}

pub use mock::MockProvider;
pub mod llama_cpp;
pub mod mock;
pub mod ollama;
pub mod registry;
