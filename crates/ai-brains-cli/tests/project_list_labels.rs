#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T212 — Project list human labels hermetic suite (AC1–AC6, AC11 soft AC12).
//!
//! Pattern: tempdir vault + context for project registration + set-alias / pin;
//! hermetic_bin + `--no-project-context`.

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

fn set_alias(vault: &Path, project_id: &str, alias: &str) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("set-alias")
        .arg(project_id)
        .arg(alias)
        .assert()
        .success();
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    // Session id from .env written by context.
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

// ---------------------------------------------------------------------------
// AC1 — alias shows as label
// ---------------------------------------------------------------------------

#[test]
fn project_list__alias_acme__human_label_contains_acme() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("acme-proj");
    let project_id = register_project(&vault, &proj);
    set_alias(&vault, &project_id, "acme");

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("acme"),
        "AC1: human label must contain acme; got: {stdout}"
    );
    // Label column is first; header uses "label".
    assert!(
        stdout.contains("label"),
        "must use label-first header; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — no-alias / baked form → label exactly "(no alias)"
// ---------------------------------------------------------------------------

#[test]
fn project_list__no_alias__label_exactly_no_alias() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("noalias-proj");
    let project_id = register_project(&vault, &proj);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Full project_id still present in its own column.
    assert!(
        stdout.contains(&project_id),
        "full project_id column required; got: {stdout}"
    );
    // Label is exactly "(no alias)" — not the baked " — short" form as sole cue.
    // Find the data row and check the first field.
    let data_line = stdout
        .lines()
        .find(|l| l.contains(&project_id))
        .expect("row with project_id");
    let label_field = data_line.split_whitespace().next().unwrap_or("");
    assert_eq!(
        label_field, "(no",
        // human table pads; first token of "(no alias)" is "(no"
        "label should start with (no alias); line={data_line}"
    );
    assert!(
        data_line.contains("(no alias)"),
        "AC2: label must be (no alias); line={data_line}"
    );
    // Baked short form should not appear as the display label body.
    assert!(
        !data_line.contains("(no alias) —"),
        "must strip baked short suffix from label; line={data_line}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — ≥1 unaliased → stderr set-alias + project_id; table on stdout
// ---------------------------------------------------------------------------

#[test]
fn project_list__unaliased__stderr_set_alias_footer() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("footer-proj");
    let project_id = register_project(&vault, &proj);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("label") && stdout.contains(&project_id),
        "table on stdout; got: {stdout}"
    );
    assert!(
        stderr.contains("project set-alias"),
        "AC3: stderr must mention project set-alias; got: {stderr}"
    );
    assert!(
        stderr.contains(&project_id),
        "AC3: stderr must include unaliased project_id; got: {stderr}"
    );
    // Footer must not leak onto stdout.
    assert!(
        !stdout.contains("project set-alias"),
        "footer must be stderr only; stdout={stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC4 — empty vault: T198 empty line; no set-alias footer
// ---------------------------------------------------------------------------

#[test]
fn project_list__empty_vault__t198_no_footer() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list empty");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("No projects registered. (0 projects)"),
        "T198 empty line; got: {stdout}"
    );
    assert!(
        !stdout.contains("set-alias") && !stderr.contains("set-alias"),
        "AC4: no set-alias footer on empty vault; stdout={stdout} stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC5 — --format json
// ---------------------------------------------------------------------------

#[test]
fn project_list__format_json__shape_and_unaliased_count() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj_a = dir.path().join("json-a");
    let pid_a = register_project(&vault, &proj_a);
    set_alias(&vault, &pid_a, "aliased-one");

    let proj_b = dir.path().join("json-b");
    let pid_b = register_project(&vault, &proj_b);
    // pid_b left unaliased

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !stdout.contains("project set-alias"),
        "JSON stdout must not contain footer; got: {stdout}"
    );
    // Footer optional on stderr for human only; JSON path should not print footer.
    assert!(
        !stderr.contains("project set-alias"),
        "JSON format should not emit set-alias footer; stderr={stderr}"
    );

    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["api_version"], "1");
    let projects = v["projects"].as_array().expect("projects array");
    assert!(
        projects.len() >= 2,
        "expected ≥2 projects; got {}",
        projects.len()
    );
    for p in projects {
        assert!(p.get("project_id").is_some());
        assert!(p.get("label").is_some());
        assert!(p.get("memory_count").is_some());
        assert!(p.get("name").is_some());
        assert!(p.get("alias").is_some());
        assert!(p.get("last_activity").is_some() || p["last_activity"].is_null());
        assert!(p.get("path").is_some() || p["path"].is_null());
    }
    let unaliased = v["unaliased_count"].as_u64().expect("unaliased_count");
    // Fixture: pid_a aliased, pid_b unaliased only → exact 1 (not ambient vault rows).
    assert_eq!(
        unaliased, 1,
        "unaliased_count must be exactly 1 (pid_b); got {unaliased}; body={stdout}"
    );

    // Aliased project label is alias.
    let acme = projects.iter().find(|p| p["project_id"] == pid_a);
    let acme = acme.expect("aliased project in JSON");
    assert_eq!(acme["label"], "aliased-one");
    assert_eq!(acme["alias"], "aliased-one");

    let bare = projects.iter().find(|p| p["project_id"] == pid_b);
    let bare = bare.expect("unaliased project in JSON");
    assert_eq!(bare["label"], "(no alias)");
}

// ---------------------------------------------------------------------------
// AC6 — project with ≥1 memory → last_activity non-empty in JSON
// ---------------------------------------------------------------------------

#[test]
fn project_list__with_memory__last_activity_nonempty_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("mem-proj");
    let project_id = register_project(&vault, &proj);
    pin_memory(
        &vault,
        &proj,
        &project_id,
        "DECISION: T212 last_activity seed",
    );

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let projects = v["projects"].as_array().expect("projects");
    let row = projects
        .iter()
        .find(|p| p["project_id"] == project_id)
        .expect("seeded project");
    let activity = row["last_activity"].as_str().unwrap_or("");
    assert!(
        !activity.is_empty(),
        "AC6: last_activity must be non-empty after pin; row={row}"
    );
    assert!(
        row["memory_count"].as_u64().unwrap_or(0) >= 1,
        "memory_count should be ≥1; row={row}"
    );
}

// ---------------------------------------------------------------------------
// Soft AC12 — AI_BRAINS_PROJECT_ID set → * on label
// ---------------------------------------------------------------------------

#[test]
fn project_list__active_project_id__star_prefix_on_label() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("star-proj");
    let project_id = register_project(&vault, &proj);
    set_alias(&vault, &project_id, "starred");

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", &project_id)
        .arg("project")
        .arg("list")
        .output()
        .expect("project list active");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("*starred")
            || stdout
                .lines()
                .any(|l| l.contains('*') && l.contains("starred")),
        "AC12 soft: active project label should have * prefix; got: {stdout}"
    );
}
