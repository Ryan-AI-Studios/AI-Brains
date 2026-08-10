//! T208 — Quiet Cozo / bridge INFO on human CLI paths (hermetic locks).
//!
//! AC1/AC2/AC6 require `--features graph`. AC7 lives as a unit test in main.rs.
//! AC8 asserts the hermetic denylist includes RUST_LOG (always runs).
//!
//! **M3 critical:** AC1 must use unset RUST_LOG (`env_remove` / denylist strip).
//! Never `.env("RUST_LOG", "")` — empty string is ERROR-only, not product default.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::path::Path;
use tempfile::tempdir;

#[cfg(feature = "graph")]
const COZO_INIT_MSG: &str = "CozoProxyBackend initialized";

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

#[cfg(feature = "graph")]
fn combined_streams(stdout: &[u8], stderr: &[u8]) -> String {
    let mut out = String::from_utf8_lossy(stdout).into_owned();
    out.push('\n');
    out.push_str(&String::from_utf8_lossy(stderr));
    out
}

// ---------------------------------------------------------------------------
// AC8 — denylist includes RUST_LOG (always; not graph-gated)
// ---------------------------------------------------------------------------

#[test]
fn ambient_denylist__includes_rust_log() {
    assert!(
        common::AMBIENT_DENYLIST.contains(&"RUST_LOG"),
        "AMBIENT_DENYLIST must include RUST_LOG (T208 F29); got: {:?}",
        common::AMBIENT_DENYLIST
    );
}

/// T218 F38 / AC19: dual-floor and RRF env keys stripped from hermetic ambient.
#[test]
fn ambient_denylist__includes_semantic_score_and_rrf_keys__ac19() {
    for key in [
        "AI_BRAINS_SEMANTIC_MIN_SCORE",
        "AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE",
        "AI_BRAINS_RRF_K",
    ] {
        assert!(
            common::AMBIENT_DENYLIST.contains(&key),
            "AMBIENT_DENYLIST must include {key} (T218 F38); got: {:?}",
            common::AMBIENT_DENYLIST
        );
    }
}

// ---------------------------------------------------------------------------
// AC1 — graph-on recall under unset RUST_LOG has no Cozo init line
// ---------------------------------------------------------------------------

#[cfg(feature = "graph")]
#[test]
fn quiet_cozo__recall_unset_rust_log__no_cozo_init_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // hermetic_bin strips RUST_LOG via denylist. Do NOT re-set RUST_LOG
    // (especially not to "") — empty is ERROR-only, not product default (M3).
    // F5: default (no --quiet) must already silence Cozo init after F2+F8.
    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .arg("recall")
        .arg("zzzzt208quietcozo")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .arg("--global")
        .output()
        .expect("recall under default filter");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_streams(&out.stdout, &out.stderr);
    assert!(
        !combined.contains(COZO_INIT_MSG),
        "AC1: default filter must not emit Cozo init; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// AC2 — RUST_LOG=ai_brains_graph=debug shows Cozo init (escape hatch)
// ---------------------------------------------------------------------------

#[cfg(feature = "graph")]
#[test]
fn quiet_cozo__recall_graph_debug__shows_cozo_init_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Re-set after hermetic strip so the child sees the debug directive.
    let out = common::hermetic_vault(&vault)
        .arg("--no-project-context")
        .env("RUST_LOG", "ai_brains_graph=debug")
        .arg("recall")
        .arg("zzzzt208debugcozo")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .arg("--global")
        .output()
        .expect("recall under graph debug");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_streams(&out.stdout, &out.stderr);
    assert!(
        combined.contains(COZO_INIT_MSG),
        "AC2: RUST_LOG=ai_brains_graph=debug must show Cozo init; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// Soft AC6 — sync query under unset RUST_LOG free of Cozo init
// ---------------------------------------------------------------------------

#[cfg(feature = "graph")]
#[test]
fn quiet_cozo__sync_query_unset_rust_log__no_cozo_init_line() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Soft AC6: default path (no --quiet) must be free of Cozo init (F5 parity).
    let out = common::hermetic_cmd(&vault)
        .arg("sync")
        .arg("query")
        .arg("zzzzt208syncquery")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("sync query under default filter");

    assert_eq!(
        out.status.code(),
        Some(0),
        "sync query must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = combined_streams(&out.stdout, &out.stderr);
    assert!(
        !combined.contains(COZO_INIT_MSG),
        "soft AC6: sync query default must not emit Cozo init; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// Soft AC4 — T81 quiet still silences human bridge-failed warning
// (thin re-lock; full coverage remains in smoke::test_recall_quiet_silences_bridge_warning)
// ---------------------------------------------------------------------------

#[test]
fn quiet_cozo__recall_quiet__no_bridge_failed_human_warning() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Non-git cwd so bridge path is more likely to warn when not quiet.
    assert!(!dir.path().join(".git").exists());

    let out = common::hermetic_bin()
        .current_dir(dir.path())
        .arg("--vault-path")
        .arg(&vault)
        .arg("--no-project-context")
        .arg("--log-format")
        .arg("off")
        .arg("recall")
        .arg("--quiet")
        .arg("--format")
        .arg("pretty")
        .arg("quiet bridge warning t208")
        .arg("--limit")
        .arg("1")
        .arg("--global")
        .output()
        .expect("recall --quiet");

    assert_eq!(
        out.status.code(),
        Some(0),
        "recall --quiet must exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("bridge query failed"),
        "AC4: --quiet must silence bridge-failed human warning; got: {stderr}"
    );
    assert!(
        !stderr.contains("falling back"),
        "AC4: --quiet must silence falling-back message; got: {stderr}"
    );
}
