//! T346 — CLI Index-fill honesty, search alias, pretty BM25 hide, T111 empty.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const UNMATCHED: &str = "zzzzt346nomatch";
const HONESTY: &str = "No FTS hits; showing in-scope pins";
const DECISION: &str = "DECISION: we chose the empty-rescue path";

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

    let stdout = pretty_recall(&vault, "recall", UNMATCHED, &[]);
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

    let stdout = pretty_recall(&vault, "search", UNMATCHED, &[]);
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

    let stdout = pretty_recall(&vault, "recall", UNMATCHED, &["--global"]);
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

    let stdout = pretty_recall(&vault, "recall", UNMATCHED, &[]);
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
        .arg(UNMATCHED)
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
    assert!(
        v.get("hint").is_none() || v["hint"].is_null(),
        "AC10: hint omitted when results non-empty; got {v}"
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
}
