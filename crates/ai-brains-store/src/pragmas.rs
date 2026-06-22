use crate::errors::Result;
use ai_brains_crypto::SqlCipherKey;
use rusqlite::Connection;

pub fn apply_pragmas(conn: &Connection, key: &SqlCipherKey) -> Result<()> {
    // 1. Inject the key. SQLCipher expects the key as a hex string or raw bytes.
    // Standard format for PRAGMA key with hex is: PRAGMA key = "x'HEX_KEY'";
    let pragma_key = format!("PRAGMA key = \"{}\"", key.expose_secret());
    conn.execute_batch(&pragma_key)?;

    // 2. Cipher compatibility
    conn.execute_batch("PRAGMA cipher_compatibility = 4;")?;

    // 3. Journal mode (WAL is standard for high concurrency, but check SQLCipher compatibility)
    // SQLCipher supports WAL.
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    // 4. Synchronous mode
    conn.execute_batch("PRAGMA synchronous = NORMAL;")?;

    // 5. Busy timeout: let SQLite retry internally for up to 5s before returning SQLITE_BUSY
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

    Ok(())
}

/// Apply only the key, cipher compatibility, and busy timeout pragmas.
/// Use this when opening a second connection to a vault that is already
/// open by another connection (e.g. backup source). Setting journal_mode
/// or synchronous requires exclusive access and will deadlock if another
/// connection holds the file open.
pub fn apply_key_pragmas(conn: &Connection, key: &SqlCipherKey) -> Result<()> {
    let pragma_key = format!("PRAGMA key = \"{}\"", key.expose_secret());
    conn.execute_batch(&pragma_key)?;
    conn.execute_batch("PRAGMA cipher_compatibility = 4;")?;
    conn.execute_batch("PRAGMA busy_timeout = 5000;")?;
    Ok(())
}
