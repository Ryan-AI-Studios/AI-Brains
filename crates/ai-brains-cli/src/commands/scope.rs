//! `ai-brains scope resolve` — governed scope resolution (T160 / #20).

use crate::commands::governed_common::{
    self, OutputFormat, PathDecision, PathFlags, emit_json, emit_scope_human, expect_daemon_ok,
    fail_cp, fail_path,
};
use crate::context::AppContext;
use crate::daemon_client::DaemonClient;
use ai_brains_contracts::scopes::ResolveScopeRequest;
use ai_brains_control_plane::{ScopeResolveInput, StorePorts, resolve_scope};
use ai_brains_core::ids::{ProjectId, UserId};
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::SqliteEventStore;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::str::FromStr;

pub struct ResolveOptions {
    pub format: String,
    pub cwd: Option<String>,
    pub project_id: Option<ProjectId>,
    pub force_personal: bool,
    pub personal_user_id: Option<String>,
    pub local: bool,
    pub daemon: bool,
    pub require_daemon: bool,
}

/// Resolve `scope resolve --format` (T249). Clap rejects unknowns; `_` is fail-closed json.
pub(crate) fn resolve_scope_format(explicit: &str, is_tty: bool) -> &'static str {
    crate::commands::format_resolve::resolve_human_json_format(explicit, is_tty)
}

fn scope_output_format(resolved: &str) -> OutputFormat {
    match resolved {
        "human" => OutputFormat::Human,
        _ => OutputFormat::Json,
    }
}

/// `ai-brains scope resolve`
pub async fn run_resolve(
    ctx: &AppContext,
    options: ResolveOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolved = resolve_scope_format(&options.format, std::io::stdout().is_terminal());
    let format = scope_output_format(resolved);
    let flags = PathFlags {
        local: options.local,
        daemon: options.daemon,
        require_daemon: options.require_daemon,
    };
    let path = match governed_common::choose_read_path(flags).await {
        Ok(p) => p,
        Err(e) => return fail_path(format, e),
    };

    match path {
        PathDecision::Daemon => run_resolve_daemon(ctx, &options, format).await,
        PathDecision::Local { .. } => run_resolve_local(ctx, &options, format),
    }
}

fn run_resolve_local(
    ctx: &AppContext,
    options: &ResolveOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = SqliteEventStore::new((*ctx.conn).clone());
    let ports = StorePorts::from_store(store);
    let identity = ports.identity_store();
    let cwd = options
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let personal_user_id = options
        .personal_user_id
        .as_deref()
        .and_then(|s| UserId::from_str(s).ok());
    let input = ScopeResolveInput {
        cwd: cwd.clone(),
        explicit_project_id: options.project_id,
        force_personal: options.force_personal,
        personal_user_id,
        git_metadata: None,
    };
    let resolved = match resolve_scope(&input, &identity) {
        Ok(r) => r,
        Err(e) => return fail_cp(format, e),
    };
    let mut wire = governed_common::map_resolved_scope(&resolved);
    match format {
        OutputFormat::Json => {
            crate::commands::identity_warn::inject_identity_mismatch_warning(&mut wire.warnings);
            crate::commands::identity_warn::inject_identity_collision_warning(
                &mut wire.warnings,
                ctx,
                &cwd,
            );
            crate::commands::identity_warn::inject_detect_env_fallback_warning(
                &mut wire.warnings,
                ctx,
                &cwd,
            );
            emit_json(&wire)
        }
        OutputFormat::Human | OutputFormat::Markdown => {
            emit_scope_human(&wire);
            Ok(())
        }
    }
}

async fn run_resolve_daemon(
    ctx: &AppContext,
    options: &ResolveOptions,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = DaemonClient::new();
    let req = DaemonRequest::ResolveScope(ResolveScopeRequest {
        api_version: ai_brains_contracts::scopes::API_VERSION.to_string(),
        cwd: options.cwd.clone().or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        }),
        signals: None,
        explicit_project_id: options.project_id.map(|p| p.to_string()),
        force_personal: options.force_personal,
        personal_user_id: options.personal_user_id.clone(),
    });
    let resp = match client.request(req).await {
        Ok(r) => r,
        Err(_e) => {
            return fail_path(format, governed_common::PathPolicyError::DaemonUnavailable);
        }
    };
    let resp = expect_daemon_ok(format, resp)?;
    let overlay_cwd = options
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match resp {
        DaemonResponse::ScopeResolved(mut wire) => match format {
            OutputFormat::Json => {
                crate::commands::identity_warn::inject_identity_mismatch_warning(
                    &mut wire.warnings,
                );
                crate::commands::identity_warn::inject_identity_collision_warning(
                    &mut wire.warnings,
                    ctx,
                    &overlay_cwd,
                );
                crate::commands::identity_warn::inject_detect_env_fallback_warning(
                    &mut wire.warnings,
                    ctx,
                    &overlay_cwd,
                );
                emit_json(&wire)
            }
            OutputFormat::Human | OutputFormat::Markdown => {
                emit_scope_human(&wire);
                Ok(())
            }
        },
        other => Err(format!("unexpected daemon response: {other:?}").into()),
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn resolve_scope_format__auto_tty__human() {
        assert_eq!(resolve_scope_format("auto", true), "human");
    }

    #[test]
    fn resolve_scope_format__auto_pipe__json() {
        assert_eq!(resolve_scope_format("auto", false), "json");
    }

    #[test]
    fn resolve_scope_format__pretty__human_regardless_of_tty() {
        assert_eq!(resolve_scope_format("pretty", true), "human");
        assert_eq!(resolve_scope_format("pretty", false), "human");
    }

    #[test]
    fn resolve_scope_format__human__human_regardless_of_tty() {
        assert_eq!(resolve_scope_format("human", true), "human");
        assert_eq!(resolve_scope_format("human", false), "human");
    }

    #[test]
    fn resolve_scope_format__text__human_regardless_of_tty() {
        assert_eq!(resolve_scope_format("text", true), "human");
        assert_eq!(resolve_scope_format("text", false), "human");
    }

    #[test]
    fn resolve_scope_format__markdown__human_regardless_of_tty() {
        assert_eq!(resolve_scope_format("markdown", true), "human");
        assert_eq!(resolve_scope_format("markdown", false), "human");
    }

    #[test]
    fn resolve_scope_format__md__human_regardless_of_tty() {
        assert_eq!(resolve_scope_format("md", true), "human");
        assert_eq!(resolve_scope_format("md", false), "human");
    }

    #[test]
    fn resolve_scope_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_scope_format("json", true), "json");
        assert_eq!(resolve_scope_format("json", false), "json");
    }

    #[test]
    fn resolve_scope_format__unknown__fail_closed_json() {
        assert_eq!(resolve_scope_format("xml", true), "json");
        assert_eq!(resolve_scope_format("JSON", false), "json");
        assert_eq!(resolve_scope_format("Pretty", true), "json");
    }
}
