//! Human-readable fingerprint formatting (R24).
//!
//! SHA-256 hex in **4-character groups separated by hyphens** (16 groups for
//! 32 bytes). Lowercase preferred for machine stability; CLI may uppercase.

/// Format a 32-byte fingerprint as lowercase hex in 4-char hyphen groups.
///
/// Example for 32 bytes → 16 groups: `5f3a-9b1c-4e8f-…`.
pub fn format_fingerprint_hyphen(fingerprint: &[u8; 32]) -> String {
    let hex = hex::encode(fingerprint);
    let mut parts = Vec::with_capacity(16);
    for chunk in hex.as_bytes().chunks(4) {
        // hex::encode is ASCII; chunks are valid UTF-8.
        parts.push(std::str::from_utf8(chunk).unwrap_or("????"));
    }
    parts.join("-")
}

/// Raw lowercase hex without hyphens (machine API).
pub fn format_fingerprint_raw(fingerprint: &[u8; 32]) -> String {
    hex::encode(fingerprint)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn fingerprint_format__hyphen_groups__16_groups() {
        let mut fp = [0u8; 32];
        for (i, b) in fp.iter_mut().enumerate() {
            *b = i as u8;
        }
        let s = format_fingerprint_hyphen(&fp);
        let groups: Vec<&str> = s.split('-').collect();
        assert_eq!(groups.len(), 16, "expected 16 hyphen groups, got {s}");
        for g in &groups {
            assert_eq!(g.len(), 4, "group {g}");
            assert!(g.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(g.chars().all(|c| !c.is_ascii_uppercase()));
        }
        // No hyphens in raw form; length 64.
        let raw = format_fingerprint_raw(&fp);
        assert_eq!(raw.len(), 64);
        assert!(!raw.contains('-'));
        assert_eq!(raw, groups.join(""));
    }
}
