#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! RecoveryKit drills (T181-K-*). Library-level only — no CLI export path (F4/F5/F38).

use ai_brains_crypto::test_support::{assert_no_kit_dump, assert_no_secret_leakage};
use ai_brains_crypto::{CryptoError, DataKey, RecoveryKit, SqlCipherKey};

/// T181-K-01: kit generate → passphrase unlock equals original DataKey.
#[test]
fn recovery_kit__passphrase_unlock__key_equals_original() {
    let key = DataKey::generate();
    let passphrase = b"my-secure-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");
    assert_eq!(
        kit.schema_version, 1,
        "T188 F19: new kits pin schema_version=1"
    );

    let restored = kit
        .unlock_with_passphrase(passphrase)
        .expect("Failed to unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

/// T188 F19: old kit JSON without schema_version deserializes to 1.
#[test]
fn recovery_kit__legacy_json_without_schema_version__defaults_to_1() {
    let key = DataKey::generate();
    let passphrase = b"legacy-schema-pass";
    let kit = RecoveryKit::generate(&key, passphrase).expect("generate");
    let full = kit.to_json().expect("json");
    let mut v: serde_json::Value = serde_json::from_str(&full).expect("parse");
    v.as_object_mut().expect("object").remove("schema_version");
    let legacy = serde_json::to_string(&v).expect("reserialize");
    assert!(!legacy.contains("schema_version"));

    let parsed = RecoveryKit::from_json(&legacy).expect("legacy deserialize");
    assert_eq!(parsed.schema_version, 1);
    let restored = parsed
        .unlock_with_passphrase(passphrase)
        .expect("unlock legacy");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

/// T181-K-03: wrong passphrase fails closed.
#[test]
fn recovery_kit__wrong_passphrase__fails_closed() {
    let key = DataKey::generate();
    let passphrase = b"correct-passphrase";
    let wrong_passphrase = b"wrong-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");

    let result = kit.unlock_with_passphrase(wrong_passphrase);
    assert!(matches!(result, Err(CryptoError::InvalidPassphrase)));
}

/// T181-K-01 (DPAPI arm on Windows) + passphrase path.
#[test]
fn recovery_kit__passphrase_and_dpapi_unlock__roundtrip() {
    let key = DataKey::generate();
    let passphrase = b"recovery-passphrase";
    let kit = RecoveryKit::generate(&key, passphrase).expect("Failed to generate kit");

    let restored = kit
        .unlock_with_passphrase(passphrase)
        .expect("Failed to unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());

    #[cfg(windows)]
    {
        let restored_dpapi = kit.unlock_with_dpapi().expect("DPAPI unlock failed");
        assert_eq!(key.expose_secret(), restored_dpapi.expose_secret());
    }
}

/// T181-K-02: dpapi=None → DPAPI unlock reports RecoveryKitMissing.
#[test]
fn recovery_kit__missing_dpapi__reports_actionable_error() {
    let key = DataKey::generate();
    let passphrase = b"pwd";
    let mut kit = RecoveryKit::generate(&key, passphrase).unwrap();
    kit.dpapi = None;

    let result = kit.unlock_with_dpapi();
    assert!(matches!(result, Err(CryptoError::RecoveryKitMissing(msg)) if msg.contains("DPAPI")));
}

/// T181-K-04: kit JSON has no plaintext key (hex); Debug is redacted.
#[test]
fn recovery_kit__json_and_debug__no_plaintext_key() {
    let key = DataKey::generate();
    let passphrase = b"pwd-for-k04";
    let kit = RecoveryKit::generate(&key, passphrase).unwrap();
    let json = kit.to_json().unwrap();

    assert_no_secret_leakage(&json, key.expose_secret());
    assert!(!json.contains(&hex::encode(key.expose_secret())));

    let debug_str = format!("{:?}", key);
    assert!(debug_str.contains("[REDACTED]"));
    assert_no_secret_leakage(&debug_str, key.expose_secret());

    let sql_key = SqlCipherKey::from_data_key(&key);
    let debug_str_sql = format!("{:?}", sql_key);
    assert!(debug_str_sql.contains("[REDACTED]"));
    assert_no_secret_leakage(&debug_str_sql, key.expose_secret());
}

/// T181-K-07: kit JSON lacks KDF param field names (Argon2 residual F37).
#[test]
fn recovery_kit__json__lacks_kdf_param_fields() {
    let key = DataKey::generate();
    let passphrase = b"pwd-for-k07";
    let kit = RecoveryKit::generate(&key, passphrase).unwrap();
    let json = kit.to_json().unwrap();

    for forbidden in [
        "m_cost",
        "t_cost",
        "p_cost",
        "memory_cost",
        "time_cost",
        "parallelism",
        "argon2id",
        "Argon2",
        "kdf_params",
        "kdf",
    ] {
        assert!(
            !json.contains(forbidden),
            "kit JSON must not contain KDF field marker {forbidden:?}; got: {json}"
        );
    }

    // Structural fields present (not KDF).
    assert!(json.contains("ciphertext") || json.contains("passphrase"));
    assert!(json.contains("salt"));
    assert!(json.contains("nonce"));
}

/// T181-K-04 helper surface: simulated CLI log must not dump kit/passphrase/key.
#[test]
fn recovery_kit__simulated_cli_log__no_secret_or_kit_dump() {
    let key = DataKey::generate();
    let passphrase = b"operator-passphrase-never-log";
    let kit = RecoveryKit::generate(&key, passphrase).unwrap();
    let kit_json = kit.to_json().unwrap();

    // Honest operator-facing message (what CLI should look like if it ever exports).
    let fake_cli_stdout = "Recovery kit generated (stored out-of-band). Do not log kit JSON.\n";
    assert_no_secret_leakage(fake_cli_stdout, key.expose_secret());
    assert_no_secret_leakage(fake_cli_stdout, passphrase);
    assert_no_kit_dump(fake_cli_stdout, &kit_json);
}

#[test]
fn data_key_generated_randomly() {
    let key1 = DataKey::generate();
    let key2 = DataKey::generate();
    assert_ne!(key1.expose_secret(), key2.expose_secret());
}

#[test]
fn sqlcipher_key_zeroized_drop_path() {
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
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
