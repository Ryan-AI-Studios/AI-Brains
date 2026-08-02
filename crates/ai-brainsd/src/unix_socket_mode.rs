//! Unix domain socket post-bind mode hardening (T184 F-2) and pre-bind/shutdown
//! ownership hygiene (T195 F8).
//!
//! Default path resolution lives in `ai_brains_daemon_api::resolve_daemon_socket_path`
//! (XDG → `/tmp` fallback). After bind, mode must be owner-only (`0o600`).

/// Normative post-bind permission bits for the bridge UDS.
pub const UDS_OWNER_ONLY_MODE: u32 = 0o600;

/// `S_IFMT` file type mask (stat mode).
#[cfg(unix)]
const S_IFMT: u32 = 0o170000;

/// `S_IFSOCK` socket file type (stat mode).
#[cfg(unix)]
const S_IFSOCK: u32 = 0o140000;

/// Apply owner-only mode to an existing socket path.
///
/// Call after successful `UnixListener::bind`. Returns an error if chmod fails
/// so callers can fail closed rather than leave a world-open socket.
#[cfg(unix)]
pub fn apply_owner_only_mode(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(UDS_OWNER_ONLY_MODE);
    std::fs::set_permissions(path, perms)
}

/// Remove an existing path only when it is a **socket owned by this euid** (T195 F8).
///
/// Used for pre-bind stale-socket cleanup and shutdown unlink. Does **not**:
/// - clobber regular files/directories
/// - remove sockets owned by another uid
/// - invent “dead socket” listener probes
///
/// Missing path → `Ok(())`.
#[cfg(unix)]
pub fn remove_owned_socket_if_present(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };

    let mode = meta.mode();
    if mode & S_IFMT != S_IFSOCK {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "refusing to remove non-socket path {} (mode={mode:#o}; expected socket type 0o140000). \
                 Free the path manually if safe, or set AI_BRAINS_DAEMON_SOCKET to a free absolute path.",
                path.display()
            ),
        ));
    }

    let euid = effective_uid();
    if meta.uid() != euid {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "refusing to remove socket {} owned by uid {} (this process euid is {}). \
                 Another principal owns the name; free the path or set AI_BRAINS_DAEMON_SOCKET.",
                path.display(),
                meta.uid(),
                euid
            ),
        ));
    }

    std::fs::remove_file(path)
}

/// Effective UID without a direct `libc`/`nix` crate dep (T195 F14).
#[cfg(unix)]
fn effective_uid() -> u32 {
    // SAFETY: `geteuid` is always available on Unix libc and has no preconditions.
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    unsafe { geteuid() }
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn uds_owner_only_mode__is_0o600() {
        assert_eq!(UDS_OWNER_ONLY_MODE, 0o600);
        // Owner read+write only; no group/other bits.
        assert_eq!(UDS_OWNER_ONLY_MODE & 0o077, 0);
        assert_eq!(UDS_OWNER_ONLY_MODE & 0o600, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn apply_owner_only_mode__temp_file__mode_is_0o600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ledgerful-bridge.sock");
        std::fs::File::create(&path).expect("create");
        // Start from a loose mode to prove we tighten.
        let loose = std::fs::Permissions::from_mode(0o666);
        std::fs::set_permissions(&path, loose).expect("chmod loose");

        apply_owner_only_mode(&path).expect("apply owner only");

        let meta = std::fs::metadata(&path).expect("meta");
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "expected 0o600 after apply, got {mode:o}");
    }

    #[cfg(unix)]
    #[test]
    fn remove_owned_socket_if_present__missing__ok() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("no-such.sock");
        remove_owned_socket_if_present(&path).expect("missing is ok");
    }

    #[cfg(unix)]
    #[test]
    fn remove_owned_socket_if_present__regular_file__fail_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("not-a-socket");
        std::fs::write(&path, b"regular").expect("write");
        let err = remove_owned_socket_if_present(&path).expect_err("must refuse regular file");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(
            err.to_string().contains("non-socket"),
            "actionable message: {err}"
        );
        assert!(path.exists(), "must not clobber regular file");
    }

    #[cfg(unix)]
    #[test]
    fn remove_owned_socket_if_present__directory__fail_closed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("subdir");
        std::fs::create_dir(&path).expect("mkdir");
        let err = remove_owned_socket_if_present(&path).expect_err("must refuse dir");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(path.is_dir(), "must not remove directory");
    }

    /// Bind a real UDS so the path is a socket owned by euid, then remove it.
    #[cfg(unix)]
    #[test]
    fn remove_owned_socket_if_present__owned_socket__removes() {
        use std::os::unix::net::UnixListener;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ledgerful-bridge.sock");
        let _listener = UnixListener::bind(&path).expect("bind uds");
        assert!(path.exists());

        remove_owned_socket_if_present(&path).expect("owned socket unlink");
        assert!(!path.exists(), "owned socket should be removed");
    }
}
