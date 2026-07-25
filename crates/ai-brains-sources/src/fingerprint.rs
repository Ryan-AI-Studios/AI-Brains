//! Deterministic source fingerprints (T149).
//!
//! ## Formats
//!
//! | Kind | Input | Output shape |
//! |------|--------|--------------|
//! | File / Markdown / Obsidian | `id={identity}\n{normalized_utf8}\n` → SHA-256 | `v{n}:{hex}` |
//! | Git | canonical metadata bytes (see `git_fingerprint`) | `v{n}:{hex}` |
//! | Ledgerful | bridge hash / lineage JSON when present; else identity+content | `ledgerful:{hash}` or `v{n}:{hex}` |
//! | External (Manual / Other / …) | `etag:{value}` / `revision:{value}` when authoritative; else identity+payload | `v{n}:etag:{value}` / `v{n}:revision:{value}` / `v{n}:{hex}` |
//!
//! Pure content hash (no identity): [`fingerprint_bytes`].

use crate::normalization::NORMALIZER_VERSION;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourcesError {
    #[error("invalid UTF-8 content: {detail}")]
    InvalidUtf8 { detail: String },

    #[error("git metadata error: {0}")]
    Git(#[from] ai_brains_git::GitError),
}

/// Pure SHA-256 fingerprint over `content` bytes with versioned prefix.
///
/// Format: `v{NORMALIZER_VERSION}:{lowercase_hex}`.
pub fn fingerprint_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    format!("v{NORMALIZER_VERSION}:{}", hex::encode(digest))
}

/// Build the canonical file-fingerprint preimage: identity + normalized content.
///
/// Spec §3.3: SHA-256 of normalized UTF-8 **+** canonical source identity string.
pub fn file_fingerprint_preimage(identity: &str, normalized_content: &[u8]) -> Vec<u8> {
    let mut preimage = Vec::with_capacity(identity.len() + normalized_content.len() + 8);
    preimage.extend_from_slice(b"id=");
    preimage.extend_from_slice(identity.as_bytes());
    preimage.push(b'\n');
    preimage.extend_from_slice(normalized_content);
    preimage.push(b'\n');
    preimage
}

/// Fingerprint normalized file/markdown content folded with canonical source identity.
///
/// `identity` is the stable source key (typically scope+kind+locator).
pub fn fingerprint_file_with_identity(
    identity: &str,
    bytes: &[u8],
) -> Result<String, SourcesError> {
    let normalized = crate::normalize_file_bytes(bytes)?;
    let preimage = file_fingerprint_preimage(identity, &normalized);
    Ok(fingerprint_bytes(&preimage))
}

/// Ledgerful fingerprint algorithm.
///
/// Prefer an authoritative bridge hash / lineage when content is JSON with a
/// recognized field (`bridge_hash`, `lineage_hash`, `hash`, `parent_hash`);
/// otherwise hash identity + normalized content like files.
///
/// Authoritative form: `ledgerful:{value}` (value is the provided hash string).
pub fn fingerprint_ledgerful(identity: &str, bytes: &[u8]) -> Result<String, SourcesError> {
    if let Some(authoritative) = extract_ledgerful_bridge_hash(bytes) {
        return Ok(format!("ledgerful:{authoritative}"));
    }
    let normalized = crate::normalize_file_bytes(bytes).unwrap_or_else(|_| bytes.to_vec());
    let preimage = file_fingerprint_preimage(identity, &normalized);
    Ok(fingerprint_bytes(&preimage))
}

/// External / connector fingerprint algorithm (Manual, Other, Hermes, Honcho, …).
///
/// - Content that is an ETag line (`etag:…` / `ETag: …`) → `v{n}:etag:{value}`
/// - Content that is a revision line (`revision:…` / `rev:…`) → `v{n}:revision:{value}`
/// - JSON object/array payload → identity + **canonical** JSON (sorted keys) → `v{n}:{hex}`
/// - Otherwise identity + BOM/newline-normalized payload → `v{n}:{hex}`
pub fn fingerprint_external(identity: &str, bytes: &[u8]) -> Result<String, SourcesError> {
    if let Some(etag) = extract_authoritative_etag(bytes) {
        return Ok(format!("v{NORMALIZER_VERSION}:etag:{etag}"));
    }
    if let Some(rev) = extract_authoritative_revision(bytes) {
        return Ok(format!("v{NORMALIZER_VERSION}:revision:{rev}"));
    }
    // Canonical JSON for object/array so key order does not change fingerprints.
    let normalized = if let Some(canonical) = canonicalize_json_payload(bytes) {
        canonical
    } else {
        crate::normalize_file_bytes(bytes).unwrap_or_else(|_| bytes.to_vec())
    };
    let preimage = file_fingerprint_preimage(identity, &normalized);
    Ok(fingerprint_bytes(&preimage))
}

/// If `bytes` parse as a JSON object or array, return a deterministic serialization
/// with object keys sorted recursively. Scalars / non-JSON → `None`.
fn canonicalize_json_payload(bytes: &[u8]) -> Option<Vec<u8>> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    match &value {
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            let canonical = canonicalize_json_value(&value);
            serde_json::to_vec(&canonical).ok()
        }
        _ => None,
    }
}

fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = serde_json::Map::with_capacity(map.len());
            for k in keys {
                if let Some(v) = map.get(k) {
                    out.insert(k.clone(), canonicalize_json_value(v));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize_json_value).collect())
        }
        other => other.clone(),
    }
}

fn extract_ledgerful_bridge_hash(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?.trim();
    // Plain non-JSON hash line: "bridge_hash=<hex>" or "lineage:<hex>"
    if let Some(rest) = text
        .strip_prefix("bridge_hash=")
        .or_else(|| text.strip_prefix("lineage_hash="))
        .or_else(|| text.strip_prefix("lineage:"))
    {
        let v = rest.trim();
        if !v.is_empty() && !v.contains(|c: char| c.is_whitespace()) {
            return Some(v.to_string());
        }
    }

    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let obj = value.as_object()?;
    for key in ["bridge_hash", "lineage_hash", "hash", "parent_hash"] {
        if let Some(v) = obj.get(key).and_then(|x| x.as_str()) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn extract_authoritative_etag(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?.trim();
    // Single-line authoritative forms only (not multi-line documents).
    if text.contains('\n') {
        return None;
    }
    if text.len() < 5 || !text[..5].eq_ignore_ascii_case("etag:") {
        return None;
    }
    let v = text[5..].trim().trim_matches('"');
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

fn extract_authoritative_revision(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?.trim();
    if text.contains('\n') {
        return None;
    }
    for prefix in ["revision:", "rev:"] {
        if text.len() >= prefix.len() && text[..prefix.len()].eq_ignore_ascii_case(prefix) {
            let v = text[prefix.len()..].trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Fingerprinter implementing the control-plane port shape: pure hash of bytes.
///
/// Per-kind canonicalization (file normalize + identity, git fold, ledgerful/external)
/// happens *before* or via the specialized helpers on this type.
#[derive(Debug, Default, Clone, Copy)]
pub struct Sha256Fingerprinter;

impl Sha256Fingerprinter {
    pub fn new() -> Self {
        Self
    }

    /// Deterministic fingerprint of raw content bytes (no identity fold).
    pub fn fingerprint(&self, content: &[u8]) -> String {
        fingerprint_bytes(content)
    }

    /// Fingerprint normalized file/markdown content **with** canonical source identity.
    pub fn fingerprint_file(&self, identity: &str, bytes: &[u8]) -> Result<String, SourcesError> {
        fingerprint_file_with_identity(identity, bytes)
    }

    /// Ledgerful kind fingerprint (bridge hash preferred).
    pub fn fingerprint_ledgerful(
        &self,
        identity: &str,
        bytes: &[u8],
    ) -> Result<String, SourcesError> {
        fingerprint_ledgerful(identity, bytes)
    }

    /// External/connector fingerprint (etag/revision preferred).
    pub fn fingerprint_external(
        &self,
        identity: &str,
        bytes: &[u8],
    ) -> Result<String, SourcesError> {
        fingerprint_external(identity, bytes)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::NORMALIZER_VERSION;

    #[test]
    fn fingerprint_bytes__format_prefix_and_hex_length() {
        let fp = fingerprint_bytes(b"hello");
        let prefix = format!("v{NORMALIZER_VERSION}:");
        assert!(fp.starts_with(&prefix));
        let hex_part = &fp[prefix.len()..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn fingerprint_bytes__identical_input__identical_output() {
        assert_eq!(fingerprint_bytes(b"same"), fingerprint_bytes(b"same"));
    }

    #[test]
    fn fingerprint_file__same_bytes_different_identity__different_fingerprint() {
        let fp = Sha256Fingerprinter::new();
        let a = fp
            .fingerprint_file("scope:a|File|/x.md", b"same body\n")
            .unwrap();
        let b = fp
            .fingerprint_file("scope:b|File|/x.md", b"same body\n")
            .unwrap();
        assert_ne!(a, b);
        assert_eq!(
            fp.fingerprint_file("scope:a|File|/x.md", b"same body\n")
                .unwrap(),
            a
        );
    }

    #[test]
    fn fingerprint_ledgerful__bridge_hash_json__authoritative() {
        let body = br#"{"bridge_hash":"abc123def","note":"ignored"}"#;
        let fp = fingerprint_ledgerful("id1", body).unwrap();
        assert_eq!(fp, "ledgerful:abc123def");
        // Same hash, different identity → still authoritative (identity not folded).
        assert_eq!(fingerprint_ledgerful("id2", body).unwrap(), fp);
    }

    #[test]
    fn fingerprint_ledgerful__lineage_line__authoritative() {
        let fp = fingerprint_ledgerful("id", b"lineage:deadbeef").unwrap();
        assert_eq!(fp, "ledgerful:deadbeef");
    }

    #[test]
    fn fingerprint_ledgerful__plain_content__falls_back_with_identity() {
        let a = fingerprint_ledgerful("src-a", b"plain ledger text\n").unwrap();
        let b = fingerprint_ledgerful("src-b", b"plain ledger text\n").unwrap();
        assert_ne!(a, b);
        assert!(a.starts_with(&format!("v{NORMALIZER_VERSION}:")));
    }

    #[test]
    fn fingerprint_external__etag__authoritative() {
        let fp = fingerprint_external("ext1", b"ETag: \"abc123\"").unwrap();
        assert_eq!(fp, format!("v{NORMALIZER_VERSION}:etag:abc123"));
    }

    #[test]
    fn fingerprint_external__revision__authoritative() {
        let fp = fingerprint_external("ext1", b"revision:42").unwrap();
        assert_eq!(fp, format!("v{NORMALIZER_VERSION}:revision:42"));
    }

    #[test]
    fn fingerprint_external__payload__folds_identity() {
        let a = fingerprint_external("e1", b"{\"payload\":1}\n").unwrap();
        let b = fingerprint_external("e2", b"{\"payload\":1}\n").unwrap();
        assert_ne!(a, b);
        assert!(a.starts_with(&format!("v{NORMALIZER_VERSION}:")));
        assert!(!a.contains("etag:"));
    }

    #[test]
    fn fingerprint_external__reordered_json_keys__same_fingerprint() {
        let a = fingerprint_external("ext", br#"{"a":1,"b":2}"#).unwrap();
        let b = fingerprint_external("ext", br#"{"b":2,"a":1}"#).unwrap();
        assert_eq!(a, b, "canonical JSON must ignore object key order");
        // Nested objects also canonicalize.
        let n1 = fingerprint_external("ext", br#"{"z":{"a":1,"b":2},"y":3}"#).unwrap();
        let n2 = fingerprint_external("ext", br#"{"y":3,"z":{"b":2,"a":1}}"#).unwrap();
        assert_eq!(n1, n2);
    }
}
