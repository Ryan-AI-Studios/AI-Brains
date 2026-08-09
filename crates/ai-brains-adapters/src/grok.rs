//! Grok Build seamless ingest (T237): percent path codec, chat_history resolve,
//! message-only import, subagent skip, capability honesty.

use crate::agy::path_derived_display_name;
use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::errors::{AdapterError, Result};
use crate::message_only::{IngestableTurn, filter_grok_history_lines};
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Stable Grok unbound project alias (hook + batch share this SOOT).
pub const GROK_UNBOUND_ALIAS: &str = "grok-unbound";

/// Display name for the shared unbound Grok project.
pub const GROK_UNBOUND_DISPLAY_NAME: &str = "(unbound Grok)";

/// Canonical Grok harness UUID (next after AGY 0002).
pub const GROK_HARNESS_UUID: &str = "00000000-0000-0000-0000-000000000003";

/// RFC-3986 unreserved set used by Grok session group folder encoding.
fn is_unreserved(c: u8) -> bool {
    matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

/// Percent-encode a path component for Grok session group names (F7 / AC19).
///
/// Unreserved `A-Za-z0-9-._~` stay as-is; all other bytes → uppercase `%XX`.
/// Matches live encodings such as `C:\dev\AI-Brains` → `C%3A%5Cdev%5CAI-Brains`.
pub fn percent_encode_path_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for &b in s.as_bytes() {
        if is_unreserved(b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
            out.push(char::from(b"0123456789ABCDEF"[(b & 0x0f) as usize]));
        }
    }
    out
}

/// Percent-decode a component; returns `None` on malformed `%` sequences.
pub fn percent_decode_component(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return None;
                }
                let hi = from_hex(bytes[i + 1])?;
                let lo = from_hex(bytes[i + 2])?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Resolve `chat_history.jsonl` for a session under `grok_home` (F7 / AC20).
///
/// Order:
/// 1. For each of `workspace_root` then `cwd` (if Some non-empty): try
///    `sessions/<percent_encode(ws)>/<sessionId>/chat_history.jsonl`
/// 2. Scan `sessions/*/` for `.cwd` whose content trims-equals normalized workspace/cwd
/// 3. Scan `sessions/*/*/summary.json` for `info.id == sessionId` (or top-level `id`)
/// 4. None → fail-open
pub fn resolve_chat_history_path(
    grok_home: &Path,
    session_id: &str,
    workspace_root: Option<&str>,
    cwd: Option<&str>,
) -> Option<PathBuf> {
    let sessions = grok_home.join("sessions");
    if !sessions.is_dir() {
        return None;
    }

    let mut candidates: Vec<String> = Vec::new();
    for raw in [workspace_root, cwd].into_iter().flatten() {
        let t = raw.trim();
        if !t.is_empty() && !candidates.iter().any(|c| c == t) {
            candidates.push(t.to_string());
        }
    }

    // (1) Direct encode hit
    for ws in &candidates {
        let enc = percent_encode_path_component(ws);
        let path = sessions
            .join(&enc)
            .join(session_id)
            .join("chat_history.jsonl");
        if path.is_file() {
            return Some(path);
        }
    }

    // Normalize candidates for `.cwd` compare
    let normalized_candidates: Vec<String> = candidates
        .iter()
        .map(|c| normalize_path_for_compare(c))
        .collect();

    // (2) Scan groups for `.cwd` match
    if !normalized_candidates.is_empty()
        && let Ok(rd) = std::fs::read_dir(&sessions)
    {
        for entry in rd.flatten() {
            let group = entry.path();
            if !group.is_dir() {
                continue;
            }
            let cwd_file = group.join(".cwd");
            if !cwd_file.is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&cwd_file) else {
                continue;
            };
            let file_norm = normalize_path_for_compare(content.trim());
            if normalized_candidates.iter().any(|c| c == &file_norm) {
                let path = group.join(session_id).join("chat_history.jsonl");
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }

    // (3) Scan summary.json for session id
    if let Ok(groups) = std::fs::read_dir(&sessions) {
        for group_ent in groups.flatten() {
            let group = group_ent.path();
            if !group.is_dir() {
                continue;
            }
            if let Ok(sessions_rd) = std::fs::read_dir(&group) {
                for sess_ent in sessions_rd.flatten() {
                    let sess_dir = sess_ent.path();
                    if !sess_dir.is_dir() {
                        continue;
                    }
                    let summary = sess_dir.join("summary.json");
                    if !summary.is_file() {
                        continue;
                    }
                    if summary_matches_session(&summary, session_id) {
                        let path = sess_dir.join("chat_history.jsonl");
                        if path.is_file() {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    None
}

fn normalize_path_for_compare(raw: &str) -> String {
    let trimmed = raw.trim();
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.replace('/', "\\").to_ascii_lowercase(),
    }
}

fn summary_matches_session(summary_path: &Path, session_id: &str) -> bool {
    let Ok(raw) = std::fs::read_to_string(summary_path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    if v.get("info")
        .and_then(|i| i.get("id"))
        .and_then(|id| id.as_str())
        == Some(session_id)
    {
        return true;
    }
    v.get("id").and_then(|id| id.as_str()) == Some(session_id)
}

/// Parse a Grok `chat_history.jsonl` file into message-only turns (F11 filter).
pub fn parse_chat_history_file(path: &Path) -> Result<Vec<IngestableTurn>> {
    let content = std::fs::read_to_string(path)?;
    Ok(filter_grok_history_lines(&content))
}

/// Deterministic turn id SOOT: `v5(session, "turn-{i}")` on kept index (F8).
pub fn generate_grok_turn_id(session_id: &SessionId, kept_index: usize) -> TurnId {
    let name = format!("turn-{kept_index}");
    TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
}

/// Normalize a Grok project hash / workspace string for alias keys.
pub fn normalize_grok_project_hash(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(GROK_UNBOUND_ALIAS) {
        return GROK_UNBOUND_ALIAS.to_string();
    }
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Whether `AI_BRAINS_PROJECT_ID` env fallback is allowed for this hash.
pub fn grok_env_fallback_allowed(raw: &str) -> bool {
    let t = raw.trim();
    t.is_empty() || t.eq_ignore_ascii_case(GROK_UNBOUND_ALIAS)
}

/// Path-keyed source_meta: `source_meta:grok:{sha256_hex(normalized_path)}`.
pub fn grok_source_meta_key(path: &Path) -> String {
    let key_material = match ai_brains_path::normalize_project_path(&path.to_string_lossy()) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    format!("source_meta:grok:{}", hex::encode(hasher.finalize()))
}

/// True when a session path/summary looks like a subagent / worktree session (F12 / AC18).
pub fn is_subagent_session(path: &Path, summary: Option<&Value>) -> bool {
    let path_s = path.to_string_lossy();
    let path_lower = path_s.to_ascii_lowercase();
    if path_lower.contains("subagent-") {
        return true;
    }
    // worktrees segment: `.grok\worktrees`, `/worktrees/`, `\worktrees\`
    if path_lower.contains("\\worktrees\\")
        || path_lower.contains("/worktrees/")
        || path_lower.contains(".grok\\worktrees")
        || path_lower.contains(".grok/worktrees")
    {
        return true;
    }
    if let Some(s) = summary
        && let Some(name) = s.get("agent_name").and_then(|v| v.as_str())
    {
        let n = name.trim();
        if !n.is_empty() && !n.eq_ignore_ascii_case("main") {
            return true;
        }
    }
    false
}

/// Grok adapter capability (F27 honesty notes).
pub fn grok_capability() -> AdapterCapability {
    AdapterCapability {
        name: "grok".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: false,
        notes: "Live Stop+SessionEnd via `ai-brains harness install --harness grok` (empty Stop stdout — never AGY allow JSON). Batch `grok-import` walks ~/.grok/sessions/**/chat_history.jsonl. User keep: non-empty <user_query>/<USER_REQUEST> only (F11); subagent/worktree sessions skipped by default; source_ts usually none (occurred_at=ingest-time); never updates.jsonl. Turn ids = v5(session,\"turn-{i}\") on kept index — filter taxonomy changes can shift ids (duplicates risk). Grok may also load Claude/Cursor hooks (vendor-compat dual-fire). SYSTEM scheduled nightly may still --skip-import (T239)."
            .to_string(),
        governed_reads: Vec::new(),
        governed_writes: Vec::new(),
        principal_binding: None,
    }
}

/// Options for [`import_grok_sessions`].
#[derive(Debug, Clone)]
pub struct GrokImportOptions {
    pub days: usize,
    pub default_project_id: ProjectId,
    /// When false (default), unbound sessions do not attach to default_project_id.
    pub allow_default_project: bool,
    /// Skip the 300s quiescence window.
    pub force: bool,
    /// Hermetic tests: discover under this home instead of dirs::home_dir / GROK_HOME.
    pub home_override: Option<PathBuf>,
}

impl GrokImportOptions {
    pub fn new(days: usize, default_project_id: ProjectId) -> Self {
        Self {
            days,
            default_project_id,
            allow_default_project: false,
            force: false,
            home_override: None,
        }
    }
}

/// Import counters (F17) — printed as human stderr.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GrokImportStats {
    pub found: usize,
    pub imported_turns: usize,
    pub sessions: usize,
    pub skipped_quiescent: usize,
    pub skipped_unchanged: usize,
    pub skipped_subagent: usize,
    pub unbound_project: usize,
    pub bound_via_summary: usize,
    pub bound_via_path: usize,
}

/// A discovered Grok chat_history session source.
#[derive(Debug, Clone)]
pub struct GrokSessionSource {
    pub path: PathBuf,
    pub session_id: String,
    /// Binding hash from summary or decoded group folder (may be unbound).
    pub project_hash: Option<String>,
    pub bind_via_summary: bool,
}

/// Resolve Grok home: home_override → GROK_HOME → ~/.grok.
pub fn resolve_grok_home(home_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(h) = home_override {
        return Some(h.join(".grok"));
    }
    if let Ok(g) = std::env::var("GROK_HOME") {
        let t = g.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    dirs::home_dir().map(|h| h.join(".grok"))
}

/// Discover `sessions/**/chat_history.jsonl` under grok home (ignores `*.lock`).
pub fn discover_grok_sessions(grok_home: &Path) -> Result<Vec<GrokSessionSource>> {
    let sessions = grok_home.join("sessions");
    let mut out = Vec::new();
    if !sessions.is_dir() {
        return Ok(out);
    }
    walk_for_chat_history(&sessions, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_for_chat_history(dir: &Path, out: &mut Vec<GrokSessionSource>) -> Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".lock") {
            continue;
        }
        if path.is_dir() {
            walk_for_chat_history(&path, out)?;
            continue;
        }
        if name.eq_ignore_ascii_case("chat_history.jsonl")
            && let Some(source) = source_from_history_path(&path)
        {
            out.push(source);
        }
    }
    Ok(())
}

fn source_from_history_path(history: &Path) -> Option<GrokSessionSource> {
    // .../sessions/<group>/<sessionId>/chat_history.jsonl
    let sess_dir = history.parent()?;
    let session_id = sess_dir.file_name()?.to_str()?.to_string();
    if session_id.is_empty() {
        return None;
    }
    let group_dir = sess_dir.parent();
    let summary_path = sess_dir.join("summary.json");
    let summary_val = std::fs::read_to_string(&summary_path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());

    let mut project_hash: Option<String> = None;
    let mut bind_via_summary = false;
    if let Some(ref s) = summary_val {
        if let Some(gr) = s.get("git_root_dir").and_then(|v| v.as_str()) {
            let t = gr.trim();
            if !t.is_empty() {
                project_hash = Some(t.to_string());
                bind_via_summary = true;
            }
        }
        if project_hash.is_none()
            && let Some(cwd) = s
                .get("info")
                .and_then(|i| i.get("cwd"))
                .and_then(|v| v.as_str())
        {
            let t = cwd.trim();
            if !t.is_empty() {
                project_hash = Some(t.to_string());
                bind_via_summary = true;
            }
        }
    }
    if project_hash.is_none()
        && let Some(group) = group_dir
        && let Some(name) = group.file_name().and_then(|n| n.to_str())
        && let Some(decoded) = percent_decode_component(name)
    {
        // Prefer decoded path when it looks path-like
        if decoded.contains('\\') || decoded.contains('/') || decoded.contains(':') {
            project_hash = Some(decoded);
        }
    }

    Some(GrokSessionSource {
        path: history.to_path_buf(),
        session_id,
        project_hash,
        bind_via_summary,
    })
}

/// How a Grok session was bound to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrokBindKind {
    Summary,
    Path,
    Unbound,
    Default,
}

/// Resolve Grok project hash to a project id (shared hook + batch).
pub fn resolve_grok_project(
    project_hash: Option<&str>,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, GrokBindKind, bool)> {
    let raw = project_hash.unwrap_or("").trim();
    let alias = normalize_grok_project_hash(raw);

    if alias == GROK_UNBOUND_ALIAS {
        if allow_default_project {
            return Ok((
                default_project_id,
                GROK_UNBOUND_ALIAS.to_string(),
                GrokBindKind::Default,
                false,
            ));
        }
        if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(GROK_UNBOUND_ALIAS) {
            return Ok((
                pid,
                GROK_UNBOUND_ALIAS.to_string(),
                GrokBindKind::Unbound,
                false,
            ));
        }
        return Ok((
            ProjectId::new(),
            GROK_UNBOUND_ALIAS.to_string(),
            GrokBindKind::Unbound,
            true,
        ));
    }

    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(&alias) {
        return Ok((pid, alias, GrokBindKind::Path, false));
    }

    Ok((ProjectId::new(), alias, GrokBindKind::Path, true))
}

fn ensure_project_registered<S: CaptureSink>(
    sink: &mut S,
    project_id: ProjectId,
    alias: &str,
    display_name: &str,
    query_store: &dyn ai_brains_store::QueryStore,
) -> Result<()> {
    if let Ok(Some(_)) = query_store.resolve_project_id_from_alias(alias) {
        return Ok(());
    }

    let actor = Actor::User(UserId::new());
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor.clone(),
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: display_name.to_string(),
        tx_id: None,
    }))
    .map_err(|e| AdapterError::Other(format!("ProjectRegistered build: {e}")))?;
    sink.append(reg);

    let alias_ev = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
        project_id,
        alias: alias.to_string(),
    }))
    .map_err(|e| AdapterError::Other(format!("ProjectAliasAdded build: {e}")))?;
    sink.append(alias_ev);
    Ok(())
}

/// Orchestrate import of Grok sessions from discovered chat_history files.
pub fn import_grok_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: GrokImportOptions,
) -> Result<GrokImportStats> {
    let mut stats = GrokImportStats::default();
    let grok_home = match resolve_grok_home(options.home_override.as_deref()) {
        Some(h) => h,
        None => return Ok(stats),
    };

    let all_sources = discover_grok_sessions(&grok_home)?;
    if all_sources.is_empty() {
        return Ok(stats);
    }

    let cutoff = SystemTime::now() - Duration::from_secs(options.days as u64 * 24 * 60 * 60);

    let mut recent: Vec<GrokSessionSource> = Vec::new();
    for source in all_sources {
        // Subagent skip (F12) — load summary if present
        let summary_path = source
            .path
            .parent()
            .map(|p| p.join("summary.json"))
            .filter(|p| p.is_file());
        let summary_val = summary_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
        if is_subagent_session(&source.path, summary_val.as_ref()) {
            stats.skipped_subagent += 1;
            continue;
        }

        if let Ok(metadata) = std::fs::metadata(&source.path)
            && let Ok(modified) = metadata.modified()
        {
            if modified < cutoff {
                continue;
            }
            recent.push(source);
        }
    }

    stats.found = recent.len();
    if stats.found == 0 {
        return Ok(stats);
    }
    eprintln!(
        "[Grok] Found {} sessions modified in the last {} days. Scanning for new turns...",
        stats.found, options.days
    );

    let grok_harness = HarnessId::from_str(GROK_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Grok harness ID: {e}")))?;

    for (idx, source) in recent.iter().enumerate() {
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

        let meta_key = grok_source_meta_key(&source.path);
        let stored_meta = query_store.get_sync_state(&meta_key).unwrap_or(None);
        let current_meta = format!("{mtime}:{size}");

        if stored_meta.as_ref() == Some(&current_meta) {
            stats.skipped_unchanged += 1;
            continue;
        }

        if (idx + 1) % 10 == 0 || idx == 0 || idx == stats.found - 1 {
            eprintln!("[Grok] Scanning session {}/{}...", idx + 1, stats.found);
        }

        if !options.force
            && let Some(modified) = metadata.as_ref().and_then(|m| m.modified().ok())
            && SystemTime::now()
                .duration_since(modified)
                .unwrap_or(Duration::ZERO)
                < Duration::from_secs(300)
        {
            stats.skipped_quiescent += 1;
            continue;
        }

        let session_uuid = match Uuid::parse_str(&source.session_id) {
            Ok(id) => id,
            Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_URL, source.session_id.as_bytes()),
        };
        let session_id = SessionId::from_uuid(session_uuid);

        let turns = parse_chat_history_file(&source.path)?;
        if turns.is_empty() {
            update_source_meta(sink, &meta_key, &current_meta);
            continue;
        }

        let max_turn = query_store.get_max_turn_index(&session_id).unwrap_or(None);
        let next_index = max_turn.map(|m| m + 1).unwrap_or(0);
        if turns.len() <= next_index as usize {
            update_source_meta(sink, &meta_key, &current_meta);
            continue;
        }

        let (mut project_id, alias, resolved_kind, needs_create) = resolve_grok_project(
            source.project_hash.as_deref(),
            query_store,
            options.allow_default_project,
            options.default_project_id,
        )?;

        let final_kind = if source.bind_via_summary && resolved_kind != GrokBindKind::Unbound {
            GrokBindKind::Summary
        } else if resolved_kind == GrokBindKind::Default {
            GrokBindKind::Default
        } else if resolved_kind == GrokBindKind::Unbound || alias == GROK_UNBOUND_ALIAS {
            GrokBindKind::Unbound
        } else {
            GrokBindKind::Path
        };

        if needs_create {
            let display = if alias == GROK_UNBOUND_ALIAS {
                GROK_UNBOUND_DISPLAY_NAME.to_string()
            } else {
                path_derived_display_name(&alias)
            };
            if let Ok(Some(existing)) = query_store.resolve_project_id_from_alias(&alias) {
                project_id = existing;
            } else {
                ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
            }
        } else if final_kind != GrokBindKind::Default
            && query_store
                .resolve_project_id_from_alias(&alias)
                .ok()
                .flatten()
                .is_none()
        {
            let display = if alias == GROK_UNBOUND_ALIAS {
                GROK_UNBOUND_DISPLAY_NAME.to_string()
            } else {
                path_derived_display_name(&alias)
            };
            ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
        }

        match final_kind {
            GrokBindKind::Summary => stats.bound_via_summary += 1,
            GrokBindKind::Path => stats.bound_via_path += 1,
            GrokBindKind::Unbound => stats.unbound_project += 1,
            GrokBindKind::Default => {}
        }

        let capture_context = CaptureContext {
            git_working_dir: std::env::current_dir().ok(),
        };

        service.start_session(
            ai_brains_capture::SessionStartCommand {
                session_id,
                project_id,
                harness_id: grok_harness,
                privacy: Privacy::LocalOnly,
                tx_id: None,
            },
            capture_context.clone(),
            sink,
        )?;

        for (i, turn) in turns.iter().enumerate().skip(next_index as usize) {
            let turn_id = generate_grok_turn_id(&session_id, i);
            let request = IngestRequest {
                session_id,
                project_id,
                harness_id: grok_harness,
                turn_id,
                role: turn.role.as_str().to_string(),
                content: turn.content.clone(),
                privacy: Privacy::LocalOnly,
                thinking: None,
                tx_id: None,
            };
            service.ingest_request(request, capture_context.clone(), sink)?;
            stats.imported_turns += 1;
        }

        service.stop_session(
            ai_brains_capture::SessionStopCommand {
                session_id,
                harness_id: grok_harness,
                privacy: Privacy::LocalOnly,
                status: SessionStopStatus::Completed,
                reason: Some("Grok chat_history import complete".to_string()),
            },
            capture_context,
            sink,
        )?;

        update_source_meta(sink, &meta_key, &current_meta);
        stats.sessions += 1;
    }

    Ok(stats)
}

/// Print F17 human stats to stderr.
pub fn print_grok_import_stats(stats: &GrokImportStats) {
    eprintln!(
        "[Grok] Import stats: found={} imported_turns={} sessions={} skipped_quiescent={} skipped_unchanged={} skipped_subagent={} unbound_project={} bound_via_summary={} bound_via_path={}",
        stats.found,
        stats.imported_turns,
        stats.sessions,
        stats.skipped_quiescent,
        stats.skipped_unchanged,
        stats.skipped_subagent,
        stats.unbound_project,
        stats.bound_via_summary,
        stats.bound_via_path
    );
}

fn update_source_meta<S: CaptureSink>(sink: &mut S, key: &str, value: &str) {
    sink.set_sync_state(key, value);
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn percent_encode__live_ai_brains_path() {
        // AC19
        let enc = percent_encode_path_component(r"C:\dev\AI-Brains");
        assert_eq!(enc, "C%3A%5Cdev%5CAI-Brains");
        let dec = percent_decode_component(&enc).expect("decode");
        assert_eq!(dec, r"C:\dev\AI-Brains");
    }

    #[test]
    fn percent_encode__space_and_unreserved() {
        assert_eq!(percent_encode_path_component("a b"), "a%20b");
        assert_eq!(percent_encode_path_component("A-Z_9.~"), "A-Z_9.~");
    }

    #[test]
    fn percent_decode__malformed__none() {
        assert!(percent_decode_component("%").is_none());
        assert!(percent_decode_component("%G0").is_none());
    }

    #[test]
    fn resolve_chat_history__direct_encode_hit() {
        let dir = tempdir().expect("tempdir");
        let grok = dir.path().join(".grok");
        let ws = r"C:\dev\AI-Brains";
        let enc = percent_encode_path_component(ws);
        let sid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let hist = grok
            .join("sessions")
            .join(&enc)
            .join(sid)
            .join("chat_history.jsonl");
        std::fs::create_dir_all(hist.parent().unwrap()).unwrap();
        std::fs::write(&hist, "{}\n").unwrap();

        let found = resolve_chat_history_path(&grok, sid, Some(ws), None).expect("resolve");
        assert_eq!(found, hist);
    }

    #[test]
    fn resolve_chat_history__cwd_file_fallback() {
        // AC20
        let dir = tempdir().expect("tempdir");
        let grok = dir.path().join(".grok");
        let sid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let group = grok.join("sessions").join("slug-hash-group");
        let hist = group.join(sid).join("chat_history.jsonl");
        std::fs::create_dir_all(hist.parent().unwrap()).unwrap();
        std::fs::write(group.join(".cwd"), r"C:\dev\AI-Brains").unwrap();
        std::fs::write(&hist, "{}\n").unwrap();

        let found =
            resolve_chat_history_path(&grok, sid, Some(r"C:\dev\AI-Brains"), None).expect("cwd");
        assert_eq!(found, hist);
    }

    #[test]
    fn resolve_chat_history__summary_id_fallback() {
        // AC20
        let dir = tempdir().expect("tempdir");
        let grok = dir.path().join(".grok");
        let sid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        // summary.info.id matches even when workspace encode misses
        let sess = grok.join("sessions").join("other-group").join(sid);
        let hist = sess.join("chat_history.jsonl");
        std::fs::create_dir_all(&sess).unwrap();
        std::fs::write(
            sess.join("summary.json"),
            format!(r#"{{"info":{{"id":"{sid}","cwd":"D:\\elsewhere"}}}}"#),
        )
        .unwrap();
        std::fs::write(&hist, "{}\n").unwrap();

        let found =
            resolve_chat_history_path(&grok, sid, Some(r"C:\no-match"), None).expect("summary");
        assert_eq!(found, hist);
    }

    #[test]
    fn is_subagent_session__path_and_agent_name() {
        let p = Path::new(r"C:\Users\x\.grok\sessions\subagent-worker\sid\chat_history.jsonl");
        assert!(is_subagent_session(p, None));
        let p2 = Path::new(r"C:\Users\x\.grok\worktrees\foo\sessions\g\sid\chat_history.jsonl");
        assert!(is_subagent_session(p2, None));
        let main = Path::new(r"C:\Users\x\.grok\sessions\C%3A\sid\chat_history.jsonl");
        assert!(!is_subagent_session(main, None));
        let summary = serde_json::json!({"agent_name": "researcher"});
        assert!(is_subagent_session(main, Some(&summary)));
        let summary_main = serde_json::json!({"agent_name": "main"});
        assert!(!is_subagent_session(main, Some(&summary_main)));
    }

    #[test]
    fn generate_grok_turn_id__stable() {
        let sid =
            SessionId::from_uuid(Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap());
        let a = generate_grok_turn_id(&sid, 0);
        let b = generate_grok_turn_id(&sid, 0);
        assert_eq!(a, b);
        assert_ne!(
            generate_grok_turn_id(&sid, 0),
            generate_grok_turn_id(&sid, 1)
        );
    }

    #[test]
    fn normalize_grok_project_hash__unbound_and_case() {
        assert_eq!(normalize_grok_project_hash(""), GROK_UNBOUND_ALIAS);
        assert_eq!(
            normalize_grok_project_hash("grok-unbound"),
            GROK_UNBOUND_ALIAS
        );
        let a = normalize_grok_project_hash(r"C:\dev\Dedupe");
        let b = normalize_grok_project_hash(r"c:\dev\dedupe");
        assert_eq!(a, b);
    }

    #[test]
    fn grok_env_fallback_allowed__only_unbound() {
        assert!(grok_env_fallback_allowed(""));
        assert!(grok_env_fallback_allowed("grok-unbound"));
        assert!(!grok_env_fallback_allowed(r"C:\dev\proj"));
    }

    #[test]
    fn discover_and_import__subagent_skipped() {
        // AC18
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let grok = home.join(".grok");
        let sid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        let hist = grok
            .join("sessions")
            .join("subagent-role")
            .join(sid)
            .join("chat_history.jsonl");
        std::fs::create_dir_all(hist.parent().unwrap()).unwrap();
        std::fs::write(
            &hist,
            r#"{"type":"user","content":"<user_query>\nhi\n</user_query>"}
{"type":"assistant","content":"yo"}
"#,
        )
        .unwrap();
        // Touch mtime is now — within days

        let sources = discover_grok_sessions(&grok).expect("discover");
        assert_eq!(sources.len(), 1);
        assert!(is_subagent_session(&sources[0].path, None));
    }

    #[test]
    fn parse_chat_history_file__message_only() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("chat_history.jsonl");
        std::fs::write(
            &path,
            r#"{"type":"user","content":"<user_query>\nkeep\n</user_query>"}
{"type":"reasoning","content":"drop"}
{"type":"assistant","content":"answer"}
{"type":"user","content":"bare drop"}
"#,
        )
        .unwrap();
        let turns = parse_chat_history_file(&path).unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].content, "keep");
        assert_eq!(turns[1].content, "answer");
    }

    #[test]
    fn grok_capability__full_with_honesty() {
        let c = grok_capability();
        assert_eq!(c.level, CapabilityLevel::Full);
        assert!(c.notes.contains("user_query"));
        assert!(c.notes.contains("empty Stop stdout") || c.notes.contains("empty Stop"));
        assert!(c.notes.contains("subagent"));
    }
}
