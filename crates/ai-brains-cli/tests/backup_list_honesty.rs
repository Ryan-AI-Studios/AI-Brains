//! T209 — Backup list SQLCipher honesty (hermetic AC locks).
//!
//! AC1: plain-header bak + unset RUST_LOG → no per-file WARN + `(legacy plain)`
//! AC2: short garbage + RUST_LOG=warn → Corrupt WARN
//! AC3: ≥2 plain → ≤1 eprintln summary
//! AC4: --verbose per-file detail
//! AC5: --quiet / dual quiet+verbose quiet wins
//! AC7: readable path-end token check
//! AC9: large wrong-key → `(unreadable key)` summary, no per-file WARN flood

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

const SQLITE_MAGIC: &[u8] = b"SQLite format 3\0";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn ensure_backups_dir(vault_path: &Path) -> std::path::PathBuf {
    let backup_dir = vault_path
        .parent()
        .expect("vault has parent")
        .join("backups");
    fs::create_dir_all(&backup_dir).expect("create backups dir");
    backup_dir
}

/// F33: valid plain SQLite magic + padding (header sniffs plain).
fn write_plain_bak(backup_dir: &Path, name: &str) {
    let path = backup_dir.join(name);
    let mut f = fs::File::create(&path).expect("create plain bak");
    f.write_all(SQLITE_MAGIC).expect("magic");
    f.write_all(&[0u8; 100]).expect("pad");
}

fn write_short_garbage(backup_dir: &Path, name: &str) {
    let path = backup_dir.join(name);
    fs::write(&path, b"not a valid sqlite database").expect("write garbage");
}

fn write_large_non_plain(backup_dir: &Path, name: &str) {
    let path = backup_dir.join(name);
    fs::write(&path, vec![0xABu8; 600]).expect("write large garbage");
}

fn list_output(vault: &Path, args: &[&str], rust_log: Option<&str>) -> std::process::Output {
    let mut cmd = common::hermetic_bin();
    // hermetic_bin already strips RUST_LOG (F16). Re-set only when requested.
    if let Some(v) = rust_log {
        cmd.env("RUST_LOG", v);
    }
    cmd.arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault)
        .arg("backup")
        .arg("list");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("backup list must run")
}

// ---------------------------------------------------------------------------
// AC1 — plain + unset RUST_LOG → (legacy plain), no per-file key/metadata WARN
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__plain_unset_rust_log__legacy_plain_no_per_file_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    // Do NOT set RUST_LOG — denylist already removed it (product default filter).
    let out = list_output(&vault, &[], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "list must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("(legacy plain)"),
        "AC1: table must show (legacy plain); stdout={stdout}"
    );

    // Summary via eprintln is OK (stderr). Per-file key/metadata WARN is not.
    let stderr_lower = stderr.to_ascii_lowercase();
    let has_per_file_key_warn = stderr_lower.contains("warn")
        && (stderr_lower.contains("key verification")
            || stderr_lower.contains("backup key")
            || stderr_lower.contains("could not read backup metadata")
            || stderr_lower.contains("legacy plaintext")
            || stderr_lower.contains("not readable with current key"));
    assert!(
        !has_per_file_key_warn,
        "AC1: default must not emit per-file key/metadata WARN; stderr={stderr}"
    );
    // Summary expected for ≥1 residual.
    assert!(
        stderr.contains("not fully readable"),
        "AC1: default summary expected; stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — short garbage + RUST_LOG=warn → Corrupt WARN
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__short_garbage_rust_log_warn__corrupt_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_short_garbage(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &[], Some("warn"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "list must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}").to_ascii_lowercase();

    assert!(
        combined.contains("corrupt or unreadable")
            || combined.contains("file is not a database")
            || (combined.contains("warn") && combined.contains("corrupt")),
        "AC2: short garbage must emit Corrupt WARN; combined={combined}"
    );
    assert!(
        stdout.contains("(corrupt)"),
        "AC2: table token (corrupt); stdout={stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC3 — two plain files → ≤1 summary line
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__two_plain__at_most_one_summary() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");
    write_plain_bak(&backup_dir, "vault-2026-01-02T00-00-00.db.bak");

    let out = list_output(&vault, &[], None);
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let summary_lines: Vec<_> = stderr
        .lines()
        .filter(|l| l.contains("not fully readable"))
        .collect();
    assert!(
        summary_lines.len() <= 1,
        "AC3: ≤1 summary line; got {} in stderr={stderr}",
        summary_lines.len()
    );
    assert_eq!(
        summary_lines.len(),
        1,
        "AC3: exactly one summary for 2 plain; stderr={stderr}"
    );
    assert!(
        summary_lines[0].contains("2 backup(s) not fully readable")
            || summary_lines[0].contains("not fully readable"),
        "AC3: summary content; line={}",
        summary_lines[0]
    );
}

// ---------------------------------------------------------------------------
// AC4 — --verbose → per-file detail for LegacyPlain
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__verbose_plain__per_file_detail() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &["--verbose"], Some("warn"));
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        stdout.contains("(legacy plain)"),
        "AC4: token still present; stdout={stdout}"
    );
    // Require the verbose WARN text (not filename alone — table always prints name).
    assert!(
        combined.to_ascii_lowercase().contains("legacy plaintext"),
        "AC4: verbose must emit per-file legacy detail; combined={combined}"
    );
    // Prefer omit summary under verbose (F7).
    let summary_count = stderr
        .lines()
        .filter(|l| l.contains("not fully readable"))
        .count();
    assert_eq!(
        summary_count, 0,
        "AC4: verbose omits summary; stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC5 — --quiet no summary; dual --quiet --verbose quiet wins
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__quiet__no_summary() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &["--quiet"], None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("(legacy plain)"),
        "AC5: quiet still shows table tokens; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not fully readable"),
        "AC5: quiet must not print summary; stderr={stderr}"
    );
}

#[test]
fn backup_list_honesty__quiet_and_verbose__quiet_wins() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &["--quiet", "--verbose"], Some("warn"));
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("(legacy plain)"),
        "dual flags still show tokens; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not fully readable"),
        "AC5: quiet wins — no summary; stderr={stderr}"
    );
    // Quiet: no per-file legacy WARN either.
    let has_legacy_warn = stderr.to_ascii_lowercase().contains("legacy plaintext")
        && stderr.to_ascii_lowercase().contains("warn");
    assert!(
        !has_legacy_warn,
        "AC5: quiet wins — no per-file WARN; stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC7 — real backup list shows path-end (readable)
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__readable_backup__path_end() {
    let dir = tempdir().unwrap();
    let subdir = dir
        .path()
        .join("very-long-directory-name-that-makes-path-exceed-forty");
    fs::create_dir_all(&subdir).unwrap();
    let vault = subdir.join("vault.db");
    init_vault(&vault);

    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .assert()
        .success();

    let out = list_output(&vault, &[], None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("vault.db"),
        "AC7: Source Vault path-end; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)"),
        "readable backup must not use legacy token; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(corrupt)"),
        "readable backup must not use corrupt token; stdout={stdout}"
    );
}

// ---------------------------------------------------------------------------
// AC9 — large non-plain garbage → (unreadable key), summary, no per-file WARN
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__large_key_mismatch__summary_not_warn_flood() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_large_non_plain(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &[], Some("warn"));
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("(unreadable key)"),
        "AC9: table token (unreadable key); stdout={stdout}"
    );
    assert!(
        stderr.contains("not fully readable"),
        "AC9: summary expected; stderr={stderr}"
    );

    // Under Default, KeyMismatch is debug only — with RUST_LOG=warn no per-file WARN.
    let stderr_lower = stderr.to_ascii_lowercase();
    let per_file_key_warn = stderr_lower.contains("warn")
        && (stderr_lower.contains("not readable with current key")
            || stderr_lower.contains("key verification")
            || stderr_lower.contains("backup key verification")
            || stderr_lower.contains("key mismatch"));
    assert!(
        !per_file_key_warn,
        "AC9: no per-file KeyMismatch WARN under default; stderr={stderr}"
    );
}
