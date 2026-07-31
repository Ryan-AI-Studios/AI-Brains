//! Device private-key inner blob seal under vault DataKey (§5.1.1 / R28).
//!
//! ```text
//! plaintext = ed25519_seed[32] ‖ x25519_seed[32]
//! aad = b"AIBC" ‖ 0x03 ‖ wrap_schema_version u32 BE ‖ device_id 16
//! ```
//!
//! Windows: outer DPAPI on ct‖tag via `ai_brains_crypto::dpapi`; keep wrap_nonce clear.

use crate::device_keys::{DeviceKeyPair, SEED_LEN};
use crate::error::{Result, SyncError};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use ai_brains_core::ids::DeviceId;
use ai_brains_crypto::DataKey;
use ai_brains_crypto::content_key_store::AAD_MAGIC;
use rand::TryRng;
use rand::rngs::SysRng;
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// AAD kind for device private key seal (distinct from content 0x01 / DEK wrap 0x02).
pub const AAD_KIND_DEVICE_PRIVATE_KEY: u8 = 0x03;

/// wrap_schema_version v1.
pub const DEVICE_PRIVATE_WRAP_SCHEMA_VERSION: u32 = 1;

/// Plaintext length: two 32-byte seeds.
pub const DEVICE_PRIVATE_PLAINTEXT_LEN: usize = SEED_LEN * 2;

pub const NONCE_LEN: usize = 12;

/// Protection labels stored in `device_private_key_store.protection`.
pub const PROTECTION_DATAKEY: &str = "datakey";
pub const PROTECTION_DATAKEY_DPAPI: &str = "datakey_dpapi";

/// Zeroizing dual seeds (same layout as DeviceKeyPair seeds).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DevicePrivateSeeds {
    pub ed25519_seed: [u8; SEED_LEN],
    pub x25519_seed: [u8; SEED_LEN],
}

impl DevicePrivateSeeds {
    pub fn from_key_pair(keys: &DeviceKeyPair) -> Self {
        Self {
            ed25519_seed: keys.ed25519_seed,
            x25519_seed: keys.x25519_seed,
        }
    }

    pub fn into_key_pair(self) -> DeviceKeyPair {
        DeviceKeyPair::from_seeds(self.ed25519_seed, self.x25519_seed)
    }

    fn plaintext_bytes(&self) -> [u8; DEVICE_PRIVATE_PLAINTEXT_LEN] {
        let mut out = [0u8; DEVICE_PRIVATE_PLAINTEXT_LEN];
        out[..SEED_LEN].copy_from_slice(&self.ed25519_seed);
        out[SEED_LEN..].copy_from_slice(&self.x25519_seed);
        out
    }

    fn from_plaintext(bytes: &[u8; DEVICE_PRIVATE_PLAINTEXT_LEN]) -> Self {
        let mut ed = [0u8; SEED_LEN];
        let mut x = [0u8; SEED_LEN];
        ed.copy_from_slice(&bytes[..SEED_LEN]);
        x.copy_from_slice(&bytes[SEED_LEN..]);
        Self {
            ed25519_seed: ed,
            x25519_seed: x,
        }
    }
}

impl fmt::Debug for DevicePrivateSeeds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DevicePrivateSeeds([REDACTED])")
    }
}

/// Sealed device private material for SQL persistence.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedDevicePrivate {
    pub wrap_schema_version: u32,
    pub protection: String,
    pub wrap_nonce: [u8; NONCE_LEN],
    /// ct‖tag under DataKey, or DPAPI(ct‖tag) when protection = datakey_dpapi.
    pub wrap_ciphertext: Vec<u8>,
}

impl fmt::Debug for SealedDevicePrivate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SealedDevicePrivate")
            .field("wrap_schema_version", &self.wrap_schema_version)
            .field("protection", &self.protection)
            .field("wrap_nonce_len", &self.wrap_nonce.len())
            .field("wrap_ciphertext_len", &self.wrap_ciphertext.len())
            .finish()
    }
}

/// Build AAD for device private-key seal (§5.1.1).
pub fn build_device_private_aad(wrap_schema_version: u32, device_id: &DeviceId) -> Vec<u8> {
    let mut aad = Vec::with_capacity(4 + 1 + 4 + 16);
    aad.extend_from_slice(AAD_MAGIC);
    aad.push(AAD_KIND_DEVICE_PRIVATE_KEY);
    aad.extend_from_slice(&wrap_schema_version.to_be_bytes());
    aad.extend_from_slice(device_id.as_uuid().as_bytes());
    aad
}

/// Seal dual seeds under vault DataKey. On Windows, also DPAPI-protect ct‖tag.
pub fn seal_device_private_blob(
    data_key: &DataKey,
    seeds: &DevicePrivateSeeds,
    device_id: &DeviceId,
) -> Result<SealedDevicePrivate> {
    let mut nonce = [0u8; NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce)
        .map_err(|e| SyncError::EntropyFailed(format!("device private nonce: {e}")))?;

    let aad = build_device_private_aad(DEVICE_PRIVATE_WRAP_SCHEMA_VERSION, device_id);
    let mut plaintext = seeds.plaintext_bytes();
    let cipher = Aes256Gcm::new_from_slice(data_key.expose_secret())
        .map_err(|e| SyncError::Crypto(e.to_string()))?;
    let ct = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext.as_slice(),
                aad: &aad,
            },
        )
        .map_err(|e| SyncError::Crypto(format!("device private seal: {e}")))?;
    plaintext.zeroize();

    #[cfg(windows)]
    {
        let protected = ai_brains_crypto::dpapi::wrap_key(&ct)
            .map_err(|e| SyncError::Crypto(format!("DPAPI wrap: {e}")))?;
        Ok(SealedDevicePrivate {
            wrap_schema_version: DEVICE_PRIVATE_WRAP_SCHEMA_VERSION,
            protection: PROTECTION_DATAKEY_DPAPI.to_string(),
            wrap_nonce: nonce,
            wrap_ciphertext: protected,
        })
    }
    #[cfg(not(windows))]
    {
        Ok(SealedDevicePrivate {
            wrap_schema_version: DEVICE_PRIVATE_WRAP_SCHEMA_VERSION,
            protection: PROTECTION_DATAKEY.to_string(),
            wrap_nonce: nonce,
            wrap_ciphertext: ct,
        })
    }
}

/// Open sealed device private blob (DPAPI then AES-GCM on Windows when labeled).
pub fn open_device_private_blob(
    data_key: &DataKey,
    sealed: &SealedDevicePrivate,
    device_id: &DeviceId,
) -> Result<DevicePrivateSeeds> {
    let ct_tag = if sealed.protection == PROTECTION_DATAKEY_DPAPI {
        ai_brains_crypto::dpapi::unwrap_key(&sealed.wrap_ciphertext)
            .map_err(|e| SyncError::Crypto(format!("DPAPI unwrap: {e}")))?
    } else if sealed.protection == PROTECTION_DATAKEY {
        sealed.wrap_ciphertext.clone()
    } else {
        return Err(SyncError::InvalidEncoding(format!(
            "unknown protection: {}",
            sealed.protection
        )));
    };

    let aad = build_device_private_aad(sealed.wrap_schema_version, device_id);
    let cipher = Aes256Gcm::new_from_slice(data_key.expose_secret())
        .map_err(|e| SyncError::Crypto(e.to_string()))?;
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                Nonce::from_slice(&sealed.wrap_nonce),
                Payload {
                    msg: &ct_tag,
                    aad: &aad,
                },
            )
            .map_err(|_| SyncError::AuthenticationFailed)?,
    );
    let arr: [u8; DEVICE_PRIVATE_PLAINTEXT_LEN] = plaintext
        .as_slice()
        .try_into()
        .map_err(|_| SyncError::AuthenticationFailed)?;
    Ok(DevicePrivateSeeds::from_plaintext(&arr))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use crate::device_keys::generate_device_keys;
    use uuid::Uuid;

    #[test]
    fn device_private_blob__aad__kind_0x03() {
        let device = DeviceId::from_uuid(
            Uuid::parse_str("aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee").expect("uuid"),
        );
        let aad = build_device_private_aad(DEVICE_PRIVATE_WRAP_SCHEMA_VERSION, &device);
        assert_eq!(aad.len(), 4 + 1 + 4 + 16);
        assert_eq!(&aad[0..4], b"AIBC");
        assert_eq!(aad[4], AAD_KIND_DEVICE_PRIVATE_KEY);
        assert_eq!(aad[4], 0x03);
        assert_eq!(&aad[5..9], &1u32.to_be_bytes());
        assert_eq!(&aad[9..25], device.as_uuid().as_bytes());
    }

    #[test]
    fn device_private_blob__seal_open__roundtrip() {
        let data_key = DataKey::generate();
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::from_uuid(Uuid::new_v4());
        let seeds = DevicePrivateSeeds::from_key_pair(&keys);
        let sealed = seal_device_private_blob(&data_key, &seeds, &device).expect("seal");
        #[cfg(windows)]
        assert_eq!(sealed.protection, PROTECTION_DATAKEY_DPAPI);
        #[cfg(not(windows))]
        assert_eq!(sealed.protection, PROTECTION_DATAKEY);
        let opened = open_device_private_blob(&data_key, &sealed, &device).expect("open");
        assert_eq!(opened.ed25519_seed, keys.ed25519_seed);
        assert_eq!(opened.x25519_seed, keys.x25519_seed);
    }
}
