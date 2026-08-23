//! T227 — Briefing format honesty + granted substance hermetic suite.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_control_plane::{
    ProposeConclusionRequest, ProposeDecisionRequest, StorePorts, SystemClock, activate_conclusion,
    approve_decision, issue_grant, make_principal, propose_conclusion, propose_decision,
    register_principal,
};
use ai_brains_core::ids::{EvidenceId, PrincipalId, ProjectId};
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
const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
/// Default System principal used by `cli_principal()` (F33).
const SYSTEM_PRINCIPAL: &str = "a1b2a1b2-a1b2-a1b2-a1b2-a1b2a1b2a1b2";

const DECISION_STATEMENT: &str = "T227 hermetic decision: use deterministic briefings";
const CONCLUSION_STATEMENT: &str = "T227 hermetic conclusion: authority is policy-first";

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

fn briefing_project(vault: &Path) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT);
    cmd
}

fn briefing_personal(vault: &Path) -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("briefing")
        .arg("personal");
    cmd
}

/// Seed System read grants + Human write lifecycle + Approved decision + Active conclusion.
fn seed_granted_substance(vault_path: &Path) {
    let ports = open_ports(vault_path);
    let clock = SystemClock;
    let project = ProjectId::from_uuid(Uuid::parse_str(PROJECT).unwrap());
    let scope = ScopeRef::Repository(project);

    let system = make_principal(
        PrincipalKind::System,
        PrincipalId::from_uuid(Uuid::parse_str(SYSTEM_PRINCIPAL).unwrap()),
        "cli-system",
    );
    let human = make_principal(PrincipalKind::Human, PrincipalId::new(), "t227-human");
    register_principal(&ports.writer, &clock, &system).expect("register system");
    register_principal(&ports.writer, &clock, &human).expect("register human");

    // CLI briefing reads as System — discovery grants only on System principal (F33).
    for cap in [
        GrantCapability::ReadDecisions,
        GrantCapability::ReadConclusions,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            system.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .expect("system read grant");
    }

    // Human proposes + approves (approve requires Human principal).
    for cap in [
        GrantCapability::ProposeDecision,
        GrantCapability::ApproveDecision,
        GrantCapability::ProposeConclusion,
        GrantCapability::ReadDecisions,
        GrantCapability::ReadConclusions,
    ] {
        issue_grant(
            &ports.writer,
            &clock,
            human.id,
            scope.clone(),
            cap,
            Privacy::LocalOnly,
        )
        .expect("human write grant");
    }

    let policy = ports.production_policy();
    let dec = propose_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeDecisionRequest {
            principal: human.clone(),
            scope: scope.clone(),
            title: "T227 decision".into(),
            statement: DECISION_STATEMENT.into(),
            conclusion_ids: None,
            evidence_ids: Some(vec![EvidenceId::new()]),
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            decision_id: None,
        },
    )
    .expect("propose decision");
    approve_decision(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human,
        dec.decision_id,
        Privacy::LocalOnly,
    )
    .expect("approve decision");

    let conc = propose_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        ProposeConclusionRequest {
            principal: human.clone(),
            scope: scope.clone(),
            statement: CONCLUSION_STATEMENT.into(),
            evidence_ids: vec![EvidenceId::new()],
            privacy: Privacy::LocalOnly,
            valid_from: None,
            valid_until: None,
            protected_category: None,
            conclusion_id: None,
        },
    )
    .expect("propose conclusion");
    activate_conclusion(
        &ports.writer,
        &ports.query,
        &clock,
        &policy,
        &human,
        conc.conclusion_id,
        Privacy::LocalOnly,
    )
    .expect("activate conclusion");

    // Sanity: scope key matches CLI project id.
    assert_eq!(ai_brains_control_plane::scope_identity_key(&scope), SCOPE);
}

// ---------------------------------------------------------------------------
// AC1–AC3 — human/pretty/text/markdown/md → markdown; json → JSON
// ---------------------------------------------------------------------------

#[test]
fn briefing_project__format_human_aliases__markdown_header() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    for fmt in ["human", "pretty", "text", "markdown", "md"] {
        let out = briefing_project(&vault)
            .arg("--format")
            .arg(fmt)
            .output()
            .expect("briefing project");
        assert_eq!(
            out.status.code(),
            Some(0),
            "format {fmt} must exit 0; stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("# Project Briefing"),
            "format {fmt} must emit markdown header; got: {stdout}"
        );
        assert!(
            !stdout.trim_start().starts_with('{'),
            "format {fmt} must not emit JSON; got: {stdout}"
        );
    }
}

#[test]
fn briefing_personal__format_human__markdown_header() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_personal(&vault)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing personal");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("# Personal Continuity Briefing"),
        "personal human → markdown; got: {stdout}"
    );
}

#[test]
fn briefing_project__format_json__parses_packet() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_project(&vault)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing json");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim_start().starts_with('{'),
        "json must start with object: {stdout}"
    );
    let v: Value = serde_json::from_str(&stdout).expect("parse json packet");
    assert_eq!(v["kind"], "Project");
}

// ---------------------------------------------------------------------------
// AC4 — unknown format exit 2, empty stdout
// ---------------------------------------------------------------------------

#[test]
fn briefing_project__format_banana__exit_2_no_stdout_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_project(&vault)
        .arg("--format")
        .arg("banana")
        .output()
        .expect("briefing banana");
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown format must exit 2; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown --format") || stderr.contains("accepted"),
        "stderr must list accepted formats: {stderr}"
    );
    assert!(
        stderr.contains("human") && stderr.contains("json"),
        "stderr must include accepted tokens: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // F3: zero stdout on usage fail (no JSON pollution, no markdown).
    assert!(
        stdout.is_empty(),
        "stdout must be empty on usage fail: {stdout:?}"
    );
}

// ---------------------------------------------------------------------------
// AC10 — soft deny exit 0 without grants
// ---------------------------------------------------------------------------

#[test]
fn briefing_project__no_grants__soft_deny_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_project(&vault)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing soft deny");
    assert_eq!(
        out.status.code(),
        Some(0),
        "soft deny must stay exit 0 (T221); stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(v["denied"], true, "packet={v}");
    // T241 AC7: denied JSON includes denial_hint with bootstrap; exit still 0.
    let hint = v["denial_hint"].as_str().unwrap_or("");
    assert!(
        !hint.is_empty() && hint.contains("policy bootstrap"),
        "denied JSON must include denial_hint with policy bootstrap; got {v}"
    );
    // AC7 soft: denied packets must not carry empty_authority warnings.
    if let Some(warnings) = v["warnings"].as_array() {
        for w in warnings {
            let kind = w["kind"].as_str().unwrap_or("");
            assert_ne!(
                kind, "empty_authority",
                "denied packet must not emit empty_authority; got {v}"
            );
            assert_ne!(
                kind, "empty_continuity",
                "denied project packet must not emit empty_continuity; got {v}"
            );
        }
    }
}

/// T289 AC2 — denied Personal human omits `_None_` (not empty preferences).
#[test]
fn briefing_personal__no_grants__human_omits_none() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_personal(&vault)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing personal human deny");
    assert_eq!(
        out.status.code(),
        Some(0),
        "soft deny must stay exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("# Personal Continuity Briefing"),
        "personal human header: {stdout}"
    );
    assert!(
        stdout.contains("**Denied:**"),
        "personal deny blockquote: {stdout}"
    );
    assert!(
        stdout.contains("recall"),
        "personal deny must name recall: {stdout}"
    );
    assert!(
        !stdout.contains("_None_"),
        "denied personal must not print _None_: {stdout}"
    );
    assert!(
        !stdout.contains("policy bootstrap"),
        "personal deny must not recommend policy bootstrap: {stdout}"
    );
}

/// T241 AC7 / CX1 P2 — personal soft deny JSON also includes denial_hint (CP path).
#[test]
fn briefing_personal__no_grants__soft_deny_denial_hint() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let out = briefing_personal(&vault)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing personal soft deny");
    assert_eq!(
        out.status.code(),
        Some(0),
        "soft deny must stay exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).expect("json");
    // Packet may be top-level or nested under "packet".
    let denied = v["denied"]
        .as_bool()
        .or_else(|| v["packet"]["denied"].as_bool())
        .unwrap_or(false);
    assert!(denied, "personal without grants should be denied; got {v}");
    let hint = v["denial_hint"]
        .as_str()
        .or_else(|| v["packet"]["denial_hint"].as_str())
        .unwrap_or("");
    assert!(
        !hint.is_empty() && hint.contains("recall"),
        "personal denied JSON denial_hint must name recall; got {v}"
    );
    assert!(
        !hint.contains("policy bootstrap"),
        "personal denied JSON must not recommend policy bootstrap; got {v}"
    );
}

// ---------------------------------------------------------------------------
// AC11 — help lists aliases + human example
// ---------------------------------------------------------------------------

#[test]
fn briefing_project__help__lists_human_pretty_and_example() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("briefing")
        .arg("project")
        .arg("--help")
        .output()
        .expect("help");
    assert_eq!(out.status.code(), Some(0));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("human") && combined.contains("pretty"),
        "help must list human/pretty aliases: {combined}"
    );
    assert!(
        combined.contains("markdown") && combined.contains("json"),
        "help must list markdown/json: {combined}"
    );
    assert!(
        combined.contains("--format human") || combined.contains("format human"),
        "help after_help must include human example: {combined}"
    );
    assert!(
        combined.contains("not Approved") && combined.contains("vault_pin_count"),
        "T288 AC10: after_help must name vault-pin stanza + JSON extras; got {combined}"
    );
}

// ---------------------------------------------------------------------------
// AC6 — granted + decision + conclusion in md + JSON
// ---------------------------------------------------------------------------

#[test]
fn briefing_project__granted_substance__decision_and_conclusion_in_md_and_json() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_granted_substance(&vault);

    let md_out = briefing_project(&vault)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing human granted");
    assert_eq!(
        md_out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&md_out.stderr),
        String::from_utf8_lossy(&md_out.stdout)
    );
    let md = String::from_utf8_lossy(&md_out.stdout);
    assert!(md.contains("# Project Briefing"), "markdown header: {md}");
    assert!(
        md.contains(DECISION_STATEMENT),
        "decision statement in md: {md}"
    );
    assert!(
        md.contains(CONCLUSION_STATEMENT),
        "conclusion statement in md: {md}"
    );
    assert!(
        !md.contains("> **Denied:**"),
        "granted packet must not be denied: {md}"
    );
    assert!(
        !md.contains("## Vault pins"),
        "T288 AC7: overlay off when authority is non-empty; got {md}"
    );

    let json_out = briefing_project(&vault)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing json granted");
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let v: Value = serde_json::from_slice(&json_out.stdout).expect("json packet");
    assert_eq!(v["denied"], false, "packet={v}");
    let decisions = v["decisions"].as_array().expect("decisions array");
    let conclusions = v["conclusions"].as_array().expect("conclusions array");
    assert!(
        !decisions.is_empty(),
        "decisions must be non-empty; got {v}"
    );
    assert!(
        !conclusions.is_empty(),
        "conclusions must be non-empty; got {v}"
    );
    assert!(
        v.get("vault_pin_count").is_none() && v.get("vault_pin_previews").is_none(),
        "T288 AC7: overlay JSON keys omit when authority is non-empty; got {v}"
    );
    let dec_text = decisions
        .iter()
        .filter_map(|d| d["statement"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let conc_text = conclusions
        .iter()
        .filter_map(|c| c["statement"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        dec_text.contains(DECISION_STATEMENT),
        "decision in json: {dec_text}"
    );
    assert!(
        conc_text.contains(CONCLUSION_STATEMENT),
        "conclusion in json: {conc_text}"
    );
}
