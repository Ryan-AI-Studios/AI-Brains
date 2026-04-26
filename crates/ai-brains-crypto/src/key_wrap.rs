use crate::passphrase::{NONCE_LEN, SALT_LEN};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DpapiWrappedKey {
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassphraseWrappedKey {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}
