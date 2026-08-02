#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T181 CLI recovery drills: R-01, R-03, F-01, F-02 (+ secret non-leak on CLI surfaces).

mod common;

use ai_brains_crypto::test_support::assert_no_secret_leakage;
use predicates::prelude::*;
use std::fs::{self, OpenOptions};
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const ZERO_KEY: &str = "x'0000000000000000000000000000000000000000000000000000000000000000'";
const ALT_KEY: &str = "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'";
const SEED_CONTENT: &str = "T181-R-01 seeded recovery content unique-token-a1b2c3";
/// Default CLI zero key material (32 zero bytes).
const ZERO_KEY_BYTES: [u8; 32] = [0u8; 32];
/// ALT_KEY material (32 0xff bytes).
const ALT_KEY_BYTES: [u8; 32] = [0xffu8; 32];

fn hermetic_with_key(vault_path: &Path, key: &str) -> assert_cmd::Command {
    // T187: --no-project-context + explicit key env (after ambient strip) so
    // repo `.env` cannot inject AI_BRAINS_KEY. Prefer env over --key argv to
    // avoid Windows quoting quirks around x'…' material.
    let mut cmd = common::hermetic_bin();
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .env("AI_BRAINS_KEY", key)
        .arg("--key")
        .arg(key);
    cmd
}

fn init_vault(vault_path: &Path) {
    hermetic_with_key(vault_path, ZERO_KEY)
        .arg("init")
        .assert()
        .success();
}

fn init_vault_with_key(vault_path: &Path, key: &str) {
    hermetic_with_key(vault_path, key)
        .arg("init")
        .assert()
        .success();
}

fn create_backup(vault_path: &Path) -> PathBuf {
    create_backup_with_key(vault_path, ZERO_KEY)
}

fn create_backup_with_key(vault_path: &Path, key: &str) -> PathBuf {
    let output = hermetic_with_key(vault_path, key)
        .arg("backup")
        .output()
        .expect("backup must run");
    assert!(
        output.status.success(),
        "backup create failed; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = combined_output(&output);
    assert_no_secret_leakage(&combined, &ZERO_KEY_BYTES);
    if key == ALT_KEY {
        assert_no_secret_leakage(&combined, &ALT_KEY_BYTES);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let backup_path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path must be printed")
        .trim();
    PathBuf::from(backup_path)
}

fn corrupt_at(path: &Path, offset: u64) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    file.write_all(b"CORRUPTION!!").unwrap();
    file.sync_all().unwrap();
}

fn combined_output(output: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    s.push_str(&String::from_utf8_lossy(&output.stderr));
    s
}

/// F46 corruption class — normative tokens only (no bare "fail"/"error").
fn matches_corrupt_class(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("integrity")
        || lower.contains("corrupt")
        || lower.contains("not a database")
        || lower.contains("query failed")
}

/// F46 wrong-key class — normative / empirical open failures (no bare "fail"/"error").
fn matches_wrong_key_class(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("not a database")
        || lower.contains("key verification")
        || lower.contains("vaultlocked")
        || lower.contains("vault locked")
        || lower.contains("file is not a database")
        || lower.contains("unable to open")
}

fn meta_present_in_backup(backup_path: &Path) -> bool {
    let conn = rusqlite::Connection::open(backup_path).unwrap();
    ai_brains_store::pragmas::apply_key_pragmas(
        &conn,
        &ai_brains_crypto::SqlCipherKey::from_raw(ZERO_KEY.to_string()),
    )
    .unwrap();
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_aibrains_backup_meta'",
        [],
        |_| Ok(true),
    )
    .unwrap_or(false)
}

fn meta_present_in_vault(vault_path: &Path) -> bool {
    meta_present_in_backup(vault_path)
}

/// T181-R-01: seed → backup → force restore → content smoke + meta F20 asserts.
#[test]
fn backup_restore__seeded_content__present_after_force_restore() {
    let dir = tempdir().unwrap();
    let source_vault = dir.path().join("source.db");
    let dest_vault = dir.path().join("dest.db");

    init_vault(&source_vault);

    let turn_json = format!(
        r#"{{
            "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "project_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
            "harness_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
            "turn_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
            "privacy": "LocalOnly",
            "role": "user",
            "content": "{SEED_CONTENT}"
        }}"#
    );
    hermetic_with_key(&source_vault, ZERO_KEY)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let backup_path = create_backup(&source_vault);
    assert!(
        meta_present_in_backup(&backup_path),
        "T181-R-01 F20: meta must be present in backup file"
    );

    init_vault(&dest_vault);
    // Different content on dest so we prove overwrite brought source content.
    hermetic_with_key(&dest_vault, ZERO_KEY)
        .env(
            "AI_BRAINS_PROJECT_ID",
            "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        )
        .env(
            "AI_BRAINS_SESSION_ID",
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        )
        .arg("pin")
        .arg("dest-only content that must be replaced by restore")
        .assert()
        .success();

    let restore_out = hermetic_with_key(&dest_vault, ZERO_KEY)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--force")
        .output()
        .expect("restore");
    assert!(
        restore_out.status.success(),
        "force restore must succeed; out={}",
        combined_output(&restore_out)
    );
    let restore_combined = combined_output(&restore_out);
    assert!(
        restore_combined.contains("Vault restored from"),
        "restore confirmation missing: {restore_combined}"
    );
    assert_no_secret_leakage(&restore_combined, &ZERO_KEY_BYTES);

    assert!(
        !meta_present_in_vault(&dest_vault),
        "T181-R-01 F20: meta must be ABSENT on live vault post-restore"
    );

    // Content smoke via recall.
    let recall = hermetic_with_key(&dest_vault, ZERO_KEY)
        .env(
            "AI_BRAINS_PROJECT_ID",
            "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        )
        .arg("recall")
        .arg("unique-token-a1b2c3")
        .arg("--format")
        .arg("json")
        .arg("--global")
        .output()
        .expect("recall");
    assert!(
        recall.status.success(),
        "recall must succeed; stderr={}",
        String::from_utf8_lossy(&recall.stderr)
    );
    let recall_stdout = String::from_utf8_lossy(&recall.stdout);
    assert!(
        recall_stdout.contains(SEED_CONTENT) || recall_stdout.contains("unique-token-a1b2c3"),
        "restored vault must return seeded content; got: {recall_stdout}"
    );
}

/// T188: daemon-down force restore succeeds (integration; live probe offline).
#[test]
fn backup_restore__daemon_down_force__succeeds() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_path = create_backup(&vault);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--force")
        .output()
        .expect("restore force");
    assert!(
        out.status.success(),
        "daemon-down force restore must succeed; out={}",
        combined_output(&out)
    );
    let msg = combined_output(&out);
    assert!(
        msg.contains("Vault restored from"),
        "restore confirmation missing: {msg}"
    );
    assert_no_secret_leakage(&msg, &ZERO_KEY_BYTES);
}

/// T188 recovery export via passphrase-file: unlockable kit, schema_version=1.
#[test]
fn recovery_export__passphrase_file__writes_unlockable_kit() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("recovery-kit.json");
    let pw_path = dir.path().join("passphrase.txt");
    let passphrase = b"integration-passphrase-ok";
    fs::write(&pw_path, passphrase).unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("recovery export");
    assert!(
        out.status.success(),
        "export must succeed; out={}",
        combined_output(&out)
    );
    let combined = combined_output(&out);
    assert!(
        combined.contains("dpapi: present") || combined.contains("dpapi: absent"),
        "must print dpapi status; got: {combined}"
    );
    assert!(kit_path.exists(), "kit file must exist");

    let json = fs::read_to_string(&kit_path).unwrap();
    let kit = ai_brains_crypto::RecoveryKit::from_json(&json).expect("parse kit");
    assert_eq!(kit.schema_version, 1);
    let unlocked = kit
        .unlock_with_passphrase(passphrase)
        .expect("unlock with same passphrase");
    assert_eq!(unlocked.expose_secret(), &ZERO_KEY_BYTES);

    // Stdout/stderr must not dump kit or secrets.
    assert_no_secret_leakage(&combined, &ZERO_KEY_BYTES);
    assert_no_secret_leakage(&combined, passphrase);
    ai_brains_crypto::test_support::assert_no_kit_dump(&combined, &json);
}

/// T188: export stdout has no kit JSON / secrets.
#[test]
fn recovery_export__stdout__no_kit_json_or_secrets() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    let passphrase = b"no-leak-passphrase!!";
    fs::write(&pw_path, passphrase).unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("export");
    assert!(out.status.success(), "out={}", combined_output(&out));
    let combined = combined_output(&out);
    let json = fs::read_to_string(&kit_path).unwrap();
    let parsed = ai_brains_crypto::RecoveryKit::from_json(&json).expect("parse kit");
    // Ciphertext bytes present in kit file but must not appear in operator output.
    let ct_bytes = parsed.passphrase.ciphertext.as_slice();
    assert_no_secret_leakage(&combined, ct_bytes);
    assert_no_secret_leakage(&combined, &ZERO_KEY_BYTES);
    assert_no_secret_leakage(&combined, passphrase);
    ai_brains_crypto::test_support::assert_no_kit_dump(&combined, &json);
    // Coarse markers: full kit JSON body and structural field dumps.
    assert!(!combined.contains(&json));
    assert!(
        !combined.contains("\"ciphertext\""),
        "operator output must not dump ciphertext JSON fields"
    );
}

/// T188: output exists refuses without --force.
#[test]
fn recovery_export__output_exists__refuses_without_force() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    fs::write(&kit_path, b"already-here").unwrap();
    let pw_path = dir.path().join("pw.txt");
    fs::write(&pw_path, b"force-needed-passphrase").unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("export exists");
    assert!(!out.status.success(), "must refuse when output exists");
    let msg = combined_output(&out).to_ascii_lowercase();
    assert!(
        msg.contains("exists") || msg.contains("output exists"),
        "must match exists class; got: {msg}"
    );
    // File content unchanged.
    assert_eq!(fs::read(&kit_path).unwrap(), b"already-here");
}

/// T188: dry-run does not write kit file.
#[test]
fn recovery_export__dry_run__no_file() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    fs::write(&pw_path, b"dry-run-passphrase-ok").unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .arg("--dry-run")
        .output()
        .expect("dry-run export");
    assert!(
        out.status.success(),
        "dry-run must succeed; out={}",
        combined_output(&out)
    );
    assert!(!kit_path.exists(), "dry-run must not write kit file");
    let msg = combined_output(&out).to_ascii_lowercase();
    assert!(
        msg.contains("dry-run") || msg.contains("would write"),
        "dry-run notice missing: {msg}"
    );
}

/// T188: short passphrase fails with passphrase/too short class.
#[test]
fn recovery_export__short_passphrase__fails() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let kit_path = dir.path().join("kit.json");
    let pw_path = dir.path().join("pw.txt");
    fs::write(&pw_path, b"short").unwrap();

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("recovery")
        .arg("export")
        .arg("--output")
        .arg(&kit_path)
        .arg("--passphrase-file")
        .arg(&pw_path)
        .output()
        .expect("short passphrase");
    assert!(!out.status.success());
    let msg = combined_output(&out).to_ascii_lowercase();
    assert!(
        msg.contains("passphrase") && (msg.contains("short") || msg.contains("minimum")),
        "must match passphrase too short class; got: {msg}"
    );
    assert!(!kit_path.exists());
}

/// T181-R-03: missing backup path → non-zero + not-found class.
#[test]
fn backup_restore__missing_path__not_found_class() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let missing = dir.path().join("does-not-exist.db.bak");
    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .arg("restore")
        .arg(&missing)
        .arg("--force")
        .output()
        .expect("restore missing");
    assert!(
        !out.status.success(),
        "missing backup path must fail non-zero"
    );
    let msg = combined_output(&out).to_ascii_lowercase();
    assert!(
        msg.contains("not found") || msg.contains("backup file not found"),
        "R-03 must match not-found class; got: {msg}"
    );
}

/// T181-F-01 (header): corrupt offset 0 → fail + corruption class.
#[test]
fn backup_verify__corrupt_header__corruption_class() {
    corrupt_backup_case(0);
}

/// T181-F-01 (body): corrupt offset ≥100 → fail + corruption class.
#[test]
fn backup_verify__corrupt_body__corruption_class() {
    corrupt_backup_case(100);
}

fn corrupt_backup_case(offset: u64) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_path = create_backup(&vault);
    corrupt_at(&backup_path, offset);

    let out = hermetic_with_key(&vault, ZERO_KEY)
        .arg("backup")
        .arg("verify")
        .arg(&backup_path)
        .output()
        .expect("verify corrupt");
    assert!(
        !out.status.success(),
        "corrupt backup verify must fail; offset={offset}"
    );
    let msg = combined_output(&out);
    assert!(
        matches_corrupt_class(&msg),
        "F-01 must match corruption class at offset {offset}; got: {msg}"
    );
}

/// T181-F-02 / T187-H-02: wrong SQLCipher key on verify → non-zero + wrong-key class.
///
/// T187: live SQLCipher — strict fail-closed (no plain dual-mode residual).
/// Uses two **non-zero** keys so the proof does not depend on zero-key escape hatch.
#[test]
fn backup_verify__wrong_key__wrong_key_class() {
    let dir = tempdir().unwrap();
    // Separate parent dirs so second-resolution backup filenames cannot collide
    // (both would otherwise write dir/backups/vault-<same-second>.db.bak).
    let a_dir = dir.path().join("a");
    let b_dir = dir.path().join("b");
    fs::create_dir_all(&a_dir).unwrap();
    fs::create_dir_all(&b_dir).unwrap();
    let a_vault = a_dir.join("vault.db");
    let b_vault = b_dir.join("vault.db");
    // Distinct non-zero product keys (32 bytes each).
    const KEY_A: &str = "x'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'";
    const KEY_B: &str = "x'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'";
    const KEY_A_BYTES: [u8; 32] = [0xaa; 32];
    const KEY_B_BYTES: [u8; 32] = [0xbb; 32];

    init_vault_with_key(&a_vault, KEY_A);
    let a_backup = create_backup_with_key(&a_vault, KEY_A);

    init_vault_with_key(&b_vault, KEY_B);
    let b_backup = create_backup_with_key(&b_vault, KEY_B);
    assert_ne!(
        a_backup, b_backup,
        "backup paths must not collide across vault parents"
    );

    // Positive control: same key verifies.
    hermetic_with_key(&b_vault, KEY_B)
        .arg("backup")
        .arg("verify")
        .arg(&b_backup)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));

    assert!(
        !is_plain_sqlite_header(&a_backup),
        "T187: backup must not have plain SQLite header under SQLCipher build"
    );

    // Cross-key: KEY_A backup verified under KEY_B context must fail closed.
    let out = hermetic_with_key(&b_vault, KEY_B)
        .arg("backup")
        .arg("verify")
        .arg(&a_backup)
        .output()
        .expect("verify wrong key");
    let msg = combined_output(&out);
    assert_no_secret_leakage(&msg, &KEY_A_BYTES);
    assert_no_secret_leakage(&msg, &KEY_B_BYTES);

    assert!(
        !out.status.success(),
        "T187-H-02 / F-02: wrong-key verify must fail non-zero; out={msg}"
    );
    assert!(
        matches_wrong_key_class(&msg),
        "F-02 must match wrong-key class; pinned message was: {msg}"
    );
}

fn is_plain_sqlite_header(path: &Path) -> bool {
    ai_brains_store::is_plain_sqlite_header(path)
}

/// Soft T181-F-03 documentation: daemon warn path is product-side; no hard-fail claim.
/// Covered by existing backup restore warn code; this test only asserts dry-run still works
/// without requiring a live daemon.
#[test]
fn backup_restore__dry_run__integrity_ok_no_mutation() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.db");
    let dest = dir.path().join("dest.db");
    init_vault(&source);
    init_vault(&dest);
    let size_before = fs::metadata(&dest).unwrap().len();
    let backup_path = create_backup(&source);

    hermetic_with_key(&dest, ZERO_KEY)
        .arg("backup")
        .arg("restore")
        .arg(&backup_path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));

    let size_after = fs::metadata(&dest).unwrap().len();
    assert_eq!(size_before, size_after, "dry-run must not mutate dest");
}
