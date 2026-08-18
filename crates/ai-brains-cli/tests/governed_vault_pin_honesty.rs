//! T263 — Governed H1 honesty (granted-empty briefing, expand preview, list next_step).
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

/// Discovery grants only — no Approved decisions / Active conclusions (granted-empty).
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

/// AC4 — granted-empty project briefing names recall; authority arrays stay empty.
#[test]
fn briefing_project__granted_empty__empty_authority_names_recall() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);

    let human = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing human granted-empty");
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        md.contains("empty_authority") || md.contains("No current authority"),
        "granted-empty must keep empty_authority notice: {md}"
    );
    assert!(
        md.contains("recall"),
        "granted-empty next must name recall: {md}"
    );
    assert!(
        !md.contains("seed an Approved"),
        "granted-empty must not keep seed-Approved lead-in: {md}"
    );
    assert!(
        !md.contains("**Denied:**"),
        "granted-empty is allowed: {md}"
    );

    let json = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing json granted-empty");
    assert_eq!(
        json.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&json.stderr),
        String::from_utf8_lossy(&json.stdout)
    );
    let v = stdout_json(&json);
    assert_eq!(v["denied"], false, "packet={v}");
    let decisions = v["decisions"].as_array().expect("decisions");
    let conclusions = v["conclusions"].as_array().expect("conclusions");
    assert!(decisions.is_empty(), "authority must stay empty; {v}");
    assert!(conclusions.is_empty(), "authority must stay empty; {v}");
    let kinds: Vec<&str> = v["warnings"]
        .as_array()
        .map(|ws| {
            ws.iter()
                .filter_map(|w| w["kind"].as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert!(
        kinds.contains(&"empty_authority"),
        "JSON warning kind must stay empty_authority; got {v}"
    );
}

/// AC5 — unknown handle expand: kind Unknown, non-empty preview, exit 0.
#[test]
fn query_expand__unknown__preview_nonempty_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("expand")
        .arg(UNKNOWN)
        .arg("--project-id")
        .arg(PROJECT)
        .output()
        .expect("query expand unknown");
    assert_eq!(
        out.status.code(),
        Some(0),
        "Unknown expand stays exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["kind"], "Unknown", "packet={v}");
    let preview = v["preview"].as_str().unwrap_or("");
    assert!(
        !preview.is_empty(),
        "Unknown preview must be a non-empty SOOT; got {v}"
    );
}

/// AC6 — missing trace stays scalar JSON `null` + exit 0 (F6 frozen).
#[test]
fn query_trace__unknown__stdout_null_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("trace")
        .arg(UNKNOWN)
        .output()
        .expect("query trace unknown");
    assert_eq!(
        out.status.code(),
        Some(0),
        "missing trace stays exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let trimmed = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(
        trimmed, "null",
        "trace empty-success must be the token null; got {trimmed:?}"
    );
}

fn list_json(vault: &Path, noun: &str) -> std::process::Output {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg(noun)
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .unwrap_or_else(|_| panic!("{noun} list"))
}

fn assert_authorized_empty_list_next(noun: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let out = list_json(&vault, noun);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{noun} authorized-empty must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    let items = v["items"]
        .as_array()
        .unwrap_or_else(|| panic!("{noun} items; {v}"));
    assert!(items.is_empty(), "{noun} items must stay []; got {v}");
    let step = v["next_step"].as_str().unwrap_or("");
    assert!(
        step.contains("recall"),
        "{noun} authorized-empty next_step must name recall; got {v}"
    );
}

fn assert_denied_list_bootstrap_no_empty_next(noun: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let out = list_json(&vault, noun);
    assert_eq!(
        out.status.code(),
        Some(3),
        "{noun} denied list stays exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["code"], "POLICY_DENIED", "{noun} packet={v}");
    let hint = v["details"]["hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("policy bootstrap") || hint.contains("bootstrap"),
        "{noun} denied list must keep bootstrap hint; got {v}"
    );
    assert!(
        v.get("next_step").is_none(),
        "{noun} denied list must not get authorized-empty next_step; got {v}"
    );
    let blob = v.to_string();
    assert!(
        !blob.contains("empty_authority"),
        "{noun} denied list must not mention empty_authority; got {v}"
    );
}

/// AC7 — authorized-empty evidence list emits additive next_step naming recall.
#[test]
fn evidence_list__authorized_empty__next_step_names_recall() {
    assert_authorized_empty_list_next("evidence");
}

/// AC7 — authorized-empty source list emits additive next_step naming recall.
#[test]
fn source_list__authorized_empty__next_step_names_recall() {
    assert_authorized_empty_list_next("source");
}

/// AC7 — authorized-empty review list emits additive next_step naming recall.
#[test]
fn review_list__authorized_empty__next_step_names_recall() {
    assert_authorized_empty_list_next("review");
}

/// AC8 — denied evidence list stays exit 3 + bootstrap; no authorized-empty next_step.
#[test]
fn evidence_list__no_grants__exit_3_bootstrap_no_empty_next() {
    assert_denied_list_bootstrap_no_empty_next("evidence");
}

/// AC8 — denied source list stays exit 3 + bootstrap; no authorized-empty next_step.
#[test]
fn source_list__no_grants__exit_3_bootstrap_no_empty_next() {
    assert_denied_list_bootstrap_no_empty_next("source");
}

/// AC8 — denied review list stays exit 3 + bootstrap; no authorized-empty next_step.
#[test]
fn review_list__no_grants__exit_3_bootstrap_no_empty_next() {
    assert_denied_list_bootstrap_no_empty_next("review");
}
