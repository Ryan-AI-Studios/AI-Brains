//! `ai-brains device` — multi-device enrollment (T176 / ADR-0018).
//!
//! Honesty: optional multi-device; **not** post-quantum; **not** remote wipe;
//! **not** metadata-private. First-device uses `bootstrap`; peers use OOB
//! fingerprint + `enroll` (T177 delivers history via relay).
//!
//! # Dual-write (intentional for T176)
//!
//! Bootstrap / enroll / revoke:
//! 1. Append immutable `DeviceEnrolled` / `DeviceRevoked` to the canonical
//!    event log (SOV — AGENTS.md + spec §9.1).
//! 2. Write side tables (`device_identity`, `signed_replication_control`, …)
//!    for signed wire control material used by multi-device apply.
//!
//! Projection apply-from-log for `device_identity` is deferred (T177 may unify).

use crate::context::AppContext;
use ai_brains_core::ids::DeviceId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::DataKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, DeviceEnrolledPayload as EventDeviceEnrolledPayload,
    DeviceRevokedPayload as EventDeviceRevokedPayload, Payload,
};
use ai_brains_store::EventStore;
use ai_brains_store::projections::replication::{
    self, BootstrapLocalDeviceInput, DeviceIdentityRow, DevicePrivateKeyRow, EnvelopeIndexRow,
    SignedControlRow,
};
use ai_brains_sync::{
    ControlPayload, DeviceEnrolledPayload, DevicePrivateSeeds, DeviceRevokedPayload,
    REPLICATION_SCHEMA_VERSION, build_and_sign_control, enrollment_package, fingerprint_sha256,
    format_fingerprint_hyphen, generate_device_keys, nil_content_key_id, open_device_private_blob,
    parse_enrollment_package, private_blob, verify_envelope,
};
use ed25519_dalek::VerifyingKey;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use zeroize::Zeroizing;

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

fn signed_control_rows(
    built: &ai_brains_sync::SignedControlEnvelope,
    created_at: &str,
) -> (SignedControlRow, EnvelopeIndexRow) {
    let outer = &built.signed.outer;
    let control = SignedControlRow {
        event_id: outer.event_id.to_string(),
        envelope_id: outer.envelope_id.as_uuid().to_string(),
        sender_device_id: outer.device_id.to_string(),
        content_type_code: outer.content_type_code.as_u16() as i64,
        body: built.body.clone(),
        signature: built.signed.signature.to_vec(),
        schema_version: outer.schema_version as i64,
        local_seq: outer.local_seq as i64,
        created_at: created_at.to_string(),
    };
    let index = EnvelopeIndexRow {
        envelope_id: outer.envelope_id.as_uuid().to_string(),
        event_id: outer.event_id.to_string(),
        sender_device_id: outer.device_id.to_string(),
        local_seq: outer.local_seq as i64,
        content_type_code: outer.content_type_code.as_u16() as i64,
        content_key_id: Some(outer.content_key_id.to_string()),
        body_len: built.body.len() as i64,
        padding_bucket: None,
        applied_at: Some(created_at.to_string()),
    };
    (control, index)
}

fn sealed_to_row(
    device_id: &DeviceId,
    sealed: &ai_brains_sync::SealedDevicePrivate,
    created_at: &str,
) -> DevicePrivateKeyRow {
    DevicePrivateKeyRow {
        device_id: device_id.to_string(),
        wrap_schema_version: sealed.wrap_schema_version as i64,
        algorithm: "AES-256-GCM".to_string(),
        protection: sealed.protection.clone(),
        wrap_nonce: sealed.wrap_nonce.to_vec(),
        wrap_ciphertext: sealed.wrap_ciphertext.clone(),
        created_at: created_at.to_string(),
    }
}

fn load_local_signing_key(
    conn: &rusqlite::Connection,
    data_key: &DataKey,
) -> Result<(DeviceId, ed25519_dalek::SigningKey), Box<dyn std::error::Error>> {
    let devices = replication::list_enrolled_devices(conn)?;
    let local = devices
        .iter()
        .find(|d| d.status == "local")
        .or_else(|| devices.iter().find(|d| d.status == "active"))
        .ok_or("No enrolled device on this vault. Run `ai-brains device bootstrap` first.")?;
    let device_id: DeviceId = local
        .device_id
        .parse()
        .map_err(|e| format!("invalid local device_id: {e}"))?;
    let wrap = replication::get_device_private_key_wrap(conn, &local.device_id)?
        .ok_or("Local device private key wrap missing; vault may be incomplete.")?;
    let sealed = ai_brains_sync::SealedDevicePrivate {
        wrap_schema_version: wrap.wrap_schema_version as u32,
        protection: wrap.protection,
        wrap_nonce: wrap
            .wrap_nonce
            .as_slice()
            .try_into()
            .map_err(|_| "wrap_nonce must be 12 bytes")?,
        wrap_ciphertext: wrap.wrap_ciphertext,
    };
    let seeds =
        open_device_private_blob(data_key, &sealed, &device_id).map_err(|e| e.to_string())?;
    let key_pair = seeds.into_key_pair();
    Ok((device_id, key_pair.signing_key()))
}

/// Append membership control to the canonical event log (SOV). Side stores are separate.
#[allow(clippy::too_many_arguments)]
fn append_device_enrolled_event(
    ctx: &AppContext,
    device_id: DeviceId,
    enrolled_by: DeviceId,
    status: &str,
    fingerprint: &[u8; 32],
    ed25519_pub: &[u8; 32],
    x25519_pub: &[u8; 32],
    schema_version: u16,
    built: &ai_brains_sync::SignedControlEnvelope,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = EventBuilder::new(
        AggregateType::System,
        device_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceEnrolled(EventDeviceEnrolledPayload {
        device_id,
        enrolled_by_device_id: enrolled_by,
        status: status.to_string(),
        fingerprint_sha256: hex::encode(fingerprint),
        ed25519_public: hex::encode(ed25519_pub),
        x25519_public: hex::encode(x25519_pub),
        schema_version,
        replication_event_id: built.signed.outer.event_id,
        local_seq: built.signed.outer.local_seq,
    }))?;
    let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    store.append_event(&event)?;
    Ok(())
}

fn append_device_revoked_event(
    ctx: &AppContext,
    device_id: DeviceId,
    revoked_by: DeviceId,
    reason_code: &str,
    built: &ai_brains_sync::SignedControlEnvelope,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = EventBuilder::new(
        AggregateType::System,
        device_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::DeviceRevoked(EventDeviceRevokedPayload {
        device_id,
        revoked_by_device_id: revoked_by,
        reason_code: reason_code.to_string(),
        replication_event_id: built.signed.outer.event_id,
        local_seq: built.signed.outer.local_seq,
    }))?;
    let store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    store.append_event(&event)?;
    Ok(())
}

/// First-device local bootstrap (R26/R27): identity + private key + signed DeviceEnrolled
/// in one SQLite transaction, plus canonical event-log SOV append (dual-write).
pub fn run_bootstrap(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    {
        let conn = ctx.conn.lock()?;
        if replication::has_active_or_local_device(&conn)? {
            return Err(
                "Bootstrap already enrolled: an active or local device already exists (R27). \
                 Use `device enroll` for additional devices."
                    .into(),
            );
        }
    }

    let data_key = data_key_from_sqlcipher(&ctx._key)?;
    let keys = generate_device_keys().map_err(|e| e.to_string())?;
    let device_id = DeviceId::new();
    let ed_pub = keys.verifying_key().to_bytes();
    let x_pub = keys.x25519_public().to_bytes();
    let package = enrollment_package(&device_id, &ed_pub, &x_pub);
    let fp = fingerprint_sha256(&package);
    let enrolled_at = now_rfc3339()?;

    // Self-sign DeviceEnrolled with in-memory signing key before seal (ID-1 / R26).
    let local_seq = 1u64;
    let control_payload = ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
        schema_version: REPLICATION_SCHEMA_VERSION,
        device_id,
        ed25519_pub: ed_pub,
        x25519_pub: x_pub,
    });
    let built = build_and_sign_control(
        ai_brains_sync::ContentTypeCode::DeviceEnrolled,
        &control_payload,
        device_id,
        local_seq,
        &keys.signing_key(),
        nil_content_key_id(),
    )
    .map_err(|e| e.to_string())?;
    // Fail closed: signature must verify against the public we store.
    verify_envelope(&built.signed, &keys.verifying_key()).map_err(|e| e.to_string())?;

    // SOV first: immutable event log (AGENTS.md + spec §9.1). Side store next.
    append_device_enrolled_event(
        ctx,
        device_id,
        device_id,
        "local",
        &fp,
        &ed_pub,
        &x_pub,
        REPLICATION_SCHEMA_VERSION,
        &built,
    )?;

    let (signed_control, envelope_index) = signed_control_rows(&built, &enrolled_at);

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

    let seeds = DevicePrivateSeeds::from_key_pair(&keys);
    let sealed = private_blob::seal_device_private_blob(&data_key, &seeds, &device_id)
        .map_err(|e| e.to_string())?;
    let private_key = sealed_to_row(&device_id, &sealed, &enrolled_at);

    {
        let mut conn = ctx.conn.lock()?;
        replication::bootstrap_local_device(
            &mut conn,
            &BootstrapLocalDeviceInput {
                identity,
                private_key,
                signed_control,
                envelope_index,
            },
        )?;
    }

    println!("Device bootstrap complete (status=local).");
    println!("device_id: {device_id}");
    println!("fingerprint: {}", format_fingerprint_hyphen(&fp));
    println!("signed_control: DeviceEnrolled (self-signed, local_seq={local_seq})");
    println!("event_log: DeviceEnrolled appended (canonical SOV; dual-write with side store)");
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
///
/// By default writes **public package only**. Optional private material:
/// - Windows: DPAPI-protect seeds when `--write-private-key <path>` is set.
/// - Non-Windows: refuse insecure raw seed write (Windows-first).
pub fn run_package_export(
    out: PathBuf,
    write_private_key: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let keys = generate_device_keys().map_err(|e| e.to_string())?;
    let device_id = DeviceId::new();
    let ed_pub = keys.verifying_key().to_bytes();
    let x_pub = keys.x25519_public().to_bytes();
    let package = enrollment_package(&device_id, &ed_pub, &x_pub);
    let fp = fingerprint_sha256(&package);

    // Public package only by default (ID-2). Never write raw `.seeds` sidecar.
    fs::write(&out, &package)?;
    println!("Enrollment package written to {}", out.display());
    println!("device_id: {device_id}");
    println!("fingerprint: {}", format_fingerprint_hyphen(&fp));

    if let Some(priv_path) = write_private_key {
        #[cfg(windows)]
        {
            let mut plain = Zeroizing::new([0u8; 64]);
            plain[..32].copy_from_slice(&keys.ed25519_seed);
            plain[32..].copy_from_slice(&keys.x25519_seed);
            let wrapped = ai_brains_crypto::dpapi::wrap_key(plain.as_slice())
                .map_err(|e| format!("DPAPI protect private key: {e}"))?;
            // `plain` zeroizes on drop after wrap.
            drop(plain);
            fs::write(&priv_path, &wrapped)?;
            println!(
                "DPAPI-protected private key written to {} (this machine/user only)",
                priv_path.display()
            );
        }
        #[cfg(not(windows))]
        {
            let _ = priv_path;
            return Err(
                "Writing private key material requires OS-bound protection. \
                 On non-Windows, omit --write-private-key (Windows DPAPI is supported; \
                 passphrase wrap is not implemented in T176). Raw seed files are forbidden."
                    .into(),
            );
        }
    }

    println!(
        "Transfer the package to an enrolled vault and run: ai-brains device enroll --package <path>"
    );
    println!("Honesty: package is public; keep any private key file on the new device only.");
    Ok(())
}

/// Enroll a peer from package on an already-enrolled vault (signed DeviceEnrolled by local).
pub fn run_enroll(
    ctx: &AppContext,
    package_path: PathBuf,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let data_key = data_key_from_sqlcipher(&ctx._key)?;
    let (signer_id, signing_key, local_seq, vk_bytes) = {
        let conn = ctx.conn.lock()?;
        let (signer_id, signing_key) = load_local_signing_key(&conn, &data_key)?;
        let local_seq = replication::next_local_seq(&conn, &signer_id.to_string())? as u64;
        let signer_row = replication::get_device(&conn, &signer_id.to_string())?
            .ok_or("signer identity missing")?;
        let vk_bytes: [u8; 32] = signer_row
            .ed25519_public
            .as_slice()
            .try_into()
            .map_err(|_| "signer ed25519_public must be 32 bytes")?;
        (signer_id, signing_key, local_seq, vk_bytes)
    };

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
    let control_payload = ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
        schema_version: parsed.schema_version,
        device_id: parsed.device_id,
        ed25519_pub: parsed.ed25519_pub,
        x25519_pub: parsed.x25519_pub,
    });
    let built = build_and_sign_control(
        ai_brains_sync::ContentTypeCode::DeviceEnrolled,
        &control_payload,
        signer_id,
        local_seq,
        &signing_key,
        nil_content_key_id(),
    )
    .map_err(|e| e.to_string())?;

    let vk = VerifyingKey::from_bytes(&vk_bytes).map_err(|e| format!("verifying key: {e}"))?;
    verify_envelope(&built.signed, &vk).map_err(|e| e.to_string())?;

    // SOV first, then side store (dual-write intentional for T176).
    append_device_enrolled_event(
        ctx,
        parsed.device_id,
        signer_id,
        "active",
        &fp,
        &parsed.ed25519_pub,
        &parsed.x25519_pub,
        parsed.schema_version,
        &built,
    )?;

    let (signed_control, envelope_index) = signed_control_rows(&built, &enrolled_at);
    let identity = DeviceIdentityRow {
        device_id: parsed.device_id.to_string(),
        schema_version: parsed.schema_version as i64,
        ed25519_public: parsed.ed25519_pub.to_vec(),
        x25519_public: parsed.x25519_pub.to_vec(),
        display_name: None,
        status: "active".to_string(),
        enrolled_at,
        revoked_at: None,
        enrolled_by_device_id: signer_id.to_string(),
        fingerprint_sha256: fp.to_vec(),
    };

    {
        let mut conn = ctx.conn.lock()?;
        replication::enroll_peer_device(&mut conn, &identity, &signed_control, &envelope_index)?;
    }

    println!(
        "Enrolled peer {} as active (signed DeviceEnrolled by {}).",
        parsed.device_id, signer_id
    );
    println!(
        "History delivery to the new vault requires relay (T177). T176 only updates this vault."
    );
    Ok(())
}

/// Revoke a device: signed DeviceRevoked + tombstone + R23 delete peer wraps.
///
/// Self-revoke by the local signer alone is forbidden (ADR-0018 L4).
pub fn run_revoke(ctx: &AppContext, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data_key = data_key_from_sqlcipher(&ctx._key)?;

    // Phase 1: read identity + signer under a short lock (no nested EventStore lock).
    let (signer_id, signing_key, local_seq, vk_bytes) = {
        let conn = ctx.conn.lock()?;
        let existing = replication::get_device(&conn, device_id)?
            .ok_or_else(|| format!("Device not found: {device_id}"))?;
        if existing.status == "revoked" {
            println!("Device {device_id} is already revoked.");
            return Ok(());
        }
        let (signer_id, signing_key) = load_local_signing_key(&conn, &data_key)?;
        let local_seq = replication::next_local_seq(&conn, &signer_id.to_string())? as u64;
        let signer_row = replication::get_device(&conn, &signer_id.to_string())?
            .ok_or("signer identity missing")?;
        let vk_bytes: [u8; 32] = signer_row
            .ed25519_public
            .as_slice()
            .try_into()
            .map_err(|_| "signer ed25519_public must be 32 bytes")?;
        (signer_id, signing_key, local_seq, vk_bytes)
    };

    let revoked_device: DeviceId = device_id
        .parse()
        .map_err(|e| format!("invalid device_id: {e}"))?;

    // ADR-0018 L4: cannot self-revoke as sole authority.
    if revoked_device == signer_id {
        return Err("Cannot self-revoke as sole authority (ADR-0018 L4). \
             Revoke from a different enrolled device, or enroll a peer first."
            .into());
    }

    let reason_code = "cli-revoke".to_string();
    let revoked_at = now_rfc3339()?;

    let control_payload = ControlPayload::DeviceRevoked(DeviceRevokedPayload {
        device_id: revoked_device,
        reason_code: reason_code.clone(),
    });
    let built = build_and_sign_control(
        ai_brains_sync::ContentTypeCode::DeviceRevoked,
        &control_payload,
        signer_id,
        local_seq,
        &signing_key,
        nil_content_key_id(),
    )
    .map_err(|e| e.to_string())?;

    // Fail closed: match bootstrap/enroll — verify before persist.
    let vk = VerifyingKey::from_bytes(&vk_bytes).map_err(|e| format!("verifying key: {e}"))?;
    verify_envelope(&built.signed, &vk).map_err(|e| e.to_string())?;

    // SOV first, then side store (dual-write intentional for T176).
    append_device_revoked_event(ctx, revoked_device, signer_id, &reason_code, &built)?;

    let (signed_control, envelope_index) = signed_control_rows(&built, &revoked_at);
    {
        let mut conn = ctx.conn.lock()?;
        replication::revoke_device_with_control(
            &mut conn,
            device_id,
            &revoked_at,
            &reason_code,
            &signed_control,
            &envelope_index,
        )?;
    }

    println!("Revoked and tombstoned device {device_id}.");
    println!(
        "Signed DeviceRevoked by {signer_id} (local_seq={local_seq}). Peer wraps deleted (R23)."
    );
    println!("Past DEKs on a stolen device remain openable (not remote wipe / not NIST Purge).");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    #![allow(non_snake_case)]

    use super::*;
    use ai_brains_core::ids::{ContentKeyId, ReplicationEventId};
    use ai_brains_sync::{
        CONTENT_TYPE_DEVICE_ENROLLED, ContentTypeCode, EnvelopeId, OuterEnvelope, SignedEnvelope,
    };
    use uuid::Uuid;

    fn signed_envelope_from_row(row: &SignedControlRow) -> Result<SignedEnvelope, String> {
        let content_type =
            ContentTypeCode::from_u16(row.content_type_code as u16).map_err(|e| e.to_string())?;
        let envelope_id = EnvelopeId::from_uuid(
            Uuid::parse_str(&row.envelope_id).map_err(|e| format!("envelope_id: {e}"))?,
        );
        let device_id = DeviceId::from_uuid(
            Uuid::parse_str(&row.sender_device_id).map_err(|e| format!("sender: {e}"))?,
        );
        let event_id = ReplicationEventId::from_uuid(
            Uuid::parse_str(&row.event_id).map_err(|e| format!("event_id: {e}"))?,
        );
        let sig: [u8; 64] = row
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| "signature must be 64 bytes".to_string())?;
        Ok(SignedEnvelope {
            outer: OuterEnvelope {
                schema_version: row.schema_version as u16,
                envelope_id,
                device_id,
                local_seq: row.local_seq as u64,
                content_type_code: content_type,
                event_id,
                content_key_id: ContentKeyId::from_uuid(Uuid::nil()),
                ciphertext: row.body.clone(),
                wrap_records: vec![],
            },
            signature: sig,
        })
    }

    #[test]
    fn signed_control_rows__maps_envelope() {
        let keys = generate_device_keys().expect("keys");
        let device = DeviceId::new();
        let payload = ControlPayload::DeviceEnrolled(DeviceEnrolledPayload {
            schema_version: REPLICATION_SCHEMA_VERSION,
            device_id: device,
            ed25519_pub: keys.verifying_key().to_bytes(),
            x25519_pub: keys.x25519_public().to_bytes(),
        });
        let built = build_and_sign_control(
            ContentTypeCode::DeviceEnrolled,
            &payload,
            device,
            1,
            &keys.signing_key(),
            nil_content_key_id(),
        )
        .expect("sign");
        let (ctrl, idx) = signed_control_rows(&built, "2026-07-30T00:00:00Z");
        assert_eq!(ctrl.signature.len(), 64);
        assert_eq!(ctrl.local_seq, 1);
        assert_eq!(idx.content_type_code, CONTENT_TYPE_DEVICE_ENROLLED as i64);
        let envelope = signed_envelope_from_row(&ctrl).expect("reconstruct");
        verify_envelope(&envelope, &keys.verifying_key()).expect("verify");
    }
}
