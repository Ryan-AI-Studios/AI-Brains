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

    // T318 F1/F4/F31: Default omits residual table tokens; residuals-only → No usable backups.
    assert!(
        !stdout.contains("(legacy plain)"),
        "T318: Default must not print residual table tokens; stdout={stdout}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318 AC3: all-residual Default prints No usable backups.; stdout={stdout}"
    );

    // Per-file key/metadata WARN is not allowed on Default.
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
    // T318 F2: residual summary on stdout (not stderr / ErrorRecord).
    assert!(
        stdout.contains("not recoverable under current key"),
        "T318 AC2: default summary on stdout; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318 AC2: summary must not be on stderr; stderr={stderr}"
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
        "AC2/F21: short garbage must emit Corrupt WARN; combined={combined}"
    );
    // T318 F21: Default table omits residual (corrupt) rows; WARN may still fire under RUST_LOG=warn.
    assert!(
        !stdout.contains("(corrupt)"),
        "T318 F21: Default must not print (corrupt) table token; stdout={stdout}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318: all-residual Default prints No usable backups.; stdout={stdout}"
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
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // T318 F2/F31: summary lines move to stdout.
    let summary_lines: Vec<_> = stdout
        .lines()
        .filter(|l| l.contains("not recoverable under current key"))
        .collect();
    assert!(
        summary_lines.len() <= 1,
        "AC3: ≤1 summary line; got {} in stdout={stdout}",
        summary_lines.len()
    );
    assert_eq!(
        summary_lines.len(),
        1,
        "AC3: exactly one summary for 2 plain; stdout={stdout}"
    );
    assert!(
        summary_lines[0].contains("2 backup(s) not recoverable under current key")
            || summary_lines[0].contains("not recoverable under current key"),
        "AC3: summary content; line={}",
        summary_lines[0]
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318: summary must not be on stderr; stderr={stderr}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318: all-residual Default; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)"),
        "T318: no residual table tokens on Default; stdout={stdout}"
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
    // Prefer omit summary under verbose (F7 / T318 F3).
    let combined_summary = format!("{stdout}{stderr}")
        .lines()
        .filter(|l| l.contains("not recoverable under current key"))
        .count();
    assert_eq!(
        combined_summary, 0,
        "AC4: verbose omits summary; stdout={stdout} stderr={stderr}"
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

    // T318 AC20: all-residual quiet → No usable backups.; no residual tokens; no footer.
    assert!(
        stdout.contains("No usable backups."),
        "AC20: quiet all-residual → No usable backups.; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)"),
        "AC20: quiet must not print residual tokens; stdout={stdout}"
    );
    assert!(
        !stdout.contains("not recoverable under current key")
            && !stderr.contains("not recoverable under current key"),
        "AC20: quiet must not print footer; stdout={stdout} stderr={stderr}"
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
        stdout.contains("No usable backups."),
        "AC20: quiet wins → No usable backups.; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)"),
        "AC20: quiet wins — no residual tokens; stdout={stdout}"
    );
    assert!(
        !stdout.contains("not recoverable under current key")
            && !stderr.contains("not recoverable under current key"),
        "AC20: quiet wins — no footer; stdout={stdout} stderr={stderr}"
    );
    // Quiet: no per-file legacy WARN either.
    let has_legacy_warn = stderr.to_ascii_lowercase().contains("legacy plaintext")
        && stderr.to_ascii_lowercase().contains("warn");
    assert!(
        !has_legacy_warn,
        "AC20: quiet wins — no per-file WARN; stderr={stderr}"
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

    // T318 F1/F31: Default omits residual tokens; footer on stdout.
    assert!(
        !stdout.contains("(unreadable key)"),
        "T318: Default must not print residual table tokens; stdout={stdout}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318: all-residual Default; stdout={stdout}"
    );
    assert!(
        stdout.contains("not recoverable under current key"),
        "T318 AC2: summary on stdout; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318 AC2: summary must not be on stderr; stderr={stderr}"
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

// ---------------------------------------------------------------------------
// T244 — Incomplete residual honesty + usable-first list order
// ---------------------------------------------------------------------------

fn write_incomplete_bak(backup_dir: &Path, name: &str) {
    // F13: SQLCipher open + junk table only (no events / memory_projection).
    let path = backup_dir.join(name);
    let key = ai_brains_crypto::SqlCipherKey::from_raw(common::ZERO_SQLCIPHER_KEY.to_string());
    let conn = rusqlite::Connection::open(&path).expect("open incomplete bak");
    ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).expect("key");
    conn.execute_batch("CREATE TABLE junk(x);")
        .expect("junk table");
}

fn write_single_core_bak(backup_dir: &Path, name: &str, only: &str) {
    let path = backup_dir.join(name);
    let key = ai_brains_crypto::SqlCipherKey::from_raw(common::ZERO_SQLCIPHER_KEY.to_string());
    let conn = rusqlite::Connection::open(&path).expect("open single-core bak");
    ai_brains_store::pragmas::apply_key_pragmas(&conn, &key).expect("key");
    conn.execute_batch(&format!("CREATE TABLE {only} (id INTEGER PRIMARY KEY);"))
        .expect("single core table");
}

#[test]
fn backup_list_honesty__incomplete__token_and_residual_summary() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_incomplete_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &[], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "list must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // T318 F1/F31: Default omits Incomplete token; footer on stdout still counts it.
    assert!(
        !stdout.contains("(no core tables)"),
        "T318: Default must not print Incomplete token; stdout={stdout}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318: all-residual Default; stdout={stdout}"
    );
    assert!(
        stdout.contains("not recoverable under current key"),
        "T318: Incomplete counts in residual summary on stdout; stdout={stdout}"
    );
    assert!(
        stdout.contains("1 backup(s) not recoverable under current key"),
        "T318 residual count includes Incomplete; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318: summary must not be on stderr; stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC17 — Incomplete noise: debug under Default, warn under Verbose (RUST_LOG=warn)
// ---------------------------------------------------------------------------

#[test]
fn backup_list_honesty__incomplete_default_rust_log_warn__no_per_file_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_incomplete_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    // Default list + RUST_LOG=warn: Incomplete is debug only (no per-file WARN flood).
    let out = list_output(&vault, &[], Some("warn"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "list must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // T318 F1/F31: Default omits Incomplete token; footer on stdout.
    assert!(
        !stdout.contains("(no core tables)"),
        "T318: Default must not print Incomplete token; stdout={stdout}"
    );
    assert!(
        stdout.contains("No usable backups."),
        "T318: all-residual Default; stdout={stdout}"
    );
    assert!(
        stdout.contains("not recoverable under current key"),
        "T318: residual summary on stdout under default; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318: summary must not be on stderr; stderr={stderr}"
    );

    // Per-file Incomplete WARN is Verbose-only; Default must not emit it at warn filter.
    let stderr_lower = stderr.to_ascii_lowercase();
    let per_file_incomplete_warn = stderr_lower.contains("missing core tables")
        || stderr_lower.contains("backup missing core tables")
        || stderr_lower.contains("not restorable");
    assert!(
        !per_file_incomplete_warn,
        "AC17: default must not emit per-file Incomplete WARN; stderr={stderr}"
    );
}

#[test]
fn backup_list_honesty__incomplete_verbose_rust_log_warn__per_file_warn() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_incomplete_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");

    let out = list_output(&vault, &["--verbose"], Some("warn"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "list must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        stdout.contains("(no core tables)"),
        "AC17: token still present under verbose; stdout={stdout}"
    );
    // emit_list_noise Incomplete Verbose: "Backup missing core tables ...; not restorable"
    let combined_lower = combined.to_ascii_lowercase();
    assert!(
        combined_lower.contains("missing core tables") || combined_lower.contains("not restorable"),
        "AC17: verbose must emit per-file Incomplete WARN; combined={combined}"
    );
}

#[test]
fn backup_list_honesty__mixed_usable_and_residual__usable_first() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Create a real Readable backup first.
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .assert()
        .success();

    let backup_dir = ensure_backups_dir(&vault);
    // Fresher-named Incomplete residual — chronological sort would put it first.
    write_incomplete_bak(&backup_dir, "vault-2099-12-31T23-59-59.db.bak");
    write_plain_bak(&backup_dir, "vault-2099-12-30T00-00-00.db.bak");

    let out = list_output(&vault, &[], None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("vault-") && l.contains(".db.bak"))
        .collect();
    // T318 AC1: Default table = usable only (no residual token rows).
    assert_eq!(
        lines.len(),
        1,
        "T318 AC1: Default prints usable row only; stdout={stdout}"
    );
    assert!(
        !lines[0].contains("(no core tables)")
            && !lines[0].contains("(legacy plain)")
            && !lines[0].contains("(unreadable key)")
            && !lines[0].contains("(corrupt)"),
        "T318 AC1: usable row must not carry residual tokens; line={}",
        lines[0]
    );
    assert!(
        !stdout.contains("(no core tables)")
            && !stdout.contains("(legacy plain)")
            && !stdout.contains("(unreadable key)")
            && !stdout.contains("(corrupt)"),
        "T318 AC1: no residual tokens anywhere in Default stdout; stdout={stdout}"
    );
    // T318 AC2: footer on stdout with residual count; absent on stderr.
    assert!(
        stdout.contains("2 backup(s) not recoverable under current key"),
        "T318 AC2: two residuals (incomplete + plain) on stdout; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "T318 AC2: footer must not be on stderr; stderr={stderr}"
    );
}

#[test]
fn backup_verify__incomplete_and_single_core__missing_core_tables() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_incomplete_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");
    write_single_core_bak(&backup_dir, "vault-2026-01-02T00-00-00.db.bak", "events");

    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .arg("verify")
        .arg("--format")
        .arg("json")
        .output()
        .expect("verify");
    assert_ne!(
        out.status.code(),
        Some(0),
        "verify must fail; out={}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("missing core tables"),
        "T244 AC8: fail reason; stdout={stdout}"
    );
    // JSON tables still populated from IN query (single-core → one name; zero-core → []).
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let results = v["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2, "two backup results");
    for r in results {
        assert!(
            r["tables"].is_array(),
            "tables field must remain an array; got {r}"
        );
        assert_eq!(
            r["status"].as_str().unwrap_or(""),
            "fail",
            "JSON status is lowercase fail; got {r}"
        );
        let err = r["error"].as_str().unwrap_or("");
        assert!(
            err.contains("missing core tables"),
            "error must mention missing core tables; got {err}"
        );
    }
    // Single-core shell should still list the one present table name in JSON.
    let single = results
        .iter()
        .find(|r| {
            r["tables"]
                .as_array()
                .map(|t| t.len() == 1)
                .unwrap_or(false)
        })
        .expect("one result with single table entry");
    assert_eq!(single["tables"][0], "events");
}

// ---------------------------------------------------------------------------
// T318 — usable-only Default + empty / quiet-mixed / after_help
// ---------------------------------------------------------------------------

#[test]
fn backup_list__all_residual__no_usable_and_footer() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let backup_dir = ensure_backups_dir(&vault);
    write_plain_bak(&backup_dir, "vault-2026-01-01T00-00-00.db.bak");
    write_plain_bak(&backup_dir, "vault-2026-01-02T00-00-00.db.bak");

    let out = list_output(&vault, &[], None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("No usable backups."),
        "AC3: residuals-only Default; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)"),
        "AC3: no residual table tokens; stdout={stdout}"
    );
    assert!(
        stdout.contains("2 backup(s) not recoverable under current key"),
        "AC3: footer count on stdout; stdout={stdout}"
    );
    assert!(
        !stderr.contains("not recoverable under current key"),
        "AC3: footer not on stderr; stderr={stderr}"
    );
}

#[test]
fn backup_list_honesty__quiet_mixed__usable_row_no_footer() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&vault)
        .arg("backup")
        .assert()
        .success();

    let backup_dir = ensure_backups_dir(&vault);
    write_incomplete_bak(&backup_dir, "vault-2099-12-31T23-59-59.db.bak");
    write_plain_bak(&backup_dir, "vault-2099-12-30T00-00-00.db.bak");

    let out = list_output(&vault, &["--quiet"], None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("vault-") && l.contains(".db.bak"))
        .collect();

    assert_eq!(
        lines.len(),
        1,
        "AC5: quiet mixed prints usable row only; stdout={stdout}"
    );
    assert!(
        !stdout.contains("(legacy plain)")
            && !stdout.contains("(no core tables)")
            && !stdout.contains("(unreadable key)")
            && !stdout.contains("(corrupt)"),
        "AC5: no residual tokens; stdout={stdout}"
    );
    assert!(
        !stdout.contains("not recoverable under current key")
            && !stderr.contains("not recoverable under current key"),
        "AC5: quiet omits footer; stdout={stdout} stderr={stderr}"
    );
    assert!(
        !stdout.contains("No usable backups."),
        "AC5: mixed quiet must not claim no usable; stdout={stdout}"
    );
}

#[test]
fn backup_list__empty__no_backups_found_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let _backup_dir = ensure_backups_dir(&vault);

    let out = list_output(&vault, &[], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "AC6: empty list exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("No backups found."),
        "AC6: empty dir message; stdout={stdout}"
    );
    assert!(
        !stdout.contains("No usable backups."),
        "AC6: must not use residuals-only message; stdout={stdout}"
    );
    assert!(
        !stdout.contains("not recoverable under current key")
            && !stderr.contains("not recoverable under current key"),
        "AC6: no residual footer; stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn backup_list_help__after_help__names_usable_only_and_verbose() {
    let out = common::hermetic_bin()
        .arg("backup")
        .arg("list")
        .arg("--help")
        .output()
        .expect("backup list --help must spawn");
    assert!(
        out.status.success(),
        "AC14: help must exit 0; out={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
    .to_ascii_lowercase();
    assert!(
        help.contains("usable") && help.contains("verbose"),
        "AC14: after_help names usable-only default and --verbose; help={help}"
    );
    assert!(
        help.contains("footer") || help.contains("residual") || help.contains("stdout"),
        "AC14: after_help names residual footer / stdout; help={help}"
    );
    assert!(
        help.contains("quiet"),
        "AC14: after_help names --quiet; help={help}"
    );
}
