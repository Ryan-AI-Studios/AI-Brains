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

/// T197: shared operator resolver (no silent zero).
fn default_sql_key(
    key: Option<String>,
) -> Result<SqlCipherKey, crate::key_resolve::KeyResolveError> {
    crate::key_resolve::resolve_operator_sqlcipher_key(key)
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
    // Only rewrite known turn payloads. Unknown and all other kinds pass through
    // with their original payload_hash (T148 R0 — no strip/re-hash of Unknown).
    let changed = match &mut envelope.payload {
        Payload::UserPromptRecorded(p) => {
            p.content = REDACTED_PLACEHOLDER.to_string();
            true
        }
        Payload::AssistantFinalRecorded(p) => {
            p.content = REDACTED_PLACEHOLDER.to_string();
            true
        }
        _ => false,
    };
    if changed {
        envelope.payload_hash = compute_payload_hash(&envelope.payload)
            .map_err(|e| format!("failed to recompute payload_hash after redaction: {e}"))?;
    }
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

    // Codex R5: do not gate on `exists()`. Dangling symlinks have exists()==false
    // while symlink_metadata / is_reparse_or_symlink still detect them; File::create
    // or SQLite open can follow/create through the link.
    if let Err(msg) = refuse_if_reparse(destination, is_reparse_or_symlink(destination)?) {
        return Err(msg.into());
    }
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
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

    let sql_key = default_sql_key(key)?;
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
    let body = format!("{}\n", serde_json::to_string_pretty(&manifest)?);
    let parent = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| {
            format!(
                "shadow-manifest path has no parent: {}",
                manifest_path.display()
            )
        })?;
    let file_name = manifest_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            format!(
                "shadow-manifest missing UTF-8 name: {}",
                manifest_path.display()
            )
        })?;
    // T193 P1: nofollow SOOT Replace for shadow-manifest.json.
    ai_brains_path::write_file_nofollow_under_parent_path(
        parent,
        file_name,
        body.as_bytes(),
        ai_brains_path::CreateMode::Replace,
    )
    .map_err(|e| {
        format!(
            "failed to write shadow-manifest {}: {e}",
            manifest_path.display()
        )
    })?;

    println!(
        "Shadow vault created at {} ({} event(s), redaction={})",
        destination.display(),
        written,
        redaction_policy
    );
    println!("Manifest written to {}", manifest_path.display());
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;
    use ai_brains_core::privacy::Privacy;
    use ai_brains_events::{Actor, AggregateType, EventKind};
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[test]
    fn redact_turn_content__unknown_payload__preserves_hash_and_fields() {
        let raw = serde_json::json!({
            "type": "TotallyFutureEvent",
            "foo": 1,
            "bar": "x"
        });
        let original_hash = "preserve-me-hash";
        let envelope = Envelope {
            event_id: Uuid::from_u128(1),
            schema_version: 1,
            aggregate_type: AggregateType::System,
            aggregate_id: Uuid::nil(),
            event_type: EventKind::Unknown("TotallyFutureEvent".to_string()),
            occurred_at: OffsetDateTime::from_unix_timestamp(1_700_000_000)
                .expect("valid timestamp for test"),
            actor: Actor::System,
            causation_id: None,
            correlation_id: None,
            privacy: Privacy::LocalOnly,
            payload: Payload::Unknown(raw.clone()),
            payload_hash: original_hash.to_string(),
        };

        let out = redact_turn_content(envelope).expect("redact should succeed");
        assert_eq!(out.payload_hash, original_hash);
        match out.payload {
            Payload::Unknown(v) => assert_eq!(v, raw),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    /// Codex R5 — dangling symlink dest (exists()==false) must still refuse.
    #[test]
    fn refuse_unsafe_destination__dangling_symlink__refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("source.db");
        let dest = dir.path().join("dest-dangling.db");
        std::fs::write(&source, b"x").expect("source");
        let missing_target = dir.path().join("missing-target-does-not-exist.db");

        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&missing_target, &dest);
        #[cfg(not(windows))]
        let created = std::os::unix::fs::symlink(&missing_target, &dest);

        if let Err(e) = created {
            eprintln!(
                "skipping refuse_unsafe_destination__dangling_symlink__refuses: {e} \
                 (needs Developer Mode or elevation on Windows)"
            );
            return;
        }
        assert!(
            !dest.exists(),
            "precondition: dangling symlink must have exists()==false"
        );
        assert!(
            is_reparse_or_symlink(&dest).expect("metadata"),
            "precondition: dangling symlink must be detected as reparse"
        );

        let err = refuse_unsafe_destination(&source, &dest, None).expect_err("must refuse");
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "expected reparse refuse for dangling dest symlink, got: {err}"
        );
    }
}
