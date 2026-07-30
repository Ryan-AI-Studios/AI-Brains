//! Dual-layer safe open: Rust validators + capability-mirror + scoped opener caps (U3/U20).
//!
//! Frontend must invoke only these commands — no JS `@tauri-apps/plugin-opener`.
//!
//! **Effective dual-layer on the custom-command path:**
//! 1. **Layer 1 — validators** (`validate_https_url` / `validate_reveal_path`): refuse
//!    non-https schemes, empty input, `..` segments, etc.
//! 2. **Layer 2 — capability-mirror** (`url_capability_allows` / `path_capability_allows`):
//!    independent allowlist matching the scoped objects in `capabilities/default.json`.
//!    Plugin IPC scopes alone do **not** gate `OpenerExt` calls from custom commands
//!    (tauri-plugin-opener scopes apply to the plugin's own IPC handlers). This mirror
//!    is the effective second authorization layer for our invoke path.
//!
//! Scoped capability objects remain in `default.json` so plugin IPC (if ever invoked)
//! is also constrained by the same `https://*` / path allow shapes.

use super::InvokeApiError;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

// ---------------------------------------------------------------------------
// Capability-mirror constants — MUST stay in sync with capabilities/default.json
// (enforced by `default_json_*_allows_match_rust_mirror` tests).
// ---------------------------------------------------------------------------

/// Mirrors `opener:allow-open-url` allow entries: `[{ "url": "https://*" }]`.
pub const CAPABILITY_URL_ALLOWS: &[&str] = &["https://*"];

/// Mirrors `opener:allow-open-path` allow entries: `[{ "path": "**" }]`.
///
/// Residual breadth: `"**"` matches any non-empty path string after Layer-1
/// validation. Documented residual — vault locators may live on arbitrary drives.
pub const CAPABILITY_PATH_ALLOWS: &[&str] = &["**"];

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

// ---------------------------------------------------------------------------
// Layer 2 — capability-mirror (independent of Layer-1 validators)
// ---------------------------------------------------------------------------

/// Match a URL against a single capability allow pattern (opener-style).
///
/// Supported forms used by this app:
/// - `https://*` — hierarchical https URLs with a non-empty host (case-insensitive scheme)
/// - exact string equality for other patterns (future-proofing)
fn url_matches_capability_pattern(url: &str, pattern: &str) -> bool {
    let url = url.trim();
    let pattern = pattern.trim();
    if url.is_empty() || pattern.is_empty() {
        return false;
    }

    if pattern == "https://*" {
        // Independent of validate_https_url: re-check scheme + hierarchical form + host.
        let Some(scheme_end) = url.find(':') else {
            return false;
        };
        let scheme = &url[..scheme_end];
        if !scheme.eq_ignore_ascii_case("https") {
            return false;
        }
        let rest = &url[scheme_end + 1..];
        if !rest.starts_with("//") {
            return false;
        }
        let after = &rest[2..];
        if after.is_empty() {
            return false;
        }
        let host = after.split(['/', '?', '#']).next().unwrap_or("");
        let host = host.rsplit('@').next().unwrap_or(host);
        let host_no_port = host.split(':').next().unwrap_or(host);
        return !host_no_port.is_empty();
    }

    // Prefix glob: `https://example.com/*`
    if let Some(prefix) = pattern.strip_suffix('*') {
        return url.starts_with(prefix);
    }

    url == pattern
}

/// Match a path against a single capability allow pattern (opener-style).
///
/// Supported forms used by this app:
/// - `**` — any non-empty path (residual breadth; documented)
/// - `*` — single-segment wildcard (no `/` in matched remainder beyond prefix)
/// - exact equality otherwise
fn path_matches_capability_pattern(path: &str, pattern: &str) -> bool {
    let path = path.trim();
    let pattern = pattern.trim();
    if path.is_empty() || pattern.is_empty() {
        return false;
    }

    if pattern == "**" {
        // Residual breadth: any non-empty path string.
        return true;
    }

    // Normalize separators for pattern compare only.
    let path_n = path.replace('\\', "/");
    let pat_n = pattern.replace('\\', "/");

    if pat_n == "*" {
        // Single path segment only.
        return !path_n.contains('/');
    }

    if let Some(prefix) = pat_n.strip_suffix("/**") {
        return path_n == prefix || path_n.starts_with(&format!("{prefix}/"));
    }

    if let Some(prefix) = pat_n.strip_suffix('*') {
        return path_n.starts_with(prefix);
    }

    path_n == pat_n
}

/// Layer 2 gate for URLs: must match at least one entry in `CAPABILITY_URL_ALLOWS`.
pub fn url_capability_allows(url: &str) -> Result<(), String> {
    if CAPABILITY_URL_ALLOWS
        .iter()
        .any(|pat| url_matches_capability_pattern(url, pat))
    {
        Ok(())
    } else {
        Err(format!(
            "URL is not allowed by capability mirror (allowed: {CAPABILITY_URL_ALLOWS:?})"
        ))
    }
}

/// Layer 2 gate for paths: must match at least one entry in `CAPABILITY_PATH_ALLOWS`.
///
/// With default `"**"`, any non-empty path passes this mirror. Layer 1 still
/// refuses empty / `..` / device forms. Residual breadth is intentional.
pub fn path_capability_allows(path: &str) -> Result<(), String> {
    if CAPABILITY_PATH_ALLOWS
        .iter()
        .any(|pat| path_matches_capability_pattern(path, pat))
    {
        Ok(())
    } else {
        Err(format!(
            "path is not allowed by capability mirror (allowed: {CAPABILITY_PATH_ALLOWS:?})"
        ))
    }
}

/// Open an https-only URL in the system default browser.
///
/// Dual-layer on this custom-command path: Layer1 validators + Layer2 capability-mirror,
/// then `OpenerExt`. Scoped plugin capabilities in `default.json` still constrain plugin IPC.
#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> Result<(), InvokeApiError> {
    validate_https_url(&url).map_err(|message| InvokeApiError::error(message, None))?;
    url_capability_allows(&url).map_err(|message| InvokeApiError::error(message, None))?;

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| InvokeApiError::error(format!("failed to open URL: {e}"), None))?;

    Ok(())
}

/// Reveal / open a validated local path via the opener plugin.
///
/// Dual-layer on this custom-command path: Layer1 validators + Layer2 capability-mirror,
/// then `OpenerExt`.
#[tauri::command]
pub fn reveal_path(app: AppHandle, path: String) -> Result<(), InvokeApiError> {
    validate_reveal_path(&path).map_err(|message| InvokeApiError::error(message, None))?;
    path_capability_allows(&path).map_err(|message| InvokeApiError::error(message, None))?;

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

    #[test]
    fn url_capability_allows__accepts_https_star_pattern() {
        assert!(url_capability_allows("https://example.com").is_ok());
        assert!(url_capability_allows("HTTPS://Example.COM/path?q=1").is_ok());
        assert!(url_capability_allows("https://user:pass@host/x").is_ok());
    }

    #[test]
    fn url_capability_allows__refuses_non_https() {
        for bad in [
            "http://example.com",
            "mailto:a@b.c",
            "file:///C:/tmp",
            "javascript:alert(1)",
            "",
            "https://",
            "https:///no-host",
        ] {
            assert!(
                url_capability_allows(bad).is_err(),
                "capability mirror should refuse {bad:?}"
            );
        }
    }

    #[test]
    fn path_capability_allows__accepts_non_empty_under_starstar() {
        // Default CAPABILITY_PATH_ALLOWS is ["**"] — residual breadth.
        assert!(path_capability_allows("C:/dev/AI-Brains/README.md").is_ok());
        assert!(path_capability_allows(r"C:\Users\me\file.txt").is_ok());
        assert!(path_capability_allows("relative/path.md").is_ok());
        assert!(path_capability_allows("").is_err());
        assert!(path_capability_allows("   ").is_err());
    }

    #[test]
    fn url_matches_capability_pattern__prefix_glob() {
        assert!(url_matches_capability_pattern(
            "https://example.com/a",
            "https://example.com/*"
        ));
        assert!(!url_matches_capability_pattern(
            "https://other.com/a",
            "https://example.com/*"
        ));
    }

    #[test]
    fn path_matches_capability_pattern__starstar_and_prefix() {
        assert!(path_matches_capability_pattern(r"C:\a\b", "**"));
        assert!(path_matches_capability_pattern("C:/vault/x", "C:/vault/**"));
        assert!(!path_matches_capability_pattern("D:/other", "C:/vault/**"));
        assert!(path_matches_capability_pattern("file.md", "*"));
        assert!(!path_matches_capability_pattern("dir/file.md", "*"));
    }

    /// SU18 runtime meaning: default.json allow entries must match Rust mirror constants.
    #[test]
    fn default_json_url_allows_match_rust_mirror() {
        let raw = include_str!("../../capabilities/default.json");
        let value: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("capabilities/default.json must be valid JSON: {e}"),
        };
        let permissions = match value.get("permissions").and_then(|p| p.as_array()) {
            Some(a) => a,
            None => panic!("permissions array required"),
        };

        let mut found_url = false;
        let mut found_path = false;

        for entry in permissions {
            let Some(obj) = entry.as_object() else {
                continue;
            };
            let id = obj.get("identifier").and_then(|v| v.as_str()).unwrap_or("");
            if id == "opener:allow-open-url" {
                found_url = true;
                let allow = match obj.get("allow").and_then(|a| a.as_array()) {
                    Some(a) => a,
                    None => panic!("opener:allow-open-url must have allow array"),
                };
                let urls: Vec<&str> = allow
                    .iter()
                    .filter_map(|item| item.get("url").and_then(|u| u.as_str()))
                    .collect();
                assert_eq!(
                    urls, CAPABILITY_URL_ALLOWS,
                    "default.json url allows must match CAPABILITY_URL_ALLOWS"
                );
            }
            if id == "opener:allow-open-path" {
                found_path = true;
                let allow = match obj.get("allow").and_then(|a| a.as_array()) {
                    Some(a) => a,
                    None => panic!("opener:allow-open-path must have allow array"),
                };
                let paths: Vec<&str> = allow
                    .iter()
                    .filter_map(|item| item.get("path").and_then(|p| p.as_str()))
                    .collect();
                assert_eq!(
                    paths, CAPABILITY_PATH_ALLOWS,
                    "default.json path allows must match CAPABILITY_PATH_ALLOWS"
                );
            }
        }

        assert!(
            found_url,
            "default.json must contain opener:allow-open-url scoped object"
        );
        assert!(
            found_path,
            "default.json must contain opener:allow-open-path scoped object"
        );
    }

    /// Both layers run in sequence: capability mirror is independent of Layer-1 shape.
    #[test]
    fn dual_layer__https_url_passes_both_gates() {
        let url = "https://docs.example.com/path";
        assert!(validate_https_url(url).is_ok());
        assert!(url_capability_allows(url).is_ok());
    }

    #[test]
    fn dual_layer__http_fails_layer1_and_mirror() {
        let url = "http://example.com";
        assert!(validate_https_url(url).is_err());
        assert!(url_capability_allows(url).is_err());
    }
}
