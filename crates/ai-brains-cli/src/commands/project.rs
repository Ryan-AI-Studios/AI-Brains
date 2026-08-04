use crate::context::AppContext;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Payload, ProjectAliasAddedPayload};
use ai_brains_store::{EventStore, QueryStore};
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

pub fn list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let projects = ctx.conn.list_projects()?;
    println!(
        "{:<36} {:<30} {:<25} memories",
        "project_id", "name (alias|UUID)", "alias"
    );
    if projects.is_empty() {
        println!("No projects registered. (0 projects)");
        return Ok(());
    }
    for (pid, name, alias, count) in projects {
        println!(
            "{:<36} {:<30} {:<25} {}",
            pid,
            &name[..std::cmp::min(30, name.len())],
            alias,
            count
        );
    }
    Ok(())
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

pub fn detect(ctx: &AppContext, export_shell: bool) -> Result<(), Box<dyn std::error::Error>> {
    // F2: (1) Resolve git identity slug (F31 remote-first).
    let current_dir = std::env::current_dir()?;
    let repo_slug = get_git_repo_slug(&current_dir)?;

    // F2: (2) Vault match (exact-first F3).
    if let Some(ref slug) = repo_slug {
        let projects = ctx.conn.list_projects()?;
        match match_projects_for_slug(&projects, slug) {
            // AC1: unique git match wins over wrong env PROJECT_ID.
            SlugMatch::Unique((pid, name, alias, count)) => {
                if export_shell {
                    println!("export AI_BRAINS_PROJECT_ID={}", pid);
                    println!(
                        "# AI-Brains project detected: {} | alias={} | memories={} | from git",
                        name, alias, count
                    );
                } else {
                    println!(
                        "Detected project from git: {} ({}) | alias={} | memories={}",
                        name, pid, alias, count
                    );
                }
                return Ok(());
            }
            // F5/F18/AC4: ambiguous ≥2 → stderr candidates (sorted), exit 1.
            SlugMatch::Ambiguous(matched) => {
                if export_shell {
                    eprintln!(
                        "# Ambiguous match for '{}' — multiple candidates; set AI_BRAINS_PROJECT_ID manually",
                        slug
                    );
                    for (pid, name, alias, count) in &matched {
                        eprintln!("#   {} | {} | {} | {} memories", pid, name, alias, count);
                    }
                } else {
                    eprintln!(
                        "Ambiguous match for '{}' — multiple candidates found in vault:",
                        slug
                    );
                    for (pid, name, alias, count) in &matched {
                        eprintln!("  {} | {} | {} | {} memories", pid, name, alias, count);
                    }
                }
                std::process::exit(1);
            }
            SlugMatch::None => {
                // Fall through to env fallback.
            }
        }
    }

    // F2: (3) Process AI_BRAINS_PROJECT_ID if in vault (F4 warn when slug known + mismatch).
    if let Ok(pid_str) = std::env::var("AI_BRAINS_PROJECT_ID")
        && !pid_str.is_empty()
    {
        let projects = ctx.conn.list_projects()?;
        if let Some((pid, name, alias, _count)) = projects.iter().find(|(p, _, _, _)| p == &pid_str)
        {
            let warn = env_fallback_warning(repo_slug.as_deref().unwrap_or(""), pid, name, alias);
            if export_shell {
                if let Some(ref w) = warn {
                    for line in w.lines() {
                        println!("# {}", line);
                    }
                }
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected from .env: {} | alias={} (from .env)",
                    name, alias
                );
            } else {
                if let Some(ref w) = warn {
                    eprintln!("{}", w);
                }
                println!(
                    "Detected project from .env: {} ({}) | alias={} (from .env)",
                    name, pid, alias
                );
            }
            return Ok(());
        }
    }

    // F2: (4) Miss exit 1.
    let msg = "No project detected. Set an alias with 'project set-alias', initialize a project with 'init', or run 'ai-brains context'.";
    if export_shell {
        eprintln!("# {}", msg);
    } else {
        eprintln!("{}", msg);
    }
    std::process::exit(1);
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

/// F31 / AC10: prefer origin remote repo name; fall back to toplevel dir name.
fn get_git_repo_slug(path: &std::path::Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // 1. git rev-parse --show-toplevel (fail → None)
    let output = git_command()
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if toplevel.is_empty() {
        return Ok(None);
    }
    let toplevel_path = std::path::Path::new(&toplevel);

    // 2. git remote get-url origin → extract_repo_name (success → Some)
    let remote = git_command()
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()?;

    if remote.status.success() {
        let url = String::from_utf8_lossy(&remote.stdout).trim().to_owned();
        if let Some(slug) = extract_repo_name(&url)
            && !slug.is_empty()
        {
            return Ok(Some(slug));
        }
    }

    // 3. Fallback: toplevel directory file_name
    if let Some(name) = toplevel_path.file_name().and_then(|n| n.to_str()) {
        let cleaned = name.to_string();
        if !cleaned.is_empty() {
            return Ok(Some(cleaned));
        }
    }

    Ok(None)
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
}
