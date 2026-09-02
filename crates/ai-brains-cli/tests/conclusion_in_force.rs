//! T323 AC8–AC10 / CLI AC5 — `conclusion in-force` clap, deny, JSON ruling key.
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

fn seed_read_conclusions(vault_path: &Path) -> String {
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
        GrantCapability::ReadConclusions,
        Privacy::LocalOnly,
    )
    .expect("grant ReadConclusions");
    scope_identity_key(&scope)
}

#[test]
fn conclusion_in_force__help__lists_term_scope_format() {
    let out = common::hermetic_bin()
        .arg("conclusion")
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
        stdout.contains("--term"),
        "help must list --term flag; got {stdout}"
    );
    assert!(stdout.contains("TERM"), "help must name TERM; got {stdout}");
    assert!(
        stdout.contains("--term="),
        "after_help must document --term= empty; got {stdout}"
    );
    assert!(
        !stdout.contains("'\"\"'") && !stdout.contains("--%"),
        "after_help must not recommend '\"\"' or --%; got {stdout}"
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
    assert!(
        stdout.contains("conclusion in-force"),
        "after_help must name conclusion in-force; got {stdout}"
    );
}

#[test]
fn conclusion_in_force__format_nope__clap_exit_2() {
    let out = common::hermetic_bin()
        .arg("conclusion")
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

#[rstest]
#[case::empty("")]
#[case::whitespace("   ")]
fn conclusion_in_force__empty_term__exit_2(#[case] term: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
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
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.contains("term must be non-empty"),
        "empty/whitespace must keep fail_usage message; term={term:?} got {combined}"
    );
}

fn assert_fail_usage_non_empty_term(out: &std::process::Output, label: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stderr}{stdout}");
    assert_eq!(
        out.status.code(),
        Some(2),
        "{label}: expected exit 2; stderr={stderr} stdout={stdout}"
    );
    assert!(
        combined.contains("term must be non-empty"),
        "{label}: expected fail_usage message; got {combined}"
    );
    assert!(
        !combined.contains("required arguments were not provided"),
        "{label}: must not be clap missing <TERM>; got {combined}"
    );
}

#[test]
fn conclusion_in_force__omitted_term__fail_usage_exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
        .arg("in-force")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .output()
        .expect("omitted term");
    assert_fail_usage_non_empty_term(&out, "omitted term");
}

#[rstest]
#[case::bare_flag(vec!["--term"])]
#[case::flag_then_empty(vec!["--term", ""])]
fn conclusion_in_force__term_flag_no_value__fail_usage_exit_2(#[case] term_args: Vec<&str>) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
        .arg("in-force");
    for a in &term_args {
        cmd.arg(a);
    }
    let out = cmd
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .output()
        .expect("term flag no value");
    assert_fail_usage_non_empty_term(&out, &format!("term_args={term_args:?}"));
}

#[test]
fn conclusion_in_force__term_flag_equals_empty__fail_usage_exit_2() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
        .arg("in-force")
        .arg("--term=")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("json")
        .output()
        .expect("term=");
    assert_fail_usage_non_empty_term(&out, "--term=");
}

#[test]
fn conclusion_in_force__term_flag_workspace_id__format_nope__clap_exit_2() {
    let out = common::hermetic_bin()
        .arg("conclusion")
        .arg("in-force")
        .arg("--term")
        .arg("workspace_id")
        .arg("--format")
        .arg("nope")
        .output()
        .expect("term flag format nope");
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
fn conclusion_in_force__positional_and_term_flag__clap_conflict_exit_2() {
    let out = common::hermetic_bin()
        .arg("conclusion")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--term")
        .arg("other")
        .output()
        .expect("positional+term conflict");
    assert_eq!(
        out.status.code(),
        Some(2),
        "conflict must be clap exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        combined.contains("cannot be used with") || combined.contains("conflict"),
        "expected clap conflict text; got {combined}"
    );
}

#[test]
fn conclusion_in_force__policy_denied__exit_3_omits_required_scope() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
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
fn conclusion_in_force__format_human__long_hint_stay_green() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = format!("Repository:{PROJECT}");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("human")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("conclusion human deny");

    assert_eq!(
        out.status.code(),
        Some(3),
        "deny must exit 3; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("POLICY_DENIED:"),
        "human deny prefix; stderr={stderr}"
    );
    assert!(
        stderr.contains("omit --scope"),
        "conclusion human deny stays LONG HINT; stderr={stderr}"
    );
}

#[test]
fn conclusion_in_force__unknown_term__ruling_key_null() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_conclusions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
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
    assert!(
        v.get("scope").and_then(|s| s.as_str()).is_some(),
        "AC10 scope key must exist; got {v}"
    );
    assert!(v.get("chain").and_then(|c| c.as_array()).is_some());
    assert!(
        v.get("next_step").is_none(),
        "F12 JSON has no next_step key"
    );
    assert!(
        v.get("as_of").is_none(),
        "AC10 no as_of key on conclusion in-force; got {v}"
    );
}

#[test]
fn conclusion_in_force__human__unknown_term_message() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let scope = seed_read_conclusions(&vault);

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("conclusion")
        .arg("in-force")
        .arg("workspace_id")
        .arg("--scope")
        .arg(&scope)
        .arg("--format")
        .arg("human")
        .arg("--principal-id")
        .arg(PRINCIPAL)
        .output()
        .expect("human unknown");

    assert_eq!(
        out.status.code(),
        Some(0),
        "authorized unknown human must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No in-force ruling for term \"workspace_id\""),
        "F12 human empty; got {stdout}"
    );
    assert!(
        stdout.contains("ai-brains recall") && stdout.contains("what did we decide"),
        "F12 next must mention recall needle; got {stdout}"
    );
}
