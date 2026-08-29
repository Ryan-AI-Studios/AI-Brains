//! T319 — vault memory_id vs governed handle namespace (CLI hermetics).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_control_plane::{
    StorePorts, SystemClock, issue_grant, make_principal, register_principal,
};
use ai_brains_core::ids::{PrincipalId, ProjectId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;
use uuid::Uuid;

const PROJECT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SCOPE: &str = "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const UNKNOWN: &str = "00000000-0000-0000-0000-000000000000";
const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
const SYSTEM_PRINCIPAL: &str = "a1b2a1b2-a1b2-a1b2-a1b2-a1b2a1b2a1b2";

const WRONG_NAMESPACE_PREVIEW: &str = "This UUID is a vault memory_id, not a governed handle.";
const WRONG_NAMESPACE_JSON_NEXT: &str = r#"ai-brains recall "what did we decide""#;
const WRONG_NAMESPACE_NEXT_LINE: &str = r#"next: ai-brains recall "what did we decide""#;

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn open_ports(vault_path: &Path) -> StorePorts {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = VaultConnection::open(vault_path, &key).expect("open vault");
    StorePorts::from_store(SqliteEventStore::new(conn))
}

fn seed_discovery_grants(vault_path: &Path) {
    let ports = open_ports(vault_path);
    let clock = SystemClock;
    let project = ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap());
    let scope = ScopeRef::Repository(project);
    let system = make_principal(
        PrincipalKind::System,
        PrincipalId::from_uuid(Uuid::parse_str(SYSTEM_PRINCIPAL).unwrap()),
        "cli-system",
    );
    register_principal(&ports.writer, &clock, &system).expect("register system");
    for cap in [
        GrantCapability::ReadEvidence,
        GrantCapability::ReadConclusions,
        GrantCapability::ReadDecisions,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            system.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .expect("system discovery grant");
    }
}

fn pin_via_hermetic_cmd(vault: &Path, content: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg("--")
        .arg(content)
        .assert()
        .success();
}

/// AC17 — memory_id from `memory list --format json`, not pin stdout (turn_id).
fn memory_id_from_list(vault: &Path, needle: &str) -> String {
    let listed = common::hermetic_cmd(vault)
        .arg("memory")
        .arg("list")
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("20")
        .output()
        .expect("memory list");
    assert!(
        listed.status.success(),
        "memory list failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let listed_json: Value = serde_json::from_str(String::from_utf8_lossy(&listed.stdout).trim())
        .unwrap_or_else(|e| panic!("memory list json: {e}"));
    listed_json["items"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|item| {
            let preview = item["preview"].as_str().unwrap_or("");
            if preview.contains(needle) {
                item["memory_id"].as_str().map(str::to_string)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("no memory_id for needle {needle}; list={listed_json}"))
}

fn stdout_json(out: &std::process::Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
        panic!(
            "expected JSON stdout; status={:?} stderr={} stdout={}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

fn seed_memory(vault: &Path) -> String {
    let needle = "T319-namespace-seed-memory";
    pin_via_hermetic_cmd(vault, needle);
    memory_id_from_list(vault, needle)
}

/// AC5 — expand JSON names namespace + next_step; applied_scope stays; exit 0.
#[test]
fn query_expand__memory_id__json_names_namespace() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let memory_id = seed_memory(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg(&memory_id)
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("query expand memory_id json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC5 exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["kind"], "Unknown", "AC5 kind Unknown; {v}");
    assert_eq!(
        v["preview"].as_str().unwrap_or(""),
        WRONG_NAMESPACE_PREVIEW,
        "AC5 F6 preview; {v}"
    );
    assert_eq!(
        v["next_step"].as_str().unwrap_or(""),
        WRONG_NAMESPACE_JSON_NEXT,
        "AC5 next_step; {v}"
    );
    assert!(
        v.get("applied_scope").and_then(|s| s.as_str()).is_some(),
        "AC5 applied_scope stays; {v}"
    );
}

/// AC6 — expand human: three nonempty lines Unknown / F6 preview / F6 next.
#[test]
fn query_expand__memory_id__human_three_lines() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let memory_id = seed_memory(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg(&memory_id)
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("query expand memory_id human");
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC6 exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout
        .trim_end()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 3, "AC6 three nonempty lines; got {lines:?}");
    assert_eq!(lines[0], "Unknown");
    assert_eq!(lines[1], WRONG_NAMESPACE_PREVIEW);
    assert_eq!(lines[2], WRONG_NAMESPACE_NEXT_LINE);
}

/// AC7 — evidence show names namespace; no applied_scope; exit 0.
#[test]
fn evidence_show__memory_id__names_namespace() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let memory_id = seed_memory(&vault);

    let json_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("show")
        .arg(&memory_id)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("evidence show memory_id json");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "AC7 json exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&json_out.stderr),
        String::from_utf8_lossy(&json_out.stdout)
    );
    let v = stdout_json(&json_out);
    assert_eq!(v["kind"], "Unknown", "AC7 kind; {v}");
    assert_eq!(
        v["preview"].as_str().unwrap_or(""),
        WRONG_NAMESPACE_PREVIEW,
        "AC7 preview; {v}"
    );
    assert_eq!(
        v["next_step"].as_str().unwrap_or(""),
        WRONG_NAMESPACE_JSON_NEXT,
        "AC7 next_step; {v}"
    );
    assert!(
        v.get("applied_scope").is_none(),
        "AC7 evidence must not have applied_scope; {v}"
    );

    let human_out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("show")
        .arg(&memory_id)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("human")
        .arg("--local")
        .output()
        .expect("evidence show memory_id human");
    assert_eq!(human_out.status.code(), Some(0));
    let md = String::from_utf8_lossy(&human_out.stdout);
    assert!(
        md.contains(WRONG_NAMESPACE_PREVIEW),
        "AC7 human preview; {md}"
    );
    assert!(
        md.contains(WRONG_NAMESPACE_NEXT_LINE),
        "AC7 human next; {md}"
    );
}

/// AC8 — source show miss stays exit 4 + details.hint; stderr CODE then bare hint.
#[test]
fn source_show__memory_id__not_found_hint_exit_4() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let memory_id = seed_memory(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("show")
        .arg(&memory_id)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("source show memory_id json");
    assert_eq!(
        out.status.code(),
        Some(4),
        "AC8 exit 4; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["code"], "NOT_FOUND", "AC8 code; {v}");
    assert_eq!(
        v["message"].as_str().unwrap_or(""),
        format!("source {memory_id}"),
        "AC8 message stays source {{id}}; {v}"
    );
    let hint = v["details"]["hint"].as_str().unwrap_or("");
    assert!(
        hint.contains(WRONG_NAMESPACE_PREVIEW),
        "AC8 hint preview; {v}"
    );
    assert!(
        hint.contains(r#"recall "what did we decide""#),
        "AC8 hint recall; {v}"
    );

    let human = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("show")
        .arg(&memory_id)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("human")
        .arg("--local")
        .output()
        .expect("source show memory_id human");
    assert_eq!(human.status.code(), Some(4));
    let err = String::from_utf8_lossy(&human.stderr);
    let lines: Vec<String> = err
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let expected = vec![
        format!("NOT_FOUND: source {memory_id}"),
        format!("{WRONG_NAMESPACE_PREVIEW} {WRONG_NAMESPACE_NEXT_LINE}"),
    ];
    assert_eq!(
        lines, expected,
        "AC8 exact stderr order CODE then bare hint; got {lines:?}"
    );
}

/// Evidence unknown-unknown stays T263 Handle not found. (Codex P3-002).
#[test]
fn evidence_show__unknown_unknown__handle_not_found() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("show")
        .arg(UNKNOWN)
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("evidence show unknown json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["kind"], "Unknown");
    assert_eq!(v["preview"], "Handle not found.");
    assert!(
        v.get("next_step").is_none(),
        "unknown-unknown must omit next_step; {v}"
    );
}

/// AC9 — T263 stay-green: non-memory UUID still two-line Unknown / Handle not found.
#[test]
fn query_expand__unknown_unknown__two_lines_no_next_step() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let human = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg(UNKNOWN)
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("query expand unknown human");
    assert_eq!(human.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&human.stdout);
    let lines: Vec<&str> = stdout
        .trim_end()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 2, "AC9 two nonempty lines; got {lines:?}");
    assert_eq!(lines[0], "Unknown");
    assert_eq!(lines[1], "Handle not found.");

    let json = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg(UNKNOWN)
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("query expand unknown json");
    assert_eq!(json.status.code(), Some(0));
    let v = stdout_json(&json);
    assert_eq!(v["preview"], "Handle not found.");
    assert!(v.get("next_step").is_none(), "AC9 no next_step; {v}");
}
