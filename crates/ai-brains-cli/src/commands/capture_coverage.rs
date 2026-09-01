//! T337 — read-only `ai-brains capture coverage` (disk files vs vault SessionStarted).
//!
//! Inventory + event-log query only. Never opens JSONL turn bodies or Grok
//! `summary.json`. Never imports. Never adds a doctor check.

use crate::commands::governed_common::fail_usage;
use crate::commands::multi_import::{
    MultiImportReport, MultiImportStatusView, SourceImportReport, load_multi_import_status,
};
use crate::context::AppContext;
use ai_brains_adapters::{
    CLAUDE_HARNESS_UUID, CODEX_HARNESS_UUID, CURSOR_HARNESS_UUID, GROK_HARNESS_UUID,
    OPENCODE_HARNESS_UUID, cursor_project_slug_candidates, discover_cursor_sessions,
    discover_sessions_from_home, is_claude_sidechain_path, is_cursor_sidechain_path,
    is_subagent_session, resolve_claude_home, resolve_codex_home, resolve_cursor_home,
    resolve_grok_home,
};
use ai_brains_core::ids::ProjectId;
use ai_brains_store::QueryStore;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const AGY_IMPORT_UUID: &str = "00000000-0000-0000-0000-000000000001";
const AGY_HOOK_UUID: &str = "00000000-0000-0000-0000-000000000002";

const SOURCE_ORDER: [&str; 6] = ["agy", "grok", "opencode", "claude", "codex", "cursor"];
const NEXT_STEP_MAX: usize = 140;
const SCOPE_MISSING_MSG: &str =
    "No project scope. Set AI_BRAINS_PROJECT_ID, run `ai-brains context`, or pass --global.";

pub struct CoverageOptions {
    pub days: usize,
    pub format: String,
    pub global: bool,
    pub project_id: Option<ProjectId>,
    /// Hermetic tests: user-home root (`.cursor` / `.grok` / … live under this).
    pub home_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CoverageReport {
    pub days: usize,
    pub sources: Vec<SourceCoverage>,
    pub unbound_folders: Vec<String>,
    pub warnings: Vec<String>,
    pub multi_import: Option<MultiImportReport>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SourceCoverage {
    pub source: String,
    pub mode: String,
    pub disk_eligible: Option<u64>,
    pub disk_skipped_sidechain: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_note: Option<String>,
    pub vault_sessions: u64,
    pub status: String,
    pub next_step: String,
}

pub fn run(ctx: &AppContext, opts: CoverageOptions) -> Result<(), Box<dyn std::error::Error>> {
    if !opts.global && opts.project_id.is_none() {
        return fail_usage(SCOPE_MISSING_MSG);
    }
    let report = build_report(ctx.conn.as_ref(), &opts)?;
    if opts.format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }
    Ok(())
}

pub fn build_report(
    query_store: &dyn QueryStore,
    opts: &CoverageOptions,
) -> Result<CoverageReport, Box<dyn std::error::Error>> {
    let dirs_home = opts.home_override.clone().or_else(dirs::home_dir);
    let harness_override = opts.home_override.as_deref();
    let cutoff = cutoff_for_days(opts.days);
    let project = if opts.global {
        None
    } else {
        opts.project_id.as_ref()
    };
    let include_wsl_agy = opts.home_override.is_none();

    let mut warnings = Vec::new();
    let multi_view = load_multi_import_status(query_store)?;
    let multi_import = match &multi_view {
        MultiImportStatusView::Never => None,
        MultiImportStatusView::Unreadable => {
            warnings.push("unreadable".to_string());
            None
        }
        MultiImportStatusView::Report(report) => {
            if source_is_absent_pre_t334(&report.agy)
                || source_is_absent_pre_t334(&report.grok)
                || source_is_absent_pre_t334(&report.opencode)
                || source_is_absent_pre_t334(&report.claude)
                || source_is_absent_pre_t334(&report.codex)
                || source_is_absent_pre_t334(&report.cursor)
            {
                warnings.push("stale_multi_import".to_string());
            }
            Some((**report).clone())
        }
    };

    let opencode_missing = multi_import
        .as_ref()
        .is_some_and(|r| r.opencode.skipped_missing_binary.unwrap_or(0) > 0);

    let agy_disk = count_agy_disk(dirs_home.as_deref(), cutoff, include_wsl_agy);
    let grok_disk = count_grok_disk(harness_override, cutoff);
    let claude_disk = count_claude_disk(harness_override, cutoff);
    let codex_disk = count_codex_disk(harness_override, cutoff);
    let cursor_disk = count_cursor_disk(harness_override, cutoff);

    let agy_vault = query_store
        .count_sessions_started_by_harness(&[AGY_IMPORT_UUID, AGY_HOOK_UUID], project)?;
    let grok_vault =
        query_store.count_sessions_started_by_harness(&[GROK_HARNESS_UUID], project)?;
    let opencode_vault =
        query_store.count_sessions_started_by_harness(&[OPENCODE_HARNESS_UUID], project)?;
    let claude_vault =
        query_store.count_sessions_started_by_harness(&[CLAUDE_HARNESS_UUID], project)?;
    let codex_vault =
        query_store.count_sessions_started_by_harness(&[CODEX_HARNESS_UUID], project)?;
    let cursor_vault =
        query_store.count_sessions_started_by_harness(&[CURSOR_HARNESS_UUID], project)?;

    let agy = classify_source(
        "agy",
        "hook+import",
        Some(agy_disk.eligible),
        agy_disk.sidechain,
        agy_vault,
        opts.days,
        false,
        "antigravity-import",
    );
    let grok = classify_grok(
        grok_disk.eligible,
        grok_disk.sidechain,
        grok_vault,
        opts.days,
    );
    let opencode = classify_opencode(opencode_vault, opencode_missing);
    let claude = classify_source(
        "claude",
        "hook+import",
        Some(claude_disk.eligible),
        claude_disk.sidechain,
        claude_vault,
        opts.days,
        false,
        "claude-import",
    );
    let codex = classify_source(
        "codex",
        "hook+import",
        Some(codex_disk.eligible),
        codex_disk.sidechain,
        codex_vault,
        opts.days,
        false,
        "codex-import",
    );
    let cursor = classify_source(
        "cursor",
        "import_only",
        Some(cursor_disk.eligible),
        cursor_disk.sidechain,
        cursor_vault,
        opts.days,
        false,
        "cursor-import",
    );

    if grok.status == "unverifiable_subagent" {
        warnings.push("grok_batch_empty_all_subagent".to_string());
    }
    if agy_disk.unreadable
        || grok_disk.unreadable
        || claude_disk.unreadable
        || codex_disk.unreadable
        || cursor_disk.unreadable
    {
        push_warn(&mut warnings, "disk_walk_unreadable");
    }

    let unbound_folders = match query_store.list_path_aliases() {
        Ok(aliases) => {
            let cursor_home = resolve_cursor_home(harness_override);
            match cursor_home.as_deref() {
                Some(h) => {
                    let (folders, unreadable) = cursor_unbound_folders(h, &aliases);
                    if unreadable {
                        push_warn(&mut warnings, "disk_walk_unreadable");
                    }
                    folders
                }
                None => Vec::new(),
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "list_path_aliases unreadable");
            push_warn(&mut warnings, "alias_unreadable");
            Vec::new()
        }
    };

    let sources = vec![agy, grok, opencode, claude, codex, cursor];
    for (row, expected) in sources.iter().zip(SOURCE_ORDER.iter()) {
        if row.source != *expected {
            tracing::warn!(
                source = %row.source,
                expected,
                "coverage source order mismatch"
            );
        }
    }

    Ok(CoverageReport {
        days: opts.days,
        sources,
        unbound_folders,
        warnings,
        multi_import,
    })
}

#[derive(Default)]
struct DiskCounts {
    eligible: u64,
    sidechain: u64,
    unreadable: bool,
}

fn push_warn(warnings: &mut Vec<String>, code: &str) {
    if !warnings.iter().any(|w| w == code) {
        warnings.push(code.to_string());
    }
}

fn is_absent_path(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    )
}

/// Present directory that cannot be listed is honesty-unreadable.
/// Missing path is zero files, not a warning.
///
/// Do not use `Path::is_dir()` / `exists()` as the absent gate — those collapse
/// permission/metadata failures to `false` (looks like missing).
fn read_existing_dir(dir: &Path, counts: &mut DiskCounts) -> Option<std::fs::ReadDir> {
    match std::fs::read_dir(dir) {
        Ok(rd) => Some(rd),
        Err(e) if is_absent_path(&e) => None,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %dir.display(),
                "coverage disk walk unreadable"
            );
            counts.unreadable = true;
            None
        }
    }
}

fn dirent_is_dir(entry: &std::fs::DirEntry, counts: &mut DiskCounts) -> bool {
    let path = entry.path();
    match std::fs::metadata(&path) {
        Ok(meta) => meta.is_dir(),
        Err(e) if is_absent_path(&e) => false,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "coverage metadata unreadable"
            );
            counts.unreadable = true;
            false
        }
    }
}

fn cutoff_for_days(days: usize) -> SystemTime {
    let secs = (days as u64).saturating_mul(86_400);
    SystemTime::now()
        .checked_sub(Duration::from_secs(secs))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn mtime_in_window(path: &Path, cutoff: SystemTime, counts: &mut DiskCounts) -> bool {
    match std::fs::metadata(path).and_then(|m| m.modified()) {
        Ok(t) => t >= cutoff,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %path.display(),
                "coverage mtime unreadable"
            );
            counts.unreadable = true;
            false
        }
    }
}

fn source_is_absent_pre_t334(report: &SourceImportReport) -> bool {
    report.status == "skipped" && report.skip_reason.as_deref() == Some("absent_pre_t334")
}

fn clip_next_step(s: String) -> String {
    if s.chars().count() <= NEXT_STEP_MAX {
        return s;
    }
    s.chars().take(NEXT_STEP_MAX).collect()
}

#[allow(clippy::too_many_arguments)]
fn classify_source(
    name: &str,
    mode: &str,
    disk_eligible: Option<u64>,
    sidechain: u64,
    vault_sessions: u64,
    days: usize,
    opencode_missing: bool,
    import_cmd: &str,
) -> SourceCoverage {
    let eligible = disk_eligible.unwrap_or(0);
    let mut status = "ok".to_string();
    let mut next_step = String::new();

    if opencode_missing {
        status = "expected_skip".to_string();
        next_step = "set AI_BRAINS_OPENCODE_BIN".to_string();
    } else if disk_eligible.is_some() && eligible == 0 && sidechain > 0 {
        status = "expected_skip".to_string();
    } else if disk_eligible.is_some() && eligible > 0 && vault_sessions == 0 {
        status = "deficit".to_string();
        next_step = format!("ai-brains {import_cmd} --days {days}");
    }

    SourceCoverage {
        source: name.to_string(),
        mode: mode.to_string(),
        disk_eligible,
        disk_skipped_sidechain: sidechain,
        disk_note: None,
        vault_sessions,
        status,
        next_step: clip_next_step(next_step),
    }
}

fn classify_grok(
    eligible: u64,
    sidechain: u64,
    vault_sessions: u64,
    days: usize,
) -> SourceCoverage {
    let mut row = classify_source(
        "grok",
        "hook+import",
        Some(eligible),
        sidechain,
        vault_sessions,
        days,
        false,
        "grok-import",
    );
    if eligible > 0 && vault_sessions == 0 {
        row.status = "unverifiable_subagent".to_string();
        row.next_step = clip_next_step(format!("ai-brains grok-import --days {days} --dry-run"));
    }
    row
}

fn classify_opencode(vault_sessions: u64, missing_binary: bool) -> SourceCoverage {
    let mut row = classify_source(
        "opencode",
        "hook+import",
        None,
        0,
        vault_sessions,
        30,
        missing_binary,
        "opencode-import",
    );
    row.disk_note = Some("requires_opencode_bin".to_string());
    row
}

fn count_agy_disk(home: Option<&Path>, cutoff: SystemTime, include_wsl: bool) -> DiskCounts {
    match discover_sessions_from_home(home, include_wsl) {
        Ok(sources) => {
            let mut counts = DiskCounts::default();
            for src in sources {
                if mtime_in_window(&src.path, cutoff, &mut counts) {
                    counts.eligible += 1;
                }
            }
            counts
        }
        Err(e) => {
            tracing::warn!(error = %e, "agy discover unreadable");
            DiskCounts {
                eligible: 0,
                sidechain: 0,
                unreadable: true,
            }
        }
    }
}

fn count_grok_disk(home: Option<&Path>, cutoff: SystemTime) -> DiskCounts {
    let Some(grok_home) = resolve_grok_home(home) else {
        return DiskCounts::default();
    };
    let sessions = grok_home.join("sessions");
    let mut counts = DiskCounts::default();
    walk_grok_chat_history(&sessions, cutoff, &mut counts);
    counts
}

fn walk_grok_chat_history(dir: &Path, cutoff: SystemTime, counts: &mut DiskCounts) {
    let Some(rd) = read_existing_dir(dir, counts) else {
        return;
    };
    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "coverage dirent unreadable");
                counts.unreadable = true;
                continue;
            }
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".lock") {
            continue;
        }
        if dirent_is_dir(&entry, counts) {
            walk_grok_chat_history(&path, cutoff, counts);
            continue;
        }
        if !name.eq_ignore_ascii_case("chat_history.jsonl") {
            continue;
        }
        if !mtime_in_window(&path, cutoff, counts) {
            continue;
        }
        if is_subagent_session(&path, None) {
            counts.sidechain += 1;
        } else {
            counts.eligible += 1;
        }
    }
}

fn count_claude_disk(home: Option<&Path>, cutoff: SystemTime) -> DiskCounts {
    let Some(claude_home) = resolve_claude_home(home) else {
        return DiskCounts::default();
    };
    let projects = claude_home.join("projects");
    let mut counts = DiskCounts::default();
    walk_claude_jsonl(&projects, cutoff, &mut counts);
    counts
}

fn walk_claude_jsonl(dir: &Path, cutoff: SystemTime, counts: &mut DiskCounts) {
    let Some(rd) = read_existing_dir(dir, counts) else {
        return;
    };
    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "coverage dirent unreadable");
                counts.unreadable = true;
                continue;
            }
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if dirent_is_dir(&entry, counts) {
            walk_claude_jsonl(&path, cutoff, counts);
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".jsonl") {
            continue;
        }
        if !mtime_in_window(&path, cutoff, counts) {
            continue;
        }
        if is_claude_sidechain_path(&path) {
            counts.sidechain += 1;
        } else {
            counts.eligible += 1;
        }
    }
}

fn count_codex_disk(home: Option<&Path>, cutoff: SystemTime) -> DiskCounts {
    let Some(codex_home) = resolve_codex_home(home) else {
        return DiskCounts::default();
    };
    let sessions = codex_home.join("sessions");
    let mut counts = DiskCounts::default();
    walk_codex_rollouts(&sessions, cutoff, &mut counts);
    counts
}

/// Filename filter matching `discover_codex_sessions` (`rollout-*.jsonl`) without
/// opening JSONL bodies (F6 / F12).
fn walk_codex_rollouts(dir: &Path, cutoff: SystemTime, counts: &mut DiskCounts) {
    let Some(rd) = read_existing_dir(dir, counts) else {
        return;
    };
    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "coverage dirent unreadable");
                counts.unreadable = true;
                continue;
            }
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if dirent_is_dir(&entry, counts) {
            walk_codex_rollouts(&path, cutoff, counts);
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if !(lower.starts_with("rollout-") && lower.ends_with(".jsonl")) {
            continue;
        }
        if mtime_in_window(&path, cutoff, counts) {
            counts.eligible += 1;
        }
    }
}

fn count_cursor_disk(home: Option<&Path>, cutoff: SystemTime) -> DiskCounts {
    let Some(cursor_home) = resolve_cursor_home(home) else {
        return DiskCounts::default();
    };
    let sources = match discover_cursor_sessions(&cursor_home) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "cursor discover unreadable");
            return DiskCounts {
                eligible: 0,
                sidechain: 0,
                unreadable: true,
            };
        }
    };
    let mut counts = DiskCounts::default();
    for src in sources {
        if !mtime_in_window(&src.path, cutoff, &mut counts) {
            continue;
        }
        if is_cursor_sidechain_path(&src.path) {
            counts.sidechain += 1;
        } else {
            counts.eligible += 1;
        }
    }
    counts
}

fn cursor_unbound_folders(
    cursor_home: &Path,
    aliases: &[(ProjectId, String)],
) -> (Vec<String>, bool) {
    let slugs: Vec<String> = aliases
        .iter()
        .flat_map(|(_, path)| cursor_project_slug_candidates(path))
        .collect();
    let projects = cursor_home.join("projects");
    let rd = match std::fs::read_dir(&projects) {
        Ok(r) => r,
        Err(e) if is_absent_path(&e) => return (Vec::new(), false),
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %projects.display(),
                "coverage cursor projects unreadable"
            );
            return (Vec::new(), true);
        }
    };
    let mut unbound = Vec::new();
    let mut counts = DiskCounts::default();
    for entry in rd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "coverage dirent unreadable");
                counts.unreadable = true;
                continue;
            }
        };
        if !dirent_is_dir(&entry, &mut counts) {
            continue;
        }
        let folder = entry.file_name().to_string_lossy().to_string();
        if folder.eq_ignore_ascii_case("subagents") {
            continue;
        }
        let matched = slugs.iter().any(|s| s.eq_ignore_ascii_case(&folder));
        if !matched {
            unbound.push(folder);
        }
    }
    unbound.sort();
    (unbound, counts.unreadable)
}

fn print_human(report: &CoverageReport) {
    println!("Capture coverage (last {} days)", report.days);
    println!(
        "{:<10} {:<12} {:>6} {:>10} {:>6} {:<24} Next step",
        "Source", "Mode", "Disk", "Sidechain", "Vault", "Status"
    );
    for src in &report.sources {
        let disk = match src.disk_eligible {
            None => "—".to_string(),
            Some(n) => n.to_string(),
        };
        let next = if src.next_step.is_empty() {
            "—"
        } else {
            src.next_step.as_str()
        };
        println!(
            "{:<10} {:<12} {:>6} {:>10} {:>6} {:<24} {}",
            src.source,
            src.mode,
            disk,
            src.disk_skipped_sidechain,
            src.vault_sessions,
            src.status,
            next
        );
    }
    if !report.unbound_folders.is_empty() {
        println!("Unbound Cursor folders:");
        for folder in &report.unbound_folders {
            println!("  {folder}");
        }
    }
    if !report.warnings.is_empty() {
        println!("Warnings: {}", report.warnings.join(", "));
    }
    if report.multi_import.is_none() && !report.warnings.iter().any(|w| w == "unreadable") {
        println!("multi_import: —");
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::commands::multi_import::LAST_MULTI_IMPORT_KEY;
    use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::{DataKey, SqlCipherKey};
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::{
        Actor, AggregateType, Payload, ProjectRegisteredPayload, RepositoryPathAliasAddedPayload,
        SessionStartedPayload,
    };
    use ai_brains_store::{EventStore, SqliteEventStore, VaultConnection};
    use std::fs;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::TempDir;

    const CURSOR_SID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa01";
    const PROJECT_PATH: &str = r"C:\dev\ai-brains";

    fn open_store() -> (TempDir, SqliteEventStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("v.db");
        let key = DataKey::generate();
        let sql_key = SqlCipherKey::from_data_key(&key);
        let conn = VaultConnection::open(&db, &sql_key).expect("open");
        conn.migrate().expect("migrate");
        (dir, SqliteEventStore::new(conn))
    }

    fn register_project(store: &SqliteEventStore, project_id: ProjectId) {
        store
            .append_event(
                &EventBuilder::new(
                    AggregateType::Project,
                    project_id.as_uuid(),
                    Actor::System,
                    Privacy::LocalOnly,
                )
                .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                    project_id,
                    name: "t337".into(),
                    tx_id: None,
                }))
                .expect("project"),
            )
            .expect("append project");
    }

    fn add_path_alias(store: &SqliteEventStore, project_id: ProjectId, normalized_path: &str) {
        store
            .append_event(
                &EventBuilder::new(
                    AggregateType::Project,
                    project_id.as_uuid(),
                    Actor::System,
                    Privacy::LocalOnly,
                )
                .build(Payload::RepositoryPathAliasAdded(
                    RepositoryPathAliasAddedPayload {
                        project_id,
                        normalized_path: normalized_path.to_string(),
                    },
                ))
                .expect("alias"),
            )
            .expect("append alias");
    }

    fn start_harness_session(store: &SqliteEventStore, project_id: ProjectId, harness_uuid: &str) {
        let session_id = SessionId::new();
        let harness = HarnessId::from_str(harness_uuid).expect("harness");
        store
            .append_event(
                &EventBuilder::new(
                    AggregateType::Session,
                    session_id.as_uuid(),
                    Actor::Harness(harness),
                    Privacy::LocalOnly,
                )
                .build(Payload::SessionStarted(SessionStartedPayload {
                    session_id,
                    project_id,
                    tx_id: None,
                }))
                .expect("session"),
            )
            .expect("append session");
    }

    fn write_file(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        let mut f = fs::File::create(path).expect("create");
        f.write_all(body.as_bytes()).expect("write");
    }

    fn set_old_mtime(path: &Path, days_ago: u64) {
        let f = fs::File::options().write(true).open(path).expect("open");
        let ts = SystemTime::now()
            .checked_sub(Duration::from_secs(days_ago.saturating_mul(86_400)))
            .unwrap_or(SystemTime::UNIX_EPOCH);
        f.set_modified(ts).expect("mtime");
    }

    fn coverage_opts(home: &Path, project_id: ProjectId, days: usize) -> CoverageOptions {
        CoverageOptions {
            days,
            format: "json".to_string(),
            global: false,
            project_id: Some(project_id),
            home_override: Some(home.to_path_buf()),
        }
    }

    fn source<'a>(report: &'a CoverageReport, name: &str) -> &'a SourceCoverage {
        report
            .sources
            .iter()
            .find(|s| s.source == name)
            .unwrap_or_else(|| panic!("missing source {name}"))
    }

    fn cursor_parent_jsonl(home: &Path, folder: &str) -> PathBuf {
        home.join(".cursor")
            .join("projects")
            .join(folder)
            .join("agent-transcripts")
            .join(CURSOR_SID)
            .join(format!("{CURSOR_SID}.jsonl"))
    }

    #[test]
    fn capture_coverage__cursor_parent_jsonl_empty_vault__deficit_next_step() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        write_file(&cursor_parent_jsonl(home.path(), "c-dev-x"), "{}\n");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let cursor = source(&report, "cursor");
        assert_eq!(cursor.mode, "import_only");
        assert!(cursor.disk_eligible.unwrap_or(0) >= 1);
        assert_eq!(cursor.status, "deficit");
        assert!(
            cursor.next_step.contains("cursor-import"),
            "next_step={}",
            cursor.next_step
        );
        assert!(!cursor.next_step.contains("--force"));
    }

    #[test]
    fn capture_coverage__subagents_dir__counted_sidechain_not_eligible() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        let side = home
            .path()
            .join(".cursor")
            .join("projects")
            .join("c-dev-x")
            .join("agent-transcripts")
            .join("subagents")
            .join("foo.jsonl");
        write_file(&side, "{}\n");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let cursor = source(&report, "cursor");
        assert!(
            cursor.disk_skipped_sidechain >= 1,
            "sidechain={}",
            cursor.disk_skipped_sidechain
        );
        assert_eq!(cursor.disk_eligible, Some(0));
        assert_ne!(cursor.status, "deficit");
        assert_eq!(cursor.status, "expected_skip");
    }

    #[test]
    fn capture_coverage__grok_subagent_path__expected_skip() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        write_file(
            &home
                .path()
                .join(".grok")
                .join("sessions")
                .join("subagent-role")
                .join("sid")
                .join("chat_history.jsonl"),
            "{}\n",
        );

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let grok = source(&report, "grok");
        assert_eq!(grok.status, "expected_skip");
        assert!(grok.disk_skipped_sidechain >= 1);
        assert_eq!(grok.disk_eligible, Some(0));
        assert!(
            !grok.next_step.contains("--force"),
            "next_step={}",
            grok.next_step
        );
    }

    #[test]
    fn capture_coverage__grok_non_path_eligible_empty_vault__unverifiable_subagent() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        write_file(
            &home
                .path()
                .join(".grok")
                .join("sessions")
                .join("C%3A")
                .join("sid")
                .join("chat_history.jsonl"),
            "{}\n",
        );

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let grok = source(&report, "grok");
        assert_eq!(grok.status, "unverifiable_subagent");
        assert!(grok.disk_eligible.unwrap_or(0) >= 1);
        assert!(
            grok.next_step.contains("grok-import"),
            "next_step={}",
            grok.next_step
        );
        assert!(
            grok.next_step.contains("--dry-run"),
            "next_step={}",
            grok.next_step
        );
        assert!(!grok.next_step.contains("--force"));
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w == "grok_batch_empty_all_subagent"),
            "warnings={:?}",
            report.warnings
        );
    }

    #[test]
    fn capture_coverage__session_started_cursor_actor__vault_sessions() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        start_harness_session(&store, project_id, CURSOR_HARNESS_UUID);
        write_file(&cursor_parent_jsonl(home.path(), "c-dev-x"), "{}\n");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let cursor = source(&report, "cursor");
        assert!(cursor.vault_sessions >= 1);
        assert_ne!(cursor.status, "deficit");
    }

    #[test]
    fn capture_coverage__alias_mixed_case_slug__not_unbound() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        add_path_alias(&store, project_id, PROJECT_PATH);
        fs::create_dir_all(
            home.path()
                .join(".cursor")
                .join("projects")
                .join("c-dev-AI-Brains"),
        )
        .expect("mkdir");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        assert!(
            !report
                .unbound_folders
                .iter()
                .any(|f| f.eq_ignore_ascii_case("c-dev-AI-Brains")),
            "unbound={:?}",
            report.unbound_folders
        );
    }

    #[test]
    fn capture_coverage__wsl_folder_windows_alias__not_unbound() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        add_path_alias(&store, project_id, PROJECT_PATH);
        fs::create_dir_all(
            home.path()
                .join(".cursor")
                .join("projects")
                .join("mnt-c-dev-AI-Brains"),
        )
        .expect("mkdir");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        assert!(
            !report
                .unbound_folders
                .iter()
                .any(|f| f.eq_ignore_ascii_case("mnt-c-dev-AI-Brains")),
            "unbound={:?}",
            report.unbound_folders
        );
    }

    #[test]
    fn capture_coverage__empty_window_folder__unbound_listed() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        fs::create_dir_all(
            home.path()
                .join(".cursor")
                .join("projects")
                .join("empty-window"),
        )
        .expect("mkdir");
        fs::create_dir_all(
            home.path()
                .join(".cursor")
                .join("projects")
                .join("zzz-later"),
        )
        .expect("mkdir");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        assert!(
            report.unbound_folders.iter().any(|f| f == "empty-window"),
            "unbound={:?}",
            report.unbound_folders
        );
        let mut sorted = report.unbound_folders.clone();
        sorted.sort();
        assert_eq!(report.unbound_folders, sorted);
    }

    #[test]
    fn capture_coverage__three_source_multi_import__stale_warning() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        let three_source = r#"{
            "v": 1,
            "at": "2026-08-31T07:00:24Z",
            "agy": {"status": "ok", "sessions": 1, "imported_turns": 1, "unbound": 0},
            "grok": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
            "opencode": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0}
        }"#;
        store
            .set_sync_state(LAST_MULTI_IMPORT_KEY, three_source)
            .expect("sync");

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        assert!(
            report.warnings.iter().any(|w| w == "stale_multi_import"),
            "warnings={:?}",
            report.warnings
        );
        let mi = report.multi_import.as_ref().expect("parsed multi_import");
        assert_eq!(mi.claude.skip_reason.as_deref(), Some("absent_pre_t334"));
        assert_eq!(mi.claude.status, "skipped");

        store
            .set_sync_state(
                LAST_MULTI_IMPORT_KEY,
                r#"{
                    "v": 1,
                    "at": "2026-08-31T07:00:24Z",
                    "agy": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
                    "grok": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
                    "opencode": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
                    "claude": {"status": "skipped", "skip_reason": "absent_pre_t334"},
                    "codex": {"status": "skipped", "skip_reason": "absent_pre_t334"},
                    "cursor": {"status": "skipped", "skip_reason": "absent_pre_t334"}
                }"#,
            )
            .expect("sync2");
        let report2 = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report2");
        assert!(
            report2.warnings.iter().any(|w| w == "stale_multi_import"),
            "explicit skip_reason must warn"
        );
    }

    #[test]
    fn capture_coverage__opencode_missing_binary__next_step_env() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        store
            .set_sync_state(
                LAST_MULTI_IMPORT_KEY,
                r#"{
                    "v": 1,
                    "at": "2026-09-01T07:00:22Z",
                    "agy": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
                    "grok": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0},
                    "opencode": {"status": "ok", "sessions": 0, "imported_turns": 0, "unbound": 0, "skipped_missing_binary": 1},
                    "claude": {"status": "skipped", "skip_reason": "absent_pre_t334"},
                    "codex": {"status": "skipped", "skip_reason": "absent_pre_t334"},
                    "cursor": {"status": "skipped", "skip_reason": "absent_pre_t334"}
                }"#,
            )
            .expect("sync");
        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let oc = source(&report, "opencode");
        assert_eq!(oc.status, "expected_skip");
        assert!(
            oc.next_step.contains("AI_BRAINS_OPENCODE_BIN"),
            "next_step={}",
            oc.next_step
        );
        assert_eq!(oc.disk_eligible, None);
        assert!(oc.next_step.chars().count() <= 140);
    }

    #[test]
    fn capture_coverage__json__six_sources_sorted() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        let names: Vec<&str> = report.sources.iter().map(|s| s.source.as_str()).collect();
        assert_eq!(
            names,
            vec!["agy", "grok", "opencode", "claude", "codex", "cursor"]
        );
        assert!(report.multi_import.is_none());
        let oc = source(&report, "opencode");
        assert_eq!(oc.disk_eligible, None);
        assert_eq!(oc.disk_note.as_deref(), Some("requires_opencode_bin"));
        let json = serde_json::to_value(&report).expect("json");
        assert!(json["multi_import"].is_null());
        assert!(json["opencode"].is_null() || json["sources"][2]["disk_eligible"].is_null());
        assert!(json["unbound_folders"].is_array());
        assert!(json["warnings"].is_array());
    }

    #[test]
    fn capture_coverage__days_2__old_mtime_excluded() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        let path = cursor_parent_jsonl(home.path(), "c-dev-x");
        write_file(&path, "{}\n");
        set_old_mtime(&path, 10);

        let report_30 = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("30");
        assert!(
            source(&report_30, "cursor").disk_eligible.unwrap_or(0) >= 1,
            "default 30d includes 10-day-old file"
        );

        let report_2 = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 2),
        )
        .expect("2");
        assert_eq!(
            source(&report_2, "cursor").disk_eligible,
            Some(0),
            "days=2 excludes 10-day-old file"
        );
    }

    #[test]
    fn capture_coverage__agy_both_harness_uuids__sum() {
        let home = tempfile::tempdir().expect("home");
        let (_vdir, store) = open_store();
        let project_id = ProjectId::new();
        register_project(&store, project_id);
        start_harness_session(&store, project_id, AGY_IMPORT_UUID);
        start_harness_session(&store, project_id, AGY_HOOK_UUID);

        let report = build_report(
            store.connection(),
            &coverage_opts(home.path(), project_id, 30),
        )
        .expect("report");
        assert_eq!(source(&report, "agy").vault_sessions, 2);
    }
}
