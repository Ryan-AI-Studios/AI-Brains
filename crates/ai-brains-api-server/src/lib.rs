//! Authenticated loopback HTTP adapter for `ai-brainsd` (T161 / P7.4).
//!
//! # Design locks
//!
//! - **In-process** with the daemon: this crate is a library; `ai-brainsd` injects
//!   an [`HttpDispatch`] implementation that wraps `handle_daemon_request`.
//! - **No domain logic** in handlers — path + JSON → `DaemonRequest` → dispatch.
//! - **Loopback bind** by default; non-loopback requires double opt-in.
//! - **Bearer auth** on data routes; health is unauthenticated liveness only.
//! - **CORS deny-by-default** (no permissive layer); **1 MiB** body limit.
//! - **Token file** owner-only SDDL `D:P(A;;FA;;;OW)` on Windows.
//!
//! # Capture independence
//!
//! This crate MUST NOT become a dependency of `ai-brains-capture`.

pub mod auth;
pub mod bind;
pub mod dispatch;
pub mod error;
pub mod routes;
pub mod token;

pub use auth::{AuthConfig, token_bytes_equal, tokens_equal};
pub use bind::{
    BindError, DEFAULT_HTTP_PORT, default_http_port, is_loopback_addr, resolve_bind_addr,
};
pub use dispatch::{DispatchError, HttpDispatch};
pub use error::{ApiHttpError, http_status_for_api_error_code, map_daemon_response};
pub use routes::{AppState, BODY_LIMIT_BYTES, build_router};
pub use token::{
    USER_TOKEN_FILE_SDDL, default_token_path, ensure_token, generate_token, load_or_create_token,
};

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use zeroize::Zeroizing;

/// Serve the HTTP API until `shutdown` completes.
///
/// Caller owns token loading and bind resolution. Prefer
/// [`resolve_bind_addr`] before binding.
pub async fn serve<F>(
    listener: TcpListener,
    state: AppState,
    shutdown: F,
) -> Result<(), std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let app = build_router(state);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Convenience: bind `addr` (must already pass policy) and serve.
pub async fn serve_on<F>(
    addr: SocketAddr,
    state: AppState,
    shutdown: F,
) -> Result<SocketAddr, std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    serve(listener, state, shutdown).await?;
    Ok(local)
}

/// Build application state from a dispatch port and bearer token.
///
/// Takes `Zeroizing<String>` so callers that already hold a zeroizing token
/// (e.g. `ensure_token`) can move it without a plain-`String` intermediate.
/// Stored as `Arc<Zeroizing<String>>` so FromRef clones only the Arc and the
/// secret is zeroized on final drop (CR1-P2-02).
pub fn app_state(dispatch: Arc<dyn HttpDispatch>, bearer_token: Zeroizing<String>) -> AppState {
    AppState {
        dispatch,
        auth: AuthConfig {
            bearer_token: Arc::new(bearer_token),
        },
    }
}
