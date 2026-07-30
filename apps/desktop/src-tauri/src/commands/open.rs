//! Dual-layer safe open: Rust validators + scoped opener capabilities (U3/U20).
//!
//! Frontend must invoke only these commands — no JS `@tauri-apps/plugin-opener`.

use super::InvokeApiError;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Validate that `url` is an absolute `https:` URL.
///
/// Refuses empty input and every non-https scheme (http, mailto, tel, javascript,
/// data, file, vbscript, smb-style, etc.).
pub fn validate_https_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("URL is empty".to_string());
    }

    // Reject whitespace / control characters that can smuggle schemes.
    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("URL must not contain whitespace or control characters".to_string());
    }

    let Some(scheme_end) = trimmed.find(':') else {
        return Err("URL is missing a scheme (only https: is allowed)".to_string());
    };
    if scheme_end == 0 {
        return Err("URL is missing a scheme (only https: is allowed)".to_string());
    }

    let scheme = &trimmed[..scheme_end];
    if !scheme
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.')
    {
        return Err("URL has an invalid scheme".to_string());
    }

    if !scheme.eq_ignore_ascii_case("https") {
        return Err(format!(
            "only https: URLs are allowed; refused scheme '{scheme}'"
        ));
    }

    let rest = &trimmed[scheme_end + 1..];
    if !rest.starts_with("//") {
        return Err("https URL must use hierarchical form https://…".to_string());
    }
    let after_slashes = &rest[2..];
    if after_slashes.is_empty() {
        return Err("https URL is missing a host".to_string());
    }
    // Host ends at first `/`, `?`, or `#`.
    let host = after_slashes.split(['/', '?', '#']).next().unwrap_or("");
    // Strip optional userinfo (user:pass@host) — still require a non-empty host.
    let host = host.rsplit('@').next().unwrap_or(host);
    // Strip port for emptiness check.
    let host_no_port = host.split(':').next().unwrap_or(host);
    if host_no_port.is_empty() {
        return Err("https URL is missing a host".to_string());
    }

    Ok(())
}

/// Validate a filesystem path intended for reveal/open.
///
/// Refuses empty paths, `..` segments, and obvious UNC / device traversal forms.
pub fn validate_reveal_path(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("path is empty".to_string());
    }

    if trimmed.chars().any(|c| c.is_control()) {
        return Err("path must not contain control characters".to_string());
    }

    // Normalize separators for segment inspection only (do not rewrite OS path).
    let normalized = trimmed.replace('\\', "/");

    // Reject `..` path segments in any form.
    for segment in normalized.split('/') {
        if segment == ".." {
            return Err("path must not contain '..' segments".to_string());
        }
    }

    // Extra belt-and-suspenders for adjacent forms the split might miss when
    // mixed with Windows long-path / UNC prefixes.
    if normalized.contains("/../")
        || normalized.starts_with("../")
        || normalized.ends_with("/..")
        || normalized == ".."
    {
        return Err("path must not contain '..' segments".to_string());
    }

    // Refuse device / ambiguous namespace roots that are not normal user paths.
    let lower = normalized.to_ascii_lowercase();
    if lower.starts_with("//./") || lower.starts_with("//?/unc/") || lower.contains("/../") {
        return Err("path uses a disallowed device or UNC traversal form".to_string());
    }

    Ok(())
}

/// Open an https-only URL in the system default browser (dual-layer: validators + plugin scope).
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), InvokeApiError> {
    validate_https_url(&url).map_err(|message| InvokeApiError::error(message, None))?;

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| InvokeApiError::error(format!("failed to open URL: {e}"), None))?;

    Ok(())
}

/// Reveal / open a validated local path via the opener plugin.
#[tauri::command]
pub fn reveal_path(app: AppHandle, path: String) -> Result<(), InvokeApiError> {
    validate_reveal_path(&path).map_err(|message| InvokeApiError::error(message, None))?;

    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| InvokeApiError::error(format!("failed to open path: {e}"), None))?;

    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn validate_https_url__accepts_https() {
        assert!(validate_https_url("https://example.com").is_ok());
        assert!(validate_https_url("HTTPS://Example.COM/path?q=1#frag").is_ok());
        assert!(validate_https_url("https://user:pass@example.com/a").is_ok());
    }

    #[test]
    fn validate_https_url__refuses_non_https_and_empty() {
        for bad in [
            "",
            "   ",
            "http://example.com",
            "mailto:a@b.c",
            "tel:+123",
            "javascript:alert(1)",
            "data:text/html,hi",
            "file:///C:/tmp",
            "vbscript:msgbox",
            "smb://server/share",
            "ftp://example.com",
            "https:",
            "https://",
            "https:///no-host",
            "example.com",
            "https:// example.com",
        ] {
            assert!(
                validate_https_url(bad).is_err(),
                "expected refusal for {bad:?}"
            );
        }
    }

    #[test]
    fn validate_reveal_path__accepts_normal_paths() {
        assert!(validate_reveal_path("C:/dev/AI-Brains/README.md").is_ok());
        assert!(validate_reveal_path(r"C:\Users\me\file.txt").is_ok());
        assert!(validate_reveal_path("/tmp/scope-a.md").is_ok());
        assert!(validate_reveal_path("relative/path/file.md").is_ok());
    }

    #[test]
    fn validate_reveal_path__refuses_empty_and_dotdot() {
        for bad in [
            "",
            "   ",
            "../etc/passwd",
            "foo/../bar",
            r"C:\Users\..\Windows",
            "C:/a/../../b",
            "..",
        ] {
            assert!(
                validate_reveal_path(bad).is_err(),
                "expected refusal for {bad:?}"
            );
        }
    }
}
