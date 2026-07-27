//! Reparse-point / symlink detection shared by CLI artifact hardening and
//! path-bearing connectors (T154).
//!
//! # Residual TOCTOU
//!
//! Check-then-open without `openat` / cap-std remains racy if an attacker swaps
//! a link between detection and open. Callers should still fail closed when a
//! reparse is detected at check time.

use std::path::Path;

/// True when `path` exists and is a symlink or (on Windows) any reparse point
/// (junction, mount point, …).
///
/// Missing path → `Ok(false)`. Does not follow the reparse for the check.
pub fn is_reparse_or_symlink(path: &Path) -> std::io::Result<bool> {
    #[cfg(not(windows))]
    {
        match path.symlink_metadata() {
            Ok(meta) => Ok(meta.file_type().is_symlink()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    #[cfg(windows)]
    {
        is_reparse_or_symlink_windows(path)
    }
}

/// Pure helper: refuse when `is_reparse` is true (unit-testable without FS).
///
/// Production passes `is_reparse_or_symlink(path)?` as the second argument.
pub fn refuse_if_reparse(path: &Path, is_reparse: bool) -> Result<(), String> {
    if is_reparse {
        Err(format!(
            "refusing to write through reparse point/symlink/junction at {}",
            path.display()
        ))
    } else {
        Ok(())
    }
}

// --- Windows implementation ---

#[cfg(windows)]
fn is_reparse_or_symlink_windows(path: &Path) -> std::io::Result<bool> {
    match path.symlink_metadata() {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Ok(true);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    }

    // Also detect directory junctions / other reparse points that may not
    // report as is_symlink() on all Rust/Windows combinations.
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, GetFileAttributesW, INVALID_FILE_ATTRIBUTES,
    };
    use windows::core::PCWSTR;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let attrs = unsafe { GetFileAttributesW(PCWSTR(wide.as_ptr())) };
    if attrs == INVALID_FILE_ATTRIBUTES {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::NotFound {
            return Ok(false);
        }
        return Err(err);
    }
    Ok((attrs & FILE_ATTRIBUTE_REPARSE_POINT.0) != 0)
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn is_reparse_or_symlink__regular_file__false() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("regular.txt");
        std::fs::write(&file, b"hello").expect("write");
        let result = is_reparse_or_symlink(&file).expect("metadata");
        assert!(
            !result,
            "regular file must not be reported as reparse/symlink"
        );
    }

    #[test]
    fn is_reparse_or_symlink__missing_path__false() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist.txt");
        let result = is_reparse_or_symlink(&missing).expect("not found is ok");
        assert!(!result);
    }

    #[test]
    fn refuse_if_reparse__true__err() {
        let path = Path::new(r"C:\ProgramData\AI-Brains\nightly-task.bat");
        let err = refuse_if_reparse(path, true).expect_err("must refuse when is_reparse");
        let msg = err.to_ascii_lowercase();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "unexpected err: {err}"
        );
        assert!(
            msg.contains("nightly-task.bat"),
            "err should include path: {err}"
        );
    }

    #[test]
    fn refuse_if_reparse__false__ok() {
        let path = Path::new(r"C:\ProgramData\AI-Brains\nightly-task.bat");
        refuse_if_reparse(path, false).expect("must accept when not reparse");
    }

    #[test]
    fn is_reparse_or_symlink__pathbuf_accepts() {
        let missing = PathBuf::from("definitely-missing-reparse-check-t154");
        assert!(!is_reparse_or_symlink(&missing).expect("missing ok"));
    }
}
