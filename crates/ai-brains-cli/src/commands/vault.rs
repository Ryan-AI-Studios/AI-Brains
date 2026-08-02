//! `ai-brains vault` operator commands (T187).
//!
//! `vault encrypt` converts a plaintext SQLite vault to SQLCipher via
//! `sqlcipher_export` (not Online Backup).

use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::{EncryptOptions, encrypt_plaintext_vault, is_plain_sqlite_header};
use std::path::PathBuf;

/// Default zero-key material (refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1`).
const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

pub struct EncryptCliOptions {
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub key: Option<String>,
    /// When true, replace source with encrypted file (plain moved to `*.bak-plain`).
    pub confirm: bool,
    pub dry_run: bool,
}

pub fn run_encrypt(opts: EncryptCliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let key_str = opts.key.unwrap_or_else(|| ZERO_KEY.to_string());
    let key = SqlCipherKey::try_from_raw(key_str).map_err(|e| e.to_string())?;

    let source = opts.source;
    if !source.exists() {
        return Err(format!("source vault does not exist: {}", source.display()).into());
    }

    if !is_plain_sqlite_header(&source) {
        return Err(format!(
            "source is not a plaintext SQLite vault (no SQLite format 3 header): {}. \
             vault encrypt is only for plain→SQLCipher migration.",
            source.display()
        )
        .into());
    }

    let default_dest = source.with_file_name(format!(
        "{}.encrypted",
        source
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "vault.db".into())
    ));
    let dest = opts.destination.clone().unwrap_or(default_dest);

    // Policy:
    // - `--dry-run` always previews
    // - without `--confirm` and without `--destination`: dry-run preview (safe default)
    // - with `--destination`: write dest (unless `--dry-run`)
    // - with `--confirm`: export + replace source (unless `--dry-run`)
    let dry_run = opts.dry_run || (!opts.confirm && opts.destination.is_none());

    if dry_run {
        let target = if opts.confirm {
            source.display().to_string()
        } else {
            dest.display().to_string()
        };
        println!(
            "[dry-run] Would encrypt plaintext vault {} → {} via sqlcipher_export{}; no files written.",
            source.display(),
            target,
            if opts.confirm {
                " and replace source (original → *.bak-plain)"
            } else {
                ""
            }
        );
        return Ok(());
    }

    if opts.confirm {
        let tmp = source.with_file_name(format!(
            "{}.encrypted.tmp",
            source
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "vault.db".into())
        ));
        let written = encrypt_plaintext_vault(
            &EncryptOptions {
                source: source.clone(),
                destination: tmp,
                replace_source: true,
                dry_run: false,
            },
            &key,
        )?;
        println!(
            "Vault encrypted and replaced at {} (sqlcipher_export). Original plain copy kept as sibling *.bak-plain when rename succeeds.",
            written.display()
        );
        return Ok(());
    }

    let written = encrypt_plaintext_vault(
        &EncryptOptions {
            source: source.clone(),
            destination: dest,
            replace_source: false,
            dry_run: false,
        },
        &key,
    )?;
    println!(
        "Vault encrypted via sqlcipher_export: {} (source plain vault left in place at {})",
        written.display(),
        source.display()
    );
    Ok(())
}
