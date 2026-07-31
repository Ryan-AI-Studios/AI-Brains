//! `ai-brains replicate` — multi-device replication status (T176).
//!
//! Push/pull open **no sockets**; relay is deferred to T177.
//! Honesty: optional; not PQ; not remote wipe; not metadata-private.

use crate::context::AppContext;
use ai_brains_store::projections::replication;

/// Show local replication readiness / cursors / enrolled count.
pub fn run_status(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let devices = replication::list_enrolled_devices(&conn)?;
    let cursors = replication::list_cursors(&conn)?;
    println!("Multi-device replication status");
    println!("  relay:           not configured (deferred to T177)");
    println!("  enrolled_count:  {}", devices.len());
    println!("  cursors:         {}", cursors.len());
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
pub fn run_cursors(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let cursors = replication::list_cursors(&conn)?;
    if cursors.is_empty() {
        println!("No replication cursors (empty until relay sync in T177).");
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

/// Push — structured error; no sockets (R18).
pub fn run_push(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    Err(RelayNotConfigured.into())
}

/// Pull — structured error; no sockets (R18).
pub fn run_pull(_ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    Err(RelayNotConfigured.into())
}

/// Structured error for T177 deferred relay.
#[derive(Debug)]
pub struct RelayNotConfigured;

impl std::fmt::Display for RelayNotConfigured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "relay not configured / deferred to T177 (no network client in T176)"
        )
    }
}

impl std::error::Error for RelayNotConfigured {}
