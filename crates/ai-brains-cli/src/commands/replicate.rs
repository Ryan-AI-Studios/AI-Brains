//! `ai-brains replicate` — multi-device replication (T176/T177).
//!
//! Push/pull open **no sockets**. Only an **explicitly configured** file fake
//! relay (`--fake-relay` / `AI_BRAINS_SYNC_FAKE_RELAY_PATH`) is used.
//! Honesty: optional; not PQ; not remote wipe; not metadata-private.

use crate::commands::device::data_key_from_sqlcipher;
use crate::context::AppContext;
use ai_brains_core::ids::DeviceId;
use ai_brains_store::ReplicateEngine;
use ai_brains_store::projections::replication;
use ai_brains_sync::{FileFakeRelay, SealedDevicePrivate, open_device_private_blob};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const ENV_FAKE_RELAY: &str = "AI_BRAINS_SYNC_FAKE_RELAY_PATH";

fn cursor_to_json(c: &replication::ReplicationCursorRow) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    m.insert(
        "peer_device_id".to_string(),
        serde_json::Value::String(c.peer_device_id.clone()),
    );
    m.insert(
        "high_water_seq".to_string(),
        serde_json::Value::Number(c.high_water_seq.into()),
    );
    m.insert(
        "expected_local_seq".to_string(),
        serde_json::Value::Number(c.expected_local_seq.into()),
    );
    m.insert(
        "state".to_string(),
        serde_json::Value::String(c.state.clone()),
    );
    m.insert(
        "updated_at".to_string(),
        serde_json::Value::String(c.updated_at.clone()),
    );
    serde_json::Value::Object(m)
}

/// Resolved fake-relay path from flag or environment (never default-on).
pub fn resolve_fake_relay_path(flag: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = flag {
        return Some(p);
    }
    std::env::var(ENV_FAKE_RELAY)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
}

fn format_relay_status(path: Option<&Path>) -> String {
    match path {
        Some(p) => format!("file:{}", p.display()),
        None => "not configured".to_string(),
    }
}

/// Show local replication readiness / cursors / enrolled count / relay config.
pub fn run_status(
    ctx: &AppContext,
    fake_relay: Option<PathBuf>,
    format_json: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let relay_path = resolve_fake_relay_path(fake_relay);
    let conn = ctx.conn.lock()?;
    let devices = replication::list_enrolled_devices(&conn)?;
    let cursors = replication::list_cursors(&conn)?;
    let gap_or_blocked: Vec<_> = cursors
        .iter()
        .filter(|c| c.state == "sync_gap" || c.state == "blocked")
        .cloned()
        .collect();

    if format_json {
        let devices_json: Vec<serde_json::Value> = devices
            .iter()
            .map(|d| {
                let mut m = serde_json::Map::new();
                m.insert(
                    "device_id".to_string(),
                    serde_json::Value::String(d.device_id.clone()),
                );
                m.insert(
                    "status".to_string(),
                    serde_json::Value::String(d.status.clone()),
                );
                serde_json::Value::Object(m)
            })
            .collect();
        let cursors_json: Vec<serde_json::Value> = cursors.iter().map(cursor_to_json).collect();
        let mut out = serde_json::Map::new();
        out.insert(
            "relay".to_string(),
            serde_json::Value::String(format_relay_status(relay_path.as_deref())),
        );
        out.insert(
            "enrolled_count".to_string(),
            serde_json::Value::Number(devices.len().into()),
        );
        out.insert(
            "cursors".to_string(),
            serde_json::Value::Array(cursors_json),
        );
        out.insert(
            "gap_or_blocked".to_string(),
            serde_json::Value::Number(gap_or_blocked.len().into()),
        );
        out.insert(
            "devices".to_string(),
            serde_json::Value::Array(devices_json),
        );
        out.insert(
            "honesty".to_string(),
            serde_json::Value::String(
                "optional multi-device; not PQ; not remote wipe; not metadata-private".to_string(),
            ),
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(out))?
        );
        return Ok(());
    }

    if quiet {
        println!("{}", format_relay_status(relay_path.as_deref()));
        return Ok(());
    }

    println!("Multi-device replication status");
    println!(
        "  relay:           {}",
        format_relay_status(relay_path.as_deref())
    );
    println!("  enrolled_count:  {}", devices.len());
    println!("  cursors:         {}", cursors.len());
    if !gap_or_blocked.is_empty() {
        println!("  gap_or_blocked:  {}", gap_or_blocked.len());
        for c in &gap_or_blocked {
            println!(
                "    - peer={} state={} expected={} high_water={}",
                c.peer_device_id, c.state, c.expected_local_seq, c.high_water_seq
            );
        }
    }
    println!(
        "  honesty:         optional multi-device; not PQ; not remote wipe; not metadata-private"
    );
    if devices.is_empty() {
        println!("  hint:            run `ai-brains device bootstrap` to enroll first device");
    } else {
        println!("  devices:");
        for d in &devices {
            println!("    - {} ({})", d.device_id, d.status);
        }
    }
    Ok(())
}

/// Dump replication_cursor rows.
pub fn run_cursors(ctx: &AppContext, format_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let cursors = replication::list_cursors(&conn)?;
    if format_json {
        let rows: Vec<serde_json::Value> = cursors.iter().map(cursor_to_json).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Array(rows))?
        );
        return Ok(());
    }
    if cursors.is_empty() {
        println!("No replication cursors.");
        return Ok(());
    }
    println!(
        "{:<38} {:>12} {:>12} {:<12} UPDATED_AT",
        "PEER_DEVICE_ID", "HIGH_WATER", "EXPECTED", "STATE"
    );
    for c in cursors {
        println!(
            "{:<38} {:>12} {:>12} {:<12} {}",
            c.peer_device_id, c.high_water_seq, c.expected_local_seq, c.state, c.updated_at
        );
    }
    Ok(())
}

fn require_relay_path(fake_relay: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    resolve_fake_relay_path(fake_relay).ok_or_else(|| RelayNotConfigured.into())
}

fn load_local_device(
    conn: &rusqlite::Connection,
    data_key: &ai_brains_crypto::DataKey,
) -> Result<(DeviceId, ai_brains_crypto::DataKey), Box<dyn std::error::Error>> {
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
    let sealed = SealedDevicePrivate {
        wrap_schema_version: wrap.wrap_schema_version as u32,
        protection: wrap.protection,
        wrap_nonce: wrap
            .wrap_nonce
            .as_slice()
            .try_into()
            .map_err(|_| "wrap_nonce must be 12 bytes")?,
        wrap_ciphertext: wrap.wrap_ciphertext,
    };
    // Open once to validate the wrap is usable with this DataKey.
    let _seeds =
        open_device_private_blob(data_key, &sealed, &device_id).map_err(|e| e.to_string())?;
    Ok((device_id, data_key.clone()))
}

/// Push pending local envelopes to the configured file fake relay.
pub fn run_push(
    ctx: &AppContext,
    fake_relay: Option<PathBuf>,
    format_json: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = require_relay_path(fake_relay)?;
    let data_key = data_key_from_sqlcipher(&ctx._key)?;
    let relay = Arc::new(FileFakeRelay::open_or_create(&path).map_err(|e| e.to_string())?);
    let conn = ctx.conn.lock()?;
    let (device_id, data_key) = load_local_device(&conn, &data_key)?;
    let mut engine = ReplicateEngine::new(&conn, relay, data_key, device_id);
    let n = engine.push_pending().map_err(|e| e.to_string())?;
    if format_json {
        let mut out = serde_json::Map::new();
        out.insert("ok".to_string(), serde_json::Value::Bool(true));
        out.insert("pushed".to_string(), serde_json::Value::Number(n.into()));
        out.insert(
            "relay".to_string(),
            serde_json::Value::String(format!("file:{}", path.display())),
        );
        println!(
            "{}",
            serde_json::to_string(&serde_json::Value::Object(out))?
        );
        return Ok(());
    }
    if !quiet {
        // Always report count (including 0) so empty outbox is observable.
        println!(
            "replicate push: pushed {n} envelope(s) → file:{}",
            path.display()
        );
        println!("Honesty: fake relay is local disk only; not a secure production relay.");
    }
    Ok(())
}

/// Pull peer streams from the configured file fake relay and apply.
pub fn run_pull(
    ctx: &AppContext,
    fake_relay: Option<PathBuf>,
    format_json: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = require_relay_path(fake_relay)?;
    let data_key = data_key_from_sqlcipher(&ctx._key)?;
    let relay = Arc::new(FileFakeRelay::open_or_create(&path).map_err(|e| e.to_string())?);
    let conn = ctx.conn.lock()?;
    let (device_id, data_key) = load_local_device(&conn, &data_key)?;
    let mut engine = ReplicateEngine::new(&conn, relay, data_key, device_id);
    let n = engine.pull_all_peers().map_err(|e| e.to_string())?;
    if format_json {
        let mut out = serde_json::Map::new();
        out.insert("ok".to_string(), serde_json::Value::Bool(true));
        // `applied` = envelope count successfully applied this pull (not peer count).
        out.insert("applied".to_string(), serde_json::Value::Number(n.into()));
        out.insert(
            "relay".to_string(),
            serde_json::Value::String(format!("file:{}", path.display())),
        );
        println!(
            "{}",
            serde_json::to_string(&serde_json::Value::Object(out))?
        );
        return Ok(());
    }
    if !quiet {
        println!(
            "replicate pull: applied {n} envelope(s) from file:{}",
            path.display()
        );
        println!("Honesty: fake relay is local disk only; not a secure production relay.");
    }
    Ok(())
}

/// Structured error when fake relay is not configured (F12).
#[derive(Debug)]
pub struct RelayNotConfigured;

impl std::fmt::Display for RelayNotConfigured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "relay not configured: pass --fake-relay <path> or set {ENV_FAKE_RELAY} \
             (never default-on; no production network client)"
        )
    }
}

impl std::error::Error for RelayNotConfigured {}
