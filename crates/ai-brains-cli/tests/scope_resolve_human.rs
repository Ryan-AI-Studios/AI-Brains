//! T249 — hermetic `scope resolve` human/json/auto/reject CLI.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;

fn init_vault(vault: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("init")
        .assert()
        .success();
}

fn parse_pretty_json_object(stdout: &str) -> Value {
    let start = stdout
        .find('{')
        .unwrap_or_else(|| panic!("no JSON object in stdout: {stdout}"));
    serde_json::from_str(&stdout[start..]).unwrap_or_else(|e| {
        panic!("scope JSON parse failed: {e}; full_stdout={stdout}");
    })
}

/// AC3: `--format json --local` is pretty JSON with T180 keys.
#[test]
fn scope_resolve__format_json__pretty_t180_keys() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("scope resolve --format json --local");
    assert_eq!(
        out.status.code(),
        Some(0),
        "json resolve must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains('\n'),
        "json production style is pretty: {stdout}"
    );
    let v = parse_pretty_json_object(&stdout);
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("expected object; got {v}"));
    let keys: Vec<&String> = obj.keys().collect();
    assert!(
        obj.contains_key("api_version"),
        "frozen key api_version missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("scope"),
        "frozen key scope missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("confidence"),
        "frozen key confidence missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("authoritative"),
        "frozen key authoritative missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("evidence"),
        "frozen key evidence missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("warnings"),
        "frozen key warnings missing; keys={keys:?}"
    );
    assert!(
        obj.contains_key("alternatives"),
        "frozen key alternatives missing; keys={keys:?}"
    );
    assert_eq!(v["api_version"], "1");
}

/// AC4: `--format human --local` is the T160 human listing, not a JSON object.
#[test]
fn scope_resolve__format_human__scope_and_confidence_not_json() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("human")
        .arg("--local")
        .output()
        .expect("scope resolve --format human --local");
    assert_eq!(
        out.status.code(),
        Some(0),
        "human resolve must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("scope:"), "got:\n{stdout}");
    assert!(stdout.contains("confidence:"), "got:\n{stdout}");
    assert!(
        !stdout.trim_start().starts_with('{'),
        "human must not be a JSON object: {stdout}"
    );
}

/// AC5: unknown `--format xml` is clap exit 2 with zero stdout JSON.
#[test]
fn scope_resolve__format_xml__exit_2_no_stdout_json() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("xml")
        .output()
        .expect("scope resolve --format xml");
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown format must be clap exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim_start().starts_with('{'),
        "xml reject must not emit JSON; stdout={stdout}"
    );
}

/// AC16: case-sensitive tokens `JSON` and `Pretty` each exit 2 with zero stdout JSON.
#[test]
fn scope_resolve__format_JSON_and_Pretty__exit_2_no_stdout_json() {
    let json_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("JSON")
        .output()
        .expect("scope resolve --format JSON");
    assert_eq!(
        json_out.status.code(),
        Some(2),
        "JSON must be clap exit 2; stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let json_stdout = String::from_utf8_lossy(&json_out.stdout);
    assert!(
        !json_stdout.trim_start().starts_with('{'),
        "JSON reject must not emit JSON; stdout={json_stdout}"
    );

    let pretty_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("scope")
        .arg("resolve")
        .arg("--format")
        .arg("Pretty")
        .output()
        .expect("scope resolve --format Pretty");
    assert_eq!(
        pretty_out.status.code(),
        Some(2),
        "Pretty must be clap exit 2; stderr={}",
        String::from_utf8_lossy(&pretty_out.stderr)
    );
    let pretty_stdout = String::from_utf8_lossy(&pretty_out.stdout);
    assert!(
        !pretty_stdout.trim_start().starts_with('{'),
        "Pretty reject must not emit JSON; stdout={pretty_stdout}"
    );
}

/// AC6: default `--format auto` on a pipe is JSON.
#[test]
fn scope_resolve__default_auto_pipe__json() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("scope")
        .arg("resolve")
        .arg("--local")
        .output()
        .expect("scope resolve --local (auto/pipe)");
    assert_eq!(
        out.status.code(),
        Some(0),
        "auto pipe must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_pretty_json_object(&stdout);
    assert_eq!(v["api_version"], "1");
    assert!(v.get("authoritative").and_then(Value::as_bool).is_some());
}
