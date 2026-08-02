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

/// Truthy env values shared by `AI_BRAINS_HTTP` and `AI_BRAINS_HTTP_SERVICE` (T195 F33):
/// `1` / `true` / `yes` (case-insensitive, trimmed).
pub fn is_http_env_truthy(raw: &str) -> bool {
    let t = raw.trim();
    t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
}

/// True when HTTP should start: `AI_BRAINS_HTTP=1|true|yes` or `--http` in args.
pub fn http_enabled_from_env_and_args(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--http") {
        return true;
    }
    match std::env::var("AI_BRAINS_HTTP") {
        Ok(v) => is_http_env_truthy(&v),
        Err(_) => false,
    }
}

/// Pure service-HTTP opt-in check (T195 F10/F33).
///
/// `None` / non-truthy → refuse service HTTP. Same truthy set as [`is_http_env_truthy`].
/// Gate is applied only in `windows_service` before [`maybe_start_http`] — not here.
pub fn service_http_opt_in_from_value(raw: Option<&str>) -> bool {
    raw.map(is_http_env_truthy).unwrap_or(false)
}

/// Read `AI_BRAINS_HTTP_SERVICE` and return whether service host may start HTTP.
pub fn service_http_opt_in_from_env() -> bool {
    match std::env::var("AI_BRAINS_HTTP_SERVICE") {
        Ok(v) => service_http_opt_in_from_value(Some(&v)),
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

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    #[test]
    fn is_http_env_truthy__accepts_1_true_yes() {
        assert!(is_http_env_truthy("1"));
        assert!(is_http_env_truthy(" true "));
        assert!(is_http_env_truthy("YES"));
        assert!(!is_http_env_truthy("0"));
        assert!(!is_http_env_truthy("no"));
        assert!(!is_http_env_truthy(""));
    }

    #[test]
    fn service_http_opt_in_from_value__without_opt_in__false() {
        assert!(!service_http_opt_in_from_value(None));
        assert!(!service_http_opt_in_from_value(Some("")));
        assert!(!service_http_opt_in_from_value(Some("0")));
        assert!(!service_http_opt_in_from_value(Some("no")));
    }

    #[test]
    fn service_http_opt_in_from_value__with_opt_in__true() {
        assert!(service_http_opt_in_from_value(Some("1")));
        assert!(service_http_opt_in_from_value(Some("true")));
        assert!(service_http_opt_in_from_value(Some("Yes")));
    }

    #[test]
    fn service_http_opt_in_from_env__matches_truthy_set() {
        let _clear = TempEnv::remove("AI_BRAINS_HTTP_SERVICE");
        assert!(!service_http_opt_in_from_env());
        {
            let _g = TempEnv::set("AI_BRAINS_HTTP_SERVICE", "1");
            assert!(service_http_opt_in_from_env());
        }
        {
            let _g = TempEnv::set("AI_BRAINS_HTTP_SERVICE", "true");
            assert!(service_http_opt_in_from_env());
        }
        {
            let _g = TempEnv::set("AI_BRAINS_HTTP_SERVICE", "no");
            assert!(!service_http_opt_in_from_env());
        }
    }
}
