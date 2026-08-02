//! `ai-brains recovery export` — write a RecoveryKit to a restricted file path (T188).
//!
//! Secrets discipline: passphrase via `--passphrase-file` or zero-echo TTY double-entry
//! (`rpassword`). Never kit JSON / DataKey / passphrase on stdout or tracing.
//! Does **not** call `AppContext::from_cli` (avoids `migrate()` while daemon is up — F16b).

use crate::commands::backup::probe_restore_daemon_busy;
use crate::commands::device::data_key_from_sqlcipher;
use crate::daemon_client::DaemonClient;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{RecoveryKit, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload, RecoveryKitCreatedPayload};
use ai_brains_store::EventStore;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zeroize::Zeroizing;

const MIN_PASSPHRASE_LEN: usize = 8;
const MAX_PASSPHRASE_FILE_BYTES: usize = 8 * 1024;

/// Options for `recovery export`.
pub struct ExportOptions {
    pub vault_path: PathBuf,
    pub key: Option<String>,
    pub output: PathBuf,
    pub passphrase_file: Option<PathBuf>,
    pub dry_run: bool,
    pub force: bool,
}

/// Production entry: probe daemon, then export (or dry-run).
pub async fn run_export(opts: ExportOptions) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let daemon_up = probe_restore_daemon_busy(&client).await;
    run_export_with_daemon_state(opts, daemon_up)
}

/// Core export with injectable daemon-up (unit tests; production via [`run_export`]).
pub fn run_export_with_daemon_state(
    opts: ExportOptions,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = resolve_sqlcipher_key(opts.key)?;
    let data_key = data_key_from_sqlcipher(&key)?;

    // Passphrase acquisition (F8 / F8b / F14).
    let passphrase = if opts.dry_run {
        validate_passphrase_source_dry_run(opts.passphrase_file.as_deref())?
    } else {
        acquire_passphrase(opts.passphrase_file.as_deref())?
    };

    if opts.dry_run {
        println!(
            "[dry-run] Would write RecoveryKit to {} (no file, no event).",
            opts.output.display()
        );
        // passphrase is Zeroizing; dropped here.
        drop(passphrase);
        return Ok(());
    }

    // Output exists → refuse unless --force/--overwrite (F9 / AC14).
    // Note: Path::exists follows reparse; reparse refuse below catches symlink outputs.
    if opts.output.exists() && !opts.force {
        return Err(format!(
            "output exists: {} (pass --force or --overwrite to replace)",
            opts.output.display()
        )
        .into());
    }

    // F8b defense-in-depth: refuse kit output through reparse/symlink/junction.
    let out_reparse = ai_brains_path::is_reparse_or_symlink(&opts.output)
        .map_err(|e| format!("output path check failed ({}): {e}", opts.output.display()))?;
    if let Err(msg) = ai_brains_path::refuse_if_reparse(&opts.output, out_reparse) {
        return Err(msg.into());
    }

    refuse_public_output_path(&opts.output)?;

    // Generate kit (F3). Never log passphrase/key/kit (F21).
    let kit = RecoveryKit::generate(&data_key, passphrase.as_slice())
        .map_err(|e| format!("RecoveryKit generate failed: {e}"))?;
    // Zeroize passphrase as soon as kit is generated.
    drop(passphrase);

    let kit_json = kit
        .to_json()
        .map_err(|e| format!("RecoveryKit serialize failed: {e}"))?;

    write_kit_file(&opts.output, kit_json.as_bytes())?;

    let dpapi_status = if kit.dpapi.is_some() {
        "present"
    } else {
        "absent"
    };
    // Stdout: path + dpapi status only (F11). Never kit JSON.
    println!("{}", opts.output.display());
    println!("dpapi: {dpapi_status}");

    // Event best-effort (F12). Kit file success is DoD even if append fails.
    if let Err(e) = try_append_recovery_kit_created(&opts.vault_path, &key, daemon_up) {
        // No secrets in warn path.
        eprintln!(
            "warning: RecoveryKitCreated event not appended ({e}); kit file was written successfully"
        );
        tracing::warn!(
            "RecoveryKitCreated event append failed; kit file written (event best-effort)"
        );
    }

    Ok(())
}

fn resolve_sqlcipher_key(key: Option<String>) -> Result<SqlCipherKey, Box<dyn std::error::Error>> {
    // Same default zero-key path as AppContext::from_cli (tests use ALLOW_ZERO_KEY).
    let key_str = key.unwrap_or_else(|| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });
    let sql = SqlCipherKey::from_raw(key_str);
    if let Err(e) = sql.validate() {
        return Err(format!("invalid vault key: {e}").into());
    }
    Ok(sql)
}

/// Read passphrase from file or TTY double-entry.
fn acquire_passphrase(
    passphrase_file: Option<&Path>,
) -> Result<Zeroizing<Vec<u8>>, Box<dyn std::error::Error>> {
    match passphrase_file {
        Some(path) => read_passphrase_file(path),
        None => read_passphrase_tty(),
    }
}

/// Dry-run: validate passphrase *source* without prompting TTY content (F14).
///
/// Returns a zeroizing buffer (file contents if file path; empty placeholder for TTY).
fn validate_passphrase_source_dry_run(
    passphrase_file: Option<&Path>,
) -> Result<Zeroizing<Vec<u8>>, Box<dyn std::error::Error>> {
    match passphrase_file {
        Some(path) => {
            // Read+zeroize to prove readability; min-length still enforced.
            read_passphrase_file(path)
        }
        None => {
            if !is_terminal::is_terminal(io::stdin()) {
                return Err("dry-run: no --passphrase-file and stdin is not a TTY; \
                     cannot validate interactive passphrase source"
                    .into());
            }
            // TTY present — do not prompt content in dry-run.
            Ok(Zeroizing::new(Vec::new()))
        }
    }
}

fn read_passphrase_file(path: &Path) -> Result<Zeroizing<Vec<u8>>, Box<dyn std::error::Error>> {
    // F8b: refuse symlink/reparse/junction before any follow-open (Codex R1 P2).
    // is_reparse_or_symlink uses symlink_metadata and does not follow.
    let is_reparse = ai_brains_path::is_reparse_or_symlink(path)
        .map_err(|e| format!("passphrase file not readable ({}): {e}", path.display()))?;
    if let Err(msg) = ai_brains_path::refuse_if_reparse(path, is_reparse) {
        return Err(msg.into());
    }

    // Prefer symlink_metadata for size/type so we do not follow (post-refuse defense).
    let meta = fs::symlink_metadata(path)
        .map_err(|e| format!("passphrase file not readable ({}): {e}", path.display()))?;
    if meta.is_dir() {
        return Err(format!(
            "passphrase path is a directory, not a regular file: {}",
            path.display()
        )
        .into());
    }
    // Best-effort: refuse non-regular files where the platform reports file_type.
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        let ft = meta.file_type();
        if ft.is_fifo() || ft.is_socket() || ft.is_block_device() || ft.is_char_device() {
            return Err(format!(
                "passphrase path must be a regular file (not pipe/device/socket): {}",
                path.display()
            )
            .into());
        }
    }
    if meta.len() > MAX_PASSPHRASE_FILE_BYTES as u64 {
        return Err(format!(
            "passphrase file too large (max {MAX_PASSPHRASE_FILE_BYTES} bytes): {}",
            path.display()
        )
        .into());
    }

    let mut file = File::open(path)
        .map_err(|e| format!("failed to open passphrase file {}: {e}", path.display()))?;
    let mut buf = Zeroizing::new(Vec::with_capacity(meta.len() as usize));
    file.read_to_end(&mut buf)
        .map_err(|e| format!("failed to read passphrase file: {e}"))?;

    // Strip a single trailing newline (`\n` or `\r\n`) common for text files.
    trim_trailing_newline(&mut buf);

    if buf.len() < MIN_PASSPHRASE_LEN {
        return Err(format!(
            "passphrase too short: minimum {MIN_PASSPHRASE_LEN} bytes after trailing-newline trim"
        )
        .into());
    }
    Ok(buf)
}

fn trim_trailing_newline(buf: &mut Vec<u8>) {
    if buf.ends_with(b"\r\n") {
        buf.truncate(buf.len() - 2);
    } else if buf.ends_with(b"\n") {
        buf.truncate(buf.len() - 1);
    }
}

fn read_passphrase_tty() -> Result<Zeroizing<Vec<u8>>, Box<dyn std::error::Error>> {
    if !is_terminal::is_terminal(io::stdin()) {
        return Err(
            "no --passphrase-file and stdin is not a TTY; cannot prompt for passphrase \
             (use --passphrase-file <path>)"
                .into(),
        );
    }

    // rpassword: zero-echo (F17 / F20).
    eprint!("Recovery kit passphrase: ");
    let _ = io::stderr().flush();
    let first =
        rpassword::read_password().map_err(|e| format!("failed to read passphrase: {e}"))?;
    eprint!("Confirm passphrase: ");
    let _ = io::stderr().flush();
    let second = rpassword::read_password()
        .map_err(|e| format!("failed to read passphrase confirm: {e}"))?;

    let first_z = Zeroizing::new(first);
    let second_z = Zeroizing::new(second);

    if first_z.as_str() != second_z.as_str() {
        return Err("passphrases do not match".into());
    }
    if first_z.len() < MIN_PASSPHRASE_LEN {
        return Err(format!("passphrase too short: minimum {MIN_PASSPHRASE_LEN} bytes").into());
    }

    Ok(Zeroizing::new(first_z.as_bytes().to_vec()))
}

/// Refuse well-known public/shared paths on Windows (F9b).
fn refuse_public_output_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        let display = path.to_string_lossy();
        let lower = display.to_ascii_lowercase();
        // Well-known public roots (case-insensitive).
        let forbidden = [
            r"c:\users\public",
            r"c:\users\public\",
            r"\users\public\",
            r"c:\public",
        ];
        for f in forbidden {
            if lower.starts_with(f) || lower.contains(r"\users\public\") {
                return Err(format!(
                    "refusing public/shared output path (Windows): {} \
                     (do not write RecoveryKit under C:\\Users\\Public)",
                    path.display()
                )
                .into());
            }
        }
    }
    let _ = path;
    Ok(())
}

fn write_kit_file(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    // Prefer create_new when not overwriting; force path already checked by caller
    // (exists + force). When force, truncate existing.
    let mut opts = OpenOptions::new();
    opts.write(true);
    if path.exists() {
        opts.truncate(true);
    } else {
        opts.create_new(true);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let mut file = opts
        .open(path)
        .map_err(|e| format!("failed to create kit file {}: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("failed to write kit file: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("failed to sync kit file: {e}"))?;

    // Best-effort owner-only ACL on Windows when portable (F9b).
    #[cfg(windows)]
    {
        if let Err(e) = restrict_windows_acl_best_effort(path) {
            tracing::warn!(
                "best-effort owner-only ACL for RecoveryKit file failed: {e} (file written)"
            );
        }
    }

    Ok(())
}

#[cfg(windows)]
fn restrict_windows_acl_best_effort(path: &Path) -> Result<(), String> {
    // Portable best-effort: invoke icacls to grant only the current user full control
    // and remove inheritance. Failure is non-fatal for the export.
    let path_str = path.to_string_lossy().to_string();
    let username = std::env::var("USERNAME").unwrap_or_else(|_| String::from("%USERNAME%"));
    let status = std::process::Command::new("icacls")
        .args([
            &path_str,
            "/inheritance:r",
            "/grant:r",
            &format!("{username}:(F)"),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("icacls spawn failed: {e}"))?;
    if !status.success() {
        return Err(format!("icacls exited with {status}"));
    }
    Ok(())
}

/// Append RecoveryKitCreated when possible. Prefer no-migrate open when daemon is up (F16b).
fn try_append_recovery_kit_created(
    vault_path: &Path,
    key: &SqlCipherKey,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let key_id = Uuid::new_v4();
    let event = EventBuilder::new(
        AggregateType::System,
        key_id,
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::RecoveryKitCreated(RecoveryKitCreatedPayload {
        key_id: key_id.to_string(),
    }))?;

    if daemon_up {
        // Avoid migrate while daemon holds the vault. Event append needs a writer —
        // open R/W without migrate; if that fails (busy), surface soft error.
        let conn = open_without_migrate(vault_path, key)?;
        let store = SqliteEventStore::new(conn);
        store.append_event(&event)?;
        return Ok(());
    }

    // Daemon down: open + migrate if needed, then append.
    let conn = VaultConnection::open(vault_path, key)?;
    conn.migrate()?;
    let store = SqliteEventStore::new(conn);
    store.append_event(&event)?;
    Ok(())
}

/// Open vault R/W without running migrations (F16b when daemon may be up).
fn open_without_migrate(
    vault_path: &Path,
    key: &SqlCipherKey,
) -> Result<VaultConnection, Box<dyn std::error::Error>> {
    // VaultConnection::open does not migrate — only AppContext::from_cli does.
    // So open() alone is the no-migrate path.
    Ok(VaultConnection::open(vault_path, key)?)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use std::sync::Mutex;

    // Serialize tests that touch process env / filesystem edge cases.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn trim_trailing_newline__strips_lf_and_crlf() {
        let mut a = b"secret-pass\n".to_vec();
        trim_trailing_newline(&mut a);
        assert_eq!(a, b"secret-pass");

        let mut b = b"secret-pass\r\n".to_vec();
        trim_trailing_newline(&mut b);
        assert_eq!(b, b"secret-pass");

        let mut c = b"secret-pass".to_vec();
        trim_trailing_newline(&mut c);
        assert_eq!(c, b"secret-pass");
    }

    #[test]
    fn read_passphrase_file__too_short__errors() {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pw.txt");
        fs::write(&path, b"short").unwrap();
        let err = read_passphrase_file(&path).unwrap_err().to_string();
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("passphrase") && lower.contains("short"),
            "got: {err}"
        );
    }

    #[test]
    fn read_passphrase_file__ok_min_length() {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pw.txt");
        fs::write(&path, b"12345678\n").unwrap();
        let buf = read_passphrase_file(&path).unwrap();
        assert_eq!(buf.as_slice(), b"12345678");
    }

    /// F8b / Codex R1 P2: passphrase-file must refuse symlink/reparse paths.
    #[test]
    fn recovery_export__passphrase_file_symlink__refuses() {
        let _g = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("pw.txt");
        fs::write(&target, b"test-passphrase-long-enough").unwrap();
        let link = dir.path().join("pw-link.txt");

        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&target, &link);
        #[cfg(not(windows))]
        let created = std::os::unix::fs::symlink(&target, &link);

        if let Err(e) = created {
            eprintln!(
                "skipping recovery_export__passphrase_file_symlink__refuses: {e} \
                 (needs Developer Mode or elevation on Windows)"
            );
            return;
        }

        assert!(
            ai_brains_path::is_reparse_or_symlink(&link).expect("symlink_metadata"),
            "precondition: file symlink must be detected as reparse"
        );

        let err = read_passphrase_file(&link).unwrap_err().to_string();
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("symlink") || lower.contains("reparse") || lower.contains("junction"),
            "expected reparse/symlink refuse for passphrase-file, got: {err}"
        );
    }

    #[test]
    fn recovery_export__output_exists__refuses_without_force() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = VaultConnection::open(&vault, &key).unwrap();
        conn.migrate().unwrap();
        drop(conn);

        let out = dir.path().join("kit.json");
        fs::write(&out, b"existing").unwrap();
        let pw = dir.path().join("pw.txt");
        fs::write(&pw, b"test-passphrase-long-enough").unwrap();

        let err = run_export_with_daemon_state(
            ExportOptions {
                vault_path: vault,
                key: Some(
                    "x'0000000000000000000000000000000000000000000000000000000000000000'"
                        .to_string(),
                ),
                output: out,
                passphrase_file: Some(pw),
                dry_run: false,
                force: false,
            },
            false,
        )
        .unwrap_err()
        .to_string();
        let lower = err.to_ascii_lowercase();
        assert!(
            lower.contains("exists") || lower.contains("output exists"),
            "got: {err}"
        );
    }

    #[test]
    fn recovery_export__dry_run__no_file() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        let key = SqlCipherKey::from_raw(
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string(),
        );
        let conn = VaultConnection::open(&vault, &key).unwrap();
        conn.migrate().unwrap();
        drop(conn);

        let out = dir.path().join("kit.json");
        let pw = dir.path().join("pw.txt");
        fs::write(&pw, b"test-passphrase-long-enough").unwrap();

        run_export_with_daemon_state(
            ExportOptions {
                vault_path: vault,
                key: Some(
                    "x'0000000000000000000000000000000000000000000000000000000000000000'"
                        .to_string(),
                ),
                output: out.clone(),
                passphrase_file: Some(pw),
                dry_run: true,
                force: false,
            },
            false,
        )
        .expect("dry-run must succeed");
        assert!(!out.exists(), "dry-run must not write kit file");
    }

    #[test]
    fn recovery_export__passphrase_file__writes_unlockable_kit() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        let key_str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
        let key = SqlCipherKey::from_raw(key_str.clone());
        let conn = VaultConnection::open(&vault, &key).unwrap();
        conn.migrate().unwrap();
        drop(conn);

        let out = dir.path().join("kit.json");
        let pw_path = dir.path().join("pw.txt");
        let passphrase = b"test-passphrase-long-enough";
        fs::write(&pw_path, passphrase).unwrap();

        run_export_with_daemon_state(
            ExportOptions {
                vault_path: vault,
                key: Some(key_str),
                output: out.clone(),
                passphrase_file: Some(pw_path),
                dry_run: false,
                force: false,
            },
            false,
        )
        .expect("export must succeed");

        assert!(out.exists(), "kit file must exist");
        let json = fs::read_to_string(&out).unwrap();
        let kit = RecoveryKit::from_json(&json).expect("parse kit");
        assert_eq!(kit.schema_version, 1);
        let unlocked = kit
            .unlock_with_passphrase(passphrase)
            .expect("unlock with same passphrase");
        let expected = data_key_from_sqlcipher(&key).unwrap();
        assert_eq!(unlocked.expose_secret(), expected.expose_secret());
    }

    #[test]
    fn recovery_export__daemon_down__appends_recovery_kit_created_event() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        let key_str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
        let key = SqlCipherKey::from_raw(key_str.clone());
        let conn = VaultConnection::open(&vault, &key).unwrap();
        conn.migrate().unwrap();
        drop(conn);

        let out = dir.path().join("kit.json");
        let pw_path = dir.path().join("pw.txt");
        fs::write(&pw_path, b"test-passphrase-long-enough").unwrap();

        run_export_with_daemon_state(
            ExportOptions {
                vault_path: vault.clone(),
                key: Some(key_str.clone()),
                output: out.clone(),
                passphrase_file: Some(pw_path),
                dry_run: false,
                force: false,
            },
            false, // daemon down — migrate + append path
        )
        .expect("export must succeed");

        assert!(out.exists(), "kit file must exist");

        let conn = VaultConnection::open(&vault, &key).expect("reopen vault");
        let store = SqliteEventStore::new(conn);
        let events = store.read_all_events().expect("read events");
        let kit_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.payload, Payload::RecoveryKitCreated(_)))
            .collect();
        assert!(
            !kit_events.is_empty(),
            "expected at least one RecoveryKitCreated event; got {} total events",
            events.len()
        );
        assert!(
            matches!(
                kit_events[0].payload,
                Payload::RecoveryKitCreated(RecoveryKitCreatedPayload { ref key_id })
                    if !key_id.is_empty()
            ),
            "RecoveryKitCreated payload must include non-empty key_id"
        );
    }

    #[test]
    fn recovery_export__daemon_up__no_migrate_kit_ok() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set(ai_brains_store::connection::ALLOW_ZERO_KEY_ENV, "1");
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        let key_str =
            "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string();
        let key = SqlCipherKey::from_raw(key_str.clone());
        let conn = VaultConnection::open(&vault, &key).unwrap();
        conn.migrate().unwrap();
        drop(conn);

        let out = dir.path().join("kit.json");
        let pw_path = dir.path().join("pw.txt");
        fs::write(&pw_path, b"test-passphrase-long-enough").unwrap();

        // daemon_up=true: kit file still written; event soft-fail is OK.
        run_export_with_daemon_state(
            ExportOptions {
                vault_path: vault,
                key: Some(key_str),
                output: out.clone(),
                passphrase_file: Some(pw_path),
                dry_run: false,
                force: false,
            },
            true, // daemon_up — must not require migrate for kit success
        )
        .expect("kit export must succeed even when daemon_up");

        assert!(out.exists(), "kit file must exist with daemon_up");
        let json = fs::read_to_string(&out).unwrap();
        let kit = RecoveryKit::from_json(&json).expect("parse kit");
        assert_eq!(kit.schema_version, 1);
    }

    #[cfg(windows)]
    #[test]
    fn refuse_public_output_path__users_public__refuses() {
        let p = PathBuf::from(r"C:\Users\Public\recovery-kit.json");
        let err = refuse_public_output_path(&p).unwrap_err().to_string();
        assert!(err.to_ascii_lowercase().contains("public"), "got: {err}");
    }
}
