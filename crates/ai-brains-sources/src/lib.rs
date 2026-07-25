//! Deterministic source fingerprints and content normalization (T149).
//!
//! Digest algorithm is **SHA-256** only (plus authoritative string forms for
//! Ledgerful bridge hashes and External ETag/revision).
//!
//! Common format: `v{NORMALIZER_VERSION}:{sha256_hex}`.
//! File fingerprints fold canonical source identity into the preimage.
//!
//! Git I/O goes exclusively through [`ai_brains_git::collect_metadata`] — this
//! crate never shells out to `git` and never hashes `.git` wholesale.

mod fingerprint;
mod git_fingerprint;
mod normalization;

pub use fingerprint::{
    Sha256Fingerprinter, file_fingerprint_preimage, fingerprint_bytes, fingerprint_external,
    fingerprint_file_with_identity, fingerprint_ledgerful,
};
pub use git_fingerprint::{
    canonicalize_git_metadata, fingerprint_git_metadata, fingerprint_git_path,
};
pub use normalization::{NORMALIZER_VERSION, normalize_file_bytes, normalize_utf8_text};

pub use fingerprint::SourcesError;
pub type Result<T> = std::result::Result<T, SourcesError>;
