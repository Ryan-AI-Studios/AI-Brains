//! Cursor IDE batch ingest (T334): fail-open `cursor-import` of
//! `~/.cursor/projects/<slug>/agent-transcripts/{uuid}/{uuid}.jsonl`
//! and flat `{uuid}.jsonl` (not arbitrary nested JSONL).
//!
//! Message-only: keep user/assistant text after [`filter_turn`]. Skip `subagents/`
//! (same class as Claude). Do **not** open Composer `state.vscdb`. No live hooks.

use crate::agy::path_derived_display_name;
use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::claude::source_meta_key;
use crate::errors::{AdapterError, Result};
use crate::message_only::{IngestableTurn, extract_text_from_json_content, filter_turn};
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Stable Cursor unbound project alias (batch import).
pub const CURSOR_UNBOUND_ALIAS: &str = "cursor-unbound";

/// Display name for the shared unbound Cursor project.
pub const CURSOR_UNBOUND_DISPLAY_NAME: &str = "(unbound Cursor)";

/// Canonical Cursor harness UUID (next after Codex `...0006`).
pub const CURSOR_HARNESS_UUID: &str = "00000000-0000-0000-0000-000000000007";

/// Message-only turn from Cursor JSONL (no per-line uuid).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorIngestTurn {
    pub turn: IngestableTurn,
}

/// How a Cursor session was bound to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorBindKind {
    Path,
    Unbound,
    Default,
}

/// Options for [`import_cursor_sessions`].
#[derive(Debug, Clone)]
pub struct CursorImportOptions {
    pub days: usize,
    pub default_project_id: ProjectId,
    pub allow_default_project: bool,
    pub force: bool,
    pub home_override: Option<PathBuf>,
    pub dry_run: bool,
}

impl CursorImportOptions {
    pub fn new(days: usize, default_project_id: ProjectId) -> Self {
        Self {
            days,
            default_project_id,
            allow_default_project: false,
            force: false,
            home_override: None,
            dry_run: false,
        }
    }
}

/// Import counters printed as human stderr.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CursorImportStats {
    pub found: usize,
    pub imported_turns: usize,
    pub sessions: usize,
    pub skipped_quiescent: usize,
    pub skipped_unchanged: usize,
    pub skipped_sidechain: usize,
    pub skipped_query: usize,
    pub unbound_project: usize,
    pub bound_via_path: usize,
}

/// A discovered Cursor agent-transcripts JSONL session.
#[derive(Debug, Clone)]
pub struct CursorSessionSource {
    pub path: PathBuf,
    pub session_id: String,
    /// Cursor `projects/<folder>` name (may be mixed-case, e.g. `c-dev-AI-Brains`).
    pub project_folder: String,
}

pub fn cursor_capability() -> AdapterCapability {
    AdapterCapability {
        name: "cursor".to_string(),
        level: CapabilityLevel::Partial,
        supports_hooks: false,
        supports_wrapper_mode: false,
        notes: "Batch JSONL only (`ai-brains cursor-import` / nightly sixth source). Walks ~/.cursor/projects/<slug>/agent-transcripts (nested uuid folders and flat jsonl). Message-only after filter_turn; skips subagents/; does not open Composer state.vscdb. No live hooks, no harness install --harness cursor, no doctor sixth row. Unbound alias cursor-unbound. Bind is case-insensitive slug of list_path_aliases (not hyphen reverse-decode)."
            .to_string(),
        governed_reads: Vec::new(),
        governed_writes: Vec::new(),
        principal_binding: None,
    }
}

/// Map a **normalized** project path to a Cursor `projects/<folder>` slug.
///
/// Trim, then drop leading/trailing `/` only (Unix Cursor folders omit the
/// root separator). Then drop `:`, map `\`/`/` → `-`, ascii-lowercase.
/// Do **not** strip `\` or post-map `-` (drive root `C:\` → `c-`; UNC stays
/// `--server-share`).
///
/// `/Users/foo/dev/AI-Brains` → `users-foo-dev-ai-brains` (matches folder
/// `Users-foo-dev-AI-Brains` via `eq_ignore_ascii_case`).
/// `C:\dev\ai-brains` → `c-dev-ai-brains`.
pub fn cursor_project_slug(normalized_path: &str) -> String {
    let trimmed = normalized_path
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let mut out = String::new();
    for ch in trimmed.chars() {
        match ch {
            ':' => {}
            '\\' | '/' => out.push('-'),
            c => out.push(c.to_ascii_lowercase()),
        }
    }
    out
}

/// Cursor folder-name candidates for one stored alias: T341 primary slug plus
/// at most one WSL ↔ Windows drive twin (`/mnt/<letter>/…` ↔ `X:\…`).
///
/// Bind and coverage match if **any** candidate equals the folder
/// (`eq_ignore_ascii_case`). Does **not** change [`cursor_project_slug`].
pub fn cursor_project_slug_candidates(normalized_path: &str) -> Vec<String> {
    let mut set = BTreeSet::new();
    set.insert(cursor_project_slug(normalized_path));
    if let Ok(win) = ai_brains_path::wsl_to_windows(normalized_path) {
        set.insert(cursor_project_slug(&win));
    }
    if let Ok(mnt) = ai_brains_path::windows_drive_to_wsl_mount(normalized_path) {
        set.insert(cursor_project_slug(&mnt));
    }
    set.into_iter().collect()
}

/// Filter one Cursor JSONL record (`role` + `message.content[]`; drop `turn_ended`).
pub fn filter_cursor_jsonl_record(record: &Value) -> Option<CursorIngestTurn> {
    if record
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|t| t.eq_ignore_ascii_case("turn_ended"))
    {
        return None;
    }
    let role = record.get("role").and_then(Value::as_str)?;
    let raw = record
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(extract_text_from_json_content)
        .unwrap_or_default();
    let turn = filter_turn(role, &raw)?;
    Some(CursorIngestTurn { turn })
}

/// Filter Cursor agent-transcripts JSONL text (malformed lines skipped).
pub fn filter_cursor_jsonl_lines(jsonl: &str) -> Vec<CursorIngestTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(turn) = filter_cursor_jsonl_record(&value) {
            out.push(turn);
        }
    }
    out
}

/// Parse a Cursor JSONL file into message-only turns.
pub fn parse_cursor_jsonl_file(path: &Path) -> Result<Vec<CursorIngestTurn>> {
    let content = std::fs::read_to_string(path)?;
    Ok(filter_cursor_jsonl_lines(&content))
}

/// Batch turn id: `v5(session, "turn-{kept_index}")`.
pub fn generate_cursor_turn_id(session_id: &SessionId, kept_index: usize) -> TurnId {
    let name = format!("turn-{kept_index}");
    TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
}

/// Map a Cursor session string to a stable [`SessionId`].
pub fn session_id_from_cursor(raw: &str) -> SessionId {
    let t = raw.trim();
    if let Ok(u) = Uuid::parse_str(t) {
        return SessionId::from_uuid(u);
    }
    SessionId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, t.as_bytes()))
}

/// Path-keyed source_meta: `source_meta:cursor:{sha256_hex(normalized_path)}`.
pub fn cursor_source_meta_key(path: &Path) -> String {
    source_meta_key("cursor", path)
}

/// True when a session path lives under `subagents/`.
pub fn is_cursor_sidechain_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("subagents"))
    })
}

/// Resolve Cursor home: home_override (user home) → `CURSOR_HOME` (the `.cursor` dir) → `~/.cursor`.
pub fn resolve_cursor_home(home_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(h) = home_override {
        return Some(h.join(".cursor"));
    }
    if let Ok(g) = std::env::var("CURSOR_HOME") {
        let t = g.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    dirs::home_dir().map(|h| h.join(".cursor"))
}

/// Discover F5 layouts only: nested `{uuid}/{uuid}.jsonl` and flat `{uuid}.jsonl`
/// under `agent-transcripts/`. Session folder/stem must parse as UUID. Extra
/// nested JSONL and non-UUID names are ignored. `subagents/` is listed so
/// import can count `skipped_sidechain` without ingesting.
pub fn discover_cursor_sessions(cursor_home: &Path) -> Result<Vec<CursorSessionSource>> {
    let projects = cursor_home.join("projects");
    let mut out = Vec::new();
    if !projects.is_dir() {
        return Ok(out);
    }
    let rd = match std::fs::read_dir(&projects) {
        Ok(r) => r,
        Err(_) => return Ok(out),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        if folder.eq_ignore_ascii_case("subagents") {
            continue;
        }
        let transcripts = path.join("agent-transcripts");
        if transcripts.is_dir() {
            walk_cursor_transcripts(&transcripts, &folder, &mut out)?;
        }
    }
    prefer_nested_over_flat_dual_layout(&mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_cursor_transcripts(
    dir: &Path,
    project_folder: &str,
    out: &mut Vec<CursorSessionSource>,
) -> Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if name.eq_ignore_ascii_case("subagents") {
                collect_sidechain_jsonl(&path, project_folder, out)?;
                continue;
            }
            if !is_cursor_session_uuid(&name) {
                continue;
            }
            let nested = path.join(format!("{name}.jsonl"));
            if nested.is_file()
                && let Some(source) = source_from_cursor_jsonl(&nested, project_folder)
            {
                out.push(source);
            }
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".jsonl") {
            continue;
        }
        if let Some(source) = source_from_cursor_jsonl(&path, project_folder)
            && is_cursor_session_uuid(&source.session_id)
        {
            out.push(source);
        }
    }
    Ok(())
}

fn collect_sidechain_jsonl(
    dir: &Path,
    project_folder: &str,
    out: &mut Vec<CursorSessionSource>,
) -> Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            collect_sidechain_jsonl(&path, project_folder, out)?;
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".jsonl") {
            continue;
        }
        if let Some(source) = source_from_cursor_jsonl(&path, project_folder) {
            out.push(source);
        }
    }
    Ok(())
}

fn is_cursor_session_uuid(s: &str) -> bool {
    Uuid::parse_str(s.trim()).is_ok()
}

fn transcript_parent_is_agent_transcripts(path: &Path) -> bool {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("agent-transcripts"))
}

/// Spec nested layout is primary. If the same session exists as both
/// `{uuid}.jsonl` and `{uuid}/{uuid}.jsonl`, keep the nested file only.
fn prefer_nested_over_flat_dual_layout(sources: &mut Vec<CursorSessionSource>) {
    use std::collections::{HashMap, HashSet};
    let mut preferred: HashMap<(String, String), usize> = HashMap::new();
    for (i, src) in sources.iter().enumerate() {
        if is_cursor_sidechain_path(&src.path) {
            continue;
        }
        let key = (
            src.project_folder.to_ascii_lowercase(),
            src.session_id.clone(),
        );
        match preferred.get(&key).copied() {
            None => {
                preferred.insert(key, i);
            }
            Some(prev) => {
                let nested = !transcript_parent_is_agent_transcripts(&src.path);
                let prev_nested = !transcript_parent_is_agent_transcripts(&sources[prev].path);
                if nested && !prev_nested {
                    preferred.insert(key, i);
                }
            }
        }
    }
    let keep: HashSet<usize> = preferred.values().copied().collect();
    let mut kept = Vec::with_capacity(sources.len());
    for (i, src) in sources.drain(..).enumerate() {
        if is_cursor_sidechain_path(&src.path) || keep.contains(&i) {
            kept.push(src);
        }
    }
    *sources = kept;
}

fn source_from_cursor_jsonl(jsonl: &Path, project_folder: &str) -> Option<CursorSessionSource> {
    let stem = jsonl.file_stem()?.to_str()?.to_string();
    if stem.is_empty() {
        return None;
    }
    let parent_name = jsonl
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let session_id = if parent_name.eq_ignore_ascii_case("agent-transcripts") {
        stem
    } else {
        parent_name.to_string()
    };
    if session_id.is_empty() {
        return None;
    }
    Some(CursorSessionSource {
        path: jsonl.to_path_buf(),
        session_id,
        project_folder: project_folder.to_string(),
    })
}

/// Resolve Cursor project folder to a project id via slug match on `list_path_aliases`.
/// After an exact miss, T356 F9 may bind a unique child named in `turn_text`.
pub fn resolve_cursor_project(
    project_folder: &str,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
    turn_text: &str,
) -> Result<(ProjectId, String, CursorBindKind, bool)> {
    let folder = project_folder.trim();
    if folder.is_empty() {
        return unbound_cursor_project(query_store, allow_default_project, default_project_id);
    }

    match query_store.list_path_aliases() {
        Ok(aliases) => {
            for (pid, path) in &aliases {
                if cursor_project_slug_candidates(path)
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(folder))
                {
                    return Ok((*pid, path.clone(), CursorBindKind::Path, false));
                }
            }
            if let Some((pid, path)) = unique_child_from_turns(folder, &aliases, turn_text) {
                return Ok((pid, path, CursorBindKind::Path, false));
            }
        }
        Err(e) => {
            return Err(AdapterError::Other(format!(
                "list_path_aliases failed: {e}"
            )));
        }
    }

    unbound_cursor_project(query_store, allow_default_project, default_project_id)
}

/// Folder slug is a proper prefix (`folder-…`) of ≥1 alias slug; unique turn-text hit wins.
fn unique_child_from_turns(
    folder: &str,
    aliases: &[(ProjectId, String)],
    turn_text: &str,
) -> Option<(ProjectId, String)> {
    let folder_l = folder.to_ascii_lowercase();
    let children: Vec<(ProjectId, String, Vec<String>, String)> = aliases
        .iter()
        .filter_map(|(pid, path)| {
            let slugs = cursor_project_slug_candidates(path);
            let is_child = slugs.iter().any(|s| {
                let s_l = s.to_ascii_lowercase();
                s_l.len() > folder_l.len()
                    && s_l.starts_with(&folder_l)
                    && s_l.as_bytes().get(folder_l.len()) == Some(&b'-')
            });
            if !is_child {
                return None;
            }
            let last = path_last_component(path);
            Some((*pid, path.clone(), slugs, last))
        })
        .collect();
    if children.is_empty() {
        return None;
    }
    let mut hits: Vec<(ProjectId, String)> = Vec::new();
    for (pid, path, slugs, last) in &children {
        let slug_hit = slugs.iter().any(|s| child_token_mentioned(s, turn_text));
        let last_hit = child_token_mentioned(last, turn_text);
        let path_hit = path_token_mentioned(path, turn_text);
        if (slug_hit || last_hit || path_hit) && !hits.iter().any(|(p, _)| p == pid) {
            hits.push((*pid, path.clone()));
        }
    }
    if hits.len() == 1 {
        return hits.pop();
    }
    None
}

fn path_last_component(path: &str) -> String {
    path.replace('\\', "/")
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .to_string()
}

fn child_token_mentioned(token: &str, text: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return false;
    }
    let token_l = token.to_ascii_lowercase();
    let text_l = text.to_ascii_lowercase();
    if token_l.len() < 4 {
        return path_delimited_hit(&text_l, &token_l);
    }
    path_delimited_hit(&text_l, &token_l) || word_boundary_hit(&text_l, &token_l)
}

fn path_token_mentioned(path: &str, text: &str) -> bool {
    let p = path.trim();
    if p.is_empty() {
        return false;
    }
    text.to_ascii_lowercase().contains(&p.to_ascii_lowercase())
}

fn path_delimited_hit(text: &str, slug: &str) -> bool {
    let needles = [
        format!("/{slug}/"),
        format!("\\{slug}\\"),
        format!("/{slug}"),
        format!("\\{slug}"),
    ];
    needles.iter().any(|n| text.contains(n))
}

fn is_slug_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn word_boundary_hit(text: &str, slug: &str) -> bool {
    let hay = text.as_bytes();
    let needle = slug.as_bytes();
    if needle.is_empty() || hay.len() < needle.len() {
        return false;
    }
    let mut i = 0;
    while i + needle.len() <= hay.len() {
        if hay[i..i + needle.len()] == *needle {
            let before_ok = i == 0 || !is_slug_char(hay[i - 1]);
            let after = i + needle.len();
            let after_ok = after == hay.len() || !is_slug_char(hay[after]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn unbound_cursor_project(
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, CursorBindKind, bool)> {
    let alias = CURSOR_UNBOUND_ALIAS.to_string();
    if allow_default_project {
        return Ok((default_project_id, alias, CursorBindKind::Default, false));
    }
    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(CURSOR_UNBOUND_ALIAS) {
        return Ok((pid, alias, CursorBindKind::Unbound, false));
    }
    Ok((ProjectId::new(), alias, CursorBindKind::Unbound, true))
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

/// Shared batch ingest of already-filtered turns (`thinking` always `None`).
pub fn append_cursor_turns<S: CaptureSink>(
    service: &CaptureService,
    sink: &mut S,
    session_id: SessionId,
    project_id: ProjectId,
    turns: &[CursorIngestTurn],
    start_index: usize,
    capture_context: &CaptureContext,
) -> Result<usize> {
    let harness_id = HarnessId::from_str(CURSOR_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Cursor harness ID: {e}")))?;
    let mut count = 0;
    for (i, item) in turns.iter().enumerate().skip(start_index) {
        let turn_id = generate_cursor_turn_id(&session_id, i);
        let request = IngestRequest {
            session_id,
            project_id,
            harness_id,
            turn_id,
            role: item.turn.role.as_str().to_string(),
            content: item.turn.content.clone(),
            privacy: Privacy::LocalOnly,
            thinking: None,
            tx_id: None,
        };
        service.ingest_request(request, capture_context.clone(), sink)?;
        count += 1;
    }
    Ok(count)
}

/// Orchestrate import of Cursor agent-transcripts JSONL sessions.
pub fn import_cursor_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: CursorImportOptions,
) -> Result<CursorImportStats> {
    let mut stats = CursorImportStats::default();
    let cursor_home = match resolve_cursor_home(options.home_override.as_deref()) {
        Some(h) => h,
        None => return Ok(stats),
    };

    let all_sources = discover_cursor_sessions(&cursor_home)?;
    if all_sources.is_empty() {
        return Ok(stats);
    }

    let cutoff = SystemTime::now() - Duration::from_secs(options.days as u64 * 24 * 60 * 60);
    let mut recent: Vec<CursorSessionSource> = Vec::new();
    for source in all_sources {
        if is_cursor_sidechain_path(&source.path) {
            stats.skipped_sidechain += 1;
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
        "[Cursor] Found {} sessions modified in the last {} days. Scanning for new turns...",
        stats.found, options.days
    );
    if options.dry_run {
        eprintln!("[Cursor] dry-run mode: scanning only — no vault writes.");
        for source in &recent {
            eprintln!(
                "[Cursor] dry-run session {} path={}",
                source.session_id,
                source.path.display()
            );
        }
    }

    let cursor_harness = HarnessId::from_str(CURSOR_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Cursor harness ID: {e}")))?;

    for (idx, source) in recent.iter().enumerate() {
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let meta_key = cursor_source_meta_key(&source.path);
        let stored_meta = match query_store.get_sync_state(&meta_key) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[Cursor] skip session {} path={}: sync_state query failed: {e} — continue (fail-open)",
                    source.session_id,
                    source.path.display()
                );
                stats.skipped_query += 1;
                continue;
            }
        };
        let current_meta = format!("{mtime}:{size}");

        if stored_meta.as_ref() == Some(&current_meta) {
            stats.skipped_unchanged += 1;
            continue;
        }

        if (idx + 1) % 10 == 0 || idx == 0 || idx == stats.found.saturating_sub(1) {
            eprintln!("[Cursor] Scanning session {}/{}...", idx + 1, stats.found);
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

        let session_id = session_id_from_cursor(&source.session_id);
        let turns = match parse_cursor_jsonl_file(&source.path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[Cursor] skip session {} path={}: {e} — continue (fail-open)",
                    source.session_id,
                    source.path.display()
                );
                continue;
            }
        };
        if turns.is_empty() {
            if !options.dry_run {
                sink.set_sync_state(&meta_key, &current_meta);
            }
            continue;
        }

        let max_turn = match query_store.get_max_turn_index(&session_id) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[Cursor] skip session {} path={}: max_turn query failed: {e} — continue (fail-open)",
                    source.session_id,
                    source.path.display()
                );
                stats.skipped_query += 1;
                continue;
            }
        };
        let next_index = max_turn.map(|m| m + 1).unwrap_or(0);
        if turns.len() <= next_index as usize {
            if !options.dry_run {
                sink.set_sync_state(&meta_key, &current_meta);
            }
            continue;
        }

        let session_result: crate::errors::Result<()> = (|| {
            let turn_text: String = turns
                .iter()
                .map(|t| t.turn.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let (mut project_id, alias, kind, needs_create) = resolve_cursor_project(
                &source.project_folder,
                query_store,
                options.allow_default_project,
                options.default_project_id,
                &turn_text,
            )?;
            match kind {
                CursorBindKind::Path => stats.bound_via_path += 1,
                CursorBindKind::Unbound => stats.unbound_project += 1,
                CursorBindKind::Default => {}
            }
            if options.dry_run {
                return Ok(());
            }
            if needs_create {
                let display = if alias == CURSOR_UNBOUND_ALIAS {
                    CURSOR_UNBOUND_DISPLAY_NAME.to_string()
                } else {
                    path_derived_display_name(&alias)
                };
                if let Ok(Some(existing)) = query_store.resolve_project_id_from_alias(&alias) {
                    project_id = existing;
                } else {
                    ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
                }
            }
            let capture_context = CaptureContext {
                git_working_dir: std::env::current_dir().ok(),
            };
            service.start_session(
                ai_brains_capture::SessionStartCommand {
                    session_id,
                    project_id,
                    harness_id: cursor_harness,
                    privacy: Privacy::LocalOnly,
                    tx_id: None,
                },
                capture_context.clone(),
                sink,
            )?;
            stats.imported_turns += append_cursor_turns(
                service,
                sink,
                session_id,
                project_id,
                &turns,
                next_index as usize,
                &capture_context,
            )?;
            service.stop_session(
                ai_brains_capture::SessionStopCommand {
                    session_id,
                    harness_id: cursor_harness,
                    privacy: Privacy::LocalOnly,
                    status: SessionStopStatus::Completed,
                    reason: Some("Cursor agent-transcripts JSONL import complete".to_string()),
                },
                capture_context,
                sink,
            )?;
            sink.set_sync_state(&meta_key, &current_meta);
            stats.sessions += 1;
            Ok(())
        })();

        if let Err(e) = session_result {
            eprintln!(
                "[Cursor] session {} path={} failed: {e} — continue (fail-open; prior sessions kept)",
                source.session_id,
                source.path.display()
            );
        }
    }

    Ok(stats)
}

pub fn print_cursor_import_stats(stats: &CursorImportStats) {
    eprintln!(
        "[Cursor] Import stats: found={} imported_turns={} sessions={} skipped_quiescent={} skipped_unchanged={} skipped_sidechain={} skipped_query={} unbound_project={} bound_via_path={}",
        stats.found,
        stats.imported_turns,
        stats.sessions,
        stats.skipped_quiescent,
        stats.skipped_unchanged,
        stats.skipped_sidechain,
        stats.skipped_query,
        stats.unbound_project,
        stats.bound_via_path
    );
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::message_only::IngestRole;

    #[test]
    fn cursor_project_slug__normalized_drive_path__lowercase_hyphen() {
        assert_eq!(cursor_project_slug(r"C:\dev\ai-brains"), "c-dev-ai-brains");
        assert_eq!(
            cursor_project_slug(r"C:\dev\orca\orcaslicer-zr"),
            "c-dev-orca-orcaslicer-zr"
        );
    }

    #[test]
    fn cursor_project_slug__eq_ignore_ascii_case__mixed_folder() {
        let slug = cursor_project_slug(r"C:\dev\ai-brains");
        assert!(slug.eq_ignore_ascii_case("c-dev-AI-Brains"));
        assert!(!slug.eq_ignore_ascii_case("c-dev-Orca"));
    }

    #[test]
    fn cursor_project_slug__unix_absolute_path__no_leading_hyphen() {
        assert_eq!(
            cursor_project_slug("/Users/foo/dev/AI-Brains"),
            "users-foo-dev-ai-brains"
        );
        assert_eq!(cursor_project_slug("//Users/foo"), "users-foo");
        assert_eq!(
            cursor_project_slug("/Users/foo/dev/AI-Brains/"),
            "users-foo-dev-ai-brains"
        );
    }

    #[test]
    fn cursor_project_slug__unix_mixed_folder__eq_ignore_ascii_case() {
        let slug = cursor_project_slug("/Users/foo/dev/AI-Brains");
        assert!(slug.eq_ignore_ascii_case("Users-foo-dev-AI-Brains"));
    }

    #[test]
    fn cursor_project_slug__drive_root_and_unc__no_hyphen_strip() {
        assert_eq!(cursor_project_slug(r"C:\"), "c-");
        assert_eq!(cursor_project_slug(r"\\server\share"), "--server-share");
    }

    #[test]
    fn cursor_project_slug_candidates__drive_path__includes_mnt_twin() {
        assert_eq!(
            cursor_project_slug_candidates(r"C:\dev\ai-brains"),
            vec![
                "c-dev-ai-brains".to_string(),
                "mnt-c-dev-ai-brains".to_string()
            ]
        );
    }

    #[test]
    fn cursor_project_slug_candidates__wsl_mnt_drive__includes_windows_twin() {
        assert_eq!(
            cursor_project_slug_candidates("/mnt/c/dev/AI-Brains"),
            vec![
                "c-dev-ai-brains".to_string(),
                "mnt-c-dev-ai-brains".to_string()
            ]
        );
    }

    #[test]
    fn cursor_project_slug_candidates__unix_users_unc_mnt_wsl__no_drive_twin() {
        assert_eq!(
            cursor_project_slug_candidates("/Users/foo/dev/AI-Brains"),
            vec!["users-foo-dev-ai-brains".to_string()]
        );
        assert_eq!(
            cursor_project_slug_candidates(r"\\server\share"),
            vec!["--server-share".to_string()]
        );
        let wsl = cursor_project_slug("/mnt/wsl/foo");
        assert_eq!(cursor_project_slug_candidates("/mnt/wsl/foo"), vec![wsl]);
    }

    #[test]
    fn adapter_capability__cursor__partial_no_hooks() {
        let capability = crate::adapter_capability(crate::AdapterKind::Cursor);
        assert_eq!(capability.level, CapabilityLevel::Partial);
        assert!(!capability.supports_hooks);
        assert!(!capability.supports_wrapper_mode);
        assert_eq!(capability.name, "cursor");
    }

    #[test]
    fn filter_cursor_jsonl_lines__user_query_kept_tools_and_turn_ended_dropped() {
        let jsonl = r#"{"role":"user","message":{"content":[{"type":"text","text":"<manually_attached_skills>\nskills dump\n</manually_attached_skills>\n<timestamp>Monday, Aug 31, 2026, 5:52 AM (UTC-4)</timestamp>\n<user_query>\nhello-cursor\n</user_query>"}]}}
{"role":"user","message":{"content":[{"type":"text","text":"<timestamp>only chrome</timestamp>"}]}}
{"role":"assistant","message":{"content":[{"type":"text","text":"ok-cursor"},{"type":"tool_use","name":"Shell","input":{}}]}}
{"type":"turn_ended","status":"success"}
not-json
"#;
        let turns = filter_cursor_jsonl_lines(jsonl);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].turn.role, IngestRole::User);
        assert_eq!(turns[0].turn.content, "hello-cursor");
        assert!(!turns[0].turn.content.contains("manually_attached_skills"));
        assert!(!turns[0].turn.content.contains("timestamp"));
        assert!(!turns[0].turn.content.contains("skills dump"));
        assert_eq!(turns[1].turn.role, IngestRole::Assistant);
        assert_eq!(turns[1].turn.content, "ok-cursor");
        assert!(!turns[1].turn.content.contains("Shell"));
        assert!(!turns[1].turn.content.contains("tool_use"));
    }

    #[test]
    fn unique_child_from_turns__folder_prefix_unique_name__binds() {
        let child = ProjectId::new();
        let aliases = vec![(child, r"C:\dev\ledgerful-web".to_string())];
        let hit = unique_child_from_turns("c-dev", &aliases, "please open ledgerful-web today");
        assert_eq!(hit.map(|(p, _)| p), Some(child));
    }

    #[test]
    fn unique_child_from_turns__hyphenated_longer_sibling__binds_longer() {
        let web = ProjectId::new();
        let api = ProjectId::new();
        let aliases = vec![
            (web, r"C:\dev\ledgerful-web".to_string()),
            (api, r"C:\dev\ledgerful-web-api".to_string()),
        ];
        let hit = unique_child_from_turns("c-dev", &aliases, "please open ledgerful-web-api today");
        assert_eq!(
            hit.map(|(p, _)| p),
            Some(api),
            "hyphen is inside the slug; shorter prefix must not also match"
        );
    }

    #[test]
    fn unique_child_from_turns__two_prefix_children_one_named__binds_named() {
        let web = ProjectId::new();
        let api = ProjectId::new();
        let aliases = vec![
            (web, r"C:\dev\ledgerful-web".to_string()),
            (api, r"C:\dev\ledgerful-api".to_string()),
        ];
        let hit = unique_child_from_turns("c-dev", &aliases, "unique hit ledgerful-web only");
        assert_eq!(hit.map(|(p, _)| p), Some(web));
    }

    #[test]
    fn unique_child_from_turns__two_children_named__none() {
        let web = ProjectId::new();
        let api = ProjectId::new();
        let aliases = vec![
            (web, r"C:\dev\ledgerful-web".to_string()),
            (api, r"C:\dev\ledgerful-api".to_string()),
        ];
        let hit =
            unique_child_from_turns("c-dev", &aliases, "compare ledgerful-web and ledgerful-api");
        assert!(hit.is_none());
    }

    #[test]
    fn child_token_mentioned__short_slug__requires_path_delimiter() {
        assert!(!child_token_mentioned("ab", "see ab in the notes"));
        assert!(child_token_mentioned("ab", r"path \ab\ file"));
        assert!(child_token_mentioned(
            "ledgerful-web",
            "word ledgerful-web here"
        ));
    }
}
