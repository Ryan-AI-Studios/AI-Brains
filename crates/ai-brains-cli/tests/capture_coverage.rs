//! T337 AC11 / Codex — command-level capture coverage exit contract.
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

use ai_brains_adapters::GROK_HARNESS_UUID;
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::SqlCipherKey;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{
    Actor, AggregateType, Payload, ProjectRegisteredPayload, SessionStartedPayload,
};
use ai_brains_store::{EventStore, SqliteEventStore};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tempfile::tempdir;

const CURSOR_SID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaa01";

fn init_vault(vault: &std::path::Path) {
    common::hermetic_bin()
        .arg("--vault-path")
        .arg(vault)
        .arg("--no-project-context")
        .arg("init")
        .assert()
        .success();
}

/// Production coverage consults `CURSOR_HOME` / `GROK_HOME` / … before `HOME`.
/// `dirs::home_dir()` on Windows uses the Known Folder profile, not `USERPROFILE`,
/// so CLI fixtures must set the harness `*_HOME` vars (not only HOME/USERPROFILE).
fn strip_harness_homes(cmd: &mut assert_cmd::Command) {
    cmd.env_remove("CURSOR_HOME");
    cmd.env_remove("GROK_HOME");
    cmd.env_remove("CLAUDE_HOME");
    cmd.env_remove("CODEX_HOME");
}

#[test]
fn capture_coverage__no_scope__exit_2() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("v.db");
    init_vault(&vault);
    common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn capture_coverage__cursor_deficit__exit_0() {
    let home = tempdir().expect("home");
    let vault_dir = tempdir().expect("vault");
    let vault = vault_dir.path().join("v.db");
    init_vault(&vault);
    let jsonl = home
        .path()
        .join(".cursor")
        .join("projects")
        .join("c-dev-x")
        .join("agent-transcripts")
        .join(CURSOR_SID)
        .join(format!("{CURSOR_SID}.jsonl"));
    fs::create_dir_all(jsonl.parent().expect("parent")).expect("mkdir");
    fs::write(&jsonl, "{}\n").expect("write jsonl");

    let mut cmd = common::hermetic_vault(&vault);
    strip_harness_homes(&mut cmd);
    let output = cmd
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .arg("--global")
        .arg("--format")
        .arg("json")
        .env("USERPROFILE", home.path())
        .env("HOME", home.path())
        .env("CURSOR_HOME", home.path().join(".cursor"))
        .output()
        .expect("capture coverage");
    assert!(
        output.status.success(),
        "AC11 deficit exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    let sources = parsed["sources"].as_array().expect("sources");
    assert_eq!(sources.len(), 6);
    let cursor = sources
        .iter()
        .find(|s| s["source"] == "cursor")
        .expect("cursor row");
    assert_eq!(
        cursor["status"].as_str(),
        Some("deficit"),
        "cursor row={cursor}"
    );
    assert_eq!(cursor["mode"], "import_only");
    let next = cursor["next_step"].as_str().unwrap_or("");
    assert!(next.contains("cursor-import"), "next_step={next}");
}

#[test]
fn capture_coverage__grok_home_env__honored_without_user_home_override() {
    let grok_home = tempdir().expect("grok home");
    let empty_user_home = tempdir().expect("empty user home");
    let vault_dir = tempdir().expect("vault");
    let vault = vault_dir.path().join("v.db");
    init_vault(&vault);
    let history = grok_home
        .path()
        .join("sessions")
        .join("C%3A")
        .join("sid")
        .join("chat_history.jsonl");
    fs::create_dir_all(history.parent().expect("parent")).expect("mkdir");
    fs::write(&history, "{}\n").expect("write grok history");

    let mut cmd = common::hermetic_vault(&vault);
    strip_harness_homes(&mut cmd);
    let output = cmd
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .arg("--global")
        .arg("--format")
        .arg("json")
        .env("USERPROFILE", empty_user_home.path())
        .env("HOME", empty_user_home.path())
        .env("GROK_HOME", grok_home.path())
        .output()
        .expect("capture coverage");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    let grok = parsed["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .find(|s| s["source"] == "grok")
        .expect("grok row");
    assert!(
        grok["disk_eligible"].as_u64().unwrap_or(0) >= 1,
        "GROK_HOME must be consulted when home_override is None; grok={grok}"
    );
}

fn start_harness_session(vault: &Path, harness_uuid: &str) {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::from_raw(common::ZERO_SQLCIPHER_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(
        vault.to_str().expect("utf8 vault"),
        &key,
    )
    .expect("open vault");
    let store = SqliteEventStore::new(conn);
    let project_id = ProjectId::from_str(common::DEFAULT_PROJECT).expect("project id");
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Project,
                project_id.as_uuid(),
                Actor::System,
                Privacy::LocalOnly,
            )
            .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
                project_id,
                name: "t360".into(),
                tx_id: None,
            }))
            .expect("project envelope"),
        )
        .expect("append ProjectRegistered");
    let session_id = SessionId::new();
    let harness = HarnessId::from_str(harness_uuid).expect("harness");
    store
        .append_event(
            &EventBuilder::new(
                AggregateType::Session,
                session_id.as_uuid(),
                Actor::Harness(harness),
                Privacy::LocalOnly,
            )
            .build(Payload::SessionStarted(SessionStartedPayload {
                session_id,
                project_id,
                tx_id: None,
            }))
            .expect("session envelope"),
        )
        .expect("append SessionStarted");
}

#[test]
fn capture_coverage__grok_partial_vault__unverifiable_exit_0() {
    let home = tempdir().expect("home");
    let vault_dir = tempdir().expect("vault");
    let vault = vault_dir.path().join("v.db");
    init_vault(&vault);
    start_harness_session(&vault, GROK_HARNESS_UUID);
    let history = home
        .path()
        .join(".grok")
        .join("sessions")
        .join("C%3A")
        .join("sid")
        .join("chat_history.jsonl");
    fs::create_dir_all(history.parent().expect("parent")).expect("mkdir");
    fs::write(&history, "{}\n").expect("write grok history");
    let history2 = home
        .path()
        .join(".grok")
        .join("sessions")
        .join("C%3A")
        .join("sid2")
        .join("chat_history.jsonl");
    fs::create_dir_all(history2.parent().expect("parent")).expect("mkdir");
    fs::write(&history2, "{}\n").expect("write grok history 2");

    let mut cmd = common::hermetic_vault(&vault);
    strip_harness_homes(&mut cmd);
    let output = cmd
        .arg("--no-project-context")
        .arg("capture")
        .arg("coverage")
        .arg("--global")
        .arg("--format")
        .arg("json")
        .env("USERPROFILE", home.path())
        .env("HOME", home.path())
        .env("GROK_HOME", home.path().join(".grok"))
        .output()
        .expect("capture coverage");
    assert!(
        output.status.success(),
        "AC9 exit 0; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    let grok = parsed["sources"]
        .as_array()
        .expect("sources")
        .iter()
        .find(|s| s["source"] == "grok")
        .expect("grok row");
    assert_eq!(
        grok["status"].as_str(),
        Some("unverifiable_subagent"),
        "grok={grok}"
    );
    let next = grok["next_step"].as_str().unwrap_or("");
    assert!(next.contains("--dry-run"), "next_step={next}");
    assert!(!next.contains("--force"));
    let warnings = parsed["warnings"].as_array().cloned().unwrap_or_default();
    assert!(
        !warnings
            .iter()
            .any(|w| w.as_str() == Some("grok_batch_empty_all_subagent")),
        "warnings={warnings:?}"
    );
}
