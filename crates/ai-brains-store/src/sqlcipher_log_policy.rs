//! Process-wide SQLCipher / SQLite errlog filter (T197 F1/F26/F27/F29).
//!
//! Community SQLCipher 4.10 emits multi-line native CORE logs on wrong-key open
//! (`hmac check failed`, decrypt errors) via its own logger to stderr — not only
//! through SQLite `SQLITE_CONFIG_LOG`. Primary controls:
//! 1. `rusqlite::trace::config_log` drops known noise that *does* hit errlog.
//! 2. `PRAGMA cipher_log_level = NONE` on a throwaway connection after SQLCipher
//!    init (global static; available on community 4.10.0 per vendored source).

use std::ffi::c_int;
use std::sync::OnceLock;

static INSTALL: OnceLock<()> = OnceLock::new();

/// Install the process-wide SQLite / SQLCipher log silence policy (idempotent).
///
/// Call **before** any vault `Connection::open` / `VaultConnection::open` and
/// before multi-threaded SQLite use (CLI main, daemon main, windows_service, AC1 tests).
pub fn install() {
    INSTALL.get_or_init(|| {
        // SAFETY:
        // - Invoked once via OnceLock before multi-threaded SQLite use in production entry points.
        // - Callback is a plain `fn` pointer (no captures).
        // - Callback does not re-enter SQLite and performs no unbounded allocation.
        if let Err(e) = unsafe { rusqlite::trace::config_log(Some(filter_cb)) } {
            tracing::warn!(
                error = %e,
                "failed to install SQLite errlog filter; wrong-key opens may spam stderr"
            );
        }

        // Silence SQLCipher CORE stderr logger (global). Must run after SQLCipher
        // extra_init so NONE is not reset to the default WARN threshold.
        // Fire-and-forget: community builds support this pragma (4.10.0); failures
        // leave config_log as the remaining filter.
        silence_sqlcipher_core_log();
    });
}

/// True when the log policy has been installed (tests / diagnostics).
#[cfg(test)]
pub fn is_installed() -> bool {
    INSTALL.get().is_some()
}

fn silence_sqlcipher_core_log() {
    // Opening any connection runs sqlcipher_extra_init (sets log defaults once).
    // Then set NONE so ERROR/WARN CORE messages (hmac check failed) are skipped.
    let conn = match rusqlite::Connection::open_in_memory() {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(error = %e, "sqlcipher_log_policy: in-memory open failed");
            return;
        }
    };
    // ANY unknown value path sets NONE first in SQLCipher; "NONE" leaves level at 0.
    if let Err(e) = conn.execute_batch("PRAGMA cipher_log_level = NONE;") {
        tracing::debug!(
            error = %e,
            "sqlcipher_log_policy: PRAGMA cipher_log_level=NONE failed (optional)"
        );
    }
    drop(conn);
}

fn filter_cb(_code: c_int, msg: &str) {
    if is_known_noise(msg) {
        return;
    }
    tracing::debug!(code = _code, msg, "sqlite errlog");
}

/// Drop only known wrong-key / encrypted-file flood lines (F22 allow-by-default).
fn is_known_noise(msg: &str) -> bool {
    // Case-sensitive contains is fine: SQLCipher / SQLite emit fixed English strings.
    msg.contains("hmac check failed")
        || msg.contains("file is encrypted or is not a database")
        || msg.contains("file is not a database")
        || msg.contains("error decrypting page")
        || msg.contains("sqlcipher_codec_ctx_set_error")
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::connection::{ALLOW_ZERO_KEY_ENV, VaultConnection};
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::SqlCipherKey;
    use tempfile::tempdir;

    #[test]
    fn install__idempotent__second_call_ok() {
        install();
        install();
        assert!(is_installed());
    }

    #[test]
    fn is_known_noise__hmac_and_encrypted_file() {
        // Production SQLCipher community builds emit lowercase "hmac check failed".
        assert!(is_known_noise("hmac check failed for page 2"));
        assert!(is_known_noise("file is encrypted or is not a database"));
        assert!(is_known_noise("file is not a database"));
        assert!(is_known_noise("error decrypting page 1 data: 1"));
        assert!(!is_known_noise("disk I/O error"));
        assert!(!is_known_noise("database disk image is malformed"));
    }

    /// Wrong key open after install fails closed (VaultLocked).
    #[test]
    fn install__wrong_key_open__fails_closed_vault_locked() {
        install();
        let _allow = TempEnv::set(ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("v.db");
        let zero = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".into(),
        );
        {
            let c = VaultConnection::open(&path, &zero).expect("create");
            c.migrate().expect("migrate");
        }
        let wrong = SqlCipherKey::from_raw(
            "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'".into(),
        );
        match VaultConnection::open(&path, &wrong) {
            Ok(_) => panic!("wrong key must fail closed"),
            Err(err) => {
                assert!(
                    matches!(err, crate::StoreError::VaultLocked(_)),
                    "got {err}"
                );
            }
        }
    }
}
