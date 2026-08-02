use crate::errors::{CryptoError, Result};
use crate::passphrase::{NONCE_LEN, SALT_LEN};
use serde::{Deserialize, Serialize};

/// Product/legacy Argon2id memory cost (KiB blocks). Matches argon2 0.5.x historical default.
pub const LEGACY_M_COST: u32 = 19_456;
/// Product/legacy Argon2id time cost.
pub const LEGACY_T_COST: u32 = 2;
/// Product/legacy Argon2id parallelism.
pub const LEGACY_P_COST: u32 = 1;
/// Wire version for Argon2 0x13 (PHC-style decimal 19).
pub const LEGACY_VERSION: u32 = 19;

/// Read-side DoS caps (F14): reject malicious kits before KDF runs.
pub const MAX_M_COST: u32 = 1_048_576; // 1 GiB
pub const MAX_T_COST: u32 = 32;
pub const MAX_P_COST: u32 = 16;

/// Wire algorithm id for Argon2id (case-sensitive).
pub const ALGORITHM_ARGON2ID: &str = "argon2id";

/// KDF parameters stored with passphrase-wrapped DataKey material (T194).
///
/// `output_len` is intentionally omitted from the wire schema: it is always **32**
/// (AES-256 DataKey length). Changing DataKey size is out of scope (F30).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KdfParams {
    /// Wire algorithm id; generation always uses `"argon2id"`.
    pub algorithm: String,
    /// Argon2 version as PHC-style decimal (19 = 0x13).
    pub version: u32,
    /// Memory cost in KiB blocks.
    pub m_cost: u32,
    /// Time (iterations) cost.
    pub t_cost: u32,
    /// Parallelism degree.
    pub p_cost: u32,
}

impl KdfParams {
    /// Historical pre-T194 / product-generation defaults (F7/F9).
    ///
    /// Fixed constants — **not** `argon2::Params::default()` / crate Default, so crate
    /// default drift cannot brick pre-T194 kits.
    pub fn legacy() -> Self {
        Self {
            algorithm: ALGORITHM_ARGON2ID.into(),
            version: LEGACY_VERSION,
            m_cost: LEGACY_M_COST,
            t_cost: LEGACY_T_COST,
            p_cost: LEGACY_P_COST,
        }
    }

    /// Product generation params (F7) — currently equal to [`Self::legacy()`];
    /// kept named so a future strength bump can diverge from legacy dual-read.
    pub fn product_generation() -> Self {
        Self::legacy()
    }

    /// Validate params before running Argon2 on unlock (or wrap).
    ///
    /// Enforces algorithm/version, non-zero costs, argon2 minimum memory
    /// (`m_cost >= 8 * p_cost`), and F14 DoS caps.
    pub fn validate_for_unlock(&self) -> Result<()> {
        if self.algorithm != ALGORITHM_ARGON2ID {
            return Err(CryptoError::InvalidKdfParams(format!(
                "unsupported KDF algorithm {:?}; only \"{}\" is supported",
                self.algorithm, ALGORITHM_ARGON2ID
            )));
        }
        if self.version != LEGACY_VERSION {
            return Err(CryptoError::InvalidKdfParams(format!(
                "unsupported Argon2 version {}; only {} (0x13) is supported",
                self.version, LEGACY_VERSION
            )));
        }
        if self.m_cost == 0 || self.t_cost == 0 || self.p_cost == 0 {
            return Err(CryptoError::InvalidKdfParams(
                "KDF costs m_cost, t_cost, and p_cost must be non-zero".into(),
            ));
        }
        if self.m_cost > MAX_M_COST {
            return Err(CryptoError::InvalidKdfParams(format!(
                "m_cost {} exceeds maximum allowed {}",
                self.m_cost, MAX_M_COST
            )));
        }
        if self.t_cost > MAX_T_COST {
            return Err(CryptoError::InvalidKdfParams(format!(
                "t_cost {} exceeds maximum allowed {}",
                self.t_cost, MAX_T_COST
            )));
        }
        if self.p_cost > MAX_P_COST {
            return Err(CryptoError::InvalidKdfParams(format!(
                "p_cost {} exceeds maximum allowed {}",
                self.p_cost, MAX_P_COST
            )));
        }
        // argon2 minimum: memory cost must be at least 8 * parallelism.
        let min_m = self.p_cost.saturating_mul(8);
        if self.m_cost < min_m {
            return Err(CryptoError::InvalidKdfParams(format!(
                "m_cost {} is below argon2 minimum 8*p_cost ({})",
                self.m_cost, min_m
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DpapiWrappedKey {
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassphraseWrappedKey {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
    /// Argon2id parameters used to wrap this material.
    ///
    /// Absent on pre-T194 kits → unlock uses [`KdfParams::legacy()`] (F9).
    /// New kits always serialize this field (F11: no `skip_serializing_if`).
    #[serde(default)]
    pub kdf: Option<KdfParams>,
}
