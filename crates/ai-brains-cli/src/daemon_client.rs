use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Resolve UDS path for CLI connect (T195 F7). Fail closed on invalid env override.
#[cfg(not(windows))]
fn resolve_unix_socket_path_for_client() -> String {
    match ai_brains_daemon_api::resolve_daemon_socket_path() {
        Ok(resolved) => {
            if resolved.used_tmp_fallback {
                // Match daemon warn: residual when XDG unset/invalid (common on macOS).
                eprintln!(
                    "warning: XDG_RUNTIME_DIR missing or invalid; using {} for daemon UDS \
                     (set AI_BRAINS_DAEMON_SOCKET or a valid XDG_RUNTIME_DIR to match the daemon)",
                    resolved.path.display()
                );
            }
            resolved.path.display().to_string()
        }
        Err(e) => {
            // Fail closed: do not silently fall back to /tmp when override is invalid.
            eprintln!(
                "error: {e}; daemon client refusing guessed path (fix AI_BRAINS_DAEMON_SOCKET)"
            );
            String::new()
        }
    }
}

/// Default bound for a full request/response cycle on the governed surface.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors from [`DaemonClient::request`] / probe transport.
#[derive(Debug)]
pub enum DaemonClientError {
    /// Named pipe / socket is not accepting connections (daemon not running).
    NotRunning(String),
    /// Connect or write failed before the request body was fully sent.
    Transport { message: String, request_sent: bool },
    /// Timeout. When `request_sent` is true the outcome is **ambiguous**
    /// (daemon may have applied the mutation).
    Timeout { request_sent: bool },
    /// Response bytes were not valid line-delimited DaemonResponse JSON.
    Protocol(String),
}

impl std::fmt::Display for DaemonClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRunning(m) => write!(f, "daemon not running: {m}"),
            Self::Transport {
                message,
                request_sent,
            } => {
                if *request_sent {
                    write!(
                        f,
                        "daemon transport error after send (outcome may be unknown): {message}"
                    )
                } else {
                    write!(f, "daemon transport error: {message}")
                }
            }
            Self::Timeout { request_sent } => {
                if *request_sent {
                    write!(
                        f,
                        "daemon timeout after request was sent (outcome unknown; retry same --command-id on daemon)"
                    )
                } else {
                    write!(f, "daemon timeout before request could be sent")
                }
            }
            Self::Protocol(m) => write!(f, "daemon protocol error: {m}"),
        }
    }
}

impl std::error::Error for DaemonClientError {}

impl DaemonClientError {
    /// True when the request body was fully written and the outcome is unknown.
    pub fn is_ambiguous(&self) -> bool {
        match self {
            Self::Timeout { request_sent } => *request_sent,
            Self::Transport { request_sent, .. } => *request_sent,
            Self::NotRunning(_) | Self::Protocol(_) => false,
        }
    }

    /// True when the daemon was unreachable before any send (safe for local fallback).
    pub fn is_pre_send_unavailable(&self) -> bool {
        match self {
            Self::NotRunning(_) => true,
            Self::Timeout { request_sent } => !*request_sent,
            Self::Transport { request_sent, .. } => !*request_sent,
            Self::Protocol(_) => false,
        }
    }
}

/// Live Windows named-pipe endpoint (must match ledgerful IpcClient / track 0064).
#[cfg(windows)]
pub const DEFAULT_DAEMON_TRANSPORT_PATH: &str = r"\\.\pipe\ledgerful-bridge";

/// Documented Unix UDS fallback when XDG is missing/invalid (T195 F9).
/// Runtime path is resolved via [`ai_brains_daemon_api::resolve_daemon_socket_path`].
#[cfg(not(windows))]
pub const DEFAULT_DAEMON_TRANSPORT_PATH: &str = ai_brains_daemon_api::FALLBACK_DAEMON_SOCKET_PATH;

pub struct DaemonClient {
    #[cfg(windows)]
    pipe_path: String,
    #[cfg(not(windows))]
    socket_path: String,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            // Must match ledgerful's IpcClient (track 0064: aibrains-sync → ledgerful-bridge).
            #[cfg(windows)]
            pipe_path: DEFAULT_DAEMON_TRANSPORT_PATH.to_string(),
            // T195 F7/F32: same resolver as ai-brainsd Unix bind (not hardcoded /tmp only).
            #[cfg(not(windows))]
            socket_path: resolve_unix_socket_path_for_client(),
        }
    }

    /// Live local transport endpoint (named pipe path on Windows, UDS path on Unix).
    /// Portable multi-OS product IPC remains loopback HTTP + bearer (T161 / F23).
    pub fn transport_path(&self) -> &str {
        #[cfg(windows)]
        {
            &self.pipe_path
        }
        #[cfg(not(windows))]
        {
            &self.socket_path
        }
    }

    pub fn spawn_daemon(
        &self,
        vault_path: &std::path::Path,
        key: &ai_brains_crypto::SqlCipherKey,
    ) -> std::io::Result<()> {
        let exe_path = std::env::current_exe()?;
        let daemon_name = if cfg!(windows) {
            "ai-brainsd.exe"
        } else {
            "ai-brainsd"
        };
        let mut daemon_path = exe_path
            .parent()
            .ok_or_else(|| std::io::Error::other("Failed to get executable parent dir"))?
            .to_path_buf();
        daemon_path.push(daemon_name);

        let mut cmd = if daemon_path.exists() {
            std::process::Command::new(daemon_path)
        } else {
            std::process::Command::new(daemon_name)
        };

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;
            cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
        }

        cmd.env("AI_BRAINS_VAULT_PATH", vault_path)
            .env("AI_BRAINS_KEY", key.expose_secret())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        Ok(())
    }

    pub async fn ensure_running(
        &self,
        vault_path: &std::path::Path,
        key: &ai_brains_crypto::SqlCipherKey,
    ) -> bool {
        if self.probe(Duration::from_millis(10)).await {
            return true;
        }

        let jitter = (std::process::id() % 50) as u64;
        tokio::time::sleep(Duration::from_millis(10 + jitter)).await;
        if self.probe(Duration::from_millis(10)).await {
            return true;
        }

        if self.spawn_daemon(vault_path, key).is_ok() {
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_millis(50)).await;
                if self.probe(Duration::from_millis(10)).await {
                    return true;
                }
            }
        }

        false
    }

    pub async fn probe(&self, timeout: Duration) -> bool {
        matches!(
            self.request_with_timeout(DaemonRequest::Ping, timeout)
                .await,
            Ok(DaemonResponse::Pong)
        )
    }

    /// Send a full line-delimited [`DaemonRequest`] and await one [`DaemonResponse`].
    pub async fn request(&self, req: DaemonRequest) -> Result<DaemonResponse, DaemonClientError> {
        self.request_with_timeout(req, DEFAULT_REQUEST_TIMEOUT)
            .await
    }

    /// Like [`Self::request`] with an explicit timeout bound.
    pub async fn request_with_timeout(
        &self,
        req: DaemonRequest,
        timeout: Duration,
    ) -> Result<DaemonResponse, DaemonClientError> {
        let mut payload = serde_json::to_vec(&req)
            .map_err(|e| DaemonClientError::Protocol(format!("serialize request failed: {e}")))?;
        payload.push(b'\n');

        #[cfg(windows)]
        {
            self.request_windows(&payload, timeout).await
        }

        #[cfg(not(windows))]
        {
            self.request_unix(&payload, timeout).await
        }
    }

    #[cfg(windows)]
    async fn request_windows(
        &self,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<DaemonResponse, DaemonClientError> {
        use tokio::net::windows::named_pipe::ClientOptions;
        use tokio::time::timeout as tokio_timeout;

        let mut stream = match ClientOptions::new().open(&self.pipe_path) {
            Ok(s) => s,
            Err(e) => {
                return Err(DaemonClientError::NotRunning(format!(
                    "open pipe {}: {e}",
                    self.pipe_path
                )));
            }
        };

        match tokio_timeout(timeout, stream.write_all(payload)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                return Err(DaemonClientError::Transport {
                    message: format!("write failed: {e}"),
                    request_sent: false,
                });
            }
            Err(_) => {
                return Err(DaemonClientError::Timeout {
                    request_sent: false,
                });
            }
        }

        let mut buffer = Vec::with_capacity(8192);
        let mut chunk = [0u8; 8192];
        let read_deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = read_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(DaemonClientError::Timeout { request_sent: true });
            }
            match tokio_timeout(remaining, stream.read(&mut chunk)).await {
                Ok(Ok(0)) => {
                    if buffer.is_empty() {
                        return Err(DaemonClientError::Transport {
                            message: "connection closed before response".into(),
                            request_sent: true,
                        });
                    }
                    break;
                }
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if buffer.contains(&b'\n') || looks_like_complete_json(&buffer) {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    return Err(DaemonClientError::Transport {
                        message: format!("read failed: {e}"),
                        request_sent: true,
                    });
                }
                Err(_) => {
                    return Err(DaemonClientError::Timeout { request_sent: true });
                }
            }
        }

        parse_daemon_response_line(&buffer)
    }

    #[cfg(not(windows))]
    async fn request_unix(
        &self,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<DaemonResponse, DaemonClientError> {
        use tokio::net::UnixStream;
        use tokio::time::timeout as tokio_timeout;

        let mut stream = match tokio_timeout(timeout, UnixStream::connect(&self.socket_path)).await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                return Err(DaemonClientError::NotRunning(format!(
                    "connect {}: {e}",
                    self.socket_path
                )));
            }
            Err(_) => {
                return Err(DaemonClientError::Timeout {
                    request_sent: false,
                });
            }
        };

        match tokio_timeout(timeout, stream.write_all(payload)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                return Err(DaemonClientError::Transport {
                    message: format!("write failed: {e}"),
                    request_sent: false,
                });
            }
            Err(_) => {
                return Err(DaemonClientError::Timeout {
                    request_sent: false,
                });
            }
        }

        let mut buffer = Vec::with_capacity(8192);
        let mut chunk = [0u8; 8192];
        let read_deadline = tokio::time::Instant::now() + timeout;
        loop {
            let remaining = read_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(DaemonClientError::Timeout { request_sent: true });
            }
            match tokio_timeout(remaining, stream.read(&mut chunk)).await {
                Ok(Ok(0)) => {
                    if buffer.is_empty() {
                        return Err(DaemonClientError::Transport {
                            message: "connection closed before response".into(),
                            request_sent: true,
                        });
                    }
                    break;
                }
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if buffer.contains(&b'\n') || looks_like_complete_json(&buffer) {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    return Err(DaemonClientError::Transport {
                        message: format!("read failed: {e}"),
                        request_sent: true,
                    });
                }
                Err(_) => {
                    return Err(DaemonClientError::Timeout { request_sent: true });
                }
            }
        }

        parse_daemon_response_line(&buffer)
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.request(DaemonRequest::Shutdown).await {
            Ok(_) => Ok(()),
            Err(DaemonClientError::Transport { .. })
            | Err(DaemonClientError::Timeout { .. })
            | Err(DaemonClientError::NotRunning(_)) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DaemonClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonClient")
            .field("transport_path", &self.transport_path())
            .finish()
    }
}

fn looks_like_complete_json(buf: &[u8]) -> bool {
    let s = std::str::from_utf8(buf).unwrap_or("").trim();
    if s.is_empty() {
        return false;
    }
    let open = s.chars().filter(|c| *c == '{').count();
    let close = s.chars().filter(|c| *c == '}').count();
    open > 0 && open == close && s.starts_with('{')
}

fn parse_daemon_response_line(buffer: &[u8]) -> Result<DaemonResponse, DaemonClientError> {
    let line = match buffer.iter().position(|&b| b == b'\n') {
        Some(i) => &buffer[..i],
        None => buffer,
    };
    let trimmed = line
        .iter()
        .copied()
        .skip_while(|b| b.is_ascii_whitespace())
        .collect::<Vec<u8>>();
    if trimmed.is_empty() {
        return Err(DaemonClientError::Protocol(
            "empty response from daemon".into(),
        ));
    }
    serde_json::from_slice::<DaemonResponse>(&trimmed).map_err(|e| {
        DaemonClientError::Protocol(format!(
            "invalid DaemonResponse JSON: {e}; body={}",
            String::from_utf8_lossy(&trimmed)
        ))
    })
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn daemon_client_error__timeout_after_send__is_ambiguous() {
        let err = DaemonClientError::Timeout { request_sent: true };
        assert!(err.is_ambiguous());
        assert!(!err.is_pre_send_unavailable());
    }

    #[test]
    fn daemon_client_error__not_running__pre_send() {
        let err = DaemonClientError::NotRunning("pipe closed".into());
        assert!(!err.is_ambiguous());
        assert!(err.is_pre_send_unavailable());
    }

    #[test]
    fn parse_daemon_response_line__pong() {
        let raw = br#"{"type":"pong"}"#;
        let resp = parse_daemon_response_line(raw).expect("parse");
        assert!(matches!(resp, DaemonResponse::Pong));
    }

    /// T179 F23: live DaemonClient transport is OS-native (pipe vs UDS), not HTTP.
    #[test]
    fn daemon_client__new__uses_os_native_transport_path() {
        #[cfg(windows)]
        {
            let client = DaemonClient::new();
            assert_eq!(client.transport_path(), DEFAULT_DAEMON_TRANSPORT_PATH);
            assert_eq!(client.transport_path(), r"\\.\pipe\ledgerful-bridge");
        }
        #[cfg(not(windows))]
        {
            use ai_brains_core::temp_env::TempEnv;

            let _clear_socket = TempEnv::remove("AI_BRAINS_DAEMON_SOCKET");
            let _clear_xdg = TempEnv::remove("XDG_RUNTIME_DIR");
            let client = DaemonClient::new();
            assert_eq!(
                client.transport_path(),
                DEFAULT_DAEMON_TRANSPORT_PATH,
                "without XDG, fallback is /tmp/ledgerful-bridge.sock"
            );
            assert_eq!(client.transport_path(), "/tmp/ledgerful-bridge.sock");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn daemon_client__new__absolute_socket_env__uses_override() {
        use ai_brains_core::temp_env::TempEnv;

        let abs = "/run/user/1000/custom-bridge.sock";
        let _set = TempEnv::set("AI_BRAINS_DAEMON_SOCKET", abs);
        let client = DaemonClient::new();
        assert_eq!(client.transport_path(), abs);
    }

    #[cfg(not(windows))]
    #[test]
    fn daemon_client__new__relative_socket_env__fail_closed_empty_path() {
        use ai_brains_core::temp_env::TempEnv;

        let _set = TempEnv::set("AI_BRAINS_DAEMON_SOCKET", "relative.sock");
        let client = DaemonClient::new();
        assert_eq!(
            client.transport_path(),
            "",
            "invalid override must not guess /tmp"
        );
    }
}
