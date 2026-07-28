//! Port trait: HTTP handlers call only this — never `GovernedServices` alone.
//!
//! The daemon implements this as a thin wrap of
//! `ai_brainsd::dispatch::handle_daemon_request`.

use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use async_trait::async_trait;
use thiserror::Error;

/// Error from the dispatch port (transport/runtime — not domain `ApiError`).
#[derive(Debug, Error)]
pub enum DispatchError {
    /// Operation not available on the HTTP `/v1` surface (e.g. Shutdown, MultiLine Sync).
    #[error("not supported over HTTP: {0}")]
    NotSupported(String),
    /// Internal daemon/runtime failure.
    #[error("internal dispatch error: {0}")]
    Internal(String),
}

/// Injected by the daemon (or tests) so `ai-brains-api-server` never depends on `ai-brainsd`.
#[async_trait]
pub trait HttpDispatch: Send + Sync {
    async fn dispatch(&self, request: DaemonRequest) -> Result<DaemonResponse, DispatchError>;
}

/// Test helpers (also usable from integration tests).
pub mod test_support {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    type Handler =
        Arc<dyn Fn(DaemonRequest) -> Result<DaemonResponse, DispatchError> + Send + Sync>;

    /// Closure-backed mock for route unit tests.
    pub struct MockHttpDispatch {
        handler: Handler,
        pub calls: Arc<Mutex<Vec<DaemonRequest>>>,
    }

    impl MockHttpDispatch {
        pub fn new(
            handler: impl Fn(DaemonRequest) -> Result<DaemonResponse, DispatchError>
            + Send
            + Sync
            + 'static,
        ) -> Self {
            Self {
                handler: Arc::new(handler),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn always(response: DaemonResponse) -> Self {
            Self::new(move |_req| Ok(response.clone()))
        }
    }

    #[async_trait]
    impl HttpDispatch for MockHttpDispatch {
        async fn dispatch(&self, request: DaemonRequest) -> Result<DaemonResponse, DispatchError> {
            {
                let mut guard = self.calls.lock().await;
                guard.push(request.clone());
            }
            (self.handler)(request)
        }
    }
}
