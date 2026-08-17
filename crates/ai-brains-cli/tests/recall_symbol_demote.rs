//! T260 — hermetic CLI: default recall excludes T70 stubs (AC3/AC5).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

const DECISION_FOO: &str = "DECISION: we chose foo for the bar path";
const STUB_FOO: &str = "Module foo (src/foo.rs:1)";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
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

fn contents(v: &Value) -> Vec<String> {
    v.get("results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|hit| {
                    hit.get("content")
                        .and_then(|c| c.as_str())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn recall__hermetic_decision_vs_stub__default_excludes_stub() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(DECISION_FOO)
        .assert()
        .success();
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(STUB_FOO)
        .assert()
        .success();

    let out = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg("what did we decide about foo")
        .arg("--no-bridge")
        .arg("--format")
        .arg("json")
        .output()
        .expect("recall json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json = parse_last_json_object(&stdout);
    let hits = contents(&json);
    assert!(
        hits.iter().any(|c| c.contains("DECISION: we chose foo")),
        "AC3: DECISION must be present; hits={hits:?} stdout={stdout}"
    );
    assert!(
        hits.iter()
            .all(|c| !c.contains("Module foo (src/foo.rs:1)")),
        "AC3: T70 stub must be absent from default recall; hits={hits:?} stdout={stdout}"
    );
}

#[test]
fn recall__hermetic_symbols_flag__returns_stub_and_pretty_marker() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(DECISION_FOO)
        .assert()
        .success();
    common::hermetic_cmd(&vault)
        .arg("pin")
        .arg(STUB_FOO)
        .assert()
        .success();

    let pretty = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg("foo")
        .arg("--symbols")
        .arg("--no-bridge")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("recall --symbols pretty");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains("[symbol]"),
        "AC4: pretty --symbols must badge the stub; stdout={pretty_out}"
    );
    assert!(
        pretty_out.contains("Module foo (src/foo.rs:1)"),
        "AC4: stub content must appear; stdout={pretty_out}"
    );

    let json_out = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg("foo")
        .arg("--symbols")
        .arg("--no-bridge")
        .arg("--format")
        .arg("json")
        .output()
        .expect("recall --symbols json");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    let json = parse_last_json_object(&stdout);
    let hits = contents(&json);
    assert!(
        hits.iter().any(|c| c.contains("Module foo (src/foo.rs:1)")),
        "AC4: JSON content is the raw stub; hits={hits:?}"
    );
    assert!(
        hits.iter().all(|c| !c.starts_with("[symbol]")),
        "AC4: JSON must not prefix [symbol]; hits={hits:?}"
    );
    assert!(
        json.get("results")
            .and_then(|r| r.as_array())
            .is_some_and(|arr| arr.iter().all(|h| h.get("is_symbol").is_none())),
        "AC4/F11: no is_symbol DTO key; json={json}"
    );
}

#[test]
fn search__symbols_flag__accepted() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_cmd(&vault)
        .arg("search")
        .arg("foo")
        .arg("--symbols")
        .arg("--no-bridge")
        .arg("--format")
        .arg("json")
        .output()
        .expect("search --symbols");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC5: search --symbols must be accepted; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
