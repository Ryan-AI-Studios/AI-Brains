//! `ai-brains harness` — detect, install, uninstall, reset-decline (T235).
//!
//! Vault-path-free: pure fs/env probes; no models/graph.

use crate::commands::governed_common::{self, GovernedResult};
use crate::harness::{
    HARNESS_ORDER, HarnessId, InstallOutcome, UninstallOutcome, collect_status_report,
    f34_map_contract_summary, install_agy, install_grok, install_pending, install_pending_summary,
    load_prefs, parse_harness_id, resolve_home, save_prefs, uninstall_agy, uninstall_grok,
    uninstall_pending, wiring_status_label,
};
use is_terminal::IsTerminal;

#[derive(Debug, Clone)]
pub struct HarnessStatusOptions {
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct HarnessInstallOptions {
    pub harness: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct HarnessUninstallOptions {
    pub harness: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct HarnessResetDeclineOptions {
    pub harness: Option<String>,
}

pub fn run_status(opts: HarnessStatusOptions) -> GovernedResult {
    let home = resolve_home();
    let report = collect_status_report(home.as_deref());
    let fmt = opts.format.to_ascii_lowercase();
    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Harnesses installed on machine:");
        println!("  home: {}", report.home);
        for h in &report.harnesses {
            let present = if h.present { "yes" } else { "no" };
            println!(
                "  {} ({}) present={} wiring={} install_ready={}",
                h.id,
                h.display_name,
                present,
                wiring_status_label(h.wiring),
                h.install_ready
            );
            if h.present {
                println!("    next: {}", h.next_action);
            }
        }
        println!();
        println!("Message-only capture: user prompts + final assistant text (T234).");
        println!("AGY ready: ai-brains harness install --harness agy --dry-run");
        println!("Grok ready: ai-brains harness install --harness grok --dry-run");
    }
    Ok(())
}

pub fn run_install(opts: HarnessInstallOptions) -> GovernedResult {
    let home = resolve_home().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot resolve home (USERPROFILE/HOME)",
        )
    })?;

    let ids = resolve_harness_list(opts.harness.as_deref())?;
    if ids.is_empty() {
        return governed_common::fail_usage(
            "specify --harness <id>|all (grok, agy, opencode, claude, codex)",
        );
    }

    // Consent: --yes or dry-run skips prompt; else TTY once.
    if !opts.dry_run && !opts.yes && !confirm_install(&ids)? {
        println!("Install cancelled.");
        return Ok(());
    }

    let mut pending_only = true;
    let mut any_ready_attempt = false;

    for id in &ids {
        if id.install_ready() {
            pending_only = false;
            any_ready_attempt = true;
            let result = match id {
                HarnessId::Agy => install_agy(&home, opts.dry_run),
                HarnessId::Grok => install_grok(&home, opts.dry_run),
                _ => unreachable!("install_ready only Agy|Grok"),
            };
            match result {
                Ok(InstallOutcome::DryRun { plan }) => {
                    println!("[dry-run] {} install plan:", id.display_name());
                    println!("  hooks:   {}", plan.hooks_path.display());
                    println!("  wrapper: {}", plan.wrapper_path.display());
                    println!("  command: {}", plan.command_line);
                    if *id == HarnessId::Agy {
                        println!("  {}", f34_map_contract_summary());
                    } else {
                        println!(
                            "  {}",
                            crate::harness::install::grok_stop_stdout_contract_summary()
                        );
                    }
                    println!(
                        "  next: ai-brains harness install --harness {} --yes",
                        id.as_str()
                    );
                }
                Ok(InstallOutcome::Installed { plan }) => {
                    println!("Installed {} capture hooks:", id.display_name());
                    println!("  hooks:   {}", plan.hooks_path.display());
                    println!("  wrapper: {}", plan.wrapper_path.display());
                    println!("  next: ai-brains harness status");
                    match id {
                        HarnessId::Agy => {
                            println!("  note: message-only capture via Stop → agy-hook (F34 map)");
                        }
                        HarnessId::Grok => {
                            println!(
                                "  note: message-only capture via Stop/SessionEnd → grok-hook (empty Stop stdout)"
                            );
                        }
                        _ => {}
                    }
                }
                Ok(InstallOutcome::Refused { path, reason }) => {
                    eprintln!("Refused to rewrite {}: {}", path.display(), reason);
                    eprintln!(
                        "Fix or remove the corrupt file, then re-run: ai-brains harness install --harness {}",
                        id.as_str()
                    );
                    return Err(Box::new(governed_common::GovernedCliError::emitted(
                        governed_common::EXIT_INTERNAL,
                        reason,
                    )));
                }
                Ok(InstallOutcome::BackendPending { .. }) => {
                    // unreachable for ready backends
                }
                Err(e) => {
                    eprintln!("install failed: {e}");
                    return Err(Box::new(governed_common::GovernedCliError::emitted(
                        governed_common::EXIT_INTERNAL,
                        e,
                    )));
                }
            }
        } else {
            let out = install_pending(*id, &home, opts.dry_run);
            match out {
                InstallOutcome::DryRun { plan } => {
                    println!(
                        "[dry-run] {} targets (backend pending {}):",
                        id.as_str(),
                        id.pending_track().unwrap_or("TBD")
                    );
                    for t in crate::harness::wiring::targets_for(*id, &home) {
                        println!("  {t}");
                    }
                    if !plan.hooks_path.as_os_str().is_empty() {
                        println!("  primary: {}", plan.hooks_path.display());
                    }
                    println!(
                        "  note: real install will not claim ok until {}",
                        id.pending_track().unwrap_or("backend ready")
                    );
                }
                InstallOutcome::BackendPending { plan: _ } => {
                    println!(
                        "{}: install backend pending ({}); no files written. next: ai-brains harness status",
                        id.as_str(),
                        id.pending_track().unwrap_or("track TBD")
                    );
                }
                other => {
                    println!("{}: {:?}", id.as_str(), other);
                }
            }
        }
    }

    if pending_only && !any_ready_attempt && !opts.dry_run {
        // F14 / L7: all-pending → exit 0 + one-line summary
        println!("{}", install_pending_summary(&ids));
    }

    Ok(())
}

pub fn run_uninstall(opts: HarnessUninstallOptions) -> GovernedResult {
    let home = resolve_home().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot resolve home (USERPROFILE/HOME)",
        )
    })?;

    let ids = resolve_harness_list(opts.harness.as_deref())?;
    if ids.is_empty() {
        return governed_common::fail_usage(
            "specify --harness <id>|all (grok, agy, opencode, claude, codex)",
        );
    }

    if !opts.dry_run && !opts.yes && !confirm_uninstall(&ids)? {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    for id in &ids {
        if id.install_ready() {
            let result = match id {
                HarnessId::Agy => uninstall_agy(&home, opts.dry_run),
                HarnessId::Grok => uninstall_grok(&home, opts.dry_run),
                _ => unreachable!("install_ready only Agy|Grok"),
            };
            match result {
                Ok(UninstallOutcome::DryRun {
                    hooks_path,
                    wrapper_path,
                }) => {
                    println!("[dry-run] {} uninstall would remove:", id.display_name());
                    println!("  managed artifact {}", hooks_path.display());
                    println!("  wrapper {}", wrapper_path.display());
                }
                Ok(UninstallOutcome::Removed {
                    hooks_path,
                    wrapper_path,
                }) => {
                    println!("Uninstalled {} capture hooks:", id.display_name());
                    println!("  hooks:   {}", hooks_path.display());
                    println!("  wrapper: {}", wrapper_path.display());
                    println!("  next: ai-brains harness status");
                }
                Ok(UninstallOutcome::NothingToDo) => {
                    println!("{}: nothing to uninstall.", id.display_name());
                }
                Ok(UninstallOutcome::Refused { path, reason }) => {
                    eprintln!("Refused to rewrite {}: {}", path.display(), reason);
                    return Err(Box::new(governed_common::GovernedCliError::emitted(
                        governed_common::EXIT_INTERNAL,
                        reason,
                    )));
                }
                Ok(UninstallOutcome::BackendPending { .. }) => {}
                Err(e) => {
                    return Err(Box::new(governed_common::GovernedCliError::emitted(
                        governed_common::EXIT_INTERNAL,
                        e,
                    )));
                }
            }
        } else {
            let _ = uninstall_pending(*id);
            println!(
                "{}: no managed install writer yet ({}); nothing removed",
                id.as_str(),
                id.pending_track().unwrap_or("track TBD")
            );
        }
    }
    Ok(())
}

pub fn run_reset_decline(opts: HarnessResetDeclineOptions) -> GovernedResult {
    let home = resolve_home().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cannot resolve home (USERPROFILE/HOME)",
        )
    })?;

    let ids = match opts.harness.as_deref() {
        None | Some("all") => HARNESS_ORDER.to_vec(),
        Some(raw) => vec![parse_harness_id_or_usage(raw)?],
    };

    let mut prefs = load_prefs(&home);
    for id in ids {
        prefs.clear_declined(id);
    }
    save_prefs(&home, &prefs).map_err(|e| {
        Box::new(governed_common::GovernedCliError::emitted(
            governed_common::EXIT_INTERNAL,
            e,
        )) as Box<dyn std::error::Error>
    })?;
    println!("Cleared harness decline prefs. next: ai-brains preflight --summary");
    Ok(())
}

fn resolve_harness_list(raw: Option<&str>) -> Result<Vec<HarnessId>, Box<dyn std::error::Error>> {
    match raw {
        None => Ok(vec![]),
        Some("all") => Ok(HARNESS_ORDER.to_vec()),
        Some(s) => Ok(vec![parse_harness_id_or_usage(s)?]),
    }
}

fn parse_harness_id_or_usage(raw: &str) -> Result<HarnessId, Box<dyn std::error::Error>> {
    match parse_harness_id(raw) {
        Ok(id) => Ok(id),
        Err(msg) => {
            // AC8: exit 2
            Err(Box::new(governed_common::GovernedCliError::emitted(
                governed_common::EXIT_USAGE,
                {
                    eprintln!("{msg}");
                    msg
                },
            )))
        }
    }
}

fn confirm_install(ids: &[HarnessId]) -> Result<bool, Box<dyn std::error::Error>> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("Non-interactive install requires --yes (or use --dry-run).");
        return Ok(false);
    }
    let list = ids
        .iter()
        .map(|i| i.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    eprint!("Install capture hooks for {list}? [Y/n] ");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let t = line.trim();
    Ok(t.is_empty() || t.eq_ignore_ascii_case("y") || t.eq_ignore_ascii_case("yes"))
}

fn confirm_uninstall(ids: &[HarnessId]) -> Result<bool, Box<dyn std::error::Error>> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("Non-interactive uninstall requires --yes (or use --dry-run).");
        return Ok(false);
    }
    let list = ids
        .iter()
        .map(|i| i.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    eprint!("Uninstall capture hooks for {list}? [Y/n] ");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let t = line.trim();
    Ok(t.is_empty() || t.eq_ignore_ascii_case("y") || t.eq_ignore_ascii_case("yes"))
}

/// Injectable consent ask for unit tests (AC6 / F24 / F25).
///
/// Gate order:
/// 1. No ready-missing / declined → Skip
/// 2. `--no-hook-prompt` / `--stdin` → PrintNextActionOnly (never auto, never ask)
/// 3. `auto_install: true` → AutoInstall (works non-TTY; F25)
/// 4. non-TTY → PrintNextActionOnly
/// 5. else → AskOnce
pub fn should_prompt_install(
    is_tty: bool,
    no_hook_prompt: bool,
    stdin_mode: bool,
    has_ready_missing: bool,
    declined: bool,
    auto_install: bool,
) -> PromptDecision {
    if !has_ready_missing {
        return PromptDecision::Skip;
    }
    if declined {
        return PromptDecision::Skip;
    }
    // Explicit never-prompt flags win over auto_install (CI / stdin payload safety).
    if no_hook_prompt || stdin_mode {
        return PromptDecision::PrintNextActionOnly;
    }
    // F25: auto_install installs ready backends without a TTY.
    if auto_install {
        return PromptDecision::AutoInstall;
    }
    if !is_tty {
        return PromptDecision::PrintNextActionOnly;
    }
    PromptDecision::AskOnce
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDecision {
    Skip,
    PrintNextActionOnly,
    AskOnce,
    AutoInstall,
}

/// Apply a yes/no answer from the user (testable).
pub fn interpret_consent_answer(answer: &str) -> bool {
    let t = answer.trim();
    t.is_empty() || t.eq_ignore_ascii_case("y") || t.eq_ignore_ascii_case("yes")
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use crate::harness::detect::{HarnessPresence, detect_all_with};
    use crate::harness::{save_prefs, wiring::collect_status_from_presence};
    use ai_brains_core::temp_env::TempEnv;
    use tempfile::tempdir;

    #[test]
    fn parse_unknown_harness__usage_exit_code() {
        let err = parse_harness_id_or_usage("foo").expect_err("must fail");
        let g = err
            .downcast_ref::<governed_common::GovernedCliError>()
            .expect("GovernedCliError");
        assert_eq!(g.exit_code, governed_common::EXIT_USAGE);
    }

    #[test]
    fn should_prompt__never_when_no_hook_prompt_or_stdin() {
        assert_eq!(
            should_prompt_install(true, true, false, true, false, false),
            PromptDecision::PrintNextActionOnly
        );
        assert_eq!(
            should_prompt_install(true, false, true, true, false, false),
            PromptDecision::PrintNextActionOnly
        );
        assert_eq!(
            should_prompt_install(false, false, false, true, false, false),
            PromptDecision::PrintNextActionOnly
        );
    }

    #[test]
    fn should_prompt__non_tty_auto_install__auto() {
        // F25: auto_install must work without a TTY.
        assert_eq!(
            should_prompt_install(false, false, false, true, false, true),
            PromptDecision::AutoInstall
        );
        // Explicit never-prompt flags still win over auto_install.
        assert_eq!(
            should_prompt_install(false, true, false, true, false, true),
            PromptDecision::PrintNextActionOnly
        );
        assert_eq!(
            should_prompt_install(false, false, true, true, false, true),
            PromptDecision::PrintNextActionOnly
        );
    }

    #[test]
    fn should_prompt__tty_missing_ready__ask_once() {
        assert_eq!(
            should_prompt_install(true, false, false, true, false, false),
            PromptDecision::AskOnce
        );
    }

    #[test]
    fn should_prompt__declined__skip() {
        assert_eq!(
            should_prompt_install(true, false, false, true, true, false),
            PromptDecision::Skip
        );
    }

    #[test]
    fn interpret_consent__decline_persists() {
        // AC6: decline path unit-tested with injectable answer
        assert!(!interpret_consent_answer("n"));
        assert!(!interpret_consent_answer("N"));
        assert!(interpret_consent_answer(""));
        assert!(interpret_consent_answer("y"));

        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        let mut prefs = load_prefs(home);
        assert!(!prefs.is_declined(HarnessId::Agy));
        if !interpret_consent_answer("n") {
            prefs.mark_declined(HarnessId::Agy, "2026-01-01T00:00:00Z");
        }
        save_prefs(home, &prefs).expect("save");
        let prefs2 = load_prefs(home);
        assert!(prefs2.is_declined(HarnessId::Agy));
        // Second run: declined → no prompt
        assert_eq!(
            should_prompt_install(
                true,
                false,
                false,
                true,
                prefs2.is_declined(HarnessId::Agy),
                false
            ),
            PromptDecision::Skip
        );
    }

    #[test]
    fn status_report__targets_under_home() {
        let dir = tempdir().expect("tempdir");
        let home = dir.path();
        std::fs::create_dir_all(home.join(".grok")).expect("mkdir");
        let _path = TempEnv::set("PATH", "");
        let presence = vec![HarnessPresence {
            id: HarnessId::Grok,
            present: true,
            binary: None,
            home_path: Some(home.join(".grok").display().to_string()),
        }];
        // Full report via collect
        let report = collect_status_from_presence(home, &detect_all_with(Some(home)));
        for h in &report.harnesses {
            for t in &h.targets {
                assert!(
                    PathUnder::is_under(home, std::path::Path::new(t))
                        || t.contains(&home.display().to_string()),
                    "target {t} not under {}",
                    home.display()
                );
            }
        }
        let _ = presence;
    }

    struct PathUnder;
    impl PathUnder {
        fn is_under(home: &std::path::Path, target: &std::path::Path) -> bool {
            target.starts_with(home)
        }
    }
}
