//! Shared daemon transport path helpers (T195 F7/F31).
//!
//! Unix domain socket path resolution is the SOOT for both `ai-brainsd` bind and
//! `ai-brains-cli` [`DaemonClient`] connect. Windows named-pipe path remains the
//! constant `\\.\pipe\ledgerful-bridge` (not resolved here).
//!
//! Also hosts pure [`PipeAclMode`] parse for optional reuse (F3/F4/F31).

use std::path::{Path, PathBuf};

/// Socket basename under XDG runtime or `/tmp` (ledgerful IpcClient interop).
pub const DAEMON_SOCKET_FILE_NAME: &str = "ledgerful-bridge.sock";

/// Fallback UDS path when XDG is missing/invalid (T195 F9 residual).
pub const FALLBACK_DAEMON_SOCKET_PATH: &str = "/tmp/ledgerful-bridge.sock";

/// Absolute path override for daemon UDS (bind + connect).
pub const ENV_DAEMON_SOCKET: &str = "AI_BRAINS_DAEMON_SOCKET";

/// Optional pipe ACL mode env (Windows named pipe).
pub const ENV_PIPE_ACL: &str = "AI_BRAINS_PIPE_ACL";

/// Default SDDL: SYSTEM + Administrators + Interactive (T184 F-1 / T195 F3).
pub const PIPE_SDDL_INTERACTIVE: &str = "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)";

/// Opt-in tighter SDDL without Interactive (T195 F4).
pub const PIPE_SDDL_SERVICE_ONLY: &str = "D:(A;;GA;;;SY)(A;;GA;;;BA)";

/// Resolved Unix domain socket path for daemon IPC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDaemonSocket {
    /// Absolute path to bind/connect.
    pub path: PathBuf,
    /// `true` when falling back to [`FALLBACK_DAEMON_SOCKET_PATH`] (warn at runtime).
    pub used_tmp_fallback: bool,
}

/// Fail-closed errors from [`resolve_daemon_socket_path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveDaemonSocketError {
    /// `AI_BRAINS_DAEMON_SOCKET` was set but is not an absolute path.
    RelativeOverride { value: String },
}

impl std::fmt::Display for ResolveDaemonSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RelativeOverride { value } => write!(
                f,
                "AI_BRAINS_DAEMON_SOCKET must be an absolute path (got {value:?}); \
                 refusing relative override (fail closed). Example: \
                 /run/user/$UID/ledgerful-bridge.sock or /tmp/ledgerful-bridge.sock"
            ),
        }
    }
}

impl std::error::Error for ResolveDaemonSocketError {}

/// Resolve the Unix domain socket path for daemon bind and CLI connect.
///
/// Order (T195 F7 / F30 / F31):
/// 1. If `AI_BRAINS_DAEMON_SOCKET` is set → must be absolute or fail closed.
/// 2. Else read `XDG_RUNTIME_DIR` via `std::env::var` only (not `dirs::runtime_dir`).
/// 3. Validate XDG dir (absolute; metadata ok; mode `0700`; uid == euid). Do **not** create it.
/// 4. If valid → `$XDG_RUNTIME_DIR/ledgerful-bridge.sock`.
/// 5. Else → `/tmp/ledgerful-bridge.sock` with [`ResolvedDaemonSocket::used_tmp_fallback`] = true.
///
/// Windows product callers keep the named-pipe constant; this helper is for Unix UDS.
pub fn resolve_daemon_socket_path() -> Result<ResolvedDaemonSocket, ResolveDaemonSocketError> {
    if let Ok(raw) = std::env::var(ENV_DAEMON_SOCKET) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ResolveDaemonSocketError::RelativeOverride { value: raw });
        }
        let path = PathBuf::from(trimmed);
        if !path.is_absolute() {
            return Err(ResolveDaemonSocketError::RelativeOverride {
                value: trimmed.to_string(),
            });
        }
        return Ok(ResolvedDaemonSocket {
            path,
            used_tmp_fallback: false,
        });
    }

    if let Some(path) = try_xdg_runtime_socket_path() {
        return Ok(ResolvedDaemonSocket {
            path,
            used_tmp_fallback: false,
        });
    }

    Ok(ResolvedDaemonSocket {
        path: PathBuf::from(FALLBACK_DAEMON_SOCKET_PATH),
        used_tmp_fallback: true,
    })
}

fn try_xdg_runtime_socket_path() -> Option<PathBuf> {
    let raw = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let dir = Path::new(trimmed);
    if !dir.is_absolute() {
        return None;
    }
    if !is_valid_xdg_runtime_dir(dir) {
        return None;
    }
    Some(dir.join(DAEMON_SOCKET_FILE_NAME))
}

/// F30: absolute; metadata ok; `(mode & 0o777) == 0o700`; uid == euid. Never creates the dir.
#[cfg(unix)]
fn is_valid_xdg_runtime_dir(dir: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::metadata(dir) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_dir() {
        return false;
    }
    if (meta.mode() & 0o777) != 0o700 {
        return false;
    }
    if meta.uid() != effective_uid() {
        return false;
    }
    true
}

/// Non-Unix: UDS path product path is not used; never accept XDG for resolution.
#[cfg(not(unix))]
fn is_valid_xdg_runtime_dir(_dir: &Path) -> bool {
    false
}

/// Effective UID without a direct `libc`/`nix` crate dep (T195 F14).
#[cfg(unix)]
fn effective_uid() -> u32 {
    // SAFETY: `geteuid` is always available on Unix libc, has no preconditions,
    // and is already linked via std. Avoids adding a production `libc` dep.
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    unsafe { geteuid() }
}

// --- Pipe ACL mode (T195 F3/F4) ---

/// Windows named-pipe ACL mode selected by `AI_BRAINS_PIPE_ACL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeAclMode {
    /// Default: SY+BA+IU (service ↔ interactive CLI).
    Interactive,
    /// Opt-in: SY+BA only (no Interactive) — interactive CLI cannot open SYSTEM pipe.
    ServiceOnly,
}

/// Unknown `AI_BRAINS_PIPE_ACL` value (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipeAclModeError {
    pub value: String,
}

impl std::fmt::Display for PipeAclModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown AI_BRAINS_PIPE_ACL={:?}; expected 'interactive' (default) or 'service-only'",
            self.value
        )
    }
}

impl std::error::Error for PipeAclModeError {}

/// Parse pipe ACL mode from optional env-style string (unset → Interactive).
pub fn parse_pipe_acl_mode(raw: Option<&str>) -> Result<PipeAclMode, PipeAclModeError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(PipeAclMode::Interactive),
        Some(s) if s.eq_ignore_ascii_case("interactive") => Ok(PipeAclMode::Interactive),
        Some(s) if s.eq_ignore_ascii_case("service-only") => Ok(PipeAclMode::ServiceOnly),
        Some(other) => Err(PipeAclModeError {
            value: other.to_string(),
        }),
    }
}

/// Read and parse `AI_BRAINS_PIPE_ACL` from the process environment.
pub fn pipe_acl_mode_from_env() -> Result<PipeAclMode, PipeAclModeError> {
    match std::env::var(ENV_PIPE_ACL) {
        Ok(v) => parse_pipe_acl_mode(Some(&v)),
        Err(_) => Ok(PipeAclMode::Interactive),
    }
}

/// SDDL string for a pipe ACL mode (Windows `ConvertStringSecurityDescriptor*`).
pub fn sddl_for_pipe_acl_mode(mode: PipeAclMode) -> &'static str {
    match mode {
        PipeAclMode::Interactive => PIPE_SDDL_INTERACTIVE,
        PipeAclMode::ServiceOnly => PIPE_SDDL_SERVICE_ONLY,
    }
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use std::path::PathBuf;

    #[test]
    fn resolve_daemon_socket_path__absolute_env_override__uses_path() {
        let _clear_xdg = TempEnv::remove("XDG_RUNTIME_DIR");
        let abs = if cfg!(windows) {
            r"C:\tmp\custom-bridge.sock"
        } else {
            "/var/run/custom-bridge.sock"
        };
        let _set = TempEnv::set(ENV_DAEMON_SOCKET, abs);
        let resolved = resolve_daemon_socket_path().expect("absolute override");
        assert_eq!(resolved.path, PathBuf::from(abs));
        assert!(!resolved.used_tmp_fallback);
    }

    #[test]
    fn resolve_daemon_socket_path__relative_env__fail_closed() {
        let _set = TempEnv::set(ENV_DAEMON_SOCKET, "relative/ledgerful-bridge.sock");
        let err = resolve_daemon_socket_path().expect_err("relative must fail");
        match err {
            ResolveDaemonSocketError::RelativeOverride { value } => {
                assert!(value.contains("relative"));
            }
        }
    }

    #[test]
    fn resolve_daemon_socket_path__empty_env__fail_closed() {
        let _set = TempEnv::set(ENV_DAEMON_SOCKET, "   ");
        let err = resolve_daemon_socket_path().expect_err("empty override must fail");
        assert!(matches!(
            err,
            ResolveDaemonSocketError::RelativeOverride { .. }
        ));
    }

    #[test]
    fn resolve_daemon_socket_path__missing_xdg__falls_back_tmp() {
        let _clear_socket = TempEnv::remove(ENV_DAEMON_SOCKET);
        let _clear_xdg = TempEnv::remove("XDG_RUNTIME_DIR");
        let resolved = resolve_daemon_socket_path().expect("fallback");
        assert_eq!(resolved.path, PathBuf::from(FALLBACK_DAEMON_SOCKET_PATH));
        assert!(resolved.used_tmp_fallback);
    }

    #[test]
    fn resolve_daemon_socket_path__relative_xdg__falls_back_tmp() {
        let _clear_socket = TempEnv::remove(ENV_DAEMON_SOCKET);
        let _set_xdg = TempEnv::set("XDG_RUNTIME_DIR", "relative-runtime");
        let resolved = resolve_daemon_socket_path().expect("relative xdg → fallback");
        assert!(resolved.used_tmp_fallback);
        assert_eq!(resolved.path, PathBuf::from(FALLBACK_DAEMON_SOCKET_PATH));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_daemon_socket_path__valid_xdg__path_under_xdg() {
        use std::os::unix::fs::PermissionsExt;

        let _clear_socket = TempEnv::remove(ENV_DAEMON_SOCKET);
        let dir = tempfile::tempdir().expect("tempdir");
        // tempdir is often 0700 already; force 0700 for F30.
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(dir.path(), perms).expect("chmod 0700");

        let _set_xdg = TempEnv::set(
            "XDG_RUNTIME_DIR",
            dir.path().to_str().expect("utf8 temp path"),
        );
        let resolved = resolve_daemon_socket_path().expect("valid xdg");
        assert!(!resolved.used_tmp_fallback);
        assert_eq!(resolved.path, dir.path().join(DAEMON_SOCKET_FILE_NAME));
        assert!(
            resolved.path.starts_with(dir.path()),
            "path must be under XDG_RUNTIME_DIR"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolve_daemon_socket_path__xdg_mode_not_0700__falls_back_tmp() {
        use std::os::unix::fs::PermissionsExt;

        let _clear_socket = TempEnv::remove(ENV_DAEMON_SOCKET);
        let dir = tempfile::tempdir().expect("tempdir");
        let loose = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(dir.path(), loose).expect("chmod 0755");

        let _set_xdg = TempEnv::set(
            "XDG_RUNTIME_DIR",
            dir.path().to_str().expect("utf8 temp path"),
        );
        let resolved = resolve_daemon_socket_path().expect("invalid mode → fallback");
        assert!(resolved.used_tmp_fallback);
        assert_eq!(resolved.path, PathBuf::from(FALLBACK_DAEMON_SOCKET_PATH));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_daemon_socket_path__xdg_missing_dir__falls_back_tmp() {
        let _clear_socket = TempEnv::remove(ENV_DAEMON_SOCKET);
        let _set_xdg = TempEnv::set(
            "XDG_RUNTIME_DIR",
            "/tmp/ai-brains-xdg-does-not-exist-t195-xyzzy",
        );
        let resolved = resolve_daemon_socket_path().expect("missing xdg dir → fallback");
        assert!(resolved.used_tmp_fallback);
    }

    #[test]
    fn parse_pipe_acl_mode__unset_and_interactive__default() {
        assert_eq!(
            parse_pipe_acl_mode(None).expect("unset"),
            PipeAclMode::Interactive
        );
        assert_eq!(
            parse_pipe_acl_mode(Some("interactive")).expect("interactive"),
            PipeAclMode::Interactive
        );
        assert_eq!(
            parse_pipe_acl_mode(Some("INTERACTIVE")).expect("case"),
            PipeAclMode::Interactive
        );
        assert_eq!(
            parse_pipe_acl_mode(Some("  ")).expect("blank → default"),
            PipeAclMode::Interactive
        );
    }

    #[test]
    fn parse_pipe_acl_mode__service_only() {
        assert_eq!(
            parse_pipe_acl_mode(Some("service-only")).expect("service-only"),
            PipeAclMode::ServiceOnly
        );
        assert_eq!(
            parse_pipe_acl_mode(Some("Service-Only")).expect("case"),
            PipeAclMode::ServiceOnly
        );
    }

    #[test]
    fn parse_pipe_acl_mode__unknown__fail_closed() {
        let err = parse_pipe_acl_mode(Some("world")).expect_err("unknown");
        assert_eq!(err.value, "world");
        assert!(err.to_string().contains("service-only"));
    }

    #[test]
    fn sddl_for_pipe_acl_mode__interactive_has_iu_service_only_does_not() {
        let interactive = sddl_for_pipe_acl_mode(PipeAclMode::Interactive);
        let service_only = sddl_for_pipe_acl_mode(PipeAclMode::ServiceOnly);
        assert_eq!(interactive, PIPE_SDDL_INTERACTIVE);
        assert_eq!(service_only, PIPE_SDDL_SERVICE_ONLY);
        assert!(interactive.contains(";;;IU)"));
        assert!(!service_only.contains(";;;IU)"));
        assert!(!interactive.contains("WD"));
        assert!(!service_only.contains("WD"));
        assert!(interactive.contains(";;;SY)"));
        assert!(service_only.contains(";;;BA)"));
    }

    #[test]
    fn pipe_acl_mode_from_env__service_only() {
        let _g = TempEnv::set(ENV_PIPE_ACL, "service-only");
        assert_eq!(
            pipe_acl_mode_from_env().expect("env"),
            PipeAclMode::ServiceOnly
        );
    }
}
