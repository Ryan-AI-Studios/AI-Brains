use crate::commands::identity_warn::print_json_stdout;
use crate::context::AppContext;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload, ProjectAliasAddedPayload};
use ai_brains_store::{EventStore, ProjectListDetail, QueryStore};
use serde::Serialize;
use std::process::Command;

/// Row from `list_projects`: (project_id, name, alias, memory_count).
type ProjectRow = (String, String, String, usize);

/// Outcome of matching a git repo slug against vault projects (exact-first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlugMatch {
    Unique(ProjectRow),
    Ambiguous(Vec<ProjectRow>),
    None,
}

/// Label column width (chars) for human table (F14).
const LABEL_COL_CHARS: usize = 30;
/// Path column width (chars) for human table (F14).
const PATH_COL_CHARS: usize = 40;

/// T212: list projects label-first (human table or JSON).
pub fn list(ctx: &AppContext, format: &str, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let projects = ctx.conn.list_projects_detail()?;
    let active_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .filter(|s| !s.is_empty());

    if format.eq_ignore_ascii_case("json") {
        return list_json(&projects, active_id.as_deref());
    }

    // Human default (F5/F15).
    println!(
        "{:<30} {:<36} {:>8} {:<12} path",
        "label", "project_id", "memories", "last_activity"
    );
    if projects.is_empty() {
        println!("No projects registered. (0 projects)");
        return Ok(());
    }

    // F39: cwd owner from resolve_path_alias_for_location (`:237–248`).
    let cwd_owner = match std::env::current_dir() {
        Ok(cwd) => {
            let git = collect_git_identity(&cwd).unwrap_or_default();
            resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)?
        }
        Err(_) => None,
    };
    let display =
        crate::commands::project_list_order::promote_cwd_owner(&projects, cwd_owner.as_deref());
    let (display, more) = crate::commands::project_list_order::filter_human_list_rows(
        &display,
        cwd_owner.as_deref(),
        all,
    );

    for row in &display {
        let label = display_label(&row.name, &row.alias, &row.project_id);
        let starred = match active_id.as_deref() {
            Some(id) if id == row.project_id => format!("*{}", label),
            _ => label,
        };
        let label_disp = truncate_chars(&starred, LABEL_COL_CHARS);
        let activity = format_last_activity(&row.last_activity);
        let path_disp = match row.path.as_deref() {
            Some(p) if !p.is_empty() => truncate_chars(p, PATH_COL_CHARS),
            _ => "—".to_string(),
        };
        println!(
            "{:<30} {:<36} {:>8} {:<12} {}",
            label_disp, row.project_id, row.memory_count, activity, path_disp
        );
    }
    if more > 0 {
        println!("+{more} more (ai-brains project list --all)");
    }

    // F8: no-alias footer on stderr (data stays on stdout).
    crate::commands::project_list_footer::print_unaliased_footer(ctx, &projects)?;
    Ok(())
}

fn list_json(
    projects: &[ProjectListDetail],
    active_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let unaliased_count = projects.iter().filter(|p| p.alias.is_empty()).count();
    let items: Vec<ProjectListJsonRow> = projects
        .iter()
        .map(|row| {
            let label = display_label(&row.name, &row.alias, &row.project_id);
            let active = active_id.map(|id| id == row.project_id).unwrap_or(false);
            ProjectListJsonRow {
                project_id: row.project_id.clone(),
                name: row.name.clone(),
                alias: row.alias.clone(),
                label,
                memory_count: row.memory_count,
                last_activity: if row.last_activity.is_empty() {
                    None
                } else {
                    Some(row.last_activity.clone())
                },
                path: row.path.clone(),
                active: if active { Some(true) } else { None },
            }
        })
        .collect();

    let envelope = ProjectListJson {
        api_version: "1".to_string(),
        projects: items,
        unaliased_count,
    };
    print_json_stdout(&envelope)
}

// ---------------------------------------------------------------------------
// T240 — Project identity helpers (GitIdentity, path alias, detect order)
// ---------------------------------------------------------------------------

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Git identity for path-alias + slug detect (T240 M1).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct GitIdentity {
    pub slug: Option<String>,
    pub toplevel: Option<PathBuf>,
}

/// Detect signal source (export comments / whoami).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectSource {
    PathAlias,
    GitSlug,
    Env,
}

impl DetectSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PathAlias => "path_alias",
            Self::GitSlug => "git_slug",
            Self::Env => "env",
        }
    }
}

/// Outcome of path-first detect (F5/F6) without process exit.
#[derive(Debug, Clone)]
pub(crate) struct DetectOutcome {
    pub project: ProjectRow,
    pub source: DetectSource,
    /// Stderr notes (path vs slug conflict, 0-mem extra).
    pub notes: Vec<String>,
    /// Env-fallback git/env mismatch warning (F4/F35).
    pub env_warn: Option<String>,
}

/// Pre-dotenv shell `AI_BRAINS_PROJECT_ID` (T240 L9 / whoami).
static SHELL_PROJECT_ID: OnceLock<Option<String>> = OnceLock::new();

#[cfg(test)]
pub(crate) use crate::commands::identity_warn::{
    identity_mismatch_warn_line, should_skip_identity_mismatch_warn,
};

/// Record shell PROJECT_ID before `apply_local_project_context_env` force-set.
pub fn record_shell_project_id(id: Option<String>) {
    let _ = SHELL_PROJECT_ID.set(id);
}

/// Pre-dotenv shell project id when captured.
pub fn shell_project_id_captured() -> Option<String> {
    SHELL_PROJECT_ID.get().and_then(|o| o.clone())
}

/// F5b/M1: single `rev-parse --show-toplevel` + remote extract; keep toplevel.
pub(crate) fn collect_git_identity(path: &Path) -> Result<GitIdentity, Box<dyn std::error::Error>> {
    let output = git_command()
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Ok(GitIdentity::default());
    }

    let toplevel_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if toplevel_str.is_empty() {
        return Ok(GitIdentity::default());
    }
    let toplevel_path = PathBuf::from(&toplevel_str);

    // Prefer origin remote repo name; fall back to toplevel dir name.
    let remote = git_command()
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()?;

    let slug = if remote.status.success() {
        let url = String::from_utf8_lossy(&remote.stdout).trim().to_owned();
        extract_repo_name(&url).filter(|s| !s.is_empty())
    } else {
        None
    };

    let slug = match slug {
        Some(s) => Some(s),
        None => toplevel_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .filter(|s| !s.is_empty()),
    };

    Ok(GitIdentity {
        slug,
        toplevel: Some(toplevel_path),
    })
}

/// Normalize + `find_path_alias_owner` for a filesystem path.
pub(crate) fn resolve_path_alias_project(
    store: &dyn QueryStore,
    path: &Path,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let raw = path.to_string_lossy();
    let normalized = ai_brains_path::normalize_for_location_compare(&raw);
    if normalized.is_empty() {
        return Ok(None);
    }
    Ok(store
        .find_path_alias_owner(&normalized)?
        .map(|pid| pid.to_string()))
}

/// Path alias of git toplevel else cwd (F5 step 1 / AC9).
pub(crate) fn resolve_path_alias_for_location(
    store: &dyn QueryStore,
    cwd: &Path,
    git: &GitIdentity,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(ref toplevel) = git.toplevel
        && let Some(owner) = resolve_path_alias_project(store, toplevel)?
    {
        return Ok(Some(owner));
    }
    resolve_path_alias_project(store, cwd)
}

/// F6: notes when path owner B is preferred over unique slug hit A.
///
/// Always notes A. Extra note when B.mem==0 && A.mem>0.
pub(crate) fn path_vs_slug_conflict_notes(
    path_row: &ProjectRow,
    slug_row: &ProjectRow,
) -> Vec<String> {
    let mut notes = Vec::new();
    let alias_disp = if slug_row.2.is_empty() {
        "(none)"
    } else {
        slug_row.2.as_str()
    };
    notes.push(format!(
        "Note: git slug also matches project {} (alias={}); preferring path alias owner.",
        slug_row.0, alias_disp
    ));
    if path_row.3 == 0 && slug_row.3 > 0 {
        notes.push(
            "Note: path-alias owner has 0 memories while git slug match has memories; verify path alias via `project list`."
                .to_string(),
        );
    }
    notes
}

fn project_row_by_id(projects: &[ProjectRow], id: &str) -> Option<ProjectRow> {
    projects.iter().find(|(p, _, _, _)| p == id).cloned()
}

/// Path-first detect resolution (F5/F6/F7) — no process exit.
pub(crate) fn resolve_detect(
    store: &dyn QueryStore,
    cwd: &Path,
) -> Result<Option<DetectOutcome>, Box<dyn std::error::Error>> {
    let git = collect_git_identity(cwd)?;
    let projects = store.list_projects()?;

    // F5 (1): path alias of toplevel else cwd.
    let path_owner = resolve_path_alias_for_location(store, cwd, &git)?;
    if let Some(ref path_id) = path_owner
        && let Some(path_row) = project_row_by_id(&projects, path_id)
    {
        let mut notes = Vec::new();
        // F6: if unique slug hit differs, note slug project A (path always wins).
        if let Some(ref slug) = git.slug {
            match match_projects_for_slug(&projects, slug) {
                SlugMatch::Unique(slug_row) if slug_row.0 != path_row.0 => {
                    notes.extend(path_vs_slug_conflict_notes(&path_row, &slug_row));
                }
                // Path present: do not fail-closed on ambiguous slug (F7 when no path).
                _ => {}
            }
        }
        return Ok(Some(DetectOutcome {
            project: path_row,
            source: DetectSource::PathAlias,
            notes,
            env_warn: None,
        }));
    }

    // F5 (2): git slug exact-first (T206) when no path owner.
    if let Some(ref slug) = git.slug {
        match match_projects_for_slug(&projects, slug) {
            // Unique git match wins over wrong env (AC1).
            SlugMatch::Unique(row) => {
                return Ok(Some(DetectOutcome {
                    project: row,
                    source: DetectSource::GitSlug,
                    notes: Vec::new(),
                    env_warn: None,
                }));
            }
            // F5/F18/AC4: ambiguous ≥2 → signal via special notes + caller exits 1.
            SlugMatch::Ambiguous(matched) => {
                // Encode ambiguity as empty project id sentinel for caller.
                return Ok(Some(DetectOutcome {
                    project: (String::new(), String::new(), String::new(), matched.len()),
                    source: DetectSource::GitSlug,
                    notes: ambiguous_slug_notes(slug, &matched),
                    env_warn: None,
                }));
            }
            SlugMatch::None => {}
        }
    }

    // F5 (3): process AI_BRAINS_PROJECT_ID if in vault.
    if let Ok(pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && !pid_str.is_empty()
        && let Some(row) = project_row_by_id(&projects, &pid_str)
    {
        let warn = env_fallback_warning(git.slug.as_deref().unwrap_or(""), &row.0, &row.1, &row.2);
        return Ok(Some(DetectOutcome {
            project: row,
            source: DetectSource::Env,
            notes: Vec::new(),
            env_warn: warn,
        }));
    }

    // F5 (4): miss.
    Ok(None)
}

fn ambiguous_slug_notes(slug: &str, matched: &[ProjectRow]) -> Vec<String> {
    let mut notes = vec![format!(
        "Ambiguous match for '{slug}' — multiple candidates found in vault:"
    )];
    for (pid, name, alias, count) in matched {
        notes.push(format!("  {pid} | {name} | {alias} | {count} memories"));
    }
    notes
}

/// True when resolve_detect encoded slug ambiguity (empty project id).
pub(crate) fn is_ambiguous_detect(outcome: &DetectOutcome) -> bool {
    outcome.source == DetectSource::GitSlug && outcome.project.0.is_empty()
}

pub(crate) fn sanitize_alias_suggestion(slug: &str) -> String {
    // Keep simple slug-like characters; fall back empty → caller uses my-project.
    let s: String = slug
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c == ' ' {
                '-'
            } else {
                '_'
            }
        })
        .collect();
    s.trim_matches(|c| c == '-' || c == '_').to_string()
}

// ---------------------------------------------------------------------------
// Pure display helpers (F4 / F7 / F36) — unit-tested without vault spawn
// ---------------------------------------------------------------------------

/// F4 / T230: human label order — alias → baked `(no alias)` → empty/ws name →
/// Project uuid / id → name. Never returns empty string (orphan ids use empty name).
pub(crate) fn display_label(name: &str, alias: &str, project_id: &str) -> String {
    if !alias.is_empty() {
        return alias.to_string();
    }
    // Baked UX form: "(no alias) — short" → literal "(no alias)".
    if name.starts_with("(no alias)") {
        return "(no alias)".to_string();
    }
    // T230 F32: empty / whitespace-only name → (no alias). Do not trim alias (F5/F34).
    if name.trim().is_empty() {
        return "(no alias)".to_string();
    }
    if is_non_human_project_name(name, project_id) {
        return "(no alias)".to_string();
    }
    name.to_string()
}

/// True when name is a non-human machine form: `Project <uuid-ish>` or equals
/// full/short project_id (F4 step 3). Manual string ops only — no regex (F42).
fn is_non_human_project_name(name: &str, project_id: &str) -> bool {
    if name == project_id {
        return true;
    }
    let short = project_id_short(project_id);
    if !short.is_empty() && name == short {
        return true;
    }
    if let Some(rest) = name.strip_prefix("Project ") {
        return is_uuid_ish(rest.trim());
    }
    false
}

fn project_id_short(project_id: &str) -> &str {
    // Project IDs are UUID strings (ASCII). Prefer 8-char prefix; no mid-UTF-8 risk.
    if project_id.len() >= 8 {
        &project_id[..8]
    } else {
        project_id
    }
}

/// UUID-ish: hex + hyphens, length 8..=36 (covers short prefixes and full UUIDs).
fn is_uuid_ish(s: &str) -> bool {
    let len = s.len();
    if !(8..=36).contains(&len) {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// F36: char-safe truncate; appends `…` when shortened (F14).
pub(crate) fn truncate_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    // Reserve one char for ellipsis when possible.
    let keep = max_chars.saturating_sub(1);
    let truncated: String = s.chars().take(keep).collect();
    format!("{truncated}…")
}

/// F7: relative when age < 365d (`just now` / `Nm` / `Nh` / `Nd`); else `YYYY-MM-DD`.
pub(crate) fn format_last_activity(raw: &str) -> String {
    if raw.is_empty() {
        return "—".to_string();
    }
    let updated = match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => {
            // Try common SQLite / ISO forms without offset.
            match chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S") {
                Ok(ndt) => ndt.and_utc(),
                Err(_) => match chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S") {
                    Ok(ndt) => ndt.and_utc(),
                    Err(_) => {
                        // If it already looks like a date, show as-is truncated.
                        if raw.len() >= 10 {
                            return raw.chars().take(10).collect();
                        }
                        return "—".to_string();
                    }
                },
            }
        }
    };
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(updated);
    // Future or clock skew → show absolute date.
    if duration.num_seconds() < 0 {
        return updated.format("%Y-%m-%d").to_string();
    }
    if duration.num_days() >= 365 {
        return updated.format("%Y-%m-%d").to_string();
    }
    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h", duration.num_hours())
    } else {
        format!("{}d", duration.num_days())
    }
}

#[derive(Debug, Serialize)]
struct ProjectListJson {
    api_version: String,
    projects: Vec<ProjectListJsonRow>,
    unaliased_count: usize,
}

#[derive(Debug, Serialize)]
struct ProjectListJsonRow {
    project_id: String,
    name: String,
    alias: String,
    label: String,
    memory_count: usize,
    last_activity: Option<String>,
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
}

pub fn resolve(
    ctx: &AppContext,
    alias_positional: Option<String>,
    alias: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let alias = alias_positional.or(alias).ok_or(
        "No alias provided. Use `project resolve <alias>` or `project resolve --alias <alias>`.",
    )?;
    // First try exact alias match
    if let Some(pid) = ctx.conn.resolve_project_id_from_alias(&alias)? {
        println!("{}", pid);
        return Ok(());
    }

    // Fall back to fuzzy name match
    let projects = ctx.conn.list_projects()?;
    let lower_alias = alias.to_lowercase();
    let matched: Vec<_> = projects
        .into_iter()
        .filter(|(_, name, alias_name, _)| {
            name.to_lowercase().contains(&lower_alias)
                || alias_name.to_lowercase().contains(&lower_alias)
        })
        .collect();

    if matched.len() == 1 {
        println!("{}", matched[0].0);
        Ok(())
    } else if matched.len() > 1 {
        eprintln!("Ambiguous alias '{}' — did you mean one of these?", alias);
        for (pid, name, alias_name, count) in matched {
            eprintln!("  {} | {} | {} | {} memories", pid, name, alias_name, count);
        }
        std::process::exit(1);
    } else {
        eprintln!("No project found for alias '{}'", alias);
        std::process::exit(1);
    }
}

#[derive(Debug, Serialize)]
struct DetectReport {
    project_id: Option<String>,
    name: Option<String>,
    alias: Option<String>,
    memories: Option<i64>,
    source: String,
    notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

fn emit_detect_json(outcome: Option<&DetectOutcome>) -> Result<(), Box<dyn std::error::Error>> {
    let report = match outcome {
        None => DetectReport {
            project_id: None,
            name: None,
            alias: None,
            memories: None,
            source: crate::commands::identity_warn::detect_source_label(None).to_string(),
            notes: Vec::new(),
            warning: None,
            message: Some(
                "No project detected. Set an alias with 'project set-alias', initialize a project with 'init', or run 'ai-brains context'."
                    .to_string(),
            ),
        },
        Some(o) if is_ambiguous_detect(o) => DetectReport {
            project_id: None,
            name: None,
            alias: None,
            memories: None,
            source: crate::commands::identity_warn::detect_source_label(Some(o)).to_string(),
            notes: o.notes.clone(),
            warning: None,
            message: None,
        },
        Some(o) => {
            let (pid, name, alias, count) = &o.project;
            DetectReport {
                project_id: Some(pid.clone()),
                name: Some(name.clone()),
                alias: Some(alias.clone()),
                memories: Some(i64::try_from(*count).unwrap_or(i64::MAX)),
                source: crate::commands::identity_warn::detect_source_label(Some(o)).to_string(),
                notes: o.notes.clone(),
                warning: o.env_warn.clone(),
                message: None,
            }
        }
    };
    print_json_stdout(&report)
}

pub fn detect(
    ctx: &AppContext,
    export_shell: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::IsTerminal;

    let current_dir = std::env::current_dir()?;
    let outcome = resolve_detect(ctx.conn.as_ref(), &current_dir)?;

    if !export_shell
        && crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal())
    {
        emit_detect_json(outcome.as_ref())?;
        let fail = match outcome.as_ref() {
            None => true,
            Some(o) => is_ambiguous_detect(o),
        };
        if fail {
            std::process::exit(1);
        }
        return Ok(());
    }

    let Some(outcome) = outcome else {
        // F5 (4): Miss exit 1.
        let msg = "No project detected. Set an alias with 'project set-alias', initialize a project with 'init', or run 'ai-brains context'.";
        if export_shell {
            eprintln!("# {}", msg);
        } else {
            eprintln!("{}", msg);
        }
        std::process::exit(1);
    };

    // Ambiguous slug (no path): fail-closed exit 1 (T206 AC4).
    if is_ambiguous_detect(&outcome) {
        if export_shell {
            for (i, line) in outcome.notes.iter().enumerate() {
                if i == 0 {
                    eprintln!(
                        "# Ambiguous match — multiple candidates; set AI_BRAINS_PROJECT_ID manually"
                    );
                } else {
                    eprintln!("# {}", line.trim_start());
                }
            }
        } else {
            for line in &outcome.notes {
                eprintln!("{}", line);
            }
        }
        std::process::exit(1);
    }

    let (pid, name, alias, count) = &outcome.project;
    let source = outcome.source.as_str();

    // Conflict notes always on stderr (even with --export).
    for note in &outcome.notes {
        eprintln!("{}", note);
    }

    match outcome.source {
        DetectSource::PathAlias => {
            if export_shell {
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected: {} | alias={} | memories={} | from path_alias | source={}",
                    name, alias, count, source
                );
            } else {
                println!(
                    "Detected project from path alias: {} ({}) | alias={} | memories={}",
                    name, pid, alias, count
                );
            }
        }
        DetectSource::GitSlug => {
            if export_shell {
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected: {} | alias={} | memories={} | from git | source={}",
                    name, alias, count, source
                );
            } else {
                println!(
                    "Detected project from git: {} ({}) | alias={} | memories={}",
                    name, pid, alias, count
                );
            }
        }
        DetectSource::Env => {
            if export_shell {
                if let Some(ref w) = outcome.env_warn {
                    for line in w.lines() {
                        println!("# {}", line);
                    }
                }
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected from .env: {} | alias={} (from .env) | source={}",
                    name, alias, source
                );
            } else {
                if let Some(ref w) = outcome.env_warn {
                    eprintln!("{}", w);
                }
                println!(
                    "Detected project from .env: {} ({}) | alias={} (from .env)",
                    name, pid, alias
                );
            }
        }
    }
    Ok(())
}

/// T240 F4: show all identity signals (human on TTY; JSON when piped or `--format json`).
pub fn whoami(ctx: &AppContext, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::IsTerminal;

    let no_project_context = std::env::args().any(|a| a == "--no-project-context");
    let report = build_whoami_report(ctx, no_project_context)?;

    let use_json =
        crate::commands::format_resolve::is_json_output(format, std::io::stdout().is_terminal());

    if use_json {
        print_json_stdout(&report)?;
    } else {
        if let Some(ref warn) = report.slug_miss_warning {
            eprintln!("{warn}");
        }
        emit_whoami_human(&report);
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct WhoamiReport {
    effective_project_id: Option<String>,
    env_project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shell_project_id: Option<String>,
    path_alias_project_id: Option<String>,
    detect_project_id: Option<String>,
    detect_source: String,
    git_slug: Option<String>,
    git_toplevel: Option<String>,
    mismatch: bool,
    identity_collision: bool,
    remediations: Vec<String>,
    /// Human-only stderr SOOT (T349 F6). Never serialized.
    #[serde(skip)]
    slug_miss_warning: Option<String>,
}

fn build_whoami_report(
    ctx: &AppContext,
    no_project_context: bool,
) -> Result<WhoamiReport, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let git = collect_git_identity(&cwd)?;

    // env post-dotenv (null under --no-project-context per F17).
    let env_project_id = if no_project_context {
        None
    } else {
        std::env::var("AI_BRAINS_PROJECT_ID")
            .ok()
            .filter(|s| !s.is_empty())
    };

    // Daily Scope = effective env after dotenv (F1); null when --no-project-context.
    let effective_project_id = env_project_id.clone();

    // shell pre-dotenv when set and differs from env (or env null).
    let shell_raw = shell_project_id_captured();
    let shell_project_id = match (shell_raw.as_ref(), env_project_id.as_ref()) {
        (Some(shell), Some(env)) if shell != env => Some(shell.clone()),
        (Some(shell), None) => Some(shell.clone()),
        _ => None,
    };

    let path_alias_project_id = resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)?;

    let detect_outcome = resolve_detect(ctx.conn.as_ref(), &cwd)?;
    let detect_project_id = detect_outcome
        .as_ref()
        .filter(|o| !is_ambiguous_detect(o))
        .map(|o| o.project.0.clone());
    let detect_source =
        crate::commands::identity_warn::detect_source_label(detect_outcome.as_ref()).to_string();

    let mismatch = match (env_project_id.as_deref(), path_alias_project_id.as_deref()) {
        (Some(e), Some(p)) => e != p,
        _ => false,
    };

    let identity_collision = crate::commands::identity_warn::identity_collision(
        env_project_id.as_deref(),
        path_alias_project_id.as_deref(),
        detect_project_id.as_deref(),
    );

    let mut remediations = Vec::new();
    if mismatch {
        remediations.push(
            "Daily Scope comes from .env / shell AI_BRAINS_PROJECT_ID (not auto-switched to path)."
                .to_string(),
        );
        remediations.push(
            "Run `ai-brains project adopt-path` (print-only) or `ai-brains project adopt-path --write-env --yes`."
                .to_string(),
        );
        if let Some(ref path_id) = path_alias_project_id {
            remediations.push(format!(
                "To bind daily Scope to the path owner, set AI_BRAINS_PROJECT_ID={path_id} in project .env."
            ));
        }
        remediations.push(
            "set-alias is a human label; register-path is the filesystem root (do not conflate)."
                .to_string(),
        );
    }
    if detect_outcome.as_ref().is_some_and(is_ambiguous_detect) {
        remediations.push(
            "Detect git slug is ambiguous — set AI_BRAINS_PROJECT_ID or register-path / set-alias uniquely."
                .to_string(),
        );
    }
    if detect_project_id.is_none() && path_alias_project_id.is_none() {
        remediations.push(
            "No detect hit — run `ai-brains context`, `project set-alias`, or `project register-path`."
                .to_string(),
        );
    }
    if identity_collision
        && !mismatch
        && let Some(ref detect_id) = detect_project_id
    {
        let path_display = git
            .toplevel
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| cwd.display().to_string());
        remediations.extend(
            crate::commands::identity_warn::identity_collision_remediations_path_absent(
                detect_id,
                &path_display,
            ),
        );
    }

    let env_name = detect_outcome
        .as_ref()
        .map(|o| o.project.1.as_str())
        .unwrap_or("");
    let env_alias = detect_outcome
        .as_ref()
        .map(|o| o.project.2.as_str())
        .unwrap_or("");
    let mut slug_miss_warning = None;
    if !identity_collision
        && crate::commands::identity_warn::should_emit_slug_miss_env_fallback(
            &detect_source,
            env_project_id.as_deref(),
            git.slug.as_deref(),
            env_name,
            env_alias,
        )
        && let Some(ref env_id) = env_project_id
        && let Some(ref slug) = git.slug
    {
        let path_display = git
            .toplevel
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| cwd.display().to_string());
        remediations.extend(
            crate::commands::identity_warn::slug_miss_env_fallback_remediations(
                env_id,
                slug,
                &path_display,
            ),
        );
        slug_miss_warning = env_fallback_warning(slug, env_id, env_name, env_alias);
    }

    Ok(WhoamiReport {
        effective_project_id,
        env_project_id,
        shell_project_id,
        path_alias_project_id,
        detect_project_id,
        detect_source,
        git_slug: git.slug,
        git_toplevel: git.toplevel.map(|p| p.display().to_string()),
        mismatch,
        identity_collision,
        remediations,
        slug_miss_warning,
    })
}

fn emit_whoami_human(report: &WhoamiReport) {
    fn fmt_opt(v: &Option<String>) -> &str {
        v.as_deref().unwrap_or("(none)")
    }
    println!(
        "effective_project_id:  {}",
        fmt_opt(&report.effective_project_id)
    );
    println!("env_project_id:        {}", fmt_opt(&report.env_project_id));
    if let Some(ref shell) = report.shell_project_id {
        println!("shell_project_id:      {}", shell);
    } else {
        println!("shell_project_id:      (none or same as env)");
    }
    println!(
        "path_alias_project_id: {}",
        fmt_opt(&report.path_alias_project_id)
    );
    println!(
        "detect_project_id:     {}",
        fmt_opt(&report.detect_project_id)
    );
    println!("detect_source:         {}", report.detect_source);
    println!("git_slug:              {}", fmt_opt(&report.git_slug));
    println!("git_toplevel:          {}", fmt_opt(&report.git_toplevel));
    println!("mismatch:              {}", report.mismatch);
    println!("identity_collision:    {}", report.identity_collision);
    if report.remediations.is_empty() {
        println!("remediations:          (none)");
    } else {
        println!("remediations:");
        for r in &report.remediations {
            println!("  - {}", r);
        }
    }
}

pub fn set_alias(
    ctx: &AppContext,
    project_id_str: &str,
    alias: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::str::FromStr;

    let project_id = ai_brains_core::ids::ProjectId::from_str(project_id_str)
        .map_err(|_| format!("Invalid project ID: '{}'", project_id_str))?;

    // Verify the project exists in the vault.
    let projects = ctx.conn.list_projects()?;
    if !projects.iter().any(|(pid, _, _, _)| pid == project_id_str) {
        return Err(format!("Project '{}' not found in vault.", project_id_str).into());
    }

    // Check for alias conflicts.
    if let Some(existing_pid) = ctx.conn.resolve_project_id_from_alias(alias)? {
        if existing_pid == project_id {
            println!(
                "Alias '{}' is already set for project {}.",
                alias, project_id_str
            );
            return Ok(());
        }
        eprintln!(
            "Alias '{}' is already assigned to project {}.",
            alias, existing_pid
        );
        std::process::exit(1);
    }

    // Append the ProjectAliasAdded event — projection will update the alias table.
    let event = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::User(ai_brains_core::ids::UserId::new()),
        ai_brains_core::privacy::Privacy::LocalOnly,
    )
    .build(Payload::ProjectAliasAdded(ProjectAliasAddedPayload {
        project_id,
        alias: alias.to_string(),
    }))?;

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    event_store.append_event(&event)?;

    println!("Alias '{}' set for project {}.", alias, project_id_str);
    Ok(())
}

/// Register a filesystem path alias for multi-root nightly bridge (T233).
///
/// Resolves `project_ref` as UUID or human alias, normalizes `path`, then
/// pre-checks ownership (F21) before appending `RepositoryPathAliasAdded` via
/// control-plane `register_path_alias`.
pub fn register_path(
    ctx: &AppContext,
    project_ref: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = resolve_project_ref(ctx, project_ref)?;

    let normalized = ai_brains_path::normalize_for_location_compare(path);
    if normalized.is_empty() {
        return Err("path normalized to empty; choose a non-empty filesystem path.".into());
    }

    // F21 conflict pre-check (CLI check-then-write). Projection refuse-steal
    // keeps a raced other-owner Added from moving the row (T254 F7).
    if let Some(existing) = ctx.conn.find_path_alias_owner(&normalized)? {
        if existing == project_id {
            println!(
                "Path alias '{}' is already registered to project {}.",
                normalized, project_id
            );
            return Ok(());
        }
        eprintln!(
            "path alias '{}' is already registered to project {}; choose a different path or run: ai-brains project unregister-path {}",
            normalized, existing, normalized
        );
        std::process::exit(1);
    }

    let event_store = ai_brains_store::SqliteEventStore::new((*ctx.conn).clone());
    let writer = ai_brains_control_plane::StoreEventWriter::new(event_store);
    ai_brains_control_plane::register_path_alias(&writer, path, project_id)
        .map_err(|e| format!("register path alias failed: {e}"))?;

    println!(
        "Path alias '{}' registered for project {}.",
        normalized, project_id
    );
    Ok(())
}

/// Resolve `project_ref` as UUID parse **or** human alias lookup.
fn resolve_project_ref(
    ctx: &AppContext,
    project_ref: &str,
) -> Result<ai_brains_core::ids::ProjectId, Box<dyn std::error::Error>> {
    use std::str::FromStr;

    if let Ok(pid) = ai_brains_core::ids::ProjectId::from_str(project_ref) {
        // Verify the project exists when the ref looks like a UUID.
        let projects = ctx.conn.list_projects()?;
        let id_str = pid.to_string();
        if projects.iter().any(|(p, _, _, _)| p == &id_str) {
            return Ok(pid);
        }
        return Err(format!("Project '{}' not found in vault.", project_ref).into());
    }

    // Alias lookup.
    if let Some(pid) = ctx.conn.resolve_project_id_from_alias(project_ref)? {
        return Ok(pid);
    }

    Err(format!(
        "Project '{}' not found (not a valid project UUID and not a known alias).",
        project_ref
    )
    .into())
}

// ---------------------------------------------------------------------------
// Pure helpers (F33) — unit-tested without vault spawn
// ---------------------------------------------------------------------------

/// Exact-first match of a git slug against vault projects (F3).
///
/// 1. Exact name/alias (case-insensitive).
/// 2. Contains only if zero exact and exactly one contains hit.
///
/// Ambiguous candidates are sorted by `project_id` (F21).
pub(crate) fn match_projects_for_slug(projects: &[ProjectRow], slug: &str) -> SlugMatch {
    let lower_slug = slug.to_lowercase();
    if lower_slug.is_empty() {
        return SlugMatch::None;
    }

    let exact: Vec<ProjectRow> = projects
        .iter()
        .filter(|(_, name, alias_name, _)| {
            name.to_lowercase() == lower_slug || alias_name.to_lowercase() == lower_slug
        })
        .cloned()
        .collect();

    if exact.len() == 1 {
        if let Some(row) = exact.into_iter().next() {
            return SlugMatch::Unique(row);
        }
        return SlugMatch::None;
    }
    if exact.len() > 1 {
        let mut rows = exact;
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        return SlugMatch::Ambiguous(rows);
    }

    // Zero exact — contains only when exactly one hit.
    let contains: Vec<ProjectRow> = projects
        .iter()
        .filter(|(_, name, alias_name, _)| {
            (!name.is_empty() && name.to_lowercase().contains(&lower_slug))
                || (!alias_name.is_empty() && alias_name.to_lowercase().contains(&lower_slug))
        })
        .cloned()
        .collect();

    match contains.len() {
        1 => {
            if let Some(row) = contains.into_iter().next() {
                SlugMatch::Unique(row)
            } else {
                SlugMatch::None
            }
        }
        0 => SlugMatch::None,
        _ => {
            let mut rows = contains;
            rows.sort_by(|a, b| a.0.cmp(&b.0));
            SlugMatch::Ambiguous(rows)
        }
    }
}

/// F4/F35: warning when env PROJECT_ID hits but git slug is known and does not
/// exact-match the project's name or alias. Empty slug → no warning.
pub(crate) fn env_fallback_warning(
    git_slug: &str,
    project_id: &str,
    name: &str,
    alias: &str,
) -> Option<String> {
    if git_slug.is_empty() {
        return None;
    }
    let lower = git_slug.to_lowercase();
    if name.to_lowercase() == lower || alias.to_lowercase() == lower {
        return None;
    }
    let alias_display = if alias.is_empty() { "(none)" } else { alias };
    Some(format!(
        "Warning: git/env project mismatch: AI_BRAINS_PROJECT_ID points to project {project_id} (alias={alias_display}) but git repo slug is '{git_slug}' which does not match this project's name/alias.\nHint: ai-brains project set-alias {project_id} {git_slug}"
    ))
}

/// Build a non-interactive `git` command (F7 / AC11).
///
/// Every detect-path git spawn MUST set `GIT_TERMINAL_PROMPT=0` so credential
/// helpers cannot hang hermetic tests or cold-start automation.
pub(crate) fn git_command() -> Command {
    let mut c = Command::new("git");
    c.env("GIT_TERMINAL_PROMPT", "0");
    c
}

/// Extract the repository name from a git remote URL (F32).
///
/// Supports HTTPS, SSH scp-style (`git@host:user/repo`), and ssh:// with port.
pub(crate) fn extract_repo_name(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // Soft: refuse bare Windows drive-letter local remotes (`C:\…`) so we do not
    // return a one-char slug like "C".
    if url.len() >= 2 {
        let bytes = url.as_bytes();
        if bytes[1] == b':'
            && bytes[0].is_ascii_alphabetic()
            && (url.len() == 2 || bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return None;
        }
    }

    // Remove .git suffix
    let url = url.strip_suffix(".git").unwrap_or(url);

    // Match patterns:
    // https://host/path/repo → repo
    // git@host:user/repo → repo
    // ssh://host:port/user/repo → repo
    if let Some(pos) = url.rfind('/') {
        let repo = &url[pos + 1..];
        if !repo.is_empty() {
            return Some(repo.to_string());
        }
    }

    // scp-style without a path slash: git@host:repo
    if let Some(pos) = url.rfind(':') {
        let repo = &url[pos + 1..];
        // Avoid treating host:port as a repo (ssh URL already handled via '/').
        if !repo.is_empty() && !repo.chars().all(|c| c.is_ascii_digit()) {
            return Some(repo.to_string());
        }
    }

    None
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::disallowed_methods)] // unit tests may use expect/panic
mod tests {
    use super::*;

    fn row(id: &str, name: &str, alias: &str) -> ProjectRow {
        (id.to_string(), name.to_string(), alias.to_string(), 0)
    }

    #[test]
    fn match_projects_for_slug__exact__unique() {
        let projects = vec![row("aaa", "Other", "other"), row("bbb", "MyRepo", "myrepo")];
        match match_projects_for_slug(&projects, "MyRepo") {
            SlugMatch::Unique((id, _, _, _)) => assert_eq!(id, "bbb"),
            other => panic!("expected Unique, got {other:?}"),
        }
        match match_projects_for_slug(&projects, "myrepo") {
            SlugMatch::Unique((id, _, _, _)) => assert_eq!(id, "bbb"),
            other => panic!("expected Unique via alias, got {other:?}"),
        }
    }

    #[test]
    fn match_projects_for_slug__contains_only_when_one() {
        let projects = vec![
            row("aaa", "prefix-MySlug-suffix", ""),
            row("bbb", "unrelated", "other"),
        ];
        match match_projects_for_slug(&projects, "MySlug") {
            SlugMatch::Unique((id, _, _, _)) => assert_eq!(id, "aaa"),
            other => panic!("expected Unique contains, got {other:?}"),
        }
    }

    #[test]
    fn match_projects_for_slug__two_contains__ambiguous() {
        let projects = vec![row("z-id", "foo-slug", ""), row("a-id", "bar-slug", "")];
        match match_projects_for_slug(&projects, "slug") {
            SlugMatch::Ambiguous(rows) => {
                assert_eq!(rows.len(), 2);
                // F21: sorted by project_id
                assert_eq!(rows[0].0, "a-id");
                assert_eq!(rows[1].0, "z-id");
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn match_projects_for_slug__two_exact__ambiguous() {
        let projects = vec![row("z-id", "Same", "x"), row("a-id", "other", "Same")];
        match match_projects_for_slug(&projects, "Same") {
            SlugMatch::Ambiguous(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].0, "a-id");
                assert_eq!(rows[1].0, "z-id");
            }
            other => panic!("expected Ambiguous exact, got {other:?}"),
        }
    }

    #[test]
    fn match_projects_for_slug__exact_beats_contains() {
        // Exact on one project; another only contains — prefer exact.
        let projects = vec![
            row("exact", "Slug", ""),
            row("contains", "prefix-Slug-suffix", ""),
        ];
        match match_projects_for_slug(&projects, "Slug") {
            SlugMatch::Unique((id, _, _, _)) => assert_eq!(id, "exact"),
            other => panic!("expected Unique exact over contains, got {other:?}"),
        }
    }

    #[test]
    fn match_projects_for_slug__none() {
        let projects = vec![row("aaa", "alpha", "a")];
        assert_eq!(match_projects_for_slug(&projects, "beta"), SlugMatch::None);
    }

    #[test]
    fn env_fallback_warning__mismatch__some() {
        let w = env_fallback_warning(
            "AI-Brains",
            "441837f6-0000-0000-0000-000000000001",
            "(no alias) — 441837f6",
            "test-alias",
        );
        let text = w.expect("mismatch must warn");
        assert!(
            text.contains("git/env project mismatch"),
            "F35 label; got: {text}"
        );
        assert!(text.contains("AI_BRAINS_PROJECT_ID"));
        assert!(text.contains("441837f6-0000-0000-0000-000000000001"));
        assert!(text.contains("test-alias"));
        assert!(text.contains("'AI-Brains'"));
        assert!(
            text.contains(
                "ai-brains project set-alias 441837f6-0000-0000-0000-000000000001 AI-Brains"
            ),
            "set-alias hint; got: {text}"
        );
    }

    #[test]
    fn env_fallback_warning__exact_alias_match__none() {
        assert!(
            env_fallback_warning(
                "AI-Brains",
                "7d97a456-0000-0000-0000-000000000001",
                "main",
                "AI-Brains",
            )
            .is_none()
        );
        // Case-insensitive exact name match
        assert!(env_fallback_warning("my-repo", "id", "My-Repo", "").is_none());
    }

    #[test]
    fn env_fallback_warning__empty_slug__none() {
        assert!(env_fallback_warning("", "id", "name", "alias").is_none());
    }

    #[test]
    fn extract_repo_name__https_with_git_suffix() {
        assert_eq!(
            extract_repo_name("https://github.com/user/MySlug.git").as_deref(),
            Some("MySlug")
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/MySlug").as_deref(),
            Some("MySlug")
        );
    }

    #[test]
    fn extract_repo_name__ssh_scp() {
        assert_eq!(
            extract_repo_name("git@github.com:user/KinLedger.git").as_deref(),
            Some("KinLedger")
        );
        assert_eq!(
            extract_repo_name("git@github.com:user/KinLedger").as_deref(),
            Some("KinLedger")
        );
    }

    #[test]
    fn extract_repo_name__ssh_url_with_port() {
        assert_eq!(
            extract_repo_name("ssh://git@github.com:22/user/RepoWithPort.git").as_deref(),
            Some("RepoWithPort")
        );
        assert_eq!(
            extract_repo_name("ssh://git@gitlab.example.com:2222/group/proj.git").as_deref(),
            Some("proj")
        );
    }

    #[test]
    fn extract_repo_name__windows_drive_local__none() {
        assert_eq!(extract_repo_name(r"C:\Users\me\repo.git"), None);
        assert_eq!(extract_repo_name("C:/Users/me/repo"), None);
    }

    #[test]
    fn git_command__sets_git_terminal_prompt_zero() {
        // AC11: every detect-path git spawn sets GIT_TERMINAL_PROMPT=0.
        let c = git_command();
        let envs: Vec<_> = c
            .get_envs()
            .filter_map(|(k, v)| {
                let key = k.to_string_lossy();
                v.map(|val| (key.into_owned(), val.to_string_lossy().into_owned()))
            })
            .collect();
        assert!(
            envs.iter()
                .any(|(k, v)| k == "GIT_TERMINAL_PROMPT" && v == "0"),
            "git_command must set GIT_TERMINAL_PROMPT=0; got {envs:?}"
        );
    }

    // --- T240 identity helpers ---

    #[test]
    fn path_vs_slug_conflict_notes__path_wins__always_notes_slug() {
        let path_row = ("path-id".to_string(), "Path".into(), "p".into(), 5);
        let slug_row = ("slug-id".to_string(), "Slug".into(), "s".into(), 10);
        let notes = path_vs_slug_conflict_notes(&path_row, &slug_row);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("slug-id"));
        assert!(notes[0].contains("preferring path alias owner"));
    }

    #[test]
    fn path_vs_slug_conflict_notes__zero_mem_path__extra_verify_note() {
        let path_row = ("path-id".to_string(), "Path".into(), "".into(), 0);
        let slug_row = ("slug-id".to_string(), "Slug".into(), "s".into(), 3);
        let notes = path_vs_slug_conflict_notes(&path_row, &slug_row);
        assert_eq!(notes.len(), 2);
        assert!(notes[1].contains("0 memories"));
        assert!(notes[1].contains("project list"));
    }

    #[test]
    fn path_vs_slug_conflict_notes__path_has_mem__no_extra() {
        let path_row = ("path-id".to_string(), "Path".into(), "".into(), 1);
        let slug_row = ("slug-id".to_string(), "Slug".into(), "s".into(), 99);
        let notes = path_vs_slug_conflict_notes(&path_row, &slug_row);
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn should_skip_identity_mismatch_warn__flags_and_missing() {
        let args_npc = vec!["ai-brains".into(), "--no-project-context".into()];
        assert!(should_skip_identity_mismatch_warn(
            &args_npc,
            Some("env"),
            Some("path")
        ));
        let args_global = vec!["ai-brains".into(), "recall".into(), "--global".into()];
        assert!(should_skip_identity_mismatch_warn(
            &args_global,
            Some("env"),
            Some("path")
        ));
        assert!(should_skip_identity_mismatch_warn(&[], None, Some("path")));
        assert!(should_skip_identity_mismatch_warn(&[], Some("env"), None));
        assert!(!should_skip_identity_mismatch_warn(
            &[],
            Some("env"),
            Some("path")
        ));
        let args_whoami = vec!["ai-brains".into(), "project".into(), "whoami".into()];
        assert!(
            should_skip_identity_mismatch_warn(&args_whoami, Some("env"), Some("path")),
            "AC1: consecutive project whoami must skip"
        );
        let args_adopt = vec!["ai-brains".into(), "project".into(), "adopt-path".into()];
        assert!(
            should_skip_identity_mismatch_warn(&args_adopt, Some("env"), Some("path")),
            "AC1: consecutive project adopt-path must skip"
        );
        let args_list = vec!["ai-brains".into(), "project".into(), "list".into()];
        assert!(
            !should_skip_identity_mismatch_warn(&args_list, Some("env"), Some("path")),
            "AC1: project list stays false"
        );
    }

    #[test]
    fn identity_mismatch_warn_line__soot_text() {
        let line = identity_mismatch_warn_line("env-id", "path-id");
        assert!(line.contains("project identity mismatch"));
        assert!(line.contains("env-id"));
        assert!(line.contains("path-id"));
        assert!(line.contains("project whoami"));
    }

    #[test]
    fn collect_git_identity__non_git_cwd__none_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let id = collect_git_identity(dir.path()).expect("collect");
        assert!(id.slug.is_none());
        assert!(id.toplevel.is_none());
    }

    #[test]
    fn detect_source__as_str__stable() {
        assert_eq!(DetectSource::PathAlias.as_str(), "path_alias");
        assert_eq!(DetectSource::GitSlug.as_str(), "git_slug");
        assert_eq!(DetectSource::Env.as_str(), "env");
    }

    // --- T212 display_label / truncate / last_activity ---

    #[test]
    fn display_label__nonempty_alias__returns_alias() {
        assert_eq!(
            display_label("Some Name", "acme", "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
            "acme"
        );
    }

    #[test]
    fn display_label__baked_no_alias_prefix__literal_no_alias() {
        let pid = "93e74c21-1111-1111-1111-111111111111";
        assert_eq!(
            display_label("(no alias) — 93e74c21", "", pid),
            "(no alias)"
        );
    }

    #[test]
    fn display_label__project_uuid_name__no_alias() {
        let pid = "7d97a456-f2f4-43ea-1f11-abcdef012345";
        assert_eq!(
            display_label("Project 7d97a456-f2f4-43ea-1f11", "", pid),
            "(no alias)"
        );
        assert_eq!(
            display_label(&format!("Project {pid}"), "", pid),
            "(no alias)"
        );
    }

    #[test]
    fn display_label__name_equals_full_or_short_id__no_alias() {
        let pid = "7d97a456-f2f4-43ea-1f11-abcdef012345";
        assert_eq!(display_label(pid, "", pid), "(no alias)");
        assert_eq!(display_label("7d97a456", "", pid), "(no alias)");
    }

    #[test]
    fn display_label__true_human_name__as_is() {
        assert_eq!(
            display_label(
                "AI-Brains monorepo",
                "",
                "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
            ),
            "AI-Brains monorepo"
        );
    }

    // --- T230 never-blank display_label (empty / whitespace name) ---

    #[test]
    fn display_label__empty_name__returns_no_alias() {
        // AC1 + AC16: empty name + empty alias → (no alias); token fits PROJECT_COL_MAX=20 and list label col 30.
        let pid = "eae2d22b-1111-1111-1111-111111111111";
        let label = display_label("", "", pid);
        assert_eq!(label, "(no alias)");
        assert!(
            label.chars().count() <= 20,
            "AC16 (no alias) must fit PROJECT_COL_MAX=20; len={}",
            label.chars().count()
        );
        assert!(
            label.chars().count() <= 30,
            "AC16 (no alias) must fit project list label col 30; len={}",
            label.chars().count()
        );
    }

    #[test]
    fn display_label__whitespace_name__returns_no_alias() {
        // AC2: whitespace-only name counts as empty (name.trim().is_empty).
        let pid = "eae2d22b-2222-2222-2222-222222222222";
        assert_eq!(display_label("   ", "", pid), "(no alias)");
        assert_eq!(display_label("\t\n  ", "", pid), "(no alias)");
    }

    #[test]
    fn display_label__empty_name_with_alias__alias_wins() {
        // AC3: non-empty alias wins over empty name (alias branch first; no alias.trim).
        let pid = "eae2d22b-3333-3333-3333-333333333333";
        assert_eq!(display_label("", "acme", pid), "acme");
        assert_eq!(display_label("   ", "acme", pid), "acme");
    }

    #[test]
    fn truncate_chars__multibyte_at_width__no_panic() {
        // AC11 / F36: CJK + em-dash near width boundary must not panic.
        let s = "日本語テストラベル—境界—値";
        let out = truncate_chars(s, 8);
        assert!(out.chars().count() <= 8, "got {out:?}");
        assert!(out.ends_with('…') || out.chars().count() <= 8);
        // ASCII-only short string unchanged.
        assert_eq!(truncate_chars("short", 30), "short");
        // Exact width.
        assert_eq!(truncate_chars("abcdefghij", 10), "abcdefghij");
        // One over → ellipsis.
        let t = truncate_chars("abcdefghijk", 10);
        assert_eq!(t.chars().count(), 10);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn format_last_activity__empty__em_dash() {
        assert_eq!(format_last_activity(""), "—");
    }

    #[test]
    fn format_last_activity__recent__relative() {
        let recent = chrono::Utc::now().to_rfc3339();
        let out = format_last_activity(&recent);
        assert_eq!(out, "just now", "got {out}");
    }

    #[test]
    fn format_last_activity__old__yyyy_mm_dd() {
        let old = "2020-01-15T12:00:00Z";
        let out = format_last_activity(old);
        assert_eq!(out, "2020-01-15", "got {out}");
    }
}
