//! Hermetic T253 Codex import: rollout keep-list, fail-open malformed, dry-run.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    CodexImportOptions, filter_codex_rollout_lines, import_codex_sessions,
    normalize_codex_project_hash, peek_codex_session_meta,
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

const SID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

fn rollout_body() -> String {
    format!(
        r#"{{"timestamp":"2026-08-15T00:00:00Z","type":"session_meta","payload":{{"id":"{SID}","cwd":"C:\\dev\\AI-Brains"}}}}
{{"timestamp":"2026-08-15T00:00:01Z","type":"event_msg","payload":{{"type":"agent_message","content":"event-noise"}}}}
{{"timestamp":"2026-08-15T00:00:02Z","type":"response_item","payload":{{"type":"message","role":"user","content":[{{"type":"input_text","text":"hello-codex"}}]}}}}
{{"timestamp":"2026-08-15T00:00:03Z","type":"response_item","payload":{{"type":"message","role":"assistant","content":[{{"type":"output_text","text":"ok-codex"}}]}}}}
{{"timestamp":"2026-08-15T00:00:04Z","type":"response_item","payload":{{"type":"function_call","name":"bash"}}}}
{{"timestamp":"2026-08-15T00:00:05Z","type":"response_item","payload":{{"type":"message","role":"system","content":"chrome"}}}}
{{"timestamp":"2026-08-15T00:00:06Z","type":"unknown","payload":{{}}}}
not-json
"#
    )
}

fn write_codex_rollout(user_home: &Path, body: &str) -> PathBuf {
    let dir = user_home
        .join(".codex")
        .join("sessions")
        .join("2026")
        .join("08")
        .join("15");
    fs::create_dir_all(&dir).expect("mkdir codex sessions");
    let path = dir.join(format!("rollout-2026-08-15T12-00-00-{SID}.jsonl"));
    fs::write(&path, body).expect("write rollout");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&path, past);
    path
}

#[test]
fn import_codex__rollout__only_response_item_message_roles() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    write_codex_rollout(&home, &rollout_body());

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_codex_sessions(
        &conn,
        &service,
        &mut sink,
        CodexImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: false,
            scope_paths: None,
        },
    )
    .expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.imported_turns >= 2);
    assert!(stats.skipped_malformed >= 1);
    assert_eq!(stats.bound_via_path, 1);

    let turns = conn.get_session_turns(SID).expect("turns");
    assert!(
        turns.iter().any(|(_, c)| c.contains("hello-codex")),
        "user text kept: {turns:?}"
    );
    assert!(
        turns.iter().any(|(_, c)| c.contains("ok-codex")),
        "assistant text kept: {turns:?}"
    );
    assert!(
        turns.iter().all(|(_, c)| {
            !c.contains("event-noise") && !c.contains("chrome") && !c.contains("bash")
        }),
        "event_msg/system/tool dropped: {turns:?}"
    );

    let expected_alias = normalize_codex_project_hash(r"C:\dev\AI-Brains");
    conn.resolve_project_id_from_alias(&expected_alias)
        .expect("resolve")
        .expect("path alias");
}

#[test]
fn import_codex__dry_run__finds_sessions_zero_vault_turns() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();
    write_codex_rollout(&home, &rollout_body());

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_codex_sessions(
        &conn,
        &service,
        &mut sink,
        CodexImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: true,
            scope_paths: None,
        },
    )
    .expect("dry-run");
    assert!(stats.found >= 1);
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.imported_turns, 0);
    assert!(conn.get_session_turns(SID).expect("turns").is_empty());
}

#[test]
fn filter_codex_rollout_lines__keep_user_assistant_only() {
    let turns = filter_codex_rollout_lines(&rollout_body());
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].turn.content, "hello-codex");
    assert_eq!(turns[1].turn.content, "ok-codex");
}

const SID_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

fn rollout_meta(sid: &str, cwd: &str) -> String {
    let cwd_json = cwd.replace('\\', r"\\");
    format!(
        r#"{{"timestamp":"2026-08-15T00:00:00Z","type":"session_meta","payload":{{"id":"{sid}","cwd":"{cwd_json}"}}}}
{{"timestamp":"2026-08-15T00:00:02Z","type":"response_item","payload":{{"type":"message","role":"user","content":[{{"type":"input_text","text":"hello-codex"}}]}}}}
{{"timestamp":"2026-08-15T00:00:03Z","type":"response_item","payload":{{"type":"message","role":"assistant","content":[{{"type":"output_text","text":"C:\\dev\\other"}}]}}}}
"#
    )
}

fn write_named_rollout(user_home: &Path, sid: &str, cwd: &str) -> PathBuf {
    let dir = user_home
        .join(".codex")
        .join("sessions")
        .join("2026")
        .join("08")
        .join("15");
    fs::create_dir_all(&dir).expect("mkdir named");
    let path = dir.join(format!("rollout-2026-08-15T12-00-00-{sid}.jsonl"));
    fs::write(&path, rollout_meta(sid, cwd)).expect("write named");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&path, past);
    path
}

fn dry_run_import(home: PathBuf, scope_paths: Option<Vec<String>>) -> usize {
    let vault_dir = home.parent().expect("parent").join("vault-scope");
    let _ = fs::create_dir_all(&vault_dir);
    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_codex_sessions(
        &conn,
        &service,
        &mut sink,
        CodexImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: true,
            scope_paths,
        },
    )
    .expect("dry-run scoped");
    stats.found
}

#[test]
fn import_codex__scope_paths_cwd__found_only_matching() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    fs::create_dir_all(&home).unwrap();
    write_named_rollout(&home, SID, r"C:\dev\AI-Brains");
    write_named_rollout(&home, SID_B, r"C:\dev\other");
    let found = dry_run_import(home, Some(vec![r"C:\dev\ai-brains".to_string()]));
    assert_eq!(found, 1);
}

#[test]
fn import_codex__scope_paths_none__finds_sibling_rollouts() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    fs::create_dir_all(&home).unwrap();
    write_named_rollout(&home, SID, r"C:\dev\AI-Brains");
    write_named_rollout(&home, SID_B, r"C:\dev\other");
    let found = dry_run_import(home, None);
    assert_eq!(found, 2);
}

#[test]
fn import_codex__pid_without_aliases__scopes_cwd_not_global() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    fs::create_dir_all(&home).unwrap();
    write_named_rollout(&home, SID, r"C:\dev\ai-brains");
    write_named_rollout(&home, SID_B, r"C:\dev\other");
    let found = dry_run_import(home, Some(vec![r"C:\dev\ai-brains".to_string()]));
    assert_eq!(found, 1);
}

#[test]
fn peek_codex_session_meta__stops_before_turn_lines() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rollout.jsonl");
    fs::write(
        &path,
        concat!(
            r#"{"type":"session_meta","payload":{"id":"s1","cwd":"C:\\dev\\AI-Brains"}}"#,
            "\n",
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"C:\\dev\\other"}]}}"#,
            "\n",
        ),
    )
    .expect("write peek fixture");
    let (sid, cwd) = peek_codex_session_meta(&path);
    assert_eq!(sid.as_deref(), Some("s1"));
    assert_eq!(cwd.as_deref(), Some(r"C:\dev\AI-Brains"));
}
