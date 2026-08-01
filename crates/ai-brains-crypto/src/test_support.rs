//! Test utilities for ai-brains-crypto.
//!
//! Production modules must not depend on test-only hooks that accept caller
//! nonces or omit blob_id binding (C13). Re-exports below are for integration
//! tests and unit tests only.

use base64::Engine;

/// A test passphrase for use in tests
pub const TEST_PASSPHRASE: &[u8] = b"test-passphrase-not-for-production";

// Re-export fixed-nonce / zero-blob helpers when compiled under test.
// Production builds of this module still compile (constants only).

#[cfg(test)]
pub use crate::content_envelope::{seal_aad_with_zero_blob, seal_with_nonce};
#[cfg(test)]
pub use crate::content_key_store::wrap_with_nonce;

/// T181-F34: assert `output` does not contain `secret_bytes` as hex, standard
/// base64, or raw UTF-8 (when the secret is valid UTF-8 text).
///
/// Use for CLI/test stdout/stderr and any serialized surfaces that must not
/// leak key material. Does not prove zeroization of process memory.
pub fn assert_no_secret_leakage(output: &str, secret_bytes: &[u8]) {
    let hex_form = hex::encode(secret_bytes);
    assert!(
        !output
            .to_ascii_lowercase()
            .contains(&hex_form.to_ascii_lowercase()),
        "output must not contain secret as hex"
    );

    let b64 = base64::engine::general_purpose::STANDARD.encode(secret_bytes);
    assert!(
        !output.contains(&b64),
        "output must not contain secret as standard base64"
    );

    // URL-safe base64 without padding is also common in dumps.
    let b64_url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(secret_bytes);
    if b64_url.len() >= 8 {
        assert!(
            !output.contains(&b64_url),
            "output must not contain secret as url-safe base64"
        );
    }

    if let Ok(raw) = std::str::from_utf8(secret_bytes)
        && !raw.is_empty()
        && raw.len() >= 4
    {
        assert!(
            !output.contains(raw),
            "output must not contain secret as raw UTF-8"
        );
    }

    // Byte-display forms (Debug / alternate slice Debug) — F34.
    let debug_bytes = format!("{:?}", secret_bytes);
    if debug_bytes.len() >= 8 {
        assert!(
            !output.contains(&debug_bytes),
            "output must not contain secret as Debug byte slice"
        );
    }
    let debug_vec = format!("{:?}", secret_bytes.to_vec());
    if debug_vec.len() >= 8 {
        assert!(
            !output.contains(&debug_vec),
            "output must not contain secret as Debug Vec<u8>"
        );
    }
}

/// T181-F12/F34: kit JSON / wrapped ciphertext must not appear in operator-facing output.
pub fn assert_no_kit_dump(output: &str, kit_json: &str) {
    assert!(
        !output.contains(kit_json),
        "output must not contain full RecoveryKit JSON"
    );
    // Coarse markers that suggest a serialized wrapped key dump.
    if kit_json.contains("ciphertext") {
        // Only fail if a substantial kit fragment appears (avoid matching unrelated words).
        let trimmed = kit_json.trim();
        if trimmed.len() > 32 {
            assert!(
                !output.contains(trimmed),
                "output must not contain kit JSON body"
            );
        }
    }
}
