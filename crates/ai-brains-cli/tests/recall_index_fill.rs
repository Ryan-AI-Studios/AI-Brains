//! T346 — CLI Index-fill honesty, search alias, pretty BM25 hide, T111 empty.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const UNMATCHED: &str = "zzzzt346nomatch";
const T315_QUERY: &str = "what did we decide";
const HONESTY: &str = "No FTS hits; showing in-scope pins";
const DECISION: &str = "DECISION: we chose the empty-rescue path";
const T315_DUMP: &str = r#"next: ai-brains recall "what did we decide""#;
const AUTH_NO_PHRASE: &str = "DECISION: Track 0008 shipped";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn pin_decision(vault: &Path) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(DECISION)
        .assert()
        .success();
}

fn parse_last_json_object(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("recall JSON parse failed: {e}; line={line}; full_stdout={stdout}");
    })
}

fn pretty_recall(vault: &Path, cmd: &str, query: &str, extra: &[&str]) -> String {
    let mut c = common::hermetic_cmd(vault);
    c.arg("--log-format")
        .arg("off")
        .arg(cmd)
        .arg(query)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("5");
    for a in extra {
        c.arg(a);
    }
    let out = c.output().expect("recall/search pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn recall_index_fill__fts_empty_authority_pin__honesty_and_hits() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let stdout = pretty_recall(&vault, "recall", T315_QUERY, &[]);
    assert!(
        stdout.contains(HONESTY),
        "AC1: exact honesty SOOT missing; stdout={stdout}"
    );
    let honesty_idx = stdout.find(HONESTY).expect("honesty present after assert");
    assert!(
        stdout.contains(DECISION) || stdout.contains("empty-rescue"),
        "AC1: pretty must show the pin; stdout={stdout}"
    );
    let pin_idx = stdout
        .find(DECISION)
        .or_else(|| stdout.find("empty-rescue"))
        .expect("pin text");
    assert!(
        honesty_idx < pin_idx,
        "AC1: honesty must print after headers and before the pin; stdout={stdout}"
    );
    let after_scope = stdout.find("Scope:").expect("Scope");
    assert!(
        after_scope < honesty_idx,
        "AC1: honesty after Scope; stdout={stdout}"
    );
    assert!(
        !stdout.contains("score="),
        "AC1: pretty fill must not print score=; stdout={stdout}"
    );
}

#[test]
fn search_index_fill__alias_shares_recall_path() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let stdout = pretty_recall(&vault, "search", T315_QUERY, &[]);
    assert!(
        stdout.contains(HONESTY),
        "AC5: search alias must share fill honesty; stdout={stdout}"
    );
    assert!(
        stdout.contains("empty-rescue"),
        "AC5: search alias must show the pin; stdout={stdout}"
    );
    assert!(
        !stdout.contains("score="),
        "AC5: search pretty must hide BM25 score=; stdout={stdout}"
    );
}

#[test]
fn recall_index_fill__global__no_fill_t111_hint() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let stdout = pretty_recall(&vault, "recall", T315_QUERY, &["--global"]);
    assert!(
        !stdout.contains(HONESTY),
        "AC8: --global must not Index-fill; stdout={stdout}"
    );
    assert!(
        stdout.contains("No results for"),
        "AC8: T111 hint still fires; stdout={stdout}"
    );
}

#[test]
fn recall_index_fill__no_pins__t111_hint() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let stdout = pretty_recall(&vault, "recall", T315_QUERY, &[]);
    assert!(
        !stdout.contains(HONESTY),
        "AC9: no pins → no fill honesty; stdout={stdout}"
    );
    assert!(
        stdout.contains("No results for"),
        "AC9: T111 hint when fill also empty; stdout={stdout}"
    );
}

#[test]
fn recall_index_fill__source_index__json_omits_score() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg(T315_QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("5")
        .output()
        .expect("recall json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC10: json must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let results = v["results"].as_array().expect("results array");
    assert!(
        !results.is_empty(),
        "AC10: json fill results; stdout={stdout}"
    );
    assert_eq!(results[0]["source"], "index");
    assert!(
        results[0].get("score").is_none(),
        "AC10: json omits score; got {}",
        results[0]
    );
    assert_eq!(results[0]["score_kind"], "bm25");
    assert_eq!(v["fill_kind"], "index");
    assert_eq!(v["hint"], HONESTY);
    assert!(
        v.get("empty_kind").is_none() || v["empty_kind"].is_null(),
        "fill omits empty_kind; got {v}"
    );
}

#[test]
fn recall_pretty__bm25__omits_score() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg("t346fts token stays lexical")
        .assert()
        .success();

    let pretty = pretty_recall(&vault, "recall", "t346fts", &[]);
    assert!(
        pretty.contains("t346fts"),
        "AC4: pretty must show the FTS hit; stdout={pretty}"
    );
    assert!(
        !pretty.contains("score="),
        "AC4: pretty BM25 must hide score=; stdout={pretty}"
    );

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("t346fts")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall json fts");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let results = v["results"].as_array().expect("results");
    assert!(!results.is_empty(), "AC4: json FTS hit; stdout={stdout}");
    assert_eq!(results[0]["source"], "fts");
    assert!(
        results[0]["score"].as_f64().is_some(),
        "AC4: json keeps numeric BM25 score; got {}",
        results[0]
    );
    assert!(
        v.get("fill_kind").is_none() || v["fill_kind"].is_null(),
        "AC6: FTS-only omits fill_kind; got {v}"
    );
}

fn hermetic() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn register_project(vault: &Path, work_dir: &Path) -> String {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .arg("--no-auto-bind")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = fs::read_to_string(&env_path).expect(".env");
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            let id = rest.trim();
            assert!(!id.is_empty());
            return id.to_string();
        }
    }
    panic!("AI_BRAINS_PROJECT_ID missing from .env");
}

fn register_path(vault: &Path, work_dir: &Path, project_id: &str) {
    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("register-path")
        .arg(project_id)
        .arg(work_dir)
        .assert()
        .success();
}

fn pin_owned(vault: &Path, work_dir: &Path, project_id: &str, content: &str) {
    let env_content = fs::read_to_string(work_dir.join(".env")).expect(".env");
    let mut session_id = String::new();
    for line in env_content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            session_id = rest.trim().to_string();
        }
    }
    assert!(!session_id.is_empty());
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

fn owned_cmd<'a>(vault: &'a Path, work_dir: &'a Path, project_id: &'a str) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .arg("--log-format")
        .arg("off");
    if let Ok(env_content) = fs::read_to_string(work_dir.join(".env")) {
        for line in env_content.lines() {
            if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
                cmd.env("AI_BRAINS_SESSION_ID", rest.trim());
            }
        }
    }
    cmd
}

#[test]
fn recall_index_fill__unmatched__query_miss_not_unowned() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    pin_owned(&vault, &work, &pid, DECISION);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(UNMATCHED)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall unmatched json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    assert_eq!(
        v["results"].as_array().map(|a| a.len()),
        Some(0),
        "stdout={stdout}"
    );
    assert_eq!(v["empty_kind"], "query_miss");
    assert!(
        v["project_memory_count"].as_u64().unwrap_or(0) >= 1,
        "got {v}"
    );

    let pretty = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(UNMATCHED)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall unmatched pretty");
    let pstdout = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        !pstdout.contains(HONESTY),
        "unmatched must not print fill honesty; stdout={pstdout}"
    );
    assert!(
        pstdout.contains("This project has") || pstdout.contains("No results"),
        "census/T111 missing; stdout={pstdout}"
    );
}

#[test]
fn search_index_fill__json__fill_kind_and_hint() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    pin_owned(&vault, &work, &pid, DECISION);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("search")
        .arg(T315_QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("5")
        .output()
        .expect("search json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let results = v["results"].as_array().expect("results");
    assert!(!results.is_empty(), "stdout={stdout}");
    assert_eq!(results[0]["source"], "index");
    assert_eq!(v["fill_kind"], "index");
    assert_eq!(v["hint"], HONESTY);
}

#[test]
fn sync_query__unmatched__no_index_fill_honesty() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    pin_owned(&vault, &work, &pid, DECISION);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("sync")
        .arg("query")
        .arg(UNMATCHED)
        .arg("--no-bridge")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("sync query");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(HONESTY),
        "AC11: unmatched sync must not Index-fill; stdout={stdout}"
    );
    assert!(
        stdout.contains("No results") || stdout.contains("This project has"),
        "AC11: empty census/T111; stdout={stdout}"
    );
}

fn dump_and_authority(vault: &Path, work: &Path, pid: &str) {
    pin_owned(vault, work, pid, T315_DUMP);
    pin_owned(vault, work, pid, AUTH_NO_PHRASE);
}

#[test]
fn recall_index_fill__t315_dump_present__json_fills_authority() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    dump_and_authority(&vault, &work, &pid);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(T315_QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("3")
        .output()
        .expect("recall json dump");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let results = v["results"].as_array().expect("results");
    assert!(!results.is_empty(), "stdout={stdout}");
    assert_eq!(results[0]["source"], "index");
    assert_eq!(v["fill_kind"], "index");
    assert_eq!(v["hint"], HONESTY);
    let blob = stdout.to_string();
    assert!(
        !blob.contains("next: ai-brains recall"),
        "AC4: dump body must be absent; stdout={stdout}"
    );
    assert!(
        results[0]["content"]
            .as_str()
            .is_some_and(|c| c.contains("DECISION:")),
        "AC4: authority pin; got {}",
        results[0]
    );
}

#[test]
fn recall_index_fill__t315_dump_present__pretty_prints_honesty() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    dump_and_authority(&vault, &work, &pid);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("recall")
        .arg(T315_QUERY)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("3")
        .output()
        .expect("recall pretty dump");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(HONESTY),
        "AC4b: honesty missing; stdout={stdout}"
    );
    assert!(
        stdout.contains("Track 0008") || stdout.contains("DECISION:"),
        "AC4b: authority pin missing; stdout={stdout}"
    );
    assert!(
        !stdout.contains("next: ai-brains recall"),
        "AC4b: dump body must be absent; stdout={stdout}"
    );
}

#[test]
fn search_index_fill__t315_dump_present__json_fills_authority() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    dump_and_authority(&vault, &work, &pid);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("search")
        .arg(T315_QUERY)
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("3")
        .output()
        .expect("search json dump");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    assert_eq!(v["fill_kind"], "index");
    assert_eq!(v["results"][0]["source"], "index");
    assert!(
        !stdout.contains("next: ai-brains recall"),
        "AC8: dump body must be absent; stdout={stdout}"
    );
}

#[test]
fn sync_query__t315_dump_present__pretty_honesty() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let work = dir.path().join("proj");
    let pid = register_project(&vault, &work);
    register_path(&vault, &work, &pid);
    dump_and_authority(&vault, &work, &pid);

    let out = owned_cmd(&vault, &work, &pid)
        .arg("sync")
        .arg("query")
        .arg(T315_QUERY)
        .arg("--no-bridge")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("sync query dump");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(HONESTY),
        "AC9: sync pretty honesty missing; stdout={stdout}"
    );
    assert!(
        !stdout.contains("next: ai-brains recall"),
        "AC9: dump body must be absent; stdout={stdout}"
    );
}
