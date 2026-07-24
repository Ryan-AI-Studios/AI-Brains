//! Shadow vault create — copy events into a new vault without mutating live data.
//!
//! Safety refusals (via `ai-brains-path` location helpers + reparse checks):
//! 1. source and destination are the same location
//! 2. destination equals the resolved live vault
//! 3. destination is inside the live vault's parent directory
//! 4. destination exists as a reparse/symlink

use crate::artifact_security::{is_reparse_or_symlink, refuse_if_reparse};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::hash::compute_payload_hash;
use ai_brains_events::{Envelope, Payload};
use ai_brains_path::{path_is_same_or_inside, paths_refer_to_same_location, resolve_best_effort};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::{EventStore, SqliteEventStore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const SHADOW_MANIFEST_VERSION: u32 = 1;
const REDACTED_PLACEHOLDER: &str = "[REDACTED]";

#[derive(Debug, Serialize)]
struct ShadowManifest {
    version: u32,
    created_at: String,
    source_path: String,
    destination_path: String,
    source_fingerprint: String,
    redaction_policy: String,
    event_count: usize,
    dry_run: bool,
}

/// Resolve the live vault path using the same chain as CLI env loading:
/// 1. `AI_BRAINS_VAULT_PATH` (already loaded by main)
/// 2. else `~/.ai-brains/.env`
/// 3. else `None` (only same-path source/dest enforced)
pub fn resolve_live_vault_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AI_BRAINS_VAULT_PATH") {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    let home = dirs::home_dir()?;
    let env_path = home.join(".ai-brains").join(".env");
    if !env_path.exists() {
        return None;
    }
    let Ok(iter) = dotenvy::from_path_iter(&env_path) else {
        return None;
    };
    for entry in iter.flatten() {
        let (key, value) = entry;
        if key == "AI_BRAINS_VAULT_PATH" {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    None
}

fn default_sql_key(key: Option<String>) -> SqlCipherKey {
    let key_str = key.unwrap_or_else(|| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });
    SqlCipherKey::from_raw(key_str)
}

fn source_fingerprint(
    source: &Path,
    event_count: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new();
    let resolved = resolve_best_effort(&source.to_string_lossy());
    hasher.update(resolved.as_bytes());
    hasher.update(b"|");
    if source.exists() {
        let meta = fs::metadata(source)?;
        hasher.update(meta.len().to_le_bytes());
        if let Ok(modified) = meta.modified()
            && let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH)
        {
            hasher.update(dur.as_secs().to_le_bytes());
        }
    }
    hasher.update(b"|");
    hasher.update(event_count.to_le_bytes());
    Ok(hex::encode(hasher.finalize()))
}

fn redact_turn_content(mut envelope: Envelope) -> Result<Envelope, Box<dyn std::error::Error>> {
    match &mut envelope.payload {
        Payload::UserPromptRecorded(p) => {
            p.content = REDACTED_PLACEHOLDER.to_string();
        }
        Payload::AssistantFinalRecorded(p) => {
            p.content = REDACTED_PLACEHOLDER.to_string();
        }
        _ => {}
    }
    envelope.payload_hash = compute_payload_hash(&envelope.payload)
        .map_err(|e| format!("failed to recompute payload_hash after redaction: {e}"))?;
    Ok(envelope)
}

/// Safety checks before any write. Returns Ok(()) when destination is allowed.
pub fn refuse_unsafe_destination(
    source: &Path,
    destination: &Path,
    live_vault: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if paths_refer_to_same_location(source, destination) {
        return Err(
            "refusing shadow create: source and destination refer to the same location".into(),
        );
    }

    if let Some(live) = live_vault {
        if paths_refer_to_same_location(destination, live) {
            return Err(
                "refusing shadow create: destination equals the resolved live vault path".into(),
            );
        }
        if let Some(live_parent) = live.parent()
            && !live_parent.as_os_str().is_empty()
            && path_is_same_or_inside(destination, live_parent)
        {
            return Err(format!(
                    "refusing shadow create: destination is inside the live vault parent directory ({})",
                    live_parent.display()
                )
                .into());
        }
    }

    if destination.exists()
        && let Err(msg) = refuse_if_reparse(destination, is_reparse_or_symlink(destination)?)
    {
        return Err(msg.into());
    }
    if let Some(parent) = destination.parent()
        && parent.exists()
        && let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?)
    {
        return Err(format!("destination parent: {msg}").into());
    }

    Ok(())
}

fn manifest_path_for(destination: &Path) -> PathBuf {
    match destination.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("shadow-manifest.json"),
        _ => PathBuf::from("shadow-manifest.json"),
    }
}

fn created_at_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn run_create(
    source: PathBuf,
    destination: PathBuf,
    redact_turn_content_flag: bool,
    dry_run: bool,
    key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let live = resolve_live_vault_path();
    if live.is_none() {
        eprintln!(
            "note: no live vault resolved (AI_BRAINS_VAULT_PATH unset and ~/.ai-brains/.env \
             has no vault path); only source/destination same-path checks apply"
        );
    }

    refuse_unsafe_destination(&source, &destination, live.as_deref())?;

    if !source.exists() {
        return Err(format!("source vault does not exist: {}", source.display()).into());
    }

    let sql_key = default_sql_key(key);
    // Never migrate the live/source vault — shadow create must not mutate source.
    // Source is assumed already migrated (e.g. via `ai-brains init` / normal use).
    let source_conn = VaultConnection::open(&source, &sql_key)?;
    let source_store = SqliteEventStore::new(source_conn);
    let events = source_store.read_all_events()?;

    let redaction_policy = if redact_turn_content_flag {
        "redact-turn-content"
    } else {
        "no-redact-turn-content"
    };

    let fingerprint = source_fingerprint(&source, events.len())?;
    let created_at = created_at_rfc3339();
    let manifest = ShadowManifest {
        version: SHADOW_MANIFEST_VERSION,
        created_at: created_at.clone(),
        source_path: source.display().to_string(),
        destination_path: destination.display().to_string(),
        source_fingerprint: fingerprint,
        redaction_policy: redaction_policy.to_string(),
        event_count: events.len(),
        dry_run,
    };

    if dry_run {
        println!(
            "[dry-run] Would create shadow vault at {} from {} ({} event(s), redaction={})",
            destination.display(),
            source.display(),
            events.len(),
            redaction_policy
        );
        println!(
            "[dry-run] Would write manifest at {}",
            manifest_path_for(&destination).display()
        );
        return Ok(());
    }

    if destination.exists() {
        return Err(format!(
            "destination already exists: {} (refusing to overwrite; pick a new path)",
            destination.display()
        )
        .into());
    }

    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    // Re-check reparse after create_dir_all (TOCTOU soft check; full handle design is P6).
    refuse_unsafe_destination(&source, &destination, live.as_deref())?;

    let dest_conn = VaultConnection::open(&destination, &sql_key)?;
    dest_conn.migrate()?;
    let dest_store = SqliteEventStore::new(dest_conn);

    let mut written = 0usize;
    for event in events {
        let to_append = if redact_turn_content_flag {
            redact_turn_content(event)?
        } else {
            event
        };
        dest_store.append_event(&to_append)?;
        written += 1;
    }

    let manifest_path = manifest_path_for(&destination);
    let mut file = fs::File::create(&manifest_path)?;
    let body = serde_json::to_string_pretty(&manifest)?;
    file.write_all(body.as_bytes())?;
    file.write_all(b"\n")?;

    println!(
        "Shadow vault created at {} ({} event(s), redaction={})",
        destination.display(),
        written,
        redaction_policy
    );
    println!("Manifest written to {}", manifest_path.display());
    Ok(())
}
