//! Tauri invoke commands — adapter surface only (no domain policy).

pub mod api;
pub mod http_client;

pub use http_client::InvokeApiError;

use serde::Serialize;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;

/// Default daemon HTTP port when `AI_BRAINS_HTTP_PORT` is unset (matches api-server).
const DEFAULT_HTTP_PORT: u16 = 7432;
const HTTP_PORT_ENV: &str = "AI_BRAINS_HTTP_PORT";
/// Optional absolute path override for the user-session token file.
///
/// Intended for tests and advanced installs. When unset, the default is
/// `%USERPROFILE%\.ai-brains\http.token` (or `$HOME/.ai-brains/http.token`).
const TOKEN_PATH_ENV: &str = "AI_BRAINS_HTTP_TOKEN_PATH";

#[cfg(test)]
static TOKEN_PATH_OVERRIDE_FOR_TESTS: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Static smoke response for host reachability (no HTTP, no vault).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PingResponse {
    pub ok: bool,
    pub service: &'static str,
    pub version: &'static str,
}

/// Honest connection metadata for the UI (E1). Never includes bearer material.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DaemonConnectionInfo {
    /// Loopback base URL when a port can be resolved; otherwise null.
    pub loopback_base_url: Option<String>,
    /// Whether the user-session token file exists (presence only).
    pub token_file_present: bool,
}

/// Build the static ping payload (pure; unit-tested).
pub fn ping_payload() -> PingResponse {
    PingResponse {
        ok: true,
        service: "ai-brains-desktop",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Test-only hook: force `user_session_token_path` to a fixed path (or clear).
///
/// Prefer writing a real temp token file and pointing here rather than
/// mutating the developer's home directory.
#[cfg(test)]
pub fn set_token_path_override_for_tests(path: Option<PathBuf>) {
    if let Ok(mut guard) = TOKEN_PATH_OVERRIDE_FOR_TESTS.lock() {
        *guard = path;
    }
}

/// Resolve user-session token path.
///
/// Order:
/// 1. `#[cfg(test)]` override via [`set_token_path_override_for_tests`]
/// 2. `AI_BRAINS_HTTP_TOKEN_PATH` env (absolute path; testability / advanced)
/// 3. `%USERPROFILE%\.ai-brains\http.token` (or `$HOME/...`)
pub fn user_session_token_path() -> Option<PathBuf> {
    #[cfg(test)]
    {
        if let Ok(guard) = TOKEN_PATH_OVERRIDE_FOR_TESTS.lock()
            && let Some(ref p) = *guard
        {
            return Some(p.clone());
        }
    }

    if let Ok(raw) = std::env::var(TOKEN_PATH_ENV) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    dirs::home_dir().map(|home| home.join(".ai-brains").join("http.token"))
}

/// Resolve loopback base URL from env/default port (no network I/O).
pub fn resolve_loopback_base_url() -> Option<String> {
    let port = match std::env::var(HTTP_PORT_ENV) {
        Ok(raw) => match raw.trim().parse::<u16>() {
            Ok(p) if p > 0 => p,
            _ => return None,
        },
        Err(_) => DEFAULT_HTTP_PORT,
    };
    Some(format!("http://127.0.0.1:{port}"))
}

/// Presence-only check for the user-session token file.
pub fn token_file_present() -> bool {
    user_session_token_path()
        .map(|p| p.is_file())
        .unwrap_or(false)
}

pub fn daemon_connection_info_payload() -> DaemonConnectionInfo {
    DaemonConnectionInfo {
        loopback_base_url: resolve_loopback_base_url(),
        token_file_present: token_file_present(),
    }
}

#[tauri::command]
pub fn ping() -> PingResponse {
    ping_payload()
}

#[tauri::command]
pub fn get_daemon_connection_info() -> DaemonConnectionInfo {
    daemon_connection_info_payload()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn ping__returns_ok_shape() {
        let p = ping_payload();
        assert!(p.ok);
        assert_eq!(p.service, "ai-brains-desktop");
        assert_eq!(p.version, env!("CARGO_PKG_VERSION"));
        assert!(!p.version.is_empty());
    }

    #[test]
    fn resolve_loopback_base_url__default_port__http_localhost() {
        // When env is unset (typical unit test process), expect default port.
        // If CI sets AI_BRAINS_HTTP_PORT, still assert shape when Some.
        match resolve_loopback_base_url() {
            Some(url) => {
                assert!(
                    url.starts_with("http://127.0.0.1:"),
                    "unexpected base url: {url}"
                );
            }
            None => {
                // Only when env is present but invalid.
                let raw = std::env::var(HTTP_PORT_ENV).unwrap_or_default();
                assert!(
                    !raw.is_empty(),
                    "None only expected when AI_BRAINS_HTTP_PORT is invalid"
                );
            }
        }
    }
}
