//! HTTP bearer token generation, storage, and owner-only ACL.
//!
//! # Windows SDDL freeze (T161)
//!
//! [`USER_TOKEN_FILE_SDDL`] = `D:P(A;;FA;;;OW)` — protected DACL, Full Access for
//! **Owner** only. This is **not** `artifact_security::RESTRICTIVE_FILE_SDDL`
//! (`SY`+`BA`), which is for SYSTEM ProgramData tasks.

use std::path::{Path, PathBuf};

use base64::Engine;
use thiserror::Error;
use zeroize::Zeroizing;

/// Frozen owner-only token file SDDL (Windows).
///
/// - `D:P` — DACL present and protected (no inheritance)
/// - `FA` — full access
/// - `OW` — current owner
pub const USER_TOKEN_FILE_SDDL: &str = "D:P(A;;FA;;;OW)";

/// Token entropy: 32 bytes = 256 bits.
pub const TOKEN_ENTROPY_BYTES: usize = 32;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("home directory unavailable for http.token")]
    NoHomeDir,
    #[error("failed to create token directory {path}: {source}")]
    CreateDir {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write token file {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read token file {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("token file is empty: {path}")]
    EmptyToken { path: String },
    #[error("refusing reparse/symlink path: {0}")]
    Reparse(String),
    #[error("refusing hardlink path: {0}")]
    Hardlink(String),
    #[error("ACL apply/verify failed: {0}")]
    Acl(String),
    #[error("entropy failure: {0}")]
    Entropy(String),
}

/// Default path: `%USERPROFILE%\.ai-brains\http.token` (via `dirs::home_dir`).
pub fn default_token_path() -> Result<PathBuf, TokenError> {
    let mut home = dirs::home_dir().ok_or(TokenError::NoHomeDir)?;
    home.push(".ai-brains");
    home.push("http.token");
    Ok(home)
}

/// Generate a high-entropy opaque token (base64url, no padding).
pub fn generate_token() -> Result<Zeroizing<String>, TokenError> {
    let mut bytes = [0u8; TOKEN_ENTROPY_BYTES];
    rand::fill(&mut bytes);
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    // Zeroize raw entropy.
    bytes.fill(0);
    Ok(Zeroizing::new(encoded))
}

/// Load existing token or create a new one with owner-only ACL.
///
/// On first create, logs the **path** (not the token) to stderr.
pub fn load_or_create_token(path: &Path) -> Result<Zeroizing<String>, TokenError> {
    if path.exists() {
        return load_token(path);
    }
    let token = generate_token()?;
    write_token_file(path, token.as_str())?;
    eprintln!(
        "ai-brains: created HTTP bearer token file at {} (token not printed; read the file for clients)",
        path.display()
    );
    Ok(token)
}

/// Ensure a token exists at the default path (or `override_path`).
pub fn ensure_token(
    override_path: Option<&Path>,
) -> Result<(PathBuf, Zeroizing<String>), TokenError> {
    let path = match override_path {
        Some(p) => p.to_path_buf(),
        None => default_token_path()?,
    };
    let token = load_or_create_token(&path)?;
    Ok((path, token))
}

fn load_token(path: &Path) -> Result<Zeroizing<String>, TokenError> {
    refuse_reparse(path)?;
    let raw = std::fs::read_to_string(path).map_err(|source| TokenError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        return Err(TokenError::EmptyToken {
            path: path.display().to_string(),
        });
    }
    Ok(Zeroizing::new(trimmed))
}

/// Write token with reparse refuse + owner-only ACL (apply-then-verify fail-closed).
pub fn write_token_file(path: &Path, content: &str) -> Result<(), TokenError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        refuse_reparse(parent)?;
        std::fs::create_dir_all(parent).map_err(|source| TokenError::CreateDir {
            path: parent.display().to_string(),
            source,
        })?;
        refuse_reparse(parent)?;
    }

    refuse_reparse(path)?;
    refuse_hardlink(path)?;

    std::fs::write(path, content.as_bytes()).map_err(|source| TokenError::Write {
        path: path.display().to_string(),
        source,
    })?;

    // TOCTOU: re-check reparse after write.
    if let Err(e) = refuse_reparse(path) {
        let _ = std::fs::remove_file(path);
        return Err(e);
    }

    apply_and_verify_owner_acl(path)?;
    Ok(())
}

fn refuse_reparse(path: &Path) -> Result<(), TokenError> {
    let is_reparse = ai_brains_path::is_reparse_or_symlink(path).unwrap_or(false);
    if is_reparse {
        return Err(TokenError::Reparse(format!(
            "refusing reparse/symlink at {}",
            path.display()
        )));
    }
    Ok(())
}

fn refuse_hardlink(path: &Path) -> Result<(), TokenError> {
    #[cfg(windows)]
    {
        if is_hardlink_windows(path).unwrap_or(false) {
            return Err(TokenError::Hardlink(format!(
                "refusing hardlink at {}",
                path.display()
            )));
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = path.symlink_metadata() {
            if meta.nlink() > 1 {
                return Err(TokenError::Hardlink(format!(
                    "refusing hardlink at {}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

fn apply_and_verify_owner_acl(path: &Path) -> Result<(), TokenError> {
    #[cfg(windows)]
    {
        apply_owner_acl_windows(path).map_err(TokenError::Acl)?;
        verify_owner_acl_windows(path).map_err(TokenError::Acl)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(path)
            .map_err(|e| TokenError::Acl(format!("stat for chmod {}: {e}", path.display())))?;
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|e| TokenError::Acl(format!("chmod 0600 {}: {e}", path.display())))?;
        Ok(())
    }
}

#[cfg(windows)]
fn is_hardlink_windows(path: &Path) -> std::io::Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{CloseHandle, GENERIC_READ};
    use windows::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        GetFileInformationByHandle, OPEN_EXISTING,
    };
    use windows::core::PCWSTR;

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
    .map_err(|e| std::io::Error::other(format!("CreateFileW: {e}")))?;

    if handle.is_invalid() {
        return Err(std::io::Error::last_os_error());
    }

    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
    let _ = unsafe { CloseHandle(handle) };
    ok.map_err(|e| std::io::Error::other(format!("GetFileInformationByHandle: {e}")))?;
    Ok(info.nNumberOfLinks > 1)
}

/// Apply absolute owner-only DACL via SDDL + SetNamedSecurityInfo.
#[cfg(windows)]
fn apply_owner_acl_windows(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{ERROR_SUCCESS, HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1, SE_FILE_OBJECT,
        SetNamedSecurityInfoW,
    };
    use windows::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    };
    use windows::core::PCWSTR;

    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let sddl_wide: Vec<u16> = USER_TOKEN_FILE_SDDL
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
            format!("ConvertStringSecurityDescriptor failed for {USER_TOKEN_FILE_SDDL}: {e}")
        })?;
    }

    if psd.0.is_null() {
        return Err("ConvertStringSecurityDescriptor returned null security descriptor".into());
    }

    struct SdGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SdGuard {
        fn drop(&mut self) {
            if !self.0.0.is_null() {
                unsafe {
                    let _ = LocalFree(Some(HLOCAL(self.0.0)));
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
        let _ = std::fs::remove_file(path);
        return Err(format!(
            "SetNamedSecurityInfoW failed for {}: Win32 error {} \
             (could not apply owner-only DACL {USER_TOKEN_FILE_SDDL})",
            path.display(),
            status.0
        ));
    }

    Ok(())
}

/// Verify ACL via `icacls`: must not be SY+BA-only ProgramData style; owner-full present.
#[cfg(windows)]
fn verify_owner_acl_windows(path: &Path) -> Result<(), String> {
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
        return Err(format!(
            "icacls verify query failed for {path_str}: {stdout}{stderr}"
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    verify_owner_acl_output(&stdout)
}

/// Pure helper: accept owner-only posture; reject SY+BA-only ProgramData ACL and broad grants.
///
/// Unit-testable without filesystem.
pub fn verify_owner_acl_output(icacls_stdout: &str) -> Result<(), String> {
    let mut has_owner_or_user_f = false;
    let mut has_system_f = false;
    let mut has_admins_f = false;
    let mut has_users_or_everyone = false;

    for raw_line in icacls_stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("Successfully processed") || !line.contains(':') {
            continue;
        }
        let lower = line.to_ascii_lowercase();

        // Broad principals — never allowed on the token file.
        if lower.contains("everyone")
            || lower.contains("authenticated users")
            || lower.contains("builtin\\users")
            || lower.contains("\\users:(")
        {
            has_users_or_everyone = true;
        }

        // SYSTEM / Administrators markers (ProgramData style).
        if (lower.contains("nt authority\\system") || lower.contains("*s-1-5-18"))
            && (lower.contains("(f)") || lower.contains("(f)"))
        {
            has_system_f = true;
        }
        if (lower.contains("builtin\\administrators") || lower.contains("*s-1-5-32-544"))
            && lower.contains("(f)")
        {
            has_admins_f = true;
        }

        // Owner-style: current user SID or username with (F), or explicit OWNER RIGHTS.
        if (lower.contains("(f)") || lower.contains("(f)"))
            && (lower.contains("owner rights")
                || (!lower.contains("nt authority\\system")
                    && !lower.contains("builtin\\administrators")
                    && !lower.contains("everyone")
                    && !lower.contains("authenticated users")))
        {
            // Heuristic: any non-SY/BA full ACE counts as owner/user.
            if !lower.contains("nt authority\\system") && !lower.contains("builtin\\administrators")
            {
                has_owner_or_user_f = true;
            }
        }
    }

    if has_users_or_everyone {
        return Err(
            "token ACL grants broad Users/Everyone access; expected owner-only D:P(A;;FA;;;OW)"
                .into(),
        );
    }

    // Fail if ACL looks exactly like ProgramData SY+BA and nothing else.
    if has_system_f && has_admins_f && !has_owner_or_user_f {
        return Err(
            "token ACL is SYSTEM+Administrators only (RESTRICTIVE_FILE_SDDL style); \
             expected owner-only USER_TOKEN_FILE_SDDL D:P(A;;FA;;;OW)"
                .into(),
        );
    }

    if !(has_owner_or_user_f || has_system_f || has_admins_f) {
        // Empty or unparseable — fail closed.
        // Note: some locales print the username differently; if we have any (F)
        // ACE at all we already set has_owner_or_user_f. Empty → fail.
        return Err(
            "token ACL missing owner full control; expected D:P(A;;FA;;;OW) posture".into(),
        );
    }

    // Owner-only may still list the resolved username as (F); accept.
    if has_owner_or_user_f {
        return Ok(());
    }

    Err("token ACL verification failed: not owner-only".into())
}
