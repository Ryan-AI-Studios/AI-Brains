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
///
/// Uses non-panicking entropy (`SysRng::try_fill_bytes`); maps failure to
/// [`TokenError::Entropy`] (matches `ai-brains-crypto` passphrase path).
pub fn generate_token() -> Result<Zeroizing<String>, TokenError> {
    use rand::TryRng;
    use rand::rngs::SysRng;

    let mut bytes = [0u8; TOKEN_ENTROPY_BYTES];
    SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| TokenError::Entropy(format!("SysRng::try_fill_bytes failed: {e}")))?;
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
    // Fail-closed: re-verify owner ACL on every load so a weakened ACL is not
    // accepted forever after first create (R1-04). Re-apply once if verify fails.
    ensure_owner_acl_on_load(path)?;

    // T193 F15 / AC3: ambient parent Dir once → nofollow open leaf → read handle.
    // Never sole ambient read_to_string after check.
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| TokenError::Read {
            path: path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "token path has no parent directory",
            ),
        })?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TokenError::Read {
            path: path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "token path missing UTF-8 file name",
            ),
        })?;
    const MAX_TOKEN_BYTES: u64 = 16 * 1024;
    let parent_dir =
        ai_brains_path::open_ambient_dir(parent).map_err(|e| map_cap_to_token_read(path, e))?;
    let bytes = ai_brains_path::read_file_nofollow_leaf(&parent_dir, file_name, MAX_TOKEN_BYTES)
        .map_err(|e| map_cap_to_token_read(path, e))?;
    let raw = String::from_utf8(bytes).map_err(|e| TokenError::Read {
        path: path.display().to_string(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;
    let trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        return Err(TokenError::EmptyToken {
            path: path.display().to_string(),
        });
    }
    Ok(Zeroizing::new(trimmed))
}

/// Re-verify owner ACL on load; if weak, re-apply once then re-verify fail-closed.
fn ensure_owner_acl_on_load(path: &Path) -> Result<(), TokenError> {
    #[cfg(windows)]
    {
        if verify_owner_acl_windows(path).is_ok() {
            return Ok(());
        }
        apply_owner_acl_windows(path).map_err(TokenError::Acl)?;
        verify_owner_acl_windows(path).map_err(TokenError::Acl)
    }
    #[cfg(not(windows))]
    {
        // Unix: re-apply 0600 (same as create path).
        apply_and_verify_owner_acl(path)
    }
}

/// Write token with reparse refuse + owner-only ACL (apply-then-verify fail-closed).
///
/// T193 F15 / AC3: SOOT create/replace under ambient parent Dir; no ambient
/// `std::fs::write` success path.
pub fn write_token_file(path: &Path, content: &str) -> Result<(), TokenError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| TokenError::Write {
            path: path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "token path has no parent directory",
            ),
        })?;
    refuse_reparse(parent)?;
    std::fs::create_dir_all(parent).map_err(|source| TokenError::CreateDir {
        path: parent.display().to_string(),
        source,
    })?;
    refuse_reparse(parent)?;

    refuse_reparse(path)?;
    // Ambient hardlink pre-check (defense-in-depth); SOOT also refuses handle nlink>1.
    refuse_hardlink(path)?;

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TokenError::Write {
            path: path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "token path missing UTF-8 file name",
            ),
        })?;
    // Replace if present (regenerate), CreateNew if missing.
    let mode = if path.exists() {
        ai_brains_path::CreateMode::Replace
    } else {
        ai_brains_path::CreateMode::CreateNew
    };
    ai_brains_path::write_file_nofollow_under_parent_path(
        parent,
        file_name,
        content.as_bytes(),
        mode,
    )
    .map_err(|e| map_cap_to_token_write(path, e))?;

    // TOCTOU: re-check reparse after write.
    if let Err(e) = refuse_reparse(path) {
        let _ = std::fs::remove_file(path);
        return Err(e);
    }

    apply_and_verify_owner_acl(path)?;
    Ok(())
}

fn map_cap_to_token_read(path: &Path, e: ai_brains_path::CapOpenError) -> TokenError {
    use ai_brains_path::CapOpenError;
    match e {
        CapOpenError::ReparseRefused(label) => TokenError::Reparse(format!(
            "refusing reparse/symlink at {} ({label})",
            path.display()
        )),
        CapOpenError::HardlinkRefused(label) => {
            TokenError::Hardlink(format!("refusing hardlink at {} ({label})", path.display()))
        }
        CapOpenError::NotFound(_) => TokenError::Read {
            path: path.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()),
        },
        CapOpenError::Oversized { .. } => TokenError::Read {
            path: path.display().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
        },
        other => TokenError::Read {
            path: path.display().to_string(),
            source: std::io::Error::other(other.to_string()),
        },
    }
}

fn map_cap_to_token_write(path: &Path, e: ai_brains_path::CapOpenError) -> TokenError {
    use ai_brains_path::CapOpenError;
    match e {
        CapOpenError::ReparseRefused(label) => TokenError::Reparse(format!(
            "refusing reparse/symlink at {} ({label})",
            path.display()
        )),
        CapOpenError::HardlinkRefused(label) => {
            TokenError::Hardlink(format!("refusing hardlink at {} ({label})", path.display()))
        }
        other => TokenError::Write {
            path: path.display().to_string(),
            source: std::io::Error::other(other.to_string()),
        },
    }
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
        if let Ok(meta) = path.symlink_metadata()
            && meta.nlink() > 1
        {
            return Err(TokenError::Hardlink(format!(
                "refusing hardlink at {}",
                path.display()
            )));
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

/// Pure helper: accept **owner-only** posture; reject SY/BA Full, broad grants, extra ACEs.
///
/// Strict icacls parse (R1-03):
/// - Exactly owner / current-user style Full (`(F)`) is required
/// - `NT AUTHORITY\SYSTEM` / `BUILTIN\Administrators` with Full **must fail** (even with owner)
/// - Everyone / Authenticated Users / BUILTIN\Users / World — never allowed
/// - Unexpected principals with any ACE — fail closed
///
/// Unit-testable without filesystem.
pub fn verify_owner_acl_output(icacls_stdout: &str) -> Result<(), String> {
    let mut has_owner_or_user_f = false;
    let mut owner_principals: Vec<String> = Vec::new();

    for raw_line in icacls_stdout.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("Successfully processed") || !line.contains(':') {
            continue;
        }

        let Some(ace) = extract_ace_segment(line) else {
            continue;
        };
        let principal = principal_from_ace(&ace);
        let rights = rights_from_ace(&ace);
        if principal.is_empty() {
            continue;
        }

        if is_forbidden_broad_principal(&principal) {
            return Err(format!(
                "token ACL grants access to broad principal '{principal}' (rights={rights}); \
                 expected owner-only D:P(A;;FA;;;OW)"
            ));
        }

        // SYSTEM / Administrators Full — never allowed on user token (reject SY+Owner).
        if is_system_principal(&principal) && has_full_control(&rights) {
            return Err(format!(
                "token ACL grants SYSTEM full control ('{principal}'); \
                 expected owner-only D:P(A;;FA;;;OW) (not SY+Owner / not SY+BA)"
            ));
        }
        if is_administrators_principal(&principal) && has_full_control(&rights) {
            return Err(format!(
                "token ACL grants Administrators full control ('{principal}'); \
                 expected owner-only D:P(A;;FA;;;OW)"
            ));
        }

        // Any SYSTEM/BA ACE (even non-F) is unexpected for pure OW posture.
        if is_system_principal(&principal) || is_administrators_principal(&principal) {
            return Err(format!(
                "token ACL includes privileged principal '{principal}' (rights={rights}); \
                 expected owner-only D:P(A;;FA;;;OW)"
            ));
        }

        if has_full_control(&rights) {
            has_owner_or_user_f = true;
            if !owner_principals
                .iter()
                .any(|p| p.eq_ignore_ascii_case(&principal))
            {
                owner_principals.push(principal);
            }
        } else {
            // Non-full ACE on a non-owner principal is unexpected; fail closed.
            return Err(format!(
                "token ACL has unexpected non-full ACE for '{principal}' (rights={rights}); \
                 expected single owner full control only"
            ));
        }
    }

    if !has_owner_or_user_f {
        return Err(
            "token ACL missing owner full control; expected D:P(A;;FA;;;OW) posture".into(),
        );
    }

    // Owner-only: at most one distinct owner/user Full principal (username or OWNER RIGHTS).
    // Two distinct Full principals (e.g. user + Everyone already rejected; user + other) fail.
    if owner_principals.len() > 1 {
        // Allow OWNER RIGHTS alongside a single resolved username (both represent OW).
        let non_owner_rights: Vec<_> = owner_principals
            .iter()
            .filter(|p| !is_owner_rights_principal(p))
            .collect();
        if non_owner_rights.len() > 1 {
            return Err(format!(
                "token ACL has multiple owner Full principals ({}); \
                 expected single owner-only D:P(A;;FA;;;OW)",
                owner_principals.join(", ")
            ));
        }
    }

    Ok(())
}

// --- Pure icacls parse helpers (owner-only posture; mirrored from artifact_security style) ---

fn extract_ace_segment(line: &str) -> Option<String> {
    let rights_marker = line.rfind(":(")?;
    let before = line[..rights_marker].trim_end();
    if before.is_empty() {
        return None;
    }
    let after = &line[rights_marker..];
    let rights_end = after.rfind(')').map(|i| i + 1).unwrap_or(after.len());
    let rights = after[..rights_end].trim();
    if rights.is_empty() {
        return None;
    }
    let principal = principal_before_rights(before)?;
    Some(format!("{principal}{rights}"))
}

fn principal_before_rights(before: &str) -> Option<String> {
    let s = before.trim();
    if s.is_empty() {
        return None;
    }
    if looks_like_windows_path_prefix(s) {
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
    if b.len() >= 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/') && b[0].is_ascii_alphabetic()
    {
        return true;
    }
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

fn is_system_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    p == "S-1-5-18" || p == "SYSTEM" || p == "NT AUTHORITY\\SYSTEM"
}

fn is_administrators_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    p == "S-1-5-32-544" || p == "ADMINISTRATORS" || p == "BUILTIN\\ADMINISTRATORS"
}

fn is_owner_rights_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    p == "OWNER RIGHTS" || p == "S-1-3-4"
}

/// Broad / world / users principals never allowed on the HTTP token file.
fn is_forbidden_broad_principal(principal: &str) -> bool {
    let p = normalize_principal(principal);
    if is_administrators_principal(principal) || is_system_principal(principal) {
        return false; // handled separately (still rejected, but not as "broad")
    }
    p == "EVERYONE"
        || p == "S-1-1-0"
        || p == "WORLD"
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
        || p.contains("AUTHENTICATED USERS")
        || p.contains("\\EVERYONE")
}
