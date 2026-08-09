//! OpenCode seamless ingest (T238): message-only filter, plugin hook path,
//! list+export batch import with watermark — **never** open `opencode.db`.
//!
//! Content SOOT is `message_only::{filter_opencode_export,filter_opencode_message*}`.
//! Live path prefers SDK messages (plugin) → export-shaped JSON → `opencode-hook`.
//! Batch uses CLI `session list` + `export` only.

use crate::agy::path_derived_display_name;
use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::errors::{AdapterError, Result};
use crate::message_only::{OpenCodeIngestTurn, filter_opencode_export};
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, TurnId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime};
use uuid::Uuid;

/// Stable OpenCode unbound project alias (hook + batch share this SOOT).
pub const OPENCODE_UNBOUND_ALIAS: &str = "opencode-unbound";

/// Display name for the shared unbound OpenCode project.
pub const OPENCODE_UNBOUND_DISPLAY_NAME: &str = "(unbound OpenCode)";

/// Canonical OpenCode harness UUID (next after Grok 0003).
pub const OPENCODE_HARNESS_UUID: &str = "00000000-0000-0000-0000-000000000004";

/// Default export subprocess timeout (F12 / F19).
pub const OPENCODE_EXPORT_TIMEOUT_SECS: u64 = 120;

/// Default session list max-count (OpenCode default; AC23).
pub const OPENCODE_LIST_DEFAULT_CAP: usize = 100;

/// Cursor file name under `~/.ai-brains/` (F18).
pub const OPENCODE_IMPORT_CURSOR_FILENAME: &str = "opencode-import-cursor.json";

// ---------------------------------------------------------------------------
// Capability
// ---------------------------------------------------------------------------

/// OpenCode adapter capability (F34 / F39 honesty notes).
pub fn opencode_capability() -> AdapterCapability {
    AdapterCapability {
        name: "opencode".to_string(),
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: false,
        notes: "Live: `harness install --harness opencode` writes `~/.config/opencode/plugins/ai-brains-capture.js` (session.idle; schema-deprecated risk — batch is completeness backstop). Child/subagent sessions with parentID are skipped (plugin fail-closed on session.get failure; list parentID skip). Synthetic/ignored/editor_context text parts dropped; bare non-synthetic user text kept. Live prefers client.session.messages (export fallback, 120s). Batch: `opencode session list --format json` + `opencode export` + watermark at ~/.ai-brains/opencode-import-cursor.json — never opens opencode.db. List default cap 100 (list_capped warn when len>=100 even if --max-sessions higher). Turn ids use msg_* for stability (v5(session,msg_id)); delta is max turn_index + watermark (same class as Grok turn-{i} residual — not existence-check per msg_id). OPENCODE_CONFIG_DIR relocates config. --pure / OPENCODE_DISABLE_DEFAULT_PLUGINS soft. SYSTEM scheduled import may still --skip-import (T239)."
            .to_string(),
        governed_reads: Vec::new(),
        governed_writes: Vec::new(),
        principal_binding: None,
    }
}

// ---------------------------------------------------------------------------
// Turn ids / path / meta
// ---------------------------------------------------------------------------

/// Deterministic turn id SOOT (F13): prefer `v5(session, msg_id)`; else `v5(session, "turn-{i}")`.
pub fn generate_opencode_turn_id(
    session_id: &SessionId,
    msg_id: Option<&str>,
    kept_index: usize,
) -> TurnId {
    let name = match msg_id.map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => id.to_string(),
        None => format!("turn-{kept_index}"),
    };
    TurnId::from_uuid(Uuid::new_v5(&session_id.as_uuid(), name.as_bytes()))
}

/// Map OpenCode `ses_*` (or any non-UUID string) to a stable [`SessionId`].
pub fn session_id_from_opencode(raw: &str) -> SessionId {
    let t = raw.trim();
    if let Ok(u) = Uuid::parse_str(t) {
        return SessionId::from_uuid(u);
    }
    SessionId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, t.as_bytes()))
}

/// Normalize a project path / hash for OpenCode alias keys.
pub fn normalize_opencode_project_hash(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(OPENCODE_UNBOUND_ALIAS) {
        return OPENCODE_UNBOUND_ALIAS.to_string();
    }
    match ai_brains_path::normalize_project_path(trimmed) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Whether `AI_BRAINS_PROJECT_ID` env fallback is allowed for this hash (F21).
pub fn opencode_env_fallback_allowed(raw: &str) -> bool {
    let t = raw.trim();
    t.is_empty() || t.eq_ignore_ascii_case(OPENCODE_UNBOUND_ALIAS)
}

/// Path/session-keyed source_meta: `source_meta:opencode:{sha256_hex(key)}` (F22).
pub fn opencode_source_meta_key(session_or_path_key: &str) -> String {
    let key_material = match ai_brains_path::normalize_project_path(session_or_path_key) {
        Ok(p) => p.canonical().to_string(),
        Err(_) => session_or_path_key.to_string(),
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key_material.as_bytes());
    format!("source_meta:opencode:{}", hex::encode(hasher.finalize()))
}

/// Resolve OpenCode config dir: `OPENCODE_CONFIG_DIR` if set, else `home/.config/opencode` (F40).
pub fn resolve_opencode_config_dir(home: Option<&Path>) -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        let t = dir.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    home.map(|h| h.join(".config").join("opencode"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".config").join("opencode")))
}

// ---------------------------------------------------------------------------
// Parse export
// ---------------------------------------------------------------------------

/// Parse export-shaped JSON (`{info,messages}` or bare messages array) into kept turns.
pub fn parse_export_json(doc: &Value) -> Vec<OpenCodeIngestTurn> {
    filter_opencode_export(doc)
}

/// Read and parse an export file.
pub fn parse_export_file(path: &Path) -> Result<Vec<OpenCodeIngestTurn>> {
    let raw = std::fs::read_to_string(path)?;
    let doc: Value = serde_json::from_str(&raw)
        .map_err(|e| AdapterError::Other(format!("parse export {}: {e}", path.display())))?;
    Ok(parse_export_json(&doc))
}

// ---------------------------------------------------------------------------
// Append turns (shared live + batch)
// ---------------------------------------------------------------------------

/// Shared live+batch ingest of already-filtered OpenCode turns.
///
/// Capture privacy SOOT: `thinking` is **always** `None`.
/// Turn ids use [`generate_opencode_turn_id`] so hook and batch share SOOT.
pub fn append_opencode_turns<S: CaptureSink>(
    service: &CaptureService,
    sink: &mut S,
    session_id: SessionId,
    project_id: ProjectId,
    turns: &[OpenCodeIngestTurn],
    start_index: usize,
    capture_context: &CaptureContext,
) -> Result<usize> {
    let harness_id = HarnessId::from_str(OPENCODE_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static OpenCode harness ID: {e}")))?;
    let mut count = 0;
    for (i, item) in turns.iter().enumerate().skip(start_index) {
        let turn_id = generate_opencode_turn_id(&session_id, item.msg_id.as_deref(), i);
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

// ---------------------------------------------------------------------------
// Project bind
// ---------------------------------------------------------------------------

/// How an OpenCode session was bound to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenCodeBindKind {
    Worktree,
    Directory,
    Unbound,
    Default,
}

/// Resolve worktree/directory to a project id (shared hook + batch).
///
/// Prefer **worktree** → **directory** → unbound (F20). Env project only when unbound
/// is handled by callers via [`opencode_env_fallback_allowed`].
pub fn resolve_opencode_project(
    worktree: Option<&str>,
    directory: Option<&str>,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, OpenCodeBindKind, bool)> {
    let bind_raw = worktree
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| (s, OpenCodeBindKind::Worktree))
        .or_else(|| {
            directory
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| (s, OpenCodeBindKind::Directory))
        });

    let (raw, kind) = match bind_raw {
        Some((r, k)) => (r, k),
        None => {
            if allow_default_project {
                return Ok((
                    default_project_id,
                    OPENCODE_UNBOUND_ALIAS.to_string(),
                    OpenCodeBindKind::Default,
                    false,
                ));
            }
            if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(OPENCODE_UNBOUND_ALIAS)
            {
                return Ok((
                    pid,
                    OPENCODE_UNBOUND_ALIAS.to_string(),
                    OpenCodeBindKind::Unbound,
                    false,
                ));
            }
            return Ok((
                ProjectId::new(),
                OPENCODE_UNBOUND_ALIAS.to_string(),
                OpenCodeBindKind::Unbound,
                true,
            ));
        }
    };

    let alias = normalize_opencode_project_hash(raw);
    if alias == OPENCODE_UNBOUND_ALIAS {
        if allow_default_project {
            return Ok((
                default_project_id,
                OPENCODE_UNBOUND_ALIAS.to_string(),
                OpenCodeBindKind::Default,
                false,
            ));
        }
        if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(OPENCODE_UNBOUND_ALIAS) {
            return Ok((
                pid,
                OPENCODE_UNBOUND_ALIAS.to_string(),
                OpenCodeBindKind::Unbound,
                false,
            ));
        }
        return Ok((
            ProjectId::new(),
            OPENCODE_UNBOUND_ALIAS.to_string(),
            OpenCodeBindKind::Unbound,
            true,
        ));
    }

    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(&alias) {
        return Ok((pid, alias, kind, false));
    }

    Ok((ProjectId::new(), alias, kind, true))
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

// ---------------------------------------------------------------------------
// Import options / stats / discovery
// ---------------------------------------------------------------------------

/// Options for [`import_opencode_sessions`].
#[derive(Debug, Clone)]
pub struct OpenCodeImportOptions {
    pub days: usize,
    pub force: bool,
    pub dry_run: bool,
    pub max_sessions: usize,
    pub default_project_id: ProjectId,
    pub allow_default_project: bool,
    /// Hermetic: inject session list JSON instead of spawning `opencode`.
    pub list_json_override: Option<String>,
    /// Hermetic: directory of `{sessionId}.json` export fixtures (or file map via path).
    pub export_json_override_dir: Option<PathBuf>,
    /// Override cursor path (default `~/.ai-brains/opencode-import-cursor.json`).
    pub cursor_path_override: Option<PathBuf>,
    /// Hermetic / relocated: set `OPENCODE_CONFIG_DIR` on list+export subprocesses.
    pub config_dir_override: Option<PathBuf>,
    /// When set, treat binary as missing (hermetic AC12).
    pub force_missing_binary: bool,
    /// Override list cap for AC23 tests (default [`OPENCODE_LIST_DEFAULT_CAP`]).
    pub list_cap: usize,
}

impl OpenCodeImportOptions {
    pub fn new(days: usize, default_project_id: ProjectId) -> Self {
        Self {
            days,
            force: false,
            dry_run: false,
            max_sessions: OPENCODE_LIST_DEFAULT_CAP,
            default_project_id,
            allow_default_project: false,
            list_json_override: None,
            export_json_override_dir: None,
            cursor_path_override: None,
            config_dir_override: None,
            force_missing_binary: false,
            list_cap: OPENCODE_LIST_DEFAULT_CAP,
        }
    }
}

/// Import counters (F23) — printed as human stderr.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenCodeImportStats {
    pub found: usize,
    pub exported: usize,
    pub imported_turns: usize,
    pub skipped_watermark: usize,
    pub skipped_days: usize,
    pub skipped_missing_binary: usize,
    pub skipped_child_session: usize,
    pub export_errors: usize,
    pub unbound_project: usize,
    pub bound_via_worktree: usize,
    pub bound_via_directory: usize,
    pub timed_out: usize,
    pub list_capped: usize,
    pub sessions: usize,
}

/// A discovered OpenCode session from list JSON.
#[derive(Debug, Clone)]
pub struct OpenCodeSessionSource {
    pub id: String,
    pub directory: Option<String>,
    pub worktree: Option<String>,
    pub project_id_field: Option<String>,
    pub updated_ms: Option<i64>,
    pub parent_id: Option<String>,
}

/// Parse `opencode session list --format json` output (array of session objects).
pub fn parse_session_list_json(raw: &str) -> Result<Vec<OpenCodeSessionSource>> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| AdapterError::Other(format!("parse session list JSON: {e}")))?;
    let arr = match v {
        Value::Array(a) => a,
        Value::Object(ref map) if map.contains_key("sessions") => map
            .get("sessions")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => {
            return Err(AdapterError::Other(
                "session list JSON must be an array or {sessions:[]}".into(),
            ));
        }
    };

    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if id.is_empty() {
            continue;
        }
        // Tolerant projectId / projectID (F17).
        let project_id_field = item
            .get("projectId")
            .or_else(|| item.get("projectID"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let directory = item
            .get("directory")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let worktree = item
            .get("worktree")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let updated_ms = item
            .get("updated")
            .or_else(|| item.get("time").and_then(|t| t.get("updated")))
            .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)));
        let parent_id = item
            .get("parentID")
            .or_else(|| item.get("parentId"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string);

        out.push(OpenCodeSessionSource {
            id,
            directory,
            worktree,
            project_id_field,
            updated_ms,
            parent_id,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Cursor watermark
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct ImportCursor {
    /// session id → last updated ms
    sessions: HashMap<String, i64>,
    /// Optional additive: last observed message id per session (non-breaking).
    last_msg_ids: HashMap<String, String>,
}

fn load_cursor(path: &Path) -> ImportCursor {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return ImportCursor::default();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        // Corrupt cursor: warn once and start empty (force still recovers).
        eprintln!(
            "[OpenCode] cursor_corrupt: could not parse {}; starting empty watermark",
            path.display()
        );
        return ImportCursor::default();
    };
    let mut sessions = HashMap::new();
    let mut last_msg_ids = HashMap::new();
    if let Some(map) = v.get("sessions").and_then(|s| s.as_object()) {
        for (k, val) in map {
            if let Some(ms) = val.as_i64().or_else(|| val.as_u64().map(|u| u as i64)) {
                sessions.insert(k.clone(), ms);
            } else if let Some(ms) = val
                .get("updated_ms")
                .and_then(|u| u.as_i64().or_else(|| u.as_u64().map(|x| x as i64)))
            {
                sessions.insert(k.clone(), ms);
                if let Some(mid) = val
                    .get("last_msg_id")
                    .and_then(|m| m.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    last_msg_ids.insert(k.clone(), mid.to_string());
                }
            }
        }
    }
    ImportCursor {
        sessions,
        last_msg_ids,
    }
}

fn save_cursor(path: &Path, cursor: &ImportCursor) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut map = serde_json::Map::new();
    let mut sess = serde_json::Map::new();
    let mut keys: Vec<&String> = cursor.sessions.keys().collect();
    keys.sort();
    for k in keys {
        if let Some(ms) = cursor.sessions.get(k) {
            if let Some(mid) = cursor.last_msg_ids.get(k) {
                let mut entry = serde_json::Map::new();
                entry.insert("updated_ms".to_string(), Value::from(*ms));
                entry.insert("last_msg_id".to_string(), Value::from(mid.clone()));
                sess.insert(k.clone(), Value::Object(entry));
            } else {
                sess.insert(k.clone(), Value::from(*ms));
            }
        }
    }
    map.insert("sessions".to_string(), Value::Object(sess));
    let body = serde_json::to_string_pretty(&Value::Object(map))
        .map_err(|e| AdapterError::Other(format!("serialize cursor: {e}")))?;
    // Atomic-ish write via temp + rename
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &body)?;
    std::fs::rename(&tmp, path).or_else(|_| {
        std::fs::copy(&tmp, path)?;
        std::fs::remove_file(&tmp)
    })?;
    Ok(())
}

fn default_cursor_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".ai-brains").join(OPENCODE_IMPORT_CURSOR_FILENAME))
}

// ---------------------------------------------------------------------------
// Export subprocess / inject
// ---------------------------------------------------------------------------

fn export_session(
    session_id: &str,
    options: &OpenCodeImportOptions,
) -> std::result::Result<Value, ExportErr> {
    // Injected fixture path for hermetic tests (never open opencode.db).
    if let Some(dir) = options.export_json_override_dir.as_ref() {
        let candidates = [
            dir.join(format!("{session_id}.json")),
            dir.join(session_id),
            dir.join("export.json"),
        ];
        for c in &candidates {
            if c.is_file() {
                let raw = std::fs::read_to_string(c).map_err(|e| ExportErr::Io(e.to_string()))?;
                let v: Value =
                    serde_json::from_str(&raw).map_err(|e| ExportErr::Parse(e.to_string()))?;
                return Ok(v);
            }
        }
        return Err(ExportErr::MissingFixture(session_id.to_string()));
    }

    // Real CLI export with timeout + child kill. Never opens opencode.db (AC14).
    let output = run_opencode_export_blocking(
        session_id,
        Duration::from_secs(OPENCODE_EXPORT_TIMEOUT_SECS),
        options.config_dir_override.as_deref(),
    )?;
    let v: Value = serde_json::from_str(&output).map_err(|e| ExportErr::Parse(e.to_string()))?;
    Ok(v)
}

enum ExportErr {
    Timeout,
    Io(String),
    Parse(String),
    MissingFixture(String),
    Binary,
    NonZero(String),
}

/// Public F12 live fallback: run `opencode export <sessionId>` with 120s timeout.
///
/// Returns parsed turns, or empty on soft failure (missing binary, timeout, parse).
/// **Never** opens `opencode.db`.
pub fn export_session_via_cli(session_id: &str) -> Result<Vec<OpenCodeIngestTurn>> {
    match run_opencode_export_blocking(
        session_id,
        Duration::from_secs(OPENCODE_EXPORT_TIMEOUT_SECS),
        None,
    ) {
        Ok(stdout) => {
            let v: Value = serde_json::from_str(&stdout)
                .map_err(|e| AdapterError::Other(format!("opencode export parse: {e}")))?;
            Ok(filter_opencode_export(&v))
        }
        Err(ExportErr::Binary) => {
            // Soft: binary missing — live path fail-open empty
            Ok(Vec::new())
        }
        Err(ExportErr::Timeout) => Ok(Vec::new()),
        Err(e) => {
            // Soft fail-open for non-parse; parse errors already handled above
            let _ = e;
            Ok(Vec::new())
        }
    }
}

/// Spawn `opencode` with optional config-dir env, poll until exit or deadline, kill on timeout.
///
/// Reads stdout/stderr on background threads so large exports cannot deadlock the pipe buffer.
fn run_opencode_command_blocking(
    args: &[&str],
    timeout: Duration,
    config_dir: Option<&Path>,
) -> std::result::Result<String, ExportErr> {
    let mut cmd = Command::new("opencode");
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = config_dir {
        cmd.env("OPENCODE_CONFIG_DIR", dir);
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ExportErr::Binary
        } else {
            ExportErr::Io(e.to_string())
        }
    })?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut out) = stdout_pipe {
            let _ = out.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut err) = stderr_pipe {
            let _ = err.read_to_end(&mut buf);
        }
        buf
    });

    let start = Instant::now();
    let poll = Duration::from_millis(50);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    // Drain readers so we do not leak join handles with open pipes.
                    let _ = stdout_handle.join();
                    let _ = stderr_handle.join();
                    return Err(ExportErr::Timeout);
                }
                std::thread::sleep(poll);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                return Err(ExportErr::Io(e.to_string()));
            }
        }
    };

    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();

    if !status.success() {
        return Err(ExportErr::NonZero(
            String::from_utf8_lossy(&stderr).to_string(),
        ));
    }
    String::from_utf8(stdout).map_err(|e| ExportErr::Io(e.to_string()))
}

fn run_opencode_export_blocking(
    session_id: &str,
    timeout: Duration,
    config_dir: Option<&Path>,
) -> std::result::Result<String, ExportErr> {
    run_opencode_command_blocking(&["export", session_id], timeout, config_dir)
}

fn run_opencode_list(
    max_n: usize,
    config_dir: Option<&Path>,
) -> std::result::Result<String, ExportErr> {
    let n = max_n.to_string();
    run_opencode_command_blocking(
        &["session", "list", "--format", "json", "-n", &n],
        Duration::from_secs(OPENCODE_EXPORT_TIMEOUT_SECS),
        config_dir,
    )
}

// ---------------------------------------------------------------------------
// Import orchestration
// ---------------------------------------------------------------------------

/// Orchestrate import of OpenCode sessions via list + export + watermark.
///
/// **Never** opens `opencode.db` as content SOOT (AC14) — only CLI export or injected fixtures.
pub fn import_opencode_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: OpenCodeImportOptions,
) -> Result<OpenCodeImportStats> {
    let mut stats = OpenCodeImportStats::default();

    // Discovery
    let list_raw = if let Some(ref injected) = options.list_json_override {
        injected.clone()
    } else if options.force_missing_binary {
        stats.skipped_missing_binary = 1;
        eprintln!("[OpenCode] opencode binary missing — soft skip (no sessions imported).");
        return Ok(stats);
    } else {
        match run_opencode_list(
            options.max_sessions.max(1),
            options.config_dir_override.as_deref(),
        ) {
            Ok(s) => s,
            Err(ExportErr::Binary) => {
                stats.skipped_missing_binary = 1;
                eprintln!(
                    "[OpenCode] opencode binary not on PATH — soft skip (batch requires opencode)."
                );
                return Ok(stats);
            }
            Err(ExportErr::Timeout) => {
                stats.timed_out += 1;
                stats.export_errors += 1;
                eprintln!("[OpenCode] session list timed out — soft skip.");
                return Ok(stats);
            }
            Err(e) => {
                stats.export_errors += 1;
                eprintln!("[OpenCode] session list failed: {}", export_err_msg(&e));
                return Ok(stats);
            }
        }
    };

    let mut sources = parse_session_list_json(&list_raw)?;

    // AC23: list length at user cap OR vendor default hard cap (100) → warn.
    // Even when --max-sessions > 100, a 100-row result may be vendor-capped.
    let list_cap = options.list_cap.max(1);
    if sources.len() >= OPENCODE_LIST_DEFAULT_CAP
        || sources.len() >= list_cap
        || sources.len() >= options.max_sessions
    {
        stats.list_capped = 1;
        eprintln!(
            "[OpenCode] list_capped: returned {} session(s) at cap (max-sessions={}, list_cap={}, vendor_default={}); older sessions may be missing",
            sources.len(),
            options.max_sessions,
            list_cap,
            OPENCODE_LIST_DEFAULT_CAP
        );
    }

    // Cap processing
    if sources.len() > options.max_sessions {
        sources.truncate(options.max_sessions);
    }

    let cutoff_ms = {
        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let window = (options.days as i64).saturating_mul(24 * 60 * 60 * 1000);
        now_ms.saturating_sub(window)
    };

    let cursor_path = options
        .cursor_path_override
        .clone()
        .or_else(default_cursor_path);
    let mut cursor = cursor_path
        .as_ref()
        .map(|p| load_cursor(p))
        .unwrap_or_default();

    // Filter days / child / watermark
    let mut eligible: Vec<OpenCodeSessionSource> = Vec::new();
    for src in sources {
        if src.parent_id.is_some() {
            stats.skipped_child_session += 1;
            continue;
        }
        if let Some(updated) = src.updated_ms {
            if updated < cutoff_ms && !options.force {
                stats.skipped_days += 1;
                continue;
            }
            if !options.force
                && let Some(&prev) = cursor.sessions.get(&src.id)
                && updated <= prev
            {
                stats.skipped_watermark += 1;
                continue;
            }
        } else if !options.force {
            // No updated field: still process unless watermark says we saw it with 0
            if cursor.sessions.contains_key(&src.id) && !options.force {
                stats.skipped_watermark += 1;
                continue;
            }
        }
        eligible.push(src);
    }

    stats.found = eligible.len();
    if stats.found == 0 {
        return Ok(stats);
    }

    eprintln!(
        "[OpenCode] Found {} session(s) to process (days={}, force={}).",
        stats.found, options.days, options.force
    );
    if options.dry_run {
        eprintln!("[OpenCode] dry-run mode: scanning only — no vault writes.");
    }

    let oc_harness = HarnessId::from_str(OPENCODE_HARNESS_UUID)
        .map_err(|e| AdapterError::Other(format!("Invalid static OpenCode harness ID: {e}")))?;

    for (idx, source) in eligible.iter().enumerate() {
        if (idx + 1) % 10 == 0 || idx == 0 || idx + 1 == stats.found {
            eprintln!(
                "[OpenCode] Processing session {}/{} ({})...",
                idx + 1,
                stats.found,
                source.id
            );
        }

        let export_doc = match export_session(&source.id, &options) {
            Ok(v) => {
                stats.exported += 1;
                v
            }
            Err(ExportErr::Timeout) => {
                stats.timed_out += 1;
                stats.export_errors += 1;
                eprintln!(
                    "[OpenCode] export timeout for {} — skip (fail-open)",
                    source.id
                );
                continue;
            }
            Err(ExportErr::Binary) => {
                stats.skipped_missing_binary += 1;
                eprintln!("[OpenCode] opencode binary missing during export — stop.");
                break;
            }
            Err(e) => {
                stats.export_errors += 1;
                eprintln!(
                    "[OpenCode] export error for {}: {} — skip",
                    source.id,
                    export_err_msg(&e)
                );
                continue;
            }
        };

        // Prefer worktree from list; directory from list or export.info.directory
        let export_dir = export_doc
            .get("info")
            .and_then(|i| i.get("directory"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let worktree = source.worktree.as_deref();
        let directory = source.directory.as_deref().or(export_dir.as_deref());

        // Parent skip from export info if present
        if let Some(parent) = export_doc
            .get("info")
            .and_then(|i| i.get("parentID").or_else(|| i.get("parentId")))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
        {
            let _ = parent;
            stats.skipped_child_session += 1;
            continue;
        }

        let turns = parse_export_json(&export_doc);
        let session_id = session_id_from_opencode(&source.id);

        // Delta: skip by max turn index (msg-id SOOT still stable on re-ingest of same kept set)
        let max_turn = query_store.get_max_turn_index(&session_id).unwrap_or(None);
        let next_index = max_turn.map(|m| m + 1).unwrap_or(0);
        if turns.len() <= next_index as usize {
            if !options.dry_run
                && let Some(updated) = source.updated_ms
            {
                cursor.sessions.insert(source.id.clone(), updated);
            }
            continue;
        }

        // T239 D21/F22: soft-fail capture unit; prior sessions + health counters kept.
        let session_result: crate::errors::Result<()> = (|| {
            let (mut project_id, alias, kind, needs_create) = resolve_opencode_project(
                worktree,
                directory,
                query_store,
                options.allow_default_project,
                options.default_project_id,
            )?;

            match kind {
                OpenCodeBindKind::Worktree => stats.bound_via_worktree += 1,
                OpenCodeBindKind::Directory => stats.bound_via_directory += 1,
                OpenCodeBindKind::Unbound => stats.unbound_project += 1,
                OpenCodeBindKind::Default => {}
            }

            if options.dry_run {
                return Ok(());
            }

            if needs_create {
                let display = if alias == OPENCODE_UNBOUND_ALIAS {
                    OPENCODE_UNBOUND_DISPLAY_NAME.to_string()
                } else {
                    path_derived_display_name(&alias)
                };
                if let Ok(Some(existing)) = query_store.resolve_project_id_from_alias(&alias) {
                    project_id = existing;
                } else {
                    ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
                }
            } else if kind != OpenCodeBindKind::Default
                && query_store
                    .resolve_project_id_from_alias(&alias)
                    .ok()
                    .flatten()
                    .is_none()
            {
                let display = if alias == OPENCODE_UNBOUND_ALIAS {
                    OPENCODE_UNBOUND_DISPLAY_NAME.to_string()
                } else {
                    path_derived_display_name(&alias)
                };
                ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
            }

            let capture_context = CaptureContext {
                git_working_dir: std::env::current_dir().ok(),
            };

            service.start_session(
                ai_brains_capture::SessionStartCommand {
                    session_id,
                    project_id,
                    harness_id: oc_harness,
                    privacy: Privacy::LocalOnly,
                    tx_id: None,
                },
                capture_context.clone(),
                sink,
            )?;

            stats.imported_turns += append_opencode_turns(
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
                    harness_id: oc_harness,
                    privacy: Privacy::LocalOnly,
                    status: SessionStopStatus::Completed,
                    reason: Some("OpenCode export import complete".to_string()),
                },
                capture_context,
                sink,
            )?;

            // Path-keyed source meta
            let meta_key = opencode_source_meta_key(&source.id);
            let meta_val = format!("{}:{}", source.updated_ms.unwrap_or(0), turns.len());
            sink.set_sync_state(&meta_key, &meta_val);

            if let Some(updated) = source.updated_ms {
                cursor.sessions.insert(source.id.clone(), updated);
            } else {
                cursor.sessions.insert(source.id.clone(), 0);
            }
            // Additive non-breaking: remember last msg_* when present (delta remains index-based).
            if let Some(last_mid) = turns
                .iter()
                .rev()
                .find_map(|t| t.msg_id.as_deref())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                cursor
                    .last_msg_ids
                    .insert(source.id.clone(), last_mid.to_string());
            }
            stats.sessions += 1;
            Ok(())
        })();

        if let Err(e) = session_result {
            eprintln!(
                "[OpenCode] session {} failed: {e} — continue (fail-open; prior sessions kept)",
                source.id
            );
        }
    }

    if !options.dry_run
        && let Some(ref path) = cursor_path
        && let Err(e) = save_cursor(path, &cursor)
    {
        eprintln!(
            "[OpenCode] warning: could not save cursor {}: {e}",
            path.display()
        );
    }

    Ok(stats)
}

fn export_err_msg(e: &ExportErr) -> String {
    match e {
        ExportErr::Timeout => "timeout".into(),
        ExportErr::Io(s) => format!("io: {s}"),
        ExportErr::Parse(s) => format!("parse: {s}"),
        ExportErr::MissingFixture(s) => format!("missing fixture for {s}"),
        ExportErr::Binary => "binary not found".into(),
        ExportErr::NonZero(s) => format!("exit non-zero: {s}"),
    }
}

/// Print F23 human stats to stderr.
pub fn print_opencode_import_stats(stats: &OpenCodeImportStats) {
    eprintln!(
        "[OpenCode] Import stats: found={} exported={} imported_turns={} sessions={} skipped_watermark={} skipped_days={} skipped_missing_binary={} skipped_child_session={} export_errors={} unbound_project={} bound_via_worktree={} bound_via_directory={} timed_out={} list_capped={}",
        stats.found,
        stats.exported,
        stats.imported_turns,
        stats.sessions,
        stats.skipped_watermark,
        stats.skipped_days,
        stats.skipped_missing_binary,
        stats.skipped_child_session,
        stats.export_errors,
        stats.unbound_project,
        stats.bound_via_worktree,
        stats.bound_via_directory,
        stats.timed_out,
        stats.list_capped
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn generate_opencode_turn_id__msg_id_stable() {
        let sid = session_id_from_opencode("ses_abc");
        let a = generate_opencode_turn_id(&sid, Some("msg_1"), 0);
        let b = generate_opencode_turn_id(&sid, Some("msg_1"), 99);
        assert_eq!(a, b, "msg_id wins over index");
        let c = generate_opencode_turn_id(&sid, None, 0);
        let d = generate_opencode_turn_id(&sid, None, 0);
        assert_eq!(c, d);
        assert_ne!(a, c);
    }

    #[test]
    fn session_id_from_opencode__ses_prefix_stable() {
        let a = session_id_from_opencode("ses_hello");
        let b = session_id_from_opencode("ses_hello");
        assert_eq!(a, b);
        assert_ne!(a, session_id_from_opencode("ses_other"));
    }

    #[test]
    fn normalize_opencode_project_hash__unbound_and_path() {
        assert_eq!(normalize_opencode_project_hash(""), OPENCODE_UNBOUND_ALIAS);
        assert_eq!(
            normalize_opencode_project_hash("opencode-unbound"),
            OPENCODE_UNBOUND_ALIAS
        );
        let a = normalize_opencode_project_hash(r"C:\dev\AI-Brains");
        let b = normalize_opencode_project_hash(r"c:\dev\ai-brains");
        // Path normalize should collapse case on Windows
        assert_eq!(a.to_ascii_lowercase(), b.to_ascii_lowercase());
    }

    #[test]
    fn opencode_env_fallback_allowed__only_unbound() {
        assert!(opencode_env_fallback_allowed(""));
        assert!(opencode_env_fallback_allowed("opencode-unbound"));
        assert!(!opencode_env_fallback_allowed(r"C:\dev\x"));
    }

    #[test]
    fn parse_session_list__projectId_tolerant() {
        let raw = r#"[
          {"id":"ses_1","projectId":"p1","directory":"C:\\a","updated":1700000000000},
          {"id":"ses_2","projectID":"p2","directory":"C:\\b","updated":1700000001000,"parentID":"ses_1"},
          {"id":"ses_3","worktree":"C:\\wt","updated":1700000002000}
        ]"#;
        let list = parse_session_list_json(raw).expect("parse");
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].project_id_field.as_deref(), Some("p1"));
        assert_eq!(list[1].project_id_field.as_deref(), Some("p2"));
        assert_eq!(list[1].parent_id.as_deref(), Some("ses_1"));
        assert_eq!(list[2].worktree.as_deref(), Some("C:\\wt"));
    }

    #[test]
    fn opencode_capability__full_hooks() {
        let c = opencode_capability();
        assert_eq!(c.level, CapabilityLevel::Full);
        assert!(c.supports_hooks);
        assert!(c.notes.contains("never opens opencode.db") || c.notes.contains("opencode.db"));
        assert!(c.notes.contains("parentID") || c.notes.contains("Child"));
        // F14 honesty: delta is index/watermark class, not pure msg_id existence
        assert!(
            c.notes.contains("turn_index") || c.notes.contains("max turn_index"),
            "capability notes must disclose index-based delta residual"
        );
    }

    #[test]
    fn load_cursor__corrupt_json__empty_with_warn_path() {
        // Exercise parse-fail branch (eprintln once in real use).
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cursor.json");
        std::fs::write(&path, b"not-json{{{").expect("write");
        let c = load_cursor(&path);
        assert!(c.sessions.is_empty());
        assert!(c.last_msg_ids.is_empty());
    }

    #[test]
    fn load_save_cursor__last_msg_id_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("cursor.json");
        let mut c = ImportCursor::default();
        c.sessions.insert("ses_a".into(), 42);
        c.last_msg_ids.insert("ses_a".into(), "msg_last".into());
        save_cursor(&path, &c).expect("save");
        let loaded = load_cursor(&path);
        assert_eq!(loaded.sessions.get("ses_a"), Some(&42));
        assert_eq!(
            loaded.last_msg_ids.get("ses_a").map(String::as_str),
            Some("msg_last")
        );
    }
}
