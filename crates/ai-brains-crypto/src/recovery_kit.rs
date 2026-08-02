use crate::data_key::{DataKey, KEY_LEN};
use crate::dpapi;
use crate::errors::{CryptoError, Result};
use crate::key_wrap::{DpapiWrappedKey, PassphraseWrappedKey};
use crate::passphrase;
use serde::{Deserialize, Serialize};

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryKit {
    /// Kit schema version for forward-compat (T188 F19 / T189).
    ///
    /// Defaults to **1** when absent so older kits without the field still deserialize.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
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
            schema_version: default_schema_version(),
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
    #![allow(non_snake_case)]
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

    /// T188 F19: kits generated today pin schema_version=1.
    #[test]
    fn recovery_kit__generate__schema_version_is_1() {
        let key = DataKey::generate();
        let kit = RecoveryKit::generate(&key, b"schema-version-test").unwrap();
        assert_eq!(kit.schema_version, 1);
        let json = kit.to_json().unwrap();
        assert!(json.contains("\"schema_version\":1") || json.contains("\"schema_version\": 1"));
    }

    /// T188 F19: old JSON without schema_version deserializes to 1.
    #[test]
    fn recovery_kit__old_json_without_schema_version__defaults_to_1() {
        let key = DataKey::generate();
        let mut kit = RecoveryKit::generate(&key, b"legacy-kit-pass").unwrap();
        // Build a minimal legacy-shaped JSON without schema_version.
        kit.schema_version = 1;
        let full = kit.to_json().unwrap();
        // Drop the schema_version field if present.
        let v: serde_json::Value = serde_json::from_str(&full).unwrap();
        let mut obj = v.as_object().cloned().expect("object");
        obj.remove("schema_version");
        let legacy = serde_json::to_string(&obj).unwrap();
        assert!(
            !legacy.contains("schema_version"),
            "fixture must omit schema_version: {legacy}"
        );

        let parsed = RecoveryKit::from_json(&legacy).expect("legacy kit must deserialize");
        assert_eq!(parsed.schema_version, 1);
        let restored = parsed
            .unlock_with_passphrase(b"legacy-kit-pass")
            .expect("unlock");
        assert_eq!(key.expose_secret(), restored.expose_secret());
    }
}
