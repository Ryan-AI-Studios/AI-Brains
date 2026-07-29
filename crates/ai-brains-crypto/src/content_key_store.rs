//! Content DEK lifecycle: generate, wrap, unwrap (crypto material only).
//!
//! # Naming split (crypto vs SQL)
//!
//! | Name | Layer | Role |
//! |------|-------|------|
//! | SQL table `content_key_store` | `ai-brains-store` | Durable wrap rows (T163) |
//! | This module `content_key_store` | `ai-brains-crypto` | Generate/wrap/unwrap DEK bytes |
//!
//! This module has **no** SQL/rusqlite. Persist wraps via store APIs
//! (`insert_content_key_wrap` / `destroy_content_key_wrap`). CE “cannot open
//! after destroy” is proven at the store integration layer (C14), not here.
//!
//! # Residual (ADR-0016 §4)
//!
//! Each wrap uses a fresh random 96-bit nonce under the vault `DataKey` (KEK).
//! DataKey is vault-lifetime and not rotated in T164; wrap-nonce count
//! accumulates over vault life. Near-term risk is accepted; rotation is a
//! future gap (not designed here).

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
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// Content DEK length (AES-256 key material).
pub const CONTENT_DEK_LEN: usize = 32;

/// Wrap schema version (v1 = 1). Aligns with T163 `content_key_store.wrap_schema_version`.
pub const WRAP_SCHEMA_VERSION: u32 = 1;

/// AES-GCM recommended IV length (96-bit). Fresh CSPRNG nonce per wrap.
pub const NONCE_LEN: usize = 12;

/// Domain-separation magic for content-envelope AAD (`AIBC`).
pub const AAD_MAGIC: &[u8; 4] = b"AIBC";

/// AAD kind for DEK wrap under DataKey.
pub const AAD_KIND_DEK_WRAP: u8 = 0x02;

/// 32-byte random content data-encryption key (per independently erasable unit).
///
/// Not derived from DataKey (not HKDF). Destroy durable wraps to cryptographically
/// erase content sealed under this DEK.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ContentDek {
    material: [u8; CONTENT_DEK_LEN],
}

impl ContentDek {
    /// Generate a new random ContentDek (OS CSPRNG). Fails closed on entropy error.
    pub fn generate() -> Result<Self> {
        let mut material = [0u8; CONTENT_DEK_LEN];
        SysRng
            .try_fill_bytes(&mut material)
            .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {e}")))?;
        Ok(Self { material })
    }

    /// Create a ContentDek from raw bytes (consumes the array).
    pub fn from_bytes(bytes: [u8; CONTENT_DEK_LEN]) -> Self {
        Self { material: bytes }
    }

    /// Access the raw key material (caller must not log or persist plaintext).
    pub fn expose_secret(&self) -> &[u8; CONTENT_DEK_LEN] {
        &self.material
    }
}

impl fmt::Debug for ContentDek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentDek([REDACTED])")
    }
}

/// AES-256-GCM wrap of a [`ContentDek`] under vault [`DataKey`].
///
/// Wire: `nonce` separate; `ciphertext` = ciphertext \|\| tag.
/// Debug prints lengths only (no key material or ciphertext bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct WrappedContentDek {
    pub wrap_schema_version: u32,
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

impl fmt::Debug for WrappedContentDek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WrappedContentDek")
            .field("wrap_schema_version", &self.wrap_schema_version)
            .field("nonce_len", &self.nonce.len())
            .field("ciphertext_len", &self.ciphertext.len())
            .finish()
    }
}

/// Parse a 12-byte nonce from stored bytes. Wrong length → [`CryptoError::InvalidNonceLength`].
pub fn parse_nonce(bytes: &[u8]) -> Result<[u8; NONCE_LEN]> {
    <[u8; NONCE_LEN]>::try_from(bytes).map_err(|_| CryptoError::InvalidNonceLength)
}

/// Build DEK-wrap AAD (kind `0x02`):
///
/// ```text
/// magic[4] = b"AIBC"
/// kind[1]  = 0x02
/// version  = u32 BE (wrap_schema_version)
/// content_key_id[16]
/// ```
pub fn build_dek_wrap_aad(wrap_schema_version: u32, content_key_id: &ContentKeyId) -> Vec<u8> {
    let mut aad = Vec::with_capacity(4 + 1 + 4 + 16);
    aad.extend_from_slice(AAD_MAGIC);
    aad.push(AAD_KIND_DEK_WRAP);
    aad.extend_from_slice(&wrap_schema_version.to_be_bytes());
    aad.extend_from_slice(content_key_id.as_uuid().as_bytes());
    aad
}

/// Wrap `dek` under `data_key` with AAD bound to `content_key_id`.
/// Fresh CSPRNG nonce every call. DataKey is the direct AES-GCM key (no Argon2).
pub fn wrap_content_dek(
    data_key: &DataKey,
    dek: &ContentDek,
    content_key_id: &ContentKeyId,
) -> Result<WrappedContentDek> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {e}")))?;
    wrap_content_dek_with_nonce(
        data_key,
        dek,
        content_key_id,
        &nonce_bytes,
        WRAP_SCHEMA_VERSION,
    )
}

/// Unwrap a content DEK. Wrong DataKey, wrong id, flipped bytes, or empty/short
/// ciphertext → [`CryptoError::AuthenticationFailed`] (no panic).
pub fn unwrap_content_dek(
    data_key: &DataKey,
    wrapped: &WrappedContentDek,
    content_key_id: &ContentKeyId,
) -> Result<ContentDek> {
    let aad = build_dek_wrap_aad(wrapped.wrap_schema_version, content_key_id);
    let cipher = Aes256Gcm::new_from_slice(data_key.expose_secret())
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    let nonce = Nonce::from_slice(&wrapped.nonce);
    // Zeroize intermediate DEK plaintext so the GCM output buffer is wiped on drop.
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &wrapped.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?,
    );

    let material: [u8; CONTENT_DEK_LEN] = plaintext
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::AuthenticationFailed)?;
    Ok(ContentDek::from_bytes(material))
}

fn wrap_content_dek_with_nonce(
    data_key: &DataKey,
    dek: &ContentDek,
    content_key_id: &ContentKeyId,
    nonce_bytes: &[u8; NONCE_LEN],
    wrap_schema_version: u32,
) -> Result<WrappedContentDek> {
    let aad = build_dek_wrap_aad(wrap_schema_version, content_key_id);
    let cipher = Aes256Gcm::new_from_slice(data_key.expose_secret())
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: dek.expose_secret().as_slice(),
                aad: &aad,
            },
        )
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    Ok(WrappedContentDek {
        wrap_schema_version,
        nonce: *nonce_bytes,
        ciphertext,
    })
}

/// Test-only fixed-nonce wrap for known-answer tests. Not reachable from production
/// call sites outside `#[cfg(test)]` / test_support.
#[cfg(test)]
pub fn wrap_with_nonce(
    data_key: &DataKey,
    dek: &ContentDek,
    content_key_id: &ContentKeyId,
    nonce: &[u8; NONCE_LEN],
) -> Result<WrappedContentDek> {
    wrap_content_dek_with_nonce(data_key, dek, content_key_id, nonce, WRAP_SCHEMA_VERSION)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use uuid::Uuid;

    fn fixed_content_key_id() -> ContentKeyId {
        ContentKeyId::from_uuid(
            Uuid::parse_str("00000000-0000-4000-8000-000000000001").expect("uuid"),
        )
    }

    #[test]
    fn content_dek__generate__unique_and_redacted_debug() {
        let a = ContentDek::generate().expect("gen a");
        let b = ContentDek::generate().expect("gen b");
        assert_ne!(a.expose_secret(), b.expose_secret());
        let debug_str = format!("{:?}", a);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains(&hex::encode(a.expose_secret())));
        assert_eq!(debug_str, "ContentDek([REDACTED])");
    }

    #[test]
    fn build_dek_wrap_aad__stable_bytes() {
        let id = fixed_content_key_id();
        let aad = build_dek_wrap_aad(WRAP_SCHEMA_VERSION, &id);
        // magic + kind + version BE + 16 uuid bytes
        assert_eq!(aad.len(), 4 + 1 + 4 + 16);
        assert_eq!(&aad[0..4], b"AIBC");
        assert_eq!(aad[4], AAD_KIND_DEK_WRAP);
        assert_eq!(&aad[5..9], &1u32.to_be_bytes());
        assert_eq!(&aad[9..25], id.as_uuid().as_bytes());
        // Golden vector (fixed UUID + v1)
        let expected = [
            0x41, 0x49, 0x42, 0x43, // AIBC
            0x02, // kind
            0x00, 0x00, 0x00, 0x01, // version 1 BE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // content_key_id
        ];
        assert_eq!(aad, expected);
    }

    #[test]
    fn wrap_unwrap_content_dek__round_trip() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        assert_eq!(wrapped.wrap_schema_version, WRAP_SCHEMA_VERSION);
        assert_eq!(wrapped.nonce.len(), NONCE_LEN);
        assert!(!wrapped.ciphertext.is_empty());
        let opened = unwrap_content_dek(&data_key, &wrapped, &id).expect("unwrap");
        assert_eq!(opened.expose_secret(), dek.expose_secret());
    }

    /// Production `wrap_content_dek` must draw a fresh CSPRNG nonce per call (C3).
    /// Uses the public API only — not `wrap_with_nonce`.
    #[test]
    fn wrap_content_dek__production_nonces__unique_across_calls() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let a = wrap_content_dek(&data_key, &dek, &id).expect("wrap a");
        let b = wrap_content_dek(&data_key, &dek, &id).expect("wrap b");
        assert_ne!(a.nonce, b.nonce, "production wrap must use distinct nonces");
        assert_ne!(
            a.ciphertext, b.ciphertext,
            "distinct nonces must yield distinct ciphertext"
        );
        assert_eq!(a.nonce.len(), NONCE_LEN);
        assert_eq!(b.nonce.len(), NONCE_LEN);
    }

    #[test]
    fn wrap_content_dek__wrong_data_key__authentication_failed() {
        let data_key = DataKey::generate();
        let other = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        let err = unwrap_content_dek(&other, &wrapped, &id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn wrap_content_dek__wrong_content_key_id__authentication_failed() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let other_id = ContentKeyId::new();
        let wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        let err = unwrap_content_dek(&data_key, &wrapped, &other_id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn wrap_content_dek__bitflip_ciphertext__authentication_failed() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let mut wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        if let Some(b) = wrapped.ciphertext.first_mut() {
            *b ^= 0x01;
        }
        let err = unwrap_content_dek(&data_key, &wrapped, &id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn wrap_with_nonce__known_vector__matches_fixture() {
        // Fixed materials for KAT (not production entropy).
        let data_key = DataKey::from_bytes([0x11; 32]);
        let dek = ContentDek::from_bytes([0x22; 32]);
        let id = fixed_content_key_id();
        let nonce = [0x33; 12];
        let wrapped = wrap_with_nonce(&data_key, &dek, &id, &nonce).expect("wrap");
        assert_eq!(wrapped.nonce, nonce);
        // Re-wrap must be deterministic with fixed nonce → same ciphertext.
        let again = wrap_with_nonce(&data_key, &dek, &id, &nonce).expect("wrap2");
        assert_eq!(wrapped.ciphertext, again.ciphertext);
        let opened = unwrap_content_dek(&data_key, &wrapped, &id).expect("unwrap");
        assert_eq!(opened.expose_secret(), dek.expose_secret());
        assert_eq!(wrapped.ciphertext.len(), 48);
        assert_eq!(
            wrapped.ciphertext, again.ciphertext,
            "fixed-nonce wrap must be deterministic"
        );
        // Golden ciphertext||tag (aes-gcm 0.10 Aes256Gcm + Payload AAD; fixed materials above).
        assert_eq!(
            hex::encode(&wrapped.ciphertext),
            "a6a17064e8b0570e909ac94f8509c10a24ea2d16a8fba78eda682ba781aca8083400d6e904372fceaff83a27e1eb579b"
        );
    }

    #[test]
    fn unwrap_content_dek__empty_ciphertext__authentication_failed_not_panic() {
        let data_key = DataKey::generate();
        let id = ContentKeyId::new();
        let wrapped = WrappedContentDek {
            wrap_schema_version: WRAP_SCHEMA_VERSION,
            nonce: [0u8; NONCE_LEN],
            ciphertext: Vec::new(),
        };
        let err = unwrap_content_dek(&data_key, &wrapped, &id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));

        let short = WrappedContentDek {
            wrap_schema_version: WRAP_SCHEMA_VERSION,
            nonce: [1u8; NONCE_LEN],
            ciphertext: vec![0u8; 4],
        };
        let err = unwrap_content_dek(&data_key, &short, &id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }

    #[test]
    fn wrapped_content_dek_debug__redacts_bytes() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        let debug_str = format!("{:?}", wrapped);
        assert!(debug_str.contains("ciphertext_len"));
        assert!(!debug_str.contains(&hex::encode(&wrapped.ciphertext)));
        assert!(!debug_str.contains(&hex::encode(wrapped.nonce)));
    }

    #[test]
    fn parse_nonce__wrong_length__invalid_nonce_length() {
        assert!(matches!(
            parse_nonce(&[0u8; 11]),
            Err(CryptoError::InvalidNonceLength)
        ));
        assert_eq!(parse_nonce(&[0u8; 12]).expect("12 bytes"), [0u8; 12]);
    }

    #[test]
    fn authentication_failed_display_has_no_secrets() {
        let msg = format!("{}", CryptoError::AuthenticationFailed);
        assert_eq!(msg, "Authentication failed");
        assert!(!msg.to_lowercase().contains("key"));
        assert!(!msg.to_lowercase().contains("ciphertext"));
    }

    #[test]
    fn wrap_content_dek__wrong_wrap_schema_version__authentication_failed() {
        let data_key = DataKey::generate();
        let dek = ContentDek::generate().expect("dek");
        let id = ContentKeyId::new();
        let mut wrapped = wrap_content_dek(&data_key, &dek, &id).expect("wrap");
        // Tamper version field → AAD mismatch on unwrap (no hard reject of non-1 versions).
        wrapped.wrap_schema_version = WRAP_SCHEMA_VERSION.wrapping_add(1);
        let err = unwrap_content_dek(&data_key, &wrapped, &id).expect_err("must fail");
        assert!(matches!(err, CryptoError::AuthenticationFailed));
    }
}
