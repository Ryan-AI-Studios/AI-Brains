//! T211 hermetic ACs: sync query ranking + plan/stale badge.
//!
//! AC1: Shipped Decision ranks above Plan for same track + keyword.
//! AC3: Pretty demoted Plan Decision contains `plan/stale?`.
//! AC6: `sync query --no-bridge` still re-ranks vault.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use tempfile::tempdir;

const KEYWORD: &str = "keyword_rank_token";

fn init_vault(vault: &std::path::Path) {
    common::hermetic_cmd(vault).arg("init").assert().success();
}

fn pin(vault: &std::path::Path, content: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn sync_query_pretty(vault: &std::path::Path, query: &str) -> String {
    let out = common::hermetic_cmd(vault)
        .arg("--log-format")
        .arg("off")
        .arg("sync")
        .arg("query")
        .arg(query)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("sync query");
    assert!(
        out.status.success(),
        "sync query must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// AC1 + AC6: Shipped above Plan; --no-bridge re-ranks vault.
#[test]
fn sync_query_ranking__shipped_above_plan_no_bridge__ac1_ac6() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    // Pin Plan first so raw insertion order would favor it if ranking failed.
    pin(
        &vault,
        &format!("DECISION: plan-only T999 expanded ranking test {KEYWORD} until go"),
    );
    pin(
        &vault,
        &format!("DECISION: shipped T999 {KEYWORD} PR #1 complete"),
    );

    let stdout = sync_query_pretty(&vault, KEYWORD);

    let ship_pos = stdout.find("shipped");
    let plan_pos = stdout.find("plan-only");
    assert!(
        ship_pos.is_some(),
        "shipped decision must appear; got: {stdout}"
    );
    assert!(
        plan_pos.is_some(),
        "plan decision must appear; got: {stdout}"
    );
    assert!(
        ship_pos.unwrap() < plan_pos.unwrap(),
        "Shipped must rank above Plan; got: {stdout}"
    );
}

/// AC3: Pretty demoted Plan Decision contains `plan/stale?`.
#[test]
fn sync_query_ranking__plan_badge__ac3() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    pin(
        &vault,
        &format!("DECISION: plan-only T999 expanded ranking test {KEYWORD} until go"),
    );
    pin(
        &vault,
        &format!("DECISION: shipped T999 {KEYWORD} PR #1 complete"),
    );

    let stdout = sync_query_pretty(&vault, KEYWORD);
    assert!(
        stdout.contains("plan/stale?"),
        "demoted Plan must show [plan/stale?] badge; got: {stdout}"
    );
}

/// Soft AC12: `sync query --limit 1` is respected after re-rank.
#[test]
fn sync_query_ranking__limit_one_respected__ac12() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    pin(
        &vault,
        &format!("DECISION: plan-only T999 expanded ranking test {KEYWORD} until go"),
    );
    pin(
        &vault,
        &format!("DECISION: shipped T999 {KEYWORD} PR #1 complete"),
    );

    let out = common::hermetic_cmd(&vault)
        .arg("--log-format")
        .arg("off")
        .arg("sync")
        .arg("query")
        .arg(KEYWORD)
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .arg("--limit")
        .arg("1")
        .output()
        .expect("sync query --limit 1");
    assert!(
        out.status.success(),
        "sync query --limit 1 must succeed; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("shipped"),
        "top re-ranked hit should be shipped under limit 1; got: {stdout}"
    );
    assert!(
        !stdout.contains("plan-only"),
        "plan hit must be truncated by --limit 1; got: {stdout}"
    );
}

/// AC2 hermetic: CONSTRAINT outranks plain Other for same keyword.
#[test]
fn sync_query_ranking__constraint_above_other() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    pin(
        &vault,
        &format!("plain chat about {KEYWORD} without markers"),
    );
    pin(&vault, &format!("CONSTRAINT: {KEYWORD} must be safe"));

    let stdout = sync_query_pretty(&vault, KEYWORD);
    let cons_pos = stdout.find("CONSTRAINT:");
    let plain_pos = stdout.find("plain chat");
    assert!(
        cons_pos.is_some() && plain_pos.is_some(),
        "both hits required; got: {stdout}"
    );
    assert!(
        cons_pos.unwrap() < plain_pos.unwrap(),
        "CONSTRAINT must rank above Other; got: {stdout}"
    );
}
