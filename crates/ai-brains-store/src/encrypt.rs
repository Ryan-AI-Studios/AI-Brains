//! Plaintext → SQLCipher conversion via `sqlcipher_export` (T187 F4 / F18).
//!
//! Online Backup must **not** be used for plain→encrypted conversion: it is a
//! page-level copy and does not re-encrypt plaintext pages into the SQLCipher codec.

use crate::errors::{Result, StoreError};
use crate::header::{is_plain_sqlite_header, legacy_plaintext_migrate_hint};
use crate::pragmas::apply_key_pragmas;
use ai_brains_crypto::SqlCipherKey;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

/// Options for [`encrypt_plaintext_vault`].
#[derive(Debug, Clone)]
pub struct EncryptOptions {
    /// Source plaintext vault path.
    pub source: PathBuf,
    /// Destination encrypted vault path (must not equal source unless `replace_source`).
    pub destination: PathBuf,
    /// When true, after successful export, atomically replace `source` with `destination`
    /// (source moved aside to `source.bak-plain` first). Requires explicit operator confirm
    /// at the CLI layer.
    pub replace_source: bool,
    /// Preview only: validate and print plan; no file write.
    pub dry_run: bool,
}

/// Convert a **plaintext** SQLite vault to a SQLCipher-encrypted vault using
/// `SELECT sqlcipher_export('encrypted')` after `ATTACH … KEY`.
///
/// Sequence (F4):
/// 1. Refuse if source is missing or not plain-header
/// 2. Checkpoint WAL on source (best-effort)
/// 3. Open source **unkeyed**
/// 4. ATTACH destination AS encrypted KEY …
/// 5. `sqlcipher_export('encrypted')`
/// 6. DETACH
/// 7. Optional operator-confirmed replace of original
pub fn encrypt_plaintext_vault(opts: &EncryptOptions, key: &SqlCipherKey) -> Result<PathBuf> {
    if key.is_blank() {
        return Err(StoreError::VaultLocked(
            "blank key refused for vault encrypt".into(),
        ));
    }
    key.validate()
        .map_err(|e| StoreError::VaultLocked(format!("invalid key format: {e}")))?;
    if key.is_zero() && !zero_key_escape() {
        return Err(StoreError::VaultLocked(
            "zero key refused for vault encrypt; set a non-zero key or AI_BRAINS_ALLOW_ZERO_KEY=1"
                .into(),
        ));
    }

    let source = opts.source.as_path();
    if !source.exists() {
        return Err(StoreError::ConnectionFailed(format!(
            "source vault does not exist: {}",
            source.display()
        )));
    }
    if !is_plain_sqlite_header(source) {
        return Err(StoreError::ConfigError(format!(
            "source is not a plaintext SQLite database (missing SQLite format 3 header): {}. \
             If already encrypted, use backup create/restore instead of vault encrypt.",
            source.display()
        )));
    }

    let dest = opts.destination.as_path();
    if opts.dry_run {
        return Ok(dest.to_path_buf());
    }

    if dest.exists() {
        return Err(StoreError::ConfigError(format!(
            "destination already exists (refusing overwrite): {}",
            dest.display()
        )));
    }

    // Ensure parent dir exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            StoreError::ConnectionFailed(format!("create dest parent {}: {e}", parent.display()))
        })?;
    }

    // Open source unkeyed (plaintext)
    let src = Connection::open(source).map_err(|e| {
        StoreError::ConnectionFailed(format!("open plaintext source {}: {e}", source.display()))
    })?;
    let _ = src.execute_batch("PRAGMA busy_timeout = 5000;");
    // Best-effort checkpoint before export
    let _ = src.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));

    // ATTACH encrypted destination with key. Path quoting: escape single quotes.
    let dest_sql = sql_quote_path(dest);
    let key_material = key.expose_secret();
    // Zetetic: ATTACH DATABASE 'file' AS encrypted KEY "x'…'";
    let attach = format!("ATTACH DATABASE {dest_sql} AS encrypted KEY \"{key_material}\";");
    src.execute_batch(&attach).map_err(|e| {
        StoreError::ConnectionFailed(format!("ATTACH encrypted for sqlcipher_export failed: {e}"))
    })?;

    src.execute_batch("SELECT sqlcipher_export('encrypted');")
        .map_err(|e| StoreError::ConnectionFailed(format!("sqlcipher_export failed: {e}")))?;

    src.execute_batch("DETACH DATABASE encrypted;")
        .map_err(|e| StoreError::ConnectionFailed(format!("DETACH encrypted failed: {e}")))?;

    // Verify destination opens with key and is not plain-header
    if is_plain_sqlite_header(dest) {
        let _ = fs::remove_file(dest);
        return Err(StoreError::ConnectionFailed(
            "sqlcipher_export produced a plain SQLite header (SQLCipher not linked?)".into(),
        ));
    }
    {
        let verify = Connection::open(dest).map_err(|e| {
            StoreError::ConnectionFailed(format!("open exported vault failed: {e}"))
        })?;
        apply_key_pragmas(&verify, key)?;
        verify
            .query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| {
                StoreError::VaultLocked(format!("Key verification failed on exported vault: {e}"))
            })?;
    }

    if opts.replace_source {
        let bak = source.with_extension("db.bak-plain");
        // Prefer unique bak path
        let bak = if bak.exists() {
            source.with_extension(format!(
                "db.bak-plain-{}",
                chrono::Utc::now().format("%Y%m%d%H%M%S")
            ))
        } else {
            bak
        };
        fs::rename(source, &bak).map_err(|e| {
            StoreError::ConnectionFailed(format!(
                "could not move original plain vault aside to {}: {e}",
                bak.display()
            ))
        })?;
        fs::rename(dest, source).map_err(|e| {
            // Try to restore plain vault
            let _ = fs::rename(&bak, source);
            StoreError::ConnectionFailed(format!(
                "could not replace source with encrypted vault: {e}"
            ))
        })?;
        return Ok(source.to_path_buf());
    }

    Ok(dest.to_path_buf())
}

fn zero_key_escape() -> bool {
    match std::env::var(crate::connection::ALLOW_ZERO_KEY_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn sql_quote_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    // SQL single-quoted string; escape ' as ''
    format!("'{}'", s.replace('\'', "''"))
}

/// Helper for callers that need the migrate error when a path is plain.
pub fn refuse_if_not_plain_for_encrypt(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(StoreError::ConnectionFailed(format!(
            "path does not exist: {}",
            path.display()
        )));
    }
    if !is_plain_sqlite_header(path) {
        return Err(StoreError::ConfigError(legacy_plaintext_migrate_hint()));
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::connection::VaultConnection;
    use ai_brains_crypto::DataKey;
    use tempfile::tempdir;

    #[test]
    fn encrypt_plaintext_vault__content_smoke() {
        let dir = tempdir().expect("tempdir");
        let plain = dir.path().join("plain.db");
        let enc = dir.path().join("enc.db");
        {
            let conn = Connection::open(&plain).expect("plain");
            conn.execute_batch(
                "CREATE TABLE t187(x TEXT); INSERT INTO t187 VALUES ('hello-encrypt');",
            )
            .expect("seed");
        }
        assert!(is_plain_sqlite_header(&plain));

        let key = SqlCipherKey::from_data_key(&DataKey::generate());
        let out = encrypt_plaintext_vault(
            &EncryptOptions {
                source: plain.clone(),
                destination: enc.clone(),
                replace_source: false,
                dry_run: false,
            },
            &key,
        )
        .expect("encrypt");
        assert_eq!(out, enc);
        assert!(!is_plain_sqlite_header(&enc));

        let vault = VaultConnection::open(&enc, &key).expect("open enc");
        let g = vault.lock().expect("lock");
        let v: String = g
            .query_row("SELECT x FROM t187", [], |r| r.get(0))
            .expect("read");
        assert_eq!(v, "hello-encrypt");
    }
}
