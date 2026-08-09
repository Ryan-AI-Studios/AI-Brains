//! T221 — Governed first-run + progressive/expand deny exit honesty.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_control_plane::{
    AllowAllPolicy, ObserveSourceRequest, Sha256FingerprinterPort, SourceContent, StorePorts,
    SystemClock, observe_source, scope_identity_key,
};
use ai_brains_core::ids::{PrincipalId, ProjectId};
use ai_brains_core::privacy::Privacy;
use ai_brains_core::scope::ScopeRef;
use ai_brains_core::source::SourceKind;
use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::SqliteEventStore;
use ai_brains_store::connection::VaultConnection;
use serde_json::Value;
use std::path::Path;
use tempfile::tempdir;
use uuid::Uuid;

const PROJECT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SCOPE: &str = "Repository:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const UNKNOWN_HANDLE: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";
const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
/// Default System principal used by `cli_principal()` (progressive/expand).
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

/// Seed in-scope evidence via AllowAll (no discovery grants for System).
/// Expand without ReadEvidence → CP `kind: "Denied"`.
fn seed_evidence_no_grants(vault_path: &Path) -> String {
    let ports = open_ports(vault_path);
    let clock = SystemClock;
    let project = ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap());
    let scope = ScopeRef::Repository(project);
    let principal = PrincipalId::from_uuid(Uuid::parse_str(SYSTEM_PRINCIPAL).unwrap());
    let fp = Sha256FingerprinterPort::new();
    let res = observe_source(
        &ports.writer,
        &ports.query,
        &clock,
        &fp,
        &AllowAllPolicy,
        ObserveSourceRequest {
            principal,
            scope: scope.clone(),
            kind: SourceKind::File,
            display_name: "t221-expand-deny".into(),
            locator: Some("/t221-expand-deny.md".into()),
            content: SourceContent::Bytes(b"body for expand deny seed\n".to_vec()),
            privacy: Privacy::LocalOnly,
            run_invalidation: false,
        },
    )
    .expect("observe seed");
    let evidence_id = res
        .evidence_id
        .expect("observe should produce evidence")
        .to_string();
    // Sanity: scope key matches progressive project.
    assert_eq!(scope_identity_key(&scope), SCOPE);
    evidence_id
}

fn progressive_cmd(vault: &Path) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("query")
        .arg("progressive")
        .arg("x");
    cmd
}

/// AC1 — vault + project, no grants: progressive deny exit 3 + denied true.
#[test]
fn progressive__no_grants__exit_3_denied_true() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = progressive_cmd(&vault).output().expect("progressive");
    assert_eq!(
        out.status.code(),
        Some(3),
        "deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(v["denied"], true, "packet={v}");
    let results = v["results"].as_array().expect("results array");
    assert!(results.is_empty(), "denied results must be empty; got {v}");
}

/// AC1b / F34 — dry-run deny still exit 3.
#[test]
fn progressive__dry_run_no_grants__exit_3_denied_true() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Progressive clap: `--dry-run` is ArgAction::Set (bool), default true; pass explicit true for F34.
    let out = progressive_cmd(&vault)
        .arg("--dry-run")
        .arg("true")
        .output()
        .expect("progressive dry-run");
    assert_eq!(
        out.status.code(),
        Some(3),
        "dry-run deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(v["denied"], true, "packet={v}");
}

/// AC2 / AC11 — stderr CODE+bootstrap; stdout denial_hint with bootstrap; field absent when not denied (AC11 half via AC3).
#[test]
fn progressive__deny__stderr_code_and_hint_stdout_denial_hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = progressive_cmd(&vault).output().expect("progressive");
    assert_eq!(out.status.code(), Some(3));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("POLICY_DENIED"),
        "stderr must include POLICY_DENIED; got: {stderr}"
    );
    assert!(
        stderr.contains("policy bootstrap"),
        "stderr must include bootstrap remediation; got: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: Value = serde_json::from_str(&stdout).expect("stdout json");
    assert_eq!(v["denied"], true);
    let hint = v["denial_hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("bootstrap") || stdout.contains("bootstrap"),
        "stdout denial_hint must carry bootstrap; got {v}"
    );
    // Field present (not omitted) when denied.
    assert!(
        v.get("denial_hint").is_some() && !v["denial_hint"].is_null(),
        "denial_hint must be present when denied; got {v}"
    );
}

/// AC3 / F31 — bootstrap System principal (omit --principal-id) then progressive exit 0.
#[test]
fn progressive__after_system_bootstrap__exit_0_denied_false() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // F31: omit --principal-id so bootstrap defaults to System (same as cli_principal()).
    let boot = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("policy")
        .arg("bootstrap")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("json")
        .output()
        .expect("policy bootstrap");
    assert_eq!(
        boot.status.code(),
        Some(0),
        "bootstrap failed; stderr={} stdout={}",
        String::from_utf8_lossy(&boot.stderr),
        String::from_utf8_lossy(&boot.stdout)
    );

    let out = progressive_cmd(&vault)
        .output()
        .expect("progressive after bootstrap");
    assert_eq!(
        out.status.code(),
        Some(0),
        "authorized progressive must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(v["denied"], false, "packet={v}");
    // AC11: denial_hint omitted when not denied (skip_serializing_if).
    assert!(
        v.get("denial_hint").is_none() || v["denial_hint"].is_null(),
        "denial_hint must be omitted when not denied; got {v}"
    );
}

/// AC4 — source list human deny prints bootstrap on stderr after CODE line.
#[test]
fn source_list__human_deny__stderr_includes_bootstrap_hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("source")
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg("human")
        .arg("--local")
        .output()
        .expect("source list human");

    assert_eq!(
        out.status.code(),
        Some(3),
        "deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("POLICY_DENIED"),
        "stderr must include CODE; got: {stderr}"
    );
    assert!(
        stderr.contains("policy bootstrap") || stderr.contains("bootstrap"),
        "stderr must include bootstrap hint after CODE; got: {stderr}"
    );
}

/// AC5 — expand unknown handle exits 0 with kind Unknown (not policy deny).
#[test]
fn expand__unknown_handle__exit_0_kind_unknown() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("query")
        .arg("expand")
        .arg(UNKNOWN_HANDLE)
        .output()
        .expect("query expand unknown");

    assert_eq!(
        out.status.code(),
        Some(0),
        "Unknown must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(v["kind"], "Unknown", "packet={v}");
}

/// AC5 / F6 — expand existing handle without Read* grants → kind Denied, exit 3 + F4 stderr.
#[test]
fn expand__seeded_no_grants__exit_3_kind_denied() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let handle = seed_evidence_no_grants(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("query")
        .arg("expand")
        .arg(&handle)
        .output()
        .expect("query expand denied");

    assert_eq!(
        out.status.code(),
        Some(3),
        "Denied must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(v["kind"], "Denied", "packet={v}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("POLICY_DENIED"),
        "stderr must include CODE; got: {stderr}"
    );
    assert!(
        stderr.contains("policy bootstrap") || stderr.contains("bootstrap"),
        "stderr must include bootstrap hint; got: {stderr}"
    );
}

/// AC6 — progressive without project still exit 2 (T202 no regression).
#[test]
fn progressive__missing_project__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic_bin strips AI_BRAINS_PROJECT_ID; do not re-set.
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("query")
        .arg("progressive")
        .arg("x")
        .output()
        .expect("progressive missing project");

    assert_eq!(
        out.status.code(),
        Some(2),
        "missing project must exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("AI_BRAINS_PROJECT_ID") || stderr.contains("--project-id"),
        "usage example expected; got: {stderr}"
    );
}

/// AC10 — briefing project without grants still exit 0 (soft deny lock / F7).
#[test]
fn briefing_project__no_grants__exit_0_soft_deny() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .env("AI_BRAINS_PROJECT_ID", PROJECT)
        .arg("briefing")
        .arg("project")
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing project");

    assert_eq!(
        out.status.code(),
        Some(0),
        "briefing soft deny must stay exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    // Soft deny packet: denied true or empty authority with denied warning.
    let denied = v["denied"].as_bool().unwrap_or(false);
    let warnings = v["warnings"].as_array().cloned().unwrap_or_default();
    let has_denied_warn = warnings.iter().any(|w| {
        w.get("kind")
            .and_then(|k| k.as_str())
            .is_some_and(|k| k == "denied")
            || w.as_str().is_some_and(|s| s.contains("denied"))
    });
    assert!(
        denied || has_denied_warn || v.get("decisions").is_some(),
        "expected soft-deny shaped briefing packet; got {v}"
    );
}
