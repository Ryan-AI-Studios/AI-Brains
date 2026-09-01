#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T339 OpenCode Windows shim + well-known bin resolve.

use ai_brains_adapters::{
    OpenCodeImportOptions, ResolveOutcome, export_session_via_cli, import_opencode_sessions,
    resolve_opencode_bin,
};
use ai_brains_capture::{CaptureService, CaptureSink};
use ai_brains_core::ids::ProjectId;
use ai_brains_core::temp_env::TempEnv;
use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_events::Envelope;
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct TestSink;

impl CaptureSink for TestSink {
    fn append(&mut self, _envelope: Envelope) {}
}

fn touch(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, b"shim").expect("write");
}

fn isolate(tmp: &Path) -> Vec<TempEnv> {
    let empty_path = tmp.join("empty-path");
    let appdata = tmp.join("appdata");
    let home = tmp.join("home");
    fs::create_dir_all(&empty_path).expect("empty path");
    fs::create_dir_all(&appdata).expect("appdata");
    fs::create_dir_all(&home).expect("home");
    vec![
        TempEnv::remove("AI_BRAINS_OPENCODE_BIN"),
        TempEnv::remove("OPENCODE_BIN_PATH"),
        TempEnv::set("PATH", &empty_path),
        TempEnv::set("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
        TempEnv::set("APPDATA", &appdata),
        TempEnv::set("USERPROFILE", &home),
        TempEnv::set("HOME", &home),
    ]
}

fn open_import_pair(
    vault_dir: &Path,
) -> (
    VaultConnection,
    CaptureService,
    TestSink,
    OpenCodeImportOptions,
) {
    fs::create_dir_all(vault_dir).expect("vault");
    let key = DataKey::generate();
    let sql_key = SqlCipherKey::from_data_key(&key);
    let conn = VaultConnection::open(vault_dir.join("v.db"), &sql_key).expect("open");
    conn.migrate().expect("migrate");
    let _store = SqliteEventStore::new(conn.clone());
    let sink = TestSink;
    let options = OpenCodeImportOptions {
        days: 2,
        force: false,
        dry_run: true,
        max_sessions: 10,
        default_project_id: ProjectId::new(),
        allow_default_project: false,
        list_json_override: None,
        export_json_override_dir: None,
        cursor_path_override: Some(vault_dir.join("cursor.json")),
        config_dir_override: None,
        force_missing_binary: false,
        bin_override: None,
        list_cap: 10,
    };
    (conn, CaptureService::new(), sink, options)
}

#[cfg(windows)]
#[test]
fn resolve_opencode_bin__windows_cmd_shim__preferred_over_extensionless() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let path_dir = tmp.path().join("pathdir");
    fs::create_dir_all(&path_dir).expect("pathdir");
    touch(&path_dir.join("opencode"));
    let cmd = path_dir.join("opencode.cmd");
    touch(&cmd);
    let _path = TempEnv::set("PATH", &path_dir);
    let ResolveOutcome { path, .. } = resolve_opencode_bin(None);
    let got = path.expect("resolved");
    assert_eq!(got, cmd, "must prefer .cmd over extensionless POSIX shim");
}

#[cfg(windows)]
#[test]
fn resolve_opencode_bin__well_known_windows_x64__when_path_empty() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let _path = TempEnv::set("PATH", "");
    let nested = tmp
        .path()
        .join("appdata")
        .join("npm")
        .join("node_modules")
        .join("opencode-ai")
        .join("node_modules")
        .join("opencode-windows-x64")
        .join("bin")
        .join("opencode.exe");
    touch(&nested);
    let ResolveOutcome { path, .. } = resolve_opencode_bin(None);
    assert_eq!(path.expect("resolved"), nested);
}

#[cfg(windows)]
#[test]
fn resolve_opencode_bin__windows_x64_nested__beats_opencode_ai_bin() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let _path = TempEnv::set("PATH", "");
    let npm = tmp.path().join("appdata").join("npm").join("node_modules");
    let nested = npm
        .join("opencode-ai")
        .join("node_modules")
        .join("opencode-windows-x64")
        .join("bin")
        .join("opencode.exe");
    let ai_bin = npm.join("opencode-ai").join("bin").join("opencode.exe");
    touch(&nested);
    touch(&ai_bin);
    let ResolveOutcome { path, .. } = resolve_opencode_bin(None);
    assert_eq!(path.expect("resolved"), nested);
}

#[cfg(not(windows))]
#[test]
fn resolve_opencode_bin__unix_path_file__used() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let path_dir = tmp.path().join("pathdir");
    fs::create_dir_all(&path_dir).expect("pathdir");
    let bin = path_dir.join("opencode");
    touch(&bin);
    let _path = TempEnv::set("PATH", &path_dir);
    let ResolveOutcome { path, .. } = resolve_opencode_bin(None);
    assert_eq!(path.expect("resolved"), bin);
}

#[rstest::rstest]
#[case::brains_wins("brains")]
#[case::quoted("quoted")]
#[case::path_env_alone("pathenv")]
#[case::missing_brains_falls_through("missing")]
#[case::brains_beats_pathenv("both")]
fn resolve_opencode_bin__env_override__rstest_cases(#[case] kind: &str) {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let path_dir = tmp.path().join("pathdir");
    fs::create_dir_all(&path_dir).expect("pathdir");
    // Windows PATH×PATHEXT skips extensionless shims; Unix looks for `opencode`.
    let path_bin = if cfg!(windows) {
        path_dir.join("opencode.cmd")
    } else {
        path_dir.join("opencode")
    };
    touch(&path_bin);
    let brains = tmp.path().join("brains-opencode.cmd");
    let pathenv = tmp.path().join("pathenv-opencode.cmd");
    touch(&brains);
    touch(&pathenv);
    let _path = TempEnv::set("PATH", &path_dir);

    let outcome = match kind {
        "brains" => {
            let _e = TempEnv::set("AI_BRAINS_OPENCODE_BIN", &brains);
            resolve_opencode_bin(None)
        }
        "quoted" => {
            let quoted = format!("\"{}\"", brains.display());
            let _e = TempEnv::set("AI_BRAINS_OPENCODE_BIN", quoted);
            resolve_opencode_bin(None)
        }
        "pathenv" => {
            let _e = TempEnv::set("OPENCODE_BIN_PATH", &pathenv);
            resolve_opencode_bin(None)
        }
        "missing" => {
            let missing = tmp.path().join("no-such-opencode.exe");
            let _e = TempEnv::set("AI_BRAINS_OPENCODE_BIN", &missing);
            resolve_opencode_bin(None)
        }
        "both" => {
            let _a = TempEnv::set("AI_BRAINS_OPENCODE_BIN", &brains);
            let _b = TempEnv::set("OPENCODE_BIN_PATH", &pathenv);
            resolve_opencode_bin(None)
        }
        other => panic!("unknown case {other}"),
    };
    let got = outcome.path.expect("resolved");
    match kind {
        "brains" | "quoted" | "both" => assert_eq!(got, brains),
        "pathenv" => assert_eq!(got, pathenv),
        "missing" => assert_eq!(got, path_bin),
        _ => {}
    }
}

#[test]
fn import_opencode__unresolved_bin__soft_skip_with_sorted_attempts() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let _path = TempEnv::set("PATH", "");
    let vault = tmp.path().join("vault");
    let (conn, service, mut sink, options) = open_import_pair(&vault);
    let stats = import_opencode_sessions(&conn, &service, &mut sink, options).expect("import");
    assert_eq!(stats.skipped_missing_binary, 1);
    assert!(stats.resolved_bin.is_none());
    let attempts = stats.binary_attempts.expect("attempts");
    assert!(!attempts.is_empty(), "must record checked candidates");
    let mut sorted = attempts.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(attempts, sorted, "attempts must be sorted unique");
}

#[test]
fn export_session_via_cli__unresolved_bin__empty_ok() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let _path = TempEnv::set("PATH", "");
    let turns = export_session_via_cli("ses_missing").expect("ok");
    assert!(turns.is_empty());
}

struct CwdRestore(std::path::PathBuf);

impl Drop for CwdRestore {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[test]
fn resolve_opencode_bin__relative_override__absolutized() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempdir().expect("tmp");
    let _g = isolate(tmp.path());
    let prev = std::env::current_dir().expect("cwd");
    let _cwd = CwdRestore(prev);
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let rel = Path::new("opencode.cmd");
    touch(&tmp.path().join("opencode.cmd"));
    let outcome = resolve_opencode_bin(Some(rel));
    let got = outcome.path.expect("resolved");
    assert!(got.is_absolute(), "spawn path must be absolute: {got:?}");
    assert!(
        got.file_name().is_some_and(|n| n == "opencode.cmd"),
        "{got:?}"
    );
}

#[test]
fn resolve_opencode_bin__no_bare_name_spawn__source_guard() {
    let src = include_str!("../src/opencode.rs");
    assert!(
        !src.contains("Command::new(\"opencode\")"),
        "adapter must not spawn bare opencode name"
    );
    let hook = include_str!("../../ai-brains-cli/src/commands/opencode_hook.rs");
    assert!(
        !hook.contains("Command::new(\"opencode\")"),
        "hook must not spawn bare opencode"
    );
}
