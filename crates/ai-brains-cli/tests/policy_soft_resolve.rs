//! T226 hermetic locks — `policy show|check` authoritative soft-resolve + F23 canonicalize.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_control_plane::{
    StorePorts, SystemClock, issue_grant, make_principal, register_principal, scope_identity_key,
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
    StorePorts::from_store(SqliteEventStore::new(conn))
}

fn seed_read_evidence_grant(vault_path: &Path) -> String {
    let ports = open_seeded_ports(vault_path);
    let clock = SystemClock;
    let principal = make_principal(
        PrincipalKind::Human,
        PrincipalId::from_uuid(Uuid::parse_str(PRINCIPAL).unwrap()),
        "test-human",
    );
    register_principal(&ports.writer, &clock, &principal).expect("register");
    let scope = ScopeRef::Repository(ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap()));
    issue_grant(
        &ports.writer,
        &clock,
        principal.id,
        scope.clone(),
        GrantCapability::ReadEvidence,
        Privacy::LocalOnly,
    )
    .expect("grant ReadEvidence");
    scope_identity_key(&scope)
}

/// AC4 — authoritative soft-fill + seeded grant: omit --scope → show non-empty grants.
#[test]
fn policy_show__authoritative_project_id__soft_resolve_seeded_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let canonical = seed_read_evidence_grant(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("policy")
        .arg("show")
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("policy show soft-resolve");

    assert_eq!(
        out.status.code(),
        Some(0),
        "AC4 soft-resolve show must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    let grants = v["grants"].as_array().expect("grants array");
    assert!(
        !grants.is_empty(),
        "seeded grant must appear in show; got {v}"
    );
    let has_read_evidence = grants.iter().any(|g| {
        g["capability"].as_str() == Some("ReadEvidence")
            && g["scope"].as_str() == Some(canonical.as_str())
    });
    assert!(
        has_read_evidence,
        "expected ReadEvidence grant on {canonical}; got {v}"
    );
}

/// AC5 — authoritative soft-fill check allow + canonical scope field (F23/M4).
#[test]
fn policy_check__authoritative_project_id__soft_resolve_seeded_allow() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let canonical = seed_read_evidence_grant(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("policy")
        .arg("check")
        .arg("--capability")
        .arg("ReadEvidence")
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("policy check soft-resolve");

    assert_eq!(
        out.status.code(),
        Some(0),
        "AC5 soft-resolve check must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["allowed"], true, "expected allow; got {v}");
    assert_eq!(
        v["scope"].as_str(),
        Some(canonical.as_str()),
        "scope must be canonical Repository:<project>; got {v}"
    );
    let expected = format!("Repository:{PROJECT}");
    assert_eq!(
        v["scope"].as_str(),
        Some(expected.as_str()),
        "canonical form lock; got {v}"
    );
}

/// AC12 — lowercase explicit scope kind → same grants / canonical strings (F23).
#[test]
fn policy_show__lowercase_explicit_scope__canonical_grants() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let canonical = seed_read_evidence_grant(&vault);
    let lowercase = format!("repository:{PROJECT}");

    let out_lower = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(&lowercase)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("policy show lowercase scope");

    assert_eq!(
        out_lower.status.code(),
        Some(0),
        "AC12 lowercase explicit must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out_lower.stderr),
        String::from_utf8_lossy(&out_lower.stdout)
    );
    let v_lower: Value = serde_json::from_slice(&out_lower.stdout).expect("json lower");

    let out_canon = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("show")
        .arg("--scope")
        .arg(&canonical)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("policy show canonical scope");

    assert_eq!(out_canon.status.code(), Some(0));
    let v_canon: Value = serde_json::from_slice(&out_canon.stdout).expect("json canon");

    let grants_lower = v_lower["grants"].as_array().expect("grants lower");
    let grants_canon = v_canon["grants"].as_array().expect("grants canon");
    assert_eq!(
        grants_lower.len(),
        grants_canon.len(),
        "lowercase and canonical must return same grant count; lower={v_lower} canon={v_canon}"
    );
    assert!(!grants_lower.is_empty(), "seeded grant must appear");
    for g in grants_lower {
        assert_eq!(
            g["scope"].as_str(),
            Some(canonical.as_str()),
            "grant scope must be canonical after F23; got {g}"
        );
        assert_eq!(g["capability"].as_str(), Some("ReadEvidence"));
    }
}
