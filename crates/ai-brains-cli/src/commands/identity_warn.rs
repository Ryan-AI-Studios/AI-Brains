//! T257 — identity-mismatch warn pending state + JSON stdout hygiene.
//!
//! Record at vault-open; flush human SOOT from `handle_cli_result` only when
//! the command did not emit machine JSON. `print_json_stdout` lives here (F8/F11).

use crate::context::AppContext;
use serde::Serialize;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

struct PendingMismatch {
    env: String,
    path: String,
}

static PENDING: OnceLock<Option<PendingMismatch>> = OnceLock::new();
static MACHINE_STDOUT: AtomicBool = AtomicBool::new(false);

/// Mark that this process already wrote machine JSON on stdout (F2/F8).
pub fn note_machine_stdout() {
    MACHINE_STDOUT.store(true, Ordering::SeqCst);
}

/// Pretty-print one JSON value to stdout and note machine mode (F8).
pub fn print_json_stdout(value: &impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let s = serde_json::to_string_pretty(value)?;
    note_machine_stdout();
    println!("{s}");
    Ok(())
}

/// Pure skip logic for mismatch warn (T240 F3b + T257 F5 remediator skip).
pub(crate) fn should_skip_identity_mismatch_warn(
    args: &[String],
    env_project_id: Option<&str>,
    path_alias_project_id: Option<&str>,
) -> bool {
    if args
        .iter()
        .any(|a| a == "--no-project-context" || a == "--global")
    {
        return true;
    }
    if argv_has_consecutive(args, "project", "whoami")
        || argv_has_consecutive(args, "project", "adopt-path")
    {
        return true;
    }
    let env = env_project_id.filter(|s| !s.is_empty());
    let path = path_alias_project_id.filter(|s| !s.is_empty());
    env.is_none() || path.is_none()
}

fn argv_has_consecutive(args: &[String], first: &str, second: &str) -> bool {
    args.windows(2).any(|w| w[0] == first && w[1] == second)
}

/// SOOT mismatch warn line (T240 F3).
pub(crate) fn identity_mismatch_warn_line(env_id: &str, path_id: &str) -> String {
    format!(
        "Warning: project identity mismatch: daily Scope is '{env_id}', but path is registered to '{path_id}'. Run 'ai-brains project whoami'."
    )
}

/// Stable machine token for `scope resolve` JSON `warnings[]` (F3).
pub(crate) fn identity_mismatch_json_token(env: &str, path: &str) -> String {
    format!("project_identity_mismatch env={env} path={path}")
}

/// Inject the mismatch token once (F25). Testable without process pending state.
pub(crate) fn inject_identity_mismatch_token(warnings: &mut Vec<String>, env: &str, path: &str) {
    if warnings
        .iter()
        .any(|w| w.starts_with("project_identity_mismatch"))
    {
        return;
    }
    warnings.push(identity_mismatch_json_token(env, path));
}

/// Inject from recorded pending mismatch (no-op when none / skipped).
pub fn inject_identity_mismatch_warning(warnings: &mut Vec<String>) {
    let Some(Some(pending)) = PENDING.get() else {
        return;
    };
    inject_identity_mismatch_token(warnings, &pending.env, &pending.path);
}

/// Record mismatch once per process. Does not print (F6).
pub fn record_identity_mismatch(ctx: &AppContext) {
    let _ = PENDING.get_or_init(|| compute_pending(ctx));
}

fn compute_pending(ctx: &AppContext) -> Option<PendingMismatch> {
    let args: Vec<String> = std::env::args().collect();
    let env_id = std::env::var("AI_BRAINS_PROJECT_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let cwd = std::env::current_dir().ok()?;
    let git = crate::commands::project::collect_git_identity(&cwd).unwrap_or_default();
    let path_id =
        crate::commands::project::resolve_path_alias_for_location(ctx.conn.as_ref(), &cwd, &git)
            .unwrap_or_default();
    if should_skip_identity_mismatch_warn(&args, env_id.as_deref(), path_id.as_deref()) {
        return None;
    }
    let (Some(env), Some(path)) = (env_id, path_id) else {
        return None;
    };
    if env == path {
        return None;
    }
    Some(PendingMismatch { env, path })
}

/// eprint T240 SOOT only when pending, not machine, not skipped (F6).
pub fn flush_identity_mismatch_warn() {
    if MACHINE_STDOUT.load(Ordering::SeqCst) {
        return;
    }
    if let Some(Some(pending)) = PENDING.get() {
        eprintln!(
            "{}",
            identity_mismatch_warn_line(&pending.env, &pending.path)
        );
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn identity_mismatch_json_token__stable_no_warning_prefix() {
        let token = identity_mismatch_json_token("e", "p");
        assert_eq!(token, "project_identity_mismatch env=e path=p");
        assert!(!token.contains("Warning:"));
        assert!(!token.contains("project identity mismatch"));
    }

    #[test]
    fn inject_identity_mismatch_token__already_present__no_duplicate() {
        let mut warnings = vec!["project_identity_mismatch env=e path=p".to_string()];
        inject_identity_mismatch_token(&mut warnings, "e", "p");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0], "project_identity_mismatch env=e path=p");
    }

    #[test]
    fn inject_identity_mismatch_token__empty__pushes_once() {
        let mut warnings = Vec::new();
        inject_identity_mismatch_token(&mut warnings, "e", "p");
        inject_identity_mismatch_token(&mut warnings, "e", "p");
        assert_eq!(warnings, vec!["project_identity_mismatch env=e path=p"]);
    }

    #[test]
    fn should_skip_identity_mismatch_warn__whoami_and_adopt_path() {
        let whoami = vec!["ai-brains".into(), "project".into(), "whoami".into()];
        assert!(should_skip_identity_mismatch_warn(
            &whoami,
            Some("env"),
            Some("path")
        ));
        let adopt = vec!["ai-brains".into(), "project".into(), "adopt-path".into()];
        assert!(should_skip_identity_mismatch_warn(
            &adopt,
            Some("env"),
            Some("path")
        ));
        let list = vec!["ai-brains".into(), "project".into(), "list".into()];
        assert!(!should_skip_identity_mismatch_warn(
            &list,
            Some("env"),
            Some("path")
        ));
    }
}
