//! Content AEAD seal/open under a [`ContentDek`] (crypto only).
//!
//! # Naming split (crypto vs SQL)
//!
//! | Name | Layer | Role |
//! |------|-------|------|
//! | Module `content_envelope` (store) | `ai-brains-store` | SQL + erasure projections (T163) |
//! | This module `content_envelope` | `ai-brains-crypto` | AEAD seal/open of content bytes |
//!
//! No SQL/rusqlite here. Persist sealed blobs via store `insert_encrypted_blob`.
//! Public [`SealAad::blob_id`] is **mandatory** (C13) — zero-byte / unbound blob
//! AAD is test-only via [`test_support`](crate::test_support).
//!
//! # Residual (ADR-0016 §4)
//!
//! Content DEKs are per-unit with O(1) seals, so GCM random-nonce budget is not
//! a practical issue for content seals. DataKey wrap-nonce lifetime budget is
//! a separate residual documented in [`crate::content_key_store`].

use crate::content_key_store::{
    AAD_MAGIC, ContentDek, NONCE_LEN, WrappedContentDek, unwrap_content_dek, wrap_content_dek,
};
use crate::data_key::DataKey;
use crate::errors::{CryptoError, Result};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use ai_brains_core::ids::ContentKeyId;
use rand::TryRng;
use rand::rngs::SysRng;
use std::fmt;
use uuid::Uuid;
use zeroize::Zeroizing;

/// Envelope schema version (v1 = 1). Aligns with T163 `ENVELOPE_SCHEMA_VERSION`.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// Forensic algorithm label (matches T163 `ALGORITHM_AES_256_GCM`).
pub const ALGORITHM_LABEL: &str = "AES-256-GCM";

/// AAD kind for content seal under ContentDek.
pub const AAD_KIND_CONTENT_SEAL: u8 = 0x01;

/// Authenticated associated data for content seal/open.
///
/// `blob_id` is mandatory on the public path (C13) so ciphertext cannot be
/// transplanted across blob rows under the same DEK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealAad {
    pub envelope_schema_version: u32,
    pub content_key_id: ContentKeyId,
    pub blob_id: Uuid,
}

/// AES-256-GCM sealed content. Wire: `nonce` separate; `ciphertext` = ciphertext \|\| tag.
///
/// Debug prints version + lengths only.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedContent {
    pub envelope_schema_version: u32,
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

impl fmt::Debug for SealedContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SealedContent")
            .field("envelope_schema_version", &self.envelope_schema_version)
            .field("nonce_len", &self.nonce.len())
            .field("ciphertext_len", &self.ciphertext.len())
            .finish()
    }
}

/// Result of [`generate_wrap_and_seal`]: wrap + sealed content (DEK not retained).
#[derive(Debug)]
pub struct GeneratedEnvelope {
    pub content_key_id: ContentKeyId,
    pub wrapped_dek: WrappedContentDek,
    pub sealed: SealedContent,
}

/// Build content-seal AAD (kind `0x01`):
///
/// ```text
/// magic[4] = b"AIBC"
/// kind[1]  = 0x01
/// version  = u32 BE (envelope_schema_version)
/// content_key_id[16]
/// blob_id[16]
/// ```
pub fn build_content_seal_aad(aad: &SealAad) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 1 + 4 + 16 + 16);
    out.extend_from_slice(AAD_MAGIC);
    out.push(AAD_KIND_CONTENT_SEAL);
    out.extend_from_slice(&aad.envelope_schema_version.to_be_bytes());
    out.extend_from_slice(aad.content_key_id.as_uuid().as_bytes());
    out.extend_from_slice(aad.blob_id.as_bytes());
    out
}

/// Seal `plaintext` under `dek` with mandatory blob-bound AAD. Fresh CSPRNG nonce.
pub fn seal(dek: &ContentDek, plaintext: &[u8], aad: &SealAad) -> Result<SealedContent> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {e}")))?;
    seal_with_nonce_inner(dek, plaintext, aad, &nonce_bytes)
}

/// Open sealed content. Auth/AAD/key mismatch → [`CryptoError::AuthenticationFailed`].
/// Returns a zeroizing plaintext buffer.
///
/// Wire `SealedContent.envelope_schema_version` must match `aad.envelope_schema_version`
/// (the version bound into GCM AAD). Mismatch is fail-closed before AEAD so the
/// duplicated wire field cannot disagree with the authenticated AAD domain.
pub fn open(dek: &ContentDek, sealed: &SealedContent, aad: &SealAad) -> Result<Zeroizing<Vec<u8>>> {
    if sealed.envelope_schema_version != aad.envelope_schema_version {
        return Err(CryptoError::AuthenticationFailed);
    }
    let aad_bytes = build_content_seal_aad(aad);
    let cipher = Aes256Gcm::new_from_slice(dek.expose_secret())
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    let nonce = Nonce::from_slice(&sealed.nonce);
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &sealed.ciphertext,
                aad: &aad_bytes,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    Ok(Zeroizing::new(plaintext))
}

/// Generate DEK → wrap under DataKey → seal plaintext → DEK dropped (zeroized).
///
/// Caller persists wrap then blob (ADR-0016 §8 order) then events. Pure crypto;
/// no SQL.
pub fn generate_wrap_and_seal(
    data_key: &DataKey,
    content_key_id: ContentKeyId,
    blob_id: Uuid,
    plaintext: &[u8],
) -> Result<GeneratedEnvelope> {
    let dek = ContentDek::generate()?;
    let wrapped_dek = wrap_content_dek(data_key, &dek, &content_key_id)?;
    let aad = SealAad {
        envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
        content_key_id,
        blob_id,
    };
    let sealed = seal(&dek, plaintext, &aad)?;
    // dek dropped here → ZeroizeOnDrop
    Ok(GeneratedEnvelope {
        content_key_id,
        wrapped_dek,
        sealed,
    })
}

/// Unwrap DEK → open → DEK dropped (zeroized). Fails closed on any auth error.
pub fn unwrap_and_open(
    data_key: &DataKey,
    content_key_id: &ContentKeyId,
    wrapped: &WrappedContentDek,
    sealed: &SealedContent,
    blob_id: Uuid,
) -> Result<Zeroizing<Vec<u8>>> {
    let dek = unwrap_content_dek(data_key, wrapped, content_key_id)?;
    let aad = SealAad {
        envelope_schema_version: sealed.envelope_schema_version,
        content_key_id: *content_key_id,
        blob_id,
    };
    open(&dek, sealed, &aad)
    // dek dropped → ZeroizeOnDrop
}

fn seal_with_nonce_inner(
    dek: &ContentDek,
    plaintext: &[u8],
    aad: &SealAad,
    nonce_bytes: &[u8; NONCE_LEN],
) -> Result<SealedContent> {
    let aad_bytes = build_content_seal_aad(aad);
    let cipher = Aes256Gcm::new_from_slice(dek.expose_secret())
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: &aad_bytes,
            },
        )
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    Ok(SealedContent {
        envelope_schema_version: aad.envelope_schema_version,
        nonce: *nonce_bytes,
        ciphertext,
    })
}

/// Test-only fixed-nonce seal for known-answer tests.
#[cfg(test)]
pub fn seal_with_nonce(
    dek: &ContentDek,
    plaintext: &[u8],
    aad: &SealAad,
    nonce: &[u8; NONCE_LEN],
) -> Result<SealedContent> {
    seal_with_nonce_inner(dek, plaintext, aad, nonce)
}

/// Test-only: content seal AAD with **zero-byte** blob binding (`Uuid::nil()`).
///
/// Not for production writers (C13). Layout still includes 16 zero bytes for
/// blob_id so the AAD frame matches production length.
#[cfg(test)]
pub fn seal_aad_with_zero_blob(
    envelope_schema_version: u32,
    content_key_id: ContentKeyId,
) -> SealAad {
    SealAad {
        envelope_schema_version,
        content_key_id,
        blob_id: Uuid::nil(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::content_key_store::ContentDek;
    use crate::data_key::DataKey;

    fn fixed_content_key_id() -> ContentKeyId {
        ContentKeyId::from_uuid(
            Uuid::parse_str("00000000-0000-4000-8000-000000000001").expect("uuid"),
        )
    }

    fn fixed_blob_id() -> Uuid {
        Uuid::parse_str("00000000-0000-4000-8000-000000000002").expect("uuid")
    }

    fn sample_aad() -> SealAad {
        SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: fixed_content_key_id(),
            blob_id: fixed_blob_id(),
        }
    }

    #[test]
    fn build_content_seal_aad__stable_bytes() {
        let aad = sample_aad();
        let bytes = build_content_seal_aad(&aad);
        assert_eq!(bytes.len(), 4 + 1 + 4 + 16 + 16);
        assert_eq!(&bytes[0..4], b"AIBC");
        assert_eq!(bytes[4], AAD_KIND_CONTENT_SEAL);
        assert_eq!(&bytes[5..9], &1u32.to_be_bytes());
        assert_eq!(&bytes[9..25], aad.content_key_id.as_uuid().as_bytes());
        assert_eq!(&bytes[25..41], aad.blob_id.as_bytes());
        let expected = [
            0x41, 0x49, 0x42, 0x43, // AIBC
            0x01, // kind
            0x00, 0x00, 0x00, 0x01, // version 1 BE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // content_key_id
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, // blob_id
        ];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn seal_open__round_trip() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let plaintext = b"hello content envelope";
        let sealed = seal(&dek, plaintext, &aad).expect("seal");
        assert_eq!(sealed.envelope_schema_version, ENVELOPE_SCHEMA_VERSION);
        let opened = open(&dek, &sealed, &aad).expect("open");
        assert_eq!(opened.as_slice(), plaintext);
    }

    /// Production `seal` must draw a fresh CSPRNG nonce per call (C3).
    /// Uses the public API only — not `seal_with_nonce`.
    #[test]
    fn seal__production_nonces__unique_across_calls() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let plaintext = b"same plaintext for nonce freshness";
        let a = seal(&dek, plaintext, &aad).expect("seal a");
        let b = seal(&dek, plaintext, &aad).expect("seal b");
        assert_ne!(a.nonce, b.nonce, "production seal must use distinct nonces");
        assert_ne!(
            a.ciphertext, b.ciphertext,
            "distinct nonces must yield distinct ciphertext"
        );
        assert_eq!(a.nonce.len(), NONCE_LEN);
        assert_eq!(b.nonce.len(), NONCE_LEN);
    }

    #[test]
    fn seal_open__wrong_aad_blob_id__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let content_key_id = ContentKeyId::new();
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id,
            blob_id: Uuid::new_v4(),
        };
        let sealed = seal(&dek, b"secret", &aad).expect("seal");
        let wrong = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id,
            blob_id: Uuid::new_v4(),
        };
        let err = open(&dek, &sealed, &wrong).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn seal_open__wrong_aad_content_key_id__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let blob_id = Uuid::new_v4();
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id,
        };
        let sealed = seal(&dek, b"secret", &aad).expect("seal");
        let wrong = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id,
        };
        let err = open(&dek, &sealed, &wrong).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn seal_open__wrong_envelope_schema_version__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let sealed = seal(&dek, b"secret", &aad).expect("seal");
        // Wrong AAD version field → GCM AuthenticationFailed (caller-controlled AAD domain).
        let wrong = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION.wrapping_add(1),
            content_key_id: aad.content_key_id,
            blob_id: aad.blob_id,
        };
        let err = open(&dek, &sealed, &wrong).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    /// Wire version field must match AAD version even when ciphertext/nonce are intact.
    /// Without this check, tampering `SealedContent.envelope_schema_version` alone would
    /// still decrypt under the original AAD (unauthenticated dual-source version).
    #[test]
    fn seal_open__wrong_sealed_envelope_schema_version__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let mut sealed = seal(&dek, b"secret", &aad).expect("seal");
        sealed.envelope_schema_version = ENVELOPE_SCHEMA_VERSION.wrapping_add(1);
        let err = open(&dek, &sealed, &aad).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn seal_open__bitflip_ciphertext__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let mut sealed = seal(&dek, b"secret", &aad).expect("seal");
        if let Some(b) = sealed.ciphertext.first_mut() {
            *b ^= 0x01;
        }
        let err = open(&dek, &sealed, &aad).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn seal_open__wrong_dek__authentication_failed() {
        let dek = ContentDek::generate().expect("dek");
        let other = ContentDek::generate().expect("other");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let sealed = seal(&dek, b"secret", &aad).expect("seal");
        let err = open(&other, &sealed, &aad).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn seal_with_nonce__known_vector__matches_fixture() {
        let dek = ContentDek::from_bytes([0x44; 32]);
        let aad = sample_aad();
        let nonce = [0x55; 12];
        let plaintext = b"kat-plaintext";
        let sealed = seal_with_nonce(&dek, plaintext, &aad, &nonce).expect("seal");
        assert_eq!(sealed.nonce, nonce);
        let again = seal_with_nonce(&dek, plaintext, &aad, &nonce).expect("seal2");
        assert_eq!(sealed.ciphertext, again.ciphertext);
        let opened = open(&dek, &sealed, &aad).expect("open");
        assert_eq!(opened.as_slice(), plaintext);
        // 13 plaintext + 16 tag
        assert_eq!(sealed.ciphertext.len(), 13 + 16);
        assert_eq!(
            hex::encode(&sealed.ciphertext),
            "437cb3251bd5ea24f3a435f0fe3dd5fda30639204b16adb48fc362f0ca"
        );
    }

    #[test]
    fn sealed_content_debug__redacts_bytes() {
        let dek = ContentDek::generate().expect("dek");
        let aad = SealAad {
            envelope_schema_version: ENVELOPE_SCHEMA_VERSION,
            content_key_id: ContentKeyId::new(),
            blob_id: Uuid::new_v4(),
        };
        let sealed = seal(&dek, b"visible-if-leaked", &aad).expect("seal");
        let debug_str = format!("{:?}", sealed);
        assert!(debug_str.contains("ciphertext_len"));
        assert!(debug_str.contains("nonce_len"));
        assert!(!debug_str.contains("visible-if-leaked"));
        assert!(!debug_str.contains(&hex::encode(&sealed.ciphertext)));
        assert!(!debug_str.contains(&hex::encode(sealed.nonce)));
    }

    #[test]
    fn generate_wrap_and_seal__round_trip_via_unwrap_and_open() {
        let data_key = DataKey::generate();
        let content_key_id = ContentKeyId::new();
        let blob_id = Uuid::new_v4();
        let plaintext = b"high-level helper round trip";
        let env = generate_wrap_and_seal(&data_key, content_key_id, blob_id, plaintext)
            .expect("generate_wrap_and_seal");
        assert_eq!(env.content_key_id, content_key_id);
        let opened = unwrap_and_open(
            &data_key,
            &content_key_id,
            &env.wrapped_dek,
            &env.sealed,
            blob_id,
        )
        .expect("unwrap_and_open");
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn seal_aad_with_zero_blob__binds_nil_uuid() {
        let id = fixed_content_key_id();
        let aad = seal_aad_with_zero_blob(ENVELOPE_SCHEMA_VERSION, id);
        assert_eq!(aad.blob_id, Uuid::nil());
        let bytes = build_content_seal_aad(&aad);
        assert_eq!(&bytes[25..41], Uuid::nil().as_bytes());
    }
}
