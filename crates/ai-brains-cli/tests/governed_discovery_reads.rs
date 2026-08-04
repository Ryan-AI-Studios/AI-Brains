//! T203 hermetic locks — governed discovery read paths.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_control_plane::{
    AllowAllPolicy, ObserveSourceRequest, Sha256FingerprinterPort, SourceContent, StorePorts,
    SystemClock, issue_grant, make_principal, observe_source, register_principal,
    scope_identity_key,
};
use ai_brains_core::ids::{PrincipalId, ProjectId};
use ai_brains_core::principal::PrincipalKind;
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::{GrantCapability, ScopeRef};
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;
use uuid::Uuid;

const PROJECT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const PRINCIPAL: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn open_seeded_ports(vault_path: &Path) -> StorePorts {
    // Hermetic CLI vaults use zero SQLCipher key when AI_BRAINS_ALLOW_ZERO_KEY=1.
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = VaultConnection::open(vault_path, &key).expect("open vault");
    // Already migrated by init.
    StorePorts::from_store(SqliteEventStore::new(conn))
}

fn seed_grant_and_source(vault_path: &Path) -> String {
    let ports = open_seeded_ports(vault_path);
    let clock = SystemClock;
    let principal = make_principal(
        PrincipalKind::Human,
        PrincipalId::from_uuid(Uuid::parse_str(PRINCIPAL).unwrap()),
        "test-human",
    );
    register_principal(&ports.writer, &clock, &principal).expect("register");
    let project = ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap());
    let scope = ScopeRef::Repository(project);
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope.clone(),
        GrantCapability::ReadEvidence,
        Privacy::LocalOnly,
    )
    .expect("grant ReadEvidence");
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope.clone(),
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .expect("grant ReadConclusions");

    let fp = Sha256FingerprinterPort::new();
    observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &AllowAllPolicy,
        ObserveSourceRequest {
            principal: principal.id,
            scope: scope.clone(),
            kind: SourceKind::File,
            display_name: "seeded".into(),
            locator: Some("/seeded.md".into()),
            // Evidence summary is "Observed {display_name}" (not raw content).
            content: SourceContent::Bytes(b"body for seeded source\n".to_vec()),
            privacy: Privacy::LocalOnly,
            run_invalidation: false,
        },
    )
    .expect("observe");

    scope_identity_key(&scope)
}

#[test]
fn source_list__empty_with_grant__exit_0_items_array() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let ports = open_seeded_ports(&vault);
    let clock = SystemClock;
    let principal = make_principal(
        PrincipalKind::Human,
        PrincipalId::from_uuid(Uuid::parse_str(PRINCIPAL).unwrap()),
        "test-human",
    );
    register_principal(&ports.writer, &clock, &principal).unwrap();
    let scope = ScopeRef::Repository(ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap()));
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope.clone(),
        GrantCapability::ReadEvidence,
        Privacy::LocalOnly,
    )
    .unwrap();

    let scope_key = scope_identity_key(&scope);
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("list")
        .arg("--scope")
        .arg(&scope_key)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("source list");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(v["items"].as_array().expect("items").is_empty());
    assert_eq!(v["more_available"], false);
}

#[test]
fn source_list__happy__contains_seeded_source() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope_key = seed_grant_and_source(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("list")
        .arg("--scope")
        .arg(&scope_key)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("source list happy");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    let items = v["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["display_name"], "seeded");
}

#[test]
fn evidence_list__empty_and_query_hit() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope_key = seed_grant_and_source(&vault);

    let empty_scope = format!("Repository:{}", "cccccccc-cccc-cccc-cccc-cccccccccccc");
    // Grant on seeded scope only — empty other scope still allowed after grant on that scope.
    // Use seeded scope with no extra evidence → at least one from observe.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("list")
        .arg("--scope")
        .arg(&scope_key)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("evidence list");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(!v["items"].as_array().expect("items").is_empty());

    let fts = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("list")
        .arg("--scope")
        .arg(&scope_key)
        .arg("--query")
        .arg("seeded")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("evidence fts");

    assert_eq!(
        fts.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&fts.stderr),
        String::from_utf8_lossy(&fts.stdout)
    );
    let v: Value = serde_json::from_slice(&fts.stdout).expect("json");
    assert_eq!(v["items"].as_array().expect("items").len(), 1);

    // Unrelated scope without grant → deny (sanity)
    let denied = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("evidence")
        .arg("list")
        .arg("--scope")
        .arg(&empty_scope)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("evidence deny");
    assert_eq!(denied.status.code(), Some(3));
    let err: Value = serde_json::from_slice(&denied.stdout).expect("err json");
    assert_eq!(err["code"], "POLICY_DENIED");
    assert!(
        err["details"]["hint"]
            .as_str()
            .map(|h| !h.is_empty())
            .unwrap_or(false),
        "deny must carry details.hint; got {err}"
    );
}

#[test]
fn review_list__authoritative_project_id__exit_0_without_scope() {
    // AC4: hermetic_bin strips ambient PROJECT_ID; we set it after strip.
    // explicit ProjectId → High / authoritative even without vault project row.
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Grant ReadConclusions so list is allowed (empty items OK).
    let ports = open_seeded_ports(&vault);
    let clock = SystemClock;
    let principal = make_principal(
        PrincipalKind::Human,
        PrincipalId::from_uuid(Uuid::parse_str(PRINCIPAL).unwrap()),
        "test-human",
    );
    register_principal(&ports.writer, &clock, &principal).unwrap();
    let scope = ScopeRef::Repository(ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap()));
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope,
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .unwrap();

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("review")
        .arg("list")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("review list soft");

    assert_eq!(
        out.status.code(),
        Some(0),
        "AC4 soft-resolve authoritative must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(v["items"].as_array().is_some());
}

#[test]
fn review_list__non_authoritative__exit_2_fail_usage() {
    // AC5: PROJECT_ID unset + --no-project-context → fail_usage template (not clap required).
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("review")
        .arg("list")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("review list missing scope");

    assert_eq!(
        out.status.code(),
        Some(2),
        "AC5 must exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("required arguments were not provided"),
        "must not be clap required text: {stderr}"
    );
    assert!(
        stderr.contains("scope resolve") || stderr.contains("--scope"),
        "fail_usage template expected: {stderr}"
    );
    assert!(
        stderr.contains("not filled silently") || stderr.contains("not authoritative"),
        "non-authoritative note expected: {stderr}"
    );
}

#[test]
fn source_list__policy_denied__hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("list")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("source list deny");

    assert_eq!(out.status.code(), Some(3));
    let err: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(err["code"], "POLICY_DENIED");
    assert!(
        err["details"]["hint"]
            .as_str()
            .map(|h| !h.is_empty())
            .unwrap_or(false),
        "hint required: {err}"
    );
}

#[test]
fn source_show__missing_scope_non_authoritative__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("show")
        .arg("00000000-0000-0000-0000-0000000000a1")
        .arg("--format")
        .arg("json")
        .arg("--local")
        .output()
        .expect("source show");

    assert_eq!(
        out.status.code(),
        Some(2),
        "F7 show missing-scope → exit 2 not 6; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.to_lowercase().contains("invalid_payload"),
        "must not reintroduce exit-6 class: {stderr}"
    );
}
