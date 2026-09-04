//! T356 — `session reassign` hermetic ACs (print-only, writers, suggest fail-open).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_store::event_store::EventStore;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";

fn hermetic() -> assert_cmd::Command {
    let mut cmd = common::hermetic_bin();
    common::isolate_empty_home(&mut cmd);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn context_ids(vault: &Path, work_dir: &Path) -> (String, String) {
    fs::create_dir_all(work_dir).expect("work dir");
    let out = hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("context")
        .output()
        .expect("context");
    assert!(
        out.status.success(),
        "context must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env_path = work_dir.join(".env");
    let content = fs::read_to_string(&env_path).expect(".env after context");
    let mut project = String::new();
    let mut session = String::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("AI_BRAINS_PROJECT_ID=") {
            project = rest.trim().to_string();
        }
        if let Some(rest) = line.strip_prefix("AI_BRAINS_SESSION_ID=") {
            session = rest.trim().to_string();
        }
    }
    assert!(!project.is_empty(), "PROJECT_ID missing: {content}");
    assert!(!session.is_empty(), "SESSION_ID missing: {content}");
    (project, session)
}

fn set_alias(vault: &Path, project_id: &str, alias: &str) {
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("project")
        .arg("set-alias")
        .arg(project_id)
        .arg(alias)
        .assert()
        .success();
}

fn ingest_turn(vault: &Path, project_id: &str, session_id: &str, content: &str) {
    let turn_json = format!(
        r#"{{
            "session_id": "{session_id}",
            "project_id": "{project_id}",
            "harness_id": "00000000-0000-0000-0000-000000000000",
            "turn_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
            "privacy": "LocalOnly",
            "role": "user",
            "content": {content_json}
        }}"#,
        content_json = serde_json::to_string(content).expect("json")
    );
    hermetic()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();
}

fn pin_memory(vault: &Path, work_dir: &Path, project_id: &str, session_id: &str, content: &str) {
    hermetic()
        .current_dir(work_dir)
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .env("AI_BRAINS_PROJECT_ID", project_id)
        .env("AI_BRAINS_SESSION_ID", session_id)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn open_store(vault_path: &Path) -> ai_brains_store::event_store::SqliteEventStore {
    let _allow = ai_brains_core::temp_env::TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(
        vault_path.to_str().expect("utf8 vault"),
        &key,
    )
    .expect("open vault");
    ai_brains_store::event_store::SqliteEventStore::new(conn)
}

fn event_count(vault_path: &Path) -> usize {
    open_store(vault_path)
        .read_all_events()
        .expect("read events")
        .len()
}

fn session_started_hash(vault_path: &Path, session_id: &str) -> String {
    let events = open_store(vault_path).read_all_events().expect("events");
    events
        .iter()
        .find_map(|e| match &e.payload {
            ai_brains_events::Payload::SessionStarted(p)
                if p.session_id.to_string() == session_id =>
            {
                Some(e.payload_hash.clone())
            }
            _ => None,
        })
        .expect("SessionStarted")
}

fn projection_project_id(vault_path: &Path, table: &str, id_col: &str, id: &str) -> String {
    let store = open_store(vault_path);
    let conn = store.connection().lock().expect("lock");
    let sql = format!("SELECT project_id FROM {table} WHERE {id_col} = ?");
    conn.query_row(&sql, [id], |row| row.get(0))
        .unwrap_or_else(|e| panic!("{table} project_id: {e}"))
}

struct Fixture {
    _root: tempfile::TempDir,
    vault: PathBuf,
    dest_id: String,
    unbound_id: String,
    session_id: String,
}

fn setup_unbound_and_dest() -> Fixture {
    let root = tempdir().expect("tempdir");
    let vault = root.path().join("vault.db");
    init_vault(&vault);
    let dest_dir = root.path().join("dest");
    let unbound_dir = root.path().join("unbound");
    let (dest_id, _) = context_ids(&vault, &dest_dir);
    set_alias(&vault, &dest_id, "dest-proj");
    let (unbound_id, session_id) = context_ids(&vault, &unbound_dir);
    set_alias(&vault, &unbound_id, "cursor-unbound");
    ingest_turn(&vault, &unbound_id, &session_id, "work on dest-proj");
    pin_memory(
        &vault,
        &unbound_dir,
        &unbound_id,
        &session_id,
        "DECISION: dest-proj",
    );
    Fixture {
        _root: root,
        vault,
        dest_id,
        unbound_id,
        session_id,
    }
}

fn reassign_cmd(vault: &Path, args: &[&str]) -> assert_cmd::Command {
    let mut cmd = hermetic();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("session")
        .arg("reassign");
    for a in args {
        cmd.arg(a);
    }
    cmd
}

#[test]
fn session_reassign__print_only__written_false_event_count_unchanged() {
    let fx = setup_unbound_and_dest();
    let before = event_count(&fx.vault);
    let out = reassign_cmd(
        &fx.vault,
        &[
            &fx.session_id,
            "--to-project",
            "dest-proj",
            "--format",
            "json",
        ],
    )
    .output()
    .expect("reassign");
    assert!(
        out.status.success(),
        "AC1 exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    assert_eq!(v["written"], false);
    assert_eq!(event_count(&fx.vault), before);
}

#[test]
fn session_reassign__write_without_yes__exit_2() {
    let fx = setup_unbound_and_dest();
    let out = reassign_cmd(
        &fx.vault,
        &[&fx.session_id, "--to-project", "dest-proj", "--write"],
    )
    .output()
    .expect("reassign");
    assert_eq!(out.status.code(), Some(2), "AC2 exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--yes"), "AC2 stderr --yes; got: {stderr}");
}

#[test]
fn session_reassign__write_yes__moves_session_memory_turn_keeps_started_hash() {
    let fx = setup_unbound_and_dest();
    let started = session_started_hash(&fx.vault, &fx.session_id);
    let before = event_count(&fx.vault);
    let out = reassign_cmd(
        &fx.vault,
        &[
            &fx.session_id,
            "--to-project",
            "dest-proj",
            "--write",
            "--yes",
            "--format",
            "json",
        ],
    )
    .output()
    .expect("reassign");
    assert!(
        out.status.success(),
        "AC3; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    assert_eq!(v["written"], true);
    assert_eq!(v["assigned_by"], "human");
    assert_eq!(event_count(&fx.vault), before + 1);
    assert_eq!(
        session_started_hash(&fx.vault, &fx.session_id),
        started,
        "AC3 SessionStarted hash unchanged"
    );
    let dest = fx.dest_id.as_str();
    assert_eq!(
        projection_project_id(
            &fx.vault,
            "session_projection",
            "session_id",
            &fx.session_id
        ),
        dest
    );
    assert_eq!(
        projection_project_id(&fx.vault, "memory_projection", "session_id", &fx.session_id),
        dest
    );
    assert_eq!(
        projection_project_id(&fx.vault, "turn_projection", "session_id", &fx.session_id),
        dest
    );
    assert_ne!(fx.unbound_id, fx.dest_id);
}

#[cfg(feature = "graph")]
#[test]
fn session_reassign__write_yes__graph_edge_dest_only() {
    let fx = setup_unbound_and_dest();
    reassign_cmd(
        &fx.vault,
        &[
            &fx.session_id,
            "--to-project",
            "dest-proj",
            "--write",
            "--yes",
        ],
    )
    .assert()
    .success();
    let store = open_store(&fx.vault);
    let conn = store.connection().lock().expect("lock");
    let count = |dst: &str| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM graph_edge ge
             JOIN graph_node s ON ge.src_id = s.node_id
             JOIN graph_node d ON ge.dst_id = d.node_id
             WHERE s.external_id = ?1 AND d.external_id = ?2 AND ge.label = 'IN_PROJECT'",
            [&fx.session_id, dst],
            |row| row.get(0),
        )
        .expect("count edge")
    };
    assert_eq!(count(&fx.dest_id), 1, "AC4 dest IN_PROJECT");
    assert_eq!(count(&fx.unbound_id), 0, "AC4 no from IN_PROJECT");
}

#[test]
fn session_reassign__suggest_offline__exit_0_skip_no_events() {
    let fx = setup_unbound_and_dest();
    let before = event_count(&fx.vault);
    let out = reassign_cmd(&fx.vault, &["--suggest", "--format", "json"])
        .env("AI_BRAINS_MODEL_URL", "http://127.0.0.1:1")
        .output()
        .expect("suggest");
    assert!(
        out.status.success(),
        "AC5 exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.to_ascii_lowercase().contains("skip"),
        "AC5 skip on stderr; got: {stderr}"
    );
    assert_eq!(event_count(&fx.vault), before);
}

fn spawn_chat_completer(content: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let inner = content.to_string();
    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            loop {
                match stream.read(&mut tmp) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    Err(_) => break,
                }
                if let Some(header_end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header_end = header_end + 4;
                    let headers = String::from_utf8_lossy(&buf[..header_end]).to_ascii_lowercase();
                    let content_len = headers.lines().find_map(|line| {
                        line.strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    });
                    if let Some(len) = content_len {
                        while buf.len() < header_end + len {
                            match stream.read(&mut tmp) {
                                Ok(0) => break,
                                Ok(n) => buf.extend_from_slice(&tmp[..n]),
                                Err(_) => break,
                            }
                        }
                    }
                    break;
                }
            }
            let payload = serde_json::json!({
                "choices": [{ "message": { "content": inner } }]
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (format!("http://{addr}"), handle)
}

#[test]
fn session_reassign__suggest_write_yes__stdout_llm_assigned() {
    let fx = setup_unbound_and_dest();
    let (url, server) = spawn_chat_completer(r#"{"alias":"dest-proj","confidence":0.91}"#);
    let out = reassign_cmd(
        &fx.vault,
        &["--suggest", "--write", "--yes", "--format", "json"],
    )
    .env("AI_BRAINS_MODEL_URL", &url)
    .env("AI_BRAINS_COMPLETION_MODEL", "fake-llm")
    .output()
    .expect("suggest write");
    let _ = server.join();
    assert!(
        out.status.success(),
        "AC6; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let proposals = v["proposals"].as_array().expect("proposals");
    assert!(!proposals.is_empty(), "AC6 proposals: {v}");
    assert_eq!(proposals[0]["assigned_by"], "llm");
    let events = open_store(&fx.vault).read_all_events().expect("events");
    let payload = events
        .iter()
        .find_map(|e| match &e.payload {
            ai_brains_events::Payload::SessionReassigned(p) => Some(p),
            _ => None,
        })
        .expect("SessionReassigned");
    assert_eq!(payload.assigned_by, "llm");
}

#[test]
fn session_reassign__suggest_write_yes__human_llm_tag() {
    let fx = setup_unbound_and_dest();
    let (url, server) = spawn_chat_completer(r#"{"alias":"dest-proj","confidence":0.91}"#);
    let out = reassign_cmd(
        &fx.vault,
        &["--suggest", "--write", "--yes", "--format", "human"],
    )
    .env("AI_BRAINS_MODEL_URL", &url)
    .env("AI_BRAINS_COMPLETION_MODEL", "fake-llm")
    .output()
    .expect("human suggest");
    let _ = server.join();
    assert!(
        out.status.success(),
        "AC6 human; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("⟨llm-assigned⟩") || stdout.contains("assigned_by=llm"),
        "AC6 human chrome; got: {stdout}"
    );
}

#[test]
fn session_reassign__suggest_contradiction__assignment_suspicious() {
    let root = tempdir().expect("tempdir");
    let vault = root.path().join("vault.db");
    init_vault(&vault);
    let dest_dir = root.path().join("dest");
    let named_dir = root.path().join("named");
    let unbound_dir = root.path().join("unbound");
    let (dest_id, _) = context_ids(&vault, &dest_dir);
    set_alias(&vault, &dest_id, "dest-y");
    let (named_id, _) = context_ids(&vault, &named_dir);
    set_alias(&vault, &named_id, "named-x");
    let (unbound_id, session_id) = context_ids(&vault, &unbound_dir);
    set_alias(&vault, &unbound_id, "cursor-unbound");
    ingest_turn(&vault, &unbound_id, &session_id, "this is clearly named-x");
    let (url, server) = spawn_chat_completer(r#"{"alias":"dest-y","confidence":0.88}"#);
    let out = reassign_cmd(
        &vault,
        &["--suggest", "--write", "--yes", "--format", "json"],
    )
    .env("AI_BRAINS_MODEL_URL", &url)
    .env("AI_BRAINS_COMPLETION_MODEL", "fake-llm")
    .output()
    .expect("contradiction");
    let _ = server.join();
    assert!(
        out.status.success(),
        "AC7; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    let proposals = v["proposals"].as_array().expect("proposals");
    assert_eq!(proposals[0]["assignment_suspicious"], true);
    assert_eq!(proposals[0]["to_project_id"], dest_id);
    let events = open_store(&vault).read_all_events().expect("events");
    let payload = events
        .iter()
        .find_map(|e| match &e.payload {
            ai_brains_events::Payload::SessionReassigned(p) => Some(p),
            _ => None,
        })
        .expect("event");
    assert!(payload.suspicious);
    assert_eq!(payload.to_project_id.to_string(), dest_id);
}

#[test]
fn session_reassign__suggest_write_unparsable__written_false() {
    let fx = setup_unbound_and_dest();
    let before = event_count(&fx.vault);
    let (url, server) = spawn_chat_completer("not-json");
    let out = reassign_cmd(
        &fx.vault,
        &["--suggest", "--write", "--yes", "--format", "json"],
    )
    .env("AI_BRAINS_MODEL_URL", &url)
    .env("AI_BRAINS_COMPLETION_MODEL", "fake-llm")
    .output()
    .expect("unparsable");
    let _ = server.join();
    assert!(
        out.status.success(),
        "unparsable; stderr={} stdout={}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("json");
    assert_eq!(v["written"], false);
    assert_eq!(v["skipped"], false);
    assert_eq!(event_count(&fx.vault), before);
}

#[test]
fn session_reassign__help__lists_flags_no_apply_token() {
    let out = hermetic()
        .arg("session")
        .arg("reassign")
        .arg("--help")
        .output()
        .expect("help");
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("--suggest"), "{help}");
    assert!(help.contains("--write"), "{help}");
    assert!(help.contains("--yes"), "{help}");
    assert!(!help.contains("--apply"), "{help}");
}
