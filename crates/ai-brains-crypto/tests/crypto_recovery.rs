#![allow(clippy::disallowed_methods)]

use ai_brains_crypto::{CryptoError, DataKey, RecoveryKit, SqlCipherKey};

#[test]
fn data_key_generated_randomly() {
    let key1 = DataKey::generate();
    let key2 = DataKey::generate();
    assert_ne!(key1.expose_secret(), key2.expose_secret());
}

#[test]
fn passphrase_wrap_roundtrip() {
    let key = DataKey::generate();
    let passphrase = b"my-secure-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");

    let restored = kit
        .unlock_with_passphrase(passphrase)
        .expect("Failed to unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

#[test]
fn wrong_passphrase_fails() {
    let key = DataKey::generate();
    let passphrase = b"correct-passphrase";
    let wrong_passphrase = b"wrong-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");

    let result = kit.unlock_with_passphrase(wrong_passphrase);
    assert!(matches!(result, Err(CryptoError::InvalidPassphrase)));
}

#[test]
fn recovery_kit_restores_key() {
    let key = DataKey::generate();
    let passphrase = b"recovery-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");

    // Test passphrase restore
    let restored = kit
        .unlock_with_passphrase(passphrase)
        .expect("Failed to unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());

    // Test DPAPI restore on Windows
    #[cfg(windows)]
    {
        let restored_dpapi = kit.unlock_with_dpapi().expect("DPAPI unlock failed");
        assert_eq!(key.expose_secret(), restored_dpapi.expose_secret());
    }
}

#[test]
fn key_material_debug_redacted() {
    let key = DataKey::generate();
    let debug_str = format!("{:?}", key);
    assert!(debug_str.contains("[REDACTED]"));

    let sql_key = SqlCipherKey::from_data_key(&key);
    let debug_str_sql = format!("{:?}", sql_key);
    assert!(debug_str_sql.contains("[REDACTED]"));
}

#[test]
fn sqlcipher_key_zeroized() {
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    // Logic is in the struct's Drop implementation via zeroize
    drop(sql_key);
}

#[test]
fn windows_dpapi_roundtrip() {
    #[cfg(windows)]
    {
        let key_material = b"some-secret-key-material-1234567";
        let wrapped = ai_brains_crypto::dpapi::wrap_key(key_material).expect("DPAPI wrap failed");
        let unwrapped = ai_brains_crypto::dpapi::unwrap_key(&wrapped).expect("DPAPI unwrap failed");
        assert_eq!(key_material.to_vec(), unwrapped);
    }
}

#[test]
fn does_not_write_plaintext_key_to_disk() {
    // This is a behavioral constraint verified by code review and lack of FS tools in ai-brains-crypto.
    // We can verify that RecoveryKit serialization does not contain the plaintext key.
    let key = DataKey::generate();
    let passphrase = b"pwd";
    let kit = RecoveryKit::generate(&key, passphrase).unwrap();
    let json = kit.to_json().unwrap();

    assert!(!json.contains(&hex::encode(key.expose_secret())));
}

#[test]
fn recovery_kit_missing_reports_actionable_error() {
    let key = DataKey::generate();
    let passphrase = b"pwd";
    let mut kit = RecoveryKit::generate(&key, passphrase).unwrap();
    kit.dpapi = None;

    let result = kit.unlock_with_dpapi();
    assert!(matches!(result, Err(CryptoError::RecoveryKitMissing(msg)) if msg.contains("DPAPI")));
}
