//! T243 Phase 2 — `search` alias of `recall` + `--format text` honesty.
//!
//! Locks AC1, AC2, AC5, AC11 (help peers). AC15 Daily inventory is owned by
//! `cli_help_ia` / `memory_list_inventory` and must stay unchanged.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

/// Parse the last JSON object line from stdout (tracing may precede it).
fn parse_last_json_object(stdout: &str) -> Value {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(stdout);
    serde_json::from_str(line).unwrap_or_else(|e| {
        panic!("search/recall JSON parse failed: {e}; line={line}; full_stdout={stdout}");
    })
}

// ---------------------------------------------------------------------------
// AC1 — `search --help` is a visible alias of recall
// ---------------------------------------------------------------------------

#[test]
fn search__help__exit_0_alias_visible() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("search")
        .arg("--help")
        .output()
        .expect("search --help");

    assert_eq!(
        out.status.code(),
        Some(0),
        "search --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lower = stdout.to_ascii_lowercase();
    // clap 4.6 `search --help` is Recall help: Usage stays `recall`;
    // alias is in the about text (`Alias: \`search\``). Parent `--help`
    // is where clap lists `[aliases: search]`.
    let has_alias = stdout.contains("Alias: `search`")
        || stdout.contains("[aliases: search]")
        || stdout.contains("Aliases: search");
    let is_recall_surface = stdout.contains("Usage:")
        && stdout.contains("recall")
        && lower.contains("vault-first")
        && stdout.contains("--no-bridge");
    assert!(
        has_alias,
        "AC1/F2: search --help must name the search alias; got:\n{stdout}"
    );
    assert!(
        is_recall_surface,
        "AC1/F2: search --help must be the Recall surface (Usage recall + vault-first + --no-bridge); got:\n{stdout}"
    );
}

#[test]
fn root_help__lists_search_as_recall_alias() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--help")
        .output()
        .expect("root --help");
    assert_eq!(
        out.status.code(),
        Some(0),
        "root --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[aliases: search]") || stdout.contains("Aliases: search"),
        "F2: parent --help must list search as a visible alias of recall; got:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — `search --format json` is the live Recall JSON packet
// ---------------------------------------------------------------------------

#[test]
fn search__format_json_no_bridge__api_version_and_hits_array() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("search")
        .arg("t243-search-alias-json")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("search --format json");

    assert_eq!(
        out.status.code(),
        Some(0),
        "search --format json must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);

    // Live RecallResponse wire is `results` (no `api_version`, no `hits`).
    // Spec AC2 said api_version+hits; search *is* recall — lock the live shape
    // so a mis-route to progressive (also has `results`) fails.
    let results = v
        .get("results")
        .and_then(|r| r.as_array())
        .expect("Recall JSON must expose results as an array");
    assert!(
        results.iter().all(|item| item.is_object()),
        "results must be an array of objects; full={v}"
    );
    assert!(
        v.get("hits").is_none(),
        "Recall JSON must not invent a hits key; full={v}"
    );
    assert!(
        v.get("denied").is_none()
            && v.get("applied_scope").is_none()
            && v.get("query_trace_id").is_none(),
        "search JSON must be RecallResponse, not ProgressiveQueryResponse; full={v}"
    );
}

// ---------------------------------------------------------------------------
// AC5 — `recall --format text` is pretty (Scope chrome, not leading JSON)
// ---------------------------------------------------------------------------

#[test]
fn recall__format_text__prints_scope_not_leading_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let unique = "T243text-format-honesty-seed-xyzzy";
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(unique)
        .assert()
        .success();

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg(unique)
        .arg("--format")
        .arg("text")
        .arg("--limit")
        .arg("1")
        .arg("--no-bridge")
        .output()
        .expect("recall --format text");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall --format text must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "AC5: --format text must print pretty Scope chrome; got: {stdout}"
    );
    let first = stdout
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    assert!(
        !first.starts_with('{'),
        "AC5: first non-empty line must not be JSON; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC11 — help peers: recall mentions governed; progressive mentions recall+corpus
// ---------------------------------------------------------------------------

#[test]
fn recall__help__mentions_progressive_or_governed() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("recall")
        .arg("--help")
        .output()
        .expect("recall --help");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lower = stdout.to_ascii_lowercase();
    assert!(
        lower.contains("progressive") || lower.contains("governed"),
        "AC11: recall --help must mention progressive or governed; got:\n{stdout}"
    );
}

#[test]
fn query_progressive__help__mentions_recall_and_corpus() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("query")
        .arg("progressive")
        .arg("--help")
        .output()
        .expect("query progressive --help");

    assert_eq!(
        out.status.code(),
        Some(0),
        "query progressive --help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lower = stdout.to_ascii_lowercase();
    assert!(
        lower.contains("recall"),
        "AC11: query progressive --help must mention recall; got:\n{stdout}"
    );
    assert!(
        lower.contains("conclusion") || lower.contains("decision"),
        "AC11: query progressive --help must mention corpus (conclusions or decisions); got:\n{stdout}"
    );
}
