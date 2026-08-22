//! T277 — Recoverable encrypted backup under the current key (hermetic mixed fleet).
//!
//! Other-key helper is file-local (F44). Live live-vault create is AC7/owner-confirm.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_contracts::doctor::{CheckSeverity, DoctorReport};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_store::pragmas::apply_key_pragmas;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const OTHER_KEY: &str = "x'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn ensure_backups_dir(vault_path: &Path) -> PathBuf {
    let backup_dir = vault_path
        .parent()
        .expect("vault has parent")
        .join("backups");
    fs::create_dir_all(&backup_dir).expect("create backups dir");
    backup_dir
}

/// F44: keyed SQLCipher junk, size ≥512, not plain. Distinct from T209 random bytes.
fn write_other_key_bak(backup_dir: &Path, name: &str) {
    let path = backup_dir.join(name);
    let key = SqlCipherKey::from_raw(OTHER_KEY.to_string());
    let conn = rusqlite::Connection::open(&path).expect("open other-key bak");
    apply_key_pragmas(&conn, &key).expect("other key");
    conn.execute_batch("CREATE TABLE junk(x); INSERT INTO junk VALUES (1);")
        .expect("junk");
    drop(conn);
    let len = fs::metadata(&path).expect("meta").len();
    assert!(
        len >= 512,
        "other-key bak must be ≥512 for KeyMismatch; got {len}"
    );
}

fn combined(out: &std::process::Output) -> String {
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    s
}

fn create_no_prune(vault: &Path) -> PathBuf {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("backup")
        .arg("create")
        .arg("--no-prune")
        .output()
        .expect("backup create --no-prune");
    assert!(
        out.status.success(),
        "create --no-prune must succeed; out={}",
        combined(&out)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Backup created and verified:"),
        "AC14 substring; stdout={stdout}"
    );
    let path = stdout
        .lines()
        .find_map(|l| l.split("Backup created and verified: ").nth(1))
        .expect("backup path printed");
    PathBuf::from(path.trim())
}

fn mixed_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    let residual_name = "vault-2020-01-01T00-00-00.db.bak";
    write_other_key_bak(&backup_dir, residual_name);
    let created = create_no_prune(&vault);
    (dir, vault, created, backup_dir.join(residual_name))
}

#[test]
fn backup_create__key_mismatch_residual__new_readable_and_doctor_ok() {
    let (_dir, vault, created, residual) = mixed_fixture();
    let created_name = created
        .file_name()
        .expect("created name")
        .to_string_lossy()
        .into_owned();
    assert!(created.exists(), "new backup file must exist");
    assert!(
        residual.exists(),
        "F28: --no-prune keeps KeyMismatch residual"
    );

    let list = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list");
    assert_eq!(
        list.status.code(),
        Some(0),
        "list exit 0; out={}",
        combined(&list)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    let stderr = String::from_utf8_lossy(&list.stderr);
    let rows: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("vault-") && l.contains(".db.bak"))
        .collect();
    assert!(
        rows.len() >= 2,
        "mixed fleet must list new + residual; stdout={stdout}"
    );
    assert!(
        rows[0].contains(&created_name),
        "AC2: first filename is the new file; first={} created={created_name}",
        rows[0]
    );
    assert!(
        !rows[0].contains("(unreadable key)")
            && !rows[0].contains("(legacy plain)")
            && !rows[0].contains("(no core tables)")
            && !rows[0].contains("(corrupt)"),
        "AC2: new row must be Readable (meta populated, no residual token); line={}",
        rows[0]
    );
    let residual_body = rows[1..].join("\n");
    assert!(
        residual_body.contains("(unreadable key)"),
        "AC2: residual stays (unreadable key); rows={rows:?}"
    );
    assert!(
        stderr.contains("not recoverable under current key"),
        "AC13: default list residual summary; stderr={stderr}"
    );

    let doctor = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("doctor")
        .arg("--json")
        .arg("--backup-max-age")
        .arg("7d")
        .output()
        .expect("doctor json");
    assert!(
        doctor.status.success(),
        "doctor must exit 0; out={}",
        combined(&doctor)
    );
    let report: DoctorReport = serde_json::from_slice(&doctor.stdout).expect("DoctorReport JSON");
    let br = report
        .checks
        .iter()
        .find(|c| c.name == "backup_recent")
        .expect("backup_recent present");
    assert_eq!(
        br.severity,
        CheckSeverity::Ok,
        "AC3: mixed create must make backup_recent ok; msg={:?}",
        br.message
    );
    let msg = br.message.as_deref().unwrap_or("");
    assert!(
        !msg.contains("no usable encrypted backup under current key"),
        "AC3: must not be the zero-usable warn; msg={msg}"
    );
}

#[test]
fn backup_verify__mixed_ok_and_key_mismatch__one_ok_exit_1_no_nudge() {
    let (_dir, vault, _created, residual) = mixed_fixture();
    assert!(residual.exists());

    let verify = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("verify")
        .output()
        .expect("backup verify");
    assert_eq!(
        verify.status.code(),
        Some(1),
        "AC4: any FAIL → exit 1; out={}",
        combined(&verify)
    );
    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(
        stdout.contains("1 OK") && stdout.contains("1 FAIL"),
        "AC4: 1 OK, 1 FAIL; stdout={stdout}"
    );
    assert!(
        !stdout.contains("0 OK"),
        "AC4: must not report 0 OK; stdout={stdout}"
    );
    assert!(
        !stdout.contains("No usable encrypted backup under current key"),
        "AC4/F41: no create nudge when ok>=1; stdout={stdout}"
    );
}

#[test]
fn backup_list__mixed_after_create__residual_summary_not_recoverable() {
    let (_dir, vault, _created, _residual) = mixed_fixture();
    let list = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("list")
        .output()
        .expect("backup list");
    assert_eq!(list.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&list.stderr);
    assert!(
        stderr.contains("not recoverable under current key"),
        "AC13: residual summary; stderr={stderr}"
    );
}
