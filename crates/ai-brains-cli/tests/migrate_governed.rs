#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T168 — `migrate governed` integration tests (assert_cmd + tempfile).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
const PROJECT_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SESSION_ID: &str = "11111111-1111-1111-1111-111111111111";
const DISTINCTIVE: &str = "migrate-unique-plaintext-T168-xyzzy-body-secret";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn pin_via_cli(vault_path: &Path, content: &str) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .env("AI_BRAINS_PROJECT_ID", PROJECT_ID)
        .env("AI_BRAINS_SESSION_ID", SESSION_ID)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn seed_memory_pinned(vault_path: &Path, content: &str, with_project: bool) {
    use ai_brains_core::ids::{MemoryId, ProjectId};
    use ai_brains_core::privacy::Privacy;
    use ai_brains_crypto::SqlCipherKey;
    use ai_brains_events::constructors::EventBuilder;
    use ai_brains_events::payload::MemoryPinnedPayload;
    use ai_brains_events::{Actor, AggregateType, Payload};
    use ai_brains_store::connection::VaultConnection;
    use ai_brains_store::event_store::{EventStore, SqliteEventStore};

    let key = SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = VaultConnection::open(vault_path, &key).expect("open");
    // Already migrated by init; do not re-migrate (source integrity).
    let store = SqliteEventStore::new(conn);
    let memory_id = MemoryId::new();
    let project_id = if with_project {
        Some(ProjectId::from_uuid(
            uuid::Uuid::parse_str(PROJECT_ID).expect("project uuid"),
        ))
    } else {
        None
    };
    let env = EventBuilder::new(
        AggregateType::Memory,
        memory_id.as_uuid(),
        Actor::System,
        Privacy::LocalOnly,
    )
    .build(Payload::MemoryPinned(MemoryPinnedPayload {
        memory_id,
        content: content.into(),
        session_id: None,
        project_id,
        tx_id: None,
        rank: None,
        source_tag: Some("test".into()),
        query_text: None,
    }))
    .expect("build pin");
    store.append_event(&env).expect("append pin");
}

fn open_store(
    vault_path: &Path,
) -> Result<ai_brains_store::event_store::SqliteEventStore, Box<dyn std::error::Error>> {
    let key = ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string());
    let conn = ai_brains_store::connection::VaultConnection::open(vault_path, &key)?;
    Ok(ai_brains_store::event_store::SqliteEventStore::new(conn))
}

fn event_count(vault_path: &Path) -> usize {
    use ai_brains_store::event_store::EventStore;
    let store = open_store(vault_path).expect("open");
    store.read_all_events().expect("read").len()
}

fn migrate_cmd() -> Command {
    let mut c = common::hermetic_bin();
    c.arg("--no-project-context");
    c
}

fn paths(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let source = dir.join("source.db");
    let dest = dir.join("dest").join("dest.db");
    let report = dir.join("report.json");
    (source, dest, report)
}

// ---------------------------------------------------------------------------
// Dry-run
// ---------------------------------------------------------------------------

#[test]
fn migrate_governed__dry_run__writes_report_only() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, DISTINCTIVE, true);
    let source_len = fs::metadata(&source).expect("meta").len();
    let source_count = event_count(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(report.exists(), "report must be written on dry-run");
    assert!(!dest.exists(), "dry-run must not create dest vault");
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    assert!(
        !manifest.exists(),
        "dry-run must not write migrate-manifest"
    );
    assert_eq!(fs::metadata(&source).unwrap().len(), source_len);
    assert_eq!(event_count(&source), source_count);

    let body = fs::read_to_string(&report).expect("report body");
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["command"], "migrate.governed");
}

#[test]
fn migrate_governed__report_has_no_plaintext_bodies() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, DISTINCTIVE, true);
    pin_via_cli(&source, DISTINCTIVE);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();

    let body = fs::read_to_string(&report).expect("report");
    assert!(
        !body.contains(DISTINCTIVE),
        "report must not contain distinctive plaintext body"
    );
}

#[test]
fn migrate_governed__report_schema_version_1() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "schema check pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();

    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    for key in [
        "schema_version",
        "command",
        "dry_run",
        "created_at",
        "source_path",
        "destination_path",
        "source_fingerprint",
        "plan_hash",
        "report_hash",
        "event_counts",
        "classification",
        "unresolved",
        "privacy",
        "gaps",
        "content_hashes",
        "replay_consistency",
        "ce_honesty",
        "rollback",
        "legacy_import_applied",
        "t167_plan_hash",
        "manifest_written",
    ] {
        assert!(v.get(key).is_some(), "missing key {key}");
    }
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["ce_honesty"]["claims_cryptographic_erasure"], false);
    assert_eq!(v["rollback"]["source_modified"], false);
    assert_eq!(v["manifest_written"], false);
}

#[test]
fn migrate_governed__ce_honesty_false() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["ce_honesty"]["claims_cryptographic_erasure"], false);
}

#[test]
fn migrate_governed__rollback_source_modified_false() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "rollback pin", true);
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["rollback"]["source_modified"], false);
}

#[test]
fn migrate_governed__plan_hash_matches_t167() {
    use ai_brains_control_plane::{ImportOpts, classify_legacy};
    use ai_brains_core::ids::PrincipalId;
    use ai_brains_store::event_store::EventStore;
    use uuid::Uuid;

    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "plan hash pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();

    let store = open_store(&source).expect("open");
    let events = store.read_all_events().expect("events");
    let plan = classify_legacy(
        &events,
        &ImportOpts {
            dry_run: true,
            include_truncated_summaries: false,
            default_scope: None,
            principal_id: PrincipalId::from_uuid(Uuid::nil()),
            command_id: None,
        },
    )
    .expect("classify");

    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["plan_hash"], plan.plan_hash);
    assert_eq!(v["t167_plan_hash"], plan.plan_hash);
}

#[test]
fn migrate_governed__source_fingerprint__stable_across_file_mtime_change() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "fp stable", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();
    let v1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    let fp1 = v1["source_fingerprint"].as_str().unwrap().to_string();

    // Touch mtime without changing content.
    let new_time = SystemTime::now() + Duration::from_secs(120);
    filetime_touch(&source, new_time);

    let report2 = dir.path().join("report2.json");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .assert()
        .success();
    let v2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report2).unwrap()).unwrap();
    assert_eq!(v2["source_fingerprint"], fp1);
}

/// Best-effort mtime touch without extra deps (Windows + Unix).
fn filetime_touch(path: &Path, when: SystemTime) {
    let file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for touch");
    file.set_modified(when).expect("set_modified");
}

// ---------------------------------------------------------------------------
// Safety refusals
// ---------------------------------------------------------------------------

#[test]
fn migrate_governed__refuse_source_equals_dest() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    let report = dir.path().join("report.json");
    init_vault(&vault);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&vault)
        .arg("--destination")
        .arg(&vault)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("same location"));
}

#[test]
fn migrate_governed__refuse_dest_equals_live() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let live = dir.path().join("live.db");
    let report = dir.path().join("report.json");
    init_vault(&source);
    init_vault(&live);

    migrate_cmd()
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&live)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("live vault"));
}

#[test]
fn migrate_governed__refuse_dest_inside_live_parent() {
    let dir = tempdir().expect("tempdir");
    let live_parent = dir.path().join("live-home");
    fs::create_dir_all(&live_parent).unwrap();
    let live = live_parent.join("live.db");
    let dest = live_parent.join("migrate-sibling.db");
    let source_dir = dir.path().join("source-home");
    fs::create_dir_all(&source_dir).unwrap();
    let source = source_dir.join("source.db");
    let report = dir.path().join("report.json");
    init_vault(&source);
    init_vault(&live);

    migrate_cmd()
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("inside the live vault parent"));
}

#[test]
fn migrate_governed__refuse_live_source_without_flag() {
    let dir = tempdir().expect("tempdir");
    let live = dir.path().join("live.db");
    let dest = dir.path().join("dest.db");
    let report = dir.path().join("report.json");
    init_vault(&live);

    migrate_cmd()
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&live)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("live vault"));
}

#[test]
fn migrate_governed__allow_live_source_still_refuses_live_dest() {
    let dir = tempdir().expect("tempdir");
    let live = dir.path().join("live.db");
    let report = dir.path().join("report.json");
    init_vault(&live);

    // source == live with allow flag, but dest also == live → refuse dest.
    migrate_cmd()
        .env("AI_BRAINS_VAULT_PATH", live.as_os_str())
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&live)
        .arg("--destination")
        .arg(&live)
        .arg("--report")
        .arg(&report)
        .arg("--allow-live-source")
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("same location").or(predicate::str::contains("live vault")),
        );
}

#[test]
fn migrate_governed__refuse_report_equals_source() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    init_vault(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&source)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("report path equals the source"));
}

#[test]
fn migrate_governed__refuse_report_equals_dest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    init_vault(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&dest)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "report path equals the destination",
        ));
}

#[test]
fn migrate_governed__refuse_report_equals_migrate_manifest() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, _report) = paths(dir.path());
    init_vault(&source);
    // Sibling of dest: dest_dir/migrate-manifest.json
    let manifest_as_report = dest
        .parent()
        .expect("dest parent")
        .join("migrate-manifest.json");

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&manifest_as_report)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("migrate-manifest"));
}

#[test]
fn migrate_governed__missing_source__not_found_exit_4() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("does-not-exist.db");
    let dest = dir.path().join("dest.db");
    let report = dir.path().join("report.json");

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("NOT_FOUND"));
}

#[test]
fn migrate_governed__key_after_subcommand__accepted() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "key-after-flag pin", true);

    // --key after `governed` (not only before `migrate`) must be accepted.
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--key")
        .arg(ZERO_KEY)
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(
        report.exists(),
        "report written with --key after subcommand"
    );
}

#[test]
fn migrate_governed__dry_run__source_fingerprint_stable() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "fp integrity pin", true);

    let count_before = event_count(&source);
    let len_before = fs::metadata(&source).unwrap().len();

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();

    assert_eq!(event_count(&source), count_before);
    assert_eq!(fs::metadata(&source).unwrap().len(), len_before);

    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["rollback"]["source_modified"], false);
    let fp_report = v["source_fingerprint"].as_str().unwrap().to_string();

    // Second dry-run must yield the same fingerprint (source content unchanged).
    let report2 = dir.path().join("report2.json");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .assert()
        .success();
    let v2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report2).unwrap()).unwrap();
    assert_eq!(v2["source_fingerprint"], fp_report);
    assert_eq!(event_count(&source), count_before);
}

#[test]
fn migrate_governed__both_dry_run_and_confirm__invalid_payload() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--dry-run")
        .arg("--confirm")
        .assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("INVALID_PAYLOAD"));
}

// ---------------------------------------------------------------------------
// Confirm apply
// ---------------------------------------------------------------------------

#[test]
fn migrate_governed__confirm__creates_dest_and_applies() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "confirm pin content", true);
    let source_count = event_count(&source);
    let source_len = fs::metadata(&source).unwrap().len();

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicate::str::contains("migrate governed"));

    assert!(dest.exists(), "confirm must create dest");
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    assert!(manifest.exists(), "confirm must write migrate-manifest");
    assert_eq!(event_count(&source), source_count);
    assert_eq!(fs::metadata(&source).unwrap().len(), source_len);

    let dest_count = event_count(&dest);
    assert!(
        dest_count >= source_count,
        "dest should have at least source events (copied + import); got {dest_count} vs source {source_count}"
    );

    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["dry_run"], false);
    assert_eq!(v["manifest_written"], true);
    assert_eq!(v["legacy_import_applied"], true);
    assert_eq!(v["rollback"]["source_modified"], false);
    assert!(v["classification"]["evidence"].as_u64().unwrap_or(0) >= 1);

    let mf: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).unwrap()).unwrap();
    assert_eq!(mf["source_fingerprint"], v["source_fingerprint"]);
    assert_eq!(mf["plan_hash"], v["plan_hash"]);
}

#[test]
fn migrate_governed__confirm_second_run__idempotent() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "idempotent pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success();

    let dest_count_after_first = event_count(&dest);
    let v1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v1["legacy_import_applied"], true);
    let first_would_import = v1["event_counts"]["would_import_appends"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        first_would_import >= 1,
        "first confirm should report applied imports; got would_import_appends={first_would_import}"
    );

    let report2 = dir.path().join("report2.json");

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--confirm")
        .assert()
        .success();

    let dest_count_after_second = event_count(&dest);
    // Idempotent re-apply: no new source envelope copies; already_imported skips.
    // Dest may gain at most a second LegacyImportApplied if T167 emits it when
    // there were successful applies — with pure already_imported, count stable or
    // nearly stable. Must not double source events.
    assert!(
        dest_count_after_second <= dest_count_after_first + 2,
        "second run must not re-copy source events: first={dest_count_after_first} second={dest_count_after_second}"
    );

    let v2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report2).unwrap()).unwrap();
    // R1-01: fold T167 apply outcomes — second run shows dest-probed already_imported.
    let already = v2["classification"]["already_imported"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        already >= 1,
        "second confirm report must surface classification.already_imported > 0; got {already}"
    );
    let second_would_import = v2["event_counts"]["would_import_appends"]
        .as_u64()
        .unwrap_or(u64::MAX);
    assert!(
        second_would_import < first_would_import || second_would_import == 0,
        "second run would_import_appends should be low/zero (applied this run); first={first_would_import} second={second_would_import}"
    );
    // legacy_import_applied remains true when T167 still emits the audit event
    // (or false if pure skip) — either way apply path ran; require report key present.
    assert!(v2.get("legacy_import_applied").is_some());
}

#[test]
fn migrate_governed__confirm_second_run_with_copy_events__no_duplicate_source_events() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "no dup pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .arg("--copy-events")
        .assert()
        .success();

    let dest_count_1 = event_count(&dest);
    let report2 = dir.path().join("report2.json");

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--confirm")
        .arg("--copy-events")
        .assert()
        .success();

    let dest_count_2 = event_count(&dest);
    assert!(
        dest_count_2 <= dest_count_1 + 2,
        "M17: --copy-events on re-apply must not re-append source envelopes (first={dest_count_1} second={dest_count_2})"
    );
}

#[test]
fn migrate_governed__reapply_missing_manifest__refuses() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "missing mf pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success();

    // Remove manifest; leave dest non-empty.
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    fs::remove_file(&manifest).expect("remove manifest");

    let report2 = dir.path().join("report2.json");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--confirm")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("migrate-manifest"));
}

#[test]
fn migrate_governed__reapply_wrong_source_fingerprint__refuses() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "wrong fp pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success();

    // Corrupt manifest fingerprint.
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    let mut mf: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).unwrap()).unwrap();
    mf["source_fingerprint"] = serde_json::json!("deadbeef_wrong_fingerprint");
    fs::write(&manifest, serde_json::to_string_pretty(&mf).unwrap()).unwrap();

    let report2 = dir.path().join("report2.json");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--confirm")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("fingerprint"));
}

#[test]
fn migrate_governed__force_overwrite__recreates_dest() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "force overwrite pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .assert()
        .success();

    let dest_count_1 = event_count(&dest);
    assert!(dest_count_1 > 0);

    // Remove manifest so normal re-apply would refuse; force-overwrite recreates.
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    fs::remove_file(&manifest).ok();

    let report2 = dir.path().join("report2.json");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--confirm")
        .arg("--force-overwrite")
        .assert()
        .success();

    assert!(manifest.exists(), "force-overwrite must rewrite manifest");
    assert!(event_count(&dest) > 0);
}

#[test]
fn migrate_governed__default_scope_flag__forwarded() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    // Pin without project_id → missing_scope unless --default-scope provided.
    seed_memory_pinned(&source, "default scope pin body", false);

    // Without default-scope: missing_scope gap.
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .success();
    let v1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    let gaps_without = v1["gaps"]["missing_scope"].as_u64().unwrap_or(0);
    assert!(
        gaps_without >= 1,
        "expected missing_scope without --default-scope, got {gaps_without}"
    );

    // With default-scope: evidence path should open.
    let report2 = dir.path().join("report2.json");
    let scope = format!("Repository:{PROJECT_ID}");
    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report2)
        .arg("--default-scope")
        .arg(&scope)
        .assert()
        .success();
    let v2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report2).unwrap()).unwrap();
    assert!(
        v2["classification"]["evidence"].as_u64().unwrap_or(0) >= 1,
        "default_scope should allow evidence classification"
    );
    assert_eq!(v2["gaps"]["missing_scope"].as_u64().unwrap_or(0), 0);
}

#[test]
fn migrate_governed__invalid_default_scope__exit_6() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--default-scope")
        .arg("NotAValidScope")
        .assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("INVALID_PAYLOAD"));
}

/// Create a dangling file symlink (target does not exist).
/// Returns `None` when the OS denies symlink creation (Windows without Developer Mode).
fn try_dangling_file_symlink(link: &Path) -> Option<()> {
    let missing_target = link.with_extension("missing-target-does-not-exist");
    #[cfg(windows)]
    let created = std::os::windows::fs::symlink_file(&missing_target, link);
    #[cfg(not(windows))]
    let created = std::os::unix::fs::symlink(&missing_target, link);
    match created {
        Ok(()) => {
            assert!(
                !link.exists(),
                "precondition: dangling symlink must have exists()==false"
            );
            Some(())
        }
        Err(e) => {
            eprintln!(
                "skipping dangling-symlink integration test (symlink create failed: {e}; \
                 needs Developer Mode or elevation on Windows)"
            );
            None
        }
    }
}

/// Codex R5 — dangling symlink as destination must PATH_REFUSED (exists() gate bug).
#[test]
fn migrate_governed__refuse_dangling_symlink_dest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let report = dir.path().join("report.json");
    let dest = dir.path().join("dest-dangling.db");
    init_vault(&source);
    if try_dangling_file_symlink(&dest).is_none() {
        return;
    }

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );
}

/// Codex R5 — dangling symlink as report path must PATH_REFUSED.
#[test]
fn migrate_governed__refuse_dangling_symlink_report() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    let report = dir.path().join("report-dangling.json");
    init_vault(&source);
    if try_dangling_file_symlink(&report).is_none() {
        return;
    }

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );
}

/// Codex R5 — dangling symlink at migrate-manifest sibling must PATH_REFUSED.
#[test]
fn migrate_governed__refuse_dangling_symlink_manifest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest_dir = dir.path().join("dest");
    let dest = dest_dir.join("dest.db");
    let report = dir.path().join("report.json");
    init_vault(&source);
    fs::create_dir_all(&dest_dir).expect("dest dir");
    let manifest = dest_dir.join("migrate-manifest.json");
    if try_dangling_file_symlink(&manifest).is_none() {
        return;
    }

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );
}

/// Destination parent that is a reparse/junction (or dest symlink) must refuse (R1-04).
///
/// Windows: directory junction (no SeCreateSymbolicLinkPrivilege).
/// Non-Windows: file symlink as destination path.
#[test]
fn migrate_governed__refuse_reparse_dest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let report = dir.path().join("report.json");
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
            junction.join("dest.db")
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

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );
}

/// Report path under a reparse/junction parent must refuse (R1-04).
#[test]
fn migrate_governed__refuse_report_reparse() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    init_vault(&source);

    let report = {
        #[cfg(windows)]
        {
            let real = dir.path().join("real-report-dir");
            fs::create_dir_all(&real).expect("real report dir");
            let junction = dir.path().join("report-junction");
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
            junction.join("report.json")
        }
        #[cfg(not(windows))]
        {
            let real_file = dir.path().join("real-report.json");
            fs::write(&real_file, b"{}").expect("write real report");
            let link = dir.path().join("report-link.json");
            std::os::unix::fs::symlink(&real_file, &link).expect("unix symlink");
            link
        }
    };

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("reparse")
                .or(predicate::str::contains("symlink"))
                .or(predicate::str::contains("junction"))
                .or(predicate::str::contains("PATH_REFUSED"))
                .or(predicate::str::contains("report")),
        );
}

/// `--no-copy-events` on fresh dest: import applied without envelope copy of source events (R1-09).
#[test]
fn migrate_governed__no_copy_events__import_only_dest() {
    use ai_brains_store::event_store::EventStore;
    use std::collections::HashSet;

    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "no-copy pin body", true);
    let source_count = event_count(&source);
    assert!(source_count >= 1, "source must have at least one event");

    let source_ids: HashSet<_> = {
        let store = open_store(&source).expect("open source");
        store
            .read_all_events()
            .expect("read source")
            .into_iter()
            .map(|e| e.event_id)
            .collect()
    };

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .arg("--confirm")
        .arg("--no-copy-events")
        .assert()
        .success();

    assert!(dest.exists(), "confirm must create dest even without copy");
    let manifest = dest.parent().unwrap().join("migrate-manifest.json");
    assert!(manifest.exists(), "confirm must write migrate-manifest");

    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["legacy_import_applied"], true);
    assert_eq!(
        v["event_counts"]["would_copy_events"].as_u64().unwrap_or(1),
        0
    );
    assert!(
        v["classification"]["evidence"].as_u64().unwrap_or(0) >= 1,
        "import classification should still classify evidence"
    );

    let dest_ids: HashSet<_> = {
        let store = open_store(&dest).expect("open dest");
        store
            .read_all_events()
            .expect("read dest")
            .into_iter()
            .map(|e| e.event_id)
            .collect()
    };
    let overlap: Vec<_> = source_ids.intersection(&dest_ids).collect();
    assert!(
        overlap.is_empty(),
        "--no-copy-events must not append source envelope event_ids into dest; overlap={overlap:?}"
    );

    let dest_count = event_count(&dest);
    assert!(
        dest_count > 0,
        "import appends must still land on dest; dest_count={dest_count}"
    );
    // Dest holds import/audit appends only — not source envelope baseline + imports.
    // Envelope copy would place every source event_id into dest (overlap checked above).
    let _ = source_count;
}

/// Report path hardlinked to the source vault must refuse (Codex R2 P1-02).
///
/// `File::create` on a hardlinked report path would truncate the source vault.
/// Hardlink creation works without elevation on Windows NTFS and on Unix.
#[test]
fn migrate_governed__refuse_hardlink_report_to_source() {
    let dir = tempdir().expect("tempdir");
    let (source, dest, report) = paths(dir.path());
    init_vault(&source);
    seed_memory_pinned(&source, "hardlink report pin", true);

    // Create report path as hardlink of source before dry-run.
    fs::hard_link(&source, &report).expect("hard_link report -> source");

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("hardlink")
                .or(predicate::str::contains("link count"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );

    // Source must remain intact (refuse before write).
    let source_len = fs::metadata(&source).expect("source meta").len();
    assert!(
        source_len > 0,
        "source vault must not be truncated by refused hardlink report write"
    );
    assert!(
        event_count(&source) >= 1,
        "source events must remain readable after hardlink refuse"
    );
}

/// Destination hardlinked to source must refuse (Codex R3).
///
/// Path-string equality does not catch hardlinks; confirm would open dest R/W
/// and mutate the shared source inode (M5/M6).
#[test]
fn migrate_governed__refuse_hardlink_dest_to_source() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    let report = dir.path().join("report.json");
    init_vault(&source);
    seed_memory_pinned(&source, "hardlink dest pin", true);

    fs::hard_link(&source, &dest).expect("hard_link dest -> source");
    let source_len_before = fs::metadata(&source).expect("source meta").len();
    let source_events_before = event_count(&source);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("hardlink")
                .or(predicate::str::contains("link count"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );

    assert_eq!(
        fs::metadata(&source).expect("source meta").len(),
        source_len_before,
        "source vault must not be mutated via hardlinked dest"
    );
    assert_eq!(
        event_count(&source),
        source_events_before,
        "source events must remain intact after hardlink dest refuse"
    );
}

/// Source named `migrate-manifest.json` next to dest → manifest path collides (Codex R3).
#[test]
fn migrate_governed__refuse_source_named_migrate_manifest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("migrate-manifest.json");
    let dest = dir.path().join("vault.db");
    let report = dir.path().join("report.json");
    init_vault(&source);
    seed_memory_pinned(&source, "manifest name collision pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("migrate-manifest")
                .and(predicate::str::contains("source"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );

    // Source must still be readable (would have been overwritten by File::create).
    assert!(
        event_count(&source) >= 1,
        "source must not be overwritten by migrate-manifest write"
    );
}

/// Destination named `migrate-manifest.json` → manifest path collides with dest (Codex R3).
#[test]
fn migrate_governed__refuse_dest_named_migrate_manifest() {
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("source.db");
    let dest = dir.path().join("migrate-manifest.json");
    let report = dir.path().join("report.json");
    init_vault(&source);
    seed_memory_pinned(&source, "dest manifest name pin", true);

    migrate_cmd()
        .arg("migrate")
        .arg("governed")
        .arg("--source")
        .arg(&source)
        .arg("--destination")
        .arg(&dest)
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("migrate-manifest")
                .and(predicate::str::contains("destination"))
                .or(predicate::str::contains("PATH_REFUSED")),
        );
}
