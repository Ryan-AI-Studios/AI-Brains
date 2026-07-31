//! Per-recipient content-DEK wrap (ADR-0018 §17).
//!
//! X25519 ephemeral ECDH → HKDF-SHA256 (`Some(&[])` salt) → AES-256-GCM.
//!
//! ```text
//! info = schema_version u16 BE
//!      ‖ u16be(len(label)) ‖ label   # label = b"aib-sync-dek-wrap"
//!      ‖ content_key_id 16 ‖ recipient_device_id 16 ‖ sender_device_id 16
//! aad  = schema_version u16 BE ‖ content_key_id 16 ‖ recipient_device_id 16
//! ```

use crate::error::{Result, SyncError};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use ai_brains_core::ids::{ContentKeyId, DeviceId};
use hkdf::Hkdf;
use rand::TryRng;
use rand::rngs::SysRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::Zeroizing;

/// Exact ASCII label for HKDF info (17 bytes).
pub const LABEL_AIB_SYNC_DEK_WRAP: &[u8] = b"aib-sync-dek-wrap";

pub const DEK_LEN: usize = 32;
pub const WRAP_NONCE_LEN: usize = 12;
pub const EPH_PUB_LEN: usize = 32;

/// Result of wrapping a content DEK for one recipient.
#[derive(Clone, PartialEq, Eq)]
pub struct PeerDekWrap {
    pub eph_x25519_pub: [u8; EPH_PUB_LEN],
    pub wrap_nonce: [u8; WRAP_NONCE_LEN],
    /// AES-GCM ciphertext ‖ tag (no plaintext DEK).
    pub wrap_ct: Vec<u8>,
}

impl std::fmt::Debug for PeerDekWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeerDekWrap")
            .field("eph_x25519_pub_len", &self.eph_x25519_pub.len())
            .field("wrap_nonce_len", &self.wrap_nonce.len())
            .field("wrap_ct_len", &self.wrap_ct.len())
            .finish()
    }
}

/// Build HKDF `info` bytes (§17.2).
pub fn build_wrap_info(
    schema_version: u16,
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
    sender_device_id: &DeviceId,
) -> Vec<u8> {
    let label = LABEL_AIB_SYNC_DEK_WRAP;
    let mut info = Vec::with_capacity(2 + 2 + label.len() + 16 + 16 + 16);
    info.extend_from_slice(&schema_version.to_be_bytes());
    info.extend_from_slice(&(label.len() as u16).to_be_bytes());
    info.extend_from_slice(label);
    info.extend_from_slice(content_key_id.as_uuid().as_bytes());
    info.extend_from_slice(recipient_device_id.as_uuid().as_bytes());
    info.extend_from_slice(sender_device_id.as_uuid().as_bytes());
    info
}

/// Build AES-GCM AAD bytes (§17.3).
pub fn build_wrap_aad(
    schema_version: u16,
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(2 + 16 + 16);
    aad.extend_from_slice(&schema_version.to_be_bytes());
    aad.extend_from_slice(content_key_id.as_uuid().as_bytes());
    aad.extend_from_slice(recipient_device_id.as_uuid().as_bytes());
    aad
}

/// Derive 32-byte wrap key from X25519 shared secret via HKDF-SHA256.
///
/// Salt call style: `Hkdf::new(Some(&[]), shared)` — empty salt (R11 / §3.3).
pub fn derive_wrap_key(shared: &[u8; 32], info: &[u8]) -> Result<[u8; DEK_LEN]> {
    let hk = Hkdf::<Sha256>::new(Some(&[]), shared);
    let mut okm = [0u8; DEK_LEN];
    hk.expand(info, &mut okm)
        .map_err(|e| SyncError::Crypto(format!("HKDF expand: {e}")))?;
    Ok(okm)
}

/// Wrap `content_dek` for `recipient_static_pub` (peer X25519 public).
pub fn wrap_content_dek_for_recipient(
    schema_version: u16,
    content_dek: &[u8; DEK_LEN],
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
    sender_device_id: &DeviceId,
    recipient_static_pub: &X25519Public,
) -> Result<PeerDekWrap> {
    // Fresh ephemeral seed (R25 — no panic random()).
    let mut eph_seed = [0u8; 32];
    SysRng
        .try_fill_bytes(&mut eph_seed)
        .map_err(|e| SyncError::EntropyFailed(format!("eph seed: {e}")))?;

    let mut nonce = [0u8; WRAP_NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce)
        .map_err(|e| SyncError::EntropyFailed(format!("wrap nonce: {e}")))?;

    wrap_content_dek_for_recipient_with_seed(
        schema_version,
        content_dek,
        content_key_id,
        recipient_device_id,
        sender_device_id,
        recipient_static_pub,
        &eph_seed,
        &nonce,
    )
}

/// Crate-internal seed inject for KATs / deterministic unit tests (F20).
/// Production callers must use [`wrap_content_dek_for_recipient`] (OS CSPRNG).
/// Not re-exported from the crate root — not a consumer-facing API.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wrap_content_dek_for_recipient_with_seed(
    schema_version: u16,
    content_dek: &[u8; DEK_LEN],
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
    sender_device_id: &DeviceId,
    recipient_static_pub: &X25519Public,
    eph_seed: &[u8; 32],
    wrap_nonce: &[u8; WRAP_NONCE_LEN],
) -> Result<PeerDekWrap> {
    let eph_secret = StaticSecret::from(*eph_seed);
    let eph_pub = X25519Public::from(&eph_secret);
    wrap_with_eph(
        schema_version,
        content_dek,
        content_key_id,
        recipient_device_id,
        sender_device_id,
        recipient_static_pub,
        &eph_secret,
        &eph_pub,
        wrap_nonce,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn wrap_with_eph(
    schema_version: u16,
    content_dek: &[u8; DEK_LEN],
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
    sender_device_id: &DeviceId,
    recipient_static_pub: &X25519Public,
    eph_secret: &StaticSecret,
    eph_pub: &X25519Public,
    nonce_bytes: &[u8; WRAP_NONCE_LEN],
) -> Result<PeerDekWrap> {
    let shared = eph_secret.diffie_hellman(recipient_static_pub);
    let info = build_wrap_info(
        schema_version,
        content_key_id,
        recipient_device_id,
        sender_device_id,
    );
    let wrap_key = Zeroizing::new(derive_wrap_key(shared.as_bytes(), &info)?);
    let aad = build_wrap_aad(schema_version, content_key_id, recipient_device_id);

    let cipher = Aes256Gcm::new_from_slice(wrap_key.as_slice())
        .map_err(|e| SyncError::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let wrap_ct = cipher
        .encrypt(
            nonce,
            Payload {
                msg: content_dek.as_slice(),
                aad: &aad,
            },
        )
        .map_err(|e| SyncError::Crypto(format!("wrap encrypt: {e}")))?;

    Ok(PeerDekWrap {
        eph_x25519_pub: eph_pub.to_bytes(),
        wrap_nonce: *nonce_bytes,
        wrap_ct,
    })
}

/// Unwrap content DEK for the recipient holding `recipient_static`.
pub fn unwrap_content_dek(
    schema_version: u16,
    wrap: &PeerDekWrap,
    content_key_id: &ContentKeyId,
    recipient_device_id: &DeviceId,
    sender_device_id: &DeviceId,
    recipient_static: &StaticSecret,
) -> Result<[u8; DEK_LEN]> {
    let eph_pub = X25519Public::from(wrap.eph_x25519_pub);
    let shared = recipient_static.diffie_hellman(&eph_pub);
    let info = build_wrap_info(
        schema_version,
        content_key_id,
        recipient_device_id,
        sender_device_id,
    );
    let wrap_key = Zeroizing::new(derive_wrap_key(shared.as_bytes(), &info)?);
    let aad = build_wrap_aad(schema_version, content_key_id, recipient_device_id);

    let cipher = Aes256Gcm::new_from_slice(wrap_key.as_slice())
        .map_err(|e| SyncError::Crypto(e.to_string()))?;
    let nonce = Nonce::from_slice(&wrap.wrap_nonce);
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &wrap.wrap_ct,
                    aad: &aad,
                },
            )
            .map_err(|_| SyncError::WrapOpenFailed)?,
    );
    let dek: [u8; DEK_LEN] = plaintext
        .as_slice()
        .try_into()
        .map_err(|_| SyncError::WrapOpenFailed)?;
    Ok(dek)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::device_keys::generate_device_keys;
    use crate::enrollment::REPLICATION_SCHEMA_VERSION;
    use uuid::Uuid;

    fn fixed_id(n: u8) -> Uuid {
        let mut b = [0u8; 16];
        b[15] = n;
        Uuid::from_bytes(b)
    }

    #[test]
    fn wrap_dek__hkdf_okm__kat() {
        // Fixed IKM + info → stable OKM (Some(&[]) salt style).
        let ikm = [0x42u8; 32];
        let content_key = ContentKeyId::from_uuid(fixed_id(1));
        let recipient = DeviceId::from_uuid(fixed_id(2));
        let sender = DeviceId::from_uuid(fixed_id(3));
        let info = build_wrap_info(
            REPLICATION_SCHEMA_VERSION,
            &content_key,
            &recipient,
            &sender,
        );
        // Expected info layout
        assert_eq!(&info[0..2], &1u16.to_be_bytes());
        assert_eq!(&info[2..4], &17u16.to_be_bytes());
        assert_eq!(&info[4..21], LABEL_AIB_SYNC_DEK_WRAP);
        let okm = derive_wrap_key(&ikm, &info).expect("hkdf");
        // Second call identical.
        let okm2 = derive_wrap_key(&ikm, &info).expect("hkdf2");
        assert_eq!(okm, okm2);
        assert_eq!(okm.len(), 32);
        assert_ne!(okm, [0u8; 32]);
        let hk = Hkdf::<Sha256>::new(Some(&[]), &ikm);
        let mut direct = [0u8; 32];
        hk.expand(&info, &mut direct).expect("expand");
        assert_eq!(okm, direct);
        // Fixed IKM ([0x42;32]) + info (schema=1, content/recipient/sender fixed_id 1/2/3)
        // → pinned OKM (T176 residual; T178 owns full WRAP KATs).
        assert_eq!(
            hex::encode(okm),
            "ac8bacd1a06000523db2170a84db49ccecb1ed2fbc5f6642c975840d21aadde3"
        );
        let aad = build_wrap_aad(REPLICATION_SCHEMA_VERSION, &content_key, &recipient);
        assert_eq!(aad.len(), 2 + 16 + 16);
        assert_eq!(&aad[0..2], &1u16.to_be_bytes());
    }

    #[test]
    fn wrap_dek__roundtrip_recipient__ok() {
        let recipient_keys = generate_device_keys().expect("recip");
        let sender_keys = generate_device_keys().expect("sender");
        let content_key = ContentKeyId::from_uuid(fixed_id(10));
        let recipient_id = DeviceId::from_uuid(fixed_id(20));
        let sender_id = DeviceId::from_uuid(fixed_id(30));
        let dek = [0xABu8; 32];
        let wrap = wrap_content_dek_for_recipient(
            REPLICATION_SCHEMA_VERSION,
            &dek,
            &content_key,
            &recipient_id,
            &sender_id,
            &recipient_keys.x25519_public(),
        )
        .expect("wrap");
        let opened = unwrap_content_dek(
            REPLICATION_SCHEMA_VERSION,
            &wrap,
            &content_key,
            &recipient_id,
            &sender_id,
            &recipient_keys.x25519_secret(),
        )
        .expect("unwrap");
        assert_eq!(opened, dek);
        let _ = sender_keys;
    }

    #[test]
    fn wrap_dek__wrong_static_key__err() {
        let recipient_keys = generate_device_keys().expect("recip");
        let wrong_keys = generate_device_keys().expect("wrong");
        let content_key = ContentKeyId::from_uuid(fixed_id(10));
        let recipient_id = DeviceId::from_uuid(fixed_id(20));
        let sender_id = DeviceId::from_uuid(fixed_id(30));
        let dek = [0xCDu8; 32];
        let wrap = wrap_content_dek_for_recipient(
            REPLICATION_SCHEMA_VERSION,
            &dek,
            &content_key,
            &recipient_id,
            &sender_id,
            &recipient_keys.x25519_public(),
        )
        .expect("wrap");
        let err = unwrap_content_dek(
            REPLICATION_SCHEMA_VERSION,
            &wrap,
            &content_key,
            &recipient_id,
            &sender_id,
            &wrong_keys.x25519_secret(),
        )
        .expect_err("wrong key");
        assert!(matches!(err, SyncError::WrapOpenFailed));
    }

    // -----------------------------------------------------------------------
    // T178 WRAP / crypto unit claims
    // -----------------------------------------------------------------------

    /// T178-WRAP-kat-info-aad-bytes — static info/aad/okm exact hex (F20/F6).
    #[test]
    fn t178_wrap_kat_info_aad_bytes__static_hex() {
        // T178-WRAP-kat-info-aad-bytes
        let content_key = ContentKeyId::from_uuid(fixed_id(1));
        let recipient = DeviceId::from_uuid(fixed_id(2));
        let sender = DeviceId::from_uuid(fixed_id(3));
        let info = build_wrap_info(
            REPLICATION_SCHEMA_VERSION,
            &content_key,
            &recipient,
            &sender,
        );
        let aad = build_wrap_aad(REPLICATION_SCHEMA_VERSION, &content_key, &recipient);
        let ikm = [0x42u8; 32];
        let okm = derive_wrap_key(&ikm, &info).expect("okm");

        // info = schema u16 BE ‖ u16be(17) ‖ "aib-sync-dek-wrap" ‖ content‖recipient‖sender
        let expected_info = hex::decode(concat!(
            "0001",                               // schema 1
            "0011",                               // label len 17
            "6169622d73796e632d64656b2d77726170", // aib-sync-dek-wrap
            "00000000000000000000000000000001",   // content_key
            "00000000000000000000000000000002",   // recipient
            "00000000000000000000000000000003",   // sender
        ))
        .expect("info hex");
        assert_eq!(info, expected_info, "info got {}", hex::encode(&info));

        let expected_aad = hex::decode(concat!(
            "0001",                             // schema
            "00000000000000000000000000000001", // content_key
            "00000000000000000000000000000002", // recipient
        ))
        .expect("aad hex");
        assert_eq!(aad, expected_aad, "aad got {}", hex::encode(&aad));

        assert_eq!(
            hex::encode(okm),
            "ac8bacd1a06000523db2170a84db49ccecb1ed2fbc5f6642c975840d21aadde3"
        );
    }

    /// T178-WRAP-kat-seeded-ciphertext — fixed eph_seed + nonce → pin wrap_ct hex.
    #[test]
    fn t178_wrap_kat_seeded_ciphertext__pin_hex() {
        // T178-WRAP-kat-seeded-ciphertext
        use x25519_dalek::StaticSecret;
        let recipient_seed = [0x55u8; 32];
        let recipient_static = StaticSecret::from(recipient_seed);
        let recipient_pub = X25519Public::from(&recipient_static);
        let content_key = ContentKeyId::from_uuid(fixed_id(1));
        let recipient = DeviceId::from_uuid(fixed_id(2));
        let sender = DeviceId::from_uuid(fixed_id(3));
        let dek = [0xABu8; 32];
        let eph_seed = [0x11u8; 32];
        let wrap_nonce = [0x22u8; 12];
        let wrap = wrap_content_dek_for_recipient_with_seed(
            REPLICATION_SCHEMA_VERSION,
            &dek,
            &content_key,
            &recipient,
            &sender,
            &recipient_pub,
            &eph_seed,
            &wrap_nonce,
        )
        .expect("seeded wrap");
        // eph_pub from seed [0x11;32]
        let eph_secret = StaticSecret::from(eph_seed);
        let expected_eph = X25519Public::from(&eph_secret).to_bytes();
        assert_eq!(wrap.eph_x25519_pub, expected_eph);
        assert_eq!(wrap.wrap_nonce, wrap_nonce);
        assert_eq!(wrap.wrap_ct.len(), DEK_LEN + 16); // ct + tag
        // Pinned wrap_ct (AES-GCM under ADR §17 HKDF+AAD).
        assert_eq!(
            hex::encode(&wrap.wrap_ct),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/kats/wrap_seeded_ct.hex"
            ))
            .trim()
        );
        // Round-trip under same static.
        let opened = unwrap_content_dek(
            REPLICATION_SCHEMA_VERSION,
            &wrap,
            &content_key,
            &recipient,
            &sender,
            &recipient_static,
        )
        .expect("unwrap");
        assert_eq!(opened, dek);
    }

    /// T178-WRAP-per-recipient-roundtrip
    #[test]
    fn t178_wrap_per_recipient_roundtrip__ok() {
        // T178-WRAP-per-recipient-roundtrip
        wrap_dek__roundtrip_recipient__ok();
    }

    /// T178-WRAP-wrong-recipient-fails
    #[test]
    fn t178_wrap_wrong_recipient_fails__err() {
        // T178-WRAP-wrong-recipient-fails
        wrap_dek__wrong_static_key__err();
    }

    /// T178-WRAP-no-shared-datakey-over-relay — structural encode fields only.
    #[test]
    fn t178_wrap_no_shared_datakey_over_relay__structural() {
        // T178-WRAP-no-shared-datakey-over-relay
        use crate::signed_bytes::{WrapRecord, encode_wrap_record};
        let recipient_keys = generate_device_keys().expect("recip");
        let content_key = ContentKeyId::from_uuid(fixed_id(10));
        let recipient_id = DeviceId::from_uuid(fixed_id(20));
        let sender_id = DeviceId::from_uuid(fixed_id(30));
        let dek = [0xEFu8; 32];
        let wrap = wrap_content_dek_for_recipient(
            REPLICATION_SCHEMA_VERSION,
            &dek,
            &content_key,
            &recipient_id,
            &sender_id,
            &recipient_keys.x25519_public(),
        )
        .expect("wrap");
        assert_eq!(wrap.wrap_ct.len(), DEK_LEN + 16);
        let rec = WrapRecord {
            recipient_device_id: recipient_id,
            eph_x25519_pub: wrap.eph_x25519_pub,
            wrap_nonce: wrap.wrap_nonce,
            wrap_ct: wrap.wrap_ct.clone(),
        };
        let encoded = encode_wrap_record(&rec);
        // recipient(16) ‖ eph(32) ‖ nonce(12) ‖ ct_len u32 BE ‖ wrap_ct
        assert_eq!(encoded.len(), 16 + 32 + 12 + 4 + wrap.wrap_ct.len());
        // No clear DEK / DataKey: encoded body after header must not equal plain DEK.
        let ct_start = 16 + 32 + 12 + 4;
        assert_ne!(&encoded[ct_start..ct_start + 32], &dek);
        // Field inventory: only recipient‖eph‖nonce‖ct — no separate DataKey field.
        assert_eq!(&encoded[0..16], recipient_id.as_uuid().as_bytes());
        assert_eq!(&encoded[16..48], &wrap.eph_x25519_pub);
        assert_eq!(&encoded[48..60], &wrap.wrap_nonce);
    }

    /// T178-WRAP-nonce-uniqueness (Should): N=100 wraps, all nonces distinct.
    #[test]
    fn t178_wrap_nonce_uniqueness__n100_distinct() {
        // T178-WRAP-nonce-uniqueness
        use std::collections::HashSet;
        let recipient_keys = generate_device_keys().expect("recip");
        let content_key = ContentKeyId::from_uuid(fixed_id(10));
        let recipient_id = DeviceId::from_uuid(fixed_id(20));
        let sender_id = DeviceId::from_uuid(fixed_id(30));
        let dek = [0x01u8; 32];
        let mut nonces = HashSet::new();
        for _ in 0..100 {
            let wrap = wrap_content_dek_for_recipient(
                REPLICATION_SCHEMA_VERSION,
                &dek,
                &content_key,
                &recipient_id,
                &sender_id,
                &recipient_keys.x25519_public(),
            )
            .expect("wrap");
            assert!(
                nonces.insert(wrap.wrap_nonce),
                "duplicate wrap nonce observed"
            );
        }
        assert_eq!(nonces.len(), 100);
    }
}
