//! Vault DataKey rotation (T189 / ADR-0020).
//!
//! **Primary path:** encrypted→encrypted `sqlcipher_export` under the new key,
//! re-wrap active content DEKs + re-seal local device private on the **new** DB
//! only, verify, then atomic replace. Old vault stays openable with the old key
//! until replace succeeds (F5 / F7).
//!
//! **Opt-in path:** `PRAGMA rekey` via `execute_batch` only, behind
//! `accept_rekey_risk`, with pre-rotate snapshot auto-restore (F5b / F7b).

use crate::errors::{Result, StoreError};
use crate::pragmas::{apply_key_pragmas, apply_pragmas};
use crate::projections::content_envelope::{
    self, ContentKeyWrapRow, list_active_content_key_wraps, list_blobs_for_content_key,
    update_content_key_wrap,
};
use crate::projections::replication::{
    self, DevicePrivateKeyRow, list_device_private_key_wraps, put_device_private_key_wrap,
};
use ai_brains_core::ids::{ContentKeyId, DeviceId};
use ai_brains_crypto::content_key_store::{
    WrappedContentDek, parse_nonce, rotate_content_dek_wrap, unwrap_content_dek,
};
use ai_brains_crypto::{DataKey, SealedContent, SqlCipherKey, unwrap_and_open};
use ai_brains_sync::{SealedDevicePrivate, open_device_private_blob, seal_device_private_blob};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

/// Page-key change method used for this rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateMethod {
    /// Crash-safe `sqlcipher_export` (default / primary).
    Export,
    /// Opt-in in-place `PRAGMA rekey` (`--accept-rekey-risk`).
    Rekey,
}

impl RotateMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            RotateMethod::Export => "export",
            RotateMethod::Rekey => "rekey",
        }
    }
}

/// Dry-run plan: counts only; zero wraps is valid (F29).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotateDryRunPlan {
    pub living_wrap_count: u64,
    /// 0 or 1 expected; higher is reported honestly.
    pub device_private_count: u32,
}

/// Result of a successful rotation (library layer; kit/event are CLI).
#[derive(Debug, Clone)]
pub struct RotateDataKeyResult {
    pub living_wraps_rewrapped: u64,
    pub device_private_resealed: u32,
    pub method: RotateMethod,
    pub new_sqlcipher_key: SqlCipherKey,
}

/// Options for [`rotate_datakey`].
#[derive(Debug, Clone)]
pub struct RotateDataKeyOptions {
    pub vault_path: PathBuf,
    /// When true, use in-place rekey protocol (F5b). Default false = export.
    pub accept_rekey_risk: bool,
}

/// Plan-only: open with old key, count active wraps + device private rows.
pub fn plan_rotate_datakey(vault_path: &Path, old_key: &SqlCipherKey) -> Result<RotateDryRunPlan> {
    let conn = Connection::open(vault_path)
        .map_err(|e| StoreError::ConnectionFailed(format!("open vault for rotate plan: {e}")))?;
    apply_key_pragmas(&conn, old_key)?;
    verify_open(&conn)?;
    let wraps = list_active_content_key_wraps(&conn)?;
    let devices = list_device_private_key_wraps(&conn)?;
    Ok(RotateDryRunPlan {
        living_wrap_count: wraps.len() as u64,
        device_private_count: devices.len() as u32,
    })
}

/// Rotate vault DataKey + page key.
///
/// `new_data_key` is supplied by the caller (tests: `DataKey::from_bytes`;
/// production CLI: `DataKey::generate()`).
pub fn rotate_datakey(
    opts: &RotateDataKeyOptions,
    old_sql_key: &SqlCipherKey,
    old_data_key: &DataKey,
    new_data_key: &DataKey,
) -> Result<RotateDataKeyResult> {
    let new_sql_key = SqlCipherKey::from_data_key(new_data_key);
    if opts.accept_rekey_risk {
        rotate_datakey_rekey(
            &opts.vault_path,
            old_sql_key,
            old_data_key,
            new_data_key,
            &new_sql_key,
        )
    } else {
        rotate_datakey_export(
            &opts.vault_path,
            old_sql_key,
            old_data_key,
            new_data_key,
            &new_sql_key,
        )
    }
}

// ---------------------------------------------------------------------------
// Primary: export
// ---------------------------------------------------------------------------

fn rotate_datakey_export(
    vault_path: &Path,
    old_sql_key: &SqlCipherKey,
    old_data_key: &DataKey,
    new_data_key: &DataKey,
    new_sql_key: &SqlCipherKey,
) -> Result<RotateDataKeyResult> {
    if !vault_path.exists() {
        return Err(StoreError::ConnectionFailed(format!(
            "vault does not exist: {}",
            vault_path.display()
        )));
    }

    let dest = sibling_temp_path(vault_path, "rotate-export.tmp")?;
    if dest.exists() {
        fs::remove_file(&dest).map_err(|e| {
            StoreError::ConnectionFailed(format!(
                "could not remove stale rotate temp {}: {e}",
                dest.display()
            ))
        })?;
    }

    // 1) Export pages under new key (application wraps still under old DataKey).
    if let Err(e) = sqlcipher_export_encrypted(vault_path, old_sql_key, &dest, new_sql_key) {
        remove_db_and_sidecars(&dest);
        return Err(e);
    }

    // 2) Re-wrap + re-seal on the NEW DB only.
    let apply_result = (|| {
        let new_conn = Connection::open(&dest)
            .map_err(|e| StoreError::ConnectionFailed(format!("open exported rotate dest: {e}")))?;
        apply_pragmas(&new_conn, new_sql_key)?;
        verify_open(&new_conn)?;

        let wraps = list_active_content_key_wraps(&new_conn)?;
        let living = rewrap_active_wraps(&new_conn, old_data_key, new_data_key, &wraps)?;
        let resealed = reseal_device_private(&new_conn, old_data_key, new_data_key)?;

        // 3) Verify under new key before replace.
        verify_rotation(&new_conn, new_data_key, living)?;
        // Drop connection before replace.
        drop(new_conn);

        Ok::<RotateDataKeyResult, StoreError>(RotateDataKeyResult {
            living_wraps_rewrapped: living,
            device_private_resealed: resealed,
            method: RotateMethod::Export,
            new_sqlcipher_key: new_sql_key.clone(),
        })
    })();

    let result = match apply_result {
        Ok(r) => r,
        Err(e) => {
            remove_db_and_sidecars(&dest);
            return Err(e);
        }
    };

    // 4) Atomic replace: on failure, leave dest for recovery but old vault intact.
    if let Err(e) = atomic_replace_file(&dest, vault_path) {
        remove_db_and_sidecars(&dest);
        return Err(e);
    }

    // 5) Post-replace verify with new key.
    {
        let verify = Connection::open(vault_path)
            .map_err(|e| StoreError::ConnectionFailed(format!("post-replace open failed: {e}")))?;
        apply_key_pragmas(&verify, new_sql_key)?;
        verify_open(&verify)?;
    }

    Ok(result)
}

fn sqlcipher_export_encrypted(
    source: &Path,
    old_key: &SqlCipherKey,
    dest: &Path,
    new_key: &SqlCipherKey,
) -> Result<()> {
    let src = Connection::open(source).map_err(|e| {
        StoreError::ConnectionFailed(format!("open source for sqlcipher_export: {e}"))
    })?;
    apply_pragmas(&src, old_key)?;
    verify_open(&src)?;
    let _ = src.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));

    let dest_sql = sql_quote_path(dest);
    let key_material = new_key.expose_secret();
    let attach = format!("ATTACH DATABASE {dest_sql} AS rotated KEY \"{key_material}\";");
    src.execute_batch(&attach).map_err(|e| {
        StoreError::ConnectionFailed(format!(
            "ATTACH rotated for sqlcipher_export failed ({})",
            redact_sql_err(e)
        ))
    })?;
    src.execute_batch("SELECT sqlcipher_export('rotated');")
        .map_err(|e| {
            StoreError::ConnectionFailed(format!("sqlcipher_export failed ({})", redact_sql_err(e)))
        })?;
    src.execute_batch("DETACH DATABASE rotated;").map_err(|e| {
        StoreError::ConnectionFailed(format!("DETACH rotated failed ({})", redact_sql_err(e)))
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Opt-in rekey
// ---------------------------------------------------------------------------

fn rotate_datakey_rekey(
    vault_path: &Path,
    old_sql_key: &SqlCipherKey,
    old_data_key: &DataKey,
    new_data_key: &DataKey,
    new_sql_key: &SqlCipherKey,
) -> Result<RotateDataKeyResult> {
    if !vault_path.exists() {
        return Err(StoreError::ConnectionFailed(format!(
            "vault does not exist: {}",
            vault_path.display()
        )));
    }

    let snapshot = vault_path.with_extension("db.pre-rotate.bak");
    // Prefer unique if exists
    let snapshot = if snapshot.exists() {
        vault_path.with_extension(format!(
            "db.pre-rotate.bak-{}",
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        ))
    } else {
        snapshot
    };

    fs::copy(vault_path, &snapshot).map_err(|e| {
        StoreError::ConnectionFailed(format!(
            "pre-rotate snapshot failed ({}): {e}",
            snapshot.display()
        ))
    })?;
    // Best-effort copy of WAL companions if present
    let wal = sidecar(vault_path, "-wal");
    if wal.exists() {
        let _ = fs::copy(&wal, sidecar(&snapshot, "-wal"));
    }
    let shm = sidecar(vault_path, "-shm");
    if shm.exists() {
        let _ = fs::copy(&shm, sidecar(&snapshot, "-shm"));
    }

    let run: Result<RotateDataKeyResult> = (|| {
        let conn = Connection::open(vault_path)
            .map_err(|e| StoreError::ConnectionFailed(format!("open vault for rekey: {e}")))?;
        apply_pragmas(&conn, old_sql_key)?;
        verify_open(&conn)?;

        // Wrap updates under old page key.
        let tx = conn
            .unchecked_transaction()
            .map_err(StoreError::DatabaseError)?;
        let wraps = list_active_content_key_wraps(&tx)?;
        let living = rewrap_active_wraps(&tx, old_data_key, new_data_key, &wraps)?;
        let resealed = reseal_device_private(&tx, old_data_key, new_data_key)?;
        tx.commit().map_err(StoreError::DatabaseError)?;

        // F5b journal protocol
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        conn.execute_batch("PRAGMA journal_mode = DELETE;")
            .map_err(|e| {
                StoreError::ConnectionFailed(format!(
                    "journal_mode=DELETE before rekey ({})",
                    redact_sql_err(e)
                ))
            })?;
        let rekey_sql = format!("PRAGMA rekey = \"{}\";", new_sql_key.expose_secret());
        conn.execute_batch(&rekey_sql).map_err(|e| {
            StoreError::ConnectionFailed(format!("PRAGMA rekey failed ({})", redact_sql_err(e)))
        })?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| {
                StoreError::ConnectionFailed(format!(
                    "journal_mode=WAL after rekey ({})",
                    redact_sql_err(e)
                ))
            })?;
        conn.execute_batch("PRAGMA synchronous = NORMAL; PRAGMA busy_timeout = 5000;")
            .map_err(|e| {
                StoreError::ConnectionFailed(format!(
                    "restore pragmas after rekey ({})",
                    redact_sql_err(e)
                ))
            })?;

        verify_open(&conn)?;
        verify_rotation(&conn, new_data_key, living)?;

        Ok(RotateDataKeyResult {
            living_wraps_rewrapped: living,
            device_private_resealed: resealed,
            method: RotateMethod::Rekey,
            new_sqlcipher_key: new_sql_key.clone(),
        })
    })();

    match run {
        Ok(r) => {
            // Success: leave snapshot for operator; do not auto-delete.
            Ok(r)
        }
        Err(e) => {
            // Auto-restore snapshot (F7b).
            if let Err(restore_err) = restore_snapshot(&snapshot, vault_path) {
                return Err(StoreError::ConnectionFailed(format!(
                    "rekey failed ({e}); snapshot restore also failed: {restore_err}"
                )));
            }
            Err(StoreError::ConnectionFailed(format!(
                "rekey path failed (vault restored from pre-rotate snapshot): {e}"
            )))
        }
    }
}

fn restore_snapshot(snapshot: &Path, vault_path: &Path) -> Result<()> {
    // Remove live vault + sidecars then copy snapshot back.
    remove_db_and_sidecars(vault_path);
    fs::copy(snapshot, vault_path).map_err(|e| {
        StoreError::ConnectionFailed(format!(
            "copy snapshot {} → {}: {e}",
            snapshot.display(),
            vault_path.display()
        ))
    })?;
    let snap_wal = sidecar(snapshot, "-wal");
    if snap_wal.exists() {
        let _ = fs::copy(&snap_wal, sidecar(vault_path, "-wal"));
    }
    let snap_shm = sidecar(snapshot, "-shm");
    if snap_shm.exists() {
        let _ = fs::copy(&snap_shm, sidecar(vault_path, "-shm"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared re-wrap / re-seal / verify
// ---------------------------------------------------------------------------

fn rewrap_active_wraps(
    conn: &Connection,
    old_key: &DataKey,
    new_key: &DataKey,
    wraps: &[ContentKeyWrapRow],
) -> Result<u64> {
    let mut count = 0u64;
    for row in wraps {
        if row.status != "active" {
            continue;
        }
        let (nonce_bytes, ct) = match (&row.wrap_nonce, &row.wrap_ciphertext) {
            (Some(n), Some(c)) if !n.is_empty() && !c.is_empty() => (n.as_slice(), c.as_slice()),
            _ => {
                return Err(StoreError::ConfigError(format!(
                    "active wrap missing material: {}",
                    row.content_key_id
                )));
            }
        };
        let nonce = parse_nonce(nonce_bytes).map_err(|e| {
            StoreError::ConfigError(format!(
                "invalid wrap nonce for {}: {e}",
                row.content_key_id
            ))
        })?;
        let wrapped = WrappedContentDek {
            wrap_schema_version: row.wrap_schema_version as u32,
            nonce,
            ciphertext: ct.to_vec(),
        };
        let content_key_id = parse_content_key_id(&row.content_key_id)?;
        let rotated = rotate_content_dek_wrap(old_key, new_key, &content_key_id, &wrapped)
            .map_err(|e| {
                StoreError::ConfigError(format!("re-wrap failed for {}: {e}", row.content_key_id))
            })?;
        update_content_key_wrap(
            conn,
            &row.content_key_id,
            &rotated.nonce,
            &rotated.ciphertext,
        )?;
        count = count.saturating_add(1);
    }
    Ok(count)
}

fn reseal_device_private(conn: &Connection, old_key: &DataKey, new_key: &DataKey) -> Result<u32> {
    let rows = list_device_private_key_wraps(conn)?;
    let mut count = 0u32;
    for row in rows {
        let device_id = parse_device_id(&row.device_id)?;
        let sealed = row_to_sealed(&row)?;
        let seeds = open_device_private_blob(old_key, &sealed, &device_id).map_err(|e| {
            StoreError::ConfigError(format!("open device private for {}: {e}", row.device_id))
        })?;
        let resealed = seal_device_private_blob(new_key, &seeds, &device_id).map_err(|e| {
            StoreError::ConfigError(format!("reseal device private for {}: {e}", row.device_id))
        })?;
        let updated = DevicePrivateKeyRow {
            device_id: row.device_id.clone(),
            wrap_schema_version: resealed.wrap_schema_version as i64,
            algorithm: row.algorithm.clone(),
            protection: resealed.protection,
            wrap_nonce: resealed.wrap_nonce.to_vec(),
            wrap_ciphertext: resealed.wrap_ciphertext,
            created_at: row.created_at.clone(),
        };
        put_device_private_key_wrap(conn, &updated)?;
        count = count.saturating_add(1);
    }
    Ok(count)
}

fn verify_rotation(conn: &Connection, new_key: &DataKey, expected_living: u64) -> Result<()> {
    verify_open(conn)?;
    let wraps = list_active_content_key_wraps(conn)?;
    if wraps.len() as u64 != expected_living {
        return Err(StoreError::ConfigError(format!(
            "post-rotate active wrap count {} != expected {expected_living}",
            wraps.len()
        )));
    }
    for row in &wraps {
        let (nonce_bytes, ct) = match (&row.wrap_nonce, &row.wrap_ciphertext) {
            (Some(n), Some(c)) if !n.is_empty() && !c.is_empty() => (n.as_slice(), c.as_slice()),
            _ => {
                return Err(StoreError::ConfigError(format!(
                    "verify: active wrap missing material: {}",
                    row.content_key_id
                )));
            }
        };
        let nonce = parse_nonce(nonce_bytes).map_err(|e| {
            StoreError::ConfigError(format!("verify nonce {}: {e}", row.content_key_id))
        })?;
        let wrapped = WrappedContentDek {
            wrap_schema_version: row.wrap_schema_version as u32,
            nonce,
            ciphertext: ct.to_vec(),
        };
        let id = parse_content_key_id(&row.content_key_id)?;
        // DEK unwrap under new DataKey (always).
        unwrap_content_dek(new_key, &wrapped, &id).map_err(|e| {
            StoreError::ConfigError(format!(
                "verify unwrap failed for {}: {e}",
                row.content_key_id
            ))
        })?;

        // F6v: open corresponding encrypted_content_blob sample when present.
        // Zero-blob keys: DEK unwrap alone is sufficient.
        let blobs = list_blobs_for_content_key(conn, &row.content_key_id)?;
        for blob in blobs {
            let blob_id = uuid::Uuid::parse_str(&blob.blob_id).map_err(|e| {
                StoreError::ConfigError(format!(
                    "verify: invalid blob_id {} for {}: {e}",
                    blob.blob_id, row.content_key_id
                ))
            })?;
            let sealed_nonce = parse_nonce(&blob.nonce).map_err(|e| {
                StoreError::ConfigError(format!(
                    "verify: invalid blob nonce {} for {}: {e}",
                    blob.blob_id, row.content_key_id
                ))
            })?;
            let sealed = SealedContent {
                envelope_schema_version: blob.envelope_schema_version as u32,
                nonce: sealed_nonce,
                ciphertext: blob.ciphertext,
            };
            unwrap_and_open(new_key, &id, &wrapped, &sealed, blob_id).map_err(|e| {
                StoreError::ConfigError(format!(
                    "verify: encrypted_content_blob open failed for {} blob {}: {e}",
                    row.content_key_id, blob.blob_id
                ))
            })?;
        }
    }
    Ok(())
}

/// Redact SQLCipher raw-key material (`x'…64 hex…'`) from rusqlite / PRAGMA error strings.
///
/// Keyed statements (ATTACH KEY / PRAGMA rekey) may echo the key in error variants; never
/// forward raw `{e}` to operators or logs without this pass (F26).
pub(crate) fn redact_sql_err(e: impl std::fmt::Display) -> String {
    redact_key_hex_in_sql(&e.to_string())
}

/// Replace `x'<64 ascii hex digits>'` (prefix case-insensitive) with `x'[REDACTED]'`.
fn redact_key_hex_in_sql(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        let redacted_len = rest
            .get(..2)
            .filter(|p| p.eq_ignore_ascii_case("x'"))
            .and_then(|_| {
                let after = &rest[2..];
                let end = after.find('\'')?;
                let hex = &after[..end];
                if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                    Some(2 + end + 1)
                } else {
                    None
                }
            });
        if let Some(len) = redacted_len {
            out.push_str("x'[REDACTED]'");
            i += len;
        } else {
            let Some(ch) = rest.chars().next() else {
                break;
            };
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn verify_open(conn: &Connection) -> Result<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| StoreError::VaultLocked(format!("key verification failed: {e}")))?;
    Ok(())
}

fn parse_content_key_id(s: &str) -> Result<ContentKeyId> {
    let u = uuid::Uuid::parse_str(s)
        .map_err(|e| StoreError::ConfigError(format!("invalid content_key_id {s}: {e}")))?;
    Ok(ContentKeyId::from_uuid(u))
}

fn parse_device_id(s: &str) -> Result<DeviceId> {
    let u = uuid::Uuid::parse_str(s)
        .map_err(|e| StoreError::ConfigError(format!("invalid device_id {s}: {e}")))?;
    Ok(DeviceId::from_uuid(u))
}

fn row_to_sealed(row: &DevicePrivateKeyRow) -> Result<SealedDevicePrivate> {
    let nonce: [u8; 12] = row.wrap_nonce.as_slice().try_into().map_err(|_| {
        StoreError::ConfigError(format!(
            "device private wrap_nonce must be 12 bytes for {}",
            row.device_id
        ))
    })?;
    Ok(SealedDevicePrivate {
        wrap_schema_version: row.wrap_schema_version as u32,
        protection: row.protection.clone(),
        wrap_nonce: nonce,
        wrap_ciphertext: row.wrap_ciphertext.clone(),
    })
}

fn sibling_temp_path(vault: &Path, suffix: &str) -> Result<PathBuf> {
    let parent = vault.parent().unwrap_or_else(|| Path::new("."));
    let stem = vault
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "vault.db".into());
    Ok(parent.join(format!("{stem}.{suffix}")))
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

/// Best-effort remove main DB file and its `-wal` / `-shm` companions.
fn remove_db_and_sidecars(path: &Path) {
    let _ = fs::remove_file(path);
    clear_wal_shm(path);
}

/// Best-effort clear SQLite WAL/SHM sidecars next to `path` (after replace or abandon).
fn clear_wal_shm(path: &Path) {
    let _ = fs::remove_file(sidecar(path, "-wal"));
    let _ = fs::remove_file(sidecar(path, "-shm"));
}

fn sql_quote_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("'{}'", s.replace('\'', "''"))
}

/// Atomically replace `target` with `source` (source is the new vault file).
///
/// After a successful replace, clears `target-wal` / `target-shm` so stale WAL
/// pages under the old page codec cannot sit beside the new main DB (P1-3 / F5).
pub fn atomic_replace_file(source: &Path, target: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        if let Err(e) = windows_replace_file(source, target) {
            // Fallback: rename target aside then rename source → target.
            let bak = target.with_extension("db.rotate-old");
            if bak.exists() {
                let _ = fs::remove_file(&bak);
            }
            if target.exists() {
                fs::rename(target, &bak).map_err(|re| {
                    StoreError::ConnectionFailed(format!(
                        "atomic replace fallback: move old aside failed ({e}; {re})"
                    ))
                })?;
            }
            if let Err(re) = fs::rename(source, target) {
                // Try restore old
                if bak.exists() {
                    let _ = fs::rename(&bak, target);
                }
                return Err(StoreError::ConnectionFailed(format!(
                    "atomic replace fallback rename failed: {re}"
                )));
            }
            let _ = fs::remove_file(&bak);
            // Stale WAL/SHM under old codec must not remain next to new main DB.
            clear_wal_shm(target);
            return Ok(());
        }
        clear_wal_shm(target);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        fs::rename(source, target).map_err(|e| {
            StoreError::ConnectionFailed(format!(
                "atomic replace rename {} → {}: {e}",
                source.display(),
                target.display()
            ))
        })?;
        clear_wal_shm(target);
        Ok(())
    }
}

#[cfg(windows)]
fn windows_replace_file(source: &Path, target: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    // MOVEFILE_REPLACE_EXISTING = 0x1; MOVEFILE_WRITE_THROUGH = 0x8
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
    let from = wide(source);
    let to = wide(target);
    // extern kernel32 MoveFileExW
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            lp_existing_file_name: *const u16,
            lp_new_file_name: *const u16,
            dw_flags: u32,
        ) -> i32;
    }
    let ok = unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        let err = std::io::Error::last_os_error();
        return Err(StoreError::ConnectionFailed(format!(
            "MoveFileEx REPLACE_EXISTING failed: {err}"
        )));
    }
    Ok(())
}

// Re-export helpers used by CLI tests if needed
pub use content_envelope::list_active_content_key_wraps as list_active_wraps_for_tests;
#[allow(unused_imports)]
pub use replication::list_device_private_key_wraps as list_device_private_for_tests;

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::connection::VaultConnection;
    use crate::projections::content_envelope::{
        self, ALGORITHM_AES_256_GCM, EncryptedBlobRow, destroy_content_key_wrap,
        insert_content_key_wrap, insert_encrypted_blob,
    };
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::content_envelope::generate_wrap_and_seal;
    use ai_brains_crypto::content_key_store::{ContentDek, WRAP_SCHEMA_VERSION, wrap_content_dek};
    use ai_brains_sync::{generate_device_keys, seal_device_private_blob};
    use tempfile::tempdir;
    use uuid::Uuid;

    const CREATED_AT: &str = "2026-08-02T12:00:00Z";

    fn allow_zero() -> TempEnv {
        // Not always zero; allow for safety in mixed suites.
        TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1")
    }

    fn seed_vault(dir: &Path, data_key: &DataKey) -> PathBuf {
        let path = dir.join("vault.db");
        let sql = SqlCipherKey::from_data_key(data_key);
        let conn = VaultConnection::open(&path, &sql).expect("open");
        conn.migrate().expect("migrate");
        path
    }

    fn insert_active_ce(path: &Path, data_key: &DataKey, plaintext: &[u8]) -> (ContentKeyId, Uuid) {
        let sql = SqlCipherKey::from_data_key(data_key);
        let conn = VaultConnection::open(path, &sql).expect("open");
        let content_key_id = ContentKeyId::new();
        let blob_id = Uuid::new_v4();
        let env =
            generate_wrap_and_seal(data_key, content_key_id, blob_id, plaintext).expect("seal");
        {
            let c = conn.lock().expect("lock");
            insert_content_key_wrap(
                &c,
                &content_key_id.to_string(),
                i64::from(env.wrapped_dek.wrap_schema_version),
                &env.wrapped_dek.nonce,
                &env.wrapped_dek.ciphertext,
                CREATED_AT,
            )
            .expect("insert wrap");
            insert_encrypted_blob(
                &c,
                &EncryptedBlobRow {
                    blob_id: blob_id.to_string(),
                    content_key_id: content_key_id.to_string(),
                    envelope_schema_version: i64::from(env.sealed.envelope_schema_version),
                    algorithm: ALGORITHM_AES_256_GCM.to_string(),
                    nonce: env.sealed.nonce.to_vec(),
                    ciphertext: env.sealed.ciphertext.clone(),
                    content_class: None,
                    subject_kind: None,
                    subject_id: None,
                    size_bytes: env.sealed.ciphertext.len() as i64,
                    created_at: CREATED_AT.to_string(),
                },
            )
            .expect("insert blob");
        }
        (content_key_id, blob_id)
    }

    #[test]
    fn rotate_datakey__old_sqlcipher_key__fails_closed() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x11; 32]);
        let new = DataKey::from_bytes([0x22; 32]);
        let path = seed_vault(dir.path(), &old);
        let _ = insert_active_ce(&path, &old, b"hello-rotate");

        let result = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate");
        assert_eq!(result.method, RotateMethod::Export);
        assert_eq!(result.living_wraps_rewrapped, 1);

        // Old page key fails closed.
        let old_open = VaultConnection::open(&path, &SqlCipherKey::from_data_key(&old));
        assert!(old_open.is_err(), "old key must fail closed after rotate");

        // New key opens.
        let new_open =
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("new open");
        let c = new_open.lock().unwrap();
        let n: i64 = c
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))
            .unwrap();
        assert!(n > 0);
    }

    #[test]
    fn rotate_datakey__living_ce__open_after_rotate() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x31; 32]);
        let new = DataKey::from_bytes([0x32; 32]);
        let path = seed_vault(dir.path(), &old);
        let plaintext = b"living content after rotate";
        let (ck, blob_id) = insert_active_ce(&path, &old, plaintext);

        rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate");

        let conn =
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("open new");
        let c = conn.lock().unwrap();
        let wrap = content_envelope::get_content_key_wrap(&c, &ck.to_string())
            .unwrap()
            .expect("wrap");
        assert_eq!(wrap.status, "active");
        let nonce = parse_nonce(wrap.wrap_nonce.as_ref().unwrap()).unwrap();
        let wrapped = WrappedContentDek {
            wrap_schema_version: wrap.wrap_schema_version as u32,
            nonce,
            ciphertext: wrap.wrap_ciphertext.unwrap(),
        };
        let blob = content_envelope::get_encrypted_blob(&c, &blob_id.to_string())
            .unwrap()
            .expect("blob");
        let sealed = ai_brains_crypto::SealedContent {
            envelope_schema_version: blob.envelope_schema_version as u32,
            nonce: parse_nonce(&blob.nonce).unwrap(),
            ciphertext: blob.ciphertext,
        };
        let opened =
            ai_brains_crypto::unwrap_and_open(&new, &ck, &wrapped, &sealed, blob_id).unwrap();
        assert_eq!(opened.as_slice(), plaintext);

        // Destroyed not applicable here; insert + destroy path:
        drop(c);
        drop(conn);
    }

    #[test]
    fn rotate_datakey__zero_active_wraps__succeeds() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x41; 32]);
        let new = DataKey::from_bytes([0x42; 32]);
        let path = seed_vault(dir.path(), &old);

        let plan = plan_rotate_datakey(&path, &SqlCipherKey::from_data_key(&old)).unwrap();
        assert_eq!(plan.living_wrap_count, 0);
        assert_eq!(plan.device_private_count, 0);

        let r = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate zero wraps");
        assert_eq!(r.living_wraps_rewrapped, 0);
        assert_eq!(r.device_private_resealed, 0);
        VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("new key");
    }

    #[test]
    fn rotate_datakey__no_device_enrolled__succeeds_no_reseal() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x51; 32]);
        let new = DataKey::from_bytes([0x52; 32]);
        let path = seed_vault(dir.path(), &old);
        let _ = insert_active_ce(&path, &old, b"no-device");

        let r = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate");
        assert_eq!(r.device_private_resealed, 0);
        assert_eq!(r.living_wraps_rewrapped, 1);
    }

    #[test]
    fn rotate_datakey__device_private__reseal_roundtrip() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x61; 32]);
        let new = DataKey::from_bytes([0x62; 32]);
        let path = seed_vault(dir.path(), &old);

        let keys = generate_device_keys().expect("device keys");
        let device_id = DeviceId::new();
        let seeds = ai_brains_sync::DevicePrivateSeeds::from_key_pair(&keys);
        let sealed = seal_device_private_blob(&old, &seeds, &device_id).expect("seal");
        {
            let sql = SqlCipherKey::from_data_key(&old);
            let conn = VaultConnection::open(&path, &sql).unwrap();
            let c = conn.lock().unwrap();
            // FK: device_private_key_store → device_identity
            let fp = [0u8; 32];
            c.execute(
                "INSERT INTO device_identity (
                    device_id, schema_version, ed25519_public, x25519_public,
                    display_name, status, enrolled_at, revoked_at,
                    enrolled_by_device_id, fingerprint_sha256
                 ) VALUES (?1, 1, ?2, ?3, 'local', 'local', ?4, NULL, ?1, ?5)",
                rusqlite::params![
                    device_id.to_string(),
                    keys.verifying_key().to_bytes().as_slice(),
                    keys.x25519_public().to_bytes().as_slice(),
                    CREATED_AT,
                    fp.as_slice(),
                ],
            )
            .expect("insert device_identity");
            put_device_private_key_wrap(
                &c,
                &DevicePrivateKeyRow {
                    device_id: device_id.to_string(),
                    wrap_schema_version: sealed.wrap_schema_version as i64,
                    algorithm: "AES-256-GCM".into(),
                    protection: sealed.protection.clone(),
                    wrap_nonce: sealed.wrap_nonce.to_vec(),
                    wrap_ciphertext: sealed.wrap_ciphertext.clone(),
                    created_at: CREATED_AT.into(),
                },
            )
            .expect("put private");
        }

        let r = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate");
        assert_eq!(r.device_private_resealed, 1);

        let conn = VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("open");
        let c = conn.lock().unwrap();
        let row = list_device_private_key_wraps(&c).unwrap();
        assert_eq!(row.len(), 1);
        let sealed2 = row_to_sealed(&row[0]).unwrap();
        // Old DataKey must not open.
        assert!(open_device_private_blob(&old, &sealed2, &device_id).is_err());
        let opened = open_device_private_blob(&new, &sealed2, &device_id).expect("open new");
        assert_eq!(opened.ed25519_seed, seeds.ed25519_seed);
        assert_eq!(opened.x25519_seed, seeds.x25519_seed);
    }

    #[test]
    fn rotate_wrap__destroyed_row__skipped_at_list() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x71; 32]);
        let path = seed_vault(dir.path(), &old);
        let (ck, _) = insert_active_ce(&path, &old, b"to-destroy");
        {
            let sql = SqlCipherKey::from_data_key(&old);
            let conn = VaultConnection::open(&path, &sql).unwrap();
            let c = conn.lock().unwrap();
            destroy_content_key_wrap(&c, &ck.to_string(), "2026-08-02T13:00:00Z").unwrap();
            // also insert another active
            let dek = ContentDek::generate().unwrap();
            let ck2 = ContentKeyId::new();
            let w = wrap_content_dek(&old, &dek, &ck2).unwrap();
            insert_content_key_wrap(
                &c,
                &ck2.to_string(),
                i64::from(WRAP_SCHEMA_VERSION),
                &w.nonce,
                &w.ciphertext,
                CREATED_AT,
            )
            .unwrap();
            let active = list_active_content_key_wraps(&c).unwrap();
            assert_eq!(active.len(), 1);
            assert_eq!(active[0].content_key_id, ck2.to_string());
        }
    }

    #[test]
    fn list_and_update_active_wrap__roundtrip() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let key = DataKey::from_bytes([0x81; 32]);
        let path = seed_vault(dir.path(), &key);
        let (ck, _) = insert_active_ce(&path, &key, b"upd");
        let sql = SqlCipherKey::from_data_key(&key);
        let conn = VaultConnection::open(&path, &sql).unwrap();
        let c = conn.lock().unwrap();
        let list = list_active_content_key_wraps(&c).unwrap();
        assert_eq!(list.len(), 1);
        let dek = ContentDek::from_bytes([0x99; 32]);
        let w = wrap_content_dek(&key, &dek, &ck).unwrap();
        update_content_key_wrap(&c, &ck.to_string(), &w.nonce, &w.ciphertext).unwrap();
        let row = content_envelope::get_content_key_wrap(&c, &ck.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(row.wrap_nonce.as_ref().unwrap(), &w.nonce);
    }

    #[test]
    fn redact_sql_err__strips_x_hex_key_material() {
        let key_hex = "a".repeat(64);
        let leaked = format!(
            "ATTACH DATABASE 'x' AS rotated KEY \"x'{key_hex}'\"; failed near x'{key_hex}'"
        );
        let redacted = redact_sql_err(&leaked);
        assert!(
            !redacted.contains(&key_hex),
            "hex body must not appear: {redacted}"
        );
        assert!(
            redacted.contains("x'[REDACTED]'"),
            "expected redaction marker: {redacted}"
        );
        // Non-key text preserved
        assert!(redacted.contains("ATTACH DATABASE"));
        // Short hex not rewritten
        let short = "x'abcd'";
        assert_eq!(redact_sql_err(short), short);
        // Uppercase X' prefix
        let upper = format!("X'{key_hex}'");
        assert_eq!(redact_sql_err(&upper), "x'[REDACTED]'");
    }

    #[test]
    fn rotate_datakey__peer_wrap__bytes_unchanged() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0x91; 32]);
        let new = DataKey::from_bytes([0x92; 32]);
        let path = seed_vault(dir.path(), &old);
        let _ = insert_active_ce(&path, &old, b"peer-immutable");

        let peer_row = replication::PeerContentKeyWrapRow {
            content_key_id: "peer-ck-fixed".into(),
            recipient_device_id: "peer-recipient-dev".into(),
            sender_device_id: "peer-sender-dev".into(),
            schema_version: 1,
            eph_x25519_public: vec![0xAB; 32],
            wrap_nonce: vec![0xCD; 12],
            wrap_ciphertext: vec![0xEF; 48],
            created_at: CREATED_AT.into(),
        };
        {
            let sql = SqlCipherKey::from_data_key(&old);
            let conn = VaultConnection::open(&path, &sql).unwrap();
            let c = conn.lock().unwrap();
            replication::upsert_peer_content_key_wrap(&c, &peer_row).unwrap();
        }

        rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: false,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rotate");

        let conn =
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("new open");
        let c = conn.lock().unwrap();
        let got = replication::get_peer_wrap(&c, "peer-ck-fixed", "peer-recipient-dev")
            .unwrap()
            .expect("peer wrap must remain");
        assert_eq!(
            got, peer_row,
            "peer wrap bytes must be unchanged after rotate"
        );
        let count: i64 = c
            .query_row("SELECT count(*) FROM peer_content_key_wrap", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn rotate_datakey__rekey_path__opens_with_new_key() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xa1; 32]);
        let new = DataKey::from_bytes([0xa2; 32]);
        let path = seed_vault(dir.path(), &old);
        let plaintext = b"rekey living wrap";
        let (ck, blob_id) = insert_active_ce(&path, &old, plaintext);

        let result = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: true,
            },
            &SqlCipherKey::from_data_key(&old),
            &old,
            &new,
        )
        .expect("rekey rotate");
        assert_eq!(result.method, RotateMethod::Rekey);
        assert_eq!(result.living_wraps_rewrapped, 1);

        assert!(
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&old)).is_err(),
            "old key must fail closed after rekey"
        );
        let conn =
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).expect("new opens");
        let c = conn.lock().unwrap();
        let wrap = content_envelope::get_content_key_wrap(&c, &ck.to_string())
            .unwrap()
            .expect("wrap");
        let nonce = parse_nonce(wrap.wrap_nonce.as_ref().unwrap()).unwrap();
        let wrapped = WrappedContentDek {
            wrap_schema_version: wrap.wrap_schema_version as u32,
            nonce,
            ciphertext: wrap.wrap_ciphertext.unwrap(),
        };
        let blob = content_envelope::get_encrypted_blob(&c, &blob_id.to_string())
            .unwrap()
            .expect("blob");
        let sealed = SealedContent {
            envelope_schema_version: blob.envelope_schema_version as u32,
            nonce: parse_nonce(&blob.nonce).unwrap(),
            ciphertext: blob.ciphertext,
        };
        let opened = unwrap_and_open(&new, &ck, &wrapped, &sealed, blob_id).unwrap();
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn rotate_datakey__rekey_path__failure_restores_snapshot() {
        let _allow = allow_zero();
        let dir = tempdir().unwrap();
        let old = DataKey::from_bytes([0xb1; 32]);
        let wrong_old = DataKey::from_bytes([0xb0; 32]);
        let new = DataKey::from_bytes([0xb2; 32]);
        let path = seed_vault(dir.path(), &old);
        let _ = insert_active_ce(&path, &old, b"rekey-fail-restore");

        let err = rotate_datakey(
            &RotateDataKeyOptions {
                vault_path: path.clone(),
                accept_rekey_risk: true,
            },
            &SqlCipherKey::from_data_key(&old),
            &wrong_old, // wrong DataKey → re-wrap fails after snapshot
            &new,
        )
        .expect_err("rekey must fail with wrong old DataKey");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("restored") || msg.contains("rekey") || msg.contains("re-wrap"),
            "unexpected error: {msg}"
        );

        // Snapshot restore: old key still opens living vault.
        let conn =
            VaultConnection::open(&path, &SqlCipherKey::from_data_key(&old)).expect("restored");
        let c = conn.lock().unwrap();
        let n: i64 = c
            .query_row("SELECT count(*) FROM content_key_store", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
        // New key must not open restored vault.
        assert!(VaultConnection::open(&path, &SqlCipherKey::from_data_key(&new)).is_err());
    }
}
