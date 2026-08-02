//! Vault-relative path safety for Markdown / Obsidian connectors (T154).
//!
//! # Policy locks
//!
//! - **Containment:** candidates must stay under the configured vault root
//!   via lexical `lexical_same_or_inside` (no symlink follow — T179 Linux
//!   honesty so reparse refuse is not pre-empted by PathEscape).
//! - **Reparse refuse:** detect symlink/junction via `ai_brains_path` and fail
//!   closed on the vault root **and every intermediate path component** (does
//!   **not** follow attacker-controlled links out of the vault).
//! - **Reserved Windows stems:** blanket case-insensitive stem match against
//!   classic device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`,
//!   `LPT1`–`LPT9`). Intentionally conservative; false-positives such as
//!   `aux.md` are refused. Stem = last component with **one** extension stripped
//!   (so `com1-meeting-notes.md` is allowed; `aux.md` is not).
//!
//! # Path open (T190)
//!
//! Vault-relative reads use [`ai_brains_path`] capability helpers: ambient root
//! once + per-component nofollow open + handle-bound size/read (ADR-0021).
//! Lexical resolve + reserved stems remain pre-open gates (F31).

use std::path::{Component, Path, PathBuf};

use ai_brains_path::CapOpenError;
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
    RESERVED_STEMS.iter().any(|r| stem.eq_ignore_ascii_case(r))
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
///
/// Does **not** open the path or perform open-time reparse refuse. For vault
/// content I/O use [`read_file_under_root`] (T190 cap-std component nofollow).
/// [`refuse_reparse_along_path`] remains available for legacy pre-checks but is
/// **not** the SOOT for read/list (check-then-open residual).
///
/// Containment is **lexical** (no symlink follow). Following reparse here would
/// turn intermediate symlink escapes into [`VaultFsError::PathEscape`] before
/// open-time refuse can run (T154 R1-01 / T179 Linux).
pub fn resolve_under_root(root: &Path, relative: &str) -> Result<PathBuf, VaultFsError> {
    let safe_parts = safe_relative_components(relative)?;
    let root_abs = absolute_root(root)?;
    if safe_parts.is_empty() {
        return Ok(root_abs);
    }

    let mut candidate = root_abs.clone();
    for part in &safe_parts {
        candidate.push(part);
    }

    if !lexical_same_or_inside(&candidate, &root_abs) {
        return Err(VaultFsError::PathEscape(format!(
            "{} is outside vault root {}",
            candidate.display(),
            root_abs.display()
        )));
    }

    Ok(candidate)
}

/// Component-wise containment without following reparse/symlinks.
///
/// Prefer this over [`ai_brains_path::path_is_same_or_inside`] when the candidate
/// was built from Normal components only and intermediate reparse will be
/// checked separately — that helper resolves best-effort and follows links.
fn lexical_same_or_inside(candidate: &Path, root: &Path) -> bool {
    if candidate == root {
        return true;
    }
    candidate.starts_with(root)
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
    let is_reparse =
        ai_brains_path::is_reparse_or_symlink(path).map_err(|e| VaultFsError::Io(e.to_string()))?;
    if is_reparse {
        return Err(VaultFsError::ReparseRefused(path.display().to_string()));
    }
    Ok(())
}

/// Refuse reparse/symlink/junction on `root` and **every intermediate component**
/// along `relative_components`, including the final path.
///
/// Spec lock (T154 R1-01): independent of whether the resolved target lands
/// inside the vault — intermediate reparse points are refused.
///
/// Missing path components are not treated as reparse (`is_reparse_or_symlink`
/// returns false for not-found); callers still get NotFound on read.
pub fn refuse_reparse_along_path(
    root: &Path,
    relative_components: &[impl AsRef<str>],
) -> Result<(), VaultFsError> {
    refuse_reparse_path(root)?;
    let mut current = root.to_path_buf();
    for part in relative_components {
        current.push(part.as_ref());
        refuse_reparse_path(&current)?;
    }
    Ok(())
}

/// Collect safe relative components (no FS access) — shared with resolve/read.
fn safe_relative_components(relative: &str) -> Result<Vec<String>, VaultFsError> {
    let relative = relative.trim();
    if relative.is_empty() || relative == "." {
        return Ok(Vec::new());
    }

    let rel_path = Path::new(relative);
    if rel_path.is_absolute() {
        return Err(VaultFsError::AbsolutePath(relative.to_string()));
    }
    if relative.starts_with('/') || relative.starts_with('\\') {
        return Err(VaultFsError::AbsolutePath(relative.to_string()));
    }
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
                if s == ".." || s == "." {
                    return Err(VaultFsError::PathEscape(format!(
                        "invalid path component '{s}' in {relative}"
                    )));
                }
                safe_parts.push(s.into_owned());
            }
            Component::CurDir => {}
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
    Ok(safe_parts)
}

/// Resolve under root (lexical) → component-wise nofollow open → handle-bound read.
///
/// Never falls back to ambient `std::fs::read` (T190 F26).
pub fn read_file_under_root(
    root: &Path,
    relative: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, VaultFsError> {
    // Lexical containment + reserved stems (F31) before any open.
    let _path = resolve_under_root(root, relative)?;
    let components = safe_relative_components(relative)?;
    if components.is_empty() {
        return Err(VaultFsError::EmptyRelative);
    }

    ai_brains_path::read_file_nofollow_components(root, &components, max_bytes)
        .map_err(map_cap_open_err)
}

fn map_cap_open_err(e: CapOpenError) -> VaultFsError {
    match e {
        CapOpenError::PathEscape(s) => VaultFsError::PathEscape(s),
        CapOpenError::ReparseRefused(s) => VaultFsError::ReparseRefused(s),
        CapOpenError::Oversized { size, max_bytes } => VaultFsError::Oversized { size, max_bytes },
        CapOpenError::NotFound(s) => VaultFsError::NotFound(s),
        CapOpenError::NotAFile(s) => VaultFsError::Io(format!("not a regular file: {s}")),
        CapOpenError::NotADir(s) => VaultFsError::Io(format!("not a directory: {s}")),
        CapOpenError::Io(s) => {
            // Cap-open NotFound is typed; remaining Io may still mention missing paths.
            if s.to_ascii_lowercase().contains("not found")
                || s.to_ascii_lowercase().contains("cannot find")
                || s.to_ascii_lowercase().contains("no such file")
            {
                VaultFsError::NotFound(s)
            } else {
                VaultFsError::Io(s)
            }
        }
    }
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
        let err =
            resolve_under_root(dir.path(), "notes/../../outside.md").expect_err("parent escape");
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
        assert!(ai_brains_path::path_is_same_or_inside(
            &resolved,
            dir.path()
        ));
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
    fn refuse_reparse_along_path__regular_chain__ok() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");
        let file = notes.join("alpha.md");
        std::fs::write(&file, b"hi").expect("write");
        refuse_reparse_along_path(dir.path(), &["notes", "alpha.md"]).expect("regular chain");
    }

    #[test]
    fn refuse_reparse_along_path__walks_each_component() {
        // Pure structural: every component is joined and checked; missing leaves
        // are not reparse (NotFound → false), so a chain with a missing tail is ok.
        let dir = tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("notes")).expect("mkdir");
        refuse_reparse_along_path(dir.path(), &["notes", "missing.md"])
            .expect("missing leaf is not reparse");
    }

    #[test]
    fn refuse_reparse_along_path__intermediate_symlink__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");

        let outside = tempdir().expect("outside");
        std::fs::write(outside.path().join("file.md"), b"secret").expect("outside file");

        let link = notes.join("evil");
        let created = create_dir_symlink(outside.path(), &link);
        if !created {
            eprintln!(
                "soft-skip: could not create dir symlink/junction (privilege missing). \
                 Component-walk coverage remains in refuse_reparse_along_path__regular_chain__ok."
            );
            return;
        }

        let err = refuse_reparse_along_path(dir.path(), &["notes", "evil", "file.md"])
            .expect_err("intermediate reparse");
        assert!(
            matches!(err, VaultFsError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "{err}"
        );
    }

    #[test]
    fn read_file_under_root__intermediate_reparse__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");

        let outside = tempdir().expect("outside");
        std::fs::write(outside.path().join("file.md"), b"SECRET").expect("outside file");

        let link = notes.join("evil");
        let created = create_dir_symlink(outside.path(), &link);
        if !created {
            eprintln!("soft-skip: could not create dir symlink/junction (privilege missing).");
            return;
        }

        let err = read_file_under_root(dir.path(), "notes/evil/file.md", 1_048_576)
            .expect_err("intermediate reparse on read");
        assert!(
            matches!(err, VaultFsError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
    }

    #[test]
    fn normalize_locator__slashes() {
        assert_eq!(normalize_locator(r"notes\alpha.md"), "notes/alpha.md");
        assert_eq!(normalize_locator("./notes/a.md"), "notes/a.md");
    }

    /// Directory symlink (Unix) or Windows dir symlink/junction; false if unsupported.
    fn create_dir_symlink(target: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (target, link);
            false
        }
    }
}
