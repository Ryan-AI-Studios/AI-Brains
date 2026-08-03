//! Shared operator SQLCipher key resolution (T197 F2–F6, F8, F30).
//!
//! Lives in the CLI crate (arg + env). Store keeps `enforce_key_policy` only.

use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::ALLOW_ZERO_KEY_ENV;
use std::fmt;

/// Operator-facing key resolve failures (F8 prefixes pinned for tests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyResolveError {
    /// Neither `--key` nor `AI_BRAINS_KEY` provided (after trim).
    Missing,
    /// Product form must be `x'<64 hex chars>'`.
    Format(String),
    /// Explicit all-zero key without `AI_BRAINS_ALLOW_ZERO_KEY`.
    Zero,
}

impl fmt::Display for KeyResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(
                f,
                "Vault key missing: set --key or AI_BRAINS_KEY (see INSTALL)"
            ),
            Self::Format(detail) => write!(
                f,
                "Vault key invalid format: must be x'<64 hex chars>' ({detail})"
            ),
            Self::Zero => write!(
                f,
                "Vault key refused: zero key without AI_BRAINS_ALLOW_ZERO_KEY"
            ),
        }
    }
}

impl std::error::Error for KeyResolveError {}

/// Resolve operator key: CLI `--key` (non-empty trim) → `AI_BRAINS_KEY` env → Missing.
///
/// Validates via [`SqlCipherKey::try_from_raw`] (F5) and refuses zero without allow (F6).
pub fn resolve_operator_sqlcipher_key(
    cli_key: Option<String>,
) -> Result<SqlCipherKey, KeyResolveError> {
    let pick = non_empty_trimmed(cli_key).or_else(|| {
        std::env::var("AI_BRAINS_KEY")
            .ok()
            .and_then(|v| non_empty_trimmed(Some(v)))
    });

    let raw = match pick {
        Some(s) => s,
        None => return Err(KeyResolveError::Missing),
    };

    let key =
        SqlCipherKey::try_from_raw(raw).map_err(|e| KeyResolveError::Format(e.to_string()))?;

    if key.is_zero() && !allow_zero_key() {
        return Err(KeyResolveError::Zero);
    }

    Ok(key)
}

/// JSON / ApiError codes for [`KeyResolveError`] (F8 / AC12).
pub fn key_resolve_json_code(e: &KeyResolveError) -> &'static str {
    match e {
        KeyResolveError::Missing => "VAULT_KEY_MISSING",
        KeyResolveError::Format(_) => "VAULT_KEY_FORMAT",
        KeyResolveError::Zero => "VAULT_KEY_ZERO",
    }
}

/// JSON code for vault locked open failures.
pub const VAULT_LOCKED_JSON_CODE: &str = "VAULT_LOCKED";

/// Human prefix for wrong-key / cannot-decrypt failures (F7/F8).
pub const VAULT_LOCKED_PREFIX: &str = "Vault locked:";

/// Format a vault-locked operator message (never include key material).
pub fn vault_locked_message(detail: &str) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        format!("{VAULT_LOCKED_PREFIX} wrong key or cannot decrypt")
    } else if detail.starts_with(VAULT_LOCKED_PREFIX) {
        detail.to_string()
    } else {
        format!("{VAULT_LOCKED_PREFIX} {detail}")
    }
}

fn non_empty_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

fn allow_zero_key() -> bool {
    match std::env::var(ALLOW_ZERO_KEY_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    const ZERO: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
    const VALID: &str = "x'0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'";

    #[test]
    fn resolve_operator_sqlcipher_key__missing__both_absent() {
        let _clear = TempEnv::remove("AI_BRAINS_KEY");
        let err = resolve_operator_sqlcipher_key(None).expect_err("missing");
        assert!(matches!(err, KeyResolveError::Missing));
        assert!(err.to_string().starts_with("Vault key missing:"));
        assert_eq!(key_resolve_json_code(&err), "VAULT_KEY_MISSING");
    }

    #[test]
    fn resolve_operator_sqlcipher_key__missing__blank_cli_and_env() {
        let _env = TempEnv::set("AI_BRAINS_KEY", "   ");
        let err = resolve_operator_sqlcipher_key(Some("  ".into())).expect_err("blank");
        assert!(matches!(err, KeyResolveError::Missing));
    }

    #[test]
    fn resolve_operator_sqlcipher_key__format__bare_hex() {
        let _clear = TempEnv::remove("AI_BRAINS_KEY");
        let bare = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
        let err = resolve_operator_sqlcipher_key(Some(bare)).expect_err("format");
        assert!(matches!(err, KeyResolveError::Format(_)));
        assert!(err.to_string().starts_with("Vault key invalid format:"));
        assert_eq!(key_resolve_json_code(&err), "VAULT_KEY_FORMAT");
    }

    #[test]
    fn resolve_operator_sqlcipher_key__zero__refused_without_allow() {
        let _clear_key = TempEnv::remove("AI_BRAINS_KEY");
        let _clear_allow = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let err = resolve_operator_sqlcipher_key(Some(ZERO.into())).expect_err("zero");
        assert!(matches!(err, KeyResolveError::Zero));
        assert!(err.to_string().starts_with("Vault key refused:"));
        assert_eq!(key_resolve_json_code(&err), "VAULT_KEY_ZERO");
    }

    #[test]
    fn resolve_operator_sqlcipher_key__zero__allowed_with_env() {
        let _clear_key = TempEnv::remove("AI_BRAINS_KEY");
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let key = resolve_operator_sqlcipher_key(Some(ZERO.into())).expect("allow zero");
        assert!(key.is_zero());
    }

    #[test]
    fn resolve_operator_sqlcipher_key__valid__cli_wins_over_env() {
        let _env = TempEnv::set("AI_BRAINS_KEY", ZERO);
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let key = resolve_operator_sqlcipher_key(Some(VALID.into())).expect("cli");
        assert_eq!(key.expose_secret(), VALID);
    }

    #[test]
    fn resolve_operator_sqlcipher_key__valid__env_fallback() {
        let _env = TempEnv::set("AI_BRAINS_KEY", VALID);
        let _clear_allow = TempEnv::remove(ALLOW_ZERO_KEY_ENV);
        let key = resolve_operator_sqlcipher_key(None).expect("env");
        assert_eq!(key.expose_secret(), VALID);
    }

    #[test]
    fn key_resolve_json_code__matrix() {
        assert_eq!(
            key_resolve_json_code(&KeyResolveError::Missing),
            "VAULT_KEY_MISSING"
        );
        assert_eq!(
            key_resolve_json_code(&KeyResolveError::Format("x".into())),
            "VAULT_KEY_FORMAT"
        );
        assert_eq!(
            key_resolve_json_code(&KeyResolveError::Zero),
            "VAULT_KEY_ZERO"
        );
        assert_eq!(VAULT_LOCKED_JSON_CODE, "VAULT_LOCKED");
    }

    #[test]
    fn vault_locked_message__prefixes() {
        let m = vault_locked_message("Key verification failed: …");
        assert!(m.starts_with("Vault locked:"));
        assert!(!m.contains("0000"));
        let already = vault_locked_message("Vault locked: already");
        assert_eq!(already, "Vault locked: already");
    }
}
