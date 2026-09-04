//! T356 — `session reassign`: compensating SessionReassigned remediator.
//!
//! Default is print-only. `--write --yes` appends. `--suggest` uses a local
//! completion URL (fail-open skip). Tests inject [`SuggestCompleter`].

use crate::commands::governed_common::fail_usage;
use crate::commands::nightly::{DEFAULT_COMPLETION_MODEL, DEFAULT_MODEL_URL};
use crate::commands::project_paths::resolve_project_ref;
use crate::context::AppContext;
use ai_brains_core::ids::{ProjectId, SessionId, UserId};
use ai_brains_core::model_provenance::{EndpointClass, ModelProvenance};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::SessionReassignedPayload;
use ai_brains_events::{Actor, AggregateType, Envelope, Payload};
use ai_brains_models::llama_cpp::LlamaCppProvider;
use ai_brains_models::{CompletionRequest, ModelProvider};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore};
use serde::Serialize;
use std::io::IsTerminal;
use std::str::FromStr;

const UNBOUND_ALIASES: &[&str] = &[
    "cursor-unbound",
    "agy-unbound",
    "grok-unbound",
    "opencode-unbound",
    "claude-unbound",
    "codex-unbound",
];

const DEFAULT_MIN_CONFIDENCE: f64 = 0.6;
const DEFAULT_SUGGEST_CAP: usize = 20;

#[derive(Debug, Clone, Serialize)]
struct ReassignJson {
    api_version: String,
    session_id: String,
    from_project_id: String,
    to_project_id: String,
    already_bound: bool,
    written: bool,
    assigned_by: String,
    assignment_suspicious: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SuggestJson {
    api_version: String,
    mode: String,
    written: bool,
    skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<String>,
    proposals: Vec<ReassignJson>,
}

/// Injected completion handle (tests). Production uses llama.cpp.
pub(crate) trait SuggestCompleter: Send + Sync {
    fn complete(&self, prompt: &str) -> Result<SuggestCompletion, String>;
}

pub(crate) struct SuggestCompletion {
    pub text: String,
    pub model: String,
}

struct LlamaSuggestCompleter {
    url: String,
    model: String,
}

impl SuggestCompleter for LlamaSuggestCompleter {
    fn complete(&self, prompt: &str) -> Result<SuggestCompletion, String> {
        let provider = LlamaCppProvider::new(self.url.clone(), self.model.clone());
        let request = CompletionRequest {
            prompt: prompt.to_string(),
            system_prompt: Some(
                "Reply with JSON only: {\"alias\":\"<project-alias>\",\"confidence\":0.0}".into(),
            ),
            max_tokens: Some(128),
            temperature: Some(0.0),
        };
        let fut = provider.complete(request);
        let resp = complete_blocking(fut)?;
        Ok(SuggestCompletion {
            text: resp.text,
            model: resp.model,
        })
    }
}

fn complete_blocking(
    fut: impl std::future::Future<
        Output = ai_brains_models::Result<ai_brains_models::CompletionResponse>,
    >,
) -> Result<ai_brains_models::CompletionResponse, String> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(fut)).map_err(|e| e.to_string());
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(fut).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    ctx: &AppContext,
    session_id: Option<String>,
    suggest: bool,
    to_project: Option<String>,
    write: bool,
    yes: bool,
    format: &str,
    completer: Option<&dyn SuggestCompleter>,
) -> Result<(), Box<dyn std::error::Error>> {
    if write && !yes {
        return fail_usage("--write requires --yes (no silent session reassign)");
    }
    if suggest {
        return run_suggest(ctx, write, yes, format, completer);
    }
    let Some(session_id) = session_id else {
        return fail_usage("session_id is required unless --suggest");
    };
    let Some(to) = to_project else {
        return fail_usage("--to-project is required unless --suggest");
    };
    run_one(
        ctx,
        &session_id,
        &to,
        write,
        yes,
        format,
        "human",
        None,
        false,
        None,
        true,
    )
    .map(|_| ())
}

#[allow(clippy::too_many_arguments)]
fn run_one(
    ctx: &AppContext,
    session_id_str: &str,
    to_ref: &str,
    write: bool,
    yes: bool,
    format: &str,
    assigned_by: &str,
    confidence: Option<String>,
    suspicious: bool,
    provenance: Option<ModelProvenance>,
    emit: bool,
) -> Result<ReassignJson, Box<dyn std::error::Error>> {
    let session_id = SessionId::from_str(session_id_str)
        .map_err(|_| format!("Invalid session id '{session_id_str}'."))?;
    let dest = resolve_project_ref(ctx, to_ref)?;
    let from = load_session_project(ctx, &session_id)?;
    let already_bound = from == dest;
    let written = if write && yes && !already_bound {
        append_reassign(
            ctx,
            session_id,
            from,
            dest,
            assigned_by,
            suspicious,
            confidence.clone(),
            provenance,
        )?;
        true
    } else {
        false
    };

    let report = ReassignJson {
        api_version: "1".to_string(),
        session_id: session_id.to_string(),
        from_project_id: from.to_string(),
        to_project_id: dest.to_string(),
        already_bound,
        written,
        assigned_by: assigned_by.to_string(),
        assignment_suspicious: suspicious,
        confidence,
    };
    if emit {
        emit_one(format, &report)?;
    }
    Ok(report)
}

fn emit_one(format: &str, report: &ReassignJson) -> Result<(), Box<dyn std::error::Error>> {
    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());
    if use_json {
        crate::commands::identity_warn::print_json_stdout(report)?;
    } else {
        print_human_one(report);
    }
    Ok(())
}

fn print_human_one(report: &ReassignJson) {
    let tag = if report.assigned_by == "llm" {
        " ⟨llm-assigned⟩"
    } else {
        ""
    };
    let suspicious = if report.assignment_suspicious {
        " assignment_suspicious=true"
    } else {
        ""
    };
    let state = if report.already_bound {
        "already_bound"
    } else if report.written {
        "written: true"
    } else {
        "written: false"
    };
    println!(
        "session {} → {} ({state} assigned_by={}{tag}{suspicious})",
        report.session_id, report.to_project_id, report.assigned_by
    );
}

fn load_session_project(
    ctx: &AppContext,
    session_id: &SessionId,
) -> Result<ProjectId, Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let id_str = session_id.to_string();
    let project: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT project_id FROM session_projection WHERE session_id = ?",
        rusqlite::params![id_str],
        |row| row.get(0),
    );
    match project {
        Ok(pid) => Ok(ProjectId::from_str(&pid)?),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(format!("Session '{session_id}' not found in vault.").into())
        }
        Err(e) => Err(e.into()),
    }
}

#[allow(clippy::too_many_arguments)]
fn append_reassign(
    ctx: &AppContext,
    session_id: SessionId,
    from: ProjectId,
    to: ProjectId,
    assigned_by: &str,
    suspicious: bool,
    confidence: Option<String>,
    model_provenance: Option<ModelProvenance>,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let privacy = store
        .get_session_privacy(&session_id.to_string())?
        .unwrap_or(Privacy::LocalOnly);
    let envelope: Envelope = EventBuilder::new(
        AggregateType::Session,
        session_id.as_uuid(),
        Actor::User(UserId::new()),
        privacy,
    )
    .build(Payload::SessionReassigned(SessionReassignedPayload {
        session_id,
        from_project_id: from,
        to_project_id: to,
        assigned_by: assigned_by.to_string(),
        suspicious,
        confidence,
        model_provenance,
    }))?;
    store.append_event(&envelope)?;
    #[cfg(feature = "graph")]
    {
        let mut hook = crate::live_graph::LiveGraphHook::new(std::sync::Arc::clone(&ctx.conn));
        hook.apply_and_flush(&envelope);
    }
    Ok(())
}

fn run_suggest(
    ctx: &AppContext,
    write: bool,
    yes: bool,
    format: &str,
    completer: Option<&dyn SuggestCompleter>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cap = suggest_cap();
    let min_conf = min_confidence();
    let sessions = list_unbound_sessions(ctx, cap)?;
    let candidates = list_bound_project_aliases(ctx)?;
    let llama_owned;
    let handle: Option<&dyn SuggestCompleter> = match completer {
        Some(c) => Some(c),
        None => {
            let url =
                std::env::var("AI_BRAINS_MODEL_URL").unwrap_or_else(|_| DEFAULT_MODEL_URL.into());
            let model = std::env::var("AI_BRAINS_COMPLETION_MODEL")
                .unwrap_or_else(|_| DEFAULT_COMPLETION_MODEL.into());
            llama_owned = LlamaSuggestCompleter { url, model };
            Some(&llama_owned)
        }
    };

    let Some(completer) = handle else {
        return emit_suggest_skip(format, "no completer");
    };

    // Complete every session before any write so a later completer Err cannot
    // skip after partial SessionReassigned appends (F7 fail-open).
    let mut drafted = Vec::new();
    for (session_id, _from) in sessions {
        let prompt = build_suggest_prompt(ctx, &session_id, &candidates)?;
        let completion = match completer.complete(&prompt) {
            Ok(c) => c,
            Err(e) => return emit_suggest_skip(format, &e),
        };
        drafted.push((session_id, completion));
    }

    let mut proposals = Vec::new();
    for (session_id, completion) in drafted {
        let parsed = match parse_suggest_reply(&completion.text, &candidates) {
            Some(p) => p,
            None => continue,
        };
        let mut suspicious = parsed.confidence < min_conf;
        if contradiction(ctx, &session_id, &parsed.alias)? {
            suspicious = true;
        }
        let conf_text = format!("{:.2}", parsed.confidence);
        let provenance = ModelProvenance::from_provider(
            "llama.cpp",
            &completion.model,
            EndpointClass::LocalLoopback,
        );
        let report = run_one(
            ctx,
            &session_id,
            &parsed.alias,
            write,
            yes,
            format,
            "llm",
            Some(conf_text),
            suspicious,
            Some(provenance),
            false,
        )?;
        proposals.push(report);
    }

    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());
    let written = proposals.iter().any(|p| p.written);
    if use_json {
        crate::commands::identity_warn::print_json_stdout(&SuggestJson {
            api_version: "1".to_string(),
            mode: "suggest".to_string(),
            written,
            skipped: false,
            skip_reason: None,
            proposals: proposals.clone(),
        })?;
    } else {
        if proposals.is_empty() {
            println!("session reassign --suggest: no proposals");
        }
        for p in &proposals {
            print_human_one(p);
        }
    }
    Ok(())
}

fn emit_suggest_skip(format: &str, reason: &str) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("session reassign --suggest skip: {reason}");
    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());
    if use_json {
        crate::commands::identity_warn::print_json_stdout(&SuggestJson {
            api_version: "1".to_string(),
            mode: "suggest".to_string(),
            written: false,
            skipped: true,
            skip_reason: Some(reason.to_string()),
            proposals: Vec::new(),
        })?;
    }
    Ok(())
}

fn suggest_cap() -> usize {
    std::env::var("AI_BRAINS_REASSIGN_SUGGEST_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &usize| *n >= 1)
        .unwrap_or(DEFAULT_SUGGEST_CAP)
}

fn min_confidence() -> f64 {
    std::env::var("AI_BRAINS_REASSIGN_MIN_CONFIDENCE")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n: &f64| *n >= 0.0 && *n <= 1.0)
        .unwrap_or(DEFAULT_MIN_CONFIDENCE)
}

fn list_unbound_sessions(
    ctx: &AppContext,
    cap: usize,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let placeholders = UNBOUND_ALIASES
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT sp.session_id, sp.project_id
         FROM session_projection sp
         WHERE sp.project_id IN (
           SELECT project_id FROM project_alias_projection WHERE alias IN ({placeholders})
         )
         ORDER BY sp.session_id
         LIMIT ?"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut binds: Vec<rusqlite::types::Value> = UNBOUND_ALIASES
        .iter()
        .map(|alias| rusqlite::types::Value::Text((*alias).to_string()))
        .collect();
    binds.push(rusqlite::types::Value::Integer(cap as i64));
    let rows = stmt.query_map(rusqlite::params_from_iter(binds), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn list_bound_project_aliases(
    ctx: &AppContext,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let mut stmt = conn.prepare(
        "SELECT project_id, alias FROM project_alias_projection
         WHERE alias NOT IN ('cursor-unbound','agy-unbound','grok-unbound','opencode-unbound','claude-unbound','codex-unbound')
         ORDER BY alias",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn build_suggest_prompt(
    ctx: &AppContext,
    session_id: &str,
    candidates: &[(String, String)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut body = String::new();
    if let Some(summary) = load_summary(ctx, session_id)? {
        body.push_str(&summary);
    } else {
        for (role, content) in ctx.conn.get_session_turns(session_id)? {
            body.push_str(&format!("{role}: {content}\n"));
        }
    }
    let aliases: Vec<&str> = candidates.iter().map(|(_, a)| a.as_str()).collect();
    Ok(format!(
        "Session {session_id}\nCandidates: {}\n---\n{body}\n---\nPick one alias.",
        aliases.join(", ")
    ))
}

fn load_summary(
    ctx: &AppContext,
    session_id: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let conn = ctx.conn.lock()?;
    let mid: Option<String> = match conn.query_row(
        "SELECT summary_memory_id FROM session_projection WHERE session_id = ?",
        rusqlite::params![session_id],
        |row| row.get::<_, Option<String>>(0),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(e.into()),
    };
    let Some(mid) = mid else {
        return Ok(None);
    };
    let content: Option<String> = conn
        .query_row(
            "SELECT content FROM memory_projection WHERE memory_id = ?",
            rusqlite::params![mid],
            |row| row.get(0),
        )
        .optional()?;
    Ok(content)
}

struct ParsedSuggest {
    alias: String,
    confidence: f64,
}

fn parse_suggest_reply(text: &str, candidates: &[(String, String)]) -> Option<ParsedSuggest> {
    let trimmed = text.trim();
    let json_slice = trimmed
        .find('{')
        .and_then(|i| trimmed.rfind('}').map(|j| &trimmed[i..=j]))?;
    let value: serde_json::Value = serde_json::from_str(json_slice).ok()?;
    let alias = value.get("alias")?.as_str()?.trim().to_string();
    if alias.is_empty() {
        return None;
    }
    if !candidates.iter().any(|(_, a)| a == &alias) {
        return None;
    }
    let confidence = value
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    Some(ParsedSuggest { alias, confidence })
}

fn text_mentions(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let hay = haystack.replace('\\', "/").to_ascii_lowercase();
    let n = needle.replace('\\', "/").to_ascii_lowercase();
    hay.contains(&n)
}

fn contradiction(
    ctx: &AppContext,
    session_id: &str,
    dest_alias: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut text = String::new();
    if let Some(summary) = load_summary(ctx, session_id)? {
        text.push_str(&summary);
    }
    for (_, content) in ctx.conn.get_session_turns(session_id)? {
        text.push(' ');
        text.push_str(&content);
    }
    let dest = resolve_project_ref(ctx, dest_alias)?;
    let dest_s = dest.to_string();
    let aliases = list_bound_project_aliases(ctx)?;
    let paths = ctx.conn.list_path_aliases()?;
    let dest_mentioned = text_mentions(&text, dest_alias)
        || paths
            .iter()
            .any(|(pid, path)| *pid == dest && text_mentions(&text, path));
    let other_alias = aliases
        .iter()
        .any(|(id, alias)| id != &dest_s && text_mentions(&text, alias));
    let other_path = paths
        .iter()
        .any(|(pid, path)| pid.to_string() != dest_s && text_mentions(&text, path));
    Ok((other_alias || other_path) && !dest_mentioned)
}

use rusqlite::OptionalExtension;

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_events::payload::{
        MemoryPinnedPayload, ProjectAliasAddedPayload, ProjectRegisteredPayload,
        RepositoryPathAliasAddedPayload, SessionStartedPayload, UserPromptRecordedPayload,
    };
    use ai_brains_events::{Actor, AggregateType, Payload, constructors::EventBuilder};
    use ai_brains_store::EventStore;
    use tempfile::tempdir;

    struct FakeCompleter {
        text: String,
        model: String,
        fail: bool,
    }

    impl SuggestCompleter for FakeCompleter {
        fn complete(&self, _prompt: &str) -> Result<SuggestCompletion, String> {
            if self.fail {
                return Err("fake-down".to_string());
            }
            Ok(SuggestCompletion {
                text: self.text.clone(),
                model: self.model.clone(),
            })
        }
    }

    fn open_ctx(dir: &std::path::Path) -> AppContext {
        let db = dir.join("vault.db");
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        AppContext::from_resolved_key(db, sql_key).expect("open ctx")
    }

    fn append(store: &SqliteEventStore, envelope: Envelope) {
        store.append_event(&envelope).expect("append");
    }

    fn register_path(store: &SqliteEventStore, id: ProjectId, normalized_path: &str) {
        append(
            store,
            EventBuilder::new(
                AggregateType::Project,
                id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::RepositoryPathAliasAdded(
                RepositoryPathAliasAddedPayload {
                    project_id: id,
                    normalized_path: normalized_path.to_string(),
                },
            ))
            .expect("path alias"),
        );
    }

    struct SeqCompleter {
        replies: Vec<Result<String, String>>,
        idx: std::sync::Mutex<usize>,
    }

    impl SuggestCompleter for SeqCompleter {
        fn complete(&self, _prompt: &str) -> Result<SuggestCompletion, String> {
            let mut idx = self.idx.lock().expect("seq idx");
            let i = *idx;
            *idx += 1;
            match self.replies.get(i) {
                Some(Ok(text)) => Ok(SuggestCompletion {
                    text: text.clone(),
                    model: "fake-llm".into(),
                }),
                Some(Err(e)) => Err(e.clone()),
                None => Err("seq exhausted".into()),
            }
        }
    }

    fn register_project(store: &SqliteEventStore, id: ProjectId, alias: &str) {
        append(
            store,
            EventBuilder::new(
                AggregateType::Project,
                id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                project_id: id,
                name: alias.to_string(),
                tx_id: None,
            }))
            .expect("register"),
        );
        append(
            store,
            EventBuilder::new(
                AggregateType::Project,
                id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
                project_id: id,
                alias: alias.to_string(),
            }))
            .expect("alias"),
        );
    }

    fn start_session(store: &SqliteEventStore, session: SessionId, project: ProjectId) {
        append(
            store,
            EventBuilder::new(
                AggregateType::Session,
                session.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::SessionStarted(SessionStartedPayload {
                session_id: session,
                project_id: project,
                tx_id: None,
            }))
            .expect("session"),
        );
    }

    fn record_turn(store: &SqliteEventStore, session: SessionId, content: &str) {
        append(
            store,
            EventBuilder::new(
                AggregateType::Session,
                session.as_uuid(),
                Actor::User(UserId::new()),
                Privacy::LocalOnly,
            )
            .build(Payload::UserPromptRecorded(UserPromptRecordedPayload {
                session_id: session,
                content: content.to_string(),
                tx_id: None,
                turn_id: None,
            }))
            .expect("turn"),
        );
    }

    fn pin_on_session(
        store: &SqliteEventStore,
        session: SessionId,
        project: ProjectId,
        content: &str,
    ) {
        let memory_id = ai_brains_core::ids::MemoryId::new();
        append(
            store,
            EventBuilder::new(
                AggregateType::Memory,
                memory_id.as_uuid(),
                Actor::User(UserId::new()),
                Privacy::LocalOnly,
            )
            .build(Payload::MemoryPinned(MemoryPinnedPayload {
                memory_id,
                content: content.to_string(),
                session_id: Some(session),
                project_id: Some(project),
                tx_id: None,
                rank: None,
                source_tag: None,
                query_text: None,
            }))
            .expect("pin"),
        );
    }

    #[test]
    fn run_suggest__injected_fail__skip_exit_ok_no_events() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest = ProjectId::new();
        let session = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest, "dest-proj");
        start_session(&store, session, unbound);
        let before = store.read_all_events().expect("events").len();
        let fake = FakeCompleter {
            text: String::new(),
            model: "fake".into(),
            fail: true,
        };
        run(&ctx, None, true, None, false, false, "json", Some(&fake)).expect("fail-open");
        let after = store.read_all_events().expect("events").len();
        assert_eq!(after, before, "AC5: no events on fake-down");
    }

    #[test]
    fn run_suggest_write__injected_ok__assigned_by_llm() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest = ProjectId::new();
        let session = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest, "dest-proj");
        start_session(&store, session, unbound);
        record_turn(&store, session, "work on dest-proj");
        pin_on_session(&store, session, unbound, "DECISION: dest-proj");
        let fake = FakeCompleter {
            text: r#"{"alias":"dest-proj","confidence":0.91}"#.into(),
            model: "fake-llm".into(),
            fail: false,
        };
        run(&ctx, None, true, None, true, true, "json", Some(&fake)).expect("suggest write");
        let events = store.read_all_events().expect("events");
        let reassigned = events.iter().find_map(|e| match &e.payload {
            Payload::SessionReassigned(p) => Some(p),
            _ => None,
        });
        let payload = reassigned.expect("SessionReassigned");
        assert_eq!(payload.assigned_by, "llm");
        assert_eq!(payload.to_project_id, dest);
        assert_eq!(payload.confidence.as_deref(), Some("0.91"));
        assert!(!payload.suspicious);
        let conn = ctx.conn.lock().expect("lock");
        let sid = session.to_string();
        let dest_s = dest.to_string();
        let sp: String = conn
            .query_row(
                "SELECT project_id FROM session_projection WHERE session_id = ?",
                rusqlite::params![sid],
                |row| row.get(0),
            )
            .expect("session proj");
        assert_eq!(sp, dest_s);
        let mp: String = conn
            .query_row(
                "SELECT project_id FROM memory_projection WHERE session_id = ?",
                rusqlite::params![sid],
                |row| row.get(0),
            )
            .expect("memory proj");
        assert_eq!(mp, dest_s);
        let tp: String = conn
            .query_row(
                "SELECT project_id FROM turn_projection WHERE session_id = ?",
                rusqlite::params![sid],
                |row| row.get(0),
            )
            .expect("turn proj");
        assert_eq!(tp, dest_s);
    }

    #[test]
    fn run_suggest_write__contradiction__assignment_suspicious() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest_y = ProjectId::new();
        let named_x = ProjectId::new();
        let session = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest_y, "dest-y");
        register_project(&store, named_x, "named-x");
        start_session(&store, session, unbound);
        record_turn(&store, session, "this is clearly named-x work");
        let fake = FakeCompleter {
            text: r#"{"alias":"dest-y","confidence":0.88}"#.into(),
            model: "fake-llm".into(),
            fail: false,
        };
        run(&ctx, None, true, None, true, true, "json", Some(&fake)).expect("suggest write");
        let events = store.read_all_events().expect("events");
        let payload = events
            .iter()
            .find_map(|e| match &e.payload {
                Payload::SessionReassigned(p) => Some(p),
                _ => None,
            })
            .expect("SessionReassigned");
        assert_eq!(payload.to_project_id, dest_y);
        assert!(payload.suspicious, "AC7: contradiction sets suspicious");
        assert_eq!(payload.assigned_by, "llm");
    }

    #[test]
    fn run_suggest_write__second_complete_err__skip_no_events() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest = ProjectId::new();
        let session_a = SessionId::new();
        let session_b = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest, "dest-proj");
        start_session(&store, session_a, unbound);
        start_session(&store, session_b, unbound);
        let before = store.read_all_events().expect("events").len();
        let fake = SeqCompleter {
            replies: vec![
                Ok(r#"{"alias":"dest-proj","confidence":0.91}"#.into()),
                Err("fake-down-later".into()),
            ],
            idx: std::sync::Mutex::new(0),
        };
        run(&ctx, None, true, None, true, true, "json", Some(&fake)).expect("fail-open");
        let after = store.read_all_events().expect("events").len();
        assert_eq!(
            after, before,
            "completer Err after a successful draft must not append"
        );
        let reassigned = store
            .read_all_events()
            .expect("events")
            .iter()
            .any(|e| matches!(e.payload, Payload::SessionReassigned(_)));
        assert!(!reassigned);
    }

    #[test]
    fn run_suggest_write__path_contradiction__assignment_suspicious() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest_y = ProjectId::new();
        let named_x = ProjectId::new();
        let session = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest_y, "dest-y");
        register_project(&store, named_x, "named-x");
        register_path(&store, named_x, r"C:\dev\ledgerful-hands");
        start_session(&store, session, unbound);
        record_turn(
            &store,
            session,
            r"working in C:\dev\ledgerful-hands tonight",
        );
        let fake = FakeCompleter {
            text: r#"{"alias":"dest-y","confidence":0.88}"#.into(),
            model: "fake-llm".into(),
            fail: false,
        };
        run(&ctx, None, true, None, true, true, "json", Some(&fake)).expect("suggest write");
        let events = store.read_all_events().expect("events");
        let payload = events
            .iter()
            .find_map(|e| match &e.payload {
                Payload::SessionReassigned(p) => Some(p),
                _ => None,
            })
            .expect("SessionReassigned");
        assert_eq!(payload.to_project_id, dest_y);
        assert!(
            payload.suspicious,
            "AC7/F8: registered path other than dest is contradiction"
        );
    }

    #[test]
    fn run_suggest_write__unparsable__no_session_reassigned() {
        let dir = tempdir().expect("tempdir");
        let ctx = open_ctx(dir.path());
        let store = SqliteEventStore::new((*ctx.conn).clone());
        let unbound = ProjectId::new();
        let dest = ProjectId::new();
        let session = SessionId::new();
        register_project(&store, unbound, "cursor-unbound");
        register_project(&store, dest, "dest-proj");
        start_session(&store, session, unbound);
        let before = store.read_all_events().expect("events").len();
        let fake = FakeCompleter {
            text: "not-json".into(),
            model: "fake-llm".into(),
            fail: false,
        };
        run(&ctx, None, true, None, true, true, "json", Some(&fake)).expect("unparsable");
        let after = store.read_all_events().expect("events").len();
        assert_eq!(after, before, "unparsable proposal must not append");
    }

    #[test]
    fn suggest_cap__temp_env__parses_min_one() {
        let _cap = TempEnv::set("AI_BRAINS_REASSIGN_SUGGEST_CAP", "1");
        let _min = TempEnv::set("AI_BRAINS_REASSIGN_MIN_CONFIDENCE", "0.75");
        assert_eq!(suggest_cap(), 1);
        assert!((min_confidence() - 0.75).abs() < f64::EPSILON);
    }
}
