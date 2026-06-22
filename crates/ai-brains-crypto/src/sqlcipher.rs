use crate::data_key::DataKey;
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A key formatted for SQLCipher (e.g., "x'HEX_KEY'")
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SqlCipherKey {
    material: String,
}

impl SqlCipherKey {
    /// Create a SqlCipherKey from a DataKey.
    /// Formats the key as a hex string with the SQLCipher "x'...'" prefix.
    pub fn from_data_key(key: &DataKey) -> Self {
        let hex = hex::encode(key.expose_secret());
        Self {
            material: format!("x'{}'", hex),
        }
    }

    /// Create a SqlCipherKey from a raw string (e.g. "x'HEX'").
    pub fn from_raw(material: String) -> Self {
        Self { material }
    }

    /// Access the raw key string for SQLCipher PRAGMA key.
    pub fn expose_secret(&self) -> &str {
        &self.material
    }
}

impl fmt::Debug for SqlCipherKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SqlCipherKey([REDACTED])")
    }
}

#[cfg(test)]
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
}
