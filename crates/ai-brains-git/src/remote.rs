//! Git remote URL selection and privacy-preserving hashing.
//!
//! # Remote URL normalization (`normalize_remote_url`)
//!
//! Equivalent remotes (HTTPS / SSH / SCP-like) of the same host+path must hash
//! identically. The algorithm is deterministic and intentionally drops secrets:
//!
//! 1. **Trim** leading/trailing whitespace.
//! 2. **Trim** trailing `/` characters.
//! 3. **Parse** into `(host, path)`:
//!    - **Scheme URL** (`https://`, `http://`, `ssh://`, `git://`, …):
//!      take the authority after `://`, strip `userinfo@`, drop optional
//!      `:port`, host is the authority host, path is the remainder.
//!    - **SCP-like** (`git@host:path` or `host:path`): recognized only when
//!      there is no `://` and no `/` before the first `:` (git’s SCP heuristic).
//!      Optional `user@` is dropped; host and path are split on the first `:`.
//!    - **Fallback**: if neither form matches, the trimmed string is used as a
//!      path-only token with an empty host (local paths / opaque strings).
//! 4. **Lowercase** the host (ASCII). Path case is preserved.
//! 5. **Strip** a leading `/` from the path.
//! 6. **Strip** a trailing `.git` suffix from the path (ASCII case-insensitive).
//! 7. **Emit** `{host}/{path}` when host is non-empty, else `{path}` alone.
//!    No scheme, no userinfo, no port, no trailing slash or `.git`.
//!
//! Examples (all canonicalize to `github.com/org/repo`):
//! - `https://github.com/org/repo.git`
//! - `https://user:token@GitHub.com/org/repo/`
//! - `git@github.com:org/repo`
//! - `ssh://git@github.com/org/repo`
//!
//! # Multi-remote selection (`read_remote_selection` / `read_remote_url_hash`)
//!
//! 1. Prefer `remote.origin.url` when present.
//! 2. Else if exactly one remote exists, use that remote’s URL.
//! 3. Else (zero remotes, or multiple remotes without origin) → `hash = None`
//!    and `remote_names` lists available remotes for evidence (R-GIT2.3).
//! 4. Empty / whitespace-only remote URL after normalize → no hash.

use crate::command::run_git_timeout;
use crate::errors::{GitError, Result};
use crate::policy::{GitRunOptions, SoftFailPolicy, or_soft_default};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Result of remote selection: optional hash plus names for evidence when ambiguous.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteHashResult {
    pub hash: Option<String>,
    /// All configured remote names (sorted for determinism when multi-remote).
    pub remote_names: Vec<String>,
}

/// Prefer origin; else the sole remote; else `None` hash when ambiguous.
///
/// When multiple remotes exist without origin, returns `None` (use
/// [`read_remote_selection`] for remote-name evidence).
pub fn read_remote_url_hash(root: &Path) -> Result<Option<String>> {
    Ok(read_remote_selection(root)?.hash)
}

/// Remote selection with name evidence for multi-remote / no-origin cases.
pub fn read_remote_selection(root: &Path) -> Result<RemoteHashResult> {
    read_remote_selection_with_options(root, &GitRunOptions::default())
}

/// [`read_remote_selection`] with explicit timeout and soft-fail policy.
pub(crate) fn read_remote_selection_with_options(
    root: &Path,
    opts: &GitRunOptions,
) -> Result<RemoteHashResult> {
    let mut remote_names = match run_git_timeout(root, &["remote"], opts.timeout) {
        Ok(Some(list)) => list
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>(),
        Ok(None) => Vec::new(),
        Err(e) => or_soft_default(Err(e), opts.policy, Vec::new())?,
    };
    remote_names.sort();

    if let Some(url) = git_config_get(root, "remote.origin.url", opts)? {
        return Ok(RemoteHashResult {
            hash: hash_remote_url(&url),
            remote_names,
        });
    }

    if remote_names.len() != 1 {
        // Zero remotes or multi-remote without origin → no hash; names for evidence.
        return Ok(RemoteHashResult {
            hash: None,
            remote_names,
        });
    }

    let config_key = format!("remote.{}.url", remote_names[0]);
    match git_config_get(root, config_key.as_str(), opts)? {
        Some(url) => Ok(RemoteHashResult {
            hash: hash_remote_url(&url),
            remote_names,
        }),
        None => Ok(RemoteHashResult {
            hash: None,
            remote_names,
        }),
    }
}

/// `git config --get` treats exit-nonzero as missing key (CommandFailed).
///
/// Missing keys soft-map to `Ok(None)` under both policies. Timeout / Io still
/// soft-map under Soft and propagate under Strict.
fn git_config_get(root: &Path, key: &str, opts: &GitRunOptions) -> Result<Option<String>> {
    match run_git_timeout(root, &["config", "--get", key], opts.timeout) {
        Ok(v) => Ok(v),
        // git config --get exits 1 when the key is unset — not a hard failure.
        Err(GitError::CommandFailed { .. }) => Ok(None),
        Err(e) => match opts.policy {
            SoftFailPolicy::Soft => Ok(None),
            SoftFailPolicy::Strict => Err(e),
        },
    }
}

/// SHA-256 hex digest of the **normalized** remote URL.
///
/// Returns [`None`] when normalization yields an empty string (blank/whitespace
/// remotes must not create a shared identity key).
pub fn hash_remote_url(url: &str) -> Option<String> {
    let normalized = normalize_remote_url(url);
    if normalized.is_empty() {
        return None;
    }
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    Some(hex::encode(hasher.finalize()))
}

/// Collapse equivalent git remote URL transports into one canonical string.
///
/// See the module-level documentation for the full algorithm.
pub fn normalize_remote_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    let (host, path) = parse_host_path(trimmed);
    let host = host.to_ascii_lowercase();
    let path = strip_leading_slashes(&path);
    let path = strip_trailing_git_suffix(path);

    if host.is_empty() {
        path.to_string()
    } else if path.is_empty() {
        host
    } else {
        format!("{host}/{path}")
    }
}

fn parse_host_path(input: &str) -> (String, String) {
    if let Some(rest) = strip_scheme(input) {
        return parse_authority_url(rest);
    }

    // SCP-like: no slash before the first colon (git’s heuristic).
    if let Some(colon) = input.find(':') {
        let before = &input[..colon];
        if !before.contains('/') {
            let (host, _user) = split_userinfo_host(before);
            let path = input[colon + 1..].to_string();
            return (host, path);
        }
    }

    (String::new(), input.to_string())
}

fn strip_scheme(input: &str) -> Option<&str> {
    let idx = input.find("://")?;
    Some(&input[idx + 3..])
}

fn parse_authority_url(rest: &str) -> (String, String) {
    // rest = [userinfo@]host[:port][/path]
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i + 1..].to_string()),
        None => (rest, String::new()),
    };

    let (host_port, _userinfo) = split_userinfo_host(authority);
    let host = strip_port(&host_port);
    (host, path)
}

/// Split optional `userinfo@` from host (or host:port). Returns (host_part, userinfo).
fn split_userinfo_host(authority: &str) -> (String, Option<String>) {
    // Prefer the last '@' so passwords containing '@' still leave a host.
    match authority.rfind('@') {
        Some(i) => (
            authority[i + 1..].to_string(),
            Some(authority[..i].to_string()),
        ),
        None => (authority.to_string(), None),
    }
}

fn strip_port(host_port: &str) -> String {
    // IPv6 in brackets: [::1]:22
    if host_port.starts_with('[') {
        if let Some(end) = host_port.find(']') {
            return host_port[..=end].to_string();
        }
        return host_port.to_string();
    }
    match host_port.rfind(':') {
        Some(i) => {
            let maybe_port = &host_port[i + 1..];
            if !maybe_port.is_empty() && maybe_port.chars().all(|c| c.is_ascii_digit()) {
                host_port[..i].to_string()
            } else {
                host_port.to_string()
            }
        }
        None => host_port.to_string(),
    }
}

fn strip_leading_slashes(path: &str) -> &str {
    path.trim_start_matches('/')
}

fn strip_trailing_git_suffix(path: &str) -> &str {
    const SUFFIX: &str = ".git";
    if path.len() >= SUFFIX.len() {
        let tail = &path[path.len() - SUFFIX.len()..];
        if tail.eq_ignore_ascii_case(SUFFIX) {
            return &path[..path.len() - SUFFIX.len()];
        }
    }
    path
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn normalize_remote_url__empty_and_whitespace__empty() {
        assert_eq!(normalize_remote_url(""), "");
        assert_eq!(normalize_remote_url("   "), "");
        assert_eq!(normalize_remote_url("///"), "");
    }

    #[test]
    fn hash_remote_url__empty_normalized__none() {
        assert_eq!(hash_remote_url(""), None);
        assert_eq!(hash_remote_url("   "), None);
        assert_eq!(hash_remote_url("///"), None);
    }

    #[test]
    fn normalize_remote_url__preserves_path_case() {
        assert_eq!(
            normalize_remote_url("https://github.com/Org/Repo"),
            "github.com/Org/Repo"
        );
    }
}
