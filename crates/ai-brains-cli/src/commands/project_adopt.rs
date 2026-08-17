//! T258 — `project adopt-path`: bind daily Scope to the path-alias owner.
//!
//! Default is print-only (T240 F2). `--write-env --yes` rewrites only
//! `AI_BRAINS_PROJECT_ID` in cwd `.env`. Does not call `context`, rotate
//! session, or append events.

use crate::commands::governed_common::fail_usage;
use crate::commands::project::{collect_git_identity, resolve_path_alias_for_location};
use crate::context::AppContext;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::Path;

const PROJECT_ID_KEY: &str = "AI_BRAINS_PROJECT_ID";

#[derive(Debug, Serialize)]
struct AdoptPathJson {
    api_version: String,
    action: String,
    env_path: String,
    from_project_id: Option<String>,
    to_project_id: Option<String>,
    written: bool,
    already_bound: bool,
    keys_touched: Vec<String>,
}

/// Replace or append the unexported `AI_BRAINS_PROJECT_ID=` line.
///
/// Matches `context.rs` (`starts_with("AI_BRAINS_PROJECT_ID")`): `export `
/// prefixed lines are not replaced (soft residual).
pub(crate) fn rewrite_project_id_line(existing: &str, new_id: &str) -> String {
    let assignment = format!("{PROJECT_ID_KEY}={new_id}");
    let mut found = false;
    let lines: Vec<String> = existing
        .lines()
        .map(|line| {
            if line.starts_with(PROJECT_ID_KEY) {
                found = true;
                assignment.clone()
            } else {
                line.to_string()
            }
        })
        .collect();

    if found {
        let mut out = lines.join("\n");
        if existing.ends_with('\n') {
            out.push('\n');
        }
        out
    } else {
        let mut out = existing.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&assignment);
        out.push('\n');
        out
    }
}

/// Refuse reparse, then `fs::write` the rewritten `.env`.
pub(crate) fn adopt_write(
    path: &Path,
    new_id: &str,
    is_reparse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(msg) = crate::artifact_security::refuse_if_reparse(path, is_reparse) {
        return Err(msg.into());
    }
    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    let next = rewrite_project_id_line(&existing, new_id);
    std::fs::write(path, next)?;
    Ok(())
}

fn file_project_id(env_path: &Path) -> Option<String> {
    let existing = std::fs::read_to_string(env_path).ok()?;
    existing
        .lines()
        .find(|line| line.starts_with(PROJECT_ID_KEY))
        .and_then(|line| line.split('=').nth(1))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn use_json_output(format: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match format.to_ascii_lowercase().as_str() {
        "json" => Ok(true),
        "human" => Ok(false),
        "auto" => Ok(!std::io::stdout().is_terminal()),
        other => fail_usage(format!(
            "unknown --format '{other}' (expected auto, human, or json)"
        ))
        .map(|()| false),
    }
}

/// Print-only remediator, or confirmable write of cwd `.env` PROJECT_ID.
pub fn run(
    ctx: &AppContext,
    write_env: bool,
    yes: bool,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if write_env && !yes {
        return fail_usage("--write-env requires --yes (no silent .env write)");
    }

    let cwd = std::env::current_dir()?;
    let env_path = cwd.join(".env");
    let git = collect_git_identity(&cwd)?;
    let path_owner = resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)?;

    let no_project_context = std::env::args().any(|a| a == "--no-project-context");
    let from_project_id = if no_project_context {
        file_project_id(&env_path)
    } else {
        std::env::var(PROJECT_ID_KEY).ok().filter(|s| !s.is_empty())
    };

    let Some(to) = path_owner else {
        eprintln!(
            "No path alias registered for this location. Run `ai-brains project register-path <project-id> <path>` for the cwd or git toplevel."
        );
        return Err("no path owner; register-path required".into());
    };

    let already_bound = from_project_id.as_deref() == Some(to.as_str());

    let written = if write_env && yes && !already_bound {
        let is_reparse = crate::artifact_security::is_reparse_or_symlink(&env_path)?;
        adopt_write(&env_path, &to, is_reparse)?;
        true
    } else {
        false
    };

    let use_json = use_json_output(format)?;
    if use_json {
        let report = AdoptPathJson {
            api_version: "1".to_string(),
            action: "adopt-path".to_string(),
            env_path: env_path.display().to_string(),
            from_project_id: from_project_id.clone(),
            to_project_id: Some(to.clone()),
            written,
            already_bound,
            keys_touched: vec![PROJECT_ID_KEY.to_string()],
        };
        crate::commands::identity_warn::print_json_stdout(&report)?;
        return Ok(());
    }

    if already_bound {
        println!("Already bound to path owner {to}");
        println!("No .env write.");
        return Ok(());
    }

    if written {
        println!("Set AI_BRAINS_PROJECT_ID={to} in {}", env_path.display());
        println!("Other keys left untouched.");
    } else {
        println!(
            "Would set AI_BRAINS_PROJECT_ID={to} in {}",
            env_path.display()
        );
        println!("from: {}", from_project_id.as_deref().unwrap_or("(none)"));
        println!("to:   {to}");
        println!("Other keys would be left untouched.");
        println!("Re-run with --write-env --yes to apply.");
    }
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case, clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_project_id_in_env__preserves_other_keys() {
        let existing = concat!(
            "AI_BRAINS_PROJECT_ID=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa\n",
            "AI_BRAINS_KEY=x'deadbeefcafebabe'\n",
            "AI_BRAINS_SESSION_ID=11111111-1111-1111-1111-111111111111\n",
            "# keep me\n",
            "AI_BRAINS_VAULT_PATH=C:\\tmp\\vault.db\n",
        );
        let next = rewrite_project_id_line(existing, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
        assert!(next.contains("AI_BRAINS_PROJECT_ID=bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"));
        assert!(next.contains("AI_BRAINS_KEY=x'deadbeefcafebabe'"));
        assert!(next.contains("AI_BRAINS_SESSION_ID=11111111-1111-1111-1111-111111111111"));
        assert!(next.contains("# keep me"));
        assert!(next.contains("AI_BRAINS_VAULT_PATH=C:\\tmp\\vault.db"));
        assert!(!next.contains("AI_BRAINS_HARNESS_ID"));
        assert!(!next.contains("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
    }

    #[test]
    fn rewrite_project_id_in_env__refuse_reparse() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".env");
        let err = adopt_write(&path, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", true);
        assert!(err.is_err(), "reparse must refuse");
        let msg = err.expect_err("checked").to_string();
        assert!(
            msg.contains("reparse") || msg.contains("symlink") || msg.contains("junction"),
            "refuse message; got: {msg}"
        );
        assert!(!path.exists(), "refuse must not create the file");
    }

    #[test]
    fn rewrite_project_id_line__missing_key__appends() {
        let next = rewrite_project_id_line(
            "AI_BRAINS_KEY=x'dead'\n",
            "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        );
        assert_eq!(
            next,
            "AI_BRAINS_KEY=x'dead'\nAI_BRAINS_PROJECT_ID=bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb\n"
        );
    }
}
