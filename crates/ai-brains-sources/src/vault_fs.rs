//! Vault-relative path safety for Markdown / Obsidian connectors (T154).
//!
//! # Policy locks
//!
//! - **Containment:** candidates must stay under the configured vault root
//!   (`ai_brains_path::path_is_same_or_inside`).
//! - **Reparse refuse:** detect symlink/junction via `ai_brains_path` and fail
//!   closed (does **not** follow attacker-controlled links out of the vault).
//! - **Reserved Windows stems:** blanket case-insensitive stem match against
//!   classic device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`,
//!   `LPT1`–`LPT9`). Intentionally conservative; false-positives such as
//!   `aux.md` are refused. Stem = last component with **one** extension stripped
//!   (so `com1-meeting-notes.md` is allowed; `aux.md` is not).
//!
//! # Residual TOCTOU
//!
//! Check-then-open without `openat` / cap-std remains racy if a link is swapped
//! between reparse detection and read. Documented residual for T182-adjacent work.

use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// Errors from vault filesystem helpers.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VaultFsError {
    #[error("empty relative path")]
    EmptyRelative,

    #[error("absolute path refused: {0}")]
    AbsolutePath(String),

    #[error("path escape refused: {0}")]
    PathEscape(String),

    #[error("reserved Windows device stem refused: {0}")]
    ReservedStem(String),

    #[error("reparse/symlink/junction refused at {0}")]
    ReparseRefused(String),

    #[error("file oversized: {size} bytes exceeds max {max_bytes}")]
    Oversized { size: u64, max_bytes: u64 },

    #[error("path not found: {0}")]
    NotFound(String),

    #[error("I/O error: {0}")]
    Io(String),
}

/// Classic Windows reserved device stems (case-insensitive).
///
/// Blanket match after stripping **one** extension from the path component.
const RESERVED_STEMS: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Return the stem of a single path component (strip one extension if present).
///
/// - `aux.md` → `aux`
/// - `com1-meeting-notes.md` → `com1-meeting-notes`
/// - `con` → `con`
/// - `.obsidian` → `.obsidian` (leading-dot only; no extension strip)
fn component_stem(component: &str) -> &str {
    match component.rsplit_once('.') {
        Some((stem, _ext)) if !stem.is_empty() => stem,
        _ => component,
    }
}

/// True when `component` (a single path segment, possibly with extension) is a
/// reserved Windows device stem under the blanket T154 policy.
pub fn is_reserved_windows_stem(component: &str) -> bool {
    let stem = component_stem(component);
    RESERVED_STEMS
        .iter()
        .any(|r| stem.eq_ignore_ascii_case(r))
}

/// Normalize a vault-relative locator to forward slashes without a leading `./`.
pub fn normalize_locator(relative: &str) -> String {
    let trimmed = relative.trim().trim_start_matches("./");
    trimmed.replace('\\', "/")
}

/// Resolve `relative` under `root` with containment + reserved-stem checks.
///
/// Rejects empty, absolute, `..` escapes, and reserved device stems in **any**
/// component. Does not require the candidate to exist on disk.
pub fn resolve_under_root(root: &Path, relative: &str) -> Result<PathBuf, VaultFsError> {
    let relative = relative.trim();
    if relative.is_empty() || relative == "." {
        // Root itself (vault handle); still require containment of root under root.
        let root_abs = absolute_root(root)?;
        return Ok(root_abs);
    }

    let rel_path = Path::new(relative);
    if rel_path.is_absolute() {
        return Err(VaultFsError::AbsolutePath(relative.to_string()));
    }

    // Reject absolute-looking forms that Path may not treat as absolute on all OS.
    if relative.starts_with('/') || relative.starts_with('\\') {
        return Err(VaultFsError::AbsolutePath(relative.to_string()));
    }
    // Drive-letter absolute (Windows-style) even when running tests elsewhere.
    let bytes = relative.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(VaultFsError::AbsolutePath(relative.to_string()));
    }

    let mut safe_parts: Vec<String> = Vec::new();
    for comp in rel_path.components() {
        match comp {
            Component::Normal(os) => {
                let s = os.to_string_lossy();
                if is_reserved_windows_stem(&s) {
                    return Err(VaultFsError::ReservedStem(s.into_owned()));
                }
                // Reject empty-ish / dot-dot disguised as Normal (should not happen).
                if s == ".." || s == "." {
                    return Err(VaultFsError::PathEscape(format!(
                        "invalid path component '{s}' in {relative}"
                    )));
                }
                safe_parts.push(s.into_owned());
            }
            Component::CurDir => {
                // skip "."
            }
            Component::ParentDir => {
                return Err(VaultFsError::PathEscape(format!(
                    "parent traversal in {relative}"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(VaultFsError::AbsolutePath(relative.to_string()));
            }
        }
    }

    if safe_parts.is_empty() {
        let root_abs = absolute_root(root)?;
        return Ok(root_abs);
    }

    let root_abs = absolute_root(root)?;
    let mut candidate = root_abs.clone();
    for part in &safe_parts {
        candidate.push(part);
    }

    if !ai_brains_path::path_is_same_or_inside(&candidate, &root_abs) {
        return Err(VaultFsError::PathEscape(format!(
            "{} is outside vault root {}",
            candidate.display(),
            root_abs.display()
        )));
    }

    Ok(candidate)
}

/// Absolute form of `root` for containment compares.
fn absolute_root(root: &Path) -> Result<PathBuf, VaultFsError> {
    if let Ok(canon) = root.canonicalize() {
        return Ok(canon);
    }
    if root.is_absolute() {
        return Ok(root.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|e| VaultFsError::Io(e.to_string()))?;
    Ok(cwd.join(root))
}

/// Refuse when `path` is a reparse point / symlink / junction.
pub fn refuse_reparse_path(path: &Path) -> Result<(), VaultFsError> {
    let is_reparse = ai_brains_path::is_reparse_or_symlink(path)
        .map_err(|e| VaultFsError::Io(e.to_string()))?;
    if is_reparse {
        return Err(VaultFsError::ReparseRefused(path.display().to_string()));
    }
    Ok(())
}

/// Resolve under root → reparse check → size cap → read full bytes.
pub fn read_file_under_root(
    root: &Path,
    relative: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, VaultFsError> {
    let path = resolve_under_root(root, relative)?;
    refuse_reparse_path(&path)?;

    let meta = match std::fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(VaultFsError::NotFound(relative.to_string()));
        }
        Err(e) => return Err(VaultFsError::Io(e.to_string())),
    };

    if !meta.is_file() {
        return Err(VaultFsError::Io(format!(
            "not a regular file: {}",
            path.display()
        )));
    }

    let size = meta.len();
    if size > max_bytes {
        return Err(VaultFsError::Oversized { size, max_bytes });
    }

    // Re-check reparse immediately before open (still TOCTOU residual).
    refuse_reparse_path(&path)?;

    std::fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            VaultFsError::NotFound(relative.to_string())
        } else {
            VaultFsError::Io(e.to_string())
        }
    })
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod unit_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_under_root__parent_escape__errors() {
        let dir = tempdir().expect("tempdir");
        let err = resolve_under_root(dir.path(), "notes/../../outside.md")
            .expect_err("parent escape");
        assert!(matches!(err, VaultFsError::PathEscape(_)));
    }

    #[test]
    fn resolve_under_root__absolute_outside__errors() {
        let dir = tempdir().expect("tempdir");
        let err = resolve_under_root(dir.path(), r"C:\Windows\System32\drivers\etc\hosts")
            .expect_err("absolute");
        assert!(matches!(err, VaultFsError::AbsolutePath(_)));

        let err2 = resolve_under_root(dir.path(), "/etc/passwd").expect_err("unix abs");
        assert!(matches!(err2, VaultFsError::AbsolutePath(_)));
    }

    #[test]
    fn resolve_under_root__normalized_relative__ok() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");
        let resolved = resolve_under_root(dir.path(), "notes/alpha.md").expect("ok");
        assert!(ai_brains_path::path_is_same_or_inside(&resolved, dir.path()));
        assert!(resolved.ends_with(Path::new("notes").join("alpha.md")));
    }

    #[test]
    fn reserved_stem__aux_md__refused() {
        assert!(is_reserved_windows_stem("aux.md"));
        let dir = tempdir().expect("tempdir");
        let err = resolve_under_root(dir.path(), "aux.md").expect_err("aux");
        assert!(matches!(err, VaultFsError::ReservedStem(_)));
    }

    #[test]
    fn reserved_stem__nul_md__refused() {
        assert!(is_reserved_windows_stem("nul.md"));
    }

    #[test]
    fn reserved_stem__con_uppercase__refused() {
        assert!(is_reserved_windows_stem("CON"));
        assert!(is_reserved_windows_stem("Con.MD"));
    }

    #[test]
    fn reserved_stem__com1_meeting_notes_md__allowed() {
        assert!(!is_reserved_windows_stem("com1-meeting-notes.md"));
        let dir = tempdir().expect("tempdir");
        let _ = resolve_under_root(dir.path(), "notes/com1-meeting-notes.md").expect("allowed");
    }

    #[test]
    fn reserved_stem__notes_con_md__refused() {
        // component "con.md" stem "con"
        assert!(is_reserved_windows_stem("con.md"));
        let dir = tempdir().expect("tempdir");
        let err = resolve_under_root(dir.path(), "notes/con.md").expect_err("con");
        assert!(matches!(err, VaultFsError::ReservedStem(_)));
    }

    #[test]
    fn refuse_reparse_path__regular_file__ok() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("note.md");
        std::fs::write(&file, b"hi").expect("write");
        refuse_reparse_path(&file).expect("regular file ok");
    }

    #[test]
    fn normalize_locator__slashes() {
        assert_eq!(normalize_locator(r"notes\alpha.md"), "notes/alpha.md");
        assert_eq!(normalize_locator("./notes/a.md"), "notes/a.md");
    }
}
