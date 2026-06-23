use crate::context::AppContext;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, EventKind, Payload, ProjectAliasAddedPayload};
use ai_brains_store::{EventStore, QueryStore};

pub fn list(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let projects = ctx.conn.list_projects()?;
    println!(
        "{:<36} {:<30} {:<25} memories",
        "project_id", "name (alias|UUID)", "alias"
    );
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
    // Try to detect current repo from git
    let current_dir = std::env::current_dir()?;
    let repo_slug = get_git_repo_slug(&current_dir)?;

    if let Some(slug) = repo_slug {
        // Try to resolve slug as alias or name
        let projects = ctx.conn.list_projects()?;
        let lower_slug = slug.to_lowercase();
        let matched: Vec<_> = projects
            .into_iter()
            .filter(|(_, name, alias_name, _)| {
                name.to_lowercase() == lower_slug
                    || name.to_lowercase().contains(&lower_slug)
                    || alias_name.to_lowercase() == lower_slug
                    || alias_name.to_lowercase().contains(&lower_slug)
            })
            .collect();

        if matched.len() == 1 {
            let (pid, name, alias, count) = &matched[0];
            if export_shell {
                println!("export AI_BRAINS_PROJECT_ID={}", pid);
                println!(
                    "# AI-Brains project detected: {} | alias={} | memories={}",
                    name, alias, count
                );
            } else {
                println!(
                    "Detected project from git: {} ({}) | alias={} | memories={}",
                    name, pid, alias, count
                );
            }
            return Ok(());
        } else if matched.len() > 1 {
            tracing::info!(
                "Ambiguous match for '{}' — multiple candidates found in vault:",
                lower_slug
            );
            for (pid, name, alias, count) in &matched {
                tracing::info!("  {} | {} | {} | {} memories", pid, name, alias, count);
            }
            if export_shell {
                eprintln!("# No unambiguous match — set AI_BRAINS_PROJECT_ID manually");
                std::process::exit(1);
            }
            return Ok(());
        }
    }

    // T93: Fallback — try AI_BRAINS_PROJECT_ID already loaded from .env by main.
    if let Ok(pid_str) = std::env::var("AI_BRAINS_PROJECT_ID") {
        if !pid_str.is_empty() {
            let projects = ctx.conn.list_projects()?;
            if let Some((pid, name, alias, _count)) =
                projects.iter().find(|(p, _, _, _)| p == &pid_str)
            {
                if export_shell {
                    println!("export AI_BRAINS_PROJECT_ID={}", pid);
                    println!(
                        "# AI-Brains project detected from .env: {} | alias={} (from .env)",
                        name, alias
                    );
                } else {
                    println!(
                        "Detected project from .env: {} ({}) | alias={} (from .env)",
                        name, pid, alias
                    );
                }
                return Ok(());
            }
        }
    }

    // Nothing found via slug match or .env.
    let msg = "No project detected. Set an alias with 'project set-alias' or initialize a project with 'init'.";
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
        EventKind::ProjectAliasAdded,
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

fn get_git_repo_slug(path: &std::path::Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Try git rev-parse --show-toplevel
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let toplevel = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let toplevel_path = std::path::Path::new(&toplevel);

    // Try name from directory
    if let Some(name) = toplevel_path.file_name().and_then(|n| n.to_str()) {
        let cleaned = name.to_string();
        if !cleaned.is_empty() {
            return Ok(Some(cleaned));
        }
    }

    // Try git remote
    let remote = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()?;

    if remote.status.success() {
        let url = String::from_utf8_lossy(&remote.stdout).trim().to_owned();
        // Extract repo name from git URL
        // e.g. https://github.com/user/Sneaky-Browse.git → Sneaky-Browse
        // e.g. git@github.com:user/KinLedger.git → KinLedger
        if let Some(slug) = extract_repo_name(&url) {
            return Ok(Some(slug));
        }
    }

    Ok(None)
}

fn extract_repo_name(url: &str) -> Option<String> {
    // Remove .git suffix
    let url = url.strip_suffix(".git").unwrap_or(url);

    // Match patterns:
    // https://host/path/repo.git → repo
    // git@host:user/repo.git → repo
    // ssh://host/user/repo.git → repo

    if let Some(pos) = url.rfind('/') {
        let repo = &url[pos + 1..];
        if !repo.is_empty() {
            return Some(repo.to_string());
        }
    }

    if let Some(pos) = url.rfind(':') {
        let repo = &url[pos + 1..];
        if !repo.is_empty() {
            return Some(repo.to_string());
        }
    }

    None
}
