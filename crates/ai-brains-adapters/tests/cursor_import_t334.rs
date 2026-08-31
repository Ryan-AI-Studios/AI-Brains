//! Hermetic T334 Cursor import: slug bind (mixed-case), subagents skip, dry-run.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    CURSOR_UNBOUND_ALIAS, CursorImportOptions, discover_cursor_sessions, filter_cursor_jsonl_lines,
    import_cursor_sessions, is_cursor_sidechain_path,
};
use ai_brains_capture::{CaptureService, CaptureSink};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::{
    Actor, AggregateType, Envelope, Payload, ProjectRegisteredPayload,
    RepositoryPathAliasAddedPayload, constructors::EventBuilder,
};
use ai_brains_path::normalize_project_path;
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

const CURSOR_JSONL: &str = r#"{"role":"user","message":{"content":[{"type":"text","text":"<manually_attached_skills>\nskills dump\n</manually_attached_skills>\n<timestamp>Monday, Aug 31, 2026, 5:52 AM (UTC-4)</timestamp>\n<user_query>\nhello-cursor\n</user_query>"}]}}
{"role":"assistant","message":{"content":[{"type":"text","text":"ok-cursor"},{"type":"tool_use","name":"Shell","input":{}}]}}
{"type":"turn_ended","status":"success"}
not-json
"#;

fn write_cursor_session(user_home: &Path, folder: &str, session_id: &str, body: &str) -> PathBuf {
    let dir = user_home
        .join(".cursor")
        .join("projects")
        .join(folder)
        .join("agent-transcripts")
        .join(session_id);
    fs::create_dir_all(&dir).expect("mkdir cursor session");
    let path = dir.join(format!("{session_id}.jsonl"));
    fs::write(&path, body).expect("write jsonl");
    let past = SystemTime::now() - Duration::from_secs(600);
    let _ = filetime_set_mtime(&path, past);
    path
}

fn register_path_alias(store: &SqliteEventStore, project_id: ProjectId, mixed_path: &str) {
    let normalized = normalize_project_path(mixed_path)
        .expect("normalize")
        .canonical()
        .to_string();
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "AI-Brains".to_string(),
        tx_id: None,
    }))
    .expect("register");
    store.append_event(&reg).expect("append register");
    let alias = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::RepositoryPathAliasAdded(
        RepositoryPathAliasAddedPayload {
            project_id,
            normalized_path: normalized,
        },
    ))
    .expect("alias");
    store.append_event(&alias).expect("append alias");
}

#[test]
fn filter_cursor_jsonl_lines__user_query_kept_tools_and_turn_ended_dropped() {
    let turns = filter_cursor_jsonl_lines(CURSOR_JSONL);
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].turn.content, "hello-cursor");
    assert!(!turns[0].turn.content.contains("skills dump"));
    assert_eq!(turns[1].turn.content, "ok-cursor");
    assert!(turns.iter().all(|t| !t.turn.content.contains("Shell")));
}

#[test]
fn discover_cursor__only_nested_matching_stem_and_flat_root() {
    let root = tempdir().unwrap();
    let cursor_home = root.path().join(".cursor");
    let slug = "c-dev-AI-Brains";
    let transcripts = cursor_home
        .join("projects")
        .join(slug)
        .join("agent-transcripts");
    let nested_id = "11111111-1111-1111-1111-111111111111";
    let flat_id = "22222222-2222-2222-2222-222222222222";
    fs::create_dir_all(transcripts.join(nested_id)).unwrap();
    fs::write(
        transcripts
            .join(nested_id)
            .join(format!("{nested_id}.jsonl")),
        "{}\n",
    )
    .unwrap();
    fs::write(transcripts.join(nested_id).join("extra.jsonl"), "{}\n").unwrap();
    fs::write(transcripts.join(format!("{nested_id}.jsonl")), "{}\n").unwrap();
    fs::write(transcripts.join(format!("{flat_id}.jsonl")), "{}\n").unwrap();
    let non_uuid_nested = transcripts.join("foo");
    fs::create_dir_all(&non_uuid_nested).unwrap();
    fs::write(non_uuid_nested.join("foo.jsonl"), "{}\n").unwrap();
    fs::write(transcripts.join("notes.jsonl"), "{}\n").unwrap();
    let deep = transcripts.join("foo").join("bar");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("baz.jsonl"), "{}\n").unwrap();

    let sources = discover_cursor_sessions(&cursor_home).expect("discover");
    let ingest: Vec<_> = sources
        .iter()
        .filter(|s| !is_cursor_sidechain_path(&s.path))
        .collect();
    assert_eq!(ingest.len(), 2, "sources: {sources:?}");
    assert!(
        ingest.iter().any(|s| {
            s.session_id == nested_id
                && s.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n == format!("{nested_id}.jsonl"))
                && s.path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    == Some(nested_id)
        }),
        "nested primary kept: {ingest:?}"
    );
    assert!(
        ingest.iter().any(|s| s.session_id == flat_id),
        "flat-only kept: {ingest:?}"
    );
    assert!(
        ingest.iter().all(|s| {
            s.path.file_name().and_then(|n| n.to_str()).is_none_or(|n| {
                n != "extra.jsonl" && n != "baz.jsonl" && n != "foo.jsonl" && n != "notes.jsonl"
            })
        }),
        "non-uuid and extra nested jsonl must not be sources: {ingest:?}"
    );
}

#[test]
fn import_cursor__hermetic_path_alias_slug__bound_turns() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
    write_cursor_session(&home, "c-dev-AI-Brains", sid, CURSOR_JSONL);

    let (conn, store) = open_vault(&vault_dir);
    let project_id = ProjectId::new();
    register_path_alias(&store, project_id, r"C:\dev\AI-Brains");
    let aliases = conn.list_path_aliases().expect("aliases");
    assert!(
        aliases.iter().any(|(_, p)| p == r"C:\dev\ai-brains"),
        "stored alias must be rest-lower: {aliases:?}"
    );

    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_cursor_sessions(
        &conn,
        &service,
        &mut sink,
        CursorImportOptions {
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
    assert!(stats.sessions >= 1, "sessions: {stats:?}");
    assert!(stats.imported_turns >= 2, "turns: {stats:?}");
    assert!(stats.bound_via_path >= 1, "bound: {stats:?}");

    let turns = conn.get_session_turns(sid).expect("turns");
    assert!(
        turns.iter().any(|(_, c)| c.contains("hello-cursor")),
        "user text kept: {turns:?}"
    );
    assert!(
        turns.iter().any(|(_, c)| c.contains("ok-cursor")),
        "assistant text kept: {turns:?}"
    );
    assert!(
        turns.iter().all(|(_, c)| {
            !c.contains("skills dump")
                && !c.contains("manually_attached_skills")
                && !c.contains("Shell")
                && !c.contains("tool_use")
        }),
        "skill/tool dropped: {turns:?}"
    );
}

#[test]
fn import_cursor__subagents_dir__skipped_sidechain() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
    write_cursor_session(&home, "c-dev-AI-Brains", sid, CURSOR_JSONL);

    let side_dir = home
        .join(".cursor")
        .join("projects")
        .join("c-dev-AI-Brains")
        .join("agent-transcripts")
        .join("subagents");
    fs::create_dir_all(&side_dir).unwrap();
    let side = side_dir.join("child.jsonl");
    fs::write(
        &side,
        r#"{"role":"user","message":{"content":[{"type":"text","text":"<user_query>\nfrom-subagents-dir\n</user_query>"}]}}
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
    let stats = import_cursor_sessions(
        &conn,
        &service,
        &mut sink,
        CursorImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: false,
        },
    )
    .expect("import");
    assert!(stats.skipped_sidechain >= 1, "sidechain: {stats:?}");
    let turns = conn.get_session_turns(sid).expect("parent");
    assert!(
        turns.iter().all(|(_, c)| !c.contains("from-subagents-dir")),
        "subagent text must not land: {turns:?}"
    );
}

#[test]
fn import_cursor__dry_run__zero_writes() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "ffffffff-ffff-ffff-ffff-ffffffffffff";
    write_cursor_session(&home, "c-dev-AI-Brains", sid, CURSOR_JSONL);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_cursor_sessions(
        &conn,
        &service,
        &mut sink,
        CursorImportOptions {
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
fn import_cursor__unbound_folder__cursor_unbound_alias() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    let sid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    write_cursor_session(&home, "empty-window", sid, CURSOR_JSONL);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let stats = import_cursor_sessions(
        &conn,
        &service,
        &mut sink,
        CursorImportOptions {
            days: 30,
            default_project_id: ProjectId::new(),
            allow_default_project: false,
            force: true,
            home_override: Some(home),
            dry_run: false,
        },
    )
    .expect("import");
    assert!(stats.unbound_project >= 1, "unbound: {stats:?}");
    conn.resolve_project_id_from_alias(CURSOR_UNBOUND_ALIAS)
        .expect("resolve")
        .expect("unbound alias");
}
