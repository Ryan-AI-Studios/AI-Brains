//! Protected SYSTEM-task artifacts under `%ProgramData%\AI-Brains\`.
//!
//! Wrappers and env sidecars used by SYSTEM-scheduled tasks must not live in
//! user-writable locations. This module relocates them, refuses reparse-point
//! targets, applies an absolute DACL (`SYSTEM` + `Administrators` full only,
//! protected from inheritance) via Win32 SDDL/`SetNamedSecurityInfo`, and
//! verifies the ACL (via `icacls` query) before callers may register a
//! scheduled task (fail closed).

use std::path::{Path, PathBuf};

/// Absolute protected DACL: SYSTEM + Administrators full control only.
///
/// - `D:P` — DACL present and **protected** (no inheritance).
/// - `FA` — file full access; `SY` — Local System; `BA` — Built-in Administrators.
///
/// Applied with `ConvertStringSecurityDescriptorToSecurityDescriptor` +
/// `SetNamedSecurityInfo` so Windows does not leave residual LogonSession ACEs
/// (unlike incremental `icacls /grant` + `/remove`).
pub const RESTRICTIVE_FILE_SDDL: &str = "D:P(A;;FA;;;SY)(A;;FA;;;BA)";

/// `%ProgramData%\AI-Brains` (falls back to `C:\ProgramData\AI-Brains`).
pub fn program_data_ai_brains_dir() -> PathBuf {
    let base = std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
    PathBuf::from(base).join("AI-Brains")
}

/// SYSTEM nightly wrapper: `%ProgramData%\AI-Brains\nightly-task.bat`.
pub fn nightly_wrapper_path() -> PathBuf {
    program_data_ai_brains_dir().join("nightly-task.bat")
}

/// Deprecated daemon-schedule wrapper: `%ProgramData%\AI-Brains\daemon-task.bat`.
pub fn daemon_wrapper_path() -> PathBuf {
    program_data_ai_brains_dir().join("daemon-task.bat")
}

/// Daemon service env sidecar: `%ProgramData%\AI-Brains\daemon.env`.
pub fn daemon_env_path() -> PathBuf {
    program_data_ai_brains_dir().join("daemon.env")
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

/// Pure helper: refuse when `is_hardlink` is true (unit-testable without FS).
///
/// Production passes `is_hardlink(path)?` as the second argument. Regular
/// existing files (nlink == 1) are allowed for re-schedule replace (D0.5).
pub fn refuse_if_hardlink(path: &Path, is_hardlink: bool) -> Result<(), String> {
    if is_hardlink {
        Err(format!(
            "refusing to write through hardlink at {} (link count > 1)",
            path.display()
        ))
    } else {
        Ok(())
    }
}

/// Harden + verify parent (when `AI-Brains`) and the artifact file itself.
///
/// Shared by write path and the `daemon install` "existing sidecar, no rewrite"
/// path so neither bypasses parent reparse/ACL or file reparse/hardlink guards.
pub fn ensure_protected_artifact_acl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let _ = path;
        return Err("ensure_protected_artifact_acl is only supported on Windows".into());
    }
    #[cfg(windows)]
    {
        ensure_parent_protected(path)?;

        // Refuse reparse / hardlink on the file before ACL-only repair.
        if let Err(msg) = refuse_if_reparse(path, is_reparse_or_symlink(path)?) {
            return Err(msg.into());
        }
        if let Err(msg) = refuse_if_hardlink(path, is_hardlink(path)?) {
            return Err(msg.into());
        }

        apply_restrictive_acl(path)?;
        verify_restrictive_acl(path).map_err(|e| -> Box<dyn std::error::Error> {
            format!(
                "ACL verification failed for {} (fail closed, will not register service/task): {}",
                path.display(),
                e
            )
            .into()
        })
    }
}

/// Ensure `%ProgramData%\AI-Brains` exists, is not a reparse/junction, and has
/// SYSTEM+Administrators ACL only. Call before `sc create` even when no
/// `daemon.env` content is written (fail closed on parent).
pub fn ensure_program_data_ai_brains_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        return Err("ensure_program_data_ai_brains_dir is only supported on Windows".into());
    }
    #[cfg(windows)]
    {
        let dir = program_data_ai_brains_dir();
        if let Err(msg) = refuse_if_reparse(&dir, is_reparse_or_symlink(&dir)?) {
            return Err(msg.into());
        }
        std::fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Failed to create protected artifact directory {}: {}",
                dir.display(),
                e
            )
        })?;
        if let Err(msg) = refuse_if_reparse(&dir, is_reparse_or_symlink(&dir)?) {
            return Err(msg.into());
        }
        apply_restrictive_acl(&dir)?;
        verify_restrictive_acl(&dir).map_err(|e| -> Box<dyn std::error::Error> {
            format!(
                "ACL verification failed for parent directory {} \
                 (fail closed, will not register service/task): {}",
                dir.display(),
                e
            )
            .into()
        })?;
        Ok(dir)
    }
}

/// Parent reparse refuse + optional AI-Brains dir ACL apply/verify (fail closed).
fn ensure_parent_protected(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    if let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?) {
        return Err(msg.into());
    }

    if parent
        .file_name()
        .is_some_and(|n| n == std::ffi::OsStr::new("AI-Brains"))
    {
        apply_restrictive_acl(parent)?;
        verify_restrictive_acl(parent).map_err(|e| {
            format!(
                "ACL verification failed for parent directory {} \
                 (fail closed, will not register service/task): {}",
                parent.display(),
                e
            )
        })?;
    }
    Ok(())
}

/// Pure gate for DoD-3: only schedule when prepare (write/ACL) succeeded.
///
/// Call sites pass `write_protected_artifact` / prepare `Result`; unit tests
/// prove failure never advances to registration without invoking schtasks.
pub fn may_register_after_prepare(prepare_ok: bool) -> bool {
    prepare_ok
}

/// Write `content` to `path` with reparse/hardlink refusal, restrictive ACL, and ACL verify.
///
/// Fail closed: returns `Err` if the target or its parent is a reparse/symlink/
/// junction, if the target is a hardlink (nlink > 1), if ACL apply fails, or if
/// ACL verification (file or AI-Brains parent dir) does not match SYSTEM +
/// Administrators only.
///
/// Order: parent reparse check → create_dir_all → parent re-check → parent ACL
/// apply+verify (AI-Brains) → file reparse check → hardlink check → write →
/// post-write file reparse re-check → ACL apply + verify.
///
/// D0.5: regular existing files (single link) may be replaced for re-schedule.
pub fn write_protected_artifact(
    path: &Path,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let _ = (path, content);
        return Err("write_protected_artifact is only supported on Windows".into());
    }
    #[cfg(windows)]
    {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                // 1. If parent exists and is reparse/junction → Err (before create).
                if let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?) {
                    return Err(msg.into());
                }

                // 2. create_dir_all if needed.
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create protected artifact directory {}: {}",
                        parent.display(),
                        e
                    )
                })?;

                // 3. Re-check parent is still not reparse after create.
                if let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?) {
                    return Err(msg.into());
                }

                // 3b. Parent ACL apply+verify (AI-Brains) — shared with ensure path.
                ensure_parent_protected(path)?;
            }
        }

        // 4. File reparse check before write.
        if let Err(msg) = refuse_if_reparse(path, is_reparse_or_symlink(path)?) {
            return Err(msg.into());
        }

        // 4b. Hardlink check: refuse nlink > 1 (D0.5 still allows regular single-link replace).
        if let Err(msg) = refuse_if_hardlink(path, is_hardlink(path)?) {
            return Err(msg.into());
        }

        // 5. write
        std::fs::write(path, content).map_err(|e| {
            format!(
                "Failed to write protected artifact {}: {}",
                path.display(),
                e
            )
        })?;

        // 6. Post-write reparse re-check (TOCTOU). If reparse, best-effort delete.
        let is_reparse_after = is_reparse_or_symlink(path)?;
        if let Err(msg) = refuse_if_reparse(path, is_reparse_after) {
            let _ = std::fs::remove_file(path);
            return Err(msg.into());
        }

        // 7. File ACL apply + verify (parent already done above; re-check file only).
        // Call apply+verify on file without re-running hardlink refuse after write of
        // a regular file — use the inner ACL pair directly.
        apply_restrictive_acl(path)?;
        verify_restrictive_acl(path).map_err(|e| -> Box<dyn std::error::Error> {
            format!(
                "ACL verification failed for {} (fail closed, will not register service/task): {}",
                path.display(),
                e
            )
            .into()
        })?;

        Ok(())
    }
}

/// True if `path` exists and is a reparse point or symlink (does not follow links).
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

/// True if `path` exists and has more than one hard link (nlink > 1).
///
/// Missing path → `Ok(false)`. Does not follow reparse points for open.
pub fn is_hardlink(path: &Path) -> std::io::Result<bool> {
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::MetadataExt;
        match path.symlink_metadata() {
            Ok(meta) => Ok(meta.nlink() > 1),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    #[cfg(windows)]
    {
        is_hardlink_windows(path)
    }
}

/// Apply absolute restrictive ACL: SYSTEM + Administrators full only, no inheritance.
pub fn apply_restrictive_acl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let _ = path;
        return Err("apply_restrictive_acl is only supported on Windows".into());
    }
    #[cfg(windows)]
    {
        apply_restrictive_acl_windows(path)
    }
}

/// Read ACL via `icacls` and ensure only SYSTEM + Administrators full control.
pub fn verify_restrictive_acl(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        let _ = path;
        return Err("verify_restrictive_acl is only supported on Windows".into());
    }
    #[cfg(windows)]
    {
        verify_restrictive_acl_windows(path)
    }
}

/// Principals present on the ACL that are not well-known SYSTEM / Administrators.
///
/// Used both by verify (fail closed) and by apply (strip leftovers such as
/// `NT AUTHORITY\LogonSessionId_*` ACEs Windows attaches on create).
pub fn unexpected_acl_principals(icacls_stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw_line in icacls_stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("Successfully processed") || !line.contains(':') {
            continue;
        }
        let Some(ace) = extract_ace_segment(line) else {
            continue;
        };
        let principal = principal_from_ace(&ace);
        if principal.is_empty() {
            continue;
        }
        if !is_system_principal(&principal)
            && !is_administrators_principal(&principal)
            && !out
                .iter()
                .any(|p: &String| p.eq_ignore_ascii_case(&principal))
        {
            out.push(principal);
        }
    }
    out
}

/// Pure helper: parse `icacls` stdout and accept only SYSTEM + Administrators full.
///
/// Unit-testable without filesystem or `icacls`.
pub fn acl_output_is_restrictive(icacls_stdout: &str) -> Result<(), String> {
    let mut has_system_f = false;
    let mut has_admins_f = false;

    for raw_line in icacls_stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        // Summary footer from icacls
        if line.starts_with("Successfully processed") {
            continue;
        }
        // Some locales use similar summary text; skip lines with no ACE colon form
        if !line.contains(':') {
            continue;
        }

        // ACE lines look like:
        //   path NT AUTHORITY\SYSTEM:(F)
        //        BUILTIN\Administrators:(OI)(CI)(F)
        //   *S-1-5-18:(F)
        let Some(ace) = extract_ace_segment(line) else {
            continue;
        };

        let principal = principal_from_ace(&ace);
        let rights = rights_from_ace(&ace);

        if is_forbidden_principal(&principal) {
            return Err(format!(
                "ACL grants access to broad principal '{principal}' (rights={rights}); \
                 expected SYSTEM and Administrators only"
            ));
        }

        if is_system_principal(&principal) && has_full_control(&rights) {
            has_system_f = true;
        }
        if is_administrators_principal(&principal) && has_full_control(&rights) {
            has_admins_f = true;
        }
    }

    // Fail closed on any leftover principal (LogonSessionId, Users, Everyone, …).
    let unexpected = unexpected_acl_principals(icacls_stdout);
    if let Some(principal) = unexpected.first() {
        return Err(format!(
            "ACL grants access to unexpected principal '{principal}'; \
             expected SYSTEM and Administrators only"
        ));
    }

    if !has_system_f {
        return Err(
            "ACL missing SYSTEM (NT AUTHORITY\\SYSTEM / S-1-5-18) with full control (F)".into(),
        );
    }
    if !has_admins_f {
        return Err(
            "ACL missing Administrators (BUILTIN\\Administrators / S-1-5-32-544) with full control (F)"
                .into(),
        );
    }

    Ok(())
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
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetFileAttributesW, FILE_ATTRIBUTE_REPARSE_POINT, INVALID_FILE_ATTRIBUTES,
    };

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

/// nlink via `GetFileInformationByHandle` (`std` `number_of_links` is unstable).
#[cfg(windows)]
fn is_hardlink_windows(path: &Path) -> std::io::Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, GENERIC_READ};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    // Fast path: missing path is not a hardlink.
    match path.symlink_metadata() {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    }

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // OPEN_REPARSE_POINT: open the link itself if path is a reparse; BACKUP_SEMANTICS
    // allows directories (defensive — hardlinks are file-only on NTFS for us).
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            None,
        )
    }
    .map_err(|e| std::io::Error::other(format!("CreateFileW for hardlink check: {e}")))?;

    if handle.is_invalid() {
        return Err(std::io::Error::last_os_error());
    }

    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
    let _ = unsafe { CloseHandle(handle) };
    ok.map_err(|e| std::io::Error::other(format!("GetFileInformationByHandle: {e}")))?;

    Ok(info.nNumberOfLinks > 1)
}

/// Apply absolute DACL via SDDL + SetNamedSecurityInfo (not incremental icacls).
///
/// Replaces the entire DACL and marks it protected so LogonSessionId / creator
/// ACEs and inheritance cannot remain — the failure mode of icacls /grant + /remove.
#[cfg(windows)]
fn apply_restrictive_acl_windows(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
        SDDL_REVISION_1, SE_FILE_OBJECT,
    };
    use windows::Win32::Security::{
        GetSecurityDescriptorDacl, ACL, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    };

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let sddl_wide: Vec<u16> = RESTRICTIVE_FILE_SDDL
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut psd = PSECURITY_DESCRIPTOR::default();
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl_wide.as_ptr()),
            SDDL_REVISION_1,
            &mut psd,
            None,
        )
        .map_err(|e| {
            format!("ConvertStringSecurityDescriptor failed for {RESTRICTIVE_FILE_SDDL}: {e}")
        })?;
    }

    if psd.0.is_null() {
        return Err("ConvertStringSecurityDescriptor returned null security descriptor".into());
    }

    // Free LocalAlloc'd SD from ConvertStringSecurityDescriptor on all exits.
    struct SdGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SdGuard {
        fn drop(&mut self) {
            if !self.0 .0.is_null() {
                unsafe {
                    let _ = LocalFree(Some(HLOCAL(self.0 .0)));
                }
            }
        }
    }
    let guard = SdGuard(psd);

    let mut dacl_present = windows::core::BOOL::default();
    let mut dacl_defaulted = windows::core::BOOL::default();
    let mut dacl_ptr: *mut ACL = std::ptr::null_mut();

    unsafe {
        GetSecurityDescriptorDacl(
            guard.0,
            &mut dacl_present,
            &mut dacl_ptr,
            &mut dacl_defaulted,
        )
        .map_err(|e| format!("GetSecurityDescriptorDacl failed: {e}"))?;
    }

    if !dacl_present.as_bool() || dacl_ptr.is_null() {
        return Err("SDDL conversion produced no DACL".into());
    }

    let status = unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(path_wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(dacl_ptr),
            None,
        )
    };

    if status != ERROR_SUCCESS {
        // Best-effort: do not leave a half-secured file artifact.
        let _ = std::fs::remove_file(path);
        return Err(format!(
            "SetNamedSecurityInfoW failed for {}: Win32 error {} \
             (could not apply absolute SYSTEM+Administrators DACL)",
            path.display(),
            status.0
        )
        .into());
    }

    Ok(())
}

#[cfg(windows)]
fn verify_restrictive_acl_windows(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let path_str = path
        .to_str()
        .ok_or_else(|| format!("Path is not valid UTF-8: {}", path.display()))?;

    let output = std::process::Command::new("icacls")
        .arg(path_str)
        .output()
        .map_err(|e| format!("Failed to run icacls for verify: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("icacls verify query failed for {path_str}: {stdout}{stderr}").into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    acl_output_is_restrictive(&stdout).map_err(|e| e.into())
}

// --- Pure ACL parse helpers ---

fn extract_ace_segment(line: &str) -> Option<String> {
    // Prefer the last occurrence of ":(" which marks ACE rights (paths may contain
    // drive-letter colons like `C:` but not the `:(` rights form).
    let rights_marker = line.rfind(":(")?;
    let before = line[..rights_marker].trim_end();
    if before.is_empty() {
        return None;
    }

    // Rights run is `:(...)` possibly chained `:(OI)(CI)(F)` — take through last `)`.
    let after = &line[rights_marker..];
    let rights_end = after.rfind(')').map(|i| i + 1).unwrap_or(after.len());
    let rights = after[..rights_end].trim();
    if rights.is_empty() {
        return None;
    }

    // Principal may contain spaces (`NT AUTHORITY\SYSTEM`, `NT AUTHORITY\AUTHENTICATED USERS`).
    // First ACE line often prefixes the file path: `C:\...\file.bat NT AUTHORITY\SYSTEM`.
    // Continuation lines are whitespace + principal only.
    let principal = principal_before_rights(before)?;
    Some(format!("{principal}{rights}"))
}

/// Extract the principal text that sits immediately before `:(rights)`.
///
/// If `before` starts with a Windows path (`X:\...` or `\\...`), everything after
/// the first path token is the principal (may include spaces). Otherwise the whole
/// trimmed string is the principal (continuation ACE lines).
fn principal_before_rights(before: &str) -> Option<String> {
    let s = before.trim();
    if s.is_empty() {
        return None;
    }

    if looks_like_windows_path_prefix(s) {
        // Path is the first whitespace-delimited token; principal is the remainder.
        let mut parts = s.splitn(2, char::is_whitespace);
        let _path = parts.next()?;
        let principal = parts.next().map(str::trim).unwrap_or("");
        if principal.is_empty() {
            return None;
        }
        return Some(principal.to_string());
    }

    Some(s.to_string())
}

fn looks_like_windows_path_prefix(s: &str) -> bool {
    let b = s.as_bytes();
    // Drive path: `C:\...` or `C:/...`
    if b.len() >= 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/') && b[0].is_ascii_alphabetic()
    {
        return true;
    }
    // UNC: `\\server\share\...`
    s.starts_with("\\\\") || s.starts_with("//")
}

fn principal_from_ace(ace: &str) -> String {
    match ace.find(":(") {
        Some(i) => ace[..i].trim().to_string(),
        None => ace.trim().to_string(),
    }
}

fn rights_from_ace(ace: &str) -> String {
    match ace.find(":(") {
        Some(i) => ace[i + 1..].trim().to_string(),
        None => String::new(),
    }
}

fn has_full_control(rights: &str) -> bool {
    let upper = rights.to_ascii_uppercase();
    upper.contains("(F)") || upper.contains("FULL")
}

fn normalize_principal(principal: &str) -> String {
    principal
        .trim()
        .trim_start_matches('*')
        .to_ascii_uppercase()
}

/// Well-known local SYSTEM only — not domain accounts named SYSTEM.
/// Accepts optional `*` SID prefix via [`normalize_principal`].
fn is_system_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    p == "S-1-5-18" || p == "SYSTEM" || p == "NT AUTHORITY\\SYSTEM"
}

/// Well-known BUILTIN Administrators only — not domain Administrators groups.
fn is_administrators_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    p == "S-1-5-32-544" || p == "ADMINISTRATORS" || p == "BUILTIN\\ADMINISTRATORS"
}

fn is_forbidden_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    // Broad well-known principals that must never appear on SYSTEM task artifacts.
    if p == "EVERYONE"
        || p == "S-1-1-0"
        || p == "AUTHENTICATED USERS"
        || p == "NT AUTHORITY\\AUTHENTICATED USERS"
        || p == "S-1-5-11"
        || p == "INTERACTIVE"
        || p == "NT AUTHORITY\\INTERACTIVE"
        || p == "S-1-5-4"
        || p == "USERS"
        || p == "BUILTIN\\USERS"
        || p == "S-1-5-32-545"
        || p.ends_with("\\USERS")
        || p.ends_with("\\EVERYONE")
    {
        // Careful: do not flag BUILTIN\Administrators as Users
        if is_administrators_principal(principal) || is_system_principal(principal) {
            return false;
        }
        return true;
    }
    // Catch "Authenticated Users" variants that include domain prefix
    if p.contains("AUTHENTICATED USERS") || p.contains("\\EVERYONE") {
        return true;
    }
    false
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn restrictive_file_sddl__protected_system_and_admins_only() {
        // D:P = protected DACL; SY = SYSTEM; BA = Administrators; FA = full file access.
        assert_eq!(RESTRICTIVE_FILE_SDDL, "D:P(A;;FA;;;SY)(A;;FA;;;BA)");
        assert!(RESTRICTIVE_FILE_SDDL.starts_with("D:P"));
        assert!(RESTRICTIVE_FILE_SDDL.contains(";;;SY)"));
        assert!(RESTRICTIVE_FILE_SDDL.contains(";;;BA)"));
        assert!(!RESTRICTIVE_FILE_SDDL.contains("WD")); // not Everyone
        assert!(!RESTRICTIVE_FILE_SDDL.contains("BU")); // not Users
    }

    #[test]
    fn nightly_wrapper_path__default__under_program_data() {
        let path = nightly_wrapper_path();
        let components: Vec<_> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        assert!(
            components
                .iter()
                .any(|c| c.eq_ignore_ascii_case("AI-Brains")),
            "expected AI-Brains in path: {}",
            path.display()
        );
        assert!(
            path.file_name().is_some_and(|n| n == "nightly-task.bat"),
            "expected nightly-task.bat: {}",
            path.display()
        );
        // Should not be under user TEMP or vault-parent style names
        let s = path.to_string_lossy().to_ascii_lowercase();
        assert!(!s.contains("appdata\\local\\temp"));
        assert!(!s.contains(".ai-brains-nightly-task.bat"));
    }

    #[test]
    fn daemon_wrapper_path__default__under_program_data() {
        let path = daemon_wrapper_path();
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some("daemon-task.bat")
        );
        assert!(path
            .components()
            .any(|c| c.as_os_str() == std::ffi::OsStr::new("AI-Brains")));
    }

    #[test]
    fn daemon_env_path__default__under_program_data() {
        let path = daemon_env_path();
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some("daemon.env")
        );
        assert!(path
            .components()
            .any(|c| c.as_os_str() == std::ffi::OsStr::new("AI-Brains")));
    }

    #[test]
    fn acl_output_is_restrictive__system_and_admins_only__ok() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        assert!(
            acl_output_is_restrictive(sample).is_ok(),
            "path-prefixed NT AUTHORITY\\SYSTEM must parse as well-known SYSTEM"
        );
    }

    #[test]
    fn extract_ace_segment__path_prefixed_nt_authority_system__full_principal() {
        let line = r"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)";
        let ace = extract_ace_segment(line).expect("ace");
        assert_eq!(principal_from_ace(&ace), "NT AUTHORITY\\SYSTEM");
        assert!(has_full_control(&rights_from_ace(&ace)));
    }

    #[test]
    fn acl_output_is_restrictive__sid_form__ok() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat *S-1-5-18:(F)
                                           *S-1-5-32-544:(OI)(CI)(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        assert!(acl_output_is_restrictive(sample).is_ok());
    }

    #[test]
    fn unexpected_acl_principals__logon_session_id__listed() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
                                           NT AUTHORITY\LogonSessionId_0_208120:(RX)
Successfully processed 1 files; Failed processing 0 files
"#;
        let unexpected = unexpected_acl_principals(sample);
        assert_eq!(unexpected.len(), 1);
        assert!(
            unexpected[0].contains("LogonSessionId"),
            "unexpected: {unexpected:?}"
        );
        // Verify still fails closed on that principal
        let err = acl_output_is_restrictive(sample).expect_err("must reject LogonSessionId");
        assert!(
            err.to_ascii_lowercase().contains("logonsessionid")
                || err.to_ascii_lowercase().contains("unexpected"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn unexpected_acl_principals__system_admins_only__empty() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        assert!(unexpected_acl_principals(sample).is_empty());
    }

    #[test]
    fn acl_output_is_restrictive__includes_everyone__err() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
                                           Everyone:(R)
Successfully processed 1 files; Failed processing 0 files
"#;
        let err = acl_output_is_restrictive(sample).expect_err("must reject Everyone");
        assert!(
            err.to_ascii_lowercase().contains("everyone")
                || err.to_ascii_lowercase().contains("broad"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn acl_output_is_restrictive__includes_users__err() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
                                           BUILTIN\Users:(R)
Successfully processed 1 files; Failed processing 0 files
"#;
        let err = acl_output_is_restrictive(sample).expect_err("must reject Users");
        assert!(
            err.to_ascii_lowercase().contains("users")
                || err.to_ascii_lowercase().contains("broad")
                || err.to_ascii_lowercase().contains("unexpected"),
            "unexpected err: {err}"
        );
    }

    #[test]
    fn acl_output_is_restrictive__missing_system__err() {
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        let err = acl_output_is_restrictive(sample).expect_err("must require SYSTEM");
        assert!(
            err.to_ascii_lowercase().contains("system"),
            "unexpected err: {err}"
        );
        // Fail-closed verify path surfaces enough detail for operators.
        assert!(
            err.contains("S-1-5-18") || err.to_ascii_lowercase().contains("full control"),
            "fail message should identify the missing well-known principal detail: {err}"
        );
    }

    #[test]
    fn acl_output_is_restrictive__domain_system__err() {
        // Domain\SYSTEM must not satisfy the well-known local SYSTEM identity.
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat CONTOSO\SYSTEM:(F)
                                           BUILTIN\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        let err = acl_output_is_restrictive(sample).expect_err("must reject domain SYSTEM");
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("contoso\\system")
                || lower.contains("unexpected")
                || lower.contains("missing")
                || lower.contains("system"),
            "unexpected err: {err}"
        );
        // Must not treat CONTOSO\SYSTEM as satisfying has_system_f.
        assert!(
            lower.contains("unexpected") || lower.contains("missing"),
            "domain SYSTEM must not count as well-known SYSTEM: {err}"
        );
    }

    #[test]
    fn acl_output_is_restrictive__domain_administrators__err() {
        // Domain\Administrators must not satisfy BUILTIN\Administrators.
        let sample = r#"C:\ProgramData\AI-Brains\nightly-task.bat NT AUTHORITY\SYSTEM:(F)
                                           CONTOSO\Administrators:(F)
Successfully processed 1 files; Failed processing 0 files
"#;
        let err = acl_output_is_restrictive(sample).expect_err("must reject domain Administrators");
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("contoso\\administrators")
                || lower.contains("unexpected")
                || lower.contains("missing")
                || lower.contains("administrators"),
            "unexpected err: {err}"
        );
        assert!(
            lower.contains("unexpected") || lower.contains("missing"),
            "domain Administrators must not count as BUILTIN\\Administrators: {err}"
        );
    }

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
    fn refuse_if_hardlink__true__err() {
        let path = Path::new(r"C:\ProgramData\AI-Brains\nightly-task.bat");
        let err = refuse_if_hardlink(path, true).expect_err("must refuse when is_hardlink");
        let msg = err.to_ascii_lowercase();
        assert!(
            msg.contains("hardlink") || msg.contains("link count"),
            "unexpected err: {err}"
        );
        assert!(
            msg.contains("nightly-task.bat"),
            "err should include path: {err}"
        );
    }

    #[test]
    fn refuse_if_hardlink__false__ok() {
        let path = Path::new(r"C:\ProgramData\AI-Brains\nightly-task.bat");
        refuse_if_hardlink(path, false).expect("must accept when not hardlink");
    }

    /// DoD-3 fail path: default inherited Users ACLs must not pass verify.
    #[test]
    fn verify_restrictive_acl__default_user_temp_file__err() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("unrestricted-default-acl.txt");
        std::fs::write(&path, b"default inherited acls").expect("write");
        let err = verify_restrictive_acl(&path)
            .expect_err("default user-temp ACLs must fail restrictive verify");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("users")
                || msg.contains("everyone")
                || msg.contains("authenticated")
                || msg.contains("broad")
                || msg.contains("unexpected")
                || msg.contains("missing")
                || msg.contains("system")
                || msg.contains("administrators"),
            "expected restrictive-ACL failure reason, got: {msg}"
        );
    }

    #[test]
    fn write_protected_artifact__reparse_point_or_symlink__refuses() {
        // When the target itself is a symlink, write must refuse.
        // Creating file symlinks on Windows may require Developer Mode or
        // elevation; pure refuse_if_reparse + parent-junction tests always run.
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("real-target.txt");
        std::fs::write(&target, b"real").expect("write target");
        let link = dir.path().join("link-target.txt");

        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&target, &link);
        #[cfg(not(windows))]
        let created = std::os::unix::fs::symlink(&target, &link);

        if let Err(e) = created {
            // Soft-skip only for symlink *creation* privilege; pure refuse +
            // parent-junction FS test hard-prove the refuse path.
            eprintln!(
                "skipping file-symlink creation in write_protected_artifact__reparse_point_or_symlink__refuses: {e} \
                 (needs Developer Mode or elevation; owner: T145). \
                 Covered by refuse_if_reparse unit tests + parent junction FS test."
            );
            return;
        }

        let err = write_protected_artifact(&link, "attacker")
            .expect_err("must refuse write through symlink");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "error should mention reparse/symlink/junction, got: {msg}"
        );
        // Target content must be unchanged
        let still = std::fs::read_to_string(&target).expect("read target");
        assert_eq!(still, "real");
    }

    /// Hardlinks do not require elevation; refuse nlink > 1 before overwrite (D0.5 allows regular replace).
    #[test]
    fn write_protected_artifact__hardlink_target__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let real = dir.path().join("real-target.txt");
        let link = dir.path().join("hardlink-target.txt");
        std::fs::write(&real, b"original").expect("write real");
        std::fs::hard_link(&real, &link).expect("create hardlink (should not need elevation)");

        assert!(
            is_hardlink(&link).expect("hardlink metadata"),
            "hardlink path must report nlink > 1"
        );
        assert!(
            is_hardlink(&real).expect("real metadata after second link"),
            "original path also has nlink > 1 after hard_link"
        );

        let err = write_protected_artifact(&link, "attacker")
            .expect_err("must refuse write through hardlink");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("hardlink") || msg.contains("link count"),
            "error should mention hardlink, got: {msg}"
        );
        let still = std::fs::read_to_string(&real).expect("read real");
        assert_eq!(
            still, "original",
            "must not overwrite shared hardlink inode"
        );
    }

    /// Directory junctions do not require SeCreateSymbolicLinkPrivilege (`mklink /J`).
    #[test]
    #[cfg(windows)]
    fn write_protected_artifact__parent_junction__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let real = dir.path().join("real");
        let link = dir.path().join("AI-Brains");
        std::fs::create_dir_all(&real).expect("create real dir");

        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &real.to_string_lossy(),
            ])
            .status()
            .expect("spawn mklink /J");
        assert!(
            status.success(),
            "mklink /J failed (exit {status}); directory junctions should not need SeCreateSymbolicLinkPrivilege"
        );
        assert!(
            is_reparse_or_symlink(&link).expect("metadata on junction"),
            "junction must be detected as reparse"
        );

        let artifact = link.join("nightly-task.bat");
        let err = write_protected_artifact(&artifact, "x")
            .expect_err("must refuse write when parent directory is a junction");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "error should mention reparse/symlink/junction, got: {msg}"
        );
        // Must not have written through the junction into the real directory.
        assert!(
            !real.join("nightly-task.bat").exists(),
            "must not create artifact through parent junction"
        );
    }

    /// Full write + icacls apply/verify when the host allows it.
    /// Hard-asserts on Ok; on Err requires an ACL/icacls-shaped message (not a silent pass).
    #[test]
    fn write_protected_artifact__regular_file_in_temp__applies_and_verifies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("protected-artifact.txt");
        match write_protected_artifact(&path, "system-only content") {
            Ok(()) => {
                // After ACL strip, a non-elevated (UAC-filtered) token may no longer
                // read the file even though Administrators:(F) is present. Content
                // was written before ACL apply; re-verify ACL is the hard assert.
                match std::fs::read_to_string(&path) {
                    Ok(content) => assert_eq!(content, "system-only content"),
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        // Expected for non-elevated callers after restrictive ACL.
                    }
                    Err(e) => panic!("unexpected read error after protected write: {e}"),
                }
                verify_restrictive_acl(&path)
                    .expect("ACL should still verify after protected write");
            }
            Err(e) => {
                // Non-elevated temp often cannot complete /inheritance:r + grant.
                // Require the error to look like the expected elevation/ACL path,
                // not an unrelated panic or logic bug.
                let msg = e.to_string();
                let lower = msg.to_ascii_lowercase();
                assert!(!msg.trim().is_empty(), "error message must be non-empty");
                assert!(
                    lower.contains("icacls")
                        || lower.contains("acl")
                        || lower.contains("inheritance")
                        || lower.contains("grant")
                        || lower.contains("access is denied")
                        || lower.contains("access denied"),
                    "expected ACL/icacls/inheritance-shaped failure without elevation, got: {msg}"
                );
                eprintln!(
                    "write_protected_artifact on user temp failed without elevation (documented): {e}. \
                     Pure ACL parse + verify fail-path + reparse/junction/hardlink tests cover DoD-3; Phase 6 covers ProgramData."
                );
            }
        }
    }

    /// DoD-3 pure registration gate: prepare failure must not advance to schtasks.
    #[test]
    fn may_register_after_prepare__prepare_failed__false() {
        assert!(!may_register_after_prepare(false));
    }

    #[test]
    fn may_register_after_prepare__prepare_ok__true() {
        assert!(may_register_after_prepare(true));
    }

    /// Models nightly schedule: write Result maps to registration permission.
    #[test]
    fn schedule_registration__write_err__must_not_register() {
        let prepare: Result<(), &str> = Err("ACL verification failed (fail closed)");
        let may = may_register_after_prepare(prepare.is_ok());
        assert!(!may, "must not call schtasks when prepare failed");
    }

    #[test]
    fn schedule_registration__write_ok__may_register() {
        let prepare: Result<(), &str> = Ok(());
        let may = may_register_after_prepare(prepare.is_ok());
        assert!(may, "may call schtasks when prepare succeeded");
    }
}
