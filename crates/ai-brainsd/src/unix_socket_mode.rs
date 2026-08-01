//! Unix domain socket post-bind mode hardening (T184 F-2).
//!
//! Path remains `/tmp/ledgerful-bridge.sock` for ledgerful IpcClient interop.
//! After bind, mode must be owner-only (`0o600`).

/// Normative post-bind permission bits for the bridge UDS.
pub const UDS_OWNER_ONLY_MODE: u32 = 0o600;

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

#[cfg(test)]
#[allow(non_snake_case)]
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
}
