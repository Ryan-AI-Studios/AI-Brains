use crate::agy::{
    AGY_UNBOUND_ALIAS, AGY_UNBOUND_DISPLAY_NAME, TranscriptIngestTurn, agy_source_meta_key,
    generate_turn_id_for_ingest, normalize_agy_project_hash, parse_transcript_for_ingest,
    path_derived_display_name,
};
use crate::capability::{AdapterCapability, CapabilityLevel};
use crate::errors::Result;
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_contracts::ingest::IngestRequest;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

pub fn antigravity_capability() -> AdapterCapability {
    AdapterCapability {
        name: "antigravity".to_string(),
        // F21/F23: live hooks + batch history bind + re-summarize shipped → Full.
        // Residual notes: scheduled SYSTEM --skip-import; connector principal_binding deferred.
        level: CapabilityLevel::Full,
        supports_hooks: true,
        supports_wrapper_mode: false,
        notes: "Live Stop hooks via `ai-brains harness install --harness agy` (wrapper stdout allow-stop JSON; step-shaped + message-only SOOT). Batch `antigravity-import` binds conversationId→workspace via history.jsonl; unbound brains use stable `agy-unbound` / `(unbound AGY)` (allow_default_project=false by default). Prefer transcript_full when present. SYSTEM scheduled nightly may still use --skip-import (T239). Intended PrincipalKind::Connector binding deferred."
            .to_string(),
        governed_reads: Vec::new(),
        governed_writes: Vec::new(),
        principal_binding: None,
    }
}

pub fn manual_import_instructions() -> String {
    "Antigravity sessions are imported by `ai-brains antigravity-import` or manual `ai-brains nightly` (without --skip-import). Scheduled SYSTEM nightly may still pass --skip-import. Reinstall hooks after T236: `ai-brains harness install --harness agy`. Manual pinning is still recommended for decisions mid-session.".to_string()
}

/// A single step from an Antigravity overview/transcript JSONL file.
#[derive(Debug, Clone, Deserialize)]
pub struct AntigravityStep {
    #[serde(default)]
    pub step_index: u32,
    #[serde(default)]
    pub source: String,
    #[serde(rename = "type", default)]
    pub step_type: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}

/// A conversation turn extracted from Antigravity logs, ready for ingestion.
#[derive(Debug, Clone)]
pub struct AntigravityTurn {
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

/// Options for [`import_antigravity_sessions`] (T236).
#[derive(Debug, Clone)]
pub struct AntigravityImportOptions {
    pub days: usize,
    pub default_project_id: ProjectId,
    /// When false (default for non-interactive nightly/import), unbound brains do
    /// **not** attach to `default_project_id` / cwd env project (F12).
    pub allow_default_project: bool,
    /// Skip the 300s quiescence window (F18).
    pub force: bool,
    /// Hermetic tests: discover brains + history under this home instead of dirs::home_dir.
    pub home_override: Option<PathBuf>,
}

impl AntigravityImportOptions {
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

/// Import counters (F16) — printed as human stderr; not a JSON status object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AntigravityImportStats {
    pub found: usize,
    pub imported_turns: usize,
    pub sessions: usize,
    pub skipped_quiescent: usize,
    pub skipped_unchanged_meta: usize,
    pub unbound_project: usize,
    pub bound_via_history: usize,
    pub bound_via_path: usize,
}

/// Discover Antigravity brain directories containing overview.txt / transcript.jsonl.
/// Scans ~/.gemini/{antigravity,antigravity-cli,antigravity-ide}/brain/ and tmp chats.
pub fn discover_sessions() -> Result<Vec<AntigravitySessionSource>> {
    discover_sessions_from_home(dirs::home_dir().as_deref(), true)
}

/// Discover under an explicit home (hermetic tests / import home_override).
pub fn discover_sessions_from_home(
    home: Option<&Path>,
    include_wsl_legacy: bool,
) -> Result<Vec<AntigravitySessionSource>> {
    let mut all_sources = Vec::new();

    if let Some(home) = home {
        let gemini_base = home.join(".gemini");

        let tool_dirs = ["antigravity", "antigravity-cli", "antigravity-ide"];
        for tool in tool_dirs {
            let brain_path = gemini_base.join(tool).join("brain");
            if brain_path.exists() {
                scan_brain_dir(&brain_path, &mut all_sources)?;
            }
        }

        let tmp_path = gemini_base.join("tmp");
        if tmp_path.exists() {
            scan_tmp_dirs(&tmp_path, &mut all_sources)?;
        }
    }

    if include_wsl_legacy {
        let wsl_brain = PathBuf::from(r"\\wsl$\Ubuntu\home\ryan\.gemini\antigravity\brain");
        if wsl_brain.exists() {
            scan_brain_dir(&wsl_brain, &mut all_sources)?;
        }
    }

    Ok(all_sources)
}

#[derive(Debug, Clone)]
pub enum AntigravityFormat {
    BrainLog,    // overview.txt or transcript.jsonl
    ProjectChat, // session-*.jsonl
}

#[derive(Debug, Clone)]
pub struct AntigravitySessionSource {
    pub path: PathBuf,
    pub session_id: String,
    pub format: AntigravityFormat,
    pub project_hash: Option<String>,
}

fn scan_brain_dir(brain_dir: &Path, sources: &mut Vec<AntigravitySessionSource>) -> Result<()> {
    let entries = std::fs::read_dir(brain_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let session_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Prefer transcript.jsonl over overview.txt when both exist (F29 / AC21).
        // Dual enqueue + count-based delta would otherwise lock overview content and
        // skip the transcript path that prefers sibling transcript_full.jsonl.
        let logs = path.join(".system_generated").join("logs");
        let transcript = logs.join("transcript.jsonl");
        let overview = logs.join("overview.txt");
        if transcript.exists() {
            sources.push(AntigravitySessionSource {
                path: transcript,
                session_id,
                format: AntigravityFormat::BrainLog,
                project_hash: None,
            });
        } else if overview.exists() {
            sources.push(AntigravitySessionSource {
                path: overview,
                session_id,
                format: AntigravityFormat::BrainLog,
                project_hash: None,
            });
        }
    }
    Ok(())
}

fn scan_tmp_dirs(tmp_base: &Path, sources: &mut Vec<AntigravitySessionSource>) -> Result<()> {
    let project_entries = std::fs::read_dir(tmp_base)?;
    for project_entry in project_entries {
        let project_entry = project_entry?;
        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let project_hash = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        let chats_dir = project_path.join("chats");
        if chats_dir.exists() {
            let chat_entries = std::fs::read_dir(chats_dir)?;
            for chat_entry in chat_entries {
                let chat_entry = chat_entry?;
                let chat_path = chat_entry.path();
                if chat_path.is_file() && chat_path.extension().is_some_and(|ext| ext == "jsonl") {
                    let session_id = chat_path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .map(|s| s.replace("session-", ""))
                        .unwrap_or_default();

                    sources.push(AntigravitySessionSource {
                        path: chat_path,
                        session_id,
                        format: AntigravityFormat::ProjectChat,
                        project_hash: project_hash.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Extract the conversation ID (directory name) from an overview.txt path.
pub fn session_id_from_path(path: &Path) -> Option<String> {
    // Path: .../brain/<conversation-id>/.system_generated/logs/overview.txt
    path.parent() // logs/
        .and_then(|p| p.parent()) // .system_generated/
        .and_then(|p| p.parent()) // <conversation-id>/
        .and_then(|p| p.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

/// Parse an Antigravity overview.txt JSONL file into steps.
pub fn parse_overview_file(path: &Path) -> Result<Vec<AntigravityStep>> {
    let content = std::fs::read_to_string(path)?;
    let mut steps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<AntigravityStep>(line) {
            Ok(step) => steps.push(step),
            Err(_) => continue, // Skip malformed lines
        }
    }

    Ok(steps)
}

/// Extract ingestable turns from Antigravity steps via shared message-only SOOT (T234).
///
/// Capture Privacy: user + final assistant text only. Strict `(source, type)` match;
/// VIEW_FILE / RUN_COMMAND / TOOL_OUTPUT dropped regardless of content.
pub fn extract_turns(steps: &[AntigravityStep]) -> Vec<AntigravityTurn> {
    steps
        .iter()
        .filter_map(|step| {
            crate::message_only::classify_antigravity_step(
                step.source.as_str(),
                step.step_type.as_str(),
                step.content.as_deref(),
                &step.tool_calls,
                step.created_at.as_deref(),
            )
            .map(|turn| AntigravityTurn {
                role: turn.role.as_str().to_string(),
                content: turn.content,
                created_at: turn.source_ts,
            })
        })
        .collect()
}

/// Strip Antigravity XML metadata tags from user input content (delegates to message_only SOOT).
pub fn strip_user_xml_tags(content: &str) -> String {
    crate::message_only::extract_user_text(content)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChatTurn {
    #[allow(dead_code)]
    pub id: String,
    #[serde(rename = "type")]
    pub turn_type: String, // e.g. "user", "gemini", "claude"
    pub content: String,
    #[allow(dead_code)]
    pub thoughts: Option<String>,
}

/// Parse a project-specific session-*.jsonl file via message-only SOOT (T234).
///
/// Model type names (`gemini`, `claude`, …) map to assistant; `thoughts` is never stored.
pub fn parse_project_chat_file(path: &Path) -> Result<Vec<AntigravityTurn>> {
    let content = std::fs::read_to_string(path)?;
    let mut turns = Vec::new();

    let mut lines = content.lines();
    // Skip header line
    let _header = lines.next();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(turn) = serde_json::from_str::<ProjectChatTurn>(line) {
            // Map harness type → ingest role; tool_output/system drop via filter_turn.
            let role = match turn.turn_type.as_str() {
                "user" => "user",
                "gemini" | "claude" | "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-5.3-codex"
                | "gpt-5.5-thinking" | "assistant" => "assistant",
                // tool_output, system, unknown → not user/assistant; filter_turn drops
                other => other,
            };

            // thoughts field intentionally ignored (never concatenated into content).
            if let Some(ingested) = crate::message_only::filter_turn(role, &turn.content) {
                turns.push(AntigravityTurn {
                    role: ingested.role.as_str().to_string(),
                    content: ingested.content,
                    created_at: None,
                });
            }
        }
    }

    Ok(turns)
}

// ---------------------------------------------------------------------------
// History index (F9–F11)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HistoryLine {
    #[allow(dead_code)]
    display: Option<String>,
    timestamp: Option<serde_json::Value>,
    workspace: Option<String>,
    conversation_id: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_: Option<String>,
}

/// Load `conversationId → normalized workspace` from history.jsonl (F9).
///
/// Rows need non-empty workspace + conversationId. Sort by
/// `(timestamp_ms asc, line_index asc)`; last wins. Timestamp parse fail → 0.
pub fn load_agy_history_index(history_path: &Path) -> HashMap<String, String> {
    let Ok(content) = std::fs::read_to_string(history_path) else {
        return HashMap::new();
    };

    let mut rows: Vec<(i64, usize, String, String)> = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(rec) = serde_json::from_str::<HistoryLine>(line) else {
            continue;
        };
        let Some(cid) = rec.conversation_id.filter(|s| !s.trim().is_empty()) else {
            continue;
        };
        let Some(ws) = rec.workspace.filter(|s| !s.trim().is_empty()) else {
            continue;
        };
        let ts = parse_history_timestamp_ms(rec.timestamp.as_ref());
        let normalized = normalize_agy_project_hash(&ws);
        if normalized == AGY_UNBOUND_ALIAS {
            continue;
        }
        rows.push((ts, line_index, cid, normalized));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut map = HashMap::new();
    for (_ts, _idx, cid, ws) in rows {
        map.insert(cid, ws);
    }
    map
}

fn parse_history_timestamp_ms(v: Option<&serde_json::Value>) -> i64 {
    match v {
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_u64().map(|u| u as i64))
            .unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.trim().parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

/// Load history index from AGY2 + optional legacy paths under home.
pub fn load_agy_history_index_from_home(home: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let primary = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("history.jsonl");
    let legacy = home
        .join(".gemini")
        .join("antigravity")
        .join("history.jsonl");
    // Legacy first, primary last so primary wins on same conversationId
    if legacy.is_file() {
        for (k, v) in load_agy_history_index(&legacy) {
            map.insert(k, v);
        }
    }
    if primary.is_file() {
        for (k, v) in load_agy_history_index(&primary) {
            map.insert(k, v);
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Project resolve (F3 / F12)
// ---------------------------------------------------------------------------

/// How a session was bound to a project (F16 counters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgyBindKind {
    History,
    Path,
    Unbound,
    Default,
}

/// Resolve AGY project hash/workspace to a project id (shared hook + batch, F3).
///
/// Ordered:
/// 1. Normalize hash
/// 2. Alias resolve
/// 3. (Repository path-alias skipped — no API in adapters)
/// 4. Env default **only** when hash is unbound/empty **and** `allow_default_project`
/// 5. Path-derived (alias = normalized path) or stable unbound
pub fn resolve_agy_project(
    project_hash: Option<&str>,
    query_store: &dyn ai_brains_store::QueryStore,
    allow_default_project: bool,
    default_project_id: ProjectId,
) -> Result<(ProjectId, String, AgyBindKind, bool)> {
    // Returns (project_id, alias_key, bind_kind, needs_create)
    // needs_create=true when alias is new and project must be registered.
    let raw = project_hash.unwrap_or("").trim();
    let alias = normalize_agy_project_hash(raw);

    if alias == AGY_UNBOUND_ALIAS {
        if allow_default_project {
            return Ok((
                default_project_id,
                AGY_UNBOUND_ALIAS.to_string(),
                AgyBindKind::Default,
                false,
            ));
        }
        if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(AGY_UNBOUND_ALIAS) {
            return Ok((
                pid,
                AGY_UNBOUND_ALIAS.to_string(),
                AgyBindKind::Unbound,
                false,
            ));
        }
        return Ok((
            ProjectId::new(),
            AGY_UNBOUND_ALIAS.to_string(),
            AgyBindKind::Unbound,
            true,
        ));
    }

    if let Ok(Some(pid)) = query_store.resolve_project_id_from_alias(&alias) {
        // History vs path: caller sets bind kind; here treat as path/history-resolved alias hit
        return Ok((pid, alias, AgyBindKind::Path, false));
    }

    Ok((ProjectId::new(), alias, AgyBindKind::Path, true))
}

fn ensure_project_registered<S: CaptureSink>(
    sink: &mut S,
    project_id: ProjectId,
    alias: &str,
    display_name: &str,
    query_store: &dyn ai_brains_store::QueryStore,
) -> Result<()> {
    // Re-check alias in case another source registered it mid-loop
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
    .map_err(|e| crate::errors::AdapterError::Other(format!("ProjectRegistered build: {e}")))?;
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
    .map_err(|e| crate::errors::AdapterError::Other(format!("ProjectAliasAdded build: {e}")))?;
    sink.append(alias_ev);
    Ok(())
}

// ---------------------------------------------------------------------------
// Import orchestration
// ---------------------------------------------------------------------------

/// Orchestrates the import of Antigravity sessions from all discovered locations.
pub fn import_antigravity_sessions<S: CaptureSink>(
    query_store: &dyn ai_brains_store::QueryStore,
    service: &CaptureService,
    sink: &mut S,
    options: AntigravityImportOptions,
) -> Result<AntigravityImportStats> {
    let home = options.home_override.clone().or_else(dirs::home_dir);
    let include_wsl = options.home_override.is_none();
    let all_sources = discover_sessions_from_home(home.as_deref(), include_wsl)?;

    let mut stats = AntigravityImportStats::default();
    if all_sources.is_empty() {
        return Ok(stats);
    }

    let history = home
        .as_ref()
        .map(|h| load_agy_history_index_from_home(h))
        .unwrap_or_default();

    // Filter by recency
    let recent_sources: Vec<AntigravitySessionSource> = all_sources
        .into_iter()
        .filter(|s| {
            if let Ok(metadata) = std::fs::metadata(&s.path)
                && let Ok(modified) = metadata.modified()
            {
                let cutoff =
                    SystemTime::now() - Duration::from_secs(options.days as u64 * 24 * 60 * 60);
                return modified >= cutoff;
            }
            false
        })
        .collect();

    stats.found = recent_sources.len();
    if stats.found == 0 {
        return Ok(stats);
    }
    eprintln!(
        "[Antigravity] Found {} sessions modified in the last {} days. Scanning for new turns...",
        stats.found, options.days
    );

    let antigravity_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| crate::errors::AdapterError::Other(format!("Invalid static ID: {}", e)))?;
    let agy_harness = HarnessId::from_str("00000000-0000-0000-0000-000000000002")
        .map_err(|e| crate::errors::AdapterError::Other(format!("Invalid static ID: {}", e)))?;

    for (idx, source) in recent_sources.iter().enumerate() {
        let metadata = std::fs::metadata(&source.path).ok();
        let mtime = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

        let meta_key = agy_source_meta_key(&source.path);
        let stored_meta = query_store.get_sync_state(&meta_key).unwrap_or(None);
        let current_meta = format!("{}:{}", mtime, size);

        if stored_meta.as_ref() == Some(&current_meta) {
            stats.skipped_unchanged_meta += 1;
            continue;
        }

        if (idx + 1) % 10 == 0 || idx == 0 || idx == stats.found - 1 {
            eprintln!(
                "[Antigravity] Scanning session {}/{}...",
                idx + 1,
                stats.found
            );
        }

        // Quiescence check (F18): skip if modified in the last 5 minutes unless --force
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

        let (turns, harness_id): (Vec<TranscriptIngestTurn>, HarnessId) = match source.format {
            AntigravityFormat::BrainLog => {
                // Shared parser: step-shaped transcript/overview + legacy role/content (F1/F2/F29)
                let parsed = parse_transcript_for_ingest(&source.path)?;
                (parsed, antigravity_harness)
            }
            AntigravityFormat::ProjectChat => {
                let chat = parse_project_chat_file(&source.path)?;
                let mapped = chat
                    .into_iter()
                    .map(|t| TranscriptIngestTurn {
                        role: t.role,
                        content: t.content,
                        timestamp: t.created_at,
                        step_index: None,
                    })
                    .collect();
                (mapped, agy_harness)
            }
        };

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

        // F9/F10: history bind for BrainLog (session_id == conversationId)
        let mut bind_kind = AgyBindKind::Unbound;
        let mut hash_for_resolve: Option<String> = source.project_hash.clone();

        if matches!(source.format, AntigravityFormat::BrainLog) {
            if let Some(ws) = history.get(&source.session_id) {
                hash_for_resolve = Some(ws.clone());
                bind_kind = AgyBindKind::History;
            } else if source.project_hash.is_none() {
                hash_for_resolve = None;
                bind_kind = AgyBindKind::Unbound;
            }
        } else if source.project_hash.is_some() {
            bind_kind = AgyBindKind::Path;
        }

        let (mut project_id, alias, resolved_kind, needs_create) = resolve_agy_project(
            hash_for_resolve.as_deref(),
            query_store,
            options.allow_default_project,
            options.default_project_id,
        )?;

        // Prefer history/path classification for counters when we had a history hit
        let final_kind =
            if bind_kind == AgyBindKind::History && resolved_kind != AgyBindKind::Unbound {
                AgyBindKind::History
            } else if resolved_kind == AgyBindKind::Default {
                AgyBindKind::Default
            } else if resolved_kind == AgyBindKind::Unbound || alias == AGY_UNBOUND_ALIAS {
                AgyBindKind::Unbound
            } else {
                AgyBindKind::Path
            };

        if needs_create {
            let display = if alias == AGY_UNBOUND_ALIAS {
                AGY_UNBOUND_DISPLAY_NAME.to_string()
            } else {
                path_derived_display_name(&alias)
            };
            // If alias was created concurrently, use resolved id
            if let Ok(Some(existing)) = query_store.resolve_project_id_from_alias(&alias) {
                project_id = existing;
            } else {
                ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
            }
        } else if final_kind != AgyBindKind::Default {
            // Ensure alias link exists for resolved projects when path was re-normalized
            if query_store
                .resolve_project_id_from_alias(&alias)
                .ok()
                .flatten()
                .is_none()
            {
                let display = if alias == AGY_UNBOUND_ALIAS {
                    AGY_UNBOUND_DISPLAY_NAME.to_string()
                } else {
                    path_derived_display_name(&alias)
                };
                ensure_project_registered(sink, project_id, &alias, &display, query_store)?;
            }
        }

        match final_kind {
            AgyBindKind::History => stats.bound_via_history += 1,
            AgyBindKind::Path => stats.bound_via_path += 1,
            AgyBindKind::Unbound => stats.unbound_project += 1,
            AgyBindKind::Default => {}
        }

        let capture_context = CaptureContext {
            git_working_dir: std::env::current_dir().ok(),
        };

        service.start_session(
            ai_brains_capture::SessionStartCommand {
                session_id,
                project_id,
                harness_id,
                privacy: Privacy::LocalOnly,
                tx_id: None,
            },
            capture_context.clone(),
            sink,
        )?;

        for (i, turn) in turns.iter().enumerate().skip(next_index as usize) {
            let turn_id = generate_turn_id_for_ingest(&session_id, i, turn.step_index);

            let request = IngestRequest {
                session_id,
                project_id,
                harness_id,
                turn_id,
                role: turn.role.clone(),
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
                harness_id,
                privacy: Privacy::LocalOnly,
                status: SessionStopStatus::Completed,
                reason: Some("Antigravity multi-path import complete".to_string()),
            },
            capture_context,
            sink,
        )?;

        update_source_meta(sink, &meta_key, &current_meta);
        stats.sessions += 1;
    }

    Ok(stats)
}

/// Print F16 human stats to stderr (never claims a JSON status object).
pub fn print_import_stats(stats: &AntigravityImportStats) {
    eprintln!(
        "[Antigravity] Import stats: found={} imported_turns={} sessions={} skipped_quiescent={} skipped_unchanged_meta={} unbound_project={} bound_via_history={} bound_via_path={}",
        stats.found,
        stats.imported_turns,
        stats.sessions,
        stats.skipped_quiescent,
        stats.skipped_unchanged_meta,
        stats.unbound_project,
        stats.bound_via_history,
        stats.bound_via_path
    );
}

fn update_source_meta<S: CaptureSink>(sink: &mut S, key: &str, value: &str) {
    sink.set_sync_state(key, value);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]
    // Project test naming: function_or_feature__condition__expected_result
    #![allow(non_snake_case)]

    use super::*;
    use std::io::Write;

    #[test]
    fn extract_turns_keeps_user_and_assistant_content() {
        let steps = vec![
            AntigravityStep {
                step_index: 0,
                source: "USER_EXPLICIT".to_string(),
                step_type: "USER_INPUT".to_string(),
                content: Some("<USER_REQUEST>\nhello\n</USER_REQUEST>".to_string()),
                created_at: Some("2026-05-01T00:00:00Z".to_string()),
                tool_calls: vec![],
            },
            AntigravityStep {
                step_index: 4,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: Some("Here is the answer.".to_string()),
                created_at: Some("2026-05-01T00:00:01Z".to_string()),
                tool_calls: vec![],
            },
        ];

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "Here is the answer.");
    }

    #[test]
    fn extract_turns_skips_tool_only_responses() {
        let steps = vec![
            AntigravityStep {
                step_index: 0,
                source: "USER_EXPLICIT".to_string(),
                step_type: "USER_INPUT".to_string(),
                content: Some("<USER_REQUEST>\nread the file\n</USER_REQUEST>".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
            AntigravityStep {
                step_index: 4,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: None, // Tool call only, no text content
                created_at: None,
                tool_calls: vec![
                    serde_json::from_str(r#"{"name": "view_file"}"#).expect("valid json"),
                ],
            },
            AntigravityStep {
                step_index: 8,
                source: "MODEL".to_string(),
                step_type: "PLANNER_RESPONSE".to_string(),
                content: Some("The file contains...".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
        ];

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[1].role, "assistant");
    }

    #[test]
    fn extract_turns_skips_tool_output() {
        let steps = vec![AntigravityStep {
            step_index: 5,
            source: "MODEL".to_string(),
            step_type: "TOOL_OUTPUT".to_string(),
            content: Some("file contents here".to_string()),
            created_at: None,
            tool_calls: vec![],
        }];

        let turns = extract_turns(&steps);
        assert!(turns.is_empty());
    }

    #[test]
    fn extract_turns__view_file_and_run_command_with_content__dropped() {
        // AC16 / F7 — type-strict drop regardless of content
        let steps = vec![
            AntigravityStep {
                step_index: 1,
                source: "MODEL".to_string(),
                step_type: "VIEW_FILE".to_string(),
                content: Some("secret file body".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
            AntigravityStep {
                step_index: 2,
                source: "MODEL".to_string(),
                step_type: "RUN_COMMAND".to_string(),
                content: Some("command stdout".to_string()),
                created_at: None,
                tool_calls: vec![],
            },
        ];
        assert!(extract_turns(&steps).is_empty());
    }

    #[test]
    fn strip_xml_tags_extracts_user_request() {
        let input = "<USER_REQUEST>\ndo the ai brains preflight\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nThe current local time is: 2026-05-01\n</ADDITIONAL_METADATA>";
        let result = strip_user_xml_tags(input);
        assert_eq!(result, "do the ai brains preflight");
        assert!(!result.contains("ADDITIONAL_METADATA"));
    }

    #[test]
    fn strip_xml_tags_handles_no_tags() {
        let input = "plain text with no tags";
        assert_eq!(strip_user_xml_tags(input), "plain text with no tags");
    }

    #[test]
    fn strip_xml_tags_removes_settings_change() {
        let input = "<USER_REQUEST>\nhello\n</USER_REQUEST>\n<USER_SETTINGS_CHANGE>\nModel changed\n</USER_SETTINGS_CHANGE>";
        let result = strip_user_xml_tags(input);
        assert_eq!(result, "hello");
        assert!(!result.contains("SETTINGS_CHANGE"));
    }

    #[test]
    fn session_id_from_path_extracts_uuid() {
        let path = PathBuf::from(
            "C:/Users/RyanB/.gemini/antigravity/brain/26c85130-1a0b-4832-bb88-6cdd68d5f4ad/.system_generated/logs/overview.txt",
        );
        let id = session_id_from_path(&path);
        assert_eq!(id, Some("26c85130-1a0b-4832-bb88-6cdd68d5f4ad".to_string()));
    }

    #[test]
    fn parse_overview_file_handles_empty() {
        let dir = std::env::temp_dir().join("ai-brains-test-overview");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overview.txt");
        let _ = std::fs::write(&path, "");

        let steps = parse_overview_file(&path).expect("parse should succeed");
        assert!(steps.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_overview_file_parses_jsonl() {
        let dir = std::env::temp_dir().join("ai-brains-test-overview-jsonl");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overview.txt");

        let line1 = r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-05-01T00:00:00Z","content":"<USER_REQUEST>\nhello\n</USER_REQUEST>","tool_calls":[]}"#;
        let line2 = r#"{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-05-01T00:00:01Z","content":"Hi there","tool_calls":[]}"#;
        let line3 = r#"{"step_index":8,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-05-01T00:00:02Z","tool_calls":[{"name":"view_file"}]}"#;

        let _ = std::fs::write(&path, format!("{line1}\n{line2}\n{line3}\n"));

        let steps = parse_overview_file(&path).expect("parse should succeed");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].source, "USER_EXPLICIT");
        assert_eq!(steps[1].content.as_deref(), Some("Hi there"));
        assert!(steps[2].content.is_none());

        let turns = extract_turns(&steps);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "Hi there");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_project_chat_file_parses_jsonl() {
        let dir = std::env::temp_dir().join("ai-brains-test-project-chat");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("session-abc.jsonl");

        let header = r#"{"sessionId":"abc","projectHash":"xyz"}"#;
        let line1 = r#"{"id":"1","type":"user","content":"hello"}"#;
        let line2 = r#"{"id":"2","type":"gemini","content":"hi","thoughts":"planning..."}"#;
        let line3 = r#"{"id":"3","type":"tool_output","content":"ls output","thoughts":""}"#;

        let _ = std::fs::write(&path, format!("{header}\n{line1}\n{line2}\n{line3}\n"));

        let turns = parse_project_chat_file(&path).expect("parse should succeed");
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].role, "user");
        assert_eq!(turns[0].content, "hello");
        assert_eq!(turns[1].role, "assistant");
        assert_eq!(turns[1].content, "hi");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn history_index__latest_workspace_wins() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        // Older timestamp → workspace A
        writeln!(
            f,
            r#"{{"display":"x","timestamp":1000,"workspace":"C:\\dev\\Old","conversationId":"cid-1","type":"chat"}}"#
        )
        .unwrap();
        // Newer timestamp → workspace B wins
        writeln!(
            f,
            r#"{{"display":"y","timestamp":2000,"workspace":"C:\\dev\\New","conversationId":"cid-1","type":"chat"}}"#
        )
        .unwrap();
        // Same timestamp, later line wins
        writeln!(
            f,
            r#"{{"display":"z","timestamp":2000,"workspace":"C:\\dev\\Newest","conversationId":"cid-1","type":"chat"}}"#
        )
        .unwrap();
        // Missing conversationId skipped
        writeln!(
            f,
            r#"{{"display":"n","timestamp":3000,"workspace":"C:\\dev\\NoCid"}}"#
        )
        .unwrap();

        let map = load_agy_history_index(&path);
        assert_eq!(map.len(), 1);
        let ws = map.get("cid-1").expect("cid-1");
        let expected = normalize_agy_project_hash(r"C:\dev\Newest");
        assert_eq!(ws, &expected);
    }

    #[test]
    fn history_index__case_normalize_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        std::fs::write(
            &path,
            r#"{"timestamp":1,"workspace":"C:\\dev\\Dedupe","conversationId":"c1"}
{"timestamp":2,"workspace":"c:\\dev\\dedupe","conversationId":"c1"}
"#,
        )
        .unwrap();
        let map = load_agy_history_index(&path);
        let ws = map.get("c1").unwrap();
        assert_eq!(ws, &normalize_agy_project_hash(r"C:\dev\Dedupe"));
        assert_eq!(ws, &normalize_agy_project_hash(r"c:\dev\dedupe"));
    }

    #[test]
    fn scan_brain_dir__transcript_present__skips_overview() {
        let dir = tempfile::tempdir().unwrap();
        let brain = dir.path().join("brain");
        let session = brain.join("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
        let logs = session.join(".system_generated").join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(
            logs.join("overview.txt"),
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\noverview-only\n</USER_REQUEST>","tool_calls":[]}
"#,
        )
        .unwrap();
        std::fs::write(
            logs.join("transcript.jsonl"),
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\ntranscript-body\n</USER_REQUEST>","tool_calls":[]}
"#,
        )
        .unwrap();
        // Sibling full file (F29 path preference when transcript is selected)
        std::fs::write(
            logs.join("transcript_full.jsonl"),
            r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nfull-body-content\n</USER_REQUEST>","tool_calls":[]}
"#,
        )
        .unwrap();

        let mut sources = Vec::new();
        scan_brain_dir(&brain, &mut sources).unwrap();
        assert_eq!(
            sources.len(),
            1,
            "must not dual-enqueue overview+transcript"
        );
        assert!(
            sources[0]
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("transcript.jsonl")),
            "expected transcript.jsonl, got {:?}",
            sources[0].path
        );
        let turns = parse_transcript_for_ingest(&sources[0].path).unwrap();
        assert_eq!(turns.len(), 1);
        assert!(
            turns[0].content.contains("full-body-content"),
            "discovering transcript must allow F29 full prefer; got {:?}",
            turns[0].content
        );
    }
}
