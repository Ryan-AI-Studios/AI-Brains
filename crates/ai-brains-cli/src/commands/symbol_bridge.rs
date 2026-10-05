//! Ledgerful symbol inventory bridge (T233 / 0163 JSON).
//!
//! Spawns `ledgerful symbols --pub --json` with an explicit root `current_dir`
//! (never Task Scheduler System32 cwd). SQL inventory path deleted (F36).

use crate::context::AppContext;
use ai_brains_core::ids::{MemoryId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::{
    Actor, AggregateType, MemoryPinnedPayload, Payload, constructors::EventBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use ai_brains_store::EventStore;
use ai_brains_store::connection::VaultConnection;

/// Legacy source_tag written by pre-T191 symbol ingest (durable in vault events).
/// Dual-read is `symbol_pin_ids_present`. The nightly writer does not emit this tag.
#[cfg_attr(not(test), allow(dead_code))]
pub const SOURCE_TAG_SYMBOL_LEGACY: &str = "changeguard:symbol";
/// Canonical source_tag for new symbol ingest writes (T191 F2).
pub const SOURCE_TAG_SYMBOL: &str = "ledgerful:symbol";

/// Default / hard-max symbols per root (matches ledgerful CLI hard max).
const DEFAULT_MAX_SYMBOLS: usize = 5000;

/// Multi-pass recursion depth when inventory reports `truncated` (T373 F4; supersedes T233 depth 2).
const MULTI_PASS_MAX_DEPTH: u32 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolRecord {
    file_path: String,
    qualified_name: String,
    #[allow(dead_code)]
    symbol_name: String,
    symbol_kind: String,
    line_start: i64,
}

/// Wire shape for `ledgerful symbols --json` (schemaVersion 1).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolsEnvelope {
    schema_version: Option<u64>,
    #[serde(default)]
    truncated: bool,
    /// 0163: optional object `{ state, remediation? }` (omit when usable).
    /// Also accept a bare string for resilience.
    #[serde(default)]
    index_status: Option<serde_json::Value>,
    #[serde(default)]
    total_matching: Option<usize>,
    #[serde(default)]
    symbols: Vec<WireSymbol>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireSymbol {
    name: Option<String>,
    kind: Option<String>,
    path: Option<String>,
    line: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    is_public: Option<bool>,
    qualified_name: Option<String>,
}

/// Refresh + ingest public symbols from Ledgerful inventory for one root.
///
/// Non-fatal: spawn failures, bad JSON, unusable index → warn + Ok(0).
pub fn ingest_symbols_from_ledgerful(
    ctx: &AppContext,
    project_id: ProjectId,
    root: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    if !root.exists() {
        tracing::warn!(
            root = %root.display(),
            "[Nightly] symbol root missing; skip"
        );
        return Ok(0);
    }

    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let max_n = max_symbols_from_env();
    ingest_symbols_with_fetch(
        &event_store,
        &ctx.conn,
        project_id,
        root,
        max_n,
        &mut fetch_symbols_pass,
    )
}

fn ingest_symbols_with_fetch<F>(
    event_store: &dyn EventStore,
    conn: &VaultConnection,
    project_id: ProjectId,
    root: &Path,
    max_n: usize,
    fetch: &mut F,
) -> Result<usize, Box<dyn std::error::Error>>
where
    F: FnMut(&Path, Option<&str>, usize) -> Result<PassFetch, String>,
{
    let bookmark_key = backlog_state_key(project_id, root);
    let bookmark = load_backlog_bookmark(event_store, &bookmark_key);
    let probe = probe_git_root(root);
    if should_skip_symbol_walk(bookmark.as_ref(), &probe) {
        tracing::info!(
            root = %root.display(),
            walk_skipped = true,
            remaining_unpinned = 0,
            total_matching = Option::<usize>::None,
            "[Nightly] symbol walk skipped; clean HEAD already caught up"
        );
        return Ok(0);
    }

    let outcome = match collect_symbols_with_fetch(root, max_n, fetch) {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::warn!(
                root = %root.display(),
                error = %error,
                "[Nightly] symbol root fetch failed; bookmark unchanged"
            );
            return Ok(0);
        }
    };

    let candidate_ids: Vec<Uuid> = outcome
        .symbols
        .iter()
        .map(|symbol| symbol_memory_uuid(project_id, &symbol.qualified_name))
        .collect();
    let already_pinned = ai_brains_store::symbol_pin_ids_present(conn, &candidate_ids)?;
    let (selected, unpinned_before) =
        select_unpinned(outcome.symbols, &already_pinned, project_id, max_n);
    let capped = unpinned_before > max_n;
    let selected_len = selected.len();
    let ingested = if selected.is_empty() {
        0
    } else {
        ingest_symbol_records(
            event_store,
            project_id,
            Some(root),
            selected,
            &already_pinned,
        )?
    };
    // A budget slice that still leaves ids out is not caught up. A final slice
    // that all append is: those ids are pinned when the bookmark is written.
    let remaining_unpinned = unpinned_before.saturating_sub(ingested.min(selected_len));
    let caught_up = coverage_caught_up(
        outcome.symbols_returned,
        outcome.total_matching,
        outcome.walk_incomplete,
        remaining_unpinned > 0,
    );

    tracing::info!(
        root = %root.display(),
        symbols_returned = outcome.symbols_returned,
        symbols_ingested_cap = max_n,
        remaining_unpinned,
        total_matching = outcome.total_matching,
        walk_skipped = false,
        symbols_truncated_by_ingest_cap = capped,
        "[Nightly] symbol inventory metrics"
    );
    if outcome.walk_incomplete {
        tracing::warn!(
            root = %root.display(),
            "[Nightly] symbol coverage incomplete"
        );
    }
    if remaining_unpinned > 0 {
        tracing::warn!(
            root = %root.display(),
            remaining_unpinned,
            "[Nightly] symbol backlog remains"
        );
    }

    let saved = bookmark_from_probe(&probe, caught_up);
    persist_backlog_bookmark(event_store, &bookmark_key, true, &saved)?;

    tracing::info!(
        root = %root.display(),
        symbols_ingested = ingested,
        caught_up,
        "[Nightly] symbols ingested"
    );
    Ok(ingested)
}

#[derive(Debug, Clone)]
struct CollectOutcome {
    symbols: Vec<SymbolRecord>,
    symbols_returned: usize,
    total_matching: Option<usize>,
    walk_incomplete: bool,
}

#[derive(Debug, Clone)]
struct PassFetch {
    symbols: Vec<SymbolRecord>,
    truncated: bool,
    total_matching: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SymbolBacklogBookmark {
    git_head: Option<String>,
    clean: bool,
    caught_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitProbe {
    ok: bool,
    head: Option<String>,
    clean: bool,
}

fn backlog_state_key(project_id: ProjectId, root: &Path) -> String {
    format!(
        "nightly.symbol_backlog.v1:{project_id}:{}",
        root.to_string_lossy()
    )
}

fn load_backlog_bookmark(store: &dyn EventStore, key: &str) -> Option<SymbolBacklogBookmark> {
    let raw = store.get_sync_state(key).ok()??;
    serde_json::from_str(&raw).ok()
}

fn persist_backlog_bookmark(
    store: &dyn EventStore,
    key: &str,
    root_fetch_ok: bool,
    bookmark: &SymbolBacklogBookmark,
) -> Result<(), Box<dyn std::error::Error>> {
    if !root_fetch_ok {
        return Ok(());
    }
    let json = serde_json::to_string(bookmark)?;
    store.set_sync_state(key, &json)?;
    Ok(())
}

fn probe_git_root(root: &Path) -> GitProbe {
    let head = match ai_brains_git::run_git(root, &["rev-parse", "HEAD"]) {
        Ok(Some(sha)) => Some(sha),
        Ok(None) => None,
        Err(_) => {
            return GitProbe {
                ok: false,
                head: None,
                clean: false,
            };
        }
    };
    match ai_brains_git::run_git(root, &["status", "--porcelain"]) {
        Ok(None) => GitProbe {
            ok: true,
            head,
            clean: true,
        },
        Ok(Some(_)) => GitProbe {
            ok: true,
            head,
            clean: false,
        },
        Err(_) => GitProbe {
            ok: false,
            head,
            clean: false,
        },
    }
}

fn bookmark_from_probe(probe: &GitProbe, caught_up: bool) -> SymbolBacklogBookmark {
    SymbolBacklogBookmark {
        git_head: if probe.ok { probe.head.clone() } else { None },
        clean: probe.ok && probe.clean,
        caught_up,
    }
}

fn should_skip_symbol_walk(bookmark: Option<&SymbolBacklogBookmark>, probe: &GitProbe) -> bool {
    if !probe.ok || !probe.clean {
        return false;
    }
    let Some(head) = probe.head.as_deref() else {
        return false;
    };
    let Some(bookmark) = bookmark else {
        return false;
    };
    bookmark.caught_up && bookmark.clean && bookmark.git_head.as_deref() == Some(head)
}

fn coverage_caught_up(
    collected_unique: usize,
    total_matching: Option<usize>,
    walk_incomplete: bool,
    any_unpinned: bool,
) -> bool {
    if walk_incomplete || any_unpinned {
        return false;
    }
    match total_matching {
        Some(total) => collected_unique >= total,
        None => false,
    }
}

fn select_unpinned(
    symbols: Vec<SymbolRecord>,
    already_pinned: &HashSet<Uuid>,
    project_id: ProjectId,
    max_n: usize,
) -> (Vec<SymbolRecord>, usize) {
    let mut unpinned: Vec<SymbolRecord> = symbols
        .into_iter()
        .filter(|symbol| {
            !already_pinned.contains(&symbol_memory_uuid(project_id, &symbol.qualified_name))
        })
        .collect();
    unpinned.sort_by(|left, right| {
        (
            &left.file_path,
            left.line_start,
            &left.qualified_name,
            &left.symbol_kind,
        )
            .cmp(&(
                &right.file_path,
                right.line_start,
                &right.qualified_name,
                &right.symbol_kind,
            ))
    });
    let unpinned_before = unpinned.len();
    if max_n < unpinned.len() {
        unpinned.truncate(max_n);
    }
    (unpinned, unpinned_before)
}

fn max_symbols_from_env() -> usize {
    std::env::var("AI_BRAINS_NIGHTLY_MAX_SYMBOLS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .map(|n| n.min(DEFAULT_MAX_SYMBOLS))
        .unwrap_or(DEFAULT_MAX_SYMBOLS)
}

/// Merge a root pass with descendant passes.
///
/// `incomplete` follows the descendant walk. A finished walk is not sticky-truncated
/// just because the root page was truncated (T373 F4). Callers still compare
/// `collected` with root `totalMatching`.
fn collect_symbols_from_passes(
    first_symbols: Vec<SymbolRecord>,
    first_truncated: bool,
    multi_more: Vec<SymbolRecord>,
    multi_still_trunc: bool,
) -> (Vec<SymbolRecord>, bool) {
    if !first_truncated {
        return (dedupe_symbols(first_symbols), false);
    }
    let mut symbols = first_symbols;
    symbols.extend(multi_more);
    (dedupe_symbols(symbols), multi_still_trunc)
}

fn collect_symbols_with_fetch<F>(
    root: &Path,
    max_n: usize,
    fetch: &mut F,
) -> Result<CollectOutcome, String>
where
    F: FnMut(&Path, Option<&str>, usize) -> Result<PassFetch, String>,
{
    let root_pass = fetch(root, None, max_n)?;
    let (symbols, walk_incomplete) = if root_pass.truncated {
        let (more, still) = multi_pass_at(root, root, max_n, 1, MULTI_PASS_MAX_DEPTH, fetch);
        collect_symbols_from_passes(root_pass.symbols, true, more, still)
    } else {
        collect_symbols_from_passes(root_pass.symbols, false, Vec::new(), false)
    };
    let symbols_returned = symbols.len();
    Ok(CollectOutcome {
        symbols,
        symbols_returned,
        total_matching: root_pass.total_matching,
        walk_incomplete,
    })
}

enum ChildKind {
    Dir,
    File,
}

fn multi_pass_at<F>(
    root: &Path,
    dir: &Path,
    max_n: usize,
    depth: u32,
    max_depth: u32,
    fetch: &mut F,
) -> (Vec<SymbolRecord>, bool)
where
    F: FnMut(&Path, Option<&str>, usize) -> Result<PassFetch, String>,
{
    if depth > max_depth {
        return (Vec::new(), true);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(
                dir = %dir.display(),
                error = %error,
                "[Nightly] multi-pass read_dir failed"
            );
            return (Vec::new(), true);
        }
    };

    let mut children: Vec<(PathBuf, ChildKind)> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            if name.starts_with('.') {
                return None;
            }
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                Some((entry.path(), ChildKind::Dir))
            } else if kind.is_file() {
                Some((entry.path(), ChildKind::File))
            } else {
                None
            }
        })
        .collect();
    children.sort_by(|left, right| left.0.cmp(&right.0));

    if children.is_empty() {
        return (Vec::new(), true);
    }

    let mut all = Vec::new();
    let mut any_trunc = false;
    for (path, kind) in children {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let rel = match path.strip_prefix(root) {
            Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
            Err(_) => name.to_string(),
        };
        match fetch(root, Some(&rel), max_n) {
            Ok(pass) => {
                all.extend(pass.symbols);
                if pass.truncated {
                    if matches!(kind, ChildKind::Dir) && depth < max_depth {
                        let (more, still) =
                            multi_pass_at(root, &path, max_n, depth + 1, max_depth, fetch);
                        all.extend(more);
                        if still {
                            any_trunc = true;
                        }
                    } else {
                        any_trunc = true;
                    }
                }
            }
            Err(error) => {
                tracing::warn!(
                    root = %root.display(),
                    path = %rel,
                    error = %error,
                    "[Nightly] multi-pass symbol fetch failed (non-fatal)"
                );
                any_trunc = true;
            }
        }
    }
    (all, any_trunc)
}

/// Plan for one `ledgerful symbols` invocation (hermetic tests for cwd + args).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolsInvokePlan {
    pub cwd: PathBuf,
    pub args: Vec<String>,
}

/// Build symbols CLI plan: `current_dir = root`, never Task Scheduler System32 cwd.
pub(crate) fn symbols_invoke_plan(
    root: &Path,
    path_prefix: Option<&str>,
    limit: usize,
    auto_index: bool,
) -> SymbolsInvokePlan {
    let mut args = vec![
        "symbols".to_string(),
        "--pub".to_string(),
        "--json".to_string(),
        "--limit".to_string(),
        limit.to_string(),
    ];
    if auto_index {
        args.push("--auto-index".to_string());
    }
    if let Some(p) = path_prefix {
        args.push("--path".to_string());
        args.push(p.to_string());
    }
    SymbolsInvokePlan {
        cwd: root.to_path_buf(),
        args,
    }
}

/// Whether a soft-failed symbols pass should report `truncated=true`.
///
/// Multi-pass children (`path_prefix` set) must — otherwise `collect_symbols_from_passes`
/// can clear first-pass truncation after empty non-trunc soft-fails (F37 / Codex R1 P1).
pub(crate) fn soft_fail_marks_truncated(path_prefix: Option<&str>) -> bool {
    path_prefix.is_some()
}

/// Map a parse outcome onto a pass. Root failures (`path_prefix == None`) are `Err`.
/// Child failures stay `Ok` with `truncated = true`.
fn pass_from_outcome(
    path_prefix: Option<&str>,
    outcome: ParseOutcome,
) -> Result<PassFetch, String> {
    match outcome {
        ParseOutcome::Ok {
            symbols,
            truncated,
            total_matching,
        } => Ok(PassFetch {
            symbols,
            truncated,
            total_matching,
        }),
        ParseOutcome::Skip { reason } => soft_fail_pass(path_prefix, reason),
        ParseOutcome::Err(error) => soft_fail_pass(path_prefix, error),
    }
}

fn soft_fail_pass(path_prefix: Option<&str>, reason: String) -> Result<PassFetch, String> {
    if soft_fail_marks_truncated(path_prefix) {
        Ok(PassFetch {
            symbols: Vec::new(),
            truncated: true,
            total_matching: None,
        })
    } else {
        Err(reason)
    }
}

/// One `ledgerful symbols` invocation.
///
/// Child soft-fail (`path_prefix` is `Some`) returns an empty truncated pass so a
/// failed child cannot look complete. Root soft-fail returns `Err` so the caller
/// leaves the bookmark unchanged.
fn fetch_symbols_pass(
    root: &Path,
    path_prefix: Option<&str>,
    limit: usize,
) -> Result<PassFetch, String> {
    let plan = symbols_invoke_plan(root, path_prefix, limit, path_prefix.is_none());

    #[allow(clippy::disallowed_methods)]
    let output = match Command::new("ledgerful")
        .current_dir(&plan.cwd)
        .args(&plan.args)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            tracing::warn!(
                root = %root.display(),
                error = %error,
                "[Nightly] ledgerful not available for symbols (non-fatal)"
            );
            return soft_fail_pass(path_prefix, error.to_string());
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        tracing::warn!(
            root = %root.display(),
            stderr = %stderr,
            "[Nightly] ledgerful symbols non-zero (non-fatal)"
        );
        return soft_fail_pass(path_prefix, stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_symbols_envelope(stdout.trim()) {
        ParseOutcome::Skip { reason } => {
            tracing::warn!(
                root = %root.display(),
                reason = %reason,
                "[Nightly] symbol inventory skipped"
            );
            soft_fail_pass(path_prefix, reason)
        }
        other => pass_from_outcome(path_prefix, other),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseOutcome {
    Ok {
        symbols: Vec<SymbolRecord>,
        truncated: bool,
        total_matching: Option<usize>,
    },
    Skip {
        reason: String,
    },
    Err(String),
}

/// Parse 0163 JSON envelope into internal records (unit-tested).
fn parse_symbols_envelope(json: &str) -> ParseOutcome {
    let env: SymbolsEnvelope = match serde_json::from_str(json) {
        Ok(e) => e,
        Err(e) => return ParseOutcome::Err(format!("json: {e}")),
    };

    match env.schema_version {
        Some(1) => {}
        Some(v) => {
            return ParseOutcome::Skip {
                reason: format!("schemaVersion {v} unsupported (need 1)"),
            };
        }
        None => {
            return ParseOutcome::Skip {
                reason: "schemaVersion missing".into(),
            };
        }
    }

    if let Some(ref status) = env.index_status
        && !index_status_value_usable(status)
    {
        return ParseOutcome::Skip {
            reason: format!("indexStatus unusable: {status}"),
        };
    }

    let mut symbols = Vec::new();
    for w in env.symbols {
        if let Some(rec) = wire_to_record(w) {
            symbols.push(rec);
        }
    }

    ParseOutcome::Ok {
        symbols,
        truncated: env.truncated,
        total_matching: env.total_matching,
    }
}

/// 0163 `indexStatus` may be an object `{ "state": "missing", ... }` or a string.
/// Field absent → caller does not invoke this (usable). Null → usable.
fn index_status_value_usable(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => index_status_usable(s),
        serde_json::Value::Object(map) => {
            let state = map
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            // Object present is an honesty signal; empty/unknown state → unusable.
            if state.is_empty() {
                return false;
            }
            index_status_usable(state)
        }
        // Unexpected shapes → unusable (fail closed for honesty).
        _ => false,
    }
}

/// Usable indexStatus: missing field (handled by Option), empty/null-like,
/// or known-ok tokens. Unusable: missing/stale/error (and similar).
fn index_status_usable(status: &str) -> bool {
    let s = status.trim();
    if s.is_empty() {
        return true;
    }
    let lower = s.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "ok" | "ready" | "current" | "fresh" | "indexed" | "available"
    ) {
        return true;
    }
    // Explicit bad states.
    if lower.contains("missing")
        || lower.contains("stale")
        || lower.contains("error")
        || lower.contains("fail")
        || lower == "unavailable"
        || lower == "none"
    {
        return false;
    }
    // Unknown non-empty status: treat as usable (prefer partial over skip).
    true
}

fn wire_to_record(w: WireSymbol) -> Option<SymbolRecord> {
    let qualified_name = w
        .qualified_name
        .filter(|s| !s.is_empty())
        .or_else(|| w.name.as_ref().filter(|s| !s.is_empty()).cloned())?;
    let symbol_name = w
        .name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| qualified_name.clone());
    let file_path = w.path.unwrap_or_default();
    let symbol_kind = w.kind.unwrap_or_else(|| "Unknown".to_string());
    let line_start = w.line.unwrap_or(0);
    Some(SymbolRecord {
        file_path,
        qualified_name,
        symbol_name,
        symbol_kind,
        line_start,
    })
}

fn dedupe_symbols(symbols: Vec<SymbolRecord>) -> Vec<SymbolRecord> {
    let mut seen: HashSet<(String, String, String)> = HashSet::new();
    let mut out = Vec::new();
    for s in symbols {
        let key = (
            s.file_path.clone(),
            s.qualified_name.clone(),
            s.symbol_kind.clone(),
        );
        if seen.insert(key) {
            out.push(s);
        }
    }
    out
}

fn symbol_memory_uuid(project_id: ProjectId, qualified_name: &str) -> Uuid {
    let key = format!("{project_id}:{qualified_name}");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes())
}

fn ingest_symbol_records(
    event_store: &dyn EventStore,
    project_id: ProjectId,
    project_root: Option<&Path>,
    symbols: Vec<SymbolRecord>,
    already_pinned: &HashSet<Uuid>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut ingested = 0usize;
    for symbol in symbols
        .into_iter()
        .filter(|symbol| symbol_in_project(&symbol.file_path, project_root))
    {
        let memory_uuid = symbol_memory_uuid(project_id, &symbol.qualified_name);
        let memory_id = MemoryId::from_uuid(memory_uuid);

        if already_pinned.contains(&memory_uuid) {
            continue;
        }

        let ev = EventBuilder::new(
            AggregateType::Memory,
            memory_uuid,
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: symbol_content(&symbol),
            session_id: None,
            project_id: Some(project_id),
            tx_id: None,
            rank: None,
            source_tag: Some(SOURCE_TAG_SYMBOL.to_string()),
            query_text: None,
        }));

        match ev {
            Ok(envelope) => {
                if let Err(e) = event_store.append_event(&envelope) {
                    tracing::warn!("Failed to store symbol memory: {}", e);
                } else {
                    ingested += 1;
                }
            }
            Err(e) => tracing::warn!("Failed to build symbol event: {}", e),
        }
    }

    Ok(ingested)
}

fn symbol_content(symbol: &SymbolRecord) -> String {
    // F44: non-route only — route method/path_pattern dropped with 0163 inventory.
    format!(
        "{} {} ({}:{})",
        symbol.symbol_kind, symbol.qualified_name, symbol.file_path, symbol.line_start
    )
}

fn symbol_in_project(file_path: &str, project_root: Option<&Path>) -> bool {
    let Some(project_root) = project_root else {
        return true;
    };
    let path = Path::new(file_path);
    // Relative inventory paths (0163 default) always pass (L6 safety net).
    if !path.is_absolute() {
        return true;
    }

    match (
        std::fs::canonicalize(path),
        std::fs::canonicalize(project_root),
    ) {
        (Ok(file), Ok(root)) => file.starts_with(root),
        // Absolute path that cannot be proven under root → drop (fail closed).
        // Codex final P2: stale/missing absolute outside root must not ingest.
        _ => false,
    }
}

/// Top-level dir names under `root` (skip dotfiles) — exposed for unit tests.
#[cfg(test)]
fn list_top_level_dirs(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return names;
    };
    for e in entries.flatten() {
        if e.file_type().map(|t| t.is_dir()).unwrap_or(false)
            && let Some(n) = e.file_name().to_str()
            && !n.starts_with('.')
        {
            names.push(n.to_string());
        }
    }
    names.sort();
    names
}

#[cfg(test)]
#[allow(non_snake_case)] // test names use `feature__condition__expected` convention
mod tests {
    use super::*;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_retrieval::{RecallOptions, recall};
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::event_store::SqliteEventStore;
    use tempfile::NamedTempFile;

    fn setup_store() -> Result<SqliteEventStore, Box<dyn std::error::Error>> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file
            .path()
            .to_str()
            .ok_or("invalid temp path")?
            .to_string();
        std::mem::forget(temp_file);
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        let conn = VaultConnection::open(&db_path, &sql_key)?;
        conn.migrate()?;
        Ok(SqliteEventStore::new(conn))
    }

    fn sample_symbol() -> SymbolRecord {
        SymbolRecord {
            file_path: "src/routes/user.rs".to_string(),
            qualified_name: "crate::routes::get_user".to_string(),
            symbol_name: "get_user".to_string(),
            symbol_kind: "Function".to_string(),
            line_start: 42,
        }
    }

    fn pin_symbol_with_tag(
        store: &SqliteEventStore,
        project_id: ProjectId,
        symbol: &SymbolRecord,
        source_tag: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let namespace = Uuid::NAMESPACE_URL;
        let key = format!("{}:{}", project_id, symbol.qualified_name);
        let memory_uuid = Uuid::new_v5(&namespace, key.as_bytes());
        let memory_id = MemoryId::from_uuid(memory_uuid);
        let envelope = EventBuilder::new(
            AggregateType::Memory,
            memory_uuid,
            Actor::System,
            Privacy::LocalOnly,
        )
        .build(Payload::MemoryPinned(MemoryPinnedPayload {
            memory_id,
            content: symbol_content(symbol),
            session_id: None,
            project_id: Some(project_id),
            tx_id: None,
            rank: None,
            source_tag: Some(source_tag.to_string()),
            query_text: None,
        }))?;
        store.append_event(&envelope)?;
        Ok(())
    }

    // --- JSON parse (O1 DoD) ---

    #[test]
    fn parse_symbols_envelope__truncated_false__ok() {
        let json = r#"{
            "schemaVersion": 1,
            "truncated": false,
            "symbols": [
                {
                    "name": "foo",
                    "kind": "Function",
                    "path": "src/foo.rs",
                    "line": 10,
                    "isPublic": true,
                    "qualifiedName": "crate::foo"
                }
            ]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok {
                symbols,
                truncated,
                total_matching,
            } => {
                assert!(!truncated);
                assert_eq!(total_matching, None);
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0].qualified_name, "crate::foo");
                assert_eq!(symbols[0].file_path, "src/foo.rs");
                assert_eq!(symbols[0].line_start, 10);
                assert_eq!(symbols[0].symbol_kind, "Function");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__truncated_true__flag_preserved() {
        let json = r#"{
            "schemaVersion": 1,
            "truncated": true,
            "totalMatching": 9000,
            "symbols": []
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok {
                symbols,
                truncated,
                total_matching,
            } => {
                assert!(truncated);
                assert_eq!(total_matching, Some(9000));
                assert!(symbols.is_empty());
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__missing_line__defaults_zero() {
        let json = r#"{
            "schemaVersion": 1,
            "symbols": [
                {
                    "name": "bar",
                    "kind": "Struct",
                    "path": "src/bar.rs",
                    "isPublic": true,
                    "qualifiedName": "crate::bar"
                }
            ]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok { symbols, .. } => {
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0].line_start, 0);
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__index_status_stale__skip() {
        let json = r#"{
            "schemaVersion": 1,
            "indexStatus": "stale",
            "symbols": [{"name":"x","kind":"Fn","path":"a.rs","qualifiedName":"x"}]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Skip { reason } => {
                assert!(reason.contains("stale"), "got {reason}");
            }
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__index_status_ok__accepted() {
        let json = r#"{
            "schemaVersion": 1,
            "indexStatus": "ok",
            "symbols": [{"name":"x","kind":"Fn","path":"a.rs","line":1,"qualifiedName":"x"}]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok { symbols, .. } => assert_eq!(symbols.len(), 1),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__index_status_missing_field__accepted() {
        let json = r#"{
            "schemaVersion": 1,
            "symbols": [{"name":"x","kind":"Fn","path":"a.rs","qualifiedName":"x"}]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok { symbols, .. } => assert_eq!(symbols.len(), 1),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    /// Codex R3 P2 / 0163: indexStatus object `{ state, remediation }` must parse + skip.
    #[test]
    fn parse_symbols_envelope__index_status_object_missing__skip() {
        let json = r#"{
            "schemaVersion": 1,
            "indexStatus": {
                "state": "missing",
                "remediation": "ledgerful index --incremental"
            },
            "symbols": []
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Skip { reason } => {
                assert!(
                    reason.contains("missing") || reason.contains("unusable"),
                    "got {reason}"
                );
            }
            other => panic!("expected Skip for object indexStatus missing, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__index_status_object_ok__accepted() {
        let json = r#"{
            "schemaVersion": 1,
            "indexStatus": { "state": "ok" },
            "symbols": [{"name":"x","kind":"Fn","path":"a.rs","line":1,"qualifiedName":"x"}]
        }"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Ok { symbols, .. } => assert_eq!(symbols.len(), 1),
            other => panic!("expected Ok for object indexStatus ok, got {other:?}"),
        }
    }

    #[test]
    fn parse_symbols_envelope__wrong_schema_version__skip() {
        let json = r#"{"schemaVersion": 2, "symbols": []}"#;
        match parse_symbols_envelope(json) {
            ParseOutcome::Skip { reason } => assert!(reason.contains("2")),
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn index_status_usable__empty_and_ok__true() {
        assert!(index_status_usable(""));
        assert!(index_status_usable("ok"));
        assert!(index_status_usable("ready"));
        assert!(index_status_usable("current"));
        assert!(!index_status_usable("missing"));
        assert!(!index_status_usable("stale"));
        assert!(!index_status_usable("error: index corrupt"));
    }

    // --- multi-pass helpers / F37 truncation honesty ---

    #[test]
    fn list_top_level_dirs__skips_dotfiles_and_sorts() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        std::fs::create_dir(root.path().join("z_last"))?;
        std::fs::create_dir(root.path().join("a_first"))?;
        std::fs::create_dir(root.path().join(".hidden"))?;
        std::fs::write(root.path().join("file.txt"), b"x")?;

        let names = list_top_level_dirs(root.path());
        assert_eq!(names, vec!["a_first".to_string(), "z_last".to_string()]);
        Ok(())
    }

    /// T373 F4: a file-only directory is queried, and a complete file pass is not incomplete.
    #[test]
    fn multi_pass_at__file_only__queries_file_and_completes()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        std::fs::write(root.path().join("only_file.rs"), b"fn x() {}")?;
        let mut calls = 0usize;
        let (syms, still_trunc) = multi_pass_at(
            root.path(),
            root.path(),
            100,
            1,
            MULTI_PASS_MAX_DEPTH,
            &mut |_root, prefix, _limit| {
                calls += 1;
                assert_eq!(prefix, Some("only_file.rs"));
                Ok(PassFetch {
                    symbols: Vec::new(),
                    truncated: false,
                    total_matching: Some(0),
                })
            },
        );
        assert_eq!(calls, 1, "a top-level file is queried");
        assert!(syms.is_empty());
        assert!(!still_trunc, "a covered file does not stay incomplete");
        Ok(())
    }

    /// Codex R1 P1 / F37: multi-pass child soft-fail must mark truncated; root must not.
    #[test]
    fn soft_fail_marks_truncated__child_prefix_true_root_false() {
        assert!(!soft_fail_marks_truncated(None));
        assert!(soft_fail_marks_truncated(Some("crates")));
        // Soft-fail child (empty + trunc=true) keeps inventory truncated after first pass.
        let first = vec![sample_symbol()];
        let (_, trunc) = collect_symbols_from_passes(first, true, Vec::new(), true);
        assert!(trunc);
    }

    /// F37: pure merge preserves truncated inventory when multi-pass is inconclusive.
    #[test]
    fn collect_symbols_from_passes__first_trunc_empty_multipass__inventory_truncated() {
        let first = vec![sample_symbol()];
        let (merged, trunc) = collect_symbols_from_passes(first.clone(), true, Vec::new(), true);
        assert!(trunc);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].qualified_name, first[0].qualified_name);
    }

    /// T373 F4: a finished descendant walk is not sticky-truncated.
    #[test]
    fn collect_symbols_from_passes__first_trunc_multipass_complete__not_sticky() {
        let first = vec![sample_symbol()];
        let more = vec![SymbolRecord {
            file_path: "b.rs".into(),
            qualified_name: "crate::bar".into(),
            symbol_name: "bar".into(),
            symbol_kind: "Function".into(),
            line_start: 1,
        }];
        let (merged, trunc) = collect_symbols_from_passes(first, true, more, false);
        assert!(!trunc);
        assert_eq!(merged.len(), 2);
    }

    /// F37: first pass not truncated → inventory flag false regardless of multi args.
    #[test]
    fn collect_symbols_from_passes__first_not_trunc__inventory_false() {
        let first = vec![sample_symbol()];
        let (merged, trunc) = collect_symbols_from_passes(first.clone(), false, Vec::new(), true);
        assert!(!trunc);
        assert_eq!(merged.len(), 1);
    }

    /// AC3: symbols invoke plan pins cwd to root and includes required flags.
    #[test]
    fn symbols_invoke_plan__cwd_is_root_and_args_include_auto_index_pub_json() {
        let root = PathBuf::from(r"C:\dev\example-root");
        let plan = symbols_invoke_plan(&root, None, 5000, true);
        assert_eq!(plan.cwd, root);
        assert!(plan.args.iter().any(|a| a == "symbols"));
        assert!(plan.args.iter().any(|a| a == "--pub"));
        assert!(plan.args.iter().any(|a| a == "--json"));
        assert!(plan.args.iter().any(|a| a == "--auto-index"));
        assert!(plan.args.iter().any(|a| a == "--limit"));
        assert!(plan.args.iter().any(|a| a == "5000"));
        assert!(!plan.args.iter().any(|a| a == "--path"));
    }

    #[test]
    fn symbols_invoke_plan__path_prefix__adds_path_arg() {
        let root = PathBuf::from("/tmp/root");
        let plan = symbols_invoke_plan(&root, Some("src/lib"), 100, false);
        assert_eq!(plan.cwd, root);
        let path_pos = plan
            .args
            .iter()
            .position(|a| a == "--path")
            .expect("--path present");
        assert_eq!(
            plan.args.get(path_pos + 1).map(String::as_str),
            Some("src/lib")
        );
        assert!(
            !plan.args.iter().any(|arg| arg == "--auto-index"),
            "child --path plans omit --auto-index"
        );
    }

    #[test]
    fn dedupe_symbols__by_path_qualified_kind() {
        let a = SymbolRecord {
            file_path: "a.rs".into(),
            qualified_name: "foo".into(),
            symbol_name: "foo".into(),
            symbol_kind: "Fn".into(),
            line_start: 1,
        };
        let mut b = a.clone();
        b.line_start = 99; // same key → dropped
        let c = SymbolRecord {
            file_path: "a.rs".into(),
            qualified_name: "foo".into(),
            symbol_name: "foo".into(),
            symbol_kind: "Struct".into(), // different kind → keep
            line_start: 2,
        };
        let out = dedupe_symbols(vec![a, b, c]);
        assert_eq!(out.len(), 2);
    }

    // --- content format (no route) ---

    #[test]
    fn symbol_content__non_route_format() {
        let symbol = sample_symbol();
        assert_eq!(
            symbol_content(&symbol),
            "Function crate::routes::get_user (src/routes/user.rs:42)"
        );
        assert!(
            !symbol_content(&symbol).contains("route "),
            "must not emit route prefix after F44"
        );
    }

    #[test]
    fn project_filter_rejects_absolute_paths_outside_root() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let outside_file = outside.path().join("outside.rs");
        std::fs::write(&outside_file, "fn outside() {}")?;

        let outside_path = outside_file
            .to_str()
            .ok_or("invalid outside path")?
            .to_string();

        assert!(!symbol_in_project(&outside_path, Some(root.path())));
        assert!(symbol_in_project("src/lib.rs", Some(root.path())));
        Ok(())
    }

    /// Codex final P2 / L6: missing absolute path cannot be proven under root → drop.
    #[test]
    fn project_filter_rejects_missing_absolute_path() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let missing = if cfg!(windows) {
            r"C:\path\that\does\not\exist\ai-brains-t233\stale.rs"
        } else {
            "/tmp/ai-brains-t233-does-not-exist/stale.rs"
        };
        assert!(
            !symbol_in_project(missing, Some(root.path())),
            "stale absolute outside root must fail closed"
        );
        Ok(())
    }

    #[test]
    fn symbol_ingestion_is_idempotent_and_recallable() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbols = vec![sample_symbol()];

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, symbols.clone(), &HashSet::new())?,
            1
        );
        let pinned = ai_brains_store::symbol_pin_ids_present(
            store.connection(),
            &[symbol_memory_uuid(
                project_id,
                &sample_symbol().qualified_name,
            )],
        )?;
        assert_eq!(
            ingest_symbol_records(&store, project_id, None, symbols, &pinned)?,
            0
        );

        let hits = recall(
            store.connection(),
            None,
            "get_user",
            5,
            RecallOptions {
                project_id: Some(project_id),
                session_id: None,
                semantic: false,
                graph_boost: 0.0,
                graph_hop_depth: 0,
                include_symbols: true,
                ..Default::default()
            },
        )?;

        assert!(hits.iter().any(|hit| {
            hit.content.contains("Function crate::routes::get_user")
                && hit.content.contains("src/routes/user.rs:42")
                && !hit.content.contains("route GET")
        }));
        Ok(())
    }

    #[test]
    fn symbol_dedup__legacy_tag_only__no_double_ingest() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbol = sample_symbol();
        pin_symbol_with_tag(&store, project_id, &symbol, SOURCE_TAG_SYMBOL_LEGACY)?;
        let pinned = ai_brains_store::symbol_pin_ids_present(
            store.connection(),
            &[symbol_memory_uuid(project_id, &symbol.qualified_name)],
        )?;
        assert!(pinned.contains(&symbol_memory_uuid(project_id, &symbol.qualified_name)));

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol], &pinned)?,
            0,
            "legacy changeguard:symbol tag must count as already ingested"
        );
        Ok(())
    }

    #[test]
    fn symbol_dedup__new_tag_only__no_double_ingest() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbol = sample_symbol();
        pin_symbol_with_tag(&store, project_id, &symbol, SOURCE_TAG_SYMBOL)?;
        let pinned = ai_brains_store::symbol_pin_ids_present(
            store.connection(),
            &[symbol_memory_uuid(project_id, &symbol.qualified_name)],
        )?;
        assert!(pinned.contains(&symbol_memory_uuid(project_id, &symbol.qualified_name)));

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol], &pinned)?,
            0,
            "ledgerful:symbol tag must count as already ingested"
        );
        Ok(())
    }

    #[test]
    fn symbol_ingest__writes_ledgerful_symbol_tag() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbol = sample_symbol();
        let namespace = Uuid::NAMESPACE_URL;
        let key = format!("{}:{}", project_id, symbol.qualified_name);
        let memory_uuid = Uuid::new_v5(&namespace, key.as_bytes());

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol], &HashSet::new())?,
            1
        );

        let events = store.read_events(memory_uuid)?;
        let tag = events.iter().find_map(|event| match &event.payload {
            Payload::MemoryPinned(payload) => payload.source_tag.as_deref(),
            _ => None,
        });
        assert_eq!(
            tag,
            Some(SOURCE_TAG_SYMBOL),
            "new symbol ingest must write ledgerful:symbol"
        );
        assert_ne!(tag, Some(SOURCE_TAG_SYMBOL_LEGACY));
        Ok(())
    }

    #[test]
    fn symbol_dedup__mixed_legacy_and_new_tags__no_double_ingest()
    -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbol = sample_symbol();
        pin_symbol_with_tag(&store, project_id, &symbol, SOURCE_TAG_SYMBOL_LEGACY)?;
        pin_symbol_with_tag(&store, project_id, &symbol, SOURCE_TAG_SYMBOL)?;
        let pinned = ai_brains_store::symbol_pin_ids_present(
            store.connection(),
            &[symbol_memory_uuid(project_id, &symbol.qualified_name)],
        )?;

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol], &pinned)?,
            0,
            "mixed legacy+new tags on same identity must still dedup"
        );
        Ok(())
    }

    /// AC14 guard: ingest path must not silently drop via bare `.take(500)`.
    #[test]
    fn ingest_symbol_records__accepts_more_than_500() -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let symbols: Vec<SymbolRecord> = (0..600)
            .map(|i| SymbolRecord {
                file_path: format!("src/f{i}.rs"),
                qualified_name: format!("crate::sym_{i}"),
                symbol_name: format!("sym_{i}"),
                symbol_kind: "Function".into(),
                line_start: i as i64,
            })
            .collect();
        let n = ingest_symbol_records(&store, project_id, None, symbols, &HashSet::new())?;
        assert_eq!(n, 600, "must not take(500); got {n}");
        Ok(())
    }

    fn record(path: &str, qualified: &str, line: i64, kind: &str) -> SymbolRecord {
        SymbolRecord {
            file_path: path.to_string(),
            qualified_name: qualified.to_string(),
            symbol_name: qualified.to_string(),
            symbol_kind: kind.to_string(),
            line_start: line,
        }
    }

    fn bookmark(head: Option<&str>, clean: bool, caught_up: bool) -> SymbolBacklogBookmark {
        SymbolBacklogBookmark {
            git_head: head.map(str::to_string),
            clean,
            caught_up,
        }
    }

    fn probe(ok: bool, head: Option<&str>, clean: bool) -> GitProbe {
        GitProbe {
            ok,
            head: head.map(str::to_string),
            clean,
        }
    }

    #[test]
    fn select_unpinned__known_prefix__ingests_later_ids() -> Result<(), Box<dyn std::error::Error>>
    {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let mut symbols = Vec::with_capacity(5001);
        let mut pinned = HashSet::with_capacity(5000);
        for index in 0..5000 {
            let qualified = format!("crate::pinned_{index}");
            pinned.insert(symbol_memory_uuid(project_id, &qualified));
            symbols.push(SymbolRecord {
                file_path: "a.rs".to_string(),
                qualified_name: qualified.clone(),
                symbol_name: qualified,
                symbol_kind: "Function".to_string(),
                line_start: index as i64,
            });
        }
        symbols.push(record("z.rs", "crate::later", 1, "Function"));
        let (selected, before) = select_unpinned(symbols, &pinned, project_id, 5000);
        assert_eq!(before, 1);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].qualified_name, "crate::later");
        assert_eq!(
            ingest_symbol_records(&store, project_id, None, selected, &pinned)?,
            1
        );
        Ok(())
    }

    #[test]
    fn select_unpinned__budget__sort_order_not_first_seen_prefix() {
        let project_id = ProjectId::new();
        let symbols = vec![
            record("e.rs", "crate::e", 9, "Struct"),
            record("a.rs", "crate::a2", 2, "Function"),
            record("a.rs", "crate::a1", 2, "Function"),
            record("a.rs", "crate::a1", 1, "Function"),
            record("b.rs", "crate::b", 1, "Function"),
        ];
        let (selected, before) = select_unpinned(symbols, &HashSet::new(), project_id, 2);
        assert_eq!(before, 5);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].file_path, "a.rs");
        assert_eq!(selected[0].line_start, 1);
        assert_eq!(selected[0].qualified_name, "crate::a1");
        assert_eq!(selected[1].file_path, "a.rs");
        assert_eq!(selected[1].line_start, 2);
        assert_eq!(selected[1].qualified_name, "crate::a1");
    }

    #[test]
    fn should_skip_symbol_walk__caught_up_clean_same_head__true() {
        let saved = bookmark(Some("abc"), true, true);
        assert!(should_skip_symbol_walk(
            Some(&saved),
            &probe(true, Some("abc"), true)
        ));
    }

    #[test]
    fn should_skip_symbol_walk__same_head_backlog_remains__false() {
        let saved = bookmark(Some("abc"), true, false);
        assert!(!should_skip_symbol_walk(
            Some(&saved),
            &probe(true, Some("abc"), true)
        ));
    }

    #[test]
    fn should_skip_symbol_walk__dirty_or_git_failure__false() {
        let saved = bookmark(Some("abc"), true, true);
        assert!(!should_skip_symbol_walk(
            Some(&saved),
            &probe(true, Some("abc"), false)
        ));
        assert!(!should_skip_symbol_walk(
            Some(&saved),
            &probe(false, None, false)
        ));
        assert!(!should_skip_symbol_walk(
            Some(&saved),
            &probe(true, None, true)
        ));
        assert!(!should_skip_symbol_walk(
            None,
            &probe(true, Some("abc"), true)
        ));
        let dirty_book = bookmark(Some("abc"), false, true);
        assert!(!should_skip_symbol_walk(
            Some(&dirty_book),
            &probe(true, Some("abc"), true)
        ));
        let other_head = bookmark(Some("def"), true, true);
        assert!(!should_skip_symbol_walk(
            Some(&other_head),
            &probe(true, Some("abc"), true)
        ));
    }

    #[test]
    fn coverage_caught_up__short_collect_or_unpinned_or_child_fail__false() {
        assert!(!coverage_caught_up(1, Some(5), false, false));
        assert!(!coverage_caught_up(5, Some(5), true, false));
        assert!(!coverage_caught_up(5, Some(5), false, true));
        assert!(!coverage_caught_up(5, None, false, false));
        assert!(coverage_caught_up(5, Some(5), false, false));
        assert!(coverage_caught_up(6, Some(5), false, false));
    }

    #[test]
    fn parse_symbols_envelope__total_matching_and_root_failure_distinct() {
        let present = r#"{"schemaVersion":1,"truncated":true,"totalMatching":9000,"symbols":[]}"#;
        match parse_symbols_envelope(present) {
            ParseOutcome::Ok {
                total_matching,
                truncated,
                symbols,
            } => {
                assert_eq!(total_matching, Some(9000));
                assert!(truncated);
                assert!(symbols.is_empty());
            }
            other => panic!("expected Ok with totalMatching, got {other:?}"),
        }
        let missing = r#"{"schemaVersion":1,"truncated":false,"symbols":[]}"#;
        match parse_symbols_envelope(missing) {
            ParseOutcome::Ok { total_matching, .. } => assert_eq!(total_matching, None),
            other => panic!("expected Ok without totalMatching, got {other:?}"),
        }
        assert!(matches!(
            parse_symbols_envelope("not-json"),
            ParseOutcome::Err(_)
        ));
        assert!(matches!(
            parse_symbols_envelope(r#"{"schemaVersion":2,"symbols":[]}"#),
            ParseOutcome::Skip { .. }
        ));
        assert!(pass_from_outcome(None, ParseOutcome::Err("json: nope".into())).is_err());
        assert!(
            pass_from_outcome(
                None,
                ParseOutcome::Skip {
                    reason: "schema".into(),
                },
            )
            .is_err()
        );
        let child = pass_from_outcome(Some("src"), ParseOutcome::Err("boom".into()));
        assert!(matches!(
            child,
            Ok(PassFetch {
                truncated: true,
                ..
            })
        ));
    }

    #[test]
    fn descend__truncated_mixed_dir_queries_child_dir_and_files()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        std::fs::create_dir(root.path().join("src"))?;
        std::fs::write(root.path().join("main.rs"), b"fn main() {}")?;
        std::fs::create_dir(root.path().join(".git"))?;
        let mut queried = Vec::new();
        let outcome =
            collect_symbols_with_fetch(root.path(), 100, &mut |_root, prefix, _limit| {
                queried.push(prefix.map(str::to_string));
                if prefix.is_none() {
                    Ok(PassFetch {
                        symbols: vec![record("root.rs", "crate::root", 1, "Function")],
                        truncated: true,
                        total_matching: Some(3),
                    })
                } else {
                    Ok(PassFetch {
                        symbols: Vec::new(),
                        truncated: false,
                        total_matching: Some(0),
                    })
                }
            })?;
        assert!(!outcome.walk_incomplete);
        assert!(queried.contains(&None));
        assert!(queried.contains(&Some("src".to_string())));
        assert!(queried.contains(&Some("main.rs".to_string())));
        assert!(
            !queried
                .iter()
                .any(|prefix| { prefix.as_deref().is_some_and(|path| path.starts_with('.')) })
        );
        Ok(())
    }

    #[test]
    fn descend__depth_past_8__coverage_incomplete() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut dir = root.path().to_path_buf();
        for index in 1..=9 {
            dir.push(format!("d{index}"));
            std::fs::create_dir(&dir)?;
        }
        let mut prefixes = Vec::new();
        let outcome = collect_symbols_with_fetch(root.path(), 10, &mut |_root, prefix, _limit| {
            prefixes.push(prefix.map(str::to_string));
            Ok(PassFetch {
                symbols: Vec::new(),
                truncated: true,
                total_matching: Some(1),
            })
        })?;
        assert!(outcome.walk_incomplete);
        assert!(
            prefixes
                .iter()
                .any(|prefix| prefix.as_deref() == Some("d1"))
        );
        assert!(prefixes.iter().any(|prefix| {
            prefix
                .as_deref()
                .is_some_and(|path| path == "d1/d2/d3/d4/d5/d6/d7/d8")
        }));
        assert!(
            prefixes
                .iter()
                .all(|prefix| { prefix.as_deref().is_none_or(|path| !path.contains("d9")) }),
            "depth 9 must not be queried: {prefixes:?}"
        );
        Ok(())
    }

    #[test]
    fn collect_symbols_with_fetch__child_err__incomplete() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        std::fs::write(root.path().join("a.rs"), b"fn a() {}")?;
        let outcome = collect_symbols_with_fetch(root.path(), 10, &mut |_root, prefix, _limit| {
            if prefix.is_none() {
                Ok(PassFetch {
                    symbols: vec![sample_symbol()],
                    truncated: true,
                    total_matching: Some(2),
                })
            } else {
                Err("child down".into())
            }
        })?;
        assert!(outcome.walk_incomplete);
        assert!(!coverage_caught_up(
            outcome.symbols_returned,
            outcome.total_matching,
            outcome.walk_incomplete,
            false,
        ));
        Ok(())
    }

    #[test]
    fn ingest_symbols__root_fetch_failure__bookmark_unchanged()
    -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let root = tempfile::tempdir()?;
        let key = backlog_state_key(project_id, root.path());
        let seeded = r#"{"git_head":"deadbeef","clean":false,"caught_up":true}"#;
        store.set_sync_state(&key, seeded)?;
        let mut calls = 0usize;
        let ingested = ingest_symbols_with_fetch(
            &store,
            store.connection(),
            project_id,
            root.path(),
            5000,
            &mut |_root, _prefix, _limit| {
                calls += 1;
                Err("spawn failed".into())
            },
        )?;
        assert_eq!(ingested, 0);
        assert_eq!(calls, 1, "root failure must be reached, not skipped");
        assert_eq!(store.get_sync_state(&key)?.as_deref(), Some(seeded));
        Ok(())
    }

    #[test]
    fn ingest_symbols__empty_success__bookmark_caught_up() -> Result<(), Box<dyn std::error::Error>>
    {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let root = tempfile::tempdir()?;
        let ingested = ingest_symbols_with_fetch(
            &store,
            store.connection(),
            project_id,
            root.path(),
            5000,
            &mut |_root, prefix, _limit| {
                assert!(prefix.is_none());
                Ok(PassFetch {
                    symbols: Vec::new(),
                    truncated: false,
                    total_matching: Some(0),
                })
            },
        )?;
        assert_eq!(ingested, 0);
        let raw = store
            .get_sync_state(&backlog_state_key(project_id, root.path()))?
            .expect("empty success writes the bookmark");
        let saved: SymbolBacklogBookmark = serde_json::from_str(&raw)?;
        assert!(saved.caught_up);
        Ok(())
    }

    #[test]
    fn ingest_symbols__final_batch_fits__bookmark_caught_up()
    -> Result<(), Box<dyn std::error::Error>> {
        let store = setup_store()?;
        let project_id = ProjectId::new();
        let root = tempfile::tempdir()?;
        let ingested = ingest_symbols_with_fetch(
            &store,
            store.connection(),
            project_id,
            root.path(),
            5000,
            &mut |_root, prefix, _limit| {
                assert!(prefix.is_none());
                Ok(PassFetch {
                    symbols: vec![sample_symbol()],
                    truncated: false,
                    total_matching: Some(1),
                })
            },
        )?;
        assert_eq!(ingested, 1);
        let raw = store
            .get_sync_state(&backlog_state_key(project_id, root.path()))?
            .expect("final batch writes the bookmark");
        let saved: SymbolBacklogBookmark = serde_json::from_str(&raw)?;
        assert!(saved.caught_up);
        Ok(())
    }

    #[test]
    fn bookmark_from_probe__git_failure_keeps_head_null() {
        let saved = bookmark_from_probe(
            &GitProbe {
                ok: false,
                head: Some("abc".to_string()),
                clean: false,
            },
            true,
        );
        assert_eq!(saved.git_head, None);
        assert!(!saved.clean);
        assert!(saved.caught_up);
    }

    fn git_in(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git").args(args).current_dir(root).output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into())
        }
    }

    #[test]
    fn probe_git_root__clean_commit__ok_none_is_clean() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        git_in(root.path(), &["init"])?;
        std::fs::write(root.path().join("f.txt"), b"a")?;
        git_in(root.path(), &["add", "f.txt"])?;
        git_in(
            root.path(),
            &[
                "-c",
                "user.email=t373@example.com",
                "-c",
                "user.name=T373",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-m",
                "init",
            ],
        )?;
        let clean = probe_git_root(root.path());
        assert!(clean.ok, "clean probe should succeed");
        assert!(clean.clean, "empty porcelain is clean");
        let head = clean.head.clone().expect("rev-parse returns a sha");
        let saved = SymbolBacklogBookmark {
            git_head: Some(head),
            clean: true,
            caught_up: true,
        };
        assert!(should_skip_symbol_walk(Some(&saved), &clean));
        std::fs::write(root.path().join("dirty.txt"), b"b")?;
        let dirty = probe_git_root(root.path());
        assert!(dirty.ok);
        assert!(!dirty.clean);
        assert!(!should_skip_symbol_walk(Some(&saved), &dirty));
        Ok(())
    }
}
