//! Plain SQLite header sniff (T187 F17).
//!
//! Used before applying `PRAGMA key` on existing vault files so operators get a
//! clear migrate path (`vault encrypt`) instead of a generic VaultLocked error.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Magic for unencrypted SQLite databases (16 bytes including trailing NUL).
pub const SQLITE_PLAIN_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// True when `path` exists, is non-empty, and starts with the plain SQLite header.
///
/// Missing, empty, or unreadable files return `false` (caller handles open errors).
pub fn is_plain_sqlite_header(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 16];
    match file.read_exact(&mut buf) {
        Ok(()) => buf == *SQLITE_PLAIN_HEADER || buf.starts_with(b"SQLite format 3"),
        Err(_) => false,
    }
}

/// Standard migrate hint for [`crate::errors::StoreError::LegacyPlaintextVault`].
pub fn legacy_plaintext_migrate_hint() -> String {
    "this file is a plaintext SQLite database (not SQLCipher page-encrypted). \
     Run `ai-brains vault encrypt --confirm` with a non-zero --key to convert via \
     sqlcipher_export. Opening with PRAGMA key will not encrypt an existing plain vault."
        .to_string()
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn is_plain_sqlite_header__true_for_sqlite_magic() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("plain.db");
        let mut f = File::create(&path).expect("create");
        f.write_all(b"SQLite format 3\0").expect("write");
        f.write_all(&[0u8; 100]).expect("pad");
        assert!(is_plain_sqlite_header(&path));
    }

    #[test]
    fn is_plain_sqlite_header__false_for_encrypted_looking() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("enc.db");
        let mut f = File::create(&path).expect("create");
        f.write_all(&[0xABu8; 64]).expect("write");
        assert!(!is_plain_sqlite_header(&path));
    }
}
