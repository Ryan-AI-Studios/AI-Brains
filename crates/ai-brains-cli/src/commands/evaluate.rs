//! `ai-brains evaluate governed` — hermetic trust-gate harness (T169).
//!
//! Never opens a live vault for write. Per-scenario tempfile vaults are created
//! inside control-plane evaluation seeds. Optional `--report` writes JSON only.

use crate::artifact_security::{
    is_hardlink, is_reparse_or_symlink, refuse_if_hardlink, refuse_if_reparse,
};
use crate::commands::governed_common::{
    EXIT_HARD_GATE_FAILED, GovernedCliError, OutputFormat, api_error_from_cp, emit_error,
    emit_json, exit_code_for_api_error, fail_api,
};
use ai_brains_contracts::response::ApiError;
use ai_brains_control_plane::evaluation::{
    EvaluateOptions, evaluate_scenarios, load_scenarios_dir,
};
use std::path::{Path, PathBuf};

/// CLI options for `evaluate governed`.
#[derive(Debug, Clone)]
pub struct GovernedEvaluateOptions {
    pub fixtures: PathBuf,
    pub report: Option<PathBuf>,
    pub scenario: Vec<String>,
    pub strict_soft: bool,
    pub require_all_active: bool,
    pub allow_report_overwrite: bool,
    /// Optional live vault path (from `--vault-path`) for same-location refuse only.
    pub vault_path: Option<PathBuf>,
}

/// True when `--report` is omitted or is the stdout sentinel `-` (spec E22).
fn report_is_stdout_only(report: &Option<PathBuf>) -> bool {
    match report {
        None => true,
        Some(p) => p.as_os_str() == "-",
    }
}

/// Run evaluate governed: load fixtures, hermetic run, emit report, exit per E22.
pub fn run_governed(opts: GovernedEvaluateOptions) -> Result<(), Box<dyn std::error::Error>> {
    // P2-03: `--report -` is stdout-only (no file write / no path refuse).
    let write_report_path = if report_is_stdout_only(&opts.report) {
        None
    } else {
        opts.report.as_ref()
    };

    if let Some(report) = write_report_path {
        refuse_unsafe_evaluate_report_path(report, opts.allow_report_overwrite)?;
        if let Some(ref vault) = opts.vault_path {
            refuse_report_equals_vault(report, vault)?;
        }
    }

    let scenarios = match load_scenarios_dir(&opts.fixtures) {
        Ok(s) => s,
        Err(e) => {
            // P2-04: structured ControlPlaneError → exit code (no substring matching).
            let api = api_error_from_cp(&e);
            return fail_api(OutputFormat::Json, api);
        }
    };

    if scenarios.is_empty() {
        return fail_api(
            OutputFormat::Json,
            ApiError::new(
                "INVALID_PAYLOAD",
                format!("no scenario JSON files in {}", opts.fixtures.display()),
            ),
        );
    }

    let eval_opts = EvaluateOptions {
        strict_soft: opts.strict_soft,
        require_all_active: opts.require_all_active,
        scenario_filter: opts.scenario.clone(),
    };

    let outcome = match evaluate_scenarios(&scenarios, &eval_opts) {
        Ok(o) => o,
        Err(e) => {
            // P2-04: map via ControlPlaneError kind, not message substrings.
            let api = api_error_from_cp(&e);
            let code = exit_code_for_api_error(&api);
            let _ = emit_error(OutputFormat::Json, &api);
            return Err(Box::new(GovernedCliError::emitted(code, api.message)));
        }
    };

    // Always emit JSON report to stdout; optionally also write a file when path is real.
    if let Some(report_path) = write_report_path {
        write_report(report_path, &outcome.report)?;
    }
    emit_json(&outcome.report)?;

    if outcome.report.hard_gates_passed {
        Ok(())
    } else {
        let api = ApiError::new(
            "HARD_GATE_FAILED",
            format!(
                "hard gates failed (passed={}, soft_failures={})",
                outcome.report.hard_gates_passed,
                outcome.report.soft_failures.len()
            ),
        );
        // Report already on stdout; emit code mapping for scripts via exit.
        let _ = api;
        Err(Box::new(GovernedCliError::emitted(
            EXIT_HARD_GATE_FAILED,
            "HARD_GATE_FAILED",
        )))
    }
}

fn write_report(
    path: &Path,
    report: &ai_brains_control_plane::evaluation::EvaluateReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists()
        && let Err(msg) = refuse_if_hardlink(path, is_hardlink(path)?)
    {
        return fail_path_refused(format!("refusing evaluate report path: {msg}"));
    }
    let body = format!("{}\n", serde_json::to_string_pretty(report)?);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| format!("evaluate report path has no parent: {}", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("evaluate report missing UTF-8 name: {}", path.display()))?;
    // T193 P1: nofollow SOOT Replace (overwrite already gated by refuse_unsafe).
    match ai_brains_path::write_file_nofollow_under_parent_path(
        parent,
        file_name,
        body.as_bytes(),
        ai_brains_path::CreateMode::Replace,
    ) {
        Ok(()) => Ok(()),
        Err(ai_brains_path::CapOpenError::ReparseRefused(s))
        | Err(ai_brains_path::CapOpenError::HardlinkRefused(s)) => {
            fail_path_refused(format!("refusing evaluate report path: {s}"))
        }
        Err(other) => Err(format!(
            "failed to write evaluate report {}: {other}",
            path.display()
        )
        .into()),
    }
}

/// Refuse reparse/symlink report paths; refuse overwriting without flag.
pub fn refuse_unsafe_evaluate_report_path(
    report: &Path,
    allow_overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(msg) = refuse_if_reparse(report, is_reparse_or_symlink(report)?) {
        return fail_path_refused(format!("refusing evaluate report path: {msg}"));
    }
    if report.exists()
        && let Err(msg) = refuse_if_hardlink(report, is_hardlink(report)?)
    {
        return fail_path_refused(format!("refusing evaluate report path: {msg}"));
    }
    if let Some(parent) = report.parent()
        && !parent.as_os_str().is_empty()
        && let Err(msg) = refuse_if_reparse(parent, is_reparse_or_symlink(parent)?)
    {
        return fail_path_refused(format!("refusing evaluate report parent: {msg}"));
    }
    if report.exists() && !allow_overwrite {
        return fail_path_refused(
            "refusing evaluate: report path already exists (pass --allow-report-overwrite)",
        );
    }
    // Refuse writing report onto a path that looks like a vault db in common locations.
    if let Some(name) = report.file_name().and_then(|n| n.to_str())
        && (name.ends_with(".db") || name == "vault.db" || name.ends_with(".sqlite"))
    {
        return fail_path_refused(
            "refusing evaluate: report path looks like a vault database file",
        );
    }
    Ok(())
}

/// Refuse when report path equals a provided vault path (T168-style same-location).
pub fn refuse_report_equals_vault(
    report: &Path,
    vault: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if ai_brains_path::paths_refer_to_same_location(report, vault) {
        return fail_path_refused("refusing evaluate: report path equals a vault database path");
    }
    Ok(())
}

fn fail_path_refused(message: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
    fail_api(
        OutputFormat::Json,
        ApiError::new("PATH_REFUSED", message.into()),
    )
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn evaluate_cli__refuse_report_equals_vault_path_unit() {
        let dir = tempdir().unwrap();
        let vault = dir.path().join("vault.db");
        std::fs::write(&vault, b"x").unwrap();
        let err = refuse_report_equals_vault(&vault, &vault).expect_err("must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("PATH_REFUSED") || msg.contains("vault"),
            "{msg}"
        );
    }

    #[test]
    fn evaluate_cli__refuse_report_db_extension() {
        let dir = tempdir().unwrap();
        let report = dir.path().join("out.db");
        let err = refuse_unsafe_evaluate_report_path(&report, true).expect_err("db ext");
        assert!(err.to_string().contains("vault") || err.to_string().contains("PATH_REFUSED"));
    }

    #[test]
    fn evaluate_cli__report_dash__stdout_only() {
        assert!(report_is_stdout_only(&None));
        assert!(report_is_stdout_only(&Some(PathBuf::from("-"))));
        assert!(!report_is_stdout_only(&Some(PathBuf::from("report.json"))));
    }
}
