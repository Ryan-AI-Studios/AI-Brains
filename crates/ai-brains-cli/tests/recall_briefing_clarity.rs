//! T202 — Recall + Briefing Clarity hermetic locks (F18 / AC2 / AC4 / AC9–AC10).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use serde_json::Value;
use tempfile::tempdir;

fn init_vault(vault_path: &std::path::Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// AC9 / AC10 — progressive missing project: example + exit 2
// ---------------------------------------------------------------------------

#[test]
fn query_progressive__missing_project_id__exit_2_with_example() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic_bin strips AI_BRAINS_PROJECT_ID; do not re-set it.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("progressive")
        .arg("why was graph backend replaced?")
        .output()
        .expect("query progressive missing project");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing project must exit 2 (USAGE); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("AI_BRAINS_PROJECT_ID"),
        "stderr must name AI_BRAINS_PROJECT_ID; got: {stderr}"
    );
    assert!(
        stderr.contains("--project-id") || stderr.contains("query progressive"),
        "stderr must include copy-paste example; got: {stderr}"
    );
}

#[test]
fn query_expand__missing_project_id__exit_2_with_example() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .output()
        .expect("query expand missing project");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing project must exit 2 (USAGE); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("AI_BRAINS_PROJECT_ID"),
        "stderr must name AI_BRAINS_PROJECT_ID; got: {stderr}"
    );
    assert!(
        stderr.contains("query expand") || stderr.contains("--project-id"),
        "stderr must include expand example; got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// F31 — query trace excluded from project gate
// ---------------------------------------------------------------------------

#[test]
fn query_trace__missing_project__still_exit_0_envelope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("trace")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .output()
        .expect("query trace");

    assert_eq!(
        out.status.code(),
        Some(0),
        "trace must remain empty-success; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    assert_ne!(
        trimmed, "null",
        "T291: missing project must not print the token null; got: {stdout}"
    );
    let v: Value = serde_json::from_str(trimmed).unwrap_or_else(|_| {
        panic!("missing-project trace stdout must be JSON envelope; got {stdout}")
    });
    assert_eq!(v["found"], false, "envelope found=false; got {v}");
    assert_eq!(v["api_version"], "1", "envelope api_version; got {v}");
    assert!(
        v["next_step"]
            .as_str()
            .unwrap_or("")
            .contains("query progressive"),
        "next_step must name progressive persist; got {v}"
    );
}

// ---------------------------------------------------------------------------
// AC2 / AC4 — semantic embedding status honesty
// ---------------------------------------------------------------------------

/// Parse the last JSON object line from stdout (tracing may precede it).
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

#[test]
fn recall__semantic_connection_refused__status_unreachable_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Port 1 is almost always closed → transport/unreachable class.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_EMBEDDING_URL", "http://127.0.0.1:1")
        .env("RUST_LOG", "off")
        .arg("recall")
        .arg("test query")
        .arg("--semantic")
        .arg("--no-bridge")
        .arg("--format")
        .arg("json")
        .arg("--global")
        .output()
        .expect("recall semantic refused");

    assert_eq!(
        out.status.code(),
        Some(0),
        "semantic soft-fail must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    let status = v
        .pointer("/embedding/status")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    assert_eq!(
        status, "unreachable",
        "connection-refused / network-send class → unreachable; full={v}"
    );
}

#[test]
fn recall__without_semantic__omits_embedding_field() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("--vault-path")
        .arg(&vault)
        .arg("recall")
        .arg("test query")
        .arg("--no-bridge")
        .arg("--format")
        .arg("json")
        .arg("--global")
        .output()
        .expect("recall non-semantic");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v = parse_last_json_object(&stdout);
    assert!(
        v.get("embedding").is_none(),
        "without --semantic, embedding must be omitted; got {v}"
    );
}
