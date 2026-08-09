//! Hermetic T237 Grok import: summary bind, unbound, force/quiescence, subagent skip,
//! re-import delta, turn-id / thinking-None parity with hook path.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    GrokImportOptions, generate_grok_turn_id, import_grok_sessions, normalize_grok_project_hash,
    parse_chat_history_file, percent_encode_path_component, resolve_chat_history_path,
};
use ai_brains_capture::{CaptureService, CaptureSink};
use ai_brains_core::ids::{ProjectId, SessionId};
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

/// Layout: `{user_home}/.grok/sessions/<encoded-cwd>/<sessionId>/chat_history.jsonl`
/// Matches `resolve_grok_home(home_override)` → `home_override/.grok`.
fn write_grok_session(
    user_home: &Path,
    workspace: &str,
    session_id: &str,
    history_body: &str,
    summary_json: Option<&str>,
) -> PathBuf {
    let enc = percent_encode_path_component(workspace);
    let sess_dir = user_home
        .join(".grok")
        .join("sessions")
        .join(&enc)
        .join(session_id);
    fs::create_dir_all(&sess_dir).expect("mkdir session");
    let history = sess_dir.join("chat_history.jsonl");
    fs::write(&history, history_body).expect("write history");
    if let Some(sum) = summary_json {
        fs::write(sess_dir.join("summary.json"), sum).expect("write summary");
    }
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&history, past);
    history
}

const HISTORY_BIND: &str = r#"{"type":"user","content":"<user_query>\nbind-me-grok\n</user_query>"}
{"type":"assistant","content":"bound-ok"}
{"type":"reasoning","content":"secret-think"}
{"type":"tool_result","content":"tool-noise"}
"#;

const HISTORY_UNBOUND: &str = r#"{"type":"user","content":"<user_query>\nunbound-grok\n</user_query>"}
{"type":"assistant","content":"ok"}
"#;

const HISTORY_FORCE: &str = r#"{"type":"user","content":"<user_query>\nforce-me\n</user_query>"}
{"type":"assistant","content":"yes"}
"#;

#[test]
fn import_grok__summary_bind__project_matches_git_root() {
    // AC5: summary git_root_dir → project alias (not env/default hijack)
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    let workspace = root.path().join("ws-proj");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    fs::create_dir_all(&workspace).unwrap();
    let ws_str = workspace.to_string_lossy().to_string();
    let sid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    let summary = format!(
        r#"{{"info":{{"id":"{sid}","cwd":"C:\\other"}},"git_root_dir":{}}}"#,
        serde_json::to_string(&ws_str).unwrap()
    );
    write_grok_session(&home, &ws_str, sid, HISTORY_BIND, Some(&summary));

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let default_pid = ProjectId::new();
    let options = GrokImportOptions {
        days: 30,
        default_project_id: default_pid,
        allow_default_project: false,
        force: true,
        home_override: Some(home),
    };

    let stats = import_grok_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.imported_turns >= 2);
    assert_eq!(stats.bound_via_summary, 1);
    assert_eq!(stats.unbound_project, 0);

    let expected_alias = normalize_grok_project_hash(&ws_str);
    let bound = conn
        .resolve_project_id_from_alias(&expected_alias)
        .expect("resolve")
        .expect("alias should exist");
    assert_ne!(bound, default_pid, "must not use default/env project");

    let turns = conn.get_session_turns(sid).expect("session turns");
    assert!(
        turns.iter().any(|(_, c)| c.contains("bind-me-grok")),
        "bound project must hold imported Grok content: {turns:?}"
    );
    assert!(
        turns.iter().all(|(_, c)| !c.contains("secret-think") && !c.contains("tool-noise")),
        "must not ingest reasoning/tool: {turns:?}"
    );

    let turn_project: String = {
        let raw = conn.lock().expect("lock vault");
        raw.query_row(
            "SELECT project_id FROM turn_projection WHERE session_id = ?1 LIMIT 1",
            [sid],
            |row| row.get(0),
        )
        .expect("turn project_id")
    };
    assert_eq!(
        turn_project,
        bound.to_string(),
        "AC5: turn project_id must match summary-bound project"
    );
}

#[test]
fn import_grok__no_summary__unbound_not_cwd_env() {
    // AC6: unbound → grok-unbound; env/default project not hijacked
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    // Group name that does not decode to a path → no path bind
    let sess_dir = home
        .join(".grok")
        .join("sessions")
        .join("opaque-group")
        .join(sid);
    fs::create_dir_all(&sess_dir).unwrap();
    let history = sess_dir.join("chat_history.jsonl");
    fs::write(&history, HISTORY_UNBOUND).unwrap();
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&history, past);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let env_looking_default = ProjectId::new();
    let options = GrokImportOptions {
        days: 30,
        default_project_id: env_looking_default,
        allow_default_project: false,
        force: true,
        home_override: Some(home),
    };

    let stats = import_grok_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.unbound_project >= 1);

    let unbound = conn
        .resolve_project_id_from_alias("grok-unbound")
        .expect("resolve")
        .expect("grok-unbound alias");
    assert_ne!(unbound, env_looking_default);
}

#[test]
fn import_grok__force__skips_quiescence() {
    // AC8
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    let ws = r"C:\dev\QuiesceTest";
    let path = write_grok_session(&home, ws, sid, HISTORY_FORCE, None);
    filetime_set_mtime(&path, SystemTime::now()).unwrap();

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();

    let stats_skip = import_grok_sessions(
        &conn,
        &service,
        &mut sink,
        GrokImportOptions {
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

    let stats_force = import_grok_sessions(
        &conn,
        &service,
        &mut sink,
        GrokImportOptions {
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

#[test]
fn import_grok__reimport_unchanged__zero_new_turns() {
    // AC7
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
    let ws = r"C:\dev\DeltaTest";
    write_grok_session(&home, ws, sid, HISTORY_FORCE, None);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let opts = || GrokImportOptions {
        days: 30,
        default_project_id: ProjectId::new(),
        allow_default_project: false,
        force: true,
        home_override: Some(home.clone()),
    };

    let first = import_grok_sessions(&conn, &service, &mut sink, opts()).expect("first");
    assert_eq!(first.sessions, 1);
    assert!(first.imported_turns >= 2);
    let turns_after_first = first.imported_turns;

    let second = import_grok_sessions(&conn, &service, &mut sink, opts()).expect("second");
    assert_eq!(second.imported_turns, 0, "re-import must not duplicate turns");
    assert!(
        second.skipped_unchanged >= 1 || second.sessions == 0,
        "expect unchanged skip or zero sessions: {second:?}"
    );

    let vault_turns = conn.get_session_turns(sid).expect("turns");
    assert_eq!(
        vault_turns.len(),
        turns_after_first,
        "vault turn count must stay stable after re-import"
    );
}

#[test]
fn import_grok__subagent_session__skipped_counter() {
    // AC18: skipped_subagent increments; no import
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
    // Path segment `subagent-` triggers skip
    let sess_dir = home
        .join(".grok")
        .join("sessions")
        .join("subagent-worktree-group")
        .join(sid);
    fs::create_dir_all(&sess_dir).unwrap();
    let history = sess_dir.join("chat_history.jsonl");
    fs::write(&history, HISTORY_FORCE).unwrap();
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&history, past);
    // Also seed a normal session so found can be non-zero for comparison
    let normal_sid = "ffffffff-ffff-ffff-ffff-ffffffffffff";
    write_grok_session(&home, r"C:\dev\Normal", normal_sid, HISTORY_FORCE, None);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_grok_sessions(
        &conn,
        &service,
        &mut sink,
        GrokImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
        },
    )
    .expect("import");
    assert!(
        stats.skipped_subagent >= 1,
        "subagent path must increment skipped_subagent: {stats:?}"
    );
    // Subagent content must not appear
    let sub_turns = conn.get_session_turns(sid).expect("sub turns");
    assert!(
        sub_turns.is_empty(),
        "subagent session must not be imported: {sub_turns:?}"
    );
    // Normal session still imported
    let normal_turns = conn.get_session_turns(normal_sid).expect("normal turns");
    assert!(
        !normal_turns.is_empty(),
        "normal session should import: {normal_turns:?}"
    );
}

#[test]
fn import_grok__never_ingests_updates_jsonl() {
    // AC14: discovery only chat_history; updates.jsonl present must not become content
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "12121212-1212-1212-1212-121212121212";
    let ws = r"C:\dev\UpdatesNoise";
    let history = write_grok_session(&home, ws, sid, HISTORY_FORCE, None);
    let updates = history.parent().unwrap().join("updates.jsonl");
    fs::write(
        &updates,
        r#"{"type":"user","content":"<user_query>\nFROM-UPDATES-MUST-NOT-INGEST\n</user_query>"}
"#,
    )
    .unwrap();

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let _stats = import_grok_sessions(
        &conn,
        &service,
        &mut sink,
        GrokImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
        },
    )
    .expect("import");

    let turns = conn.get_session_turns(sid).expect("turns");
    assert!(
        turns
            .iter()
            .all(|(_, c)| !c.contains("FROM-UPDATES-MUST-NOT-INGEST")),
        "updates.jsonl must never be content SOOT: {turns:?}"
    );
    assert!(
        turns.iter().any(|(_, c)| c.contains("force-me")),
        "chat_history content should still import: {turns:?}"
    );
}

#[test]
fn hook_path__parse_and_turn_ids__thinking_none_parity() {
    // AC3 / AC4: shared filter + turn-id SOOT used by grok-hook and batch
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let ws = r"C:\dev\AI-Brains";
    let sid = "99999999-9999-9999-9999-999999999999";
    let history = write_grok_session(
        &home,
        ws,
        sid,
        r#"{"type":"user","content":"<user_query>\nhook-parity\n</user_query>"}
{"type":"assistant","content":"reply-ok"}
{"type":"reasoning","content":"hidden"}
"#,
        None,
    );

    let grok_home = home.join(".grok");
    let resolved = resolve_chat_history_path(&grok_home, sid, Some(ws), None)
        .expect("resolve via percent encode");
    assert_eq!(resolved, history);

    let turns = parse_chat_history_file(&resolved).expect("parse");
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].content, "hook-parity");
    assert_eq!(turns[1].content, "reply-ok");
    // IngestableTurn has no thinking field — SOOT is thinking:None at ingest request

    let session = SessionId::from_uuid(uuid::Uuid::parse_str(sid).unwrap());
    let id0 = generate_grok_turn_id(&session, 0);
    let id1 = generate_grok_turn_id(&session, 1);
    assert_ne!(id0, id1);
    // Stable across calls (live==batch)
    assert_eq!(id0, generate_grok_turn_id(&session, 0));
    assert_eq!(id1, generate_grok_turn_id(&session, 1));
}
