//! T242 — cross-process env-override warn marker IO (injectable home root).
//!
//! Pure fingerprint / decide policy live in `env_warn`. This module only claims
//! 0-byte marker files under `{home}/.ai-brains/cache/env-override-warn/<hex>`.

use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Result of an atomic `create_new` claim for a session-quiet marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerClaim {
    /// Created empty marker — first warn for this fingerprint.
    Claimed,
    /// Marker already existed — suppress stderr (demote to Debug).
    Exists,
    /// create_dir_all / open failed for a reason other than AlreadyExists — fail-open warn.
    IoFail,
}

/// `{home}/.ai-brains/cache/env-override-warn/<hex>`.
pub fn marker_path(home: &Path, hex: &str) -> PathBuf {
    home.join(".ai-brains")
        .join("cache")
        .join("env-override-warn")
        .join(hex)
}

/// Best-effort `create_dir_all` then atomic `OpenOptions::create_new(true)`.
///
/// - `Ok` → `Claimed` (file is empty / 0 bytes)
/// - `AlreadyExists` → `Exists`
/// - other IO → `IoFail` (caller fail-opens Stderr)
pub fn try_claim_marker(home: &Path, hex: &str) -> MarkerClaim {
    let path = marker_path(home, hex);
    if let Some(parent) = path.parent()
        && let Err(_) = std::fs::create_dir_all(parent)
    {
        // Directory missing and uncreatable — open will fail → IoFail.
    }
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(_file) => MarkerClaim::Claimed,
        Err(err) if err.kind() == ErrorKind::AlreadyExists => MarkerClaim::Exists,
        Err(_) => MarkerClaim::IoFail,
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // unit tests may expect/unwrap
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn try_claim_marker__first_claim__claimed_and_empty_file() {
        let dir = tempfile::tempdir().expect("temp home");
        let home = dir.path();
        let hex = "a".repeat(64);
        assert_eq!(try_claim_marker(home, &hex), MarkerClaim::Claimed);
        let path = marker_path(home, &hex);
        assert!(path.is_file(), "marker must exist after claim");
        let meta = std::fs::metadata(&path).expect("metadata");
        assert_eq!(meta.len(), 0, "marker content must be 0 bytes");
    }

    #[test]
    #[allow(non_snake_case)]
    fn try_claim_marker__second_claim__exists() {
        let dir = tempfile::tempdir().expect("temp home");
        let home = dir.path();
        let hex = "b".repeat(64);
        assert_eq!(try_claim_marker(home, &hex), MarkerClaim::Claimed);
        assert_eq!(try_claim_marker(home, &hex), MarkerClaim::Exists);
    }

    #[test]
    #[allow(non_snake_case)]
    fn marker_path__under_ai_brains_cache() {
        // Platform-agnostic: Path::join uses OS separators (Linux CI must not hardcode `\`).
        let home = Path::new("home-root");
        let path = marker_path(home, "deadbeef");
        assert_eq!(
            path,
            PathBuf::from("home-root")
                .join(".ai-brains")
                .join("cache")
                .join("env-override-warn")
                .join("deadbeef")
        );
        assert!(
            path.ends_with(Path::new("env-override-warn").join("deadbeef")),
            "marker leaf must be env-override-warn/<hex>; got {}",
            path.display()
        );
    }

    /// AC8: when the home path cannot host a marker dir (e.g. home is a file),
    /// claim returns IoFail so callers fail-open to Stderr.
    #[test]
    #[allow(non_snake_case)]
    fn try_claim_marker__unwritable_home_file__io_fail() {
        let dir = tempfile::tempdir().expect("temp");
        // Use a regular file as "home" so create_dir_all(home/.ai-brains/...) fails.
        let home_as_file = dir.path().join("not-a-dir");
        std::fs::write(&home_as_file, b"x").expect("write file as home");
        let hex = "c".repeat(64);
        assert_eq!(
            try_claim_marker(&home_as_file, &hex),
            MarkerClaim::IoFail,
            "AC8: unwritable home must IoFail (fail-open at call site)"
        );
    }
}
