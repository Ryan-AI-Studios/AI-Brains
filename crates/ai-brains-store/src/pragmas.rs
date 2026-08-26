use crate::errors::Result;
use ai_brains_crypto::SqlCipherKey;
use rusqlite::Connection;

/// Apply full vault open pragmas (key, cipher compat, WAL, sync, busy_timeout).
///
/// Key form uses single quotes around the SQLCipher `x'hex'` token to match
/// Zetetic examples (`PRAGMA key = 'x''…'''` is incorrect for our material which
/// already includes the `x'…'` delimiters). We pass the full material as a
/// string literal: `PRAGMA key = "x'…'"`. SQLCipher does not support bound
/// parameters for `PRAGMA key`, so string formatting is required.
///
/// Do **not** set `cipher_plaintext_header_size` — full header encryption is
/// required for AC2 (file must not start with `SQLite format 3`).
pub fn apply_pragmas(conn: &Connection, key: &SqlCipherKey) -> Result<()> {
    // Single-quoted form matching Zetetic docs when key is a passphrase;
    // our product keys are already `x'HEX'` so double-quoted string form is used.
    let pragma_key = format!("PRAGMA key = \"{}\";", key.expose_secret());
    conn.execute_batch(&pragma_key)?;

    // Cipher compatibility (SQLCipher 4 defaults)
    conn.execute_batch("PRAGMA cipher_compatibility = 4;")?;

    // SQLCipher supports WAL.
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

    // Busy timeout: let SQLite retry internally for up to 5s before SQLITE_BUSY
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

    Ok(())
}

/// Apply only the key, cipher compatibility, and busy timeout pragmas.
/// Use when opening a second connection to a vault already open (e.g. backup
/// source). Setting `journal_mode` / `synchronous` requires exclusive access
/// and will deadlock if another connection holds the file open.
pub fn apply_key_pragmas(conn: &Connection, key: &SqlCipherKey) -> Result<()> {
    let pragma_key = format!("PRAGMA key = \"{}\";", key.expose_secret());
    conn.execute_batch(&pragma_key)?;
    conn.execute_batch("PRAGMA cipher_compatibility = 4;")?;
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
    Ok(())
}

/// Return `PRAGMA cipher_version` when SQLCipher is linked.
/// Empty string indicates a plain `bundled` (non-SQLCipher) build drift;
/// query failures are preserved (not mapped to empty) so doctor can surface them.
pub fn cipher_version(conn: &Connection) -> Result<String> {
    let version: String = conn.query_row("PRAGMA cipher_version", [], |row| row.get(0))?;
    Ok(version)
}
