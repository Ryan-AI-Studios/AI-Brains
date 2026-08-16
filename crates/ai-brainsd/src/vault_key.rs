//! Daemon vault-key resolve. No silent zero (T197 residual).
//!
//! Prefer `AI_BRAINS_VAULT_KEY`, then `AI_BRAINS_KEY` (CLI product name).

use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::ALLOW_ZERO_KEY_ENV;

/// Operator-facing daemon key failures. Never include key material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonKeyError {
    Missing,
    Format(String),
    Zero,
}

impl std::fmt::Display for DaemonKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(
                f,
                "Vault key missing: set AI_BRAINS_VAULT_KEY or AI_BRAINS_KEY"
            ),
            Self::Format(detail) => write!(
                f,
                "Vault key invalid format: must be x'<64 hex chars>' ({detail})"
            ),
            Self::Zero => write!(
                f,
                "Vault key refused: zero key without {ALLOW_ZERO_KEY_ENV}"
            ),
        }
    }
}

impl std::error::Error for DaemonKeyError {}

fn non_empty_trimmed(raw: Option<String>) -> Option<String> {
    raw.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

fn allow_zero_key() -> bool {
    matches!(
        std::env::var(ALLOW_ZERO_KEY_ENV).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

/// Resolve daemon key: `AI_BRAINS_VAULT_KEY` then `AI_BRAINS_KEY`. Refuse missing/zero.
pub fn resolve_daemon_sqlcipher_key() -> Result<SqlCipherKey, DaemonKeyError> {
    let pick = non_empty_trimmed(std::env::var("AI_BRAINS_VAULT_KEY").ok())
        .or_else(|| non_empty_trimmed(std::env::var("AI_BRAINS_KEY").ok()));

    let raw = match pick {
        Some(s) => s,
        None => return Err(DaemonKeyError::Missing),
    };

    let key = SqlCipherKey::try_from_raw(raw).map_err(|e| DaemonKeyError::Format(e.to_string()))?;

    if key.is_zero() && !allow_zero_key() {
        return Err(DaemonKeyError::Zero);
    }

    Ok(key)
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    const VALID: &str = "x'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'";
    const ZERO: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

    #[test]
    fn resolve_daemon_sqlcipher_key__missing__both_absent() {
        let _g = TempEnv::remove("AI_BRAINS_VAULT_KEY");
        let _k = TempEnv::remove("AI_BRAINS_KEY");
        let _z = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let err = resolve_daemon_sqlcipher_key().expect_err("missing");
        assert_eq!(err, DaemonKeyError::Missing);
    }

    #[test]
    fn resolve_daemon_sqlcipher_key__vault_key__wins_over_key() {
        let _g = TempEnv::set("AI_BRAINS_VAULT_KEY", VALID);
        let _k = TempEnv::set("AI_BRAINS_KEY", ZERO);
        let _z = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let key = resolve_daemon_sqlcipher_key().expect("vault key");
        assert_eq!(key.expose_secret(), VALID);
    }

    #[test]
    fn resolve_daemon_sqlcipher_key__key_fallback__when_vault_key_absent() {
        let _g = TempEnv::remove("AI_BRAINS_VAULT_KEY");
        let _k = TempEnv::set("AI_BRAINS_KEY", VALID);
        let _z = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let key = resolve_daemon_sqlcipher_key().expect("key fallback");
        assert_eq!(key.expose_secret(), VALID);
    }

    #[test]
    fn resolve_daemon_sqlcipher_key__zero__refused_without_allow() {
        let _g = TempEnv::set("AI_BRAINS_VAULT_KEY", ZERO);
        let _k = TempEnv::remove("AI_BRAINS_KEY");
        let _z = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let err = resolve_daemon_sqlcipher_key().expect_err("zero");
        assert_eq!(err, DaemonKeyError::Zero);
    }

    #[test]
    fn dotenvy_quoted_product_key__roundtrip_preserves_x_form() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("daemon.env");
        let key = VALID;
        std::fs::write(&path, format!("AI_BRAINS_VAULT_KEY=\"{key}\"\n")).expect("write");
        let _clear = TempEnv::remove("AI_BRAINS_VAULT_KEY");
        dotenvy::from_path_override(&path).expect("dotenv");
        let loaded = std::env::var("AI_BRAINS_VAULT_KEY").expect("loaded");
        assert_eq!(loaded, key);
        assert_eq!(loaded.len(), 67);
    }

    #[test]
    fn dotenvy_unquoted_product_key__does_not_preserve_x_form() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("daemon.env");
        let key = VALID;
        std::fs::write(&path, format!("AI_BRAINS_VAULT_KEY={key}\n")).expect("write");
        let _clear = TempEnv::remove("AI_BRAINS_VAULT_KEY");
        dotenvy::from_path_override(&path).expect("dotenv");
        let loaded = std::env::var("AI_BRAINS_VAULT_KEY").expect("loaded");
        assert_ne!(loaded, key, "unquoted x'…' is eaten by dotenvy");
        assert_ne!(loaded.len(), 67);
    }

    #[test]
    fn resolve_daemon_sqlcipher_key__blank_vault_key__falls_back_to_key() {
        let _g = TempEnv::set("AI_BRAINS_VAULT_KEY", "   ");
        let _k = TempEnv::set("AI_BRAINS_KEY", VALID);
        let _z = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let key = resolve_daemon_sqlcipher_key().expect("blank vault");
        assert_eq!(key.expose_secret(), VALID);
    }
}
