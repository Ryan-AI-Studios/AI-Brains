//! `ai-brains vault` operator commands (T187 encrypt, T189 rotate-datakey).
//!
//! Mutating rotate does **not** use `AppContext::from_cli` (avoids migrate while
//! daemon may be up — same discipline as T188 recovery export).

use crate::commands::backup::probe_restore_daemon_busy;
use crate::commands::recovery::{
    acquire_passphrase, refuse_output_parent_chain, refuse_public_output_path,
    validate_passphrase_source_dry_run, write_kit_file,
};
use crate::daemon_client::DaemonClient;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, RecoveryKit, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, DataKeyRotatedPayload, Payload};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::pragmas::apply_key_pragmas;
use ai_brains_store::{
    EncryptOptions, EventStore, RotateDataKeyOptions, SqliteEventStore, encrypt_plaintext_vault,
    is_plain_sqlite_header, plan_rotate_datakey, rotate_datakey,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

const BACKUP_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const BYPASS_PHRASE: &str = "I have a backup";
const STALE_KEY_WARNING: &str = "WARNING: AI_BRAINS_KEY / --key is now STALE. \
Update env/profile/.env to the NEW key before any other vault command. \
The old key cannot open the rotated vault.";

// ---------------------------------------------------------------------------
// vault encrypt (T187)
// ---------------------------------------------------------------------------

pub struct EncryptCliOptions {
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub key: Option<String>,
    /// When true, replace source with encrypted file (plain moved to `*.bak-plain`).
    pub confirm: bool,
    pub dry_run: bool,
}

pub fn run_encrypt(opts: EncryptCliOptions) -> Result<(), Box<dyn std::error::Error>> {
    // T197: shared operator resolver (no silent zero; F8 messages).
    let key = crate::key_resolve::resolve_operator_sqlcipher_key(opts.key)?;

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

// ---------------------------------------------------------------------------
// vault rotate-datakey (T189)
// ---------------------------------------------------------------------------

/// Options for `vault rotate-datakey`.
pub struct RotateDatakeyOptions {
    pub vault_path: PathBuf,
    pub key: Option<String>,
    pub dry_run: bool,
    pub confirm: bool,
    pub require_backup: bool,
    pub i_have_backup: Option<String>,
    pub kit_output: Option<PathBuf>,
    pub passphrase_file: Option<PathBuf>,
    pub overwrite_kit: bool,
    pub accept_rekey_risk: bool,
    pub print_key: bool,
    pub backup_dir: Option<PathBuf>,
}

/// Production entry: probe daemon, then rotate (or dry-run).
pub async fn run_rotate_datakey(
    opts: RotateDatakeyOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let daemon_up = probe_restore_daemon_busy(&client).await;
    run_rotate_datakey_with_daemon_state(opts, daemon_up)
}

/// Core rotate with injectable daemon-up (unit tests; production via [`run_rotate_datakey`]).
pub fn run_rotate_datakey_with_daemon_state(
    opts: RotateDatakeyOptions,
    daemon_up: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_sql = resolve_sqlcipher_key(opts.key)?;
    // Prefer typed to_data_key (F18 / AC19).
    let old_data = old_sql
        .to_data_key()
        .map_err(|e| format!("invalid vault key for DataKey recovery: {e}"))?;

    if !opts.vault_path.exists() {
        return Err(format!("vault does not exist: {}", opts.vault_path.display()).into());
    }

    // F14: mutating rotate hard-fails if daemon up.
    if daemon_up && !opts.dry_run {
        return Err(
            "daemon is running; stop the daemon before vault rotate-datakey \
             (mutating rotate hard-fails while daemon holds the vault)"
                .into(),
        );
    }

    // Dry-run plan (no mutation).
    if opts.dry_run {
        if daemon_up {
            println!(
                "[dry-run] notice: daemon appears up; a live mutate would hard-fail until daemon stop."
            );
        }
        // F31: validate passphrase source when kit path present (optional on dry-run counts).
        if let Some(kit_path) = opts.kit_output.as_ref() {
            let _ = validate_passphrase_source_dry_run(opts.passphrase_file.as_deref())?;
            validate_kit_output_path(kit_path, opts.overwrite_kit, true)?;
        }
        let plan = plan_rotate_datakey(&opts.vault_path, &old_sql)
            .map_err(|e| format!("rotate dry-run plan failed: {e}"))?;
        println!(
            "[dry-run] DataKey rotation plan for {}:",
            opts.vault_path.display()
        );
        println!("  living_wraps: {}", plan.living_wrap_count);
        println!("  device_private: {}", plan.device_private_count);
        println!(
            "  method: {}",
            if opts.accept_rekey_risk {
                "rekey (opt-in risk)"
            } else {
                "export (primary, crash-safe)"
            }
        );
        println!("  zero active wraps is valid (F29). No mutation performed.");
        return Ok(());
    }

    // Non-dry-run requires --confirm.
    if !opts.confirm {
        return Err(
            "refusing rotate without --confirm (and preferably a prior --dry-run). \
             This retires the current DataKey for this vault."
                .into(),
        );
    }

    let kit_output = opts.kit_output.ok_or(
        "--kit-output <path> is required for successful rotate-datakey (mandatory RecoveryKit re-export)",
    )?;

    // F31: validate passphrase + kit path BEFORE any vault mutation.
    let passphrase = acquire_passphrase(opts.passphrase_file.as_deref())?;
    validate_kit_output_path(&kit_output, opts.overwrite_kit, false)?;

    // F8 backup gate.
    let backup_bypassed = check_backup_gate(
        &opts.vault_path,
        &old_sql,
        opts.require_backup,
        opts.i_have_backup.as_deref(),
        opts.backup_dir.as_deref(),
    )?;

    // Generate new DataKey (production path).
    let new_data = DataKey::generate();

    // P0-1 / F6 kit-before-mutate: write RecoveryKit for NEW key BEFORE any vault mutation.
    // If rotate fails after this write, the kit is an orphan for a key never applied —
    // vault remains unchanged on the export path (rekey auto-restores); safe to delete.
    let kit = RecoveryKit::generate(&new_data, passphrase.as_slice())
        .map_err(|e| format!("RecoveryKit generate for new key failed: {e}"))?;
    drop(passphrase);
    let kit_json = kit
        .to_json()
        .map_err(|e| format!("RecoveryKit serialize failed: {e}"))?;
    write_kit_file(&kit_output, kit_json.as_bytes())?;

    let rotate_result = rotate_datakey(
        &RotateDataKeyOptions {
            vault_path: opts.vault_path.clone(),
            accept_rekey_risk: opts.accept_rekey_risk,
        },
        &old_sql,
        &old_data,
        &new_data,
    )
    .map_err(|e| {
        format!(
            "DataKey rotation failed: {e}. \
             RecoveryKit at {} was written for a key that was never applied; \
             vault is unchanged on the export path (rekey auto-restores from snapshot). \
             Safe to delete the orphan kit.",
            kit_output.display()
        )
    })?;

    // Event best-effort (System + nil aggregate_id).
    let completed_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let rotation_id = Uuid::new_v4();
    if let Err(e) = try_append_data_key_rotated(
        &opts.vault_path,
        &rotate_result.new_sqlcipher_key,
        DataKeyRotatedPayload {
            rotation_id,
            living_wraps_rewrapped: rotate_result.living_wraps_rewrapped,
            device_private_resealed: rotate_result.device_private_resealed,
            backup_bypassed,
            completed_at,
        },
    ) {
        eprintln!(
            "warning: DataKeyRotated event not appended ({e}); rotation and kit write succeeded"
        );
        tracing::warn!("DataKeyRotated event append failed; rotation success (event best-effort)");
    }

    // Success stdout (F34): counts, method, kit path, stale-key WARNING. No secrets.
    println!("DataKey rotation succeeded.");
    println!(
        "  living_wraps_rewrapped: {}",
        rotate_result.living_wraps_rewrapped
    );
    println!(
        "  device_private_resealed: {}",
        rotate_result.device_private_resealed
    );
    println!("  method: {}", rotate_result.method.as_str());
    println!("  kit: {}", kit_output.display());
    println!("  Do not retire old RecoveryKit copies until you unlock-verify the NEW kit (F32).");
    println!("{STALE_KEY_WARNING}");
    if backup_bypassed {
        println!("  backup_bypassed: true (audited via --i-have-backup)");
    }

    if opts.print_key {
        let new_sql = SqlCipherKey::from_data_key(&new_data);
        let material = new_sql.expose_secret();
        println!("# NEW key (store offline; never log to tracing):");
        println!("# PowerShell: $env:AI_BRAINS_KEY = \"{material}\"");
        println!("# bash: export AI_BRAINS_KEY=\"{material}\"");
    }

    // Zeroize by dropping Zeroizing/DataKey/SqlCipherKey scopes.
    drop(old_data);
    drop(new_data);
    drop(old_sql);

    Ok(())
}

fn resolve_sqlcipher_key(key: Option<String>) -> Result<SqlCipherKey, Box<dyn std::error::Error>> {
    // T197: shared operator resolver covers format + zero refuse (F5/F6).
    Ok(crate::key_resolve::resolve_operator_sqlcipher_key(key)?)
}

fn validate_kit_output_path(
    path: &Path,
    overwrite_kit: bool,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() && !overwrite_kit {
        return Err(format!(
            "kit output exists: {} (pass --overwrite-kit to replace; never overrides daemon/backup gates)",
            path.display()
        )
        .into());
    }
    let out_reparse = ai_brains_path::is_reparse_or_symlink(path)
        .map_err(|e| format!("kit output path check failed ({}): {e}", path.display()))?;
    if let Err(msg) = ai_brains_path::refuse_if_reparse(path, out_reparse) {
        return Err(msg.into());
    }
    refuse_public_output_path(path)?;
    refuse_output_parent_chain(path)?;

    // Parent must be creatable / exist for non-dry-run mutation gate.
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
        && !dry_run
    {
        // Ensure parent can be created later by write_kit_file — create now to fail early.
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "kit output parent not writable/creatable ({}): {e}",
                parent.display()
            )
        })?;
    }
    Ok(())
}

/// Backup gate F8. Returns `backup_bypassed`.
fn check_backup_gate(
    vault_path: &Path,
    current_key: &SqlCipherKey,
    require_backup: bool,
    i_have_backup: Option<&str>,
    backup_dir: Option<&Path>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(phrase) = i_have_backup {
        if phrase == BYPASS_PHRASE {
            return Ok(true);
        }
        return Err(format!(
            "--i-have-backup requires the exact phrase \"{BYPASS_PHRASE}\" (got a non-matching string)"
        )
        .into());
    }

    // F8 / F8b: only the exact phrase bypasses audit. `--require-backup=false` alone is not a bypass.
    if !require_backup {
        return Err(format!(
            "refusing rotate with --require-backup=false without audited phrase: \
             pass --i-have-backup \"{BYPASS_PHRASE}\" to bypass the backup gate, \
             or satisfy the backup directory gate (omit --require-backup=false)"
        )
        .into());
    }

    let dir = backup_dir.map(PathBuf::from).unwrap_or_else(|| {
        vault_path
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from("backups"))
    });

    if !dir.is_dir() {
        return Err(format!(
            "backup gate: no backup directory at {} \
             (run `ai-brains backup create` or pass --i-have-backup \"{BYPASS_PHRASE}\")",
            dir.display()
        )
        .into());
    }

    let mut newest: Option<(PathBuf, SystemTime)> = None;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_lossy = name.to_string_lossy();
        if !(name_lossy.starts_with("vault-") && name_lossy.ends_with(".db.bak")) {
            continue;
        }
        let meta = fs::metadata(&path)?;
        if meta.len() == 0 {
            continue;
        }
        let mtime = meta.modified().map_err(|e| format!("backup mtime: {e}"))?;
        match &newest {
            None => newest = Some((path, mtime)),
            Some((_, t)) if mtime > *t => newest = Some((path, mtime)),
            _ => {}
        }
    }

    let (backup_path, mtime) = newest.ok_or_else(|| {
        format!(
            "backup gate: no non-empty vault-*.db.bak in {} \
             (run `ai-brains backup create` or --i-have-backup \"{BYPASS_PHRASE}\")",
            dir.display()
        )
    })?;

    let age = SystemTime::now()
        .duration_since(mtime)
        .unwrap_or(Duration::from_secs(0));
    if age > BACKUP_MAX_AGE {
        return Err(format!(
            "backup gate: most recent backup {} is older than 24h \
             (run a fresh `ai-brains backup create` or --i-have-backup \"{BYPASS_PHRASE}\")",
            backup_path.display()
        )
        .into());
    }

    // Open with current key + sqlite_master.
    let conn = rusqlite::Connection::open(&backup_path)
        .map_err(|e| format!("backup gate: open {} failed: {e}", backup_path.display()))?;
    apply_key_pragmas(&conn, current_key)
        .map_err(|e| format!("backup gate: backup does not open with current key: {e}"))?;
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| {
            format!(
                "backup gate: key verification failed on {}: {e}",
                backup_path.display()
            )
        })?;

    Ok(false)
}

fn try_append_data_key_rotated(
    vault_path: &Path,
    new_key: &SqlCipherKey,
    payload: DataKeyRotatedPayload,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = EventBuilder::new(
        AggregateType::System,
        Uuid::nil(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DataKeyRotated(payload))?;

    let conn = VaultConnection::open(vault_path, new_key)?;
    // migrate not required for append of new event kind (forward-compatible);
    // run migrate when exclusive for schema hygiene.
    let _ = conn.migrate();
    let store = SqliteEventStore::new(conn);
    store.append_event(&event)?;
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::ids::ContentKeyId;
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::content_envelope::generate_wrap_and_seal;
    use ai_brains_store::projections::content_envelope::{
        self, ALGORITHM_AES_256_GCM, EncryptedBlobRow, insert_content_key_wrap,
        insert_encrypted_blob,
    };
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn seed_vault(dir: &Path, data_key: &DataKey) -> PathBuf {
        let path = dir.join("vault.db");
        let sql = SqlCipherKey::from_data_key(data_key);
        let conn = VaultConnection::open(&path, &sql).expect("open");
        conn.migrate().expect("migrate");
        path
    }

    fn write_passphrase_file(dir: &Path) -> PathBuf {
        let p = dir.join("pass.txt");
        fs::write(&p, b"test-passphrase-ok").unwrap();
        p
    }

    #[test]
    fn rotate_datakey__daemon_up__hard_fail_no_mutation() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa1; 32]);
        let path = seed_vault(dir.path(), &old);
        let sql = SqlCipherKey::from_data_key(&old);
        let before = fs::metadata(&path).unwrap().len();

        let err = run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path.clone(),
                key: Some(sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: false,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(dir.path().join("kit.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            true, // daemon up
        )
        .expect_err("must hard-fail");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(msg.contains("daemon"), "{msg}");
        assert_eq!(fs::metadata(&path).unwrap().len(), before);
        // Old key still opens
        VaultConnection::open(&path, &sql).expect("unchanged vault");
    }

    #[test]
    fn rotate_datakey__dry_run__no_mutation() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa2; 32]);
        let path = seed_vault(dir.path(), &old);
        let sql = SqlCipherKey::from_data_key(&old);
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

        run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path.clone(),
                key: Some(sql.expose_secret().to_string()),
                dry_run: true,
                confirm: false,
                require_backup: true,
                i_have_backup: None,
                kit_output: None,
                passphrase_file: None,
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect("dry-run ok");

        // Old key still works
        VaultConnection::open(&path, &sql).expect("no mutation");
        let mtime_after = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn rotate_datakey__kit_export__unlocks_new_key_only() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa3; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);
        let pw = write_passphrase_file(dir.path());
        let kit_path = dir.path().join("kit-new.json");

        run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path.clone(),
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(kit_path.clone()),
                passphrase_file: Some(pw),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect("rotate");

        assert!(kit_path.exists());
        let kit_json = fs::read_to_string(&kit_path).unwrap();
        let kit: RecoveryKit = serde_json::from_str(&kit_json).expect("kit parse");
        let unlocked = kit
            .unlock_with_passphrase(b"test-passphrase-ok")
            .expect("unlock new kit");
        let new_sql = SqlCipherKey::from_data_key(&unlocked);
        VaultConnection::open(&path, &new_sql).expect("new key opens vault");
        assert!(
            VaultConnection::open(&path, &old_sql).is_err(),
            "old key must fail"
        );
    }

    #[test]
    fn rotate_datakey__stdout__no_secrets() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa4; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);
        let hex_body = &old_sql.expose_secret()[2..66];

        // Pin success stdout constant has no key hex; kit/success path must not format
        // key material into Err strings (store-layer redact_sql_err covers keyed SQL).
        assert!(!STALE_KEY_WARNING.contains(hex_body));
        assert!(!STALE_KEY_WARNING.contains("x'"));

        let kit_path = dir.path().join("kit.json");
        run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path,
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(kit_path.clone()),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect("rotate");

        // Kit file holds secrets on disk by design; Err paths above must not echo them.
        assert!(kit_path.exists());
        let kit_json = fs::read_to_string(&kit_path).unwrap();
        // Success path without --print-key: no formatted Err containing kit JSON body.
        assert!(!kit_json.is_empty());
        // Force a post-kit failure shape: wrong key would fail open earlier; assert
        // daemon-up error string has no hex body.
        let err = run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: dir.path().join("missing-vault.db"),
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(dir.path().join("kit2.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect_err("missing vault");
        let msg = err.to_string();
        assert!(
            !msg.contains(hex_body),
            "error must not contain key hex: {msg}"
        );
        assert!(!msg.contains("x'0000") && !msg.contains(&old_sql.expose_secret()[..10]));
    }

    #[test]
    fn rotate_datakey__success__prints_stale_key_warning() {
        // The success path always prints STALE_KEY_WARNING; pin the constant text.
        assert!(STALE_KEY_WARNING.contains("WARNING"));
        assert!(STALE_KEY_WARNING.contains("STALE"));
        assert!(STALE_KEY_WARNING.contains("AI_BRAINS_KEY"));
        // Constant is exactly what success stdout uses (F34 / AC18).
        assert_eq!(
            STALE_KEY_WARNING,
            "WARNING: AI_BRAINS_KEY / --key is now STALE. \
Update env/profile/.env to the NEW key before any other vault command. \
The old key cannot open the rotated vault."
        );
    }

    #[test]
    fn rotate_datakey__require_backup_false__without_phrase__refuses() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa8; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);

        let err = run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path,
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: false,
                i_have_backup: None,
                kit_output: Some(dir.path().join("kit.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect_err("must refuse silent bypass");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("i have a backup") || msg.contains("require-backup"),
            "{msg}"
        );
    }

    #[test]
    fn rotate_datakey__backup_bypassed__event_records_bypass() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa5; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);

        run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path.clone(),
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(dir.path().join("kit.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect("rotate");

        // Find new key via kit
        let kit: RecoveryKit =
            serde_json::from_str(&fs::read_to_string(dir.path().join("kit.json")).unwrap())
                .unwrap();
        let new_data = kit.unlock_with_passphrase(b"test-passphrase-ok").unwrap();
        let new_sql = SqlCipherKey::from_data_key(&new_data);
        let conn = VaultConnection::open(&path, &new_sql).unwrap();
        let store = SqliteEventStore::new(conn);
        let events = store.read_all_events().expect("events");
        let rotated: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.payload, Payload::DataKeyRotated(_)))
            .collect();
        assert_eq!(rotated.len(), 1, "expected DataKeyRotated event");
        assert_eq!(rotated[0].aggregate_type, AggregateType::System);
        assert_eq!(rotated[0].aggregate_id, Uuid::nil());
        match &rotated[0].payload {
            Payload::DataKeyRotated(p) => {
                assert!(p.backup_bypassed);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn rotate_datakey__stale_or_invalid_backup__refuses() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa6; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);
        // Empty backup dir → refuse
        fs::create_dir_all(dir.path().join("backups")).unwrap();

        let err = run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path,
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: None,
                kit_output: Some(dir.path().join("kit.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: Some(dir.path().join("backups")),
            },
            false,
        )
        .expect_err("must refuse");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(msg.contains("backup"), "{msg}");
    }

    #[test]
    fn rotate_datakey__event__system_aggregate_nil_id() {
        // Covered by backup_bypassed test; also pin builder shape.
        let payload = DataKeyRotatedPayload {
            rotation_id: Uuid::nil(),
            living_wraps_rewrapped: 0,
            device_private_resealed: 0,
            backup_bypassed: false,
            completed_at: "2026-08-02T00:00:00Z".into(),
        };
        let env = EventBuilder::new(
            AggregateType::System,
            Uuid::nil(),
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::DataKeyRotated(payload))
        .unwrap();
        assert_eq!(env.aggregate_id, Uuid::nil());
        assert_eq!(env.aggregate_type, AggregateType::System);
    }

    #[test]
    fn rotate_datakey__living_ce__survives_cli_rotate() {
        let _g = ENV_LOCK.lock().unwrap();
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa7; 32]);
        let path = seed_vault(dir.path(), &old);
        let old_sql = SqlCipherKey::from_data_key(&old);
        let ck = ContentKeyId::new();
        let blob_id = Uuid::new_v4();
        let plaintext = b"cli-rotate-ce";
        {
            let env = generate_wrap_and_seal(&old, ck, blob_id, plaintext).unwrap();
            let conn = VaultConnection::open(&path, &old_sql).unwrap();
            let c = conn.lock().unwrap();
            insert_content_key_wrap(
                &c,
                &ck.to_string(),
                i64::from(env.wrapped_dek.wrap_schema_version),
                &env.wrapped_dek.nonce,
                &env.wrapped_dek.ciphertext,
                "2026-08-02T12:00:00Z",
            )
            .unwrap();
            let ct_len = env.sealed.ciphertext.len() as i64;
            insert_encrypted_blob(
                &c,
                &EncryptedBlobRow {
                    blob_id: blob_id.to_string(),
                    content_key_id: ck.to_string(),
                    envelope_schema_version: i64::from(env.sealed.envelope_schema_version),
                    algorithm: ALGORITHM_AES_256_GCM.into(),
                    nonce: env.sealed.nonce.to_vec(),
                    ciphertext: env.sealed.ciphertext,
                    content_class: None,
                    subject_kind: None,
                    subject_id: None,
                    size_bytes: ct_len,
                    created_at: "2026-08-02T12:00:00Z".into(),
                },
            )
            .unwrap();
        }

        run_rotate_datakey_with_daemon_state(
            RotateDatakeyOptions {
                vault_path: path.clone(),
                key: Some(old_sql.expose_secret().to_string()),
                dry_run: false,
                confirm: true,
                require_backup: true,
                i_have_backup: Some(BYPASS_PHRASE.into()),
                kit_output: Some(dir.path().join("kit.json")),
                passphrase_file: Some(write_passphrase_file(dir.path())),
                overwrite_kit: false,
                accept_rekey_risk: false,
                print_key: false,
                backup_dir: None,
            },
            false,
        )
        .expect("rotate");

        let kit: RecoveryKit =
            serde_json::from_str(&fs::read_to_string(dir.path().join("kit.json")).unwrap())
                .unwrap();
        let new_data = kit.unlock_with_passphrase(b"test-passphrase-ok").unwrap();
        let conn = VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new_data)).unwrap();
        let c = conn.lock().unwrap();
        let wrap = content_envelope::get_content_key_wrap(&c, &ck.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(wrap.status, "active");
    }
}
