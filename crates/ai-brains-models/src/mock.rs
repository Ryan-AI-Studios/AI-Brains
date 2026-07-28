use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result, TokenizeRequest, TokenizeResponse,
};
use async_trait::async_trait;
use std::sync::Mutex;

pub struct MockProvider {
    pub responses: Mutex<Vec<CompletionResponse>>,
    pub is_local: bool,
    /// Number of `complete` invocations (including denied/empty-queue paths).
    pub complete_calls: Mutex<usize>,
    /// Number of `embed` invocations.
    pub embed_calls: Mutex<usize>,
    /// When true, `complete` / `embed` return Err after incrementing the call counter.
    /// Use to prove privacy gates never reach the model.
    pub fail_if_called: bool,
}

impl MockProvider {
    pub fn new(responses: Vec<CompletionResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            is_local: true,
            complete_calls: Mutex::new(0),
            embed_calls: Mutex::new(0),
            fail_if_called: false,
        }
    }

    /// Builder: fail any complete/embed call (after recording the attempt).
    pub fn failing_if_called(mut self) -> Self {
        self.fail_if_called = true;
        self
    }

    pub fn complete_call_count(&self) -> usize {
        self.complete_calls.lock().map(|c| *c).unwrap_or(usize::MAX)
    }

    pub fn embed_call_count(&self) -> usize {
        self.embed_calls.lock().map(|c| *c).unwrap_or(usize::MAX)
    }

    fn bump_complete(&self) -> Result<()> {
        if let Ok(mut c) = self.complete_calls.lock() {
            *c = c.saturating_add(1);
        }
        if self.fail_if_called {
            return Err(ModelError::Provider(
                "mock fail_if_called: complete must not be invoked".into(),
            ));
        }
        Ok(())
    }

    fn bump_embed(&self) -> Result<()> {
        if let Ok(mut c) = self.embed_calls.lock() {
            *c = c.saturating_add(1);
        }
        if self.fail_if_called {
            return Err(ModelError::Provider(
                "mock fail_if_called: embed must not be invoked".into(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
        self.bump_complete()?;
        let mut responses = self
            .responses
            .lock()
            .map_err(|e| ModelError::Provider(format!("mock response lock poisoned: {e}")))?;
        if responses.is_empty() {
            return Ok(CompletionResponse {
                text: "No more mock responses".to_string(),
                model: "mock".to_string(),
            });
        }
        Ok(responses.remove(0))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.bump_embed()?;
        Ok(EmbeddingResponse {
            vector: vec![0.0; 1536],
        })
    }

    async fn tokenize(&self, request: TokenizeRequest) -> Result<TokenizeResponse> {
        // Mock tokenization: 1 token per word-like unit
        let tokens = request
            .text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect();
        Ok(TokenizeResponse { tokens })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_local(&self) -> bool {
        self.is_local
    }
}
