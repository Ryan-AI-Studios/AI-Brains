//! In-process HTTP surface for `ai-brainsd` (T161).
//!
//! [`DaemonHttpDispatch`] is a thin wrap of [`crate::dispatch::handle_daemon_request`]
//! so pipe, Windows service, and HTTP are three callers of one function.

use std::net::SocketAddr;
use std::sync::Arc;

use ai_brains_api_server::dispatch::{DispatchError, HttpDispatch};
use ai_brains_api_server::{
    AppState, BindError, app_state, ensure_token, resolve_bind_addr, serve,
};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use async_trait::async_trait;
use tokio::net::TcpListener;

use crate::DaemonWriter;
use crate::dispatch::{LiveDispatchResult, handle_daemon_request};
use crate::services::GovernedServices;

/// Thin adapter: HTTP → `handle_daemon_request` (mutations + queries).
#[derive(Clone)]
pub struct DaemonHttpDispatch {
    writer: DaemonWriter,
    services: GovernedServices,
}

impl DaemonHttpDispatch {
    pub fn new(writer: DaemonWriter, services: GovernedServices) -> Self {
        Self { writer, services }
    }
}

#[async_trait]
impl HttpDispatch for DaemonHttpDispatch {
    async fn dispatch(&self, request: DaemonRequest) -> Result<DaemonResponse, DispatchError> {
        match handle_daemon_request(request, &self.writer, &self.services).await {
            Ok(LiveDispatchResult::Response(resp)) => Ok(*resp),
            Ok(LiveDispatchResult::Shutdown) => Err(DispatchError::NotSupported(
                "shutdown is not available over HTTP /v1".into(),
            )),
            Ok(LiveDispatchResult::MultiLine(_)) => Err(DispatchError::NotSupported(
                "legacy multiline Sync query is not available on HTTP /v1".into(),
            )),
            Err(e) => Err(DispatchError::Internal(e.to_string())),
        }
    }
}

/// True when HTTP should start: `AI_BRAINS_HTTP=1|true|yes` or `--http` in args.
pub fn http_enabled_from_env_and_args(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--http") {
        return true;
    }
    match std::env::var("AI_BRAINS_HTTP") {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// Parse optional `--http-bind <addr>` from argv.
pub fn parse_http_bind_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == "--http-bind" {
            return iter.next().cloned();
        }
        if let Some(rest) = a.strip_prefix("--http-bind=") {
            return Some(rest.to_string());
        }
    }
    None
}

/// Resolve bind address for daemon HTTP (double-lock non-loopback policy).
pub fn resolve_daemon_http_bind(args: &[String]) -> Result<SocketAddr, BindError> {
    let explicit = parse_http_bind_arg(args);
    resolve_bind_addr(explicit.as_deref(), None)
}

/// Load/create bearer token and build router state for the live daemon.
pub fn build_http_state(
    writer: DaemonWriter,
    services: GovernedServices,
) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let (path, token) = ensure_token(None).map_err(|e| e.to_string())?;
    tracing::info!(
        path = %path.display(),
        "HTTP bearer token ready (token not logged)"
    );
    let dispatch = Arc::new(DaemonHttpDispatch::new(writer, services));
    let dispatch: Arc<dyn HttpDispatch> = dispatch;
    // Move Zeroizing into AuthConfig (no plain String intermediate).
    Ok(app_state(dispatch, token))
}

/// Spawn the HTTP server task. Returns the bound local address.
pub async fn spawn_http_server(
    addr: SocketAddr,
    state: AppState,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    tokio::spawn(async move {
        let shutdown = async move {
            let _ = shutdown_rx.recv().await;
        };
        if let Err(e) = serve(listener, state, shutdown).await {
            tracing::error!("HTTP server exited with error: {e}");
        }
    });
    Ok(local)
}

/// Start HTTP if enabled by env/flag. Logs bind address on success.
pub async fn maybe_start_http(
    args: &[String],
    writer: DaemonWriter,
    services: GovernedServices,
    shutdown_tx: &tokio::sync::broadcast::Sender<()>,
) -> Result<Option<SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
    if !http_enabled_from_env_and_args(args) {
        return Ok(None);
    }

    let addr = resolve_daemon_http_bind(args).map_err(|e| e.to_string())?;
    let state = build_http_state(writer, services)?;
    let shutdown_rx = shutdown_tx.subscribe();
    let local = spawn_http_server(addr, state, shutdown_rx).await?;
    println!(
        "AI-Brains HTTP API listening on http://{local}/v1 (bearer required; token file under %USERPROFILE%\\.ai-brains\\http.token)"
    );
    Ok(Some(local))
}
