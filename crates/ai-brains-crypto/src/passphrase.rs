use crate::errors::{CryptoError, Result};
use crate::key_wrap::KdfParams;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::SysRng;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;

/// Argon2 output length in bytes — always 32 (AES-256 DataKey). Not on the wire (F30).
const KDF_OUTPUT_LEN: usize = 32;

/// Build an explicit Argon2id hasher from validated [`KdfParams`].
///
/// Params come from the kit (or fixed legacy constants) — never crate Default.
fn argon2_from_params(params: &KdfParams) -> Result<Argon2<'static>> {
    params.validate_for_unlock()?;

    // output_len always 32 (DataKey / AES-256); omitted from KdfParams wire schema (F30).
    let argon_params = Params::new(
        params.m_cost,
        params.t_cost,
        params.p_cost,
        Some(KDF_OUTPUT_LEN),
    )
    .map_err(|e| {
        CryptoError::InvalidKdfParams(format!("argon2 Params::new rejected costs: {e}"))
    })?;

    // Algorithm and version already validated as argon2id / 19 (0x13).
    Ok(Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        argon_params,
    ))
}

/// Derive a 32-byte key from passphrase + salt using explicit KDF params (F15).
pub fn derive_key(
    passphrase: &[u8],
    salt: &[u8],
    output: &mut [u8],
    params: &KdfParams,
) -> Result<()> {
    let argon2 = argon2_from_params(params)?;
    argon2
        .hash_password_into(passphrase, salt, output)
        .map_err(|e| CryptoError::EncryptionError(format!("KDF failed: {e}")))
}

/// Wrap key material under a passphrase-derived key using the given KDF params.
pub fn wrap_key(
    key_material: &[u8],
    passphrase: &[u8],
    params: &KdfParams,
) -> Result<(Vec<u8>, [u8; SALT_LEN], [u8; NONCE_LEN])> {
    use rand::TryRng;
    let mut salt = [0u8; SALT_LEN];
    SysRng
        .try_fill_bytes(&mut salt)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {e}")))?;

    let mut derived_key = [0u8; KDF_OUTPUT_LEN];
    derive_key(passphrase, &salt, &mut derived_key, params)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::EncryptionError(format!("Entropy failed: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    let ciphertext = cipher
        .encrypt(nonce, key_material)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    Ok((ciphertext, salt, nonce_bytes))
}

/// Unwrap passphrase-wrapped material using the given KDF params.
pub fn unwrap_key(
    wrapped_material: &[u8],
    passphrase: &[u8],
    salt: &[u8; SALT_LEN],
    nonce_bytes: &[u8; NONCE_LEN],
    params: &KdfParams,
) -> Result<Vec<u8>> {
    let mut derived_key = [0u8; KDF_OUTPUT_LEN];
    derive_key(passphrase, salt, &mut derived_key, params)?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&derived_key)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;

    let plaintext = cipher
        .decrypt(nonce, wrapped_material)
        .map_err(|_| CryptoError::InvalidPassphrase)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]
    use super::*;
    use crate::key_wrap::{ALGORITHM_ARGON2ID, KdfParams, MAX_M_COST};

    #[test]
    fn passphrase_wrap_roundtrip() {
        let key = b"secret key material";
        let passphrase = b"correct horse battery staple";
        let params = KdfParams::product_generation();

        let (wrapped, salt, nonce) = wrap_key(key, passphrase, &params).expect("Wrap failed");
        let unwrapped =
            unwrap_key(&wrapped, passphrase, &salt, &nonce, &params).expect("Unwrap failed");

        assert_eq!(key.to_vec(), unwrapped);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let key = b"secret key material";
        let passphrase = b"correct horse battery staple";
        let wrong_passphrase = b"wrong password";
        let params = KdfParams::product_generation();

        let (wrapped, salt, nonce) = wrap_key(key, passphrase, &params).expect("Wrap failed");
        let result = unwrap_key(&wrapped, wrong_passphrase, &salt, &nonce, &params);

        assert!(matches!(result, Err(CryptoError::InvalidPassphrase)));
    }

    #[test]
    fn passphrase__derive__rejects_unknown_algorithm() {
        let mut bad = KdfParams::legacy();
        bad.algorithm = "argon2i".into();
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(_)));
    }

    #[test]
    fn passphrase__derive__rejects_m_cost_over_cap() {
        let mut bad = KdfParams::legacy();
        bad.m_cost = MAX_M_COST + 1;
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("m_cost")));
    }

    #[test]
    fn passphrase__derive__rejects_t_cost_over_cap() {
        use crate::key_wrap::MAX_T_COST;
        let mut bad = KdfParams::legacy();
        bad.t_cost = MAX_T_COST + 1;
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("t_cost")));
    }

    #[test]
    fn passphrase__derive__rejects_p_cost_over_cap() {
        use crate::key_wrap::MAX_P_COST;
        let mut bad = KdfParams::legacy();
        // Keep m_cost high enough for 8*p when p is at max+1 so we hit the p_cost cap first.
        bad.p_cost = MAX_P_COST + 1;
        bad.m_cost = bad.p_cost.saturating_mul(8).max(19_456);
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("p_cost")));
    }

    #[test]
    fn passphrase__derive__rejects_zero_costs() {
        let mut bad = KdfParams::legacy();
        bad.t_cost = 0;
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("non-zero")));
    }

    #[test]
    fn passphrase__derive__rejects_m_cost_below_8p() {
        let mut bad = KdfParams::legacy();
        bad.p_cost = 4;
        bad.m_cost = 16; // need >= 32
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("8*p_cost")));
    }

    #[test]
    fn passphrase__derive__rejects_wrong_version() {
        let mut bad = KdfParams::legacy();
        bad.version = 16;
        let mut out = [0u8; 32];
        let err = derive_key(b"pw", b"saltsaltsaltsalt", &mut out, &bad).unwrap_err();
        assert!(matches!(err, CryptoError::InvalidKdfParams(msg) if msg.contains("version")));
    }

    #[test]
    fn passphrase__unwrap__tampered_m_cost__fails_closed() {
        let key = b"secret key material 32bytes!!";
        let passphrase = b"correct horse battery staple";
        let params = KdfParams {
            algorithm: ALGORITHM_ARGON2ID.into(),
            version: 19,
            m_cost: 12_288,
            t_cost: 3,
            p_cost: 1,
        };
        let (wrapped, salt, nonce) = wrap_key(key, passphrase, &params).expect("wrap");

        let mut tampered = params.clone();
        tampered.m_cost = 19_456;
        let result = unwrap_key(&wrapped, passphrase, &salt, &nonce, &tampered);
        assert!(
            matches!(result, Err(CryptoError::InvalidPassphrase)),
            "wrong m_cost must fail closed: {result:?}"
        );
    }

    #[test]
    fn passphrase__no_argon2_default_in_production_path() {
        // Structural: production path must not call crate Default for Argon2.
        // Split needle so this test body does not match itself.
        let needle = format!("{}::{}", "Argon2", "default");
        let src = include_str!("passphrase.rs");
        // Only scan production code above the test module.
        let production = src.split("#[cfg(test)]").next().unwrap_or(src);
        assert!(
            !production.contains(&needle),
            "passphrase.rs production path must not use {needle} (T194 F8)"
        );
    }
}
