#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
const PROJECT_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SESSION_ID: &str = "11111111-1111-1111-1111-111111111111";
const DISTINCTIVE: &str = "shadow-unique-turn-content-T147-xyzzy";

fn init_vault(vault_path: &Path) {
    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn ingest_user_turn(vault_path: &Path, content: &str) {
    let turn_json = format!(
        r#"{{
            "session_id": "{SESSION_ID}",
            "project_id": "{PROJECT_ID}",
            "harness_id": "00000000-0000-0000-0000-000000000000",
            "turn_id": "{}",
            "privacy": "LocalOnly",
            "role": "user",
            "content": "{}"
        }}"#,
        uuid::Uuid::new_v4(),
        content.replace('"', "\\\"")
    );
    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();
}

fn open_store(
    vault_path: &Path,
) -> Result<ai_brains_store::event_store::SqliteEventStore, Box<dyn std::error::Error>> {
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    // Read path only — do not migrate (source/dest already schema-ready).
    let conn = ai_brains_store::connection::VaultConnection::open(vault_path, &key)?;
    Ok(ai_brains_store::event_store::SqliteEventStore::new(conn))
}

fn turn_contents(vault_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use ai_brains_events::Payload;
    use ai_brains_store::event_store::EventStore;

    let store = open_store(vault_path)?;
    let events = store.read_all_events()?;
    let mut contents = Vec::new();
    for env in events {
        match env.payload {
            Payload::UserPromptRecorded(p) => contents.push(p.content),
            Payload::AssistantFinalRecorded(p) => contents.push(p.content),
            _ => {}
        }
    }
    Ok(contents)
}

#[test]
fn shadow_create__same_source_and_destination__refuses() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&vault)
        .arg("--destination")
        .arg(&vault)
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("same location"));
}

#[test]
fn shadow_create__destination_equals_live_vault__refuses() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let live = dir.path().join("live.db");
    init_vault(&source);
    init_vault(&live);

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&live)
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("live vault"));
}

#[test]
fn shadow_create__destination_inside_live_vault_parent__refuses() {
    let dir = tempdir().expect("tempdir");
    let live_parent = dir.path().join("live-home");
    fs::create_dir_all(&live_parent).expect("live parent");
    let live = live_parent.join("live.db");
    let dest = live_parent.join("shadow-sibling.db");
    let source_dir = dir.path().join("source-home");
    fs::create_dir_all(&source_dir).expect("source parent");
    let source = source_dir.join("source.db");

    init_vault(&source);
    init_vault(&live);

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("inside the live vault parent"));
}

#[test]
fn shadow_create__dry_run__writes_no_files() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("shadow").join("dest.db");
    init_vault(&source);

    let source_meta_before = fs::metadata(&source).expect("source metadata before");
    let source_len_before = source_meta_before.len();

    assert!(!dest.exists());
    assert!(!dest.parent().expect("parent").exists());

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(!dest.exists(), "dry-run must not create destination vault");
    assert!(
        !dest.parent().expect("parent").exists()
            || fs::read_dir(dest.parent().unwrap())
                .unwrap()
                .next()
                .is_none(),
        "dry-run must not create destination parent contents"
    );
    let manifest = dest.parent().expect("parent").join("shadow-manifest.json");
    assert!(
        !manifest.exists(),
        "dry-run must not write shadow-manifest.json"
    );

    // Source must not be migrated/rewritten by dry-run (T147-F5).
    let source_meta_after = fs::metadata(&source).expect("source metadata after");
    assert_eq!(
        source_meta_after.len(),
        source_len_before,
        "dry-run must not grow/rewrite the source vault (no migrate on source)"
    );
}

/// Destination that exists as a reparse/symlink must be refused (shadow wiring).
///
/// Windows: directory junction (no SeCreateSymbolicLinkPrivilege).
/// Non-Windows: file symlink as destination path.
#[test]
fn shadow_create__destination_reparse_or_symlink__refuses() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    init_vault(&source);

    let dest = {
        #[cfg(windows)]
        {
            let real = dir.path().join("real-dest-dir");
            fs::create_dir_all(&real).expect("real dest dir");
            let junction = dir.path().join("dest-junction");
            let status = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "mklink",
                    "/J",
                    &junction.to_string_lossy(),
                    &real.to_string_lossy(),
                ])
                .status()
                .expect("spawn mklink /J");
            assert!(
                status.success(),
                "mklink /J failed (exit {status}); directory junctions should not need elevation"
            );
            // Parent of destination is a junction → refuse_unsafe_destination must fire.
            junction.join("shadow.db")
        }
        #[cfg(not(windows))]
        {
            let real_file = dir.path().join("real-target.db");
            fs::write(&real_file, b"placeholder").expect("write real");
            let link = dir.path().join("dest-link.db");
            std::os::unix::fs::symlink(&real_file, &link).expect("unix symlink");
            link
        }
    };

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction")),
        );
}

#[test]
fn shadow_create__happy_path__redacts_turns_and_writes_manifest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest_dir = dir.path().join("shadow-out");
    let dest = dest_dir.join("dest.db");
    init_vault(&source);
    ingest_user_turn(&source, DISTINCTIVE);

    let source_contents = turn_contents(&source).expect("read source turns");
    assert!(
        source_contents.iter().any(|c| c.contains(DISTINCTIVE)),
        "source must contain distinctive content before shadow; got {source_contents:?}"
    );

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .assert()
        .success()
        .stdout(predicate::str::contains("Shadow vault created"));

    assert!(dest.exists(), "destination vault must exist");
    let manifest_path = dest_dir.join("shadow-manifest.json");
    assert!(
        manifest_path.exists(),
        "shadow-manifest.json must exist next to destination"
    );

    let manifest_text = fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_text).expect("manifest JSON");
    assert_eq!(manifest["version"], 1);
    assert_eq!(manifest["redaction_policy"], "redact-turn-content");
    assert!(
        manifest["event_count"].as_u64().unwrap_or(0) >= 1,
        "event_count must be >= 1; got {}",
        manifest["event_count"]
    );
    assert!(
        manifest["source_fingerprint"]
            .as_str()
            .map(|s| !s.is_empty())
            .unwrap_or(false),
        "source_fingerprint must be present"
    );
    assert_eq!(manifest["dry_run"], false);
    assert!(
        manifest["source_path"]
            .as_str()
            .map(|s| s.contains("source.db"))
            .unwrap_or(false),
        "source_path should reference source vault"
    );
    assert!(
        manifest["destination_path"]
            .as_str()
            .map(|s| s.contains("dest.db"))
            .unwrap_or(false),
        "destination_path should reference dest vault"
    );

    // Default redaction: turn content is [REDACTED], distinctive plaintext gone.
    let dest_contents = turn_contents(&dest).expect("read dest turns");
    assert!(!dest_contents.is_empty(), "dest must have turn events");
    for c in &dest_contents {
        assert_eq!(
            c, "[REDACTED]",
            "default shadow must redact turn content; got {c:?}"
        );
    }
    assert!(
        dest_contents.iter().all(|c| !c.contains(DISTINCTIVE)),
        "dest must not contain distinctive plaintext"
    );

    // Source remains unredacted.
    let source_after = turn_contents(&source).expect("re-read source");
    assert!(
        source_after.iter().any(|c| c.contains(DISTINCTIVE)),
        "source vault must not be mutated by shadow create"
    );
}

#[test]
fn shadow_create__no_redact_turn_content__preserves_content() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("shadow-out").join("dest.db");
    init_vault(&source);
    ingest_user_turn(&source, DISTINCTIVE);

    Command::cargo_bin("ai-brains")
        .expect("binary")
        .arg("--no-project-context")
        .arg("shadow")
        .arg("create")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--no-redact-turn-content")
        .assert()
        .success();

    let manifest_path = dest.parent().expect("parent").join("shadow-manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("manifest"))
            .expect("manifest JSON");
    assert_eq!(manifest["redaction_policy"], "no-redact-turn-content");

    let dest_contents = turn_contents(&dest).expect("read dest turns");
    assert!(
        dest_contents.iter().any(|c| c.contains(DISTINCTIVE)),
        "with --no-redact-turn-content, distinctive content must be preserved; got {dest_contents:?}"
    );
}
