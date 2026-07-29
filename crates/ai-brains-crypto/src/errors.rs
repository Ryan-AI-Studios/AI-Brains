use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key generation failed")]
    KeyGenerationFailed,

    #[error("Invalid passphrase")]
    InvalidPassphrase,

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("DPAPI error: {0}")]
    DpapiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid key length")]
    InvalidKeyLength,

    /// AEAD authentication failed (tag / AAD / key mismatch). Generic message —
    /// does not distinguish tag vs AAD vs key to avoid oracle leaks.
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Nonce bytes were not exactly 12 (AES-GCM recommended IV length).
    #[error("Invalid nonce length")]
    InvalidNonceLength,

    #[error("Recovery kit is missing required components: {0}")]
    RecoveryKitMissing(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
