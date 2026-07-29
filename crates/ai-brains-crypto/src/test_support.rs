//! Test utilities for ai-brains-crypto.
//!
//! Production modules must not depend on test-only hooks that accept caller
//! nonces or omit blob_id binding (C13). Re-exports below are for integration
//! tests and unit tests only.

/// A test passphrase for use in tests
pub const TEST_PASSPHRASE: &[u8] = b"test-passphrase-not-for-production";

// Re-export fixed-nonce / zero-blob helpers when compiled under test.
// Production builds of this module still compile (constants only).

#[cfg(test)]
pub use crate::content_envelope::{seal_aad_with_zero_blob, seal_with_nonce};
#[cfg(test)]
pub use crate::content_key_store::wrap_with_nonce;
