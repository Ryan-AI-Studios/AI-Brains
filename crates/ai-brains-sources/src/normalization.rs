/// Version of the normalization algorithm.
///
/// Bumping this changes the fingerprint identity tuple (`v{n}:{hex}`) even when
/// the underlying content bytes are identical — so algorithm changes are never
/// confused with content changes.
pub const NORMALIZER_VERSION: u32 = 1;

/// Normalize UTF-8 text for stable file/markdown fingerprints:
/// - strip UTF-8 BOM
/// - replace CRLF / lone CR with LF
/// - reject invalid UTF-8 sequences (returns lossy replacement only when
///   `allow_lossy` path is used — default path is strict via [`normalize_file_bytes`])
pub fn normalize_utf8_text(text: &str) -> String {
    let without_bom = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    without_bom.replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalize raw file bytes: require valid UTF-8, then apply text normalization.
pub fn normalize_file_bytes(bytes: &[u8]) -> Result<Vec<u8>, crate::SourcesError> {
    let text = std::str::from_utf8(bytes).map_err(|e| crate::SourcesError::InvalidUtf8 {
        detail: e.to_string(),
    })?;
    Ok(normalize_utf8_text(text).into_bytes())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn normalize_utf8_text__strips_bom_and_normalizes_newlines() {
        let input = "\u{FEFF}a\r\nb\rc";
        assert_eq!(normalize_utf8_text(input), "a\nb\nc");
    }
}
