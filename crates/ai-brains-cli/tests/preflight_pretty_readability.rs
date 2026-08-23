#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T219 — Preflight pretty readability hermetic suite (AC3–AC5, AC7, AC8 smoke).
//!
//! Pattern: tempdir vault + context multi-project registration + pin;
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

/// Seed a multi-section vault: safety constraints + decisions (index material).
fn seed_multi_section(vault: &Path) -> (std::path::PathBuf, String) {
    let dir = vault.parent().expect("vault parent").to_path_buf();
    let proj = dir.join("proj-pretty");
    let id = register_project(vault, &proj);
    // Safety markers (CONSTRAINT/HOTSPOT) + decisions for Memory Index.
    pin_memory(
        vault,
        &proj,
        &id,
        "CONSTRAINT: pretty must preserve section newlines under word budget",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "CONSTRAINT: Scope vocabulary is T207 SOOT not Repository",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: role prefixes are display-only stripped on pretty path",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: Memory Index lines must not lead with ASSISTANT on human output",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "HOTSPOT: crates/ai-brains-retrieval/src/word_budget.rs score=9",
    );
    (proj, id)
}

// ---------------------------------------------------------------------------
// AC3 + AC4 + AC5 — pretty multi-line + Scope + no leading ASSISTANT
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__multi_section__multiline_scope_no_assistant_prefix() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id) = seed_multi_section(&vault);

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--pretty", "-m", "800", "--format", "pretty"],
        Some(&id),
    );
    assert_eq!(code, 0, "pretty exit 0; stderr={stderr}");

    // AC3: multi-line body; blank line after --- header.
    let line_count = stdout.lines().count();
    assert!(
        line_count >= 4,
        "AC3: pretty body must be multi-line; lines={line_count}\n{stdout}"
    );
    assert!(
        stdout.contains("--- "),
        "AC3: expect legacy section header; got:\n{stdout}"
    );
    // `\n` separates `--- ` header from content (blank line after header).
    let has_header_spacing = stdout.lines().collect::<Vec<_>>().windows(2).any(|w| {
        let h = w[0].trim();
        h.starts_with("---") && h.ends_with("---") && w[1].trim().is_empty()
    });
    assert!(
        has_header_spacing || stdout.contains("--- Repository Bearings & Safety ---\n\n"),
        "AC3: blank line after --- header; got:\n{stdout}"
    );

    // AC4: Scope: vocabulary (not Repository:).
    assert!(
        stdout.contains("Scope:"),
        "AC4: must include Scope: line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Scope: project=") || stdout.contains("Scope: global"),
        "AC4: T207/T214 Scope vocabulary; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Repository:"),
        "AC4: must not use Repository: vocabulary; got:\n{stdout}"
    );
    assert!(
        !stdout.lines().any(|l| l.starts_with("Project:")),
        "AC4: no legacy Project: scope line; got:\n{stdout}"
    );

    // AC5: no displayed Memory Index / session line begins with ASSISTANT:
    for line in stdout.lines() {
        let t = line.trim();
        if t.starts_with("ASSISTANT:") {
            panic!("AC5: no display line may begin with ASSISTANT:; got {line}\n{stdout}");
        }
        // Numbered index lines
        if t.chars().next().is_some_and(|c| c.is_ascii_digit())
            && let Some(rest) = t.split_once(". ").map(|(_, r)| r)
        {
            assert!(
                !rest.starts_with("ASSISTANT:"),
                "AC5: index line must not start with ASSISTANT:; got {line}\n{stdout}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC7 — non-summary JSON: required text + word_count + sections array; multi-section text has \n
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__json_format__two_keys_and_newlines_in_text() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id) = seed_multi_section(&vault);

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--format", "json", "-m", "800"], Some(&id));
    assert_eq!(code, 0, "json exit 0; stderr={stderr}");

    let line = stdout.trim();
    // Compact JSON envelope (single top-level line) — T180.
    let v: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("AC7: must be valid JSON; err={e}; stdout:\n{stdout}");
    });
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"), "AC7: text key");
    assert!(obj.contains_key("word_count"), "AC7: word_count key");
    assert!(
        obj.get("sections").and_then(|s| s.as_array()).is_some(),
        "AC7: sections is array; got {:?}",
        obj.keys().collect::<Vec<_>>()
    );

    let text = obj["text"].as_str().expect("text string");
    assert!(
        text.contains('\n'),
        "AC7: multi-section text must contain newlines after F1; got text snippet: {:?}",
        text.chars().take(200).collect::<String>()
    );
    // JSON path: no Scope chrome.
    assert!(
        !text.starts_with("Scope:"),
        "JSON text must not include CLI Scope chrome"
    );
}

// ---------------------------------------------------------------------------
// AC8 — human --summary regression smoke (dual model unchanged)
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__summary_smoke__dual_model_unchanged() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id) = seed_multi_section(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--summary"], Some(&id));
    assert_eq!(code, 0, "summary exit 0; stderr={stderr}");
    assert!(
        stdout.contains("--- AI-Brains Preflight Summary ---"),
        "AC8: summary banner; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Scope: project=") || stdout.contains("Scope:"),
        "AC8: Scope line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Pinned memories:"),
        "AC8: vault pinned line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("In context"),
        "AC8: in-context dual block; got:\n{stdout}"
    );
    // Full pretty body chrome must not appear on summary path.
    assert!(
        !stdout.contains("--- Repository Bearings & Safety ---"),
        "AC8: summary must not dump full pretty body; got:\n{stdout}"
    );
}

/// Unique 200+ char, 6+ word seed so retrieval does not drop it as low-signal.
fn t250_long_seed() -> String {
    format!(
        "T250SEEDLONG unique pretty line-cap sentence that must stay visible after chrome strip {}",
        "word ".repeat(40).trim_end()
    )
}

fn seed_t250_long_and_overflow(vault: &Path) -> (std::path::PathBuf, String, String) {
    let dir = vault.parent().expect("vault parent").to_path_buf();
    let proj = dir.join("proj-t250");
    let id = register_project(vault, &proj);
    let long = t250_long_seed();
    pin_memory(
        vault,
        &proj,
        &id,
        "CONSTRAINT: t250 compact overflow safety item one must stay injectable",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "CONSTRAINT: t250 compact overflow safety item two must stay injectable",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "CONSTRAINT: t250 compact overflow safety item three must stay injectable",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "HOTSPOT: t250 compact overflow safety item four score=8 crates/ai-brains-cli",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "HOTSPOT: t250 compact overflow safety item five score=7 crates/ai-brains-cli",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "HOTSPOT: t250 compact overflow safety item six score=6 crates/ai-brains-cli",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item one must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item two must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item three must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item four must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item five must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item six must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item seven must appear in memory index",
    );
    pin_memory(
        vault,
        &proj,
        &id,
        "DECISION: t250 compact overflow index item eight must appear in memory index",
    );
    // Pin last so retrieval Recent (top-3 by updated_at) includes the full 200+ char body.
    pin_memory(vault, &proj, &id, &long);
    (proj, id, long)
}

// ---------------------------------------------------------------------------
// T250 AC10 — default --pretty Session/Recent line-cap 140
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__long_session_recent__line_capped_140() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id, seed) = seed_t250_long_and_overflow(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--pretty", "-m", "3000"], Some(&id));
    assert_eq!(code, 0, "pretty exit 0; stderr={stderr}");
    assert!(
        stdout.contains("T250SEEDLONG"),
        "AC10 seed prefix present; got:\n{stdout}"
    );
    // Index titles use 60-char `...` and are not the T250 line-cap path. Prefer
    // the Recent/Session display line (not `N. … -- just now`).
    let seed_line = stdout
        .lines()
        .find(|l| {
            l.contains("T250SEEDLONG")
                && !(l.trim().chars().next().is_some_and(|c| c.is_ascii_digit())
                    && l.contains(". "))
        })
        .expect("seed Recent/Session display line");
    assert!(
        seed_line.chars().count() <= 140,
        "AC10 seed line ≤140 chars; got {} `{seed_line}`",
        seed_line.chars().count()
    );
    assert!(
        seed_line.ends_with('…'),
        "AC10 truncated seed line must end with …; got `{seed_line}`"
    );
    assert!(
        !stdout.contains(&seed),
        "AC10 pretty must not emit the full 200+ char seed; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Scope:"),
        "AC10 Scope still present; got:\n{stdout}"
    );
    assert!(
        !stdout.lines().any(|l| l.trim().starts_with("ASSISTANT:")),
        "AC10: no display line may begin with ASSISTANT:; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// T250 AC11 — --compact --pretty tighter caps + F31
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__compact__tighter_caps_and_f31() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id, _seed) = seed_t250_long_and_overflow(&vault);

    let (std_code, std_out, std_err) =
        run_preflight(&vault, &["--pretty", "-m", "3000"], Some(&id));
    assert_eq!(std_code, 0, "standard pretty exit 0; stderr={std_err}");

    let (code, stdout, stderr) =
        run_preflight(&vault, &["--compact", "--pretty", "-m", "3000"], Some(&id));
    assert_eq!(code, 0, "compact pretty exit 0; stderr={stderr}");
    assert!(
        stdout.lines().count() < std_out.lines().count(),
        "AC11 compact must emit fewer lines than standard; compact={} standard={}\ncompact:\n{stdout}\nstandard:\n{std_out}",
        stdout.lines().count(),
        std_out.lines().count()
    );
    let has_safety_notice = stdout.contains("more safety entries");
    let has_recall_notice = stdout.contains("more via recall");
    assert!(
        has_safety_notice || has_recall_notice,
        "AC11 expect F31 +N more safety and/or +N more via recall; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// T250 AC12 — --compact --format json ignores compact; uncapped text
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__compact_json__uncapped_text_two_keys() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id, seed) = seed_t250_long_and_overflow(&vault);

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--compact", "--format", "json", "-m", "3000"],
        Some(&id),
    );
    assert_eq!(code, 0, "compact json exit 0; stderr={stderr}");

    let line = stdout.trim();
    let v: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("AC12: must be valid JSON; err={e}; stdout:\n{stdout}");
    });
    let obj = v.as_object().expect("object");
    assert!(obj.contains_key("text"), "AC12: text key");
    assert!(obj.contains_key("word_count"), "AC12: word_count key");
    assert!(
        obj.get("sections").and_then(|s| s.as_array()).is_some(),
        "AC12: sections is array; got {:?}",
        obj.keys().collect::<Vec<_>>()
    );
    let text = obj["text"].as_str().expect("text string");
    assert!(
        text.contains(&seed),
        "AC12: JSON text must keep full seeded body; snippet: {:?}",
        text.chars().take(240).collect::<String>()
    );
    assert!(
        !text.starts_with("Scope:"),
        "AC12: JSON text must not include CLI Scope chrome"
    );
    assert!(
        !stdout.contains("Scope:"),
        "AC12: no Scope chrome on JSON stdout; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// T250 AC13 — --summary --compact still summary (not pretty body)
// ---------------------------------------------------------------------------

#[test]
fn preflight_pretty__summary_compact__dual_model_unchanged() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let (_proj, id, _seed) = seed_t250_long_and_overflow(&vault);

    let (code, stdout, stderr) = run_preflight(&vault, &["--summary", "--compact"], Some(&id));
    assert_eq!(code, 0, "summary --compact exit 0; stderr={stderr}");
    assert!(
        stdout.contains("--- AI-Brains Preflight Summary ---"),
        "AC13: summary banner; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Pinned memories:"),
        "AC13: vault pinned line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("In context"),
        "AC13: in-context dual block; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("--- Repository Bearings"),
        "AC13: summary must not dump pretty body; got:\n{stdout}"
    );
}

/// T286 AC5 — pretty Index item 1 is the tagged DECISION pin, not `## Objective`.
#[test]
fn preflight__pretty_index_item1_is_decision_when_tagged_pin_exists() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let proj = dir.path().join("proj-t286-pretty");
    let id = register_project(&vault, &proj);
    let needle = format!("T286p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    pin_memory_tagged(
        &vault,
        &proj,
        &id,
        &format!("DECISION: {needle} pin"),
        "t286",
    );
    pin_memory(
        &vault,
        &proj,
        &id,
        "## Objective\nNewer review dump must not steal Index item 1",
    );

    let (code, stdout, stderr) = run_preflight(
        &vault,
        &["--pretty", "-m", "1500", "--no-hook-prompt"],
        Some(&id),
    );
    assert_eq!(code, 0, "AC5 exit 0; stderr={stderr}");

    let after = stdout
        .split("--- Memory Index (Briefing) ---")
        .nth(1)
        .unwrap_or("");
    let first = after
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("1."))
        .unwrap_or("");
    assert!(
        first.contains("DECISION:") || first.starts_with("1. DECISION:"),
        "AC5: Index line 1 must start with DECISION: after pretty strip; line={first:?}\n{stdout}"
    );
    assert!(
        !first.contains("## Objective"),
        "AC5: Index line 1 must not be ## Objective; line={first:?}\n{stdout}"
    );
    assert!(
        first.contains(&needle),
        "AC5: Index line 1 must include the tagged pin needle; line={first:?}\n{stdout}"
    );
}
