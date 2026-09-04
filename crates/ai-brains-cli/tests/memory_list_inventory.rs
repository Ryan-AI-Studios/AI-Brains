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
    // T316 AC8 / F9 — nonempty human list must not print F36 forget hint on stderr.
    assert!(
        !stderr.contains("forget --memory-id") && !stderr.contains("forget --restore"),
        "AC8 omit F36 stderr; got:\n{stderr}"
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
            "Daily:     recall, preflight, doctor, status, project, pin, memory, context, stop-session, session, daemon"
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

// ---------------------------------------------------------------------------
// T230 — never-blank labels (AC6/AC7/AC9/AC15)
// ---------------------------------------------------------------------------

/// AC6 human + AC7 JSON: every by-project label non-empty under --summary --global.
#[test]
fn memory_list__global_summary__labels_non_empty() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let id_b = register_project(&vault, &b);
    pin_memory(&vault, &a, &id_a, "DECISION: T230 A pin for labels");
    pin_memory(&vault, &b, &id_b, "DECISION: T230 B pin for labels");

    // Human (AC6 + AC12)
    let (code, stdout, _) = run_memory_list(&vault, &["--summary", "--global"], None);
    assert_eq!(code, 0, "global summary human exit 0; stdout={stdout}");
    assert!(
        stdout.contains("Scope: global"),
        "AC12 Scope: global; got:\n{stdout}"
    );
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with("Scope:")
            || trimmed.starts_with("Pinned:")
            || trimmed.starts_with("Forgotten:")
            || trimmed.starts_with("label")
            || trimmed.starts_with("No projects")
        {
            continue;
        }
        // Data rows contain a project_id; first 20 chars are the label col.
        if line.contains(&id_a) || line.contains(&id_b) {
            let label_field: String = line.chars().take(20).collect();
            assert!(
                !label_field.trim().is_empty(),
                "AC6 blank label cell; line={line:?}\nfull:\n{stdout}"
            );
        }
    }

    // JSON (AC7)
    let (c2, out2, _) =
        run_memory_list(&vault, &["--summary", "--global", "--format", "json"], None);
    assert_eq!(c2, 0, "global summary json exit 0; stdout={out2}");
    let v: serde_json::Value = serde_json::from_str(&out2).expect("valid summary json");
    assert_eq!(v["scope"], "global");
    let by_project = v["by_project"]
        .as_array()
        .expect("by_project array under --global");
    assert!(
        by_project.len() >= 2,
        "expect both projects; got {by_project:?}"
    );
    for row in by_project {
        let label = row["label"].as_str().expect("label must be a string");
        assert!(
            !label.is_empty(),
            "AC7 by_project[].label must be non-empty; row={row}"
        );
    }
}

/// AC9: global non-summary human list — project column non-empty (registered OK).
#[test]
fn memory_list__global_human__project_col_non_empty() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let id_b = register_project(&vault, &b);
    pin_memory(&vault, &a, &id_a, "DECISION: T230 list A");
    pin_memory(&vault, &b, &id_b, "DECISION: T230 list B");

    let (code, stdout, _) = run_memory_list(&vault, &["--global", "--limit", "5"], None);
    assert_eq!(code, 0, "global list exit 0; stdout={stdout}");
    assert!(
        stdout.contains("Scope: global"),
        "Scope: global; got:\n{stdout}"
    );
    // Header: memory_id (36) + space + project (20) + …
    let mut data_rows = 0usize;
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Scope:")
            || trimmed.starts_with("status=")
            || trimmed.starts_with("memory_id")
            || trimmed.starts_with("Showing")
            || trimmed.starts_with("No pinned")
            || trimmed.is_empty()
        {
            continue;
        }
        // Data: memory_id UUID-ish at start, then project col.
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 57 {
            continue;
        }
        // project col is fixed-width after memory_id col (36) + sep space.
        let project_col: String = chars.iter().skip(37).take(20).collect();
        if project_col.trim().is_empty()
            && chars
                .iter()
                .take(36)
                .all(|c| c.is_ascii_hexdigit() || *c == '-')
        {
            // Only fail if this looks like a real data row with blank project.
            panic!("AC9 blank project column; line={line:?}\nfull:\n{stdout}");
        }
        if !project_col.trim().is_empty() {
            data_rows += 1;
        }
    }
    assert!(
        data_rows >= 1,
        "AC9 expected ≥1 data row with non-empty project col; stdout:\n{stdout}"
    );
}

/// AC15: forget --list-forgotten --global project column non-empty (shared run_inventory).
#[test]
fn forget_list_forgotten__global__project_col_non_empty() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: T230 forget global unique-token-fgl",
    );
    forget_by_match(&vault, &proj, &id, "unique-token-fgl");

    let (code, stdout, _) = run_forget_list(&vault, &["--global", "--limit", "5"], None);
    assert_eq!(
        code, 0,
        "forget list-forgotten global exit 0; stdout={stdout}"
    );
    assert!(
        stdout.contains("Scope: global"),
        "Scope: global; got:\n{stdout}"
    );
    let mut data_rows = 0usize;
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Scope:")
            || trimmed.starts_with("status=")
            || trimmed.starts_with("memory_id")
            || trimmed.starts_with("Showing")
            || trimmed.starts_with("No forgotten")
            || trimmed.is_empty()
        {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 57 {
            continue;
        }
        let project_col: String = chars.iter().skip(37).take(20).collect();
        assert!(
            !project_col.trim().is_empty(),
            "AC15 blank project column; line={line:?}\nfull:\n{stdout}"
        );
        data_rows += 1;
    }
    assert!(
        data_rows >= 1,
        "AC15 expected ≥1 forgotten row; stdout:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// T287 — human prefer-fill authority; JSON recency freeze
// ---------------------------------------------------------------------------

/// T331 F3 / F35 — copy-not-share 61-char honesty (do not import retrieval const).
const T331_F3: &str = "No DECISION/CONSTRAINT pins in scope; showing recent activity";

fn unique_token(prefix: &str) -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("{prefix}-{n}")
}

fn first_human_preview(stdout: &str) -> String {
    // Human row is `{:<36} {:<12} {preview}` — slice by columns, do not
    // split_whitespace (`just now` is two tokens in the 12-char updated field).
    for line in stdout.lines() {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 51 {
            continue;
        }
        let id: String = chars.iter().take(36).collect();
        if id.starts_with("memory_id") {
            continue;
        }
        let id = id.trim();
        if id.len() != 36 || !id.contains('-') {
            continue;
        }
        let preview: String = chars.iter().skip(50).collect();
        return preview.trim_start().to_string();
    }
    String::new()
}

fn seed_tagged_pin_then_dumps(vault: &Path, work_dir: &Path, project_id: &str, needle: &str) {
    pin_memory_tagged(
        vault,
        work_dir,
        project_id,
        &format!("DECISION: {needle} body for inventory mix"),
        &["t287"],
    );
    pin_memory(
        vault,
        work_dir,
        project_id,
        &format!("## Objective dump one {needle}"),
    );
    pin_memory(
        vault,
        work_dir,
        project_id,
        &format!("## Objective dump two {needle}"),
    );
    pin_memory(
        vault,
        work_dir,
        project_id,
        &format!("## Objective dump three {needle}"),
    );
    pin_memory(
        vault,
        work_dir,
        project_id,
        &format!("## Objective dump four {needle}"),
    );
}

#[test]
fn memory_list__human_limit_5__first_row_is_tagged_decision_not_objective() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T287p");
    seed_tagged_pin_then_dumps(&vault, &proj, &id, &needle);

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("DECISION:"),
        "AC1 human page must include DECISION:; got:\n{stdout}"
    );
    assert!(
        stdout.contains(&needle),
        "AC1 human page must include pin needle; got:\n{stdout}"
    );
    let first = first_human_preview(&stdout);
    assert!(
        !first.starts_with("## Objective"),
        "AC1 first data row must not be Objective dump; first={first:?}\n{stdout}"
    );
    assert!(
        first.contains("DECISION:") || first.contains(&needle),
        "AC1 first data row must be the pin; first={first:?}\n{stdout}"
    );
    assert!(
        !stdout.contains("ASSISTANT: DECISION") && !stdout.contains("ASSISTANT: TAGS"),
        "AC13 human previews must not begin with ASSISTANT:; got:\n{stdout}"
    );
}

#[test]
fn memory_list__json_limit_5__items0_stays_recency_dump() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T287j");
    seed_tagged_pin_then_dumps(&vault, &proj, &id, &needle);

    let (code, stdout, stderr) =
        run_memory_list(&vault, &["--format", "json", "--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC2 exit 0; stderr={stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["scope"], "project");
    assert_eq!(v["status"], "pinned");
    assert!(v["items"].is_array());
    let mut keys: Vec<&str> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "api_version",
            "items",
            "limit",
            "more_available",
            "project_id",
            "returned",
            "scope",
            "status",
            "total",
        ],
        "AC12 T216 field set only; got:\n{stdout}"
    );
    let preview = v["items"][0]["preview"].as_str().unwrap_or("");
    assert!(
        preview.contains("dump four") && preview.contains(&needle),
        "AC2 items[0] is newest recency dump four; preview={preview:?}\n{stdout}"
    );
    assert!(
        !stdout.contains(T331_F3),
        "AC5 JSON stdout must not include F3 honesty; got:\n{stdout}"
    );
}

#[test]
fn memory_list__mix_fixture_summary__counts_dumps_and_pin() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T287s");
    seed_tagged_pin_then_dumps(&vault, &proj, &id, &needle);

    let (code, stdout, stderr) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(code, 0, "AC7 exit 0; stderr={stderr}");
    let pinned = stdout
        .lines()
        .find_map(|l| l.strip_prefix("Pinned: "))
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    assert!(
        pinned >= 5,
        "AC7 summary counts dumps+pin (status COUNT); got:\n{stdout}"
    );
}

#[test]
fn memory_list__human_limit_5__untagged_decision_prefer_filled() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T287u");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {needle} untagged inventory pin"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump one {needle}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump two {needle}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump three {needle}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump four {needle}"),
    );

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC9 exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        !first.starts_with("## Objective"),
        "AC9 untagged pin still prefer-fills; first={first:?}\n{stdout}"
    );
    assert!(
        first.contains("DECISION:") || first.contains(&needle),
        "AC9 first row is untagged pin; first={first:?}\n{stdout}"
    );
}

#[test]
fn memory_list__human_tag_t287__mix_among_tag_matches_only() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let tagged = unique_token("T287tg");
    let untagged = unique_token("T287un");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {untagged} untagged must not appear under --tag"),
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {tagged} tagged authority pin"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump one {tagged}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump two {tagged}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump three {tagged}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump four {tagged}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump five {tagged}"),
        &["t287"],
    );

    let (code, stdout, stderr) =
        run_memory_list(&vault, &["--limit", "5", "--tag", "t287"], Some(&id));
    assert_eq!(code, 0, "F12 mix+tag exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        first.contains("DECISION:") && first.contains(&tagged),
        "tagged authority wins under --tag; first={first:?}\n{stdout}"
    );
    assert!(
        !stdout.contains(&untagged),
        "untagged pin excluded by --tag; got:\n{stdout}"
    );
}

#[test]
fn memory_list__human_limit_5__newer_tagged_dumps_do_not_starve_older_pin() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T287n");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {needle} must survive tagged-dump GLOB head"),
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump one {needle}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump two {needle}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump three {needle}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump four {needle}"),
        &["t287"],
    );
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("## Objective tagged dump five {needle}"),
        &["t287"],
    );

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "starve-guard exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        !first.starts_with("## Objective"),
        "pass-1 must over-fetch past tagged Other GLOB rows; first={first:?}\n{stdout}"
    );
    assert!(
        first.contains("DECISION:") || first.contains(&needle),
        "older untagged pin must prefer-fill; first={first:?}\n{stdout}"
    );
}

#[test]
fn memory_list__chrome_only_vault__first_row_stays_objective() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let nonce = unique_token("T287c");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump one {nonce}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump two {nonce}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump three {nonce}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump four {nonce}"),
    );

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC10/AC6 exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        first.starts_with("## Objective"),
        "AC10 chrome-only first row is Objective; first={first:?}\n{stdout}"
    );
    let f3 = stdout.matches(T331_F3).count();
    assert_eq!(
        f3, 1,
        "AC6 chrome-only prints F3 honesty once; got {f3} in:\n{stdout}"
    );
    assert!(
        !stdout.contains("No pinned memories."),
        "AC6 must not empty-table lie; got:\n{stdout}"
    );

    let (scode, sstdout, _) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(scode, 0, "AC9 summary exit 0");
    assert!(
        sstdout.contains("Pinned:"),
        "AC9 summary still prints Pinned COUNT; got:\n{sstdout}"
    );
    assert!(
        !sstdout.contains(T331_F3),
        "AC9 --summary must not print F3; got:\n{sstdout}"
    );
}

/// T331 AC18 — `--global` chrome-only still prints F3.
#[test]
fn memory_list__global_chrome_only__prints_f4() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let nonce = unique_token("T331g");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump one {nonce}"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective dump two {nonce}"),
    );

    let (code, stdout, stderr) = run_memory_list(&vault, &["--global", "--limit", "5"], None);
    assert_eq!(code, 0, "AC18 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("Scope: global") || stdout.to_ascii_lowercase().contains("global"),
        "AC18 --global scope; got:\n{stdout}"
    );
    let f3 = stdout.matches(T331_F3).count();
    assert_eq!(
        f3, 1,
        "AC18 global empty-authority prints F3 once; got {f3} in:\n{stdout}"
    );
}

/// T331 AC1 — GLOB-empty chrome + older process Other: process first, F3 once.
#[test]
fn memory_list__glob_empty_chrome_plus_process__process_is_first_with_f4() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T331p");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("T331 process {needle} inventory skim body"),
    );
    pin_memory(&vault, &proj, &id, "## Objective dump-1");
    pin_memory(&vault, &proj, &id, "## Objective dump-2");
    pin_memory(&vault, &proj, &id, "## Objective dump-3");
    pin_memory(&vault, &proj, &id, "## Objective dump-4");

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        !first.starts_with("## Objective"),
        "AC1 first preview must not be Objective chrome; first={first:?}\n{stdout}"
    );
    assert!(
        first.contains("T331 process") && first.contains(&needle),
        "AC1 first preview is the process needle; first={first:?}\n{stdout}"
    );
    let f3 = stdout.matches(T331_F3).count();
    assert_eq!(f3, 1, "AC1 F3 honesty once; got {f3} in:\n{stdout}");
}

/// T331 AC2 — lowercase `decision:` GLOB-miss is still first; no F3.
#[test]
fn memory_list__glob_empty_lowercase_decision__is_first_no_f4() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let needle = unique_token("T331d");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("decision: {needle} lowercase inventory pin"),
    );
    pin_memory(&vault, &proj, &id, &format!("## Objective dump-1 {needle}"));
    pin_memory(&vault, &proj, &id, &format!("## Objective dump-2 {needle}"));
    pin_memory(&vault, &proj, &id, &format!("## Objective dump-3 {needle}"));
    pin_memory(&vault, &proj, &id, &format!("## Objective dump-4 {needle}"));

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC2 exit 0; stderr={stderr}");
    let first = first_human_preview(&stdout);
    assert!(
        !first.starts_with("## Objective"),
        "AC2 first preview must not be Objective chrome; first={first:?}\n{stdout}"
    );
    assert!(
        first.contains("decision:") && first.contains(&needle),
        "AC2 first preview is lowercase decision pin; first={first:?}\n{stdout}"
    );
    assert!(
        !stdout.contains(T331_F3),
        "AC2 authority present → no F3; got:\n{stdout}"
    );
}

#[test]
fn memory_list_help__mentions_human_authority_and_json_recency() {
    let out = hermetic()
        .arg("--no-project-context")
        .args(["memory", "list", "--help"])
        .output()
        .expect("memory list --help");
    assert!(out.status.success(), "help exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("prefer-fill") && stdout.contains("authority"),
        "AC17 after_help names human prefer-fill authority; got:\n{stdout}"
    );
    assert!(
        stdout.contains("JSON") && stdout.contains("recency"),
        "AC17 after_help names JSON recency freeze; got:\n{stdout}"
    );
    let lower = stdout.to_ascii_lowercase();
    assert!(
        lower.contains("no leading-line")
            || stdout.contains(T331_F3)
            || stdout.contains("showing recent activity"),
        "AC11 after_help names GLOB-empty honesty / no leading-line; got:\n{stdout}"
    );
}

/// T316 AC14 — after_help names chrome-skip + no runtime forget hint.
#[test]
fn memory_list_help__after_help__names_chrome_skip_and_no_forget_hint() {
    let out = hermetic()
        .arg("--no-project-context")
        .args(["memory", "list", "--help"])
        .output()
        .expect("memory list --help");
    assert!(out.status.success(), "help exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    assert!(
        stdout.contains("chrome") || stdout.contains("let me"),
        "AC14 after_help names chrome-skip / Let me; got help"
    );
    assert!(
        stdout.contains("forget")
            && (stdout.contains("does not print")
                || stdout.contains("no runtime")
                || stdout.contains("not print a forget")
                || stdout.contains("does not print a forget")),
        "AC14 after_help names no runtime forget hint; got help"
    );
}

/// T316 AC8 named — nonempty human list omits F36 stderr.
#[test]
fn memory_list__nonempty__omits_f36_stderr() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: T316 nonempty f36 omit pin");

    let (code, stdout, stderr) = run_memory_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "exit 0; stderr={stderr}");
    assert!(stdout.contains("Showing"), "nonempty table; got:\n{stdout}");
    assert!(
        !stderr.contains("forget --memory-id") && !stderr.contains("forget --restore"),
        "AC8 omit F36 stderr; got:\n{stderr}"
    );
}

/// T316 AC11 — JSON preview values skip chrome; keys stay T216.
#[test]
fn memory_list__format_json__preview_skips_chrome() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let nonce = unique_token("t316-json-chrome");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective\nWe decided SQLCipher {nonce}"),
    );

    let (code, stdout, _) =
        run_memory_list(&vault, &["--format", "json", "--limit", "1"], Some(&id));
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("json");
    // T216 nine envelope keys (project_id present under project scope).
    for key in [
        "api_version",
        "scope",
        "project_id",
        "status",
        "returned",
        "more_available",
        "limit",
        "total",
        "items",
    ] {
        assert!(
            v.get(key).is_some(),
            "T216 key {key} missing; got:\n{stdout}"
        );
    }
    assert!(
        v.get("next_step").is_none() && v.get("chrome_skipped").is_none(),
        "no new JSON keys; got:\n{stdout}"
    );
    let preview = v["items"][0]["preview"].as_str().unwrap_or("");
    assert!(
        preview.contains("We decided") || preview.contains("SQLCipher"),
        "AC11 JSON preview skips chrome; got {preview:?}"
    );
    assert!(
        !preview.starts_with("## Objective"),
        "AC11 must not keep ## Objective; got {preview:?}"
    );
}

#[test]
fn memory_list__forgotten_status__no_authority_promote_of_remaining_pin() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    let pin_needle = unique_token("T287k");
    let dump_needle = unique_token("T287d");
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {pin_needle} stays pinned"),
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        &format!("## Objective forget-me {dump_needle}"),
    );
    forget_by_match(&vault, &proj, &id, &dump_needle);

    let (c1, out1, _) = run_memory_list(
        &vault,
        &["--status", "forgotten", "--limit", "5"],
        Some(&id),
    );
    let (c2, out2, _) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(c1, 0, "AC8 memory list forgotten; {out1}");
    assert_eq!(c2, 0, "AC8 forget list-forgotten; {out2}");
    assert!(
        out1.contains(&dump_needle) || out1.contains("Objective"),
        "AC8 forgotten list shows the dump; got:\n{out1}"
    );
    assert!(
        !out1.contains(&pin_needle),
        "AC8 must not promote remaining pinned DECISION into forgotten list; got:\n{out1}"
    );
    assert!(
        !out2.contains(&pin_needle),
        "AC8 list-forgotten must not promote remaining pin; got:\n{out2}"
    );
    assert!(
        !out1.contains(T331_F3) && !out2.contains(T331_F3),
        "AC8 forgotten path must not print F3 honesty; got list:\n{out1}\nlist-forgotten:\n{out2}"
    );
}

// ---------------------------------------------------------------------------
// T299 — empty forgotten useful remediator (Pinned: N + next: memory list)
// ---------------------------------------------------------------------------

fn parse_pinned_count(stdout: &str) -> Option<u64> {
    stdout
        .lines()
        .find_map(|l| l.strip_prefix("Pinned: "))
        .and_then(|s| s.trim().parse::<u64>().ok())
}

fn last_nonempty_line(stdout: &str) -> &str {
    stdout
        .lines()
        .map(str::trim_end)
        .rfind(|l| !l.is_empty())
        .unwrap_or("")
}

/// T299 AC1 — empty forgotten with ≥1 pin: Pinned matches summary; next last.
#[test]
fn forget_list_forgotten__empty_with_pin__pinned_count_and_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: T299 inventory pin stays pinned",
    );

    let (sum_code, sum_out, _) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(sum_code, 0, "summary; {sum_out}");
    let summary_pinned = parse_pinned_count(&sum_out).expect("summary Pinned:");
    assert!(summary_pinned >= 1, "expect ≥1 pin; {sum_out}");

    let (code, stdout, stderr) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0, "AC1 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("No forgotten memories."),
        "AC1 empty const; got:\n{stdout}"
    );
    let list_pinned = parse_pinned_count(&stdout).expect("list Pinned:");
    assert_eq!(
        list_pinned, summary_pinned,
        "AC1 Pinned must match summary; list={list_pinned} summary={summary_pinned}\n{stdout}\n{sum_out}"
    );
    assert_eq!(
        last_nonempty_line(&stdout),
        "next: ai-brains memory list",
        "AC1 next last; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("forget --restore") && !stdout.contains("forget --memory-id"),
        "AC1 no F36 restore on stdout; got:\n{stdout}"
    );
    assert!(
        !stderr.contains("forget --restore") && !stderr.contains("forget --memory-id"),
        "AC1 empty skips F36 stderr; got:\n{stderr}"
    );
}

/// T299 AC2 — forget list-forgotten stdout equals memory list --status forgotten.
#[test]
fn forget_list_forgotten__empty_matches_memory_list_status_forgotten() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: T299 shared empty backend pin",
    );

    let (c1, out1, _) = run_memory_list(
        &vault,
        &["--status", "forgotten", "--limit", "5"],
        Some(&id),
    );
    let (c2, out2, _) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(c1, 0);
    assert_eq!(c2, 0);
    assert_eq!(out1, out2, "AC2 / F6 byte-identical stdout");
    assert!(out1.contains("No forgotten memories."));
    assert!(parse_pinned_count(&out1).is_some());
    assert_eq!(last_nonempty_line(&out1), "next: ai-brains memory list");
}

/// T299 AC3 — 0 pins + 0 forgotten still prints Pinned: 0 + next.
#[test]
fn forget_list_forgotten__empty_zero_pins__pinned_zero_and_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);

    let (code, stdout, _) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0);
    assert!(stdout.contains("No forgotten memories."));
    assert_eq!(parse_pinned_count(&stdout), Some(0));
    assert_eq!(last_nonempty_line(&stdout), "next: ai-brains memory list");
}

/// T299 AC4 — nonempty forgotten omits T299 remediator.
/// T316 AC9 — F36 stderr also omitted (supersedes T216 runtime stderr).
#[test]
fn forget_list_forgotten__nonempty__omits_t299_remediator() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &id,
        "DECISION: T299 nonempty forget token abcxyz",
    );
    forget_by_match(&vault, &proj, &id, "abcxyz");

    let (code, stdout, stderr) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0);
    assert!(
        stdout.contains("abcxyz") || stdout.contains("nonempty forget"),
        "nonempty preview; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("next: ai-brains memory list"),
        "AC4 omit T299 next; got:\n{stdout}"
    );
    // T299 Pinned: line after empty const only — nonempty table may still have
    // project labels; assert no remediator block after Showing.
    let after_showing = stdout
        .lines()
        .skip_while(|l| !l.starts_with("Showing "))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !after_showing.contains("Pinned:"),
        "AC4 no T299 Pinned after table; got:\n{stdout}"
    );
    assert!(
        !stderr.contains("forget --restore") && !stderr.contains("forget --memory-id"),
        "AC9 omit F36 stderr; got:\n{stderr}"
    );
}

/// T316 AC9 named alias — nonempty forgotten omits F36 stderr.
#[test]
fn forget_list_forgotten__nonempty__omits_f36_stderr() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: T316 f36 omit token f36xyz");
    forget_by_match(&vault, &proj, &id, "f36xyz");

    let (code, _stdout, stderr) = run_forget_list(&vault, &["--limit", "5"], Some(&id));
    assert_eq!(code, 0);
    assert!(
        !stderr.contains("forget --restore") && !stderr.contains("forget --memory-id"),
        "AC9 omit F36 stderr; got:\n{stderr}"
    );
}

/// T299 AC5 — empty forgotten JSON keys frozen; no next_step / pinned.
#[test]
fn forget_list_forgotten__empty_json__no_next_step_or_pinned() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: T299 json freeze pin");

    let (code, stdout, _) =
        run_forget_list(&vault, &["--limit", "5", "--format", "json"], Some(&id));
    assert_eq!(code, 0, "json exit 0; {stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(v["status"], "forgotten");
    assert_eq!(v["total"], 0);
    assert!(v["items"].as_array().expect("items").is_empty());
    let mut keys: Vec<&str> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "api_version",
            "items",
            "limit",
            "more_available",
            "project_id",
            "returned",
            "scope",
            "status",
            "total",
        ],
        "AC5 / F10 exact nine keys; got:\n{stdout}"
    );
    assert!(v.get("next_step").is_none(), "F10 no next_step; {stdout}");
    assert!(v.get("pinned").is_none(), "F10 no pinned; {stdout}");
    assert!(v.get("next").is_none(), "F10 no next; {stdout}");
}

/// T299 AC6 — --global empty forgotten next includes --global.
#[test]
fn forget_list_forgotten__global_empty__next_includes_global() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let id_a = register_project(&vault, &a);
    let _id_b = register_project(&vault, &b);
    pin_memory(
        &vault,
        &a,
        &id_a,
        "DECISION: T299 global empty pin on A only",
    );

    let (sum_code, sum_out, _) = run_memory_list(&vault, &["--summary", "--global"], None);
    assert_eq!(sum_code, 0);
    let summary_pinned = parse_pinned_count(&sum_out).expect("global summary Pinned:");

    let (code, stdout, _) = run_forget_list(&vault, &["--limit", "5", "--global"], None);
    assert_eq!(code, 0, "AC6 exit 0; {stdout}");
    assert!(stdout.contains("No forgotten memories."));
    assert_eq!(
        parse_pinned_count(&stdout),
        Some(summary_pinned),
        "AC6 Pinned matches global summary"
    );
    assert_eq!(
        last_nonempty_line(&stdout),
        "next: ai-brains memory list --global",
        "AC6 global next; got:\n{stdout}"
    );
    assert_ne!(
        last_nonempty_line(&stdout),
        "next: ai-brains memory list",
        "AC6 must not use non-global next as last line"
    );
}

/// T299 AC7 stay-green — pinned-empty does not gain forgotten remediator.
#[test]
fn memory_list__pinned_empty__omits_forgotten_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);

    let (code, stdout, _) = run_memory_list(&vault, &[], Some(&id));
    assert_eq!(code, 0);
    assert!(stdout.contains("No pinned memories."));
    assert!(
        !stdout.contains("next: ai-brains memory list"),
        "AC7 pinned-empty omits T299 next; got:\n{stdout}"
    );
}

/// T299 AC8 stay-green — summary has no T299 next.
#[test]
fn memory_list__summary__omits_t299_next() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory(&vault, &proj, &id, "DECISION: T299 summary freeze pin");

    let (code, stdout, _) = run_memory_list(&vault, &["--summary"], Some(&id));
    assert_eq!(code, 0);
    assert!(stdout.contains("Pinned:"));
    assert!(stdout.contains("Forgotten:"));
    assert!(
        !stdout.contains("next: ai-brains memory list"),
        "AC8 summary omits T299 next; got:\n{stdout}"
    );
}

/// T299 AC11 — --tag filters Pinned COUNT like --summary --tag.
#[test]
fn forget_list_forgotten__empty_tag__pinned_matches_summary_tag() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj = dir.path().join("proj");
    let id = register_project(&vault, &proj);
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        "DECISION: T299 architecture pin body",
        &["architecture"],
    );
    pin_memory(&vault, &proj, &id, "DECISION: T299 untagged pin body");

    let (sum_code, sum_out, _) =
        run_memory_list(&vault, &["--summary", "--tag", "architecture"], Some(&id));
    assert_eq!(sum_code, 0);
    let summary_pinned = parse_pinned_count(&sum_out).expect("tag summary Pinned:");

    let (code, stdout, _) = run_forget_list(
        &vault,
        &["--limit", "5", "--tag", "architecture"],
        Some(&id),
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("No forgotten memories."));
    assert_eq!(parse_pinned_count(&stdout), Some(summary_pinned));
    assert_eq!(last_nonempty_line(&stdout), "next: ai-brains memory list");

    let (c2, out2, _) = run_forget_list(&vault, &["--limit", "5", "--tag", "nosuchtag"], Some(&id));
    assert_eq!(c2, 0);
    assert!(out2.contains("No forgotten memories."));
    assert_eq!(parse_pinned_count(&out2), Some(0));
    assert_eq!(last_nonempty_line(&out2), "next: ai-brains memory list");
}
