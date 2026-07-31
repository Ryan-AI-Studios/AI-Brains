//! `ai-brains device` — multi-device enrollment (T176 / ADR-0018).
//!
//! Honesty: optional multi-device; **not** post-quantum; **not** remote wipe;
//! **not** metadata-private. First-device uses `bootstrap`; peers use OOB
//! fingerprint + `enroll` (T177 delivers history via relay).

use crate::context::AppContext;
use ai_brains_core::ids::DeviceId;
use ai_brains_crypto::DataKey;
use ai_brains_store::projections::replication::{self, DeviceIdentityRow, DevicePrivateKeyRow};
use ai_brains_sync::{
    DevicePrivateSeeds, REPLICATION_SCHEMA_VERSION, enrollment_package, fingerprint_sha256,
    format_fingerprint_hyphen, generate_device_keys, parse_enrollment_package, private_blob,
};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Parse vault DataKey from SqlCipherKey material `x'<64 hex chars>'`.
pub fn data_key_from_sqlcipher(
    sql_key: &ai_brains_crypto::SqlCipherKey,
) -> Result<DataKey, String> {
    let raw = sql_key.expose_secret().trim();
    let hex_part = raw
        .strip_prefix("x'")
        .and_then(|s| s.strip_suffix('\''))
        .ok_or_else(|| {
            "SqlCipherKey material must be x'<64 hex chars>' for device key wrap".to_string()
        })?;
    if hex_part.len() != 64 {
        return Err(format!("DataKey hex length {} != 64", hex_part.len()));
    }
    let bytes = hex::decode(hex_part).map_err(|e| format!("invalid DataKey hex: {e}"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "DataKey must be 32 bytes".to_string())?;
    Ok(DataKey::from_bytes(arr))
}

fn now_rfc3339() -> Result<String, Box<dyn std::error::Error>> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

/// First-device local bootstrap (R26/R27).
pub fn run_bootstrap(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    if replication::has_active_or_local_device(&conn)? {
        return Err(
            "Bootstrap already enrolled: an active or local device already exists (R27). \
             Use `device enroll` for additional devices."
                .into(),
        );
    }

    let data_key = data_key_from_sqlcipher(&ctx._key)?;
    let keys = generate_device_keys().map_err(|e| e.to_string())?;
    let device_id = DeviceId::new();
    let ed_pub = keys.verifying_key().to_bytes();
    let x_pub = keys.x25519_public().to_bytes();
    let package = enrollment_package(&device_id, &ed_pub, &x_pub);
    let fp = fingerprint_sha256(&package);
    let enrolled_at = now_rfc3339()?;

    let identity = DeviceIdentityRow {
        device_id: device_id.to_string(),
        schema_version: REPLICATION_SCHEMA_VERSION as i64,
        ed25519_public: ed_pub.to_vec(),
        x25519_public: x_pub.to_vec(),
        display_name: Some("local".to_string()),
        status: "local".to_string(),
        enrolled_at: enrolled_at.clone(),
        revoked_at: None,
        enrolled_by_device_id: device_id.to_string(), // self (R26)
        fingerprint_sha256: fp.to_vec(),
    };
    replication::insert_device_identity(&conn, &identity)?;

    let seeds = DevicePrivateSeeds::from_key_pair(&keys);
    let sealed = private_blob::seal_device_private_blob(&data_key, &seeds, &device_id)
        .map_err(|e| e.to_string())?;
    replication::put_device_private_key_wrap(
        &conn,
        &DevicePrivateKeyRow {
            device_id: device_id.to_string(),
            wrap_schema_version: sealed.wrap_schema_version as i64,
            algorithm: "AES-256-GCM".to_string(),
            protection: sealed.protection,
            wrap_nonce: sealed.wrap_nonce.to_vec(),
            wrap_ciphertext: sealed.wrap_ciphertext,
            created_at: enrolled_at,
        },
    )?;

    println!("Device bootstrap complete (status=local).");
    println!("device_id: {device_id}");
    println!("fingerprint: {}", format_fingerprint_hyphen(&fp));
    println!("Note: multi-device is optional; not PQ; not remote wipe; not metadata-private.");
    Ok(())
}

/// Print local device dual-key fingerprint (R24).
pub fn run_fingerprint(ctx: &AppContext, raw: bool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let devices = replication::list_enrolled_devices(&conn)?;
    let local = devices
        .iter()
        .find(|d| d.status == "local")
        .or_else(|| devices.first())
        .ok_or("No enrolled device found. Run `ai-brains device bootstrap` first.")?;
    let mut fp = [0u8; 32];
    if local.fingerprint_sha256.len() != 32 {
        return Err("stored fingerprint is not 32 bytes".into());
    }
    fp.copy_from_slice(&local.fingerprint_sha256);
    if raw {
        println!("{}", hex::encode(fp));
    } else {
        println!("{}", format_fingerprint_hyphen(&fp));
    }
    Ok(())
}

/// List enrolled devices.
pub fn run_list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let devices = replication::list_enrolled_devices(&conn)?;
    if devices.is_empty() {
        println!("No enrolled devices. Run `ai-brains device bootstrap` first.");
        return Ok(());
    }
    println!(
        "{:<38} {:<8} {:<20} FINGERPRINT",
        "DEVICE_ID", "STATUS", "ENROLLED_BY"
    );
    for d in devices {
        let mut fp = [0u8; 32];
        if d.fingerprint_sha256.len() == 32 {
            fp.copy_from_slice(&d.fingerprint_sha256);
        }
        println!(
            "{:<38} {:<8} {:<20} {}",
            d.device_id,
            d.status,
            d.enrolled_by_device_id,
            format_fingerprint_hyphen(&fp)
        );
    }
    Ok(())
}

/// Export enrollment package for a *new* machine (does not enroll into vault).
pub fn run_package_export(out: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let keys = generate_device_keys().map_err(|e| e.to_string())?;
    let device_id = DeviceId::new();
    let ed_pub = keys.verifying_key().to_bytes();
    let x_pub = keys.x25519_public().to_bytes();
    let package = enrollment_package(&device_id, &ed_pub, &x_pub);
    let fp = fingerprint_sha256(&package);

    // Persist package + seeds sidecar for the new machine operator.
    // Package is public; seeds must be kept with the new device only.
    fs::write(&out, &package)?;
    let seeds_path = out.with_extension("seeds");
    let mut seeds_blob = Vec::with_capacity(64);
    seeds_blob.extend_from_slice(&keys.ed25519_seed);
    seeds_blob.extend_from_slice(&keys.x25519_seed);
    fs::write(&seeds_path, &seeds_blob)?;

    println!("Enrollment package written to {}", out.display());
    println!(
        "Private seeds written to {} (keep on new device only)",
        seeds_path.display()
    );
    println!("device_id: {device_id}");
    println!("fingerprint: {}", format_fingerprint_hyphen(&fp));
    println!(
        "Transfer the package to an enrolled vault and run: ai-brains device enroll --package <path>"
    );
    Ok(())
}

/// Enroll a peer from package on an already-enrolled vault.
pub fn run_enroll(
    ctx: &AppContext,
    package_path: PathBuf,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let enrolled = replication::list_enrolled_devices(&conn)?;
    let signer = enrolled
        .iter()
        .find(|d| d.status == "local" || d.status == "active")
        .ok_or(
            "No enrolled device on this vault. Run `ai-brains device bootstrap` first, \
             then enroll peers.",
        )?;

    let bytes = fs::read(&package_path)?;
    let parsed = parse_enrollment_package(&bytes).map_err(|e| e.to_string())?;
    let fp = fingerprint_sha256(&bytes);
    let fp_display = format_fingerprint_hyphen(&fp);

    println!("Peer enrollment package:");
    println!("  device_id:   {}", parsed.device_id);
    println!("  fingerprint: {fp_display}");
    println!("  schema:      {}", parsed.schema_version);
    println!();
    println!("Confirm this fingerprint matches the new device out-of-band (voice/visual).");

    if !yes {
        print!("Type 'yes' to enroll: ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.trim() != "yes" {
            return Err("Enrollment cancelled.".into());
        }
    }

    let enrolled_at = now_rfc3339()?;
    let identity = DeviceIdentityRow {
        device_id: parsed.device_id.to_string(),
        schema_version: parsed.schema_version as i64,
        ed25519_public: parsed.ed25519_pub.to_vec(),
        x25519_public: parsed.x25519_pub.to_vec(),
        display_name: None,
        status: "active".to_string(),
        enrolled_at,
        revoked_at: None,
        enrolled_by_device_id: signer.device_id.clone(),
        fingerprint_sha256: fp.to_vec(),
    };
    replication::insert_device_identity(&conn, &identity)?;
    println!(
        "Enrolled peer {} as active (signed class by {}).",
        parsed.device_id, signer.device_id
    );
    println!(
        "History delivery to the new vault requires relay (T177). T176 only updates this vault."
    );
    Ok(())
}

/// Revoke a device: tombstone + R23 delete peer wraps.
pub fn run_revoke(ctx: &AppContext, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let existing = replication::get_device(&conn, device_id)?
        .ok_or_else(|| format!("Device not found: {device_id}"))?;
    if existing.status == "revoked" {
        println!("Device {device_id} is already revoked.");
        return Ok(());
    }
    let revoked_at = now_rfc3339()?;
    replication::tombstone_device(&conn, device_id, &revoked_at, "cli-revoke")?;
    println!("Revoked and tombstoned device {device_id}.");
    println!(
        "Peer wraps for this recipient deleted (R23). Past DEKs on stolen device remain openable."
    );
    Ok(())
}
