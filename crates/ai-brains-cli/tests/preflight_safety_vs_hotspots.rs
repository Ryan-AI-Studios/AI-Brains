#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T279 — Preflight Safety vs live hotspots hermetics (AC3 / AC4 / AC7 / AC11 / AC12).

mod common;

use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn hermetic() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = fs::read_to_string(&env_path).expect(".env after context");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty(), "empty project id in .env");
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from .env after context: {content}");
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    let env_path = work_dir.join(".env");
    let env_content = fs::read_to_string(&env_path).expect(".env for pin");
    let mut session_id = String::new();
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            session_id = rest.trim().to_string();
        }
    }
    assert!(!session_id.is_empty(), "SESSION_ID missing from .env");

    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .env("AI_BRAINS_SESSION_ID", &session_id)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn run_preflight(
    vault: &Path,
    preflight_args: &[&str],
    project_env: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault);
    if let Some(pid) = project_env {
        cmd.env("AI_BRAINS_PROJECT_ID", pid);
    }
    cmd.arg("preflight");
    for a in preflight_args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("preflight");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

fn safety_section(stdout: &str) -> String {
    let mut out = Vec::new();
    let mut in_safety = false;
    for line in stdout.lines() {
        let t = line.trim();
        if t.contains("Repository Bearings") {
            in_safety = true;
            continue;
        }
        if in_safety && t.starts_with("--- ") {
            break;
        }
        if in_safety {
            out.push(line);
        }
    }
    out.join("\n")
}

#[test]
fn preflight__buried_constraint_dump__not_in_safety() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279");
    let id = register_project(&vault, &proj);
    let needle = format!("T279-bearing-needle-{}", uuid::Uuid::new_v4());
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("CONSTRAINT: {needle} must stay in Safety under leading GLOB"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        "## Objective\n- Complete the review-track dump.\nCONSTRAINT: buried-must-not-steal-safety",
    );

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--pretty", "--no-hook-prompt", "-m", "400"],
        Some(&id),
    );
    assert_eq!(code, 0, "AC3 exit 0; stderr={stderr}");
    let safety = safety_section(&stdout);
    assert!(
        safety.contains(&needle),
        "AC3: leading CONSTRAINT needle in Safety; safety:\n{safety}\nfull:\n{stdout}"
    );
    assert!(
        !safety.contains("## Objective"),
        "AC3: dump heading must not steal Safety; safety:\n{safety}\nfull:\n{stdout}"
    );
}

#[test]
fn preflight__no_bearings__emits_safety_sync_remediator() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279-empty");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: t279-empty-safety-fixture is not a Safety marker",
    );

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--pretty", "--no-hook-prompt", "-m", "400"],
        Some(&id),
    );
    assert_eq!(code, 0, "AC4 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("--- Repository Bearings & Safety ---"),
        "AC4: always emit Safety header; got:\n{stdout}"
    );
    assert!(
        stdout.contains("safety sync --dry-run"),
        "AC4: honest empty names dry-run; got:\n{stdout}"
    );
    let safety = safety_section(&stdout);
    assert!(
        !safety.contains("## Objective"),
        "AC4: Safety is not a dump; safety:\n{safety}"
    );
}

#[test]
fn preflight__compact_json__required_keys_frozen() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279-json");
    let id = register_project(&vault, &proj);

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--format", "json", "--no-hook-prompt", "--compact"],
        Some(&id),
    );
    assert_eq!(code, 0, "AC7 exit 0; stderr={stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("AC7 parse: {e}; {stdout}"));
    let obj = v.as_object().expect("AC7 object");
    assert!(obj.contains_key("text"), "AC7 text");
    assert!(obj.contains_key("word_count"), "AC7 word_count");
    assert!(
        !obj.contains_key("hotspots"),
        "AC7: no new hotspots[] key; keys={:?}",
        obj.keys().collect::<Vec<_>>()
    );
}

#[test]
fn preflight__global_pretty__no_cwd_project_rs_inject() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279-global");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "CONSTRAINT: t279-global-bearing stays vault-only",
    );

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--global", "--pretty", "--no-hook-prompt", "-m", "400"],
        Some(&id),
    );
    assert_eq!(code, 0, "AC11 exit 0; stderr={stderr}");
    let safety = safety_section(&stdout);
    assert!(
        !safety.contains("project.rs"),
        "AC11: --global must not live-inject cwd project.rs; safety:\n{safety}"
    );
}

#[test]
fn preflight__summary_after_bearing__in_context_constraints_ge_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279-summary");
    let id = register_project(&vault, &proj);
    let needle = format!("T279-bearing-needle-{}", uuid::Uuid::new_v4());
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("CONSTRAINT: {needle} must count in summary"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        "## Objective\nCONSTRAINT: buried-summary-dump",
    );

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--summary", "--no-hook-prompt"], Some(&id));
    assert_eq!(code, 0, "AC12 exit 0; stderr={stderr}");
    let mut constraints: Option<u64> = None;
    let mut hotspots: Option<u64> = None;
    for line in stdout.lines() {
        if let Some(rest) = line
            .trim()
            .strip_prefix("In context constraints:")
            .or_else(|| line.trim().strip_prefix("In context Constraints:"))
        {
            constraints = rest.trim().parse().ok();
        }
        if let Some(rest) = line.trim().strip_prefix("In context hotspots:") {
            hotspots = rest.trim().parse().ok();
        }
    }
    assert!(
        constraints.unwrap_or(0) >= 1,
        "AC12: in_context_constraints >= 1; got:\n{stdout}"
    );
    let _ = hotspots;
}

#[test]
fn preflight__empty_remediator__does_not_bump_hotspots() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj-t279-empty-count");
    let id = register_project(&vault, &proj);

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--summary", "--no-hook-prompt"], Some(&id));
    assert_eq!(code, 0, "AC12 empty exit 0; stderr={stderr}");
    let mut hotspots: Option<u64> = None;
    for line in stdout.lines() {
        if let Some(rest) = line.trim().strip_prefix("In context hotspots:") {
            hotspots = rest.trim().parse().ok();
        }
    }
    assert_eq!(
        hotspots.unwrap_or(99),
        0,
        "AC12: empty remediator must not bump in_context_hotspots; got:\n{stdout}"
    );
}
