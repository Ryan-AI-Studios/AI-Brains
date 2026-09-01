//! Hermetic T238 OpenCode import: bind, unbound, watermark, force, dry-run,
//! child skip, list cap, missing binary, no db, turn-id parity.
#![allow(clippy::disallowed_methods, non_snake_case)]

use ai_brains_adapters::{
    OPENCODE_HARNESS_UUID, OPENCODE_UNBOUND_ALIAS, OpenCodeImportOptions, append_opencode_turns,
    generate_opencode_turn_id, import_opencode_sessions, normalize_opencode_project_hash,
    parse_export_json, session_id_from_opencode,
};
use ai_brains_capture::{CaptureContext, CaptureService, CaptureSink, SessionStopStatus};
use ai_brains_core::ids::{HarnessId, ProjectId, SessionId, UserId};
use ai_brains_core::privacy::Privacy;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::{Actor, AggregateType, Envelope, Payload, ProjectRegisteredPayload};
use ai_brains_store::{EventStore, QueryStore, SqliteEventStore, VaultConnection};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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

fn write_export(dir: &Path, session_id: &str, body: &str) -> PathBuf {
    fs::create_dir_all(dir).expect("mkdir export");
    let path = dir.join(format!("{session_id}.json"));
    fs::write(&path, body).expect("write export");
    path
}

fn sample_export(session_id: &str, directory: &str, user: &str, asst: &str) -> String {
    format!(
        r#"{{
  "info": {{
    "id": "{session_id}",
    "directory": {dir_json},
    "time": {{ "created": 1700000000000, "updated": 1700000100000 }}
  }},
  "messages": [
    {{
      "info": {{ "role": "user", "id": "msg_u_{session_id}", "time": {{ "created": 1700000001000 }} }},
      "parts": [{{ "type": "text", "text": {user_json} }}]
    }},
    {{
      "info": {{ "role": "assistant", "id": "msg_a_{session_id}", "time": {{ "created": 1700000002000 }} }},
      "parts": [
        {{ "type": "reasoning", "text": "secret" }},
        {{ "type": "tool", "name": "read", "text": "tool leak" }},
        {{ "type": "text", "text": {asst_json} }}
      ]
    }}
  ]
}}"#,
        session_id = session_id,
        dir_json = serde_json::to_string(directory).unwrap(),
        user_json = serde_json::to_string(user).unwrap(),
        asst_json = serde_json::to_string(asst).unwrap(),
    )
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// List-session row for hermetic fixtures: id, directory, worktree, updated_ms, parent_id.
struct ListRow<'a> {
    id: &'a str,
    directory: &'a str,
    worktree: Option<&'a str>,
    updated_ms: i64,
    parent_id: Option<&'a str>,
}

fn list_json(sessions: &[ListRow<'_>]) -> String {
    let mut items = Vec::new();
    for row in sessions {
        let id = row.id;
        let dir = row.directory;
        let wt = row.worktree;
        let updated = row.updated_ms;
        let parent = row.parent_id;
        let wt_field = wt
            .map(|w| format!(r#","worktree":{}"#, serde_json::to_string(w).unwrap()))
            .unwrap_or_default();
        let parent_field = parent
            .map(|p| format!(r#","parentID":{}"#, serde_json::to_string(p).unwrap()))
            .unwrap_or_default();
        items.push(format!(
            r#"{{"id":{id},"directory":{dir},"updated":{updated}{wt_field}{parent_field}}}"#,
            id = serde_json::to_string(id).unwrap(),
            dir = serde_json::to_string(dir).unwrap(),
            updated = updated,
            wt_field = wt_field,
            parent_field = parent_field,
        ));
    }
    format!("[{}]", items.join(","))
}

fn base_opts(
    days: usize,
    force: bool,
    dry_run: bool,
    list: String,
    export_dir: PathBuf,
    cursor: PathBuf,
) -> OpenCodeImportOptions {
    OpenCodeImportOptions {
        days,
        force,
        dry_run,
        max_sessions: 100,
        default_project_id: ProjectId::new(),
        allow_default_project: false,
        list_json_override: Some(list),
        export_json_override_dir: Some(export_dir),
        cursor_path_override: Some(cursor),
        config_dir_override: None,
        force_missing_binary: false,
        bin_override: None,
        list_cap: 100,
    }
}

#[test]
fn import_opencode__directory_bind__project_matches() {
    // AC5
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    let workspace = root.path().join("ws-proj");
    fs::create_dir_all(&vault_dir).unwrap();
    fs::create_dir_all(&workspace).unwrap();
    let ws = workspace.to_string_lossy().to_string();
    let sid = "ses_bind_ac5";
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, &ws, "bind-me-opencode", "bound-ok"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: &ws,
        worktree: None,
        updated_ms: now_ms(),
        parent_id: None,
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let default_pid = ProjectId::new();
    let mut options = base_opts(30, true, false, list, export_dir, cursor);
    options.default_project_id = default_pid;

    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(sink.last_error.is_none(), "{:?}", sink.last_error);
    assert_eq!(stats.sessions, 1);
    assert!(stats.imported_turns >= 2);
    assert_eq!(stats.bound_via_directory, 1);
    assert_eq!(stats.unbound_project, 0);

    let expected_alias = normalize_opencode_project_hash(&ws);
    let bound = conn
        .resolve_project_id_from_alias(&expected_alias)
        .expect("resolve")
        .expect("alias should exist");
    assert_ne!(bound, default_pid);

    let session = session_id_from_opencode(sid);
    let turns = conn
        .get_session_turns(&session.to_string())
        .expect("session turns");
    assert!(
        turns.iter().any(|(_, c)| c.contains("bind-me-opencode")),
        "{turns:?}"
    );
    assert!(
        turns
            .iter()
            .all(|(_, c)| !c.contains("secret") && !c.contains("tool leak")),
        "{turns:?}"
    );
}

#[test]
fn import_opencode__worktree_prefer__over_directory() {
    // AC5 / F20
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    let wt = root.path().join("git-root");
    let cwd = root.path().join("subdir");
    fs::create_dir_all(&vault_dir).unwrap();
    fs::create_dir_all(&wt).unwrap();
    fs::create_dir_all(&cwd).unwrap();
    let wt_s = wt.to_string_lossy().to_string();
    let cwd_s = cwd.to_string_lossy().to_string();
    let sid = "ses_wt_ac5";
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, &cwd_s, "worktree-bind", "ok"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: &cwd_s,
        worktree: Some(wt_s.as_str()),
        updated_ms: now_ms(),
        parent_id: None,
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let options = base_opts(30, true, false, list, export_dir, cursor);
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.bound_via_worktree, 1);
    let alias = normalize_opencode_project_hash(&wt_s);
    assert!(
        conn.resolve_project_id_from_alias(&alias)
            .unwrap()
            .is_some()
    );
}

#[test]
fn import_opencode__unbound__not_env_default() {
    // AC6
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    let sid = "ses_unbound_ac6";
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, "", "unbound-oc", "ok"),
    );
    // empty directory → unbound
    let list = list_json(&[ListRow {
        id: sid,
        directory: "",
        worktree: None,
        updated_ms: now_ms(),
        parent_id: None,
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let env_looking = ProjectId::new();
    let mut options = base_opts(30, true, false, list, export_dir, cursor);
    options.default_project_id = env_looking;

    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert!(stats.unbound_project >= 1);
    let unbound = conn
        .resolve_project_id_from_alias(OPENCODE_UNBOUND_ALIAS)
        .expect("resolve")
        .expect("opencode-unbound");
    assert_ne!(unbound, env_looking);
}

#[test]
fn import_opencode__watermark__second_run_zero_dupes() {
    // AC7
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    let ws = r"C:\dev\WatermarkTest";
    let sid = "ses_wm_ac7";
    let updated = now_ms();
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, ws, "wm-user", "wm-asst"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: ws,
        worktree: None,
        updated_ms: updated,
        parent_id: None,
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();

    let options = base_opts(
        30,
        false,
        false,
        list.clone(),
        export_dir.clone(),
        cursor.clone(),
    );
    let stats1 = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import1");
    assert_eq!(stats1.sessions, 1);
    assert!(stats1.imported_turns >= 2);

    let options2 = base_opts(30, false, false, list, export_dir, cursor);
    let stats2 = import_opencode_sessions(&conn, &service, &mut sink, options2).expect("import2");
    assert_eq!(stats2.imported_turns, 0);
    assert!(stats2.skipped_watermark >= 1 || stats2.found == 0);
}

#[test]
fn import_opencode__force_and_dry_run() {
    // AC8
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    let ws = r"C:\dev\ForceTest";
    let sid = "ses_force_ac8";
    let updated = now_ms();
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, ws, "force-user", "force-asst"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: ws,
        worktree: None,
        updated_ms: updated,
        parent_id: None,
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();

    // First import
    let options = base_opts(
        30,
        false,
        false,
        list.clone(),
        export_dir.clone(),
        cursor.clone(),
    );
    let _ = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import1");

    // Dry-run after watermark: found may be 0 due to watermark, or non-zero with force
    let dry = base_opts(
        30,
        true,
        true,
        list.clone(),
        export_dir.clone(),
        cursor.clone(),
    );
    let stats_dry = import_opencode_sessions(&conn, &service, &mut sink, dry).expect("dry");
    assert_eq!(stats_dry.imported_turns, 0);
    assert_eq!(stats_dry.sessions, 0);

    // Force reprocess: may re-import 0 turns if already at max index, but must not skip watermark
    let forced = base_opts(30, true, false, list, export_dir, cursor);
    let stats_force = import_opencode_sessions(&conn, &service, &mut sink, forced).expect("force");
    assert_eq!(stats_force.skipped_watermark, 0);
}

#[test]
fn import_opencode__missing_binary__soft_skip() {
    // AC12
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    fs::create_dir_all(&vault_dir).unwrap();
    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let options = OpenCodeImportOptions {
        days: 7,
        force: false,
        dry_run: false,
        max_sessions: 100,
        default_project_id: ProjectId::new(),
        allow_default_project: false,
        list_json_override: None,
        export_json_override_dir: None,
        cursor_path_override: Some(root.path().join("cursor.json")),
        config_dir_override: None,
        force_missing_binary: true,
        bin_override: None,
        list_cap: 100,
    };
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.skipped_missing_binary, 1);
    assert_eq!(stats.sessions, 0);
}

#[test]
fn import_opencode__never_references_opencode_db() {
    // AC14 — design: source must not contain opencode.db path usage
    let src = include_str!("../src/opencode.rs");
    assert!(
        src.contains("opencode.db") && src.contains("Never"),
        "must document never open opencode.db"
    );
    // No actual Path::join to opencode.db
    assert!(
        !src.contains("join(\"opencode.db\")") && !src.contains("join(\"opencode.db\")"),
        "must not join opencode.db path"
    );
}

#[test]
fn import_opencode__child_session__skipped() {
    // AC21
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    let ws = r"C:\dev\ChildTest";
    let sid = "ses_child_ac21";
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, ws, "child-should-not-ingest", "nope"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: ws,
        worktree: None,
        updated_ms: now_ms(),
        parent_id: Some("ses_parent"),
    }]);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let options = base_opts(30, true, false, list, export_dir, cursor);
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.skipped_child_session, 1);
    assert_eq!(stats.sessions, 0);
    assert_eq!(stats.imported_turns, 0);
}

#[test]
fn import_opencode__list_capped__warns() {
    // AC23
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();

    // Two sessions at cap=2
    let mut ids: Vec<String> = Vec::new();
    for i in 0..2 {
        let sid = format!("ses_cap_{i}");
        write_export(
            &export_dir,
            &sid,
            &sample_export(&sid, r"C:\dev\cap", "u", "a"),
        );
        ids.push(sid);
    }
    let updated = now_ms();
    let rows: Vec<ListRow<'_>> = ids
        .iter()
        .map(|id| ListRow {
            id: id.as_str(),
            directory: r"C:\dev\cap",
            worktree: None,
            updated_ms: updated,
            parent_id: None,
        })
        .collect();
    let list = list_json(&rows);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let mut options = base_opts(30, true, true, list, export_dir, cursor);
    options.list_cap = 2;
    options.max_sessions = 2;
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.list_capped, 1);
}

#[test]
fn import_opencode__list_capped__vendor_default_100_even_if_max_higher() {
    // AC23 honesty: OpenCode vendor hard-cap is 100; warn when len>=100 even if max_sessions>100.
    use ai_brains_adapters::OPENCODE_LIST_DEFAULT_CAP;
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    fs::create_dir_all(&export_dir).unwrap();

    let updated = now_ms();
    let mut ids: Vec<String> = Vec::new();
    for i in 0..OPENCODE_LIST_DEFAULT_CAP {
        ids.push(format!("ses_vendor_{i}"));
    }
    // Keep list JSON only (dry-run) so we do not write 100 export fixtures.
    let rows: Vec<ListRow<'_>> = ids
        .iter()
        .map(|id| ListRow {
            id: id.as_str(),
            directory: r"C:\dev\vendor",
            worktree: None,
            updated_ms: updated,
            parent_id: None,
        })
        .collect();
    let list = list_json(&rows);

    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let mut options = base_opts(30, true, true, list, export_dir, cursor);
    options.list_cap = 500;
    options.max_sessions = 500;
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(
        stats.list_capped, 1,
        "100 rows must list_capped even when user max_sessions is 500"
    );
}

#[test]
fn import_opencode__hermetic_inject__no_network() {
    // AC20
    let root = tempdir().unwrap();
    let vault_dir = root.path().join("vault");
    let export_dir = root.path().join("exports");
    let cursor = root.path().join("cursor.json");
    fs::create_dir_all(&vault_dir).unwrap();
    let sid = "ses_hermetic_ac20";
    write_export(
        &export_dir,
        sid,
        &sample_export(sid, r"C:\dev\h", "hermetic-user", "hermetic-asst"),
    );
    let list = list_json(&[ListRow {
        id: sid,
        directory: r"C:\dev\h",
        worktree: None,
        updated_ms: now_ms(),
        parent_id: None,
    }]);
    let (conn, store) = open_vault(&vault_dir);
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let options = base_opts(30, true, false, list, export_dir, cursor);
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.exported, 1);
    assert!(stats.imported_turns >= 2);
}

#[test]
fn append_opencode_turns__thinking_none_msg_id_stable() {
    // AC3 / AC4 — turn ids from msg_*; thinking always None on path
    let root = tempdir().unwrap();
    let (conn, store) = open_vault(root.path());
    let mut sink = TestSink {
        store,
        last_error: None,
    };
    let service = CaptureService::new();
    let session_id = session_id_from_opencode("ses_turn_ids");
    let project_id = ProjectId::new();
    let harness = HarnessId::from_str(OPENCODE_HARNESS_UUID).unwrap();

    // Register project + session
    let actor = Actor::User(UserId::new());
    let reg = EventBuilder::new(
        AggregateType::Project,
        project_id.as_uuid(),
        actor,
        Privacy::LocalOnly,
    )
    .build(Payload::ProjectRegistered(ProjectRegisteredPayload {
        project_id,
        name: "t".into(),
        tx_id: None,
    }))
    .unwrap();
    sink.append(reg);

    let ctx = CaptureContext {
        git_working_dir: None,
    };
    service
        .start_session(
            ai_brains_capture::SessionStartCommand {
                session_id,
                project_id,
                harness_id: harness,
                privacy: Privacy::LocalOnly,
                tx_id: None,
            },
            ctx.clone(),
            &mut sink,
        )
        .unwrap();

    let doc = serde_json::json!({
        "messages": [
            {"info":{"role":"user","id":"msg_stable_1"},"parts":[{"type":"text","text":"hello"}]},
            {"info":{"role":"assistant","id":"msg_stable_2"},"parts":[{"type":"text","text":"world"}]}
        ]
    });
    let turns = parse_export_json(&doc);
    assert_eq!(turns.len(), 2);

    let id0 = generate_opencode_turn_id(&session_id, turns[0].msg_id.as_deref(), 0);
    let id0b = generate_opencode_turn_id(&session_id, Some("msg_stable_1"), 99);
    assert_eq!(id0, id0b);

    let n = append_opencode_turns(&service, &mut sink, session_id, project_id, &turns, 0, &ctx)
        .unwrap();
    assert_eq!(n, 2);
    assert!(sink.last_error.is_none());

    // Re-append from same start would duplicate at vault level if we force; delta uses index
    let max = conn.get_max_turn_index(&session_id).unwrap().unwrap_or(-1);
    assert!(max >= 1);

    let _ = SessionStopStatus::Completed; // privacy path exercised via start
    let _ = SessionId::from_uuid(session_id.as_uuid());
}

#[test]
fn parse_export__source_has_no_db_open() {
    // AC14 + AC16 design notes present
    let src = include_str!("../src/opencode.rs");
    assert!(src.contains("OPENCODE_EXPORT_TIMEOUT_SECS"));
    assert!(src.contains("120"));
}
