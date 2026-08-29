//! T311 AC8–AC10 / CLI AC5 — `decision in-force` clap, deny, JSON ruling key.
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
use rstest::rstest;
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
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = VaultConnection::open(vault_path, &key).expect("open vault");
    StorePorts::from_store(SqliteEventStore::new(conn))
}

fn seed_read_decisions(vault_path: &Path) -> String {
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
        GrantCapability::ReadDecisions,
        Privacy::LocalOnly,
    )
    .expect("grant ReadDecisions");
    scope_identity_key(&scope)
}

#[test]
fn decision_in_force__help__lists_term_scope_format() {
    let out = common::hermetic_bin()
        .arg("decision")
        .arg("in-force")
        .arg("--help")
        .output()
        .expect("help");
    assert!(
        out.status.success(),
        "help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("<TERM>"),
        "help must list <TERM>; got {stdout}"
    );
    assert!(
        stdout.contains("--scope"),
        "help must list --scope; got {stdout}"
    );
    assert!(
        stdout.contains("--format"),
        "help must list --format; got {stdout}"
    );
    for token in ["auto", "pretty", "human", "text", "json", "markdown", "md"] {
        assert!(
            stdout.contains(token),
            "help must list format {token}; got {stdout}"
        );
    }
}

#[test]
fn decision_in_force__format_nope__clap_exit_2() {
    let out = common::hermetic_bin()
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--format")
        .arg("nope")
        .output()
        .expect("format nope");
    assert_eq!(
        out.status.code(),
        Some(2),
        "InvalidValue must be clap exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.contains("invalid value") || combined.contains("possible values"),
        "expected clap InvalidValue text; got {combined}"
    );
}

#[test]
fn decision_in_force__empty_term__exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    for term in ["", "   "] {
        let out = common::hermetic_bin()
            .arg("--no-project-context")
            .arg("--vault-path")
            .arg(&vault)
            .arg("decision")
            .arg("in-force")
            .arg(term)
            .arg("--scope")
            .arg(&scope)
            .arg("--format")
            .arg("json")
            .output()
            .expect("empty term");
        assert_eq!(
            out.status.code(),
            Some(2),
            "empty/whitespace term must be fail_usage exit 2; term={term:?} stderr={} stdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

#[test]
fn decision_in_force__policy_denied__exit_3_omits_required_scope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("deny");

    assert_eq!(
        out.status.code(),
        Some(3),
        "deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("POLICY_DENIED") || stderr.contains("POLICY_DENIED"),
        "POLICY_DENIED on stdout or stderr; stdout={stdout} stderr={stderr}"
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json deny");
    assert_eq!(v["code"], "POLICY_DENIED");
    let hint = v
        .pointer("/details/hint")
        .and_then(|h| h.as_str())
        .unwrap_or("");
    assert!(
        hint.contains("policy bootstrap") && hint.contains("omit --scope"),
        "hint must name bootstrap and omit required --scope; got {hint:?}"
    );
    assert!(
        !hint.contains("--scope …"),
        "hint must not require --scope ellipsis; got {hint:?}"
    );
}

#[test]
fn decision_in_force__unknown_term__ruling_key_null() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_decisions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("unknown term");

    assert_eq!(
        out.status.code(),
        Some(0),
        "authorized unknown must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(
        v.get("ruling").is_some(),
        "AC10 ruling key must exist; got {v}"
    );
    assert!(v["ruling"].is_null(), "unknown term ruling must be null");
    assert_eq!(v["term"], "workspace_id");
    assert!(v.get("chain").and_then(|c| c.as_array()).is_some());
    assert!(
        v.get("next_step").is_none(),
        "F12 JSON has no next_step key"
    );
    assert!(
        v.get("as_of").is_none(),
        "AC10 omit --as-of → no as_of key; got {v}"
    );
}

#[test]
fn decision_in_force_help__after_help__names_as_of() {
    let out = common::hermetic_bin()
        .arg("decision")
        .arg("in-force")
        .arg("--help")
        .output()
        .expect("help");
    assert!(
        out.status.success(),
        "help must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--as-of"),
        "help must list --as-of; got {stdout}"
    );
    assert!(
        stdout.contains("T") && (stdout.contains('Z') || stdout.contains('+')),
        "after_help must show RFC3339 example with T and Z/offset; got {stdout}"
    );
}

#[test]
fn decision_in_force_clap__default__as_of_absent() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_decisions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("omit as-of");
    assert_eq!(
        out.status.code(),
        Some(0),
        "in-force without --as-of must parse; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

#[rstest]
#[case::invalid_after_term(&[
    "decision",
    "in-force",
    "workspace_id",
    "--as-of",
    "not-a-date",
])]
#[case::date_only_after_term(&[
    "decision",
    "in-force",
    "workspace_id",
    "--as-of",
    "2026-01-01",
])]
#[case::flag_before_term(&[
    "decision",
    "in-force",
    "--as-of",
    "not-a-date",
    "workspace_id",
])]
#[case::empty_value(&[
    "decision",
    "in-force",
    "workspace_id",
    "--as-of",
    "",
])]
fn decision_in_force__as_of_invalid__clap_exit_2(#[case] args: &[&str]) {
    let out = common::hermetic_bin()
        .args(args)
        .output()
        .expect("spawn as-of invalid");
    assert_eq!(
        out.status.code(),
        Some(2),
        "must be clap exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.to_ascii_lowercase().contains("invalid"),
        "expected invalid value text; got {combined}"
    );
}

#[test]
fn decision_in_force__as_of_human__emits_as_of_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_decisions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("human")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--as-of")
        .arg("2026-01-15T00:00:00Z")
        .output()
        .expect("human as-of");

    assert_eq!(
        out.status.code(),
        Some(0),
        "human as-of must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("As of: 2026-01-15T00:00:00Z"),
        "F12 human must print As of: line; got {stdout}"
    );
}

#[test]
fn decision_in_force__as_of_unknown__emits_as_of_key() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_decisions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("decision")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .arg("--as-of")
        .arg("2026-01-15T00:00:00Z")
        .output()
        .expect("as-of unknown");

    assert_eq!(
        out.status.code(),
        Some(0),
        "authorized unknown+as-of must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(v["ruling"].is_null());
    assert!(
        v.get("as_of").is_some() && !v["as_of"].is_null(),
        "AC10 as_of key present; got {v}"
    );
}
