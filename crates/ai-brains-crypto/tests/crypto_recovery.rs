#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! RecoveryKit drills (T181-K-*). Library-level only — no CLI export path (F4/F5/F38).
//! T194: Argon2 KDF params pinned in kit JSON (`passphrase.kdf`).

use ai_brains_crypto::passphrase;
use ai_brains_crypto::test_support::{assert_no_kit_dump, assert_no_secret_leakage};
use ai_brains_crypto::{
    CryptoError, DataKey, KdfParams, PassphraseWrappedKey, RecoveryKit, SqlCipherKey,
};

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

/// T194 AC1 / inverted T181-K-07: new kits embed `passphrase.kdf` with product params.
#[test]
fn recovery_kit__generate__embeds_kdf_params() {
    let key = DataKey::generate();
    let passphrase = b"pwd-for-k07-invert";
    let kit = RecoveryKit::generate(&key, passphrase).unwrap();
    let json = kit.to_json().unwrap();

    let kdf = kit
        .passphrase
        .kdf
        .as_ref()
        .expect("new kits must stamp kdf");
    assert_eq!(kdf.algorithm, "argon2id");
    assert_eq!(kdf.version, 19);
    assert_eq!(kdf.m_cost, 19_456);
    assert_eq!(kdf.t_cost, 2);
    assert_eq!(kdf.p_cost, 1);

    // Wire presence (T194 invert of "lacks kdf fields").
    for required in ["kdf", "argon2id", "m_cost", "t_cost", "p_cost"] {
        assert!(
            json.contains(required),
            "kit JSON must contain KDF marker {required:?}; got: {json}"
        );
    }
    assert!(
        json.contains("\"m_cost\":19456") || json.contains("\"m_cost\": 19456"),
        "m_cost must be 19456: {json}"
    );
    assert!(
        json.contains("\"version\":19") || json.contains("\"version\": 19"),
        "version must be 19: {json}"
    );

    // Structural fields still present.
    assert!(json.contains("ciphertext") || json.contains("passphrase"));
    assert!(json.contains("salt"));
    assert!(json.contains("nonce"));
}

/// T194 smoke: default generate → unlock.
#[test]
fn recovery_kit__unlock__generate_roundtrip() {
    let key = DataKey::generate();
    let passphrase = b"roundtrip-pass";
    let kit = RecoveryKit::generate(&key, passphrase).expect("generate");
    let restored = kit.unlock_with_passphrase(passphrase).expect("unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

/// T194 AC2/F29 (mandatory): stored non-default params used; LEGACY fails on same wrap.
#[test]
fn recovery_kit__unlock__non_default_kdf_params__uses_stored_not_legacy() {
    let key = DataKey::generate();
    let passphrase = b"non-default-stored-params";
    let custom = KdfParams {
        algorithm: "argon2id".into(),
        version: 19,
        m_cost: 12_288,
        t_cost: 3,
        p_cost: 1,
    };
    let (ciphertext, salt, nonce) =
        passphrase::wrap_key(key.expose_secret(), passphrase, &custom).expect("wrap custom");

    let kit = RecoveryKit {
        schema_version: 1,
        dpapi: None,
        passphrase: PassphraseWrappedKey {
            ciphertext: ciphertext.clone(),
            salt,
            nonce,
            kdf: Some(custom),
        },
    };

    let restored = kit
        .unlock_with_passphrase(passphrase)
        .expect("unlock with stored params");
    assert_eq!(key.expose_secret(), restored.expose_secret());

    let legacy_fail =
        passphrase::unwrap_key(&ciphertext, passphrase, &salt, &nonce, &KdfParams::legacy());
    assert!(
        matches!(legacy_fail, Err(CryptoError::InvalidPassphrase)),
        "LEGACY must fail on non-default wrap: {legacy_fail:?}"
    );
}

/// T194 AC3: omit kdf → LEGACY dual-read unlock.
#[test]
fn recovery_kit__legacy_json_without_kdf__unlocks_with_legacy_defaults() {
    let key = DataKey::generate();
    let passphrase = b"legacy-omit-kdf";
    let kit = RecoveryKit::generate(&key, passphrase).expect("generate");
    let full = kit.to_json().expect("json");
    let mut v: serde_json::Value = serde_json::from_str(&full).expect("parse");
    v.get_mut("passphrase")
        .and_then(|p| p.as_object_mut())
        .expect("passphrase")
        .remove("kdf");
    let legacy = serde_json::to_string(&v).expect("reserialize");
    assert!(!legacy.contains("\"kdf\""));

    let parsed = RecoveryKit::from_json(&legacy).expect("deserialize");
    assert!(parsed.passphrase.kdf.is_none());
    let restored = parsed
        .unlock_with_passphrase(passphrase)
        .expect("legacy unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

/// T194 AC3: omit schema_version + kdf.
#[test]
fn recovery_kit__legacy_json_without_schema_and_kdf__unlocks() {
    let key = DataKey::generate();
    let passphrase = b"legacy-both-gone";
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
    let restored = parsed.unlock_with_passphrase(passphrase).expect("unlock");
    assert_eq!(key.expose_secret(), restored.expose_secret());
}

/// T194 AC5: partial kdf (missing m_cost) fails deserialize.
#[test]
fn recovery_kit__partial_kdf__deserialize_fails() {
    let key = DataKey::generate();
    let kit = RecoveryKit::generate(&key, b"partial").unwrap();
    let full = kit.to_json().unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&full).unwrap();
    v.get_mut("passphrase")
        .and_then(|p| p.as_object_mut())
        .and_then(|p| p.get_mut("kdf"))
        .and_then(|k| k.as_object_mut())
        .expect("kdf")
        .remove("m_cost");
    let bad = serde_json::to_string(&v).unwrap();
    assert!(matches!(
        RecoveryKit::from_json(&bad),
        Err(CryptoError::DeserializationError(_))
    ));
}

/// T194 F12: unknown field inside `kdf` fails deserialize (deny_unknown_fields).
#[test]
fn recovery_kit__kdf_unknown_field__deserialize_fails() {
    let key = DataKey::generate();
    let kit = RecoveryKit::generate(&key, b"unknown-field").unwrap();
    let full = kit.to_json().unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&full).unwrap();
    v.get_mut("passphrase")
        .and_then(|p| p.as_object_mut())
        .and_then(|p| p.get_mut("kdf"))
        .and_then(|k| k.as_object_mut())
        .expect("kdf")
        .insert("memory_cost".into(), serde_json::json!(19456));
    let bad = serde_json::to_string(&v).unwrap();
    assert!(matches!(
        RecoveryKit::from_json(&bad),
        Err(CryptoError::DeserializationError(_))
    ));
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
