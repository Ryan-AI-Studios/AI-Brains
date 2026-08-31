//! Codex CLI seamless ingest (T253): message-only live UPS/Stop plus fail-open
//! `codex-import` of `~/.codex/sessions/**/rollout-*.jsonl`.
//!
//! Live SOOT is payload `prompt` / `lastAssistantMessage` after [`filter_turn`].
//! Do **not** parse `transcript_path` on the live hook path. Rollout JSONL is
//! not a vendor-stable API — malformed / unknown records are skipped.

use crate::agy::path_derived_display_name;
use crate::capability::{AdapterCapability, CapabilityLevel, full_harness_governed_reads};
use crate::claude::{json_str, map_nonempty, source_meta_key};
use crate::errors::{AdapterError, Result};
use crate::message_only::{
    IngestableTurn, extract_text_from_json_content, extract_user_text, filter_turn,
};
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

/// Stable Codex unbound project alias (hook + batch share this SOOT).
pub const CODEX_UNBOUND_ALIAS: &str = "codex-unbound";

/// Display name for the shared unbound Codex project.
pub const CODEX_UNBOUND_DISPLAY_NAME: &str = "(unbound Codex)";

/// Canonical Codex harness UUID (next after Claude `...0005`).
pub const CODEX_HARNESS_UUID: &str = "00000000-0000-0000-0000-000000000006";

/// Official `--payload` contract (camelCase, `deny_unknown_fields`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CodexHookPayload {
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
pub struct CodexHookMapped {
    pub session_id: String,
    pub project_hash: String,
    pub event: String,
    pub prompt: Option<String>,
    pub last_assistant: Option<String>,
    pub cwd: Option<String>,
    pub turn_id: Option<String>,
    pub uuid: Option<String>,
}

/// Message-only turn plus optional Codex `turn_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexIngestTurn {
    pub turn: IngestableTurn,
    pub turn_id: Option<String>,
}

/// How a Codex session was bound to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexBindKind {
    Path,
    Unbound,
    Default,
}

/// Options for [`import_codex_sessions`].
#[derive(Debug, Clone)]
pub struct CodexImportOptions {
    pub days: usize,
    pub default_project_id: ProjectId,
    pub allow_default_project: bool,
    pub force: bool,
    pub home_override: Option<PathBuf>,
    pub dry_run: bool,
}

impl CodexImportOptions {
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
pub struct CodexImportStats {
    pub found: usize,
    pub imported_turns: usize,
    pub sessions: usize,
    pub skipped_quiescent: usize,
    pub skipped_unchanged: usize,
    pub skipped_malformed: usize,
    pub skipped_query: usize,
    pub unbound_project: usize,
    pub bound_via_path: usize,
}

/// A discovered Codex rollout JSONL session.
#[derive(Debug, Clone)]
pub struct CodexSessionSource {
    pub path: PathBuf,
    pub session_id: String,
    pub project_hash: Option<String>,
}

pub fn codex_capability() -> AdapterCapability {
    AdapterCapability {
        name: "codex".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: true,
        notes: "Install via `harness install --harness codex`. Live UserPromptSubmit+Stop ingest `prompt` / `last_assistant_message` after T234 filter_turn (message-only). Do not parse transcript_path on the live hook. No SessionStart injection. Nightly multi-import includes Codex as the fifth source (after Claude). Codex `/hooks` trust required for live fire (`wiring=ok` is files only). Feature key is `hooks` not `codex_hooks`. Unbound alias codex-unbound. Rollout JSONL batch is fail-open (format not vendor-stable). Full harnesses bind as PrincipalKind::Agent (not Connector) so ProposeConclusion is in-matrix; principal_binding deferred until registry wiring. Connector observe-only remains ReadEvidence.".to_string(),
        governed_reads: full_harness_governed_reads(),
        governed_writes: vec![GrantCapability::ProposeConclusion],
        principal_binding: None,
    }
}

/// Parse the official camelCase payload (`deny_unknown_fields`).
pub fn parse_codex_hook_payload_strict(
    json: &str,
) -> std::result::Result<CodexHookPayload, serde_json::Error> {
    serde_json::from_str(json)
}

/// Live `--payload` gate (F14). Unrecognized → `Ok(None)`; extra keys → `Err`.
pub fn accept_codex_live_payload(
    json: &str,
) -> std::result::Result<Option<CodexHookMapped>, serde_json::Error> {
    let value: Value = serde_json::from_str(json)?;
    let Some(mapped) = map_codex_hook_payload(&value) else {
        return Ok(None);
    };
    if let Err(err) = parse_codex_hook_payload_strict(json)
        && err.to_string().contains("unknown field")
    {
        return Err(err);
    }
    Ok(Some(mapped))
}

/// Map vendor or official hook JSON to a filtered live turn set (AC9).
///
/// Returns `None` when required fields are missing. Never panics.
pub fn map_codex_hook_payload(value: &Value) -> Option<CodexHookMapped> {
    let obj = value.as_object()?;
    if is_unrecognized_codex_hook(obj) {
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
    let project_hash = normalize_codex_project_hash(&project_raw);

    let prompt = json_str(value, "prompt", "prompt")
        .and_then(|p| filter_turn("user", &extract_user_text(&p)).map(|t| t.content));
    let last_assistant = json_str(value, "lastAssistantMessage", "last_assistant_message")
        .and_then(|m| filter_turn("assistant", &m).map(|t| t.content));

    Some(CodexHookMapped {
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

fn is_unrecognized_codex_hook(obj: &serde_json::Map<String, Value>) -> bool {
    let has_prompt = map_nonempty(obj, "prompt");
    let has_last =
        map_nonempty(obj, "lastAssistantMessage") || map_nonempty(obj, "last_assistant_message");
    let has_event = map_nonempty(obj, "event") || map_nonempty(obj, "hook_event_name");
    let has_session = map_nonempty(obj, "sessionId") || map_nonempty(obj, "session_id");
    if !has_session {
        return true;
    }
    !has_event && !has_prompt && !has_last
}

/// Filter one Codex rollout record.
///
/// Keep only `type=response_item` whose `payload.type=message` and
/// `payload.role` is user/assistant. Drop `event_msg` / `session_meta` / unknown.
pub fn filter_codex_rollout_record(record: &Value) -> Option<CodexIngestTurn> {
    let type_str = record.get("type").and_then(Value::as_str)?;
    if type_str != "response_item" {
        return None;
    }
    let payload = record.get("payload")?;
    if payload.get("type").and_then(Value::as_str) != Some("message") {
        return None;
    }
    let role = payload.get("role").and_then(Value::as_str)?;
    if role != "user" && role != "assistant" {
        return None;
    }
    let text = payload
        .get("content")
        .and_then(extract_text_from_json_content)?;
    let cleaned = if role == "user" {
        extract_user_text(&text)
    } else {
        text
    };
    let turn = filter_turn(role, &cleaned)?;
    let turn_id = payload
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| record.get("turn_id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some(CodexIngestTurn { turn, turn_id })
}

/// Filter Codex rollout JSONL text (malformed lines skipped).
pub fn filter_codex_rollout_lines(jsonl: &str) -> Vec<CodexIngestTurn> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(turn) = filter_codex_rollout_record(&value) {
            out.push(turn);
        }
    }
    out
}

/// Parse a Codex rollout JSONL file into message-only turns.
pub fn parse_codex_rollout_file(path: &Path) -> Result<Vec<CodexIngestTurn>> {
    let content = std::fs::read_to_string(path)?;
    Ok(filter_codex_rollout_lines(&content))
}

/// Live turn id: `v5(session, "{event}:{turn_id-or-stable}")`.
pub fn generate_codex_live_turn_id(
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

/// Batch turn id: prefer payload `turn_id`; else `v5(session, "turn-{i}")`.
pub fn generate_codex_turn_id(
    session_id: &SessionId,
    turn_id: Option<&str>,
    kept_index: usize,
) -> TurnId {
    match turn_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), id.as_bytes())),
        None => {
            let name = format!("turn-{kept_index}");
            TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
        }
    }
}

/// Map a Codex session string to a stable [`SessionId`].
pub fn session_id_from_codex(raw: &str) -> SessionId {
    let t = raw.trim();
    if let Ok(u) = Uuid::parse_str(t) {
        return SessionId::from_uuid(u);
    }
    SessionId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, t.as_bytes()))
}

/// Normalize a Codex project hash / cwd for alias keys.
pub fn normalize_codex_project_hash(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(CODEX_UNBOUND_ALIAS) {
        return CODEX_UNBOUND_ALIAS.to_string();
    }
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Whether `AI_BRAINS_PROJECT_ID` env fallback is allowed for this hash.
pub fn codex_env_fallback_allowed(raw: &str) -> bool {
    let t = raw.trim();
    t.is_empty() || t.eq_ignore_ascii_case(CODEX_UNBOUND_ALIAS)
}

/// Path-keyed source_meta: `source_meta:codex:{sha256_hex(normalized_path)}`.
pub fn codex_source_meta_key(path: &Path) -> String {
    source_meta_key("codex", path)
}

/// Resolve Codex home: home_override → `CODEX_HOME` → `~/.codex`.
pub fn resolve_codex_home(home_override: Option<&Path>) -> Option<PathBuf> {
    if let Some(h) = home_override {
        return Some(h.join(".codex"));
    }
    if let Ok(g) = std::env::var("CODEX_HOME") {
        let t = g.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    dirs::home_dir().map(|h| h.join(".codex"))
}

/// Discover `sessions/**/rollout-*.jsonl` under Codex home.
pub fn discover_codex_sessions(codex_home: &Path) -> Result<Vec<CodexSessionSource>> {
    let sessions = codex_home.join("sessions");
    let mut out = Vec::new();
    if !sessions.is_dir() {
        return Ok(out);
    }
    walk_codex_rollouts(&sessions, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_codex_rollouts(dir: &Path, out: &mut Vec<CodexSessionSource>) -> Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            walk_codex_rollouts(&path, out)?;
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if !(lower.starts_with("rollout-") && lower.ends_with(".jsonl")) {
            continue;
        }
        if let Some(source) = source_from_codex_rollout(&path) {
            out.push(source);
        }
    }
    Ok(())
}

fn source_from_codex_rollout(path: &Path) -> Option<CodexSessionSource> {
    let (session_id, project_hash) = peek_codex_session_meta(path);
    let session_id = session_id.unwrap_or_else(|| session_id_from_rollout_name(path));
    if session_id.is_empty() {
        return None;
    }
    Some(CodexSessionSource {
        path: path.to_path_buf(),
        session_id,
        project_hash,
    })
}

fn session_id_from_rollout_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    // rollout-2026-08-15T12-00-00-<uuid>
    if let Some(idx) = stem.rfind('-') {
        let tail = &stem[idx + 1..];
        if Uuid::parse_str(tail).is_ok() {
            return tail.to_string();
        }
    }
    stem.to_string()
}

fn peek_codex_session_meta(path: &Path) -> (Option<String>, Option<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return (None, None);
    };
    let mut session_id = None;
    let mut cwd = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("session_meta") {
            continue;
        }
        let payload = value.get("payload").unwrap_or(&value);
        if session_id.is_none() {
            session_id = payload
                .get("id")
                .and_then(Value::as_str)
                .or_else(|| payload.get("session_id").and_then(Value::as_str))
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
        }
        if cwd.is_none() {
            cwd = payload
                .get("cwd")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
        }
        if session_id.is_some() && cwd.is_some() {
            break;
        }
    }
    (session_id, cwd)
}

/// Resolve Codex project hash to a project id (shared hook + batch).
pub fn resolve_codex_project(
    project_hash: Option<&str>,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, CodexBindKind, bool)> {
    let raw = project_hash.unwrap_or("").trim();
    let alias = normalize_codex_project_hash(raw);

    if alias == CODEX_UNBOUND_ALIAS {
        if allow_default_project {
            return Ok((default_project_id, alias, CodexBindKind::Default, false));
        }
        if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(CODEX_UNBOUND_ALIAS) {
            return Ok((pid, alias, CodexBindKind::Unbound, false));
        }
        return Ok((ProjectId::new(), alias, CodexBindKind::Unbound, true));
    }

    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(&alias) {
        return Ok((pid, alias, CodexBindKind::Path, false));
    }
    Ok((ProjectId::new(), alias, CodexBindKind::Path, true))
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
pub fn append_codex_turns<S: CaptureSink>(
    service: &CaptureService,
    sink: &mut S,
    session_id: SessionId,
    project_id: ProjectId,
    turns: &[CodexIngestTurn],
    start_index: usize,
    capture_context: &CaptureContext,
) -> Result<usize> {
    let harness_id = HarnessId::from_str(CODEX_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Codex harness ID: {e}")))?;
    let mut count = 0;
    for (i, item) in turns.iter().enumerate().skip(start_index) {
        let turn_id = generate_codex_turn_id(&session_id, item.turn_id.as_deref(), i);
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

/// Orchestrate import of Codex rollout JSONL sessions (fail-open).
pub fn import_codex_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: CodexImportOptions,
) -> Result<CodexImportStats> {
    let mut stats = CodexImportStats::default();
    let codex_home = match resolve_codex_home(options.home_override.as_deref()) {
        Some(h) => h,
        None => return Ok(stats),
    };

    let all_sources = discover_codex_sessions(&codex_home)?;
    if all_sources.is_empty() {
        return Ok(stats);
    }

    let cutoff = SystemTime::now() - Duration::from_secs(options.days as u64 * 24 * 60 * 60);
    let mut recent: Vec<CodexSessionSource> = Vec::new();
    for source in all_sources {
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
        "[Codex] Found {} rollout files modified in the last {} days. Format is not vendor-stable — malformed lines are skipped.",
        stats.found, options.days
    );
    if options.dry_run {
        eprintln!("[Codex] dry-run mode: scanning only — no vault writes.");
        for source in &recent {
            eprintln!(
                "[Codex] dry-run session {} path={}",
                source.session_id,
                source.path.display()
            );
        }
    }

    let codex_harness = HarnessId::from_str(CODEX_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static Codex harness ID: {e}")))?;

    for (idx, source) in recent.iter().enumerate() {
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let meta_key = codex_source_meta_key(&source.path);
        let stored_meta = match query_store.get_sync_state(&meta_key) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[Codex] skip session {} path={}: sync_state query failed: {e} — continue (fail-open)",
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
            eprintln!("[Codex] Scanning session {}/{}...", idx + 1, stats.found);
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

        let session_id = session_id_from_codex(&source.session_id);
        let raw = match std::fs::read_to_string(&source.path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[Codex] skip session {} path={}: {e} — continue (fail-open)",
                    source.session_id,
                    source.path.display()
                );
                stats.skipped_malformed += 1;
                continue;
            }
        };
        let mut malformed = 0usize;
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if serde_json::from_str::<Value>(line).is_err() {
                malformed += 1;
            }
        }
        stats.skipped_malformed += malformed;
        let turns = filter_codex_rollout_lines(&raw);
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
                    "[Codex] skip session {} path={}: max_turn query failed: {e} — continue (fail-open)",
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
            let (mut project_id, alias, kind, needs_create) = resolve_codex_project(
                source.project_hash.as_deref(),
                query_store,
                options.allow_default_project,
                options.default_project_id,
            )?;
            match kind {
                CodexBindKind::Path => stats.bound_via_path += 1,
                CodexBindKind::Unbound => stats.unbound_project += 1,
                CodexBindKind::Default => {}
            }
            if options.dry_run {
                return Ok(());
            }
            if needs_create {
                let display = if alias == CODEX_UNBOUND_ALIAS {
                    CODEX_UNBOUND_DISPLAY_NAME.to_string()
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
                    harness_id: codex_harness,
                    privacy: Privacy::LocalOnly,
                    tx_id: None,
                },
                capture_context.clone(),
                sink,
            )?;
            stats.imported_turns += append_codex_turns(
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
                    harness_id: codex_harness,
                    privacy: Privacy::LocalOnly,
                    status: SessionStopStatus::Completed,
                    reason: Some("Codex rollout JSONL import complete".to_string()),
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
                "[Codex] session {} path={} failed: {e} — continue (fail-open; prior sessions kept)",
                source.session_id,
                source.path.display()
            );
        }
    }

    Ok(stats)
}

pub fn print_codex_import_stats(stats: &CodexImportStats) {
    eprintln!(
        "[Codex] Import stats: found={} imported_turns={} sessions={} skipped_quiescent={} skipped_unchanged={} skipped_malformed={} skipped_query={} unbound_project={} bound_via_path={}",
        stats.found,
        stats.imported_turns,
        stats.sessions,
        stats.skipped_quiescent,
        stats.skipped_unchanged,
        stats.skipped_malformed,
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
    use serde_json::json;

    #[test]
    fn codex_filter__user_and_assistant__kept() {
        let user = filter_codex_rollout_record(&json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": "hello vault"}]
            }
        }))
        .expect("user");
        assert_eq!(user.turn.role, IngestRole::User);
        assert_eq!(user.turn.content, "hello vault");

        let asst = filter_codex_rollout_record(&json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "assistant",
                "content": [{"type": "output_text", "text": "final answer"}]
            }
        }))
        .expect("assistant");
        assert_eq!(asst.turn.role, IngestRole::Assistant);
        assert_eq!(asst.turn.content, "final answer");
    }

    #[test]
    fn codex_filter__event_msg_session_meta_tool_thinking__dropped() {
        assert!(
            filter_codex_rollout_record(&json!({
                "type": "event_msg",
                "payload": {"type": "message", "role": "user", "content": "x"}
            }))
            .is_none()
        );
        assert!(
            filter_codex_rollout_record(&json!({
                "type": "session_meta",
                "payload": {"id": "s", "cwd": r"C:\dev"}
            }))
            .is_none()
        );
        assert!(
            filter_codex_rollout_record(&json!({
                "type": "response_item",
                "payload": {"type": "function_call", "name": "bash"}
            }))
            .is_none()
        );
        assert!(
            filter_codex_rollout_record(&json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "system",
                    "content": "chrome"
                }
            }))
            .is_none()
        );
        assert!(
            filter_codex_rollout_record(&json!({
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "thinking", "text": "secret"}]
                }
            }))
            .is_none()
        );
    }

    #[test]
    fn codex_map__ups_prompt_and_stop_last_message() {
        let ups = map_codex_hook_payload(&json!({
            "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "projectHash": r"C:\dev\AI-Brains",
            "event": "UserPromptSubmit",
            "prompt": "summarize"
        }))
        .expect("ups");
        assert_eq!(ups.prompt.as_deref(), Some("summarize"));
        assert!(ups.last_assistant.is_none());

        let stop = map_codex_hook_payload(&json!({
            "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "cwd": r"C:\dev\AI-Brains",
            "hook_event_name": "Stop",
            "last_assistant_message": "looks good",
            "turn_id": "turn_1"
        }))
        .expect("stop");
        assert_eq!(stop.last_assistant.as_deref(), Some("looks good"));
        assert_eq!(stop.turn_id.as_deref(), Some("turn_1"));
    }

    #[test]
    fn codex_map__missing_fields__none() {
        assert!(
            map_codex_hook_payload(&json!({
                "sessionId": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
            }))
            .is_none()
        );
        assert!(map_codex_hook_payload(&json!({"prompt": "hi"})).is_none());
    }

    #[test]
    fn codex_hook_payload__deny_unknown_fields() {
        let err = parse_codex_hook_payload_strict(
            r#"{"sessionId":"a","projectHash":"b","event":"Stop","transcript_path":"x"}"#,
        )
        .expect_err("unknown field");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn codex_live_payload__unknown_field_on_valid__err() {
        let err = accept_codex_live_payload(
            r#"{"sessionId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa","projectHash":"p","event":"Stop","lastAssistantMessage":"ok","transcript_path":"x"}"#,
        )
        .expect_err("extra key");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn codex_live_payload__unrecognized__none() {
        let skip =
            accept_codex_live_payload(r#"{"sessionId":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"}"#)
                .expect("missing fields skip");
        assert!(skip.is_none());
    }

    #[test]
    fn codex_capability__full_hooks_honest_notes() {
        let c = codex_capability();
        assert_eq!(c.level, CapabilityLevel::Full);
        assert!(c.supports_hooks);
        assert!(c.notes.contains("harness install --harness codex"));
        assert!(c.notes.contains("No SessionStart injection"));
        assert!(c.notes.contains("Nightly multi-import"));
        assert!(!c.notes.contains("No nightly multi-import"));
        assert!(c.notes.contains("/hooks"));
        assert!(c.notes.contains("message-only"));
        assert!(!c.notes.contains("codex_hooks = true"));
    }

    #[test]
    fn generate_codex_turn_id__uuid_or_index() {
        let sid = session_id_from_codex("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let a = generate_codex_turn_id(&sid, Some("turn_1"), 0);
        let b = generate_codex_turn_id(&sid, Some("turn_1"), 9);
        let c = generate_codex_turn_id(&sid, None, 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn codex_import_stats__default__skipped_query_zero() {
        let stats = CodexImportStats::default();
        assert_eq!(stats.skipped_query, 0);
        assert_eq!(stats.skipped_malformed, 0);
    }
}
