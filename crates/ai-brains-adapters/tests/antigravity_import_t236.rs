//! Hermetic T236 import binding / force / unbound tests.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    AntigravityImportOptions, import_antigravity_sessions, load_agy_history_index,
    normalize_agy_project_hash,
};
use ai_brains_capture::{CaptureService, CaptureSink};
use ai_brains_core::ids::ProjectId;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::Envelope;
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

struct TestSink {
    store: SqliteEventStore,
    last_error: Option<String>,
}

impl CaptureSink for TestSink {
    fn append(&mut self, envelope: Envelope) {
        if let Err(e) = self.store.append_event(&envelope) {
            self.last_error = Some(e.to_string());
        }
    }

    fn set_sync_state(&mut self, key: &str, value: &str) {
        if let Err(e) = self.store.set_sync_state(key, value) {
            self.last_error = Some(e.to_string());
        }
    }
}

fn open_vault(dir: &Path) -> (VaultConnection, SqliteEventStore) {
    let db = dir.join("vault.db");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(db, &sql_key).expect("open vault");
    conn.migrate().expect("migrate");
    let store = SqliteEventStore::new(conn.clone());
    (conn, store)
}

fn write_brain_transcript(home: &Path, conversation_id: &str, body: &str) -> PathBuf {
    let logs = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("brain")
        .join(conversation_id)
        .join(".system_generated")
        .join("logs");
    fs::create_dir_all(&logs).expect("mkdir logs");
    let path = logs.join("transcript.jsonl");
    fs::write(&path, body).expect("write transcript");
    // Age the file so quiescence (300s) does not skip unless testing force
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&path, past);
    path
}

fn filetime_set_mtime(path: &Path, t: SystemTime) -> std::io::Result<()> {
    // Use std filetime via open + set_modified when available
    let f = fs::File::options().write(true).open(path)?;
    f.set_modified(t)?;
    Ok(())
}

#[test]
fn import_antigravity__history_bind__project_matches_workspace() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let workspace = root.path().join("ws-proj");
    fs::create_dir_all(&workspace).unwrap();
    let ws_str = workspace.to_string_lossy().to_string();
    let cid = "11111111-1111-1111-1111-111111111111";

    // history maps cid → workspace
    let hist_dir = home.join(".gemini").join("antigravity-cli");
    fs::create_dir_all(&hist_dir).unwrap();
    let hist = hist_dir.join("history.jsonl");
    let mut hf = fs::File::create(&hist).unwrap();
    writeln!(
        hf,
        r#"{{"display":"t","timestamp":1000,"workspace":{},"conversationId":"{}"}}"#,
        serde_json::to_string(&ws_str).unwrap(),
        cid
    )
    .unwrap();

    write_brain_transcript(
        &home,
        cid,
        r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nbind-me\n</USER_REQUEST>","tool_calls":[]}
{"step_index":4,"source":"MODEL","type":"PLANNER_RESPONSE","content":"bound-ok","tool_calls":[]}
"#,
    );

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let default_pid = ProjectId::new();
    let options = AntigravityImportOptions {
        days: 30,
        default_project_id: default_pid,
        allow_default_project: false,
        force: true,
        home_override: Some(home.clone()),
    };

    let stats = import_antigravity_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.imported_turns >= 2);
    assert_eq!(stats.bound_via_history, 1);
    assert_eq!(stats.unbound_project, 0);

    let expected_alias = normalize_agy_project_hash(&ws_str);
    let bound = conn
        .resolve_project_id_from_alias(&expected_alias)
        .expect("resolve")
        .expect("alias should exist");
    assert_ne!(bound, default_pid, "must not use default/env project");

    // Map check pure
    let map = load_agy_history_index(&hist);
    assert_eq!(
        map.get(cid).map(String::as_str),
        Some(expected_alias.as_str())
    );
}

#[test]
fn import_antigravity__no_history__unbound_not_cwd_env() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let cid = "22222222-2222-2222-2222-222222222222";
    write_brain_transcript(
        &home,
        cid,
        r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nunbound\n</USER_REQUEST>","tool_calls":[]}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","content":"ok","tool_calls":[]}
"#,
    );

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let env_looking_default = ProjectId::new();
    let options = AntigravityImportOptions {
        days: 30,
        default_project_id: env_looking_default,
        allow_default_project: false,
        force: true,
        home_override: Some(home),
    };

    let stats = import_antigravity_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.unbound_project >= 1);

    let unbound = conn
        .resolve_project_id_from_alias("agy-unbound")
        .expect("resolve")
        .expect("agy-unbound alias");
    assert_ne!(unbound, env_looking_default);
}

#[test]
fn import_antigravity__force__skips_quiescence() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let cid = "33333333-3333-3333-3333-333333333333";
    let path = write_brain_transcript(
        &home,
        cid,
        r#"{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","content":"<USER_REQUEST>\nforce\n</USER_REQUEST>","tool_calls":[]}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","content":"yes","tool_calls":[]}
"#,
    );
    // Make mtime "now" so quiescence would skip without force
    filetime_set_mtime(&path, SystemTime::now()).unwrap();

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();

    // Without force → quiescent skip
    let stats_skip = import_antigravity_sessions(
        &conn,
        &service,
        &mut sink,
        AntigravityImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: false,
            home_override: Some(home.clone()),
        },
    )
    .expect("import skip");
    assert_eq!(stats_skip.sessions, 0);
    assert!(stats_skip.skipped_quiescent >= 1);

    // With force → imports (reuse same sink/store)
    let stats_force = import_antigravity_sessions(
        &conn,
        &service,
        &mut sink,
        AntigravityImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
        },
    )
    .expect("import force");
    assert_eq!(stats_force.sessions, 1);
    assert!(stats_force.imported_turns >= 2);
}
