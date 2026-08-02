pub mod content_envelope;
pub mod content_key_store;
pub mod data_key;
pub mod dpapi;
pub mod errors;
pub mod key_wrap;
pub mod passphrase;
pub mod recovery_kit;
pub mod sqlcipher;

pub use content_envelope::{
    ALGORITHM_LABEL, ENVELOPE_SCHEMA_VERSION, GeneratedEnvelope, SealAad, SealedContent,
    build_content_seal_aad, generate_wrap_and_seal, open, seal, unwrap_and_open,
};
pub use content_key_store::{
    CONTENT_DEK_LEN, ContentDek, NONCE_LEN, WRAP_SCHEMA_VERSION, WrappedContentDek,
    build_dek_wrap_aad, parse_nonce, rotate_content_dek_wrap, unwrap_content_dek, wrap_content_dek,
};
pub use data_key::DataKey;
pub use errors::{CryptoError, Result};
pub use recovery_kit::RecoveryKit;
pub use sqlcipher::SqlCipherKey;

pub mod test_support;
