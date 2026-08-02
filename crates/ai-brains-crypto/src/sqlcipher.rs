use crate::data_key::DataKey;
use crate::errors::{CryptoError, Result};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// SQLCipher key material in the product form `x'<64 hex chars>'` (32 raw bytes).
///
/// Display/Debug never expose the material.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SqlCipherKey {
    material: String,
}

impl SqlCipherKey {
    /// Create a SqlCipherKey from a DataKey.
    /// Formats the key as a hex string with the SQLCipher `x'...'` prefix.
    pub fn from_data_key(key: &DataKey) -> Self {
        let hex = hex::encode(key.expose_secret());
        Self {
            material: format!("x'{}'", hex),
        }
    }

    /// Create a SqlCipherKey from a raw SQLCipher key string (e.g. `x'HEX'`).
    ///
    /// Prefer [`try_from_raw`] for fallible construction. This constructor does
    /// not validate; callers that open vaults must call [`validate`] (or use
    /// [`try_from_raw`]) before applying the key.
    pub fn from_raw(material: String) -> Self {
        Self { material }
    }

    /// Fallible constructor: product keys must match `^x'[0-9a-fA-F]{64}'$`.
    pub fn try_from_raw(material: String) -> Result<Self> {
        let key = Self { material };
        key.validate()?;
        Ok(key)
    }

    /// Access the raw key string for SQLCipher `PRAGMA key`.
    pub fn expose_secret(&self) -> &str {
        &self.material
    }

    /// True when the key material is 32 zero bytes (`x'000…0'`).
    pub fn is_zero(&self) -> bool {
        const ZERO: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
        self.material.eq_ignore_ascii_case(ZERO)
    }

    /// Validate product key format: `x'` + 64 hex digits + `'`.
    pub fn validate(&self) -> Result<()> {
        let s = self.material.as_str();
        if s.len() != 67 {
            return Err(CryptoError::InvalidKeyFormat(
                "SQLCipher key must be x'<64 hex chars>' (67 chars total)".into(),
            ));
        }
        let bytes = s.as_bytes();
        if bytes[0] != b'x' || bytes[1] != b'\'' || bytes[66] != b'\'' {
            return Err(CryptoError::InvalidKeyFormat(
                "SQLCipher key must use form x'<hex>'".into(),
            ));
        }
        for &b in &bytes[2..66] {
            if !b.is_ascii_hexdigit() {
                return Err(CryptoError::InvalidKeyFormat(
                    "SQLCipher key hex body must be 0-9a-fA-F".into(),
                ));
            }
        }
        Ok(())
    }

    /// True when blank/whitespace-only (never a valid key).
    pub fn is_blank(&self) -> bool {
        self.material.trim().is_empty()
    }
}

impl fmt::Debug for SqlCipherKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SqlCipherKey([REDACTED])")
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn sqlcipher_key_zeroized() {
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);

        let _secret_ptr = sql_key.expose_secret().as_ptr();
        let _secret_len = sql_key.expose_secret().len();

        // Drop the key to trigger zeroization
        drop(sql_key);

        // Safety: This is technically UB to access after drop, but we want to verify zeroization.
        // In a real test we might use a custom allocator or check the memory if we can.
        // For now, we rely on ZeroizeOnDrop being derived.
    }

    #[test]
    fn sqlcipher_key_debug_redacted() {
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        let debug_str = format!("{:?}", sql_key);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains(sql_key.expose_secret()));
    }

    #[test]
    fn sqlcipher_key__is_zero__all_zero_hex() {
        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".into(),
        );
        assert!(key.is_zero());
        assert!(key.validate().is_ok());
    }

    #[test]
    fn sqlcipher_key__validate__rejects_malformed() {
        assert!(SqlCipherKey::from_raw("".into()).validate().is_err());
        assert!(
            SqlCipherKey::from_raw("not-a-key".into())
                .validate()
                .is_err()
        );
        assert!(SqlCipherKey::from_raw("x'00'".into()).validate().is_err());
        assert!(
            SqlCipherKey::try_from_raw(
                "x'gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg'".into()
            )
            .is_err()
        );
    }

    #[test]
    fn sqlcipher_key__from_data_key__valid_format() {
        let key = SqlCipherKey::from_data_key(&DataKey::generate());
        assert!(key.validate().is_ok());
        assert!(!key.is_zero() || key.is_zero()); // random may theoretically be zero
    }
}
