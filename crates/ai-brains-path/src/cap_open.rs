//! Capability-relative open helpers with **component-wise nofollow / refuse-reparse**.
//!
//! # SOOT (T190 / ADR-0021 / F27; write path T193 / F4–F11)
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
//! # Write SOOT (T193)
//!
//! - Create path: `create_new` + nofollow flags only (never `truncate` + Windows OPEN_REPARSE).
//! - Replace path (preferred O1): RO nofollow probe (regular + nlink==1) → temp `create_new`
//!   under same parent → write+sync → atomic rename to final name.
//! - Symlink/reparse leaf: refuse without destroying target content (AC13).
//! - Hardlink leaf (nlink>1): refuse via **handle** metadata (AC14 / F10).
//! - **Never** ambient `std::fs::write` fallback (F7).
//!
//! # Non-claims
//!
//! Not plugin isolation; not all ambient CLI paths; soft-canonicalize remains
//! non-claim for TOCTOU.

use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};
use thiserror::Error;

/// Create vs replace for nofollow leaf writes (T193 F9).
///
/// **Only** these two modes. Truncate-on-open is forbidden (Windows
/// `TRUNCATE_EXISTING` + `FILE_FLAG_OPEN_REPARSE_POINT` truncates the reparse
/// node at open — F31 / AC13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateMode {
    /// Fail if the leaf name already exists (`create_new`).
    CreateNew,
    /// Replace an existing regular single-link file via temp-rename (O1),
    /// or create if missing. Refuses symlink/reparse/hardlink leaves.
    Replace,
}

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

/// Write/create OpenOptions: `create_new` + nofollow; **never** `truncate(true)`.
///
/// Platform flags are applied in [`create_file_component_nofollow_impl`].
/// Public for call-site audit / unit proof that truncate is unset.
pub fn nofollow_write_options_create_new() -> OpenOptions {
    let mut opts = OpenOptions::new();
    opts.write(true);
    opts.create_new(true);
    opts.follow(FollowSymlinks::No);
    // NEVER: opts.truncate(true) — especially with Windows OPEN_REPARSE_POINT (F31).
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

    #[error("hardlink refused (nlink > 1) at {0}")]
    HardlinkRefused(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

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

/// Thin alias for non-vault trusted parents (token, ProgramData, kit) — F8 / F34.
///
/// Functionally identical to [`open_ambient_vault_dir`]; name avoids implying
/// vault-only use. Prefer this at non-vault call sites.
#[inline]
pub fn open_ambient_dir(path: &Path) -> Result<Dir, CapOpenError> {
    open_ambient_vault_dir(path)
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

/// Create a new regular-file leaf under `parent` with nofollow + `create_new`.
///
/// Post-open: refuse reparse/symlink handles, require `is_file()`, require nlink==1.
/// Does **not** set `maybe_dir(true)` (F6 / L4).
pub fn create_file_component_nofollow(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    validate_single_component(name)?;
    create_file_component_nofollow_impl(parent, name)
}

/// Read a single leaf under an already-opened parent Dir (nofollow handle read).
///
/// Preferred for absolute-path token load: ambient parent once, then this.
pub fn read_file_nofollow_leaf(
    parent: &Dir,
    file_name: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, CapOpenError> {
    validate_single_component(file_name)?;
    let mut file = open_file_component_nofollow(parent, file_name)?;
    read_file_handle_capped(&mut file, file_name, max_bytes)
}

/// Write `bytes` to a single leaf under `parent` with nofollow create/replace SOOT.
///
/// - [`CreateMode::CreateNew`]: create_new → write_all → sync_all
/// - [`CreateMode::Replace`]: O1 temp create_new + write+sync + atomic rename; or
///   create if missing. Symlink/reparse leaf → refuse without destroying target.
///
/// Never falls back to ambient `std::fs::write` (F7).
pub fn write_file_nofollow_leaf(
    parent: &Dir,
    file_name: &str,
    bytes: &[u8],
    mode: CreateMode,
) -> Result<(), CapOpenError> {
    validate_single_component(file_name)?;
    match mode {
        CreateMode::CreateNew => {
            let mut file = create_file_component_nofollow(parent, file_name)?;
            write_and_sync_handle(&mut file, file_name, bytes)
        }
        CreateMode::Replace => replace_file_nofollow_leaf(parent, file_name, bytes),
    }
}

/// Convenience: ambient-open `parent_dir`, then [`write_file_nofollow_leaf`].
pub fn write_file_nofollow_under_parent_path(
    parent_dir: &Path,
    file_name: &str,
    bytes: &[u8],
    mode: CreateMode,
) -> Result<(), CapOpenError> {
    let parent = open_ambient_dir(parent_dir)?;
    write_file_nofollow_leaf(&parent, file_name, bytes, mode)
}

/// Replace path (F9 O1): probe existing leaf RO nofollow → temp create_new → rename.
fn replace_file_nofollow_leaf(
    parent: &Dir,
    file_name: &str,
    bytes: &[u8],
) -> Result<(), CapOpenError> {
    // Probe whether the leaf exists and is a safe replace target.
    match open_file_component_nofollow_impl(parent, file_name) {
        Ok(probe) => {
            // Existing regular file (post-open already refused reparse).
            refuse_if_hardlink_handle(&probe, file_name)?;
            drop(probe);
            write_via_temp_rename(parent, file_name, bytes)
        }
        Err(CapOpenError::NotFound(_)) => {
            // Missing: create_new path.
            let mut file = create_file_component_nofollow(parent, file_name)?;
            write_and_sync_handle(&mut file, file_name, bytes)
        }
        Err(CapOpenError::ReparseRefused(label)) => {
            // Symlink/reparse leaf: refuse without writing or truncating target (AC13).
            Err(CapOpenError::ReparseRefused(label))
        }
        Err(other) => {
            // NotAFile / Io / etc.: if leaf is a symlink we may have missed open;
            // classify via symlink_metadata for clearer refuse.
            if leaf_is_reparse(parent, file_name) {
                return Err(CapOpenError::ReparseRefused(file_name.to_string()));
            }
            Err(other)
        }
    }
}

/// O1: create temp leaf under same parent, write+sync, atomic rename to final.
fn write_via_temp_rename(parent: &Dir, file_name: &str, bytes: &[u8]) -> Result<(), CapOpenError> {
    let temp_name = unique_temp_leaf_name(file_name);
    let mut file = create_file_component_nofollow(parent, &temp_name)?;
    if let Err(e) = write_and_sync_handle(&mut file, &temp_name, bytes) {
        drop(file);
        let _ = parent.remove_file(&temp_name);
        return Err(e);
    }
    drop(file);

    // Atomic replace of final name (Unix rename; Windows MoveFileEx REPLACE_EXISTING).
    // Replaces a symlink entry if raced; never follows into target content (AC13).
    if let Err(e) = parent.rename(&temp_name, parent, file_name) {
        let _ = parent.remove_file(&temp_name);
        // If final is somehow reparse and rename fails oddly, still no ambient write.
        if leaf_is_reparse(parent, file_name) {
            return Err(CapOpenError::ReparseRefused(file_name.to_string()));
        }
        return Err(map_io_err(e, file_name));
    }
    Ok(())
}

fn write_and_sync_handle(file: &mut File, label: &str, bytes: &[u8]) -> Result<(), CapOpenError> {
    file.write_all(bytes)
        .map_err(|e| CapOpenError::Io(format!("write {label}: {e}")))?;
    file.sync_all()
        .map_err(|e| CapOpenError::Io(format!("sync {label}: {e}")))?;
    Ok(())
}

/// Handle-bound nlink check (AC14 / F10). Prefer over ambient path pre-check alone.
fn refuse_if_hardlink_handle(file: &File, name: &str) -> Result<(), CapOpenError> {
    let nlink = handle_nlink(file, name)?;
    if nlink > 1 {
        return Err(CapOpenError::HardlinkRefused(name.to_string()));
    }
    Ok(())
}

fn handle_nlink(file: &File, name: &str) -> Result<u64, CapOpenError> {
    #[cfg(unix)]
    {
        use cap_std::fs::MetadataExt;
        let meta = file
            .metadata()
            .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
        Ok(meta.nlink())
    }
    #[cfg(windows)]
    {
        // Handle-bound nlink via GetFileInformationByHandle (AC14 / F10).
        // std / cap MetadataExt::number_of_links may be feature-gated; query the handle.
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
        };

        let std_file = file
            .try_clone()
            .map_err(|e| CapOpenError::Io(format!("clone handle {name}: {e}")))?
            .into_std();
        let raw = std_file.as_raw_handle();
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        unsafe {
            GetFileInformationByHandle(HANDLE(raw), &mut info)
                .map_err(|e| CapOpenError::Io(format!("GetFileInformationByHandle {name}: {e}")))?;
        }
        Ok(u64::from(info.nNumberOfLinks))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(CapOpenError::Io(format!(
            "hardlink check unsupported on this platform ({name})"
        )))
    }
}

fn leaf_is_reparse(parent: &Dir, name: &str) -> bool {
    match parent.symlink_metadata(name) {
        Ok(meta) => meta.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Single-component temp name under the same parent (O1).
fn unique_temp_leaf_name(final_name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    // Keep single-component; strip path separators from final_name (already validated).
    format!(".aibw-{pid}-{nanos}-{final_name}.tmp")
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
        std::io::ErrorKind::AlreadyExists => CapOpenError::AlreadyExists(label.to_string()),
        _ if is_symlink_loop_error(&e) => CapOpenError::ReparseRefused(label.to_string()),
        _ => CapOpenError::Io(format!("{label}: {e}")),
    }
}

/// Classify create_new failures: prefer ReparseRefused when leaf is a symlink.
fn map_create_new_err(parent: &Dir, name: &str, e: std::io::Error) -> CapOpenError {
    if is_symlink_loop_error(&e) {
        return CapOpenError::ReparseRefused(name.to_string());
    }
    if let Ok(meta) = parent.symlink_metadata(name) {
        if meta.file_type().is_symlink() {
            return CapOpenError::ReparseRefused(name.to_string());
        }
        // Name exists as non-symlink → AlreadyExists for create_new.
        if e.kind() == std::io::ErrorKind::AlreadyExists || meta.is_file() || meta.is_dir() {
            return CapOpenError::AlreadyExists(name.to_string());
        }
    }
    map_io_err(e, name)
}

/// Post-open checks shared by create_new write handles (F6 / F10).
fn verify_new_write_handle(file: &File, name: &str) -> Result<(), CapOpenError> {
    #[cfg(windows)]
    {
        refuse_if_handle_reparse(file, name)?;
    }
    let meta = file
        .metadata()
        .map_err(|e| CapOpenError::Io(format!("metadata {name}: {e}")))?;
    if meta.is_symlink() {
        return Err(CapOpenError::ReparseRefused(name.to_string()));
    }
    if !meta.is_file() {
        return Err(CapOpenError::NotAFile(name.to_string()));
    }
    refuse_if_hardlink_handle(file, name)?;
    Ok(())
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
        .map_err(|e| map_unix_dir_open_err(parent, name, e))?;
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
        .map_err(|e| map_unix_file_open_err(parent, name, e))?;
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

/// Classify Unix dir-open failures under nofollow.
///
/// Linux often returns **ENOTDIR (20)** for `O_NOFOLLOW|O_DIRECTORY` on a symlink
/// (rather than ELOOP). Probe `symlink_metadata` so F9 still maps to `ReparseRefused`.
#[cfg(unix)]
fn map_unix_dir_open_err(parent: &Dir, name: &str, e: std::io::Error) -> CapOpenError {
    if let Ok(meta) = parent.symlink_metadata(name) {
        if meta.file_type().is_symlink() {
            return CapOpenError::ReparseRefused(name.to_string());
        }
        if !meta.is_dir() {
            return CapOpenError::NotADir(name.to_string());
        }
    }
    map_io_err(e, name)
}

/// Classify Unix file-open failures under nofollow (ELOOP or symlink metadata).
#[cfg(unix)]
fn map_unix_file_open_err(parent: &Dir, name: &str, e: std::io::Error) -> CapOpenError {
    if is_symlink_loop_error(&e) {
        return CapOpenError::ReparseRefused(name.to_string());
    }
    if let Ok(meta) = parent.symlink_metadata(name) {
        if meta.file_type().is_symlink() {
            return CapOpenError::ReparseRefused(name.to_string());
        }
        if !meta.is_file() {
            return CapOpenError::NotAFile(name.to_string());
        }
    }
    map_io_err(e, name)
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

// --- Write create_new implementations (cross-platform SOOT; F27 not windows-only) ---

#[cfg(unix)]
fn create_file_component_nofollow_impl(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use rustix::fs::OFlags;

    let mut opts = nofollow_write_options_create_new();
    // O_NOFOLLOW on final component; O_EXCL implied by create_new.
    opts.custom_flags(OFlags::NOFOLLOW.bits() as i32);
    // Owner-only default for secret-bearing leaves (token/kit); callers may loosen.
    opts.mode(0o600);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_create_new_err(parent, name, e))?;
    verify_new_write_handle(&file, name)?;
    Ok(file)
}

#[cfg(windows)]
fn create_file_component_nofollow_impl(parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    use cap_std::fs::OpenOptionsExt;
    use windows::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

    let mut opts = nofollow_write_options_create_new();
    // OPEN_REPARSE_POINT + CREATE_NEW only — never TRUNCATE_EXISTING (F31).
    opts.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);

    let file = parent
        .open_with(name, &opts)
        .map_err(|e| map_create_new_err(parent, name, e))?;
    verify_new_write_handle(&file, name)?;
    Ok(file)
}

#[cfg(not(any(unix, windows)))]
fn create_file_component_nofollow_impl(_parent: &Dir, name: &str) -> Result<File, CapOpenError> {
    Err(CapOpenError::Io(format!(
        "nofollow file create unsupported on this platform ({name})"
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

    // --- T193 write SOOT ---

    #[test]
    fn write_leaf__create_new_happy__ok() {
        let dir = tempdir().expect("tempdir");
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        write_file_nofollow_leaf(&parent, "secret.txt", b"payload-v1", CreateMode::CreateNew)
            .expect("create_new write");
        let got = fs::read(dir.path().join("secret.txt")).expect("ambient verify read");
        assert_eq!(got, b"payload-v1");
    }

    #[test]
    fn write_leaf__create_new_existing__already_exists() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("exists.txt"), b"old").expect("seed");
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        let err = write_file_nofollow_leaf(&parent, "exists.txt", b"new", CreateMode::CreateNew)
            .expect_err("must fail create_new on existing");
        assert!(
            matches!(err, CapOpenError::AlreadyExists(_) | CapOpenError::Io(_)),
            "expected AlreadyExists, got {err:?}"
        );
        assert_eq!(
            fs::read(dir.path().join("exists.txt")).expect("unchanged"),
            b"old"
        );
    }

    #[test]
    fn write_leaf__create_new_symlink_leaf__refused() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("target.txt");
        fs::write(&target, b"TARGET-BYTES-MUST-STAY").expect("target");
        let link = dir.path().join("link.txt");
        if !create_file_symlink(&target, &link) {
            eprintln!(
                "soft-skip: could not create file symlink (privilege missing). \
                 CreateNew symlink refuse covered when privilege available."
            );
            return;
        }
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        let err = write_file_nofollow_leaf(&parent, "link.txt", b"attacker", CreateMode::CreateNew)
            .expect_err("create_new on symlink must refuse");
        assert!(
            matches!(
                err,
                CapOpenError::ReparseRefused(_) | CapOpenError::AlreadyExists(_)
            ),
            "expected ReparseRefused (or AlreadyExists fail-closed), got {err:?}"
        );
        assert_eq!(
            fs::read(&target).expect("target intact"),
            b"TARGET-BYTES-MUST-STAY",
            "symlink target must not be truncated on CreateNew refuse"
        );
    }

    /// AC13: Replace/force with symlink leaf refuses AND target bytes unchanged.
    #[test]
    fn write_leaf__replace_symlink_leaf__refuses_target_intact() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("real-secret.txt");
        fs::write(&target, b"REAL-CONTENT-DO-NOT-DESTROY").expect("target");
        let link = dir.path().join("leaf.txt");
        if !create_file_symlink(&target, &link) {
            eprintln!(
                "soft-skip: could not create file symlink (privilege missing). \
                 AC13 replace refuse covered when privilege available."
            );
            return;
        }
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        let err = write_file_nofollow_leaf(&parent, "leaf.txt", b"OVERWRITE", CreateMode::Replace)
            .expect_err("replace on symlink must refuse");
        assert!(
            matches!(err, CapOpenError::ReparseRefused(_)),
            "expected ReparseRefused for replace-via-symlink, got {err:?}"
        );
        assert_eq!(
            fs::read(&target).expect("target must survive"),
            b"REAL-CONTENT-DO-NOT-DESTROY",
            "AC13: replace must not truncate/destroy reparse target content"
        );
        // Link path must still be a symlink (not replaced by our write).
        let meta = fs::symlink_metadata(&link).expect("link meta");
        assert!(
            meta.file_type().is_symlink(),
            "symlink leaf must remain after refuse"
        );
    }

    /// AC14: hardlink leaf (nlink>1) refuses via handle metadata.
    #[test]
    fn write_leaf__replace_hardlink_leaf__refused() {
        let dir = tempdir().expect("tempdir");
        let real = dir.path().join("real.txt");
        let link = dir.path().join("hard.txt");
        fs::write(&real, b"shared-inode").expect("seed");
        if let Err(e) = fs::hard_link(&real, &link) {
            eprintln!("soft-skip: hard_link failed: {e}");
            return;
        }
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        let err =
            write_file_nofollow_leaf(&parent, "hard.txt", b"would-clobber", CreateMode::Replace)
                .expect_err("hardlink must refuse");
        assert!(
            matches!(err, CapOpenError::HardlinkRefused(_)),
            "expected HardlinkRefused, got {err:?}"
        );
        assert_eq!(
            fs::read(&real).expect("shared content intact"),
            b"shared-inode"
        );
    }

    #[test]
    fn write_leaf__replace_regular__ok() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("f.txt"), b"v1").expect("seed");
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        write_file_nofollow_leaf(&parent, "f.txt", b"v2-replaced", CreateMode::Replace)
            .expect("replace");
        assert_eq!(
            fs::read(dir.path().join("f.txt")).expect("read"),
            b"v2-replaced"
        );
    }

    #[test]
    fn write_leaf__replace_missing__creates() {
        let dir = tempdir().expect("tempdir");
        let parent = open_ambient_dir(dir.path()).expect("open parent");
        write_file_nofollow_leaf(&parent, "new.txt", b"created", CreateMode::Replace)
            .expect("replace missing = create");
        assert_eq!(
            fs::read(dir.path().join("new.txt")).expect("read"),
            b"created"
        );
    }

    #[test]
    fn write_leaf__open_fail__no_ambient_success_path() {
        let dir = tempdir().expect("tempdir");
        // Parent path that is a file (not a dir): ambient open must fail closed.
        let file_as_parent = dir.path().join("not-a-dir");
        fs::write(&file_as_parent, b"x").expect("seed");
        let err = write_file_nofollow_under_parent_path(
            &file_as_parent,
            "child.txt",
            b"should-not-write",
            CreateMode::CreateNew,
        )
        .expect_err("parent not a dir");
        assert!(
            matches!(err, CapOpenError::NotADir(_) | CapOpenError::Io(_)),
            "expected fail-closed without ambient write, got {err:?}"
        );
        assert!(!dir.path().join("child.txt").exists());
    }

    #[test]
    fn write_leaf__under_parent_path_convenience__ok() {
        let dir = tempdir().expect("tempdir");
        write_file_nofollow_under_parent_path(
            dir.path(),
            "via-path.txt",
            b"convenience",
            CreateMode::CreateNew,
        )
        .expect("convenience");
        assert_eq!(
            fs::read(dir.path().join("via-path.txt")).expect("read"),
            b"convenience"
        );
    }

    #[test]
    fn read_leaf__nofollow_under_parent__ok() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("tok"), b"bearer-value").expect("seed");
        let parent = open_ambient_dir(dir.path()).expect("open");
        let bytes = read_file_nofollow_leaf(&parent, "tok", 4096).expect("read");
        assert_eq!(bytes, b"bearer-value");
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
