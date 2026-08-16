//! Hermetic T253 Claude import: path bind, sidechain skip, message-only, dry-run.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    CLAUDE_UNBOUND_ALIAS, ClaudeImportOptions, filter_claude_jsonl_lines, import_claude_sessions,
    normalize_claude_project_hash, percent_encode_path_component,
};
use ai_brains_capture::{CaptureService, CaptureSink};
use ai_brains_core::ids::ProjectId;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::Envelope;
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::fs;
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

fn filetime_set_mtime(path: &Path, t: SystemTime) -> std::io::Result<()> {
    let f = fs::File::options().write(true).open(path)?;
    f.set_modified(t)?;
    Ok(())
}

const CLAUDE_JSONL: &str = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"hello-claude"}}
{"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"ok-claude"},{"type":"tool_use","name":"bash","input":{}}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","text":"secret-think"}]}}
{"type":"system","message":{"role":"system","content":"chrome"}}
{"type":"user","isSidechain":true,"message":{"role":"user","content":"sidechain-must-skip"}}
{"type":"attachment","message":{"content":"noise"}}
not-json
"#;

fn write_claude_session(user_home: &Path, encoded: &str, session_id: &str, body: &str) -> PathBuf {
    let dir = user_home.join(".claude").join("projects").join(encoded);
    fs::create_dir_all(&dir).expect("mkdir claude project");
    let path = dir.join(format!("{session_id}.jsonl"));
    fs::write(&path, body).expect("write jsonl");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&path, past);
    path
}

#[test]
fn import_claude__project_jsonl__user_assistant_only_sidechain_skipped() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    let ws = r"C:\dev\AI-Brains";
    let enc = percent_encode_path_component(ws);
    write_claude_session(&home, &enc, sid, CLAUDE_JSONL);

    let side_dir = home
        .join(".claude")
        .join("projects")
        .join(&enc)
        .join("subagents");
    fs::create_dir_all(&side_dir).unwrap();
    let side = side_dir.join("child.jsonl");
    fs::write(
        &side,
        r#"{"type":"user","message":{"role":"user","content":"from-subagents-dir"}}
"#,
    )
    .unwrap();
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&side, past);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_claude_sessions(
        &conn,
        &service,
        &mut sink,
        ClaudeImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: false,
        },
    )
    .expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.imported_turns >= 2);
    assert_eq!(stats.bound_via_path, 1);

    let turns = conn.get_session_turns(sid).expect("turns");
    assert!(
        turns.iter().any(|(_, c)| c.contains("hello-claude")),
        "user text kept: {turns:?}"
    );
    assert!(
        turns.iter().any(|(_, c)| c.contains("ok-claude")),
        "assistant text kept: {turns:?}"
    );
    assert!(
        turns.iter().all(|(_, c)| {
            !c.contains("secret-think")
                && !c.contains("chrome")
                && !c.contains("sidechain-must-skip")
                && !c.contains("from-subagents-dir")
                && !c.contains("bash")
        }),
        "tool/thinking/system/sidechain dropped: {turns:?}"
    );

    let expected_alias = normalize_claude_project_hash(ws);
    conn.resolve_project_id_from_alias(&expected_alias)
        .expect("resolve")
        .expect("path alias");
}

#[test]
fn import_claude__dry_run__finds_sessions_zero_vault_turns() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    write_claude_session(&home, "C--dev-Dry", sid, CLAUDE_JSONL);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_claude_sessions(
        &conn,
        &service,
        &mut sink,
        ClaudeImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: true,
        },
    )
    .expect("dry-run");
    assert!(stats.found >= 1);
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.imported_turns, 0);
    assert!(conn.get_session_turns(sid).expect("turns").is_empty());
}

#[test]
fn import_claude__unbound_folder__claude_unbound_alias() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    write_claude_session(&home, "opaque-group", sid, CLAUDE_JSONL);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_claude_sessions(
        &conn,
        &service,
        &mut sink,
        ClaudeImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: false,
        },
    )
    .expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert_eq!(stats.unbound_project, 1);
    conn.resolve_project_id_from_alias(CLAUDE_UNBOUND_ALIAS)
        .expect("resolve")
        .expect("unbound alias");
}

#[test]
fn filter_claude_jsonl_lines__thinking_none_on_kept() {
    let turns = filter_claude_jsonl_lines(CLAUDE_JSONL);
    assert_eq!(turns.len(), 2);
    assert!(turns.iter().all(|t| t.turn.source_ts.is_none()));
}
