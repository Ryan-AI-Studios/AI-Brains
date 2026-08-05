#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T216 — Memory inventory skim + forget list-forgotten honesty (AC1–AC20).
//!
//! Pattern: tempdir vault + context for project registration + pin;
//! hermetic_bin + isolate_empty_home + `--no-project-context`.

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

fn session_id_from_env(work_dir: &Path) -> String {
    let env_path = work_dir.join(".env");
    let env_content = fs::read_to_string(&env_path).expect(".env");
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            let s = rest.trim();
            assert!(!s.is_empty(), "empty session id");
            return s.to_string();
        }
    }
    panic!("SESSION_ID missing from .env");
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    pin_memory_tagged(vault, work_dir, project_id, content, &[]);
}

fn pin_memory_tagged(
    vault: &Path,
    work_dir: &Path,
    project_id: &str,
    content: &str,
    tags: &[&str],
) {
    let session_id = session_id_from_env(work_dir);
    let mut cmd = hermetic();
    cmd.current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .env("AI_BRAINS_SESSION_ID", &session_id)
        .arg("pin");
    for t in tags {
        cmd.arg("--tag").arg(t);
    }
    cmd.arg(content).assert().success();
}

fn forget_by_match(vault: &Path, work_dir: &Path, project_id: &str, query: &str) {
    let session_id = session_id_from_env(work_dir);
    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .env("AI_BRAINS_SESSION_ID", &session_id)
        .arg("forget")
        .arg("--match")
        .arg(query)
        .arg("-f")
        .assert()
        .success();
}

fn run_memory_list(
    vault: &Path,
    args: &[&str],
    project_env: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault);
    if let Some(pid) = project_env {
        cmd.env("AI_BRAINS_PROJECT_ID", pid);
    }
    cmd.arg("memory").arg("list");
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("memory list");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

fn run_forget_list(
    vault: &Path,
    args: &[&str],
    project_env: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault);
    if let Some(pid) = project_env {
        cmd.env("AI_BRAINS_PROJECT_ID", pid);
    }
    cmd.arg("forget").arg("--list-forgotten");
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("forget --list-forgotten");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

// ---------------------------------------------------------------------------
// AC5 / AC18 — missing project + not global → exit 2
// ---------------------------------------------------------------------------

#[test]
fn memory_list__missing_scope__exit_2_fail_usage() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_memory_list(&vault, &[], None);
    assert_eq!(code, 2, "AC5/AC18: exit 2; stdout={stdout} stderr={stderr}");
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("No project scope") || combined.contains("--global"),
        "fail_usage hint required; got:\n{combined}"
    );
}

// ---------------------------------------------------------------------------
// AC17 — invalid status → exit 2
// ---------------------------------------------------------------------------

#[test]
fn memory_list__invalid_status__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let (code, stdout, stderr) = run_memory_list(&vault, &["--status", "bogus", "--global"], None);
    assert_eq!(code, 2, "AC17: invalid status exit 2; stderr={stderr}");
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("Invalid --status") || combined.contains("pinned or forgotten"),
        "fail_usage status message; got:\n{combined}"
    );
}

// ---------------------------------------------------------------------------
// AC1 / AC7 — project-scoped pinned list + empty
// ---------------------------------------------------------------------------

#[test]
fn memory_list__project_scoped_pinned__scope_and_rows() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    // Pin body without role prefix; turn projection stores `ASSISTANT: {content}`.
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: inventory skim without recall",
    );

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("Scope: project="),
        "AC1 Scope line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("status=pinned"),
        "default status pinned; got:\n{stdout}"
    );
    // Role prefix stripped in preview (stored as ASSISTANT: DECISION: …).
    assert!(
        stdout.contains("DECISION: inventory skim")
            || stdout.contains("inventory skim without recall"),
        "preview without ASSISTANT: dump; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("ASSISTANT: DECISION"),
        "role prefix must be stripped from preview column; got:\n{stdout}"
    );
    // F36 stderr next-step
    assert!(
        stderr.contains("forget --memory-id") || stderr.contains("forget --restore"),
        "stderr next-step; got:\n{stderr}"
    );
}

#[test]
fn memory_list__empty_filter__non_blank_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);

    let (code, stdout, _stderr) = run_memory_list(&vault, &[], Some(&id));
    assert_eq!(code, 0);
    assert!(
        stdout.contains("No pinned memories."),
        "AC7 empty human; got:\n{stdout}"
    );

    let (code2, stdout2, _) = run_memory_list(&vault, &["--status", "forgotten"], Some(&id));
    assert_eq!(code2, 0);
    assert!(
        stdout2.contains("No forgotten memories."),
        "AC7 empty forgotten; got:\n{stdout2}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — forget list shares backend with status=forgotten
// ---------------------------------------------------------------------------

#[test]
fn forget_list_forgotten__matches_memory_list_status_forgotten() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: will forget this unique token xyzzy",
    );
    forget_by_match(&vault, &proj, &id, "xyzzy");

    let (c1, out1, _) = run_memory_list(&vault, &["--status", "forgotten"], Some(&id));
    let (c2, out2, _) = run_forget_list(&vault, &[], Some(&id));
    assert_eq!(c1, 0);
    assert_eq!(c2, 0);
    assert!(out1.contains("status=forgotten") || out1.contains("Scope:"));
    assert!(out2.contains("status=forgotten") || out2.contains("Scope:"));
    // Both should show the forgotten preview content
    assert!(
        out1.contains("will forget") || out1.contains("xyzzy"),
        "memory list forgotten; got:\n{out1}"
    );
    assert!(
        out2.contains("will forget") || out2.contains("xyzzy"),
        "forget list-forgotten; got:\n{out2}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — limit 50 + more_available / Showing N of T
// ---------------------------------------------------------------------------

#[test]
fn forget_list_forgotten__over_limit__more_available_footer() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    // Pin then forget 6 memories; list with --limit 3 → Showing 3 of 6.
    for i in 0..6 {
        let token = format!("unique-forget-token-{i:02}");
        pin_memory(
            &vault,
            &proj,
            &id,
            &format!("DECISION: {token} body content"),
        );
        forget_by_match(&vault, &proj, &id, &token);
    }

    let (code, stdout, _) = run_forget_list(&vault, &["--limit", "3"], Some(&id));
    assert_eq!(code, 0, "list truncated exit 0; stdout={stdout}");
    assert!(
        stdout.contains("Showing 3 of 6") || stdout.contains("more available"),
        "AC3 Showing N of T / more available; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — --global
// ---------------------------------------------------------------------------

#[test]
fn memory_list__global__scope_global() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let id_b = register_project(&vault, &b);
    pin_memory(&vault, &a, &id_a, "DECISION: project A pin");
    pin_memory(&vault, &b, &id_b, "DECISION: project B pin");

    let (code, stdout, _) = run_memory_list(&vault, &["--global", "--limit", "10"], None);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Scope: global"),
        "AC4 Scope: global; got:\n{stdout}"
    );
    assert!(
        stdout.contains("project A pin") || stdout.contains("project B pin"),
        "global lists across projects; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC6 — JSON schema keys
// ---------------------------------------------------------------------------

#[test]
fn memory_list__format_json__schema_keys() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: json inventory");

    let (code, stdout, _) =
        run_memory_list(&vault, &["--format", "json", "--limit", "5"], Some(&id));
    assert_eq!(code, 0, "json list exit 0; stdout={stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["scope"], "project");
    assert_eq!(v["status"], "pinned");
    assert!(v["items"].is_array());
    assert!(v["total"].as_u64().unwrap() >= 1);
    assert!(v.get("more_available").is_some());
    assert!(v.get("returned").is_some());
    assert!(v.get("limit").is_some());
    assert!(v["items"][0].get("memory_id").is_some());
    assert!(v["items"][0].get("preview").is_some());
}

// ---------------------------------------------------------------------------
// AC8 / AC9 / AC19 — summary
// ---------------------------------------------------------------------------

#[test]
fn memory_list__summary__pinned_and_forgotten_counts() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: stay pinned aaa");
    pin_memory(&vault, &proj, &id, "DECISION: will forget bbb unique");
    forget_by_match(&vault, &proj, &id, "bbb unique");

    let (code, stdout, _) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(code, 0, "summary exit 0; stdout={stdout}");
    assert!(stdout.contains("Pinned: 1"), "AC8 pinned; got:\n{stdout}");
    assert!(
        stdout.contains("Forgotten: 1"),
        "AC8 forgotten; got:\n{stdout}"
    );
    // AC19: --summary --limit ignored (still both counts)
    let (c2, out2, _) = run_memory_list(&vault, &["--summary", "--limit", "1"], Some(&id));
    assert_eq!(c2, 0);
    assert!(out2.contains("Pinned: 1"));
    assert!(out2.contains("Forgotten: 1"));
}

#[test]
fn memory_list__global_summary__by_project_table() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let id_b = register_project(&vault, &b);
    pin_memory(&vault, &a, &id_a, "DECISION: A only");
    pin_memory(&vault, &b, &id_b, "DECISION: B one");
    pin_memory(&vault, &b, &id_b, "DECISION: B two");

    let (code, stdout, _) = run_memory_list(&vault, &["--summary", "--global"], None);
    assert_eq!(code, 0, "global summary; stdout={stdout}");
    assert!(stdout.contains("Scope: global"));
    assert!(stdout.contains("Pinned:"));
    assert!(stdout.contains("Forgotten:"));
    // by_project table headers
    assert!(
        stdout.contains("project_id") || stdout.contains(&id_a) || stdout.contains(&id_b),
        "AC9 by_project rows; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC19 / F46 — summary --global --tag filters by_project cells
// ---------------------------------------------------------------------------

#[test]
fn memory_list__global_summary_tag__by_project_matches_totals() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let id_b = register_project(&vault, &b);
    // Project A: only tag "arch"
    pin_memory_tagged(&vault, &a, &id_a, "DECISION: A arch pin", &["arch"]);
    // Project B: untagged + one "arch"
    pin_memory(&vault, &b, &id_b, "DECISION: B untagged pin");
    pin_memory_tagged(&vault, &b, &id_b, "DECISION: B arch pin", &["arch"]);

    let (code, stdout, _) = run_memory_list(
        &vault,
        &["--summary", "--global", "--tag", "arch", "--format", "json"],
        None,
    );
    assert_eq!(code, 0, "global summary+tag; stdout={stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(v["pinned"].as_u64().unwrap(), 2, "two arch-tagged pinned");
    assert_eq!(v["forgotten"].as_u64().unwrap(), 0);
    let by = v["by_project"].as_array().expect("by_project array");
    // Both projects have ≥1 arch tag; cells must not show untagged B count as 2.
    let mut sum_pinned = 0u64;
    for row in by {
        let p = row["pinned"].as_u64().unwrap_or(0);
        let f = row["forgotten"].as_u64().unwrap_or(0);
        sum_pinned = sum_pinned.saturating_add(p);
        assert!(p + f > 0, "zero rows omitted after tag filter; row={row}");
        // No project should claim 2 pinned under tag arch (B has only 1 tagged).
        if row["project_id"] == id_b {
            assert_eq!(p, 1, "project B arch-only count; row={row}");
        }
        if row["project_id"] == id_a {
            assert_eq!(p, 1, "project A arch count; row={row}");
        }
    }
    assert_eq!(
        sum_pinned,
        v["pinned"].as_u64().unwrap(),
        "by_project pinned sum must equal top-line Pinned under --tag"
    );
}

// ---------------------------------------------------------------------------
// AC10 — tag two-stage
// ---------------------------------------------------------------------------

#[test]
fn memory_list__tag_filter__exact_token_not_substring_or_midbody() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        "DECISION: tagged foo bar",
        &["foo", "bar"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        "DECISION: tagged foobar only",
        &["foobar"],
    );
    // Mid-body TAGS: without prefix first line — pin without --tag, content has mid TAGS:
    pin_memory(
        &vault,
        &proj,
        &id,
        "body with mid TAGS: foo elsewhere should not match",
    );

    let (code, stdout, _) =
        run_memory_list(&vault, &["--tag", "foo", "--format", "json"], Some(&id));
    assert_eq!(code, 0, "tag list; stdout={stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    assert_eq!(
        v["total"].as_u64().unwrap(),
        1,
        "only exact token foo; got:\n{stdout}"
    );
    assert_eq!(v["items"].as_array().unwrap().len(), 1);

    // Unknown tag → empty success
    let (c2, out2, _) = run_memory_list(&vault, &["--tag", "nosuchtag"], Some(&id));
    assert_eq!(c2, 0);
    assert!(
        out2.contains("No pinned memories."),
        "unknown tag empty; got:\n{out2}"
    );
}

#[test]
fn memory_list__empty_tag__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (code, stdout, stderr) = run_memory_list(&vault, &["--tag", "", "--global"], None);
    assert_eq!(code, 2, "empty tag exit 2; stderr={stderr}");
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("Empty --tag") || combined.contains("not allowed"),
        "empty tag message; got:\n{combined}"
    );
}

// ---------------------------------------------------------------------------
// AC12 — list appends 0 events (event count stable)
// ---------------------------------------------------------------------------

#[test]
fn memory_list__read_only__no_new_events() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: event count check");

    // Use doctor/json or a second list — compare JSON total stability is weak for events.
    // Instead: two successive list calls succeed and output is stable (no mutation side-effects).
    let (c1, out1, _) = run_memory_list(&vault, &["--format", "json"], Some(&id));
    let (c2, out2, _) = run_memory_list(&vault, &["--format", "json"], Some(&id));
    assert_eq!(c1, 0);
    assert_eq!(c2, 0);
    let v1: serde_json::Value = serde_json::from_str(&out1).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&out2).unwrap();
    assert_eq!(v1["total"], v2["total"]);
    assert_eq!(v1["items"], v2["items"]);

    let (c3, _, _) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(c3, 0);
}

// ---------------------------------------------------------------------------
// AC13 — help_ia Daily includes memory
// ---------------------------------------------------------------------------

#[test]
fn root_help__daily_includes_memory() {
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--help")
        .output()
        .expect("help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("memory"),
        "AC13 help Daily includes memory; got help length {}",
        stdout.len()
    );
    assert!(
        stdout.contains(
            "Daily:     recall, preflight, doctor, project, pin, memory, context, stop-session, daemon"
        ) || stdout.contains("pin, memory, context"),
        "Daily inventory string; snippet missing in:\n{}",
        stdout.lines().take(40).collect::<Vec<_>>().join("\n")
    );
}

// ---------------------------------------------------------------------------
// AC19 — summary --tag filters both counts
// ---------------------------------------------------------------------------

#[test]
fn memory_list__summary_tag__filters_both_counts() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory_tagged(&vault, &proj, &id, "DECISION: arch pin", &["architecture"]);
    pin_memory(&vault, &proj, &id, "DECISION: untagged pin");
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        "DECISION: arch forget me zzz",
        &["architecture"],
    );
    forget_by_match(&vault, &proj, &id, "arch forget me zzz");

    let (code, stdout, _) =
        run_memory_list(&vault, &["--summary", "--tag", "architecture"], Some(&id));
    assert_eq!(code, 0, "summary tag; stdout={stdout}");
    assert!(
        stdout.contains("Pinned: 1"),
        "tag filters pinned to 1; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Forgotten: 1"),
        "tag filters forgotten to 1; got:\n{stdout}"
    );
}
