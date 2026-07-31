//! Device dual-keypair generation (R25 panic-free keygen).
//!
//! Independent Ed25519 signing seed and X25519 static seed — never convert
//! signing material to DH material. Secrets zeroize on drop; Debug redacts.

use crate::error::{Result, SyncError};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::TryRng;
use rand::rngs::SysRng;
use std::fmt;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 32-byte seed for Ed25519 or X25519 construction.
pub const SEED_LEN: usize = 32;

/// Fallible dual keypair for one device (Ed25519 + X25519).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DeviceKeyPair {
    /// Independent Ed25519 seed (signing).
    pub ed25519_seed: [u8; SEED_LEN],
    /// Independent X25519 seed (static DH).
    pub x25519_seed: [u8; SEED_LEN],
}

impl DeviceKeyPair {
    /// Ed25519 signing key from the stored seed.
    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.ed25519_seed)
    }

    /// Ed25519 verifying (public) key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key().verifying_key()
    }

    /// X25519 static secret from the stored seed.
    pub fn x25519_secret(&self) -> StaticSecret {
        StaticSecret::from(self.x25519_seed)
    }

    /// X25519 public key.
    pub fn x25519_public(&self) -> X25519Public {
        X25519Public::from(&self.x25519_secret())
    }

    /// Reconstruct from raw seeds (e.g. after private-blob open).
    pub fn from_seeds(ed25519_seed: [u8; SEED_LEN], x25519_seed: [u8; SEED_LEN]) -> Self {
        Self {
            ed25519_seed,
            x25519_seed,
        }
    }
}

impl fmt::Debug for DeviceKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceKeyPair([REDACTED])")
    }
}

/// Generate independent Ed25519 + X25519 seeds via fallible OS CSPRNG (R25).
///
/// Uses `SysRng.try_fill_bytes` then `SigningKey::from_bytes` /
/// `StaticSecret::from`. Never panics on entropy failure.
pub fn generate_device_keys() -> Result<DeviceKeyPair> {
    let mut ed25519_seed = [0u8; SEED_LEN];
    let mut x25519_seed = [0u8; SEED_LEN];
    SysRng
        .try_fill_bytes(&mut ed25519_seed)
        .map_err(|e| SyncError::EntropyFailed(format!("ed25519 seed: {e}")))?;
    SysRng
        .try_fill_bytes(&mut x25519_seed)
        .map_err(|e| SyncError::EntropyFailed(format!("x25519 seed: {e}")))?;
    // Construct once to validate APIs (from_bytes / from never fail for 32B).
    let _sk = SigningKey::from_bytes(&ed25519_seed);
    let _xs = StaticSecret::from(x25519_seed);
    Ok(DeviceKeyPair {
        ed25519_seed,
        x25519_seed,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn device_keys__try_fill__no_unwrap_err() {
        let keys = generate_device_keys().expect("generate");
        assert_ne!(keys.ed25519_seed, [0u8; 32]);
        assert_ne!(keys.x25519_seed, [0u8; 32]);
        // Independent seeds (overwhelmingly likely unequal).
        assert_ne!(keys.ed25519_seed, keys.x25519_seed);
        let debug = format!("{:?}", keys);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(&hex::encode(keys.ed25519_seed)));
    }

    #[test]
    fn device_keys__generate__unique_pairs() {
        let a = generate_device_keys().expect("a");
        let b = generate_device_keys().expect("b");
        assert_ne!(a.ed25519_seed, b.ed25519_seed);
        assert_ne!(a.x25519_seed, b.x25519_seed);
    }
}
