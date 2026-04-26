use crate::data_key::{DataKey, KEY_LEN};
use crate::dpapi;
use crate::errors::{CryptoError, Result};
use crate::key_wrap::{DpapiWrappedKey, PassphraseWrappedKey};
use crate::passphrase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryKit {
    pub dpapi: Option<DpapiWrappedKey>,
    pub passphrase: PassphraseWrappedKey,
}

impl RecoveryKit {
    /// Create a new RecoveryKit by wrapping a DataKey with both DPAPI and a passphrase.
    pub fn generate(data_key: &DataKey, passphrase_bytes: &[u8]) -> Result<Self> {
        let dpapi_wrapped = dpapi::wrap_key(data_key.expose_secret())
            .map(|ciphertext| DpapiWrappedKey { ciphertext })
            .ok();

        let (ciphertext, salt, nonce) =
            passphrase::wrap_key(data_key.expose_secret(), passphrase_bytes)?;
        let passphrase_wrapped = PassphraseWrappedKey {
            ciphertext,
            salt,
            nonce,
        };

        Ok(Self {
            dpapi: dpapi_wrapped,
            passphrase: passphrase_wrapped,
        })
    }

    /// Attempt to restore the DataKey using DPAPI.
    pub fn unlock_with_dpapi(&self) -> Result<DataKey> {
        let wrapped = self
            .dpapi
            .as_ref()
            .ok_or_else(|| CryptoError::RecoveryKitMissing("DPAPI wrapped key".to_string()))?;

        let material = dpapi::unwrap_key(&wrapped.ciphertext)?;
        if material.len() != KEY_LEN {
            return Err(CryptoError::InvalidKeyLength);
        }

        let mut key_bytes = [0u8; KEY_LEN];
        key_bytes.copy_from_slice(&material);
        Ok(DataKey::from_bytes(key_bytes))
    }

    /// Attempt to restore the DataKey using a passphrase.
    pub fn unlock_with_passphrase(&self, passphrase_bytes: &[u8]) -> Result<DataKey> {
        let material = passphrase::unwrap_key(
            &self.passphrase.ciphertext,
            passphrase_bytes,
            &self.passphrase.salt,
            &self.passphrase.nonce,
        )?;

        if material.len() != KEY_LEN {
            return Err(CryptoError::InvalidKeyLength);
        }

        let mut key_bytes = [0u8; KEY_LEN];
        key_bytes.copy_from_slice(&material);
        Ok(DataKey::from_bytes(key_bytes))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| CryptoError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| CryptoError::DeserializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    use super::*;

    #[test]
    fn recovery_kit_restores_key() {
        let key = DataKey::generate();
        let passphrase = b"recovery passphrase";
        let kit = RecoveryKit::generate(&key, passphrase).expect("Kit generation failed");

        // Restore via passphrase
        let restored = kit
            .unlock_with_passphrase(passphrase)
            .expect("Unlock failed");
        assert_eq!(key.expose_secret(), restored.expose_secret());

        // Restore via DPAPI (if on Windows)
        #[cfg(windows)]
        {
            let restored_dpapi = kit.unlock_with_dpapi().expect("DPAPI unlock failed");
            assert_eq!(key.expose_secret(), restored_dpapi.expose_secret());
        }
    }

    #[test]
    fn recovery_kit_missing_reports_actionable_error() {
        let key = DataKey::generate();
        let passphrase = b"pwd";
        let mut kit = RecoveryKit::generate(&key, passphrase).unwrap();
        kit.dpapi = None;

        let result = kit.unlock_with_dpapi();
        assert!(matches!(result, Err(CryptoError::RecoveryKitMissing(_))));
    }
}
