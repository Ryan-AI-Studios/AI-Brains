//! Capability-relative open helpers with **component-wise nofollow / refuse-reparse**.
//!
//! # SOOT (T190 / ADR-0021 / F27)
//!
//! `cap-std` 4.0.x is **containment**, not nofollow: bare `Dir::open` / `Dir::open_dir`
//! **follow** symlinks. Product policy refuses **all** reparse/symlink/junction
//! components (even if the target stays inside the vault).
//!
//! Normative open sequence for vault-relative files:
//! 1. Lexical resolve (callers: reserved stems, `..` refuse) — F31
//! 2. `Dir::open_ambient_dir(root)` once for the trusted root — F21
//! 3. **Per component** open with platform nofollow / refuse-reparse:
//!    - **All platforms:** set `cap_fs_ext::OpenOptionsFollowExt::follow(FollowSymlinks::No)`.
//!      Without this, cap-primitives defaults to `FollowSymlinks::Yes` and **software-follows**
//!      after an OS nofollow probe (manual resolver on macOS / Linux-without-openat2) — F27 P0.
//!    - **Unix:** also `custom_flags(O_NOFOLLOW)` (dirs also `O_DIRECTORY`);
//!      map ELOOP → [`CapOpenError::ReparseRefused`].
//!    - **Windows:** also `FILE_FLAG_OPEN_REPARSE_POINT` (+ `FILE_FLAG_BACKUP_SEMANTICS`
//!      for directories); if the opened handle has reparse attribute, close and refuse.
//! 4. Handle-bound `metadata()` for size/`is_file`; capped read on the **same** handle.
//! 5. **Never** ambient `std::fs::read(path)` after open (F26).
//!
//! # Non-claims
//!
//! Not plugin isolation; not all ambient CLI paths; soft-canonicalize remains
//! non-claim for TOCTOU.

use std::io::Read;
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};
use thiserror::Error;

/// OpenOptions with product zero-symlink policy: never software-follow.
///
/// `cap-std` / `cap-primitives` default `follow = Yes`. OS `O_NOFOLLOW` alone is
/// insufficient on the manual path resolver (macOS; Linux without openat2).
fn nofollow_read_options() -> OpenOptions {
    let mut opts = OpenOptions::new();
    opts.read(true);
    opts.follow(FollowSymlinks::No);
    opts
}

/// Errors from capability / nofollow open helpers.
///
/// Callers map into domain errors (e.g. `VaultFsError`).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CapOpenError {
    #[error("path escape refused: {0}")]
    PathEscape(String),

    #[error("reparse/symlink/junction refused at {0}")]
    ReparseRefused(String),

    #[error("file oversized: {size} bytes exceeds max {max_bytes}")]
    Oversized { size: u64, max_bytes: u64 },

    #[error("path not found: {0}")]
    NotFound(String),

    #[error("not a regular file: {0}")]
    NotAFile(String),

    #[error("not a directory: {0}")]
    NotADir(String),

    #[error("I/O error: {0}")]
    Io(String),
}

/// Open a trusted vault root with ambient authority (once per operation).
///
/// The root itself is trusted (configured by the operator). Subsequent opens
/// are relative to this [`Dir`] with component-wise nofollow.
pub fn open_ambient_vault_dir(root: &Path) -> Result<Dir, CapOpenError> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|e| map_io_err(e, &root.display().to_string()))
}

/// Open a single directory component under `parent` without following reparse/symlinks.
pub fn open_dir_component_nofollow(parent: &Dir, name: &str) -> Result<Dir, CapOpenError> {
    validate_single_component(name)?;
    open_dir_component_nofollow_impl(parent, name)
}

/// Open a single regular-file component under `parent` without following reparse/symlinks.
pub fn open_file_component_nofollow(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    validate_single_component(name)?;
    open_file_component_nofollow_impl(parent, name)
}

/// Open a directory at `components` under `root` with per-component nofollow.
///
/// Empty `components` returns the ambient root dir.
pub fn open_dir_nofollow_components(
    root: &Path,
    components: &[impl AsRef<str>],
) -> Result<Dir, CapOpenError> {
    let mut current = open_ambient_vault_dir(root)?;
    for part in components {
        let name = part.as_ref();
        current = open_dir_component_nofollow(&current, name)?;
    }
    Ok(current)
}

/// Read a file at `components` under `root` with per-component nofollow and
/// handle-bound size enforcement (F8 / F29).
///
/// Does **not** fall back to ambient `std::fs::read` (F26).
pub fn read_file_nofollow_components(
    root: &Path,
    components: &[impl AsRef<str>],
    max_bytes: u64,
) -> Result<Vec<u8>, CapOpenError> {
    if components.is_empty() {
        return Err(CapOpenError::NotAFile("empty relative path".into()));
    }

    let mut current = open_ambient_vault_dir(root)?;
    let last = components.len() - 1;
    for (i, part) in components.iter().enumerate() {
        let name = part.as_ref();
        if i < last {
            current = open_dir_component_nofollow(&current, name)?;
        } else {
            let mut file = open_file_component_nofollow(&current, name)?;
            return read_file_handle_capped(&mut file, name, max_bytes);
        }
    }
    // Unreachable: last component always returns above.
    Err(CapOpenError::Io("internal: empty component walk".into()))
}

/// List immediate child names of `dir` (deterministic sort).
///
/// Uses the [`Dir`] handle (`entries`), not ambient `std::fs::read_dir`.
pub fn list_entry_names(dir: &Dir) -> Result<Vec<String>, CapOpenError> {
    let rd = dir.entries().map_err(|e| CapOpenError::Io(e.to_string()))?;
    let mut names: Vec<String> = Vec::new();
    for ent in rd {
        let ent = ent.map_err(|e| CapOpenError::Io(e.to_string()))?;
        let name = ent.file_name();
        let s = name.to_string_lossy().into_owned();
        // Skip `.` / `..` if ever present.
        if s == "." || s == ".." {
            continue;
        }
        names.push(s);
    }
    names.sort();
    Ok(names)
}

/// Read handle-bound metadata size + capped bytes from an already-opened file.
fn read_file_handle_capped(
    file: &mut File,
    label: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, CapOpenError> {
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {label}: {e}")))?;
    if meta.is_symlink() {
        return Err(CapOpenError::ReparseRefused(label.to_string()));
    }
    if !meta.is_file() {
        return Err(CapOpenError::NotAFile(label.to_string()));
    }
    let size = meta.len();
    if size > max_bytes {
        return Err(CapOpenError::Oversized { size, max_bytes });
    }

    // Streaming cap: even if size grows after metadata, never exceed max_bytes.
    let mut buf = Vec::new();
    let mut limited = file.take(max_bytes.saturating_add(1));
    limited
        .read_to_end(&mut buf)
        .map_err(|e| CapOpenError::Io(format!("read {label}: {e}")))?;
    if (buf.len() as u64) > max_bytes {
        return Err(CapOpenError::Oversized {
            size: buf.len() as u64,
            max_bytes,
        });
    }
    Ok(buf)
}

fn validate_single_component(name: &str) -> Result<(), CapOpenError> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(CapOpenError::PathEscape(format!(
            "invalid path component '{name}'"
        )));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(CapOpenError::PathEscape(format!(
            "multi-segment component refused: '{name}'"
        )));
    }
    Ok(())
}

fn map_io_err(e: std::io::Error, label: &str) -> CapOpenError {
    match e.kind() {
        std::io::ErrorKind::NotFound => CapOpenError::NotFound(label.to_string()),
        _ if is_symlink_loop_error(&e) => CapOpenError::ReparseRefused(label.to_string()),
        _ => CapOpenError::Io(format!("{label}: {e}")),
    }
}

fn is_symlink_loop_error(e: &std::io::Error) -> bool {
    // Unix: ELOOP → "Too many levels of symbolic links"
    // Windows: ERROR_STOPPED_ON_SYMLINK or similar messages from cap-std
    let msg = e.to_string().to_ascii_lowercase();
    if msg.contains("too many levels of symbolic links")
        || msg.contains("symbolic link")
        || msg.contains("symlink")
        || msg.contains("reparse")
    {
        return true;
    }
    // Linux ELOOP=40, macOS ELOOP=62 — map common values without libc dep.
    matches!(e.raw_os_error(), Some(40) | Some(62) | Some(114))
}

// --- Platform open implementations ---

#[cfg(unix)]
fn open_dir_component_nofollow_impl(parent: &Dir, name: &str) -> Result<Dir, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use rustix::fs::OFlags;

    let mut opts = nofollow_read_options();
    // O_NOFOLLOW refuses final-component symlink; O_DIRECTORY requires a dir.
    opts.custom_flags((OFlags::NOFOLLOW | OFlags::DIRECTORY).bits() as i32);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_io_err(e, name))?;
    // Belt: refuse if somehow a symlink handle.
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if meta.is_symlink() {
        return Err(CapOpenError::ReparseRefused(name.to_string()));
    }
    if !meta.is_dir() {
        return Err(CapOpenError::NotADir(name.to_string()));
    }
    Ok(Dir::from_std_file(file.into_std()))
}

#[cfg(unix)]
fn open_file_component_nofollow_impl(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use rustix::fs::OFlags;

    let mut opts = nofollow_read_options();
    opts.custom_flags(OFlags::NOFOLLOW.bits() as i32);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_io_err(e, name))?;
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if meta.is_symlink() {
        return Err(CapOpenError::ReparseRefused(name.to_string()));
    }
    if !meta.is_file() {
        return Err(CapOpenError::NotAFile(name.to_string()));
    }
    Ok(file)
}

#[cfg(windows)]
fn open_dir_component_nofollow_impl(parent: &Dir, name: &str) -> Result<Dir, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use windows::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };

    let mut opts = nofollow_read_options();
    // Open the reparse node itself (do not follow). BACKUP_SEMANTICS required for dirs.
    opts.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0 | FILE_FLAG_BACKUP_SEMANTICS.0);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_io_err(e, name))?;
    refuse_if_handle_reparse(&file, name)?;
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if !meta.is_dir() {
        return Err(CapOpenError::NotADir(name.to_string()));
    }
    Ok(Dir::from_std_file(file.into_std()))
}

#[cfg(windows)]
fn open_file_component_nofollow_impl(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use windows::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

    let mut opts = nofollow_read_options();
    opts.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_io_err(e, name))?;
    refuse_if_handle_reparse(&file, name)?;
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if !meta.is_file() {
        return Err(CapOpenError::NotAFile(name.to_string()));
    }
    Ok(file)
}

#[cfg(windows)]
fn refuse_if_handle_reparse(file: &File, name: &str) -> Result<(), CapOpenError> {
    use std::os::windows::fs::MetadataExt;
    use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    // Clone handle → std::fs::File for Windows MetadataExt::file_attributes.
    let std_file = file
        .try_clone()
        .map_err(|e| CapOpenError::Io(format!("clone handle {name}: {e}")))?
        .into_std();
    let meta = std_file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if meta.file_type().is_symlink() {
        return Err(CapOpenError::ReparseRefused(name.to_string()));
    }
    let attrs = meta.file_attributes();
    if (attrs & FILE_ATTRIBUTE_REPARSE_POINT.0) != 0 {
        return Err(CapOpenError::ReparseRefused(name.to_string()));
    }
    Ok(())
}

// Fallback for non-unix/non-windows (should not ship, but keep compile).
#[cfg(not(any(unix, windows)))]
fn open_dir_component_nofollow_impl(_parent: &Dir, name: &str) -> Result<Dir, CapOpenError> {
    Err(CapOpenError::Io(format!(
        "nofollow dir open unsupported on this platform ({name})"
    )))
}

#[cfg(not(any(unix, windows)))]
fn open_file_component_nofollow_impl(_parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    Err(CapOpenError::Io(format!(
        "nofollow file open unsupported on this platform ({name})"
    )))
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn read_under_root_cap__regular_file__ok() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        fs::create_dir_all(&notes).expect("mkdir");
        fs::write(notes.join("alpha.md"), b"hello vault").expect("write");

        let bytes = read_file_nofollow_components(dir.path(), &["notes", "alpha.md"], 1_048_576)
            .expect("ok");
        assert_eq!(bytes, b"hello vault");
    }

    #[test]
    fn read_under_root_cap__final_symlink__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        fs::create_dir_all(&notes).expect("mkdir");
        let target = dir.path().join("target.md");
        fs::write(&target, b"secret").expect("target");

        let link = notes.join("link.md");
        if !create_file_symlink(&target, &link) {
            eprintln!(
                "soft-skip: could not create file symlink (privilege missing). \
                 Final-symlink refuse covered when privilege available."
            );
            return;
        }

        let err = read_file_nofollow_components(dir.path(), &["notes", "link.md"], 1_048_576)
            .expect_err("final symlink");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
    }

    /// In-vault target: containment alone would allow follow; F9 still refuses.
    #[test]
    fn read_under_root_cap__final_in_vault_symlink__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        fs::create_dir_all(&notes).expect("mkdir");
        let target = notes.join("real.md");
        fs::write(&target, b"in-vault secret").expect("target");

        let link = notes.join("alias.md");
        if !create_file_symlink(&target, &link) {
            eprintln!("soft-skip: could not create in-vault file symlink (privilege missing).");
            return;
        }

        let err = read_file_nofollow_components(dir.path(), &["notes", "alias.md"], 1_048_576)
            .expect_err("in-vault final symlink must refuse (F9), not follow");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused for in-vault symlink, got {err:?}"
        );
    }

    #[test]
    fn read_under_root_cap__intermediate_symlink__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        fs::create_dir_all(&notes).expect("mkdir");

        let outside = tempdir().expect("outside");
        fs::write(outside.path().join("file.md"), b"SECRET").expect("outside file");

        let link = notes.join("evil");
        if !create_dir_symlink(outside.path(), &link) {
            eprintln!("soft-skip: could not create dir symlink/junction (privilege missing).");
            return;
        }

        let err =
            read_file_nofollow_components(dir.path(), &["notes", "evil", "file.md"], 1_048_576)
                .expect_err("intermediate reparse");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
    }

    #[test]
    fn read_under_root_cap__parent_escape__refused() {
        let dir = tempdir().expect("tempdir");
        let err = read_file_nofollow_components(dir.path(), &["..", "etc", "passwd"], 1024)
            .expect_err("parent");
        assert!(
            matches!(err, CapOpenError::PathEscape(_)),
            "expected PathEscape, got {err:?}"
        );
    }

    #[test]
    fn read_under_root_cap__handle_metadata_size_cap() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("big.md"), vec![b'x'; 100]).expect("write");
        let err = read_file_nofollow_components(dir.path(), &["big.md"], 50).expect_err("cap");
        assert!(
            matches!(
                err,
                CapOpenError::Oversized {
                    size: 100,
                    max_bytes: 50
                }
            ),
            "expected Oversized, got {err:?}"
        );
    }

    #[test]
    fn read_under_root_cap__open_fail__no_ambient_fallback() {
        let dir = tempdir().expect("tempdir");
        // Missing file: must be NotFound/Io — never ambient-read success.
        let err =
            read_file_nofollow_components(dir.path(), &["missing.md"], 1024).expect_err("missing");
        assert!(
            matches!(err, CapOpenError::NotFound(_) | CapOpenError::Io(_)),
            "expected NotFound/Io without ambient fallback, got {err:?}"
        );
        // Ensure no accidental create.
        assert!(!dir.path().join("missing.md").exists());
    }

    #[cfg(windows)]
    #[test]
    fn read_under_root_cap__intermediate_junction__refused() {
        let dir = tempdir().expect("tempdir");
        let notes = dir.path().join("notes");
        fs::create_dir_all(&notes).expect("mkdir");

        let outside = tempdir().expect("outside");
        fs::write(outside.path().join("file.md"), b"JSECRET").expect("outside");

        let link = notes.join("junc");
        if !create_dir_junction(outside.path(), &link) {
            eprintln!("soft-skip: could not create directory junction (privilege or API missing).");
            return;
        }

        let err =
            read_file_nofollow_components(dir.path(), &["notes", "junc", "file.md"], 1_048_576)
                .expect_err("junction");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn read_under_root_cap__final_junction__refused() {
        // Final component as junction (directory reparse) cannot be a regular file read.
        let dir = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        fs::write(outside.path().join("x.md"), b"j").expect("write");
        let link = dir.path().join("final_junc");
        if !create_dir_junction(outside.path(), &link) {
            eprintln!("soft-skip: could not create final directory junction.");
            return;
        }
        let err = read_file_nofollow_components(dir.path(), &["final_junc"], 1_048_576)
            .expect_err("final junction not a file");
        // Fail closed: ReparseRefused preferred; NotAFile / Io (access denied on
        // reparse-as-file without BACKUP_SEMANTICS) are also closed (not ambient read).
        assert!(
            matches!(
                err,
                CapOpenError::ReparseRefused(_) | CapOpenError::NotAFile(_) | CapOpenError::Io(_)
            ),
            "expected refuse / fail-closed, got {err:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn read_under_root_cap__final_symlink__refused_windows() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("t.md");
        fs::write(&target, b"s").expect("write");
        let link = dir.path().join("l.md");
        if !create_file_symlink(&target, &link) {
            eprintln!(
                "soft-skip: could not create final file symlink (Developer Mode / SeCreateSymbolicLinkPrivilege missing)."
            );
            return;
        }
        let err = read_file_nofollow_components(dir.path(), &["l.md"], 1024).expect_err("symlink");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused, got {err:?}"
        );
    }

    fn create_file_symlink(target: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(target, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (target, link);
            false
        }
    }

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

    #[cfg(windows)]
    fn create_dir_junction(target: &Path, link: &Path) -> bool {
        // Junctions typically do not need SeCreateSymbolicLinkPrivilege.
        // Use `std::os::windows::fs::symlink_dir` first; if that fails, try `cmd /c mklink /J`.
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return true;
        }
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .status();
        matches!(status, Ok(s) if s.success())
    }
}
