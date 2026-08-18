//! T261 — hermetic CLI contentless recall (AC6–AC11).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const DECISION_FOO: &str = "DECISION: we chose foo   for the bar path";

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
        .arg(DECISION_FOO)
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

fn pretty_has_hit_line(stdout: &str) -> bool {
    stdout.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("[score=") || t.starts_with("[rank=") || t.starts_with("[session=")
    })
}

#[test]
fn recall__empty_pretty__hint_no_hits__ac6() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall empty pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC6: must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "AC6: pretty must print Scope; stdout={stdout}"
    );
    assert!(
        stdout.contains("No results for ''"),
        "AC6: pretty must print No results for ''; stdout={stdout}"
    );
    assert!(
        !pretty_has_hit_line(&stdout),
        "AC6: must not print hit lines; stdout={stdout}"
    );
}

#[test]
fn recall__whitespace_pretty__no_hit_lines__ac7() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("   ")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall whitespace pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC7: must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No results"),
        "AC7: whitespace must print No results; stdout={stdout}"
    );
    assert!(
        !pretty_has_hit_line(&stdout),
        "AC7: whitespace must not print hit lines; stdout={stdout}"
    );
}

#[test]
fn recall__stopword_pretty__no_hits__ac8() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("the the the")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall stopword pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC8: must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No results"),
        "AC8: all-stopword must print No results; stdout={stdout}"
    );
    assert!(
        !pretty_has_hit_line(&stdout),
        "AC8: all-stopword must not print hit lines; stdout={stdout}"
    );
}

#[test]
fn recall__empty_json__results_empty__ac9() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("")
        .arg("--format")
        .arg("json")
        .arg("--no-bridge")
        .output()
        .expect("recall empty json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC9: must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json = parse_last_json_object(&stdout);
    let results = json
        .get("results")
        .and_then(|r| r.as_array())
        .expect("AC9: results array");
    assert!(results.is_empty(), "AC9: results must be []; json={json}");
    let hint = json
        .get("hint")
        .and_then(|h| h.as_str())
        .unwrap_or_default();
    assert!(
        hint.contains("No results"),
        "AC9: hint must contain No results; hint={hint}"
    );
}

#[test]
fn search__empty_pretty__alias__ac10() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("search")
        .arg("")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("search empty pretty");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC10: must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scope:"),
        "AC10: search alias must print Scope; stdout={stdout}"
    );
    assert!(
        stdout.contains("No results"),
        "AC10: search alias must print No results; stdout={stdout}"
    );
    assert!(
        !pretty_has_hit_line(&stdout),
        "AC10: search alias must not print hit lines; stdout={stdout}"
    );
}

#[test]
fn recall_stdin__piped_empty__short_circuit__ac11() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    pin_decision(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("-")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .write_stdin("   ")
        .output()
        .expect("recall - empty stdin");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC11: piped empty/whitespace stdin must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Query read from stdin is empty."),
        "AC11: must not use the T86 empty-stdin error; stderr={stderr}"
    );
    assert!(
        stdout.contains("No results"),
        "AC11: piped empty must print T207 empty chrome; stdout={stdout}"
    );
    assert!(
        !pretty_has_hit_line(&stdout),
        "AC11: piped empty must not print hit lines; stdout={stdout}"
    );
}
