#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T220 — Preflight summary JSON honesty hermetic suite (AC1–AC5, AC7–AC8b, AC13–AC15).
//!
//! Pattern: tempdir vault + multi-project register/pin; hermetic_bin + isolate_empty_home.

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

/// Register a project via `context` in `work_dir` (writes `.env` there).
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

fn pin_memory_tagged(vault: &Path, work_dir: &Path, project_id: &str, content: &str, tag: &str) {
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
        .arg("--tag")
        .arg(tag)
        .assert()
        .success();
}

/// Run `preflight` with arbitrary subcommand flags after the `preflight` token.
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

/// Parse stdout as exactly one JSON object (AC15).
fn parse_summary_json(stdout: &str) -> serde_json::Value {
    let trimmed = stdout.trim();
    assert!(
        !trimmed.is_empty(),
        "stdout must be non-empty JSON; got empty"
    );
    assert!(
        !trimmed.contains("--- AI-Brains Preflight Summary ---"),
        "AC1: must not contain human banner; got:\n{stdout}"
    );
    let v: serde_json::Value = serde_json::from_str(trimmed)
        .unwrap_or_else(|e| panic!("AC15 single JSON object: {e}; stdout:\n{stdout}"));
    assert!(v.is_object(), "must be object; got: {v}");
    v
}

// ---------------------------------------------------------------------------
// AC1 / AC2 / AC15 — summary JSON purity + required keys
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__format_json__pure_object_no_banner() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--summary", "--format", "json"], None);
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    let v = parse_summary_json(&stdout);
    // AC2 required keys
    assert_eq!(v["api_version"], "1");
    assert!(
        v.get("sections").is_none(),
        "AC10: summary JSON has no sections key"
    );
    assert!(v.get("scope").is_some(), "scope key");
    assert!(
        v.get("project_id").is_some(),
        "project_id key (may be null)"
    );
    assert!(v.get("pinned").is_some());
    assert!(v.get("active_sessions").is_some());
    assert!(v.get("in_context_hotspots").is_some());
    assert!(v.get("in_context_decisions").is_some());
    assert!(v.get("in_context_constraints").is_some());
    assert!(v.get("word_count").is_some());
}

// ---------------------------------------------------------------------------
// AC3 — multi-project global
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__global_multi_project__scope_global_and_projects() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let proj_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "DECISION: use dual vault/in-context counts",
    );
    pin_memory(
        &vault,
        &proj_b,
        &id_b,
        "DECISION: Scope vocabulary is T207 SOOT",
    );

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--global", "--summary", "--format", "json"],
        Some(&id_a),
    );
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_summary_json(&stdout);
    assert_eq!(v["scope"], "global");
    assert!(v["project_id"].is_null(), "global project_id null; got {v}");
    let projects = v["projects"].as_u64().expect("projects key under global");
    assert!(
        projects >= 2,
        "AC3: projects >= 2; got {projects}\n{stdout}"
    );
    let pinned = v["pinned"].as_u64().expect("pinned");
    assert!(pinned >= 2, "AC3: pinned >= 2; got {pinned}\n{stdout}");
}

// ---------------------------------------------------------------------------
// AC4 — project-scoped: no projects key
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__project_scoped__no_projects_key() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("proj-a");
    let proj_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "DECISION: only A pin for scoped test",
    );
    pin_memory(&vault, &proj_a, &id_a, "CONSTRAINT: A constraint");
    pin_memory(&vault, &proj_b, &id_b, "DECISION: B only");

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--summary", "--format", "json"], Some(&id_a));
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_summary_json(&stdout);
    assert_eq!(v["scope"], "project");
    assert_eq!(
        v["project_id"].as_str(),
        Some(id_a.as_str()),
        "project_id must match fixture"
    );
    assert!(
        v.get("projects").is_none(),
        "AC4: no projects key under project scope; got {v}"
    );
}

// ---------------------------------------------------------------------------
// AC5 — legacy-path markers yield meaningful in_context counts
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__legacy_markers__in_context_counts_meaningful() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-markers");
    let id = register_project(&vault, &proj);
    // Legacy pin content with DECISION:/CONSTRAINT: markers (not governed-only).
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: marker scan SOOT for summary JSON",
    );
    pin_memory(&vault, &proj, &id, "CONSTRAINT: keep legacy path for AC5");
    pin_memory(&vault, &proj, &id, "HOTSPOT: temporary risk for markers");

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--summary", "--format", "json"], Some(&id));
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_summary_json(&stdout);
    let decisions = v["in_context_decisions"].as_u64().unwrap_or(0);
    let constraints = v["in_context_constraints"].as_u64().unwrap_or(0);
    let hotspots = v["in_context_hotspots"].as_u64().unwrap_or(0);
    // Legacy pins each carry one marker type; budget window must surface each (SOOT).
    assert!(
        decisions >= 1,
        "AC5: in_context_decisions >= 1 from DECISION: pin; got {decisions}\n{stdout}"
    );
    assert!(
        constraints >= 1,
        "AC5: in_context_constraints >= 1 from CONSTRAINT: pin; got {constraints}\n{stdout}"
    );
    assert!(
        hotspots >= 1,
        "AC5: in_context_hotspots >= 1 from HOTSPOT: pin; got {hotspots}\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC7 — human summary still has banner (regression)
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__human_summary__banner_unchanged() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--summary"], None);
    assert_eq!(code, 0, "stderr={stderr}");
    assert!(
        stdout.contains("--- AI-Brains Preflight Summary ---"),
        "AC7: human banner required; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC8b — install-hooks × JSON: stdout pure JSON
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__install_hooks__stdout_pure_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--summary", "--format", "json", "--install-hooks"],
        None,
    );
    // Exit 0 or honest non-zero on refuse — either is fine; stdout must stay pure JSON.
    assert!(
        code == 0 || code == 1,
        "AC8b: exit 0 or 1; code={code}; stderr={stderr}"
    );
    // AC8b: install path must not be a silent no-op — status on stderr (empty-home / no ready harness).
    assert!(
        !stderr.trim().is_empty()
            || stderr.contains("install")
            || stderr.contains("harness")
            || stderr.contains("No ready"),
        "AC8b: stderr must carry install-hooks outcome (not silent); stderr={stderr:?}"
    );
    if code == 0 {
        let v = parse_summary_json(&stdout);
        assert_eq!(v["api_version"], "1");
        // Banner must never leak onto stdout even when install chatter is present on stderr.
        assert!(
            !stdout.contains("--- AI-Brains Preflight Summary ---"),
            "AC8b: no banner on stdout; stdout:\n{stdout}"
        );
    } else {
        // On hard refuse, may still have partial stdout; if non-empty, must not be banner.
        assert!(
            !stdout.contains("--- AI-Brains Preflight Summary ---"),
            "AC8b: no banner on refuse path; stdout:\n{stdout}"
        );
        if !stdout.trim().is_empty() {
            let _ = parse_summary_json(&stdout);
        }
    }
}

// ---------------------------------------------------------------------------
// AC8b present-harness path — OpenCode home → real install + stderr status
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__install_hooks_present_opencode__stdout_pure_stderr_install() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Present OpenCode config root (no managed plugin yet) → install_ready + Missing wiring.
    let home = dir.path().join("home");
    fs::create_dir_all(home.join(".config").join("opencode")).expect("opencode config dir");

    let mut cmd = hermetic();
    // Override empty-home isolation with a home that has a present installable harness.
    cmd.env("USERPROFILE", &home);
    cmd.env("HOME", &home);
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("preflight")
        .arg("--summary")
        .arg("--format")
        .arg("json")
        .arg("--install-hooks");
    let out = cmd.output().expect("preflight");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    assert_eq!(
        code, 0,
        "present-harness install-hooks exit 0; stderr={stderr}"
    );
    let v = parse_summary_json(&stdout);
    assert_eq!(v["api_version"], "1");
    // No human harness sibling on stdout under JSON.
    assert!(
        !stdout.contains("Harnesses installed on machine:"),
        "AC8: harness human block must not appear on stdout; got:\n{stdout}"
    );
    // Real install path: status on stderr (Installed… or already installed).
    let stderr_l = stderr.to_ascii_lowercase();
    assert!(
        stderr_l.contains("install") || stderr_l.contains("opencode") || stderr_l.contains("hooks"),
        "AC8b present path: stderr install outcome; stderr={stderr}"
    );
    // Plugin file written under test home (side effect of --install-hooks).
    let plugin = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("ai-brains-capture.js");
    assert!(
        plugin.is_file(),
        "AC8b: --install-hooks must install OpenCode plugin; missing {plugin:?}"
    );
}

// ---------------------------------------------------------------------------
// AC13 — uppercase JSON format
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__format_JSON_uppercase__takes_json_path() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--summary", "--format", "JSON"], None);
    assert_eq!(code, 0, "AC13 exit 0; stderr={stderr}");
    let v = parse_summary_json(&stdout);
    assert_eq!(v["api_version"], "1");
    assert!(
        !stdout.contains("--- AI-Brains Preflight Summary ---"),
        "uppercase JSON must not fall back to human"
    );
}

// ---------------------------------------------------------------------------
// AC14 — scope none when no global and no project
// ---------------------------------------------------------------------------

#[test]
fn preflight_summary_json__no_project_no_global__scope_none() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic strips AI_BRAINS_PROJECT_ID; --no-project-context already set.
    let (code, stdout, stderr) = run_preflight(&vault, &["--summary", "--format", "json"], None);
    assert_eq!(code, 0, "stderr={stderr}");
    let v = parse_summary_json(&stdout);
    assert_eq!(v["scope"], "none", "AC14 scope none; got {v}");
    assert!(v["project_id"].is_null());
    assert!(
        v.get("projects").is_none(),
        "AC14: no projects key under none; got {v}"
    );
}

/// T286 AC6 — tagged TAGS envelope pin still yields in_context_decisions >= 1.
#[test]
fn preflight__summary_json_tagged_pin__in_context_decisions_nonzero() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-t286-summary");
    let id = register_project(&vault, &proj);
    let needle = format!("T286-summary-needle-{}", uuid::Uuid::new_v4());
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {needle} tagged pin must enter the summary window"),
        "t286",
    );
    // Stop the pin session so its DECISION: turn is not in the active Session section
    // (otherwise the T220 substring scan already counts 1 and AC6 is not red).
    let env_path = proj.join(".env");
    let env_content = fs::read_to_string(&env_path).expect(".env after tagged pin");
    let mut pin_session = String::new();
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            pin_session = rest.trim().to_string();
        }
    }
    assert!(!pin_session.is_empty(), "SESSION_ID after tagged pin");
    hermetic()
        .current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .env("AI_BRAINS_SESSION_ID", &pin_session)
        .arg("stop-session")
        .arg(&pin_session)
        .assert()
        .success();
    hermetic()
        .current_dir(&proj)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &id)
        .arg("context")
        .arg("--new-session")
        .assert()
        .success();
    // Three newer dumps fill Recent (cap 3) so the pin is Index-only after green.
    for i in 0..3 {
        pin_memory(
            &vault,
            &proj,
            &id,
            &format!("## Objective\nNewer dump {i} padding word padding word padding word"),
        );
    }

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--summary", "--format", "json"], Some(&id));
    assert_eq!(code, 0, "AC6 exit 0; stderr={stderr}");
    let v = parse_summary_json(&stdout);
    assert_eq!(v["api_version"], "1");
    assert!(v.get("pinned").is_some(), "T220 pinned key");
    assert!(
        v.get("in_context_decisions").is_some(),
        "T220 in_context_decisions key"
    );
    assert!(
        v.get("index_kind").is_none() && v.get("in_context_authority").is_none(),
        "AC14: no extra required keys; got {v}"
    );
    let decisions = v["in_context_decisions"].as_u64().unwrap_or(0);
    assert!(
        decisions >= 1,
        "AC6: tagged pin must yield in_context_decisions >= 1; got {decisions}\n{stdout}"
    );
}
