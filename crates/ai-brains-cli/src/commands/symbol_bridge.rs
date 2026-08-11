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
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use ai_brains_store::EventStore;

/// Legacy source_tag written by pre-T191 symbol ingest (durable in vault events).
pub const SOURCE_TAG_SYMBOL_LEGACY: &str = "changeguard:symbol";
/// Canonical source_tag for new symbol ingest writes (T191 F2).
pub const SOURCE_TAG_SYMBOL: &str = "ledgerful:symbol";

/// Default / hard-max symbols per root (matches ledgerful CLI hard max).
const DEFAULT_MAX_SYMBOLS: usize = 5000;

/// Multi-pass recursion depth when inventory reports `truncated` (F37).
const MULTI_PASS_MAX_DEPTH: u32 = 2;

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

    let max_n = max_symbols_from_env();
    let (symbols, metrics) = collect_symbols_for_root(root, max_n);

    tracing::info!(
        root = %root.display(),
        symbols_returned = metrics.symbols_returned,
        symbols_ingested_cap = max_n,
        symbols_truncated_inventory = metrics.symbols_truncated_inventory,
        symbols_truncated_by_ingest_cap = metrics.symbols_truncated_by_ingest_cap,
        "[Nightly] symbol inventory metrics"
    );
    if metrics.symbols_truncated_inventory {
        tracing::warn!(
            root = %root.display(),
            "[Nightly] symbol inventory still truncated after multi-pass; partial ingest"
        );
    }
    if metrics.symbols_truncated_by_ingest_cap {
        tracing::warn!(
            root = %root.display(),
            cap = max_n,
            returned = metrics.symbols_returned,
            "[Nightly] symbol ingest capped by AI_BRAINS_NIGHTLY_MAX_SYMBOLS"
        );
    }

    if symbols.is_empty() {
        tracing::info!(
            root = %root.display(),
            "No symbols returned from Ledgerful inventory"
        );
        return Ok(0);
    }

    #[cfg(feature = "graph")]
    let event_store = crate::live_graph::GraphAwareEventStore::new((*ctx.conn).clone());
    #[cfg(not(feature = "graph"))]
    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());

    let ingested = ingest_symbol_records(&event_store, project_id, Some(root), symbols)?;
    tracing::info!(
        root = %root.display(),
        symbols_ingested = ingested,
        "[Nightly] symbols ingested"
    );
    Ok(ingested)
}

#[derive(Debug, Default, Clone, Copy)]
struct InventoryMetrics {
    symbols_returned: usize,
    symbols_truncated_inventory: bool,
    symbols_truncated_by_ingest_cap: bool,
}

fn max_symbols_from_env() -> usize {
    std::env::var("AI_BRAINS_NIGHTLY_MAX_SYMBOLS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .map(|n| n.min(DEFAULT_MAX_SYMBOLS))
        .unwrap_or(DEFAULT_MAX_SYMBOLS)
}

/// Pure merge of first-pass + multi-pass results (F37 honesty, unit-testable).
///
/// When the first pass was truncated, inventory **stays truncated** after multi-pass
/// (Codex R3 P1). Multi-pass only walks **child directories** via `--path`; root-level
/// files are never re-fetched, so clearing the flag would silently claim complete.
/// `multi_still_trunc` is retained for metrics/tests of multi-pass coverage quality
/// but does not clear inventory honesty.
fn collect_symbols_from_passes(
    first_symbols: Vec<SymbolRecord>,
    first_truncated: bool,
    multi_more: Vec<SymbolRecord>,
    multi_still_trunc: bool,
) -> (
    Vec<SymbolRecord>,
    bool, /* symbols_truncated_inventory */
) {
    let _ = multi_still_trunc; // coverage signal only; does not clear root trunc
    if !first_truncated {
        return (first_symbols, false);
    }
    let mut symbols = first_symbols;
    symbols.extend(multi_more);
    (dedupe_symbols(symbols), true)
}

/// Collect symbols for a root: one pass + optional multi-pass on truncated.
fn collect_symbols_for_root(root: &Path, max_n: usize) -> (Vec<SymbolRecord>, InventoryMetrics) {
    let mut metrics = InventoryMetrics::default();
    let (first_symbols, truncated) = match fetch_symbols_pass(root, None, max_n) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                root = %root.display(),
                error = %e,
                "[Nightly] symbol inventory failed (non-fatal)"
            );
            return (Vec::new(), metrics);
        }
    };

    let (mut symbols, inventory_trunc) = if truncated {
        let (more, still_trunc) = multi_pass_symbols(root, max_n, MULTI_PASS_MAX_DEPTH);
        collect_symbols_from_passes(first_symbols, true, more, still_trunc)
    } else {
        collect_symbols_from_passes(first_symbols, false, Vec::new(), false)
    };
    metrics.symbols_truncated_inventory = inventory_trunc;

    metrics.symbols_returned = symbols.len();
    if symbols.len() > max_n {
        metrics.symbols_truncated_by_ingest_cap = true;
        symbols.truncate(max_n);
    }
    (symbols, metrics)
}

/// Multi-pass F37: walk top-level dirs, re-invoke with `--path <name>`, depth ≤ 2.
fn multi_pass_symbols(
    root: &Path,
    max_n: usize,
    max_depth: u32,
) -> (Vec<SymbolRecord>, bool /* still_truncated */) {
    multi_pass_at(root, root, max_n, 1, max_depth)
}

fn multi_pass_at(
    root: &Path,
    dir: &Path,
    max_n: usize,
    depth: u32,
    max_depth: u32,
) -> (Vec<SymbolRecord>, bool) {
    if depth > max_depth {
        return (Vec::new(), true);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(
                dir = %dir.display(),
                error = %e,
                "[Nightly] multi-pass read_dir failed"
            );
            // Inconclusive: parent was truncated; cannot prove full coverage.
            return (Vec::new(), true);
        }
    };

    let mut all = Vec::new();
    let mut any_trunc = false;

    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| !n.starts_with('.'))
        })
        .collect();
    dirs.sort();

    // Parent was truncated; no child dirs to expand → still truncated (F37).
    if dirs.is_empty() {
        return (Vec::new(), true);
    }

    for sub in dirs {
        let Some(name) = sub.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Prefix relative to root for --path
        let rel = match sub.strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => name.to_string(),
        };

        match fetch_symbols_pass(root, Some(&rel), max_n) {
            Ok((syms, trunc)) => {
                all.extend(syms);
                if trunc {
                    if depth < max_depth {
                        let (more, still) = multi_pass_at(root, &sub, max_n, depth + 1, max_depth);
                        all.extend(more);
                        if still {
                            any_trunc = true;
                        }
                    } else {
                        any_trunc = true;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    root = %root.display(),
                    path = %rel,
                    error = %e,
                    "[Nightly] multi-pass symbol fetch failed (non-fatal)"
                );
                // Child failure is inconclusive coverage — stay truncated (F37).
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
) -> SymbolsInvokePlan {
    let mut args = vec![
        "symbols".to_string(),
        "--pub".to_string(),
        "--json".to_string(),
        "--limit".to_string(),
        limit.to_string(),
        "--auto-index".to_string(),
    ];
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

/// One `ledgerful symbols` invocation. Returns (symbols, truncated flag).
///
/// Soft failures (spawn err, nonzero, bad schema, index skip, parse fail) return
/// empty symbols. For **multi-pass children** (`path_prefix` is Some), soft failure
/// sets `truncated=true` so F37 never silent-completes after a truncated root pass
/// (Codex R1 P1). Root-pass soft failure keeps `truncated=false` (no multi-pass claim).
fn fetch_symbols_pass(
    root: &Path,
    path_prefix: Option<&str>,
    limit: usize,
) -> Result<(Vec<SymbolRecord>, bool), String> {
    let plan = symbols_invoke_plan(root, path_prefix, limit);
    // Multi-pass child soft-fail must not clear inventory truncation (F37).
    let soft_fail_trunc = soft_fail_marks_truncated(path_prefix);

    #[allow(clippy::disallowed_methods)]
    let output = match Command::new("ledgerful")
        .current_dir(&plan.cwd)
        .args(&plan.args)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!(
                root = %root.display(),
                error = %e,
                "[Nightly] ledgerful not available for symbols (non-fatal)"
            );
            return Ok((Vec::new(), soft_fail_trunc));
        }
    };

    if !output.status.success() {
        tracing::warn!(
            root = %root.display(),
            stderr = %String::from_utf8_lossy(&output.stderr).trim(),
            "[Nightly] ledgerful symbols non-zero (non-fatal)"
        );
        return Ok((Vec::new(), soft_fail_trunc));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    match parse_symbols_envelope(stdout.trim()) {
        ParseOutcome::Ok { symbols, truncated } => Ok((symbols, truncated)),
        ParseOutcome::Skip { reason } => {
            tracing::warn!(
                root = %root.display(),
                reason = %reason,
                "[Nightly] symbol inventory skipped"
            );
            Ok((Vec::new(), soft_fail_trunc))
        }
        ParseOutcome::Err(e) => {
            tracing::warn!(
                root = %root.display(),
                error = %e,
                "[Nightly] symbol JSON parse failed (non-fatal)"
            );
            Ok((Vec::new(), soft_fail_trunc))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseOutcome {
    Ok {
        symbols: Vec<SymbolRecord>,
        truncated: bool,
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

fn ingest_symbol_records(
    event_store: &dyn EventStore,
    project_id: ProjectId,
    project_root: Option<&Path>,
    symbols: Vec<SymbolRecord>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut ingested = 0usize;
    for symbol in symbols
        .into_iter()
        .filter(|symbol| symbol_in_project(&symbol.file_path, project_root))
    {
        let namespace = Uuid::NAMESPACE_URL;
        let key = format!("{}:{}", project_id, symbol.qualified_name);
        let memory_uuid = Uuid::new_v5(&namespace, key.as_bytes());
        let memory_id = MemoryId::from_uuid(memory_uuid);

        if symbol_already_ingested(event_store, memory_uuid) {
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

fn symbol_already_ingested(event_store: &dyn EventStore, memory_uuid: Uuid) -> bool {
    event_store
        .read_events(memory_uuid)
        .map(|events| {
            events.iter().any(|event| match &event.payload {
                Payload::MemoryPinned(payload) => {
                    is_symbol_source_tag(payload.source_tag.as_deref())
                }
                _ => false,
            })
        })
        .unwrap_or(false)
}

/// Dual-read: either legacy or canonical symbol source_tag counts as ingested (F2).
fn is_symbol_source_tag(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some(SOURCE_TAG_SYMBOL_LEGACY) | Some(SOURCE_TAG_SYMBOL)
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
            ParseOutcome::Ok { symbols, truncated } => {
                assert!(!truncated);
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
            ParseOutcome::Ok { symbols, truncated } => {
                assert!(truncated);
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

    /// F37 / T233-R1: first-pass truncated + empty top-level dirs → inventory stays truncated.
    #[test]
    fn multi_pass_at__empty_top_level_dirs__still_truncated()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        // Files only — no expandable child dirs.
        std::fs::write(root.path().join("only_file.rs"), b"fn x() {}")?;
        let (syms, still_trunc) = multi_pass_at(root.path(), root.path(), 100, 1, 2);
        assert!(syms.is_empty());
        assert!(
            still_trunc,
            "empty child dirs after truncated first-pass must not silent-complete"
        );
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

    /// F37 / Codex R3 P1: multi-pass cannot clear inventory trunc (root files not re-covered).
    #[test]
    fn collect_symbols_from_passes__first_trunc_multipass_complete__still_truncated() {
        let first = vec![sample_symbol()];
        let more = vec![SymbolRecord {
            file_path: "b.rs".into(),
            qualified_name: "crate::bar".into(),
            symbol_name: "bar".into(),
            symbol_kind: "Function".into(),
            line_start: 1,
        }];
        let (merged, trunc) = collect_symbols_from_passes(first, true, more, false);
        assert!(
            trunc,
            "first-pass trunc must remain true after multi-pass (root files never re-fetched)"
        );
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
        let plan = symbols_invoke_plan(&root, None, 5000);
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
        let plan = symbols_invoke_plan(&root, Some("src/lib"), 100);
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
            ingest_symbol_records(&store, project_id, None, symbols.clone())?,
            1
        );
        assert_eq!(ingest_symbol_records(&store, project_id, None, symbols)?, 0);

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

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol])?,
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

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol])?,
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
            ingest_symbol_records(&store, project_id, None, vec![symbol])?,
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

        assert_eq!(
            ingest_symbol_records(&store, project_id, None, vec![symbol])?,
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
        let n = ingest_symbol_records(&store, project_id, None, symbols)?;
        assert_eq!(n, 600, "must not take(500); got {n}");
        Ok(())
    }
}
