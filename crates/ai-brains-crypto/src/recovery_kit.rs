use crate::data_key::{DataKey, KEY_LEN};
use crate::dpapi;
use crate::errors::{CryptoError, Result};
use crate::key_wrap::{DpapiWrappedKey, KdfParams, PassphraseWrappedKey};
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
    /// T194 adds optional `passphrase.kdf` without bumping this (F10).
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub dpapi: Option<DpapiWrappedKey>,
    pub passphrase: PassphraseWrappedKey,
}

impl RecoveryKit {
    /// Create a new RecoveryKit by wrapping a DataKey with both DPAPI and a passphrase.
    ///
    /// Always stamps `passphrase.kdf` with [`KdfParams::product_generation()`] (T194 F11).
    pub fn generate(data_key: &DataKey, passphrase_bytes: &[u8]) -> Result<Self> {
        let dpapi_wrapped = dpapi::wrap_key(data_key.expose_secret())
            .map(|ciphertext| DpapiWrappedKey { ciphertext })
            .ok();

        let kdf = KdfParams::product_generation();
        let (ciphertext, salt, nonce) =
            passphrase::wrap_key(data_key.expose_secret(), passphrase_bytes, &kdf)?;
        let passphrase_wrapped = PassphraseWrappedKey {
            ciphertext,
            salt,
            nonce,
            kdf: Some(kdf),
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
    ///
    /// Effective KDF params = stored `passphrase.kdf` or [`KdfParams::legacy()`] (F9/F13).
    pub fn unlock_with_passphrase(&self, passphrase_bytes: &[u8]) -> Result<DataKey> {
        let effective = self
            .passphrase
            .kdf
            .clone()
            .unwrap_or_else(KdfParams::legacy);

        let material = passphrase::unwrap_key(
            &self.passphrase.ciphertext,
            passphrase_bytes,
            &self.passphrase.salt,
            &self.passphrase.nonce,
            &effective,
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
    use crate::key_wrap::{ALGORITHM_ARGON2ID, LEGACY_M_COST, LEGACY_P_COST, LEGACY_T_COST};

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

    /// T194 AC1: generate embeds kdf with product params.
    #[test]
    fn recovery_kit__generate__embeds_kdf_params() {
        let key = DataKey::generate();
        let kit = RecoveryKit::generate(&key, b"kdf-embed-pass").unwrap();
        let kdf = kit.passphrase.kdf.as_ref().expect("kdf must be Some");
        assert_eq!(kdf.algorithm, ALGORITHM_ARGON2ID);
        assert_eq!(kdf.version, 19);
        assert_eq!(kdf.m_cost, LEGACY_M_COST);
        assert_eq!(kdf.t_cost, LEGACY_T_COST);
        assert_eq!(kdf.p_cost, LEGACY_P_COST);

        let json = kit.to_json().unwrap();
        assert!(json.contains("\"kdf\""));
        assert!(
            json.contains("\"algorithm\":\"argon2id\"")
                || json.contains("\"algorithm\": \"argon2id\"")
        );
        assert!(json.contains("\"m_cost\":19456") || json.contains("\"m_cost\": 19456"));
        assert!(json.contains("\"t_cost\":2") || json.contains("\"t_cost\": 2"));
        assert!(json.contains("\"p_cost\":1") || json.contains("\"p_cost\": 1"));
        assert!(json.contains("\"version\":19") || json.contains("\"version\": 19"));
    }

    /// T194 AC2/F29: unlock uses stored params, not LEGACY alone.
    #[test]
    fn recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy() {
        let key = DataKey::generate();
        let passphrase = b"non-default-kdf-pass";
        let custom = KdfParams {
            algorithm: ALGORITHM_ARGON2ID.into(),
            version: 19,
            m_cost: 12_288,
            t_cost: 3,
            p_cost: 1,
        };
        let (ciphertext, salt, nonce) =
            passphrase::wrap_key(key.expose_secret(), passphrase, &custom).expect("wrap");

        let kit = RecoveryKit {
            schema_version: 1,
            dpapi: None,
            passphrase: PassphraseWrappedKey {
                ciphertext: ciphertext.clone(),
                salt,
                nonce,
                kdf: Some(custom.clone()),
            },
        };

        let restored = kit
            .unlock_with_passphrase(passphrase)
            .expect("unlock with stored non-default params");
        assert_eq!(key.expose_secret(), restored.expose_secret());

        // Same ciphertext under LEGACY must fail (proves stored params path).
        let legacy_fail =
            passphrase::unwrap_key(&ciphertext, passphrase, &salt, &nonce, &KdfParams::legacy());
        assert!(
            matches!(legacy_fail, Err(CryptoError::InvalidPassphrase)),
            "LEGACY params must not unlock non-default wrap: {legacy_fail:?}"
        );
    }

    /// T194 AC3: kit without kdf unlocks via LEGACY constants.
    #[test]
    fn recovery_kit__legacy_json_without_kdf__unlocks_with_legacy_defaults() {
        let key = DataKey::generate();
        let passphrase = b"legacy-no-kdf-pass";
        let kit = RecoveryKit::generate(&key, passphrase).expect("generate");
        let full = kit.to_json().expect("json");
        let mut v: serde_json::Value = serde_json::from_str(&full).expect("parse");
        let passphrase_obj = v
            .get_mut("passphrase")
            .and_then(|p| p.as_object_mut())
            .expect("passphrase object");
        passphrase_obj.remove("kdf");
        let legacy = serde_json::to_string(&v).expect("reserialize");
        assert!(
            !legacy.contains("\"kdf\""),
            "fixture must omit kdf: {legacy}"
        );

        let parsed = RecoveryKit::from_json(&legacy).expect("deserialize without kdf");
        assert!(parsed.passphrase.kdf.is_none());
        let restored = parsed
            .unlock_with_passphrase(passphrase)
            .expect("legacy dual-read unlock");
        assert_eq!(key.expose_secret(), restored.expose_secret());
    }

    /// T194 AC3: both schema_version and kdf absent still unlock.
    #[test]
    fn recovery_kit__legacy_json_without_schema_and_kdf__unlocks() {
        let key = DataKey::generate();
        let passphrase = b"legacy-both-missing";
        let kit = RecoveryKit::generate(&key, passphrase).expect("generate");
        let full = kit.to_json().expect("json");
        let mut v: serde_json::Value = serde_json::from_str(&full).expect("parse");
        let obj = v.as_object_mut().expect("object");
        obj.remove("schema_version");
        obj.get_mut("passphrase")
            .and_then(|p| p.as_object_mut())
            .expect("passphrase")
            .remove("kdf");
        let legacy = serde_json::to_string(&v).expect("reserialize");
        assert!(!legacy.contains("schema_version"));
        assert!(!legacy.contains("\"kdf\""));

        let parsed = RecoveryKit::from_json(&legacy).expect("deserialize");
        assert_eq!(parsed.schema_version, 1);
        assert!(parsed.passphrase.kdf.is_none());
        let restored = parsed.unlock_with_passphrase(passphrase).expect("unlock");
        assert_eq!(key.expose_secret(), restored.expose_secret());
    }

    /// T194 F21: partial kdf object (missing m_cost) fails deserialize.
    #[test]
    fn recovery_kit__partial_kdf_object__deserialize_fails() {
        let key = DataKey::generate();
        let kit = RecoveryKit::generate(&key, b"partial-kdf").unwrap();
        let full = kit.to_json().unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&full).unwrap();
        let kdf = v
            .get_mut("passphrase")
            .and_then(|p| p.as_object_mut())
            .and_then(|p| p.get_mut("kdf"))
            .and_then(|k| k.as_object_mut())
            .expect("kdf object");
        kdf.remove("m_cost");
        let bad = serde_json::to_string(&v).unwrap();
        let result = RecoveryKit::from_json(&bad);
        assert!(
            matches!(result, Err(CryptoError::DeserializationError(_))),
            "partial kdf must fail closed: {result:?}"
        );
    }
}
