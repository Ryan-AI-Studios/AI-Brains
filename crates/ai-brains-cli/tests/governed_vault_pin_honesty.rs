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
use rstest::rstest;
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

const TRACE_MISSING_NEXT_STEP: &str =
    "No persisted trace. Run: ai-brains query progressive \"what did we decide\" --dry-run false";

/// T291 AC2 — missing trace is a missing-only envelope (not the token `null`) + exit 0.
#[test]
fn query_trace__unknown__stdout_envelope_exit_0() {
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
    assert_ne!(
        trimmed, "null",
        "T291: missing trace must not be the token null; got {trimmed:?}"
    );
    let v: Value = serde_json::from_str(&trimmed).unwrap_or_else(|_| {
        panic!("missing trace stdout must be JSON object; got {trimmed:?}")
    });
    assert_eq!(v["found"], false, "envelope found=false; got {v}");
    assert_eq!(v["api_version"], "1", "envelope api_version; got {v}");
    let trace_id = v["trace_id"].as_str().unwrap_or("");
    assert!(
        trace_id.contains("00000000-0000-0000-0000-000000000000") || trace_id.contains("00000000"),
        "trace_id must carry sanitized requested id; got {v}"
    );
    assert_eq!(
        v["next_step"].as_str().unwrap_or(""),
        TRACE_MISSING_NEXT_STEP,
        "next_step must equal F8 const; got {v}"
    );
}

fn list_cmd(vault: &Path, noun: &str, format: &str) -> std::process::Output {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg(noun)
        .arg("list")
        .arg("--scope")
        .arg(SCOPE)
        .arg("--format")
        .arg(format)
        .arg("--local")
        .output()
        .unwrap_or_else(|_| panic!("{noun} list --format {format}"))
}

fn list_json(vault: &Path, noun: &str) -> std::process::Output {
    list_cmd(vault, noun, "json")
}

fn list_human(vault: &Path, noun: &str) -> std::process::Output {
    list_cmd(vault, noun, "human")
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

fn none_line_for(noun: &str) -> &'static str {
    match noun {
        "evidence" => "evidence: (none)",
        "source" => "sources: (none)",
        "review" => "review items: (none)",
        other => panic!("unknown list noun {other}"),
    }
}

/// T290 AC2 — 0-pin granted-empty JSON next_step is copy-paste recall + Pinned: 0.
#[rstest]
#[case("evidence")]
#[case("source")]
#[case("review")]
fn list__authorized_empty__next_step_names_pinned_and_query(#[case] noun: &str) {
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
        step.contains("recall")
            && step.contains("what did we decide")
            && step.contains("(Pinned: 0)"),
        "{noun} next_step must name recall + query + Pinned: 0; got {v}"
    );
    assert!(
        v.get("vault_pin_count").is_none(),
        "{noun} must not grow T288 keys; got {v}"
    );
}

/// T290 AC3 — human empty prints (none) then the same next-step line (all three nouns).
#[rstest]
#[case("evidence")]
#[case("source")]
#[case("review")]
fn list__authorized_empty_human__none_then_next_line(#[case] noun: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let out = list_human(&vault, noun);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{noun} human empty must exit 0; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(none_line_for(noun)),
        "{noun} must keep (none); got {stdout}"
    );
    assert!(
        stdout.contains("recall")
            && stdout.contains("what did we decide")
            && stdout.contains("Pinned: 0"),
        "{noun} human must print next line; got {stdout}"
    );
}

/// T290 AC5 — pin raises Pinned to nonzero; items stay []; pin text not in items.
#[test]
fn evidence_list__authorized_empty_with_pin__next_step_nonzero_items_empty() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let needle = "T290-ac5-needle-list-pin-count";
    pin_via_hermetic_cmd(&vault, &format!("DECISION: {needle}"), Some("t290"));
    let out = list_json(&vault, "evidence");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    let items = v["items"].as_array().expect("items");
    assert!(items.is_empty(), "items must stay []; got {v}");
    let items_json = serde_json::to_string(&v["items"]).expect("items json");
    assert!(
        !items_json.contains(needle),
        "pin text must not appear in items; got {v}"
    );
    let step = v["next_step"].as_str().unwrap_or("");
    assert!(
        step.contains("(Pinned:") && !step.contains("(Pinned: 0)"),
        "next_step must contain (Pinned: N) with N>0; got {v}"
    );
}

fn briefing_project_human(vault: &Path) -> std::process::Output {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("human")
        .output()
        .expect("briefing project human")
}

fn briefing_project_json(vault: &Path) -> std::process::Output {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("briefing")
        .arg("project")
        .arg("--project-id")
        .arg(PROJECT)
        .arg("--format")
        .arg("json")
        .output()
        .expect("briefing project json")
}

fn pin_via_hermetic_cmd(vault: &Path, content: &str, tag: Option<&str>) {
    let mut cmd = common::hermetic_cmd(vault);
    cmd.arg("pin");
    if let Some(t) = tag {
        cmd.arg("--tag").arg(t);
    }
    cmd.arg("--").arg(content).assert().success();
}

fn human_pinned_count_nonzero(md: &str) -> bool {
    md.lines().any(|line| {
        line.strip_prefix("Pinned:")
            .and_then(|rest| rest.trim().parse::<u64>().ok())
            .is_some_and(|n| n > 0)
    })
}

/// T288 AC1 — granted + DECISION pin: vault-pin stanza, not under Decisions.
#[test]
fn briefing_project__granted_with_decision_pin__human_stanza_not_under_decisions() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let needle = "T288-ac1-needle-granted-empty-stanza";
    pin_via_hermetic_cmd(&vault, &format!("DECISION: {needle}"), Some("t288"));

    let human = briefing_project_human(&vault);
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        md.contains("## Vault pins (not Approved)"),
        "granted-empty with pin must emit heading; got {md}"
    );
    assert!(
        human_pinned_count_nonzero(&md),
        "Pinned: must be a nonzero inventory count; got {md}"
    );
    assert!(
        md.contains(needle) || md.contains("DECISION:"),
        "stanza must surface needle or DECISION:; got {md}"
    );
    assert!(md.contains("recall"), "must keep recall next; got {md}");
    assert!(
        md.contains("## Decisions (current authority)"),
        "must keep Decisions heading; got {md}"
    );
    assert!(
        md.contains("_None_"),
        "Decisions body must stay _None_; got {md}"
    );
    assert!(
        !md.contains("[Approved]"),
        "pin must not appear as Approved claim under Decisions; got {md}"
    );
}

/// T288 AC2 — same fixture: JSON overlay keys; authority arrays empty.
#[test]
fn briefing_project__granted_with_decision_pin__json_overlay_count_and_previews() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    let needle = "T288-ac2-needle-json-overlay";
    pin_via_hermetic_cmd(&vault, &format!("DECISION: {needle}"), Some("t288"));

    let json = briefing_project_json(&vault);
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
    let count = v["vault_pin_count"].as_u64().unwrap_or(0);
    assert!(count >= 1, "vault_pin_count must be ≥1; got {v}");
    let previews = v["vault_pin_previews"].as_array().expect("previews");
    assert!(
        previews.iter().any(|p| p
            .as_str()
            .is_some_and(|s| s.contains("DECISION:") || s.contains(needle))),
        "previews must contain DECISION: or needle; got {v}"
    );
}

/// T288 AC4 — granted-empty 0 pins: heading + Pinned: 0; no fabricated DECISION.
#[test]
fn briefing_project__granted_empty_zero_pins__pinned_zero_no_fabricated_decision() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);

    let human = briefing_project_human(&vault);
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        md.contains("## Vault pins (not Approved)"),
        "zero-pin granted-empty must still emit heading; got {md}"
    );
    assert!(
        md.contains("Pinned: 0"),
        "zero-pin must print Pinned: 0; got {md}"
    );
    assert!(
        !md.lines()
            .any(|l| l.trim_start().starts_with("- DECISION:")),
        "must not fabricate a DECISION preview; got {md}"
    );

    let json = briefing_project_json(&vault);
    assert_eq!(
        json.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&json.stderr),
        String::from_utf8_lossy(&json.stdout)
    );
    let v = stdout_json(&json);
    assert_eq!(v["vault_pin_count"], 0, "packet={v}");
    let previews = v["vault_pin_previews"].as_array().expect("previews");
    assert!(previews.is_empty(), "zero-pin previews must be []; got {v}");
}

/// T288 AC5 — denied: no vault-pin heading / JSON keys; grant-wall stands.
#[test]
fn briefing_project__denied__no_vault_pin_stanza() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let human = briefing_project_human(&vault);
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        !md.contains("## Vault pins"),
        "denied markdown must omit vault-pin heading; got {md}"
    );
    assert!(
        !md.contains("_None_"),
        "T275 denied must not show _None_; got {md}"
    );

    let json = briefing_project_json(&vault);
    assert_eq!(json.status.code(), Some(0));
    let v = stdout_json(&json);
    assert_eq!(v["denied"], true, "packet={v}");
    assert!(
        v.get("vault_pin_count").is_none(),
        "denied JSON must omit vault_pin_count; got {v}"
    );
    assert!(
        v.get("vault_pin_previews").is_none(),
        "denied JSON must omit vault_pin_previews; got {v}"
    );
}

/// T288 AC15 — chrome-only pin: COUNT ≥1, no DECISION preview.
#[test]
fn briefing_project__granted_chrome_only__count_without_decision_preview() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    pin_via_hermetic_cmd(&vault, "## Objective", None);

    let human = briefing_project_human(&vault);
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        human_pinned_count_nonzero(&md),
        "chrome-only must still COUNT ≥1; got {md}"
    );
    assert!(
        !md.lines().any(|l| l.contains("DECISION:")),
        "chrome-only must not fabricate DECISION preview; got {md}"
    );
}

/// T288 AC16 — Hotspot pin counted, omitted from previews.
#[test]
fn briefing_project__granted_hotspot_only__preview_omits_hotspot() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_discovery_grants(&vault);
    pin_via_hermetic_cmd(&vault, "HOTSPOT: crates/foo.rs", None);

    let human = briefing_project_human(&vault);
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&human.stderr),
        String::from_utf8_lossy(&human.stdout)
    );
    let md = String::from_utf8_lossy(&human.stdout);
    assert!(
        human_pinned_count_nonzero(&md),
        "hotspot-only must COUNT ≥1; got {md}"
    );
    assert!(
        !md.contains("HOTSPOT: crates/foo.rs"),
        "previews must omit Hotspot; got {md}"
    );
}
