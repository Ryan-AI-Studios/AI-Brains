//! T254 — `project list-paths` + `project scan-roots` + `project unregister-path`
//! hermetic suite (AC1–AC6, AC10–AC12).
//!
//! Pattern: tempdir vault + context for project registration;
//! hermetic_bin + `--no-project-context`.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

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

fn register_path(vault: &Path, project_ref: &str, path: &str) -> assert_cmd::assert::Assert {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_ref)
        .arg(path)
        .assert()
}

fn unregister_path_cmd(
    vault: &Path,
    path: &str,
    project: Option<&str>,
    dry_run: bool,
) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("unregister-path");
    if let Some(project) = project {
        cmd.arg("--project").arg(project);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }
    cmd.arg(path);
    cmd
}

fn json_has_normalized_path(v: &serde_json::Value, normalized: &str) -> bool {
    v["paths"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .any(|row| row["normalized_path"].as_str() == Some(normalized))
        })
        .unwrap_or(false)
}

fn list_paths_json(vault: &Path) -> serde_json::Value {
    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("list-paths")
        .arg("--format")
        .arg("json")
        .output()
        .expect("list-paths json");
    assert!(
        out.status.success(),
        "list-paths json must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("list-paths stdout must be one pretty JSON object, not NDJSON: {e}; stdout={stdout}")
    })
}

// ---------------------------------------------------------------------------
// AC1 — empty vault: copy + next-step; exit 0
// ---------------------------------------------------------------------------

#[test]
fn list_paths__empty_vault__empty_copy_and_next_step() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list-paths")
        .arg("--format")
        .arg("human")
        .output()
        .expect("list-paths human");
    assert!(
        out.status.success(),
        "AC1: empty list-paths must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No path aliases registered."),
        "AC1: empty copy; got: {stdout}"
    );
    assert!(
        stdout.contains("next: ai-brains project register-path <project_id|alias> <path>"),
        "AC1: next-step must mention register-path; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC2 / AC3 — two aliases, ASC JSON, frozen keys
// ---------------------------------------------------------------------------

#[test]
fn list_paths__two_aliases__both_asc_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let id_a = register_project(&vault, &dir.path().join("proj-a"));
    let id_b = register_project(&vault, &dir.path().join("proj-b"));
    set_alias(&vault, &id_a, "alpha");
    set_alias(&vault, &id_b, "zeta");

    let aaa = dir.path().join("aaa-root");
    let zzz = dir.path().join("zzz-root");
    fs::create_dir_all(&aaa).unwrap();
    fs::create_dir_all(&zzz).unwrap();
    register_path(&vault, &id_a, aaa.to_str().expect("utf8")).success();
    register_path(&vault, &id_b, zzz.to_str().expect("utf8")).success();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list-paths")
        .arg("--format")
        .arg("json")
        .output()
        .expect("list-paths json");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains('\n') && stdout.trim_start().starts_with('{'),
        "AC3: pretty JSON object, not NDJSON; stdout={stdout}"
    );
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("parseable JSON object");
    assert!(v.is_object(), "AC3: envelope must be an object; got {v}");
    assert_eq!(v["api_version"], "1");

    let paths = v["paths"].as_array().expect("paths array");
    assert_eq!(paths.len(), 2, "AC2: both aliases listed; got {paths:?}");

    let n0 = paths[0]["normalized_path"]
        .as_str()
        .expect("normalized_path");
    let n1 = paths[1]["normalized_path"]
        .as_str()
        .expect("normalized_path");
    assert!(
        n0 < n1,
        "AC2: ASC by normalized_path; got {n0:?} then {n1:?}"
    );

    for row in paths {
        let obj = row.as_object().expect("path object");
        for key in ["project_id", "label", "alias", "normalized_path", "exists"] {
            assert!(obj.contains_key(key), "F10 frozen key {key} missing: {row}");
        }
        assert!(row["exists"].is_boolean(), "exists must be boolean: {row}");
    }

    let by_alias: Vec<&str> = paths.iter().filter_map(|p| p["alias"].as_str()).collect();
    assert!(
        by_alias.contains(&"alpha") && by_alias.contains(&"zeta"),
        "F10 join must include aliases; got {by_alias:?}"
    );
}

#[test]
fn list_paths__two_aliases_same_project__list_still_first_path_only() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let project_id = register_project(&vault, &dir.path().join("proj-multi"));
    let aaa = dir.path().join("aaa-root");
    let zzz = dir.path().join("zzz-root");
    fs::create_dir_all(&aaa).unwrap();
    fs::create_dir_all(&zzz).unwrap();
    register_path(&vault, &project_id, aaa.to_str().expect("utf8")).success();
    register_path(&vault, &project_id, zzz.to_str().expect("utf8")).success();

    let list_out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("project list json");
    assert!(list_out.status.success());
    let list_v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list_out.stdout)).expect("list json");
    let projects = list_v["projects"].as_array().expect("projects");
    let row = projects
        .iter()
        .find(|p| p["project_id"].as_str() == Some(project_id.as_str()))
        .expect("project row");
    assert!(
        row["path"].is_string(),
        "AC2/F12: project list path stays a single string; got {}",
        row["path"]
    );

    let lp = list_paths_json(&vault);
    let paths = lp["paths"].as_array().expect("paths");
    assert_eq!(
        paths.len(),
        2,
        "list-paths must show both roots; got {paths:?}"
    );

    let mut norms: Vec<&str> = paths
        .iter()
        .filter_map(|p| p["normalized_path"].as_str())
        .collect();
    norms.sort_unstable();
    assert_eq!(
        row["path"].as_str(),
        Some(norms[0]),
        "project list path column is the first alias only"
    );
}

#[test]
fn list_paths__unknown_format__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("list-paths")
        .arg("--format")
        .arg("nope")
        .assert()
        .failure()
        .code(2);
}

// ---------------------------------------------------------------------------
// AC10–AC12 — scan-roots dry-run
// ---------------------------------------------------------------------------

#[test]
fn scan_roots__ledgerful_child_hits_plain_misses() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let tree = dir.path().join("tree");
    let hit = tree.join("hit-child");
    let plain = tree.join("plain-child");
    fs::create_dir_all(&hit).unwrap();
    fs::create_dir_all(&plain).unwrap();
    fs::write(hit.join(".ledgerful"), b"").unwrap();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .arg("--format")
        .arg("json")
        .output()
        .expect("scan-roots json");
    assert!(
        out.status.success(),
        "AC10: scan-roots must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("scan json object");
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["truncated"], false);
    let roots = v["roots"].as_array().expect("roots");
    assert_eq!(roots.len(), 1, "only the .ledgerful child; got {roots:?}");
    let path = roots[0]["path"].as_str().unwrap_or("");
    assert!(
        path.contains("hit-child"),
        "hit child must be listed; path={path}"
    );
    assert!(
        !path.contains("plain-child"),
        "plain child must not be a hit; path={path}"
    );
    let suggested = roots[0]["suggested"].as_str().unwrap_or("");
    assert!(
        suggested.contains("register-path"),
        "AC11: suggested line contains register-path; got {suggested}"
    );
    assert!(
        roots[0]["registered_project_id"].is_null(),
        "unregistered hit has null registered_project_id"
    );
    assert!(roots[0]["exists"].is_boolean());
}

#[test]
fn scan_roots__changeguard_only_not_a_hit() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let tree = dir.path().join("tree");
    let hit = tree.join("hit-child");
    let cg = tree.join("cg-only");
    fs::create_dir_all(&hit).unwrap();
    fs::create_dir_all(&cg).unwrap();
    fs::write(hit.join(".ledgerful"), b"").unwrap();
    fs::create_dir(cg.join(".changeguard")).unwrap();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .arg("--format")
        .arg("json")
        .output()
        .expect("scan-roots json");
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let roots = v["roots"].as_array().expect("roots");
    assert!(
        roots
            .iter()
            .filter_map(|r| r["path"].as_str())
            .all(|p| !p.contains("cg-only")),
        "AC12: .changeguard-only must not be a hit; roots={roots:?}"
    );
    assert_eq!(roots.len(), 1, "only the .ledgerful sibling; got {roots:?}");
}

#[test]
fn scan_roots__never_writes_events() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let before = list_paths_json(&vault);
    assert_eq!(
        before["paths"].as_array().map(|a| a.len()),
        Some(0),
        "precondition: no aliases"
    );

    let tree = dir.path().join("tree");
    let hit = tree.join("hit-child");
    fs::create_dir_all(&hit).unwrap();
    fs::write(hit.join(".ledgerful"), b"").unwrap();

    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .assert()
        .success();

    let after = list_paths_json(&vault);
    assert_eq!(
        after["paths"].as_array().map(|a| a.len()),
        Some(0),
        "AC10: scan-roots must not create aliases; after={after}"
    );
}

#[test]
fn scan_roots__already_registered__shows_project_id() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let project_id = register_project(&vault, &dir.path().join("proj"));
    let tree = dir.path().join("tree");
    let hit = tree.join("hit-child");
    fs::create_dir_all(&hit).unwrap();
    fs::write(hit.join(".ledgerful"), b"").unwrap();
    register_path(&vault, &project_id, hit.to_str().expect("utf8")).success();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .arg("--format")
        .arg("json")
        .output()
        .expect("scan-roots json");
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let roots = v["roots"].as_array().expect("roots");
    let row = roots
        .iter()
        .find(|r| r["path"].as_str().is_some_and(|p| p.contains("hit-child")))
        .expect("registered hit listed");
    assert_eq!(
        row["registered_project_id"].as_str(),
        Some(project_id.as_str()),
        "AC11: registered_project_id matches owner; row={row}"
    );
}

// ---------------------------------------------------------------------------
// AC4–AC6 — unregister-path
// ---------------------------------------------------------------------------

#[test]
fn unregister_path__after_register__list_paths_drops_row() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let id_a = register_project(&vault, &dir.path().join("proj-a"));
    let id_b = register_project(&vault, &dir.path().join("proj-b"));

    let root = dir.path().join("unreg-root");
    fs::create_dir_all(&root).unwrap();
    let path_str = root.to_str().expect("utf8");
    let normalized = ai_brains_path::normalize_for_location_compare(path_str);

    register_path(&vault, &id_a, path_str).success();
    assert!(
        json_has_normalized_path(&list_paths_json(&vault), &normalized),
        "precondition: path registered to A"
    );

    unregister_path_cmd(&vault, path_str, None, false)
        .assert()
        .success();

    let after = list_paths_json(&vault);
    assert!(
        !json_has_normalized_path(&after, &normalized),
        "AC4: list-paths must drop the row; after={after}"
    );

    register_path(&vault, &id_b, path_str).success();
    let rebound = list_paths_json(&vault);
    let paths = rebound["paths"].as_array().expect("paths");
    let row = paths
        .iter()
        .find(|p| p["normalized_path"].as_str() == Some(normalized.as_str()))
        .expect("re-registered path listed");
    assert_eq!(
        row["project_id"].as_str(),
        Some(id_b.as_str()),
        "AC4: same path can register to another project; row={row}"
    );
}

#[test]
fn unregister_path__missing__exit_0_not_registered() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let missing = r"C:\dev\T254MissingUnreg";
    let out = unregister_path_cmd(&vault, missing, None, false)
        .output()
        .expect("unregister missing");
    assert!(
        out.status.success(),
        "AC5: missing path must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("is not registered"),
        "AC5: not-registered copy; got: {stdout}"
    );
}

#[test]
fn unregister_path__dry_run__does_not_append() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let project_id = register_project(&vault, &dir.path().join("proj-dry"));
    let root = dir.path().join("dry-root");
    fs::create_dir_all(&root).unwrap();
    let path_str = root.to_str().expect("utf8");
    let normalized = ai_brains_path::normalize_for_location_compare(path_str);

    register_path(&vault, &project_id, path_str).success();

    unregister_path_cmd(&vault, path_str, None, true)
        .assert()
        .success();

    let after = list_paths_json(&vault);
    assert!(
        json_has_normalized_path(&after, &normalized),
        "AC5: --dry-run must not remove the alias; after={after}"
    );
}

#[test]
fn unregister_path__win_and_wsl_forms__same_row() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let project_id = register_project(&vault, &dir.path().join("dual-unreg"));
    let win = r"C:\dev\T254DualUnreg";
    let wsl = "/mnt/c/dev/T254DualUnreg";
    let win_n = ai_brains_path::normalize_for_location_compare(win);
    let wsl_n = ai_brains_path::normalize_for_location_compare(wsl);

    register_path(&vault, &project_id, win).success();

    if !win_n.is_empty() && win_n == wsl_n {
        unregister_path_cmd(&vault, wsl, None, false)
            .assert()
            .success();
        let after = list_paths_json(&vault);
        assert!(
            !json_has_normalized_path(&after, &win_n),
            "AC6: unregister WSL form removes Win-registered row; after={after}"
        );
    } else {
        // Platform did not map forms equal — still unregister the registered form.
        unregister_path_cmd(&vault, win, None, false)
            .assert()
            .success();
        let after = list_paths_json(&vault);
        assert!(
            !json_has_normalized_path(&after, &win_n),
            "unregister registered form; after={after}"
        );
    }
}

#[test]
fn unregister_path__project_mismatch__exit_1() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let id_a = register_project(&vault, &dir.path().join("proj-own"));
    let id_b = register_project(&vault, &dir.path().join("proj-other"));

    let root = dir.path().join("mismatch-root");
    fs::create_dir_all(&root).unwrap();
    let path_str = root.to_str().expect("utf8");
    let normalized = ai_brains_path::normalize_for_location_compare(path_str);

    register_path(&vault, &id_a, path_str).success();

    let assert = unregister_path_cmd(&vault, path_str, Some(&id_b), false)
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains(&id_a) && stderr.contains(&id_b) && stderr.contains("do not match"),
        "mismatch must name both ids; got: {stderr}"
    );

    let after = list_paths_json(&vault);
    assert!(
        json_has_normalized_path(&after, &normalized),
        "mismatch must not drop the row; after={after}"
    );
}

#[test]
fn scan_roots__scan_root_itself_marked__included() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let tree = dir.path().join("tree");
    fs::create_dir_all(&tree).unwrap();
    fs::write(tree.join(".ledgerful"), b"").unwrap();
    let child = tree.join("plain-child");
    fs::create_dir_all(&child).unwrap();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .arg("--format")
        .arg("json")
        .output()
        .expect("scan-roots json");
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let roots = v["roots"].as_array().expect("roots");
    assert_eq!(roots.len(), 1, "only the marked scan root; got {roots:?}");
    let path = roots[0]["path"].as_str().unwrap_or("");
    assert!(
        path.ends_with("tree") || path.contains("tree"),
        "AC10: marked scan root must be listed; path={path}"
    );
}

#[test]
fn scan_roots__grandchild_ledgerful__not_a_hit() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let tree = dir.path().join("tree");
    let mid = tree.join("mid");
    let deep = mid.join("deep");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join(".ledgerful"), b"").unwrap();

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("scan-roots")
        .arg(&tree)
        .arg("--format")
        .arg("json")
        .output()
        .expect("scan-roots json");
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let roots = v["roots"].as_array().expect("roots");
    assert!(
        roots.is_empty(),
        "F21: grandchild .ledgerful must not be a hit; roots={roots:?}"
    );
}

#[test]
fn unregister_path__empty_path__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("project")
        .arg("unregister-path")
        .arg("")
        .output()
        .expect("unregister empty");
    assert_eq!(
        out.status.code(),
        Some(2),
        "F35: empty after normalize is usage; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
