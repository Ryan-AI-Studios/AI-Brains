//! Claude Code seamless ingest (T253): message-only live UPS/Stop/SessionEnd
//! plus fail-open `claude-import` of `~/.claude/projects/<encoded-cwd>/*.jsonl`.
//!
//! Live SOOT is payload `prompt` / `lastAssistantMessage` after [`filter_turn`].
//! Do **not** parse `transcript_path` on the live hook path.

use crate::agy::path_derived_display_name;
use crate::capability::{AdapterCapability, CapabilityLevel, full_harness_governed_reads};
use crate::errors::{AdapterError, Result};
use crate::grok::percent_decode_component;
use crate::message_only::{
    IngestableTurn, extract_text_from_json_content, extract_user_text, filter_turn,
};
use crate::neutral_event::NeutralEvent;
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::GrantCapability;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Stable Claude unbound project alias (hook + batch share this SOOT).
pub const CLAUDE_UNBOUND_ALIAS: &str = "claude-unbound";

/// Display name for the shared unbound Claude project.
pub const CLAUDE_UNBOUND_DISPLAY_NAME: &str = "(unbound Claude)";

/// Canonical Claude harness UUID (next after OpenCode `...0004`).
pub const CLAUDE_HARNESS_UUID: &str = "00000000-0000-0000-0000-000000000005";

/// Official `--payload` contract (camelCase, `deny_unknown_fields`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClaudeHookPayload {
    pub session_id: String,
    pub project_hash: String,
    pub event: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub last_assistant_message: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub turn_id: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
}

/// Live map result after T234 filter (empty role texts already dropped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeHookMapped {
    pub session_id: String,
    pub project_hash: String,
    pub event: String,
    pub prompt: Option<String>,
    pub last_assistant: Option<String>,
    pub cwd: Option<String>,
    pub turn_id: Option<String>,
    pub uuid: Option<String>,
}

/// Message-only turn plus optional Claude record `uuid`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeIngestTurn {
    pub turn: IngestableTurn,
    pub uuid: Option<String>,
}

/// How a Claude session was bound to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeBindKind {
    Path,
    Unbound,
    Default,
}

/// Options for [`import_claude_sessions`].
#[derive(Debug, Clone)]
pub struct ClaudeImportOptions {
    pub days: usize,
    pub default_project_id: ProjectId,
    pub allow_default_project: bool,
    pub force: bool,
    pub home_override: Option<PathBuf>,
    pub dry_run: bool,
}

impl ClaudeImportOptions {
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
pub struct ClaudeImportStats {
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

/// A discovered Claude project JSONL session.
#[derive(Debug, Clone)]
pub struct ClaudeSessionSource {
    pub path: PathBuf,
    pub session_id: String,
    pub project_hash: Option<String>,
}

pub fn claude_capability() -> AdapterCapability {
    AdapterCapability {
        name: "claude".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Install via `harness install --harness claude`. Live UserPromptSubmit+Stop+SessionEnd ingest `prompt` / `last_assistant_message` after T234 filter_turn (message-only). Do not parse transcript_path on the live hook. No SessionStart injection. Nightly multi-import includes Claude as the fourth source (after agy → grok → opencode). Unbound alias claude-unbound. Grok-shaped stdin fail-open skip. Full harnesses bind as PrincipalKind::Agent (not Connector) so ProposeConclusion is in-matrix; principal_binding deferred until registry wiring. Connector observe-only remains ReadEvidence.".to_string(),
        governed_reads: full_harness_governed_reads(),
        governed_writes: vec![GrantCapability::ProposeConclusion],
        principal_binding: None,
    }
}

/// Legacy NeutralEvent parse — routes content through [`filter_turn`].
pub fn parse_claude_stop_payload(value: &Value) -> crate::Result<NeutralEvent> {
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("assistant");
    let raw = value
        .get("content")
        .and_then(extract_text_from_json_content)
        .unwrap_or_default();
    let status = value
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    match filter_turn(role, &raw) {
        Some(turn) => Ok(NeutralEvent {
            role: turn.role.as_str().to_string(),
            content: turn.content,
            status,
            warnings: Vec::new(),
        }),
        None => Ok(NeutralEvent {
            role: role.to_string(),
            content: String::new(),
            status,
            warnings: vec!["dropped by message-only filter".to_string()],
        }),
    }
}

/// Parse the official camelCase payload (`deny_unknown_fields`).
pub fn parse_claude_hook_payload_strict(
    json: &str,
) -> std::result::Result<ClaudeHookPayload, serde_json::Error> {
    serde_json::from_str(json)
}

/// Live `--payload` gate (F14 + F23).
///
/// 1. Invalid JSON → `Err` (CLI exit 1).
/// 2. Unrecognized / Grok-shaped → `Ok(None)` (CLI exit 0 skip).
/// 3. Recognized, then official camelCase `deny_unknown_fields` hits an extra
///    key → `Err` (CLI exit 1 JSON).
/// 4. Otherwise `Ok(Some(mapped))` — ingest the T234-filtered map.
pub fn accept_claude_live_payload(
    json: &str,
) -> std::result::Result<Option<ClaudeHookMapped>, serde_json::Error> {
    let value: Value = serde_json::from_str(json)?;
    let Some(mapped) = map_claude_hook_payload(&value) else {
        return Ok(None);
    };
    if let Err(err) = parse_claude_hook_payload_strict(json)
        && err.to_string().contains("unknown field")
    {
        return Err(err);
    }
    Ok(Some(mapped))
}

/// Map vendor or official hook JSON to a filtered live turn set (AC8 / F23).
///
/// Returns `None` for unrecognized stdin (missing Claude fields, or Grok
/// camelCase-only `hookEventName`/`sessionId` without prompt / last message /
/// `hook_event_name` / `event`). Never panics.
pub fn map_claude_hook_payload(value: &Value) -> Option<ClaudeHookMapped> {
    let obj = value.as_object()?;
    if is_unrecognized_claude_hook(obj) {
        return None;
    }

    let session_id = json_str(value, "sessionId", "session_id")?;
    let event = json_str(value, "event", "hook_event_name").unwrap_or_else(|| {
        if json_str(value, "prompt", "prompt").is_some() {
            "UserPromptSubmit".to_string()
        } else {
            "Stop".to_string()
        }
    });

    let cwd = json_str(value, "cwd", "cwd");
    let project_raw = json_str(value, "projectHash", "project_hash")
        .or_else(|| cwd.clone())
        .unwrap_or_default();
    let project_hash = normalize_claude_project_hash(&project_raw);

    let prompt = json_str(value, "prompt", "prompt")
        .and_then(|p| filter_turn("user", &extract_user_text(&p)).map(|t| t.content));
    let last_assistant = json_str(value, "lastAssistantMessage", "last_assistant_message")
        .and_then(|m| filter_turn("assistant", &m).map(|t| t.content));

    Some(ClaudeHookMapped {
        session_id,
        project_hash,
        event,
        prompt,
        last_assistant,
        cwd,
        turn_id: json_str(value, "turnId", "turn_id"),
        uuid: json_str(value, "uuid", "uuid"),
    })
}

fn is_unrecognized_claude_hook(obj: &serde_json::Map<String, Value>) -> bool {
    let has_prompt = map_nonempty(obj, "prompt");
    let has_last =
        map_nonempty(obj, "lastAssistantMessage") || map_nonempty(obj, "last_assistant_message");
    let has_event = map_nonempty(obj, "event") || map_nonempty(obj, "hook_event_name");
    let has_session = map_nonempty(obj, "sessionId") || map_nonempty(obj, "session_id");
    let grok_camel_only = obj.contains_key("hookEventName")
        && !map_nonempty(obj, "hook_event_name")
        && !map_nonempty(obj, "event")
        && !has_prompt
        && !has_last;

    if grok_camel_only {
        return true;
    }
    if !has_session {
        return true;
    }
    // Missing Claude fields (no event and no role text).
    !has_event && !has_prompt && !has_last
}

/// Filter one Claude project JSONL record (`type=user|assistant` + message.content).
pub fn filter_claude_jsonl_record(record: &Value) -> Option<ClaudeIngestTurn> {
    if record.get("isSidechain").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let type_str = record.get("type").and_then(Value::as_str)?;
    if type_str != "user" && type_str != "assistant" {
        return None;
    }
    let message = record.get("message").unwrap_or(record);
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or(type_str);
    if role != "user" && role != "assistant" {
        return None;
    }
    let text = message
        .get("content")
        .and_then(extract_text_from_json_content)
        .or_else(|| {
            record
                .get("content")
                .and_then(extract_text_from_json_content)
        })?;
    let cleaned = if role == "user" {
        extract_user_text(&text)
    } else {
        text
    };
    let turn = filter_turn(role, &cleaned)?;
    let uuid = record
        .get("uuid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some(ClaudeIngestTurn { turn, uuid })
}

/// Filter Claude project JSONL text (malformed lines skipped).
pub fn filter_claude_jsonl_lines(jsonl: &str) -> Vec<ClaudeIngestTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(turn) = filter_claude_jsonl_record(&value) {
            out.push(turn);
        }
    }
    out
}

/// Parse a Claude project JSONL file into message-only turns.
pub fn parse_claude_jsonl_file(path: &Path) -> Result<Vec<ClaudeIngestTurn>> {
    let content = std::fs::read_to_string(path)?;
    Ok(filter_claude_jsonl_lines(&content))
}

/// Live turn id: `v5(session, "{event}:{turn_id-or-stable}")`.
pub fn generate_claude_live_turn_id(
    session_id: &SessionId,
    event: &str,
    turn_or_uuid: Option<&str>,
    role_suffix: Option<&str>,
) -> TurnId {
    let stable = turn_or_uuid
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("stable");
    let name = match role_suffix.map(str::trim).filter(|s| !s.is_empty()) {
        Some(role) => format!("{event}:{stable}:{role}"),
        None => format!("{event}:{stable}"),
    };
    TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
}

/// Batch turn id: prefer record `uuid`; else `v5(session, "turn-{i}")`.
pub fn generate_claude_turn_id(
    session_id: &SessionId,
    uuid: Option<&str>,
    kept_index: usize,
) -> TurnId {
    match uuid.map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), id.as_bytes())),
        None => {
            let name = format!("turn-{kept_index}");
            TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
        }
    }
}

/// Map a Claude session string to a stable [`SessionId`].
pub fn session_id_from_claude(raw: &str) -> SessionId {
    let t = raw.trim();
    if let Ok(u) = Uuid::parse_str(t) {
        return SessionId::from_uuid(u);
    }
    SessionId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, t.as_bytes()))
}

/// Normalize a Claude project hash / cwd for alias keys.
pub fn normalize_claude_project_hash(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(CLAUDE_UNBOUND_ALIAS) {
        return CLAUDE_UNBOUND_ALIAS.to_string();
    }
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Whether `AI_BRAINS_PROJECT_ID` env fallback is allowed for this hash.
pub fn claude_env_fallback_allowed(raw: &str) -> bool {
    let t = raw.trim();
    t.is_empty() || t.eq_ignore_ascii_case(CLAUDE_UNBOUND_ALIAS)
}

/// Path-keyed source_meta: `source_meta:claude:{sha256_hex(normalized_path)}`.
pub fn claude_source_meta_key(path: &Path) -> String {
    source_meta_key("claude", path)
}

/// Decode a Claude `projects/<folder>` name to a path (percent-decode, then dash).
pub fn decode_claude_project_folder(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(decoded) = percent_decode_component(trimmed)
        && looks_like_path(&decoded)
    {
        return Some(decoded);
    }
    decode_claude_dash_folder(trimmed)
}

fn decode_claude_dash_folder(name: &str) -> Option<String> {
    // Claude Code encodes `C:\dev\Foo` as `C--dev-Foo` (`:` and `\` → `-`).
    // Hyphens inside a component (e.g. `AI-Brains`) are ambiguous — prefer
    // percent-encoded folder names (T237). This heuristic only rewrites the
    // drive `--` marker and treats remaining `-` as separators.
    if !name.contains("--") {
        return None;
    }
    let restored = name.replacen("--", ":\\", 1).replace('-', "\\");
    if looks_like_path(&restored) {
        Some(restored)
    } else {
        None
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('\\') || s.contains('/') || (s.len() >= 2 && s.as_bytes()[1] == b':')
}

/// True when a session path lives under `subagents/`.
pub fn is_claude_sidechain_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("subagents"))
    })
}

/// Resolve Claude home: home_override → `CLAUDE_HOME` → `~/.claude`.
pub fn resolve_claude_home(home_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(h) = home_override {
        return Some(h.join(".claude"));
    }
    if let Ok(g) = std::env::var("CLAUDE_HOME") {
        let t = g.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// Discover `projects/<encoded-cwd>/*.jsonl` (skips `subagents/`).
pub fn discover_claude_sessions(claude_home: &Path) -> Result<Vec<ClaudeSessionSource>> {
    let projects = claude_home.join("projects");
    let mut out = Vec::new();
    if !projects.is_dir() {
        return Ok(out);
    }
    walk_claude_jsonl(&projects, None, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_claude_jsonl(
    dir: &Path,
    encoded_cwd: Option<&str>,
    out: &mut Vec<ClaudeSessionSource>,
) -> Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.eq_ignore_ascii_case("subagents") {
            continue;
        }
        if path.is_dir() {
            let child_encoded = encoded_cwd.unwrap_or(&name);
            walk_claude_jsonl(&path, Some(child_encoded), out)?;
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".jsonl") {
            continue;
        }
        if is_claude_sidechain_path(&path) {
            continue;
        }
        let Some(source) = source_from_claude_jsonl(&path, encoded_cwd) else {
            continue;
        };
        out.push(source);
    }
    Ok(())
}

fn source_from_claude_jsonl(
    jsonl: &Path,
    encoded_cwd: Option<&str>,
) -> Option<ClaudeSessionSource> {
    let stem = jsonl.file_stem()?.to_str()?.to_string();
    if stem.is_empty() {
        return None;
    }
    let folder = encoded_cwd
        .map(str::to_string)
        .or_else(|| {
            jsonl
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let project_hash = decode_claude_project_folder(&folder);
    Some(ClaudeSessionSource {
        path: jsonl.to_path_buf(),
        session_id: stem,
        project_hash,
    })
}

/// Resolve Claude project hash to a project id (shared hook + batch).
pub fn resolve_claude_project(
    project_hash: Option<&str>,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, ClaudeBindKind, bool)> {
    resolve_path_project(
        project_hash,
        CLAUDE_UNBOUND_ALIAS,
        query_store,
        allow_default_project,
        default_project_id,
    )
    .map(|(pid, alias, unbound, needs_create)| {
        let kind = if unbound {
            if allow_default_project && alias == CLAUDE_UNBOUND_ALIAS {
                ClaudeBindKind::Default
            } else {
                ClaudeBindKind::Unbound
            }
        } else {
            ClaudeBindKind::Path
        };
        (pid, alias, kind, needs_create)
    })
}

fn resolve_path_project(
    project_hash: Option<&str>,
    unbound_alias: &str,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, bool, bool)> {
    let raw = project_hash.unwrap_or("").trim();
    let alias = if raw.is_empty() || raw.eq_ignore_ascii_case(unbound_alias) {
        unbound_alias.to_string()
    } else {
        match ai_brains_path::normalize_project_path(raw) {
            Ok(p) => p.canonical().to_string(),
            Err(_) => raw.to_string(),
        }
    };

    if alias == unbound_alias {
        if allow_default_project {
            return Ok((default_project_id, alias, true, false));
        }
        if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(unbound_alias) {
            return Ok((pid, alias, true, false));
        }
        return Ok((ProjectId::new(), alias, true, true));
    }

    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(&alias) {
        return Ok((pid, alias, false, false));
    }
    Ok((ProjectId::new(), alias, false, true))
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

/// Shared live+batch ingest of already-filtered turns (`thinking` always `None`).
pub fn append_claude_turns<S: CaptureSink>(
    service: &CaptureService,
    sink: &mut S,
    session_id: SessionId,
    project_id: ProjectId,
    turns: &[ClaudeIngestTurn],
    start_index: usize,
    capture_context: &CaptureContext,
) -> Result<usize> {
    let harness_id = HarnessId::from_str(CLAUDE_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Claude harness ID: {e}")))?;
    let mut count = 0;
    for (i, item) in turns.iter().enumerate().skip(start_index) {
        let turn_id = generate_claude_turn_id(&session_id, item.uuid.as_deref(), i);
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

/// Orchestrate import of Claude project JSONL sessions.
pub fn import_claude_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: ClaudeImportOptions,
) -> Result<ClaudeImportStats> {
    let mut stats = ClaudeImportStats::default();
    let claude_home = match resolve_claude_home(options.home_override.as_deref()) {
        Some(h) => h,
        None => return Ok(stats),
    };

    let all_sources = discover_claude_sessions(&claude_home)?;
    if all_sources.is_empty() {
        return Ok(stats);
    }

    let cutoff = SystemTime::now() - Duration::from_secs(options.days as u64 * 24 * 60 * 60);
    let mut recent: Vec<ClaudeSessionSource> = Vec::new();
    for source in all_sources {
        if is_claude_sidechain_path(&source.path) {
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
        "[Claude] Found {} sessions modified in the last {} days. Scanning for new turns...",
        stats.found, options.days
    );
    if options.dry_run {
        eprintln!("[Claude] dry-run mode: scanning only — no vault writes.");
        for source in &recent {
            eprintln!(
                "[Claude] dry-run session {} path={}",
                source.session_id,
                source.path.display()
            );
        }
    }

    let claude_harness = HarnessId::from_str(CLAUDE_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Claude harness ID: {e}")))?;

    for (idx, source) in recent.iter().enumerate() {
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let meta_key = claude_source_meta_key(&source.path);
        let stored_meta = match query_store.get_sync_state(&meta_key) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[Claude] skip session {} path={}: sync_state query failed: {e} — continue (fail-open)",
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
            eprintln!("[Claude] Scanning session {}/{}...", idx + 1, stats.found);
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

        let session_id = session_id_from_claude(&source.session_id);
        let turns = match parse_claude_jsonl_file(&source.path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[Claude] skip session {} path={}: {e} — continue (fail-open)",
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
                    "[Claude] skip session {} path={}: max_turn query failed: {e} — continue (fail-open)",
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
            let (mut project_id, alias, kind, needs_create) = resolve_claude_project(
                source.project_hash.as_deref(),
                query_store,
                options.allow_default_project,
                options.default_project_id,
            )?;
            match kind {
                ClaudeBindKind::Path => stats.bound_via_path += 1,
                ClaudeBindKind::Unbound => stats.unbound_project += 1,
                ClaudeBindKind::Default => {}
            }
            if options.dry_run {
                return Ok(());
            }
            if needs_create {
                let display = if alias == CLAUDE_UNBOUND_ALIAS {
                    CLAUDE_UNBOUND_DISPLAY_NAME.to_string()
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
                    harness_id: claude_harness,
                    privacy: Privacy::LocalOnly,
                    tx_id: None,
                },
                capture_context.clone(),
                sink,
            )?;
            stats.imported_turns += append_claude_turns(
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
                    harness_id: claude_harness,
                    privacy: Privacy::LocalOnly,
                    status: SessionStopStatus::Completed,
                    reason: Some("Claude project JSONL import complete".to_string()),
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
                "[Claude] session {} path={} failed: {e} — continue (fail-open; prior sessions kept)",
                source.session_id,
                source.path.display()
            );
        }
    }

    Ok(stats)
}

pub fn print_claude_import_stats(stats: &ClaudeImportStats) {
    eprintln!(
        "[Claude] Import stats: found={} imported_turns={} sessions={} skipped_quiescent={} skipped_unchanged={} skipped_sidechain={} skipped_query={} unbound_project={} bound_via_path={}",
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

pub(crate) fn source_meta_key(prefix: &str, path: &Path) -> String {
    let key_material = match ai_brains_path::normalize_project_path(&path.to_string_lossy()) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    format!("source_meta:{prefix}:{}", hex::encode(hasher.finalize()))
}

pub(crate) fn json_str(value: &Value, camel: &str, snake: &str) -> Option<String> {
    value
        .get(camel)
        .and_then(Value::as_str)
        .or_else(|| value.get(snake).and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

pub(crate) fn map_nonempty(obj: &serde_json::Map<String, Value>, key: &str) -> bool {
    obj.get(key)
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::message_only::IngestRole;
    use serde_json::json;

    #[test]
    fn claude_filter__user_and_assistant__kept() {
        let user = filter_claude_jsonl_record(&json!({
            "type": "user",
            "uuid": "u1",
            "message": {"role": "user", "content": "hello vault"}
        }))
        .expect("user");
        assert_eq!(user.turn.role, IngestRole::User);
        assert_eq!(user.turn.content, "hello vault");
        assert_eq!(user.uuid.as_deref(), Some("u1"));

        let asst = filter_claude_jsonl_record(&json!({
            "type": "assistant",
            "message": {"role": "assistant", "content": [
                {"type": "text", "text": "final answer"},
                {"type": "tool_use", "name": "bash", "input": {}}
            ]}
        }))
        .expect("assistant");
        assert_eq!(asst.turn.role, IngestRole::Assistant);
        assert_eq!(asst.turn.content, "final answer");
        assert!(!asst.turn.content.contains("bash"));
    }

    #[test]
    fn claude_filter__tool_thinking_system_sidechain__dropped() {
        assert!(
            filter_claude_jsonl_record(&json!({
                "type": "assistant",
                "message": {"role": "assistant", "content": [
                    {"type": "thinking", "text": "secret cot"}
                ]}
            }))
            .is_none()
        );
        assert!(
            filter_claude_jsonl_record(&json!({
                "type": "system",
                "message": {"role": "system", "content": "chrome"}
            }))
            .is_none()
        );
        assert!(
            filter_claude_jsonl_record(&json!({
                "type": "user",
                "isSidechain": true,
                "message": {"role": "user", "content": "child"}
            }))
            .is_none()
        );
        assert!(
            filter_claude_jsonl_record(&json!({
                "type": "attachment",
                "message": {"content": "nope"}
            }))
            .is_none()
        );
    }

    #[test]
    fn claude_map__ups_prompt_and_stop_last_message() {
        let ups = map_claude_hook_payload(&json!({
            "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "projectHash": r"C:\dev\AI-Brains",
            "event": "UserPromptSubmit",
            "prompt": "  ship it  "
        }))
        .expect("ups");
        assert_eq!(ups.prompt.as_deref(), Some("ship it"));
        assert!(ups.last_assistant.is_none());

        let stop = map_claude_hook_payload(&json!({
            "hook_event_name": "Stop",
            "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "cwd": r"C:\dev\AI-Brains",
            "last_assistant_message": "done"
        }))
        .expect("stop");
        assert_eq!(stop.last_assistant.as_deref(), Some("done"));
        assert_eq!(stop.event, "Stop");
        assert_eq!(stop.prompt, None);
    }

    #[test]
    fn claude_map__grok_shaped_stdin__none() {
        assert!(
            map_claude_hook_payload(&json!({
                "hookEventName": "Stop",
                "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "projectHash": r"C:\dev\AI-Brains",
                "historyPath": "C:\\tmp\\chat_history.jsonl"
            }))
            .is_none()
        );
        assert!(
            map_claude_hook_payload(&json!({
                "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
            }))
            .is_none()
        );
    }

    #[test]
    fn claude_map__empty_prompt__role_skipped() {
        let mapped = map_claude_hook_payload(&json!({
            "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "projectHash": "claude-unbound",
            "event": "UserPromptSubmit",
            "prompt": "   "
        }))
        .expect("recognized");
        assert!(mapped.prompt.is_none());
        assert!(mapped.last_assistant.is_none());
    }

    #[test]
    fn claude_hook_payload__deny_unknown_fields() {
        let err = parse_claude_hook_payload_strict(
            r#"{"sessionId":"a","projectHash":"b","event":"Stop","historyPath":"x"}"#,
        )
        .expect_err("unknown field");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn claude_live_payload__unknown_field_on_valid__err() {
        let err = accept_claude_live_payload(
            r#"{"sessionId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","projectHash":"p","event":"Stop","lastAssistantMessage":"done","historyPath":"x"}"#,
        )
        .expect_err("extra key");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn claude_live_payload__grok_shaped__none() {
        let skip = accept_claude_live_payload(
            r#"{"hookEventName":"Stop","sessionId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","projectHash":"p","historyPath":"x"}"#,
        )
        .expect("grok-shaped is skip, not deny");
        assert!(skip.is_none());
    }

    #[test]
    fn claude_live_turn_id__event_and_uuid() {
        let sid = session_id_from_claude("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let a = generate_claude_live_turn_id(&sid, "Stop", Some("u1"), None);
        let b = generate_claude_live_turn_id(&sid, "Stop", Some("u1"), None);
        let c = generate_claude_live_turn_id(&sid, "Stop", None, None);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn claude_decode__percent_and_dash() {
        let enc = crate::grok::percent_encode_path_component(r"C:\dev\AI-Brains");
        let decoded = decode_claude_project_folder(&enc).expect("percent");
        assert!(decoded.to_ascii_lowercase().contains("ai-brains"));
        let dash = decode_claude_project_folder("C--dev-AI-Brains").expect("dash");
        assert!(dash.contains(r"C:\dev"));
    }

    #[test]
    fn parse_claude_stop_payload__routes_filter_turn() {
        let event = parse_claude_stop_payload(&json!({
            "role": "assistant",
            "content": "final answer",
            "stop_reason": "end_turn"
        }))
        .expect("parse");
        assert_eq!(event.role, "assistant");
        assert_eq!(event.content, "final answer");
        assert_eq!(event.status.as_deref(), Some("end_turn"));

        let dropped = parse_claude_stop_payload(&json!({
            "role": "assistant",
            "content": {"type": "thinking", "text": "hidden"}
        }))
        .expect("dropped");
        assert!(dropped.content.is_empty());
    }

    #[test]
    fn claude_capability__full_hooks_honest_notes() {
        let c = claude_capability();
        assert_eq!(c.level, CapabilityLevel::Full);
        assert!(c.supports_hooks);
        assert!(c.notes.contains("harness install --harness claude"));
        assert!(c.notes.contains("No SessionStart injection"));
        assert!(c.notes.contains("Nightly multi-import"));
        assert!(!c.notes.contains("No nightly multi-import"));
        assert!(c.notes.contains("message-only"));
    }

    #[test]
    fn claude_import_stats__default__skipped_query_zero() {
        let stats = ClaudeImportStats::default();
        assert_eq!(stats.skipped_query, 0);
        assert_eq!(stats.skipped_unchanged, 0);
    }
}
