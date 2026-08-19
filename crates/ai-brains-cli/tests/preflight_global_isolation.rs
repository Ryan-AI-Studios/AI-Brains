#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T264 — `--global` preflight isolation hermetics (AC5–AC8, AC10, AC11).

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

fn has_hex8_tag(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 10 <= bytes.len() {
        if bytes[i] == b'[' && bytes[i + 9] == b']' {
            let inner = &bytes[i + 1..i + 9];
            if inner.iter().all(|b| b.is_ascii_hexdigit()) {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn line_starts_with_tag(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('[')
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

fn two_project_fixture(
    dir: &Path,
) -> (
    std::path::PathBuf,
    String,
    String,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let vault = dir.join("vault.db");
    init_vault(&vault);
    let proj_a = dir.join("proj-a");
    let proj_b = dir.join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "CONSTRAINT: alpha-only bearing must remain labeled under global preflight",
    );
    pin_memory(
        &vault,
        &proj_b,
        &id_b,
        "CONSTRAINT: beta-only bearing must remain labeled under global preflight",
    );
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "CONSTRAINT: two-line first stays tagged only on the opening line\nCONSTRAINT: continuation",
    );
    (vault, id_a, id_b, proj_a, proj_b)
}

#[test]
fn preflight_global_isolation__two_projects__pretty_labels_and_no_unlabeled_safety() {
    // AC5
    let dir = tempdir().unwrap();
    let (vault, _id_a, _id_b, _, _) = two_project_fixture(dir.path());

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--global", "--pretty", "--no-hook-prompt"], None);
    assert_eq!(code, 0, "AC5 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("alpha-only"),
        "AC5 alpha first-line body; got:\n{stdout}"
    );
    assert!(
        stdout.contains("beta-only"),
        "AC5 beta first-line body; got:\n{stdout}"
    );
    assert!(
        stdout.contains("CONSTRAINT: continuation") || stdout.contains("continuation"),
        "AC5 two-line continuation present; got:\n{stdout}"
    );

    let mut saw_two_line_first = false;
    let mut continuation_tagged = false;
    for line in stdout.lines() {
        let t = line.trim();
        if t.contains("alpha-only") || t.contains("beta-only") || t.contains("two-line first") {
            assert!(
                line_starts_with_tag(t),
                "AC5 Safety/item first line must start with [: {t}"
            );
        }
        if t.contains("two-line first") {
            saw_two_line_first = true;
        }
        if t.contains("CONSTRAINT: continuation") && line_starts_with_tag(t) {
            continuation_tagged = true;
        }
        if t.starts_with("--- Session:") || t.contains("--- Session:") {
            assert!(
                t.contains('['),
                "AC5 Session header must carry a project tag; got: {t}"
            );
        }
    }
    assert!(saw_two_line_first, "AC5 two-line first line present");
    assert!(
        !continuation_tagged,
        "AC5 F30: continuation line must not be retagged"
    );
}

#[test]
fn preflight_global_isolation__project_scoped__no_tags_no_span() {
    // AC6
    let dir = tempdir().unwrap();
    let (vault, id_a, _id_b, _, _) = two_project_fixture(dir.path());

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--pretty", "--no-hook-prompt"], Some(&id_a));
    assert_eq!(code, 0, "AC6 exit 0; stderr={stderr}");
    assert!(
        !has_hex8_tag(&stdout),
        "AC6 no [8hex] tags on project-scoped pretty; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("spans"),
        "AC6 no span line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("alpha-only"),
        "AC6 scoped A still shows A constraint; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("beta-only"),
        "AC6 scoped A must not show B constraint; got:\n{stdout}"
    );
}

#[test]
fn preflight_global_isolation__summary_span_and_json_key() {
    // AC7 + AC8
    let dir = tempdir().unwrap();
    let (vault, id_a, _id_b, _, _) = two_project_fixture(dir.path());

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--global", "--summary", "--no-hook-prompt"], None);
    assert_eq!(code, 0, "AC7 exit 0; stderr={stderr}");
    let span_line = stdout
        .lines()
        .find(|l| l.starts_with("In context spans "))
        .unwrap_or("");
    assert!(
        !span_line.is_empty(),
        "AC7 must print In context spans N projects; got:\n{stdout}"
    );
    let n: u32 = span_line
        .strip_prefix("In context spans ")
        .and_then(|r| r.strip_suffix(" projects"))
        .and_then(|r| r.trim().parse().ok())
        .unwrap_or(0);
    assert!(n >= 2, "AC7 N >= 2; got {n} from {span_line}\n{stdout}");

    let (code, scoped, stderr) =
        run_preflight(&vault, &["--summary", "--no-hook-prompt"], Some(&id_a));
    assert_eq!(code, 0, "AC7 scoped exit 0; stderr={stderr}");
    assert!(
        !scoped.contains("spans"),
        "AC7 project-scoped omits span line; got:\n{scoped}"
    );

    let (code, json_out, stderr) = run_preflight(
        &vault,
        &[
            "--global",
            "--summary",
            "--format",
            "json",
            "--no-hook-prompt",
        ],
        None,
    );
    assert_eq!(code, 0, "AC8 exit 0; stderr={stderr}");
    assert!(
        !json_out.contains("--- AI-Brains Preflight Summary ---"),
        "AC8 no human banner; got:\n{json_out}"
    );
    let v: serde_json::Value = serde_json::from_str(json_out.trim())
        .unwrap_or_else(|e| panic!("AC8 parse JSON: {e}; stdout:\n{json_out}"));
    assert_eq!(v["api_version"], "1");
    assert_eq!(v["scope"], "global");
    assert!(v.get("pinned").is_some());
    assert!(v.get("active_sessions").is_some());
    assert!(v.get("in_context_hotspots").is_some());
    assert!(v.get("in_context_decisions").is_some());
    assert!(v.get("in_context_constraints").is_some());
    assert!(v.get("word_count").is_some());
    let span = v["in_context_project_span"]
        .as_u64()
        .expect("AC8 in_context_project_span number");
    assert!(span >= 2, "AC8 span >= 2; got {span}\n{json_out}");

    let (code, scoped_json, stderr) = run_preflight(
        &vault,
        &["--summary", "--format", "json", "--no-hook-prompt"],
        Some(&id_a),
    );
    assert_eq!(code, 0, "AC8 scoped JSON exit 0; stderr={stderr}");
    let sv: serde_json::Value = serde_json::from_str(scoped_json.trim())
        .unwrap_or_else(|e| panic!("AC8 scoped parse: {e}; {scoped_json}"));
    assert!(
        sv.get("in_context_project_span").is_none(),
        "AC8 project-scoped omits span key; got {sv}"
    );
}

#[test]
fn preflight_global_isolation__three_a_one_b__b_appears_a_capped() {
    // AC10
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let proj_a = dir.path().join("proj-a");
    let proj_b = dir.path().join("proj-b");
    let id_a = register_project(&vault, &proj_a);
    let id_b = register_project(&vault, &proj_b);
    pin_memory(
        &vault,
        &proj_b,
        &id_b,
        "CONSTRAINT: B-only bearing must survive A recency monopoly under global",
    );
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "CONSTRAINT: A-one newer bearing must be capped at two per project",
    );
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "CONSTRAINT: A-two newer bearing must be capped at two per project",
    );
    pin_memory(
        &vault,
        &proj_a,
        &id_a,
        "CONSTRAINT: A-three newer bearing must be capped at two per project",
    );

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--global", "--pretty", "--no-hook-prompt"], None);
    assert_eq!(code, 0, "AC10 exit 0; stderr={stderr}");
    assert!(
        stdout.contains("B-only"),
        "AC10 Safety includes B; got:\n{stdout}"
    );
    let safety = safety_section(&stdout);
    assert!(
        safety.contains("B-only"),
        "AC10 Safety section includes B; got:\n{safety}"
    );
    let a_count = ["A-one", "A-two", "A-three"]
        .iter()
        .filter(|m| safety.contains(*m))
        .count();
    assert!(
        a_count <= 2,
        "AC10 at most 2 Safety items from A; got {a_count}\n{safety}\nfull:\n{stdout}"
    );
}

#[test]
fn preflight_global_isolation__compact_still_tagged() {
    // AC11
    let dir = tempdir().unwrap();
    let (vault, _, _, _, _) = two_project_fixture(dir.path());

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--global", "--pretty", "--compact", "--no-hook-prompt"],
        None,
    );
    assert_eq!(code, 0, "AC11 exit 0; stderr={stderr}");

    let mut safety_items = 0usize;
    let mut in_safety = false;
    let mut session_headers = 0usize;
    for line in stdout.lines() {
        let t = line.trim();
        if t.contains("Repository Bearings") {
            in_safety = true;
            continue;
        }
        if t.starts_with("--- ") && !t.contains("Repository Bearings") {
            in_safety = false;
        }
        if in_safety && !t.is_empty() && !t.starts_with('+') {
            safety_items += 1;
            assert!(
                line_starts_with_tag(t),
                "AC11 compact Safety line still tagged: {t}"
            );
        }
        if t.contains("--- Session:") {
            session_headers += 1;
            assert!(t.contains('['), "AC11 compact Session header tagged: {t}");
        }
    }
    assert!(
        safety_items <= 3,
        "AC11 compact Safety cap ≤3; got {safety_items}\n{stdout}"
    );
    assert!(
        session_headers <= 1,
        "AC11 compact Session cap ≤1; got {session_headers}\n{stdout}"
    );
}

#[test]
fn preflight_global_isolation__compact_json__two_keys_and_hex_tags() {
    // AC9
    let dir = tempdir().unwrap();
    let (vault, _, _, _, _) = two_project_fixture(dir.path());

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--global", "--format", "json", "--no-hook-prompt"],
        None,
    );
    assert_eq!(code, 0, "AC9 exit 0; stderr={stderr}");
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("AC9 parse: {e}; {stdout}"));
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"));
    assert!(obj.contains_key("word_count"));
    assert!(
        obj.get("sections").and_then(|s| s.as_array()).is_some(),
        "AC9: sections is array; got {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    let text = v["text"].as_str().unwrap_or("");
    assert!(
        has_hex8_tag(text),
        "AC9 JSON text carries [8hex] on Safety; got:\n{text}"
    );
}
