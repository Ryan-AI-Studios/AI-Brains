//! T285 AC12 — hermetic CLI: tagged pin is recall/search hit #1 vs live chrome dumps.
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::path::Path;
use tempfile::tempdir;

fn init_vault(vault_path: &Path) {
    common::hermetic_bin()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(vault_path)
        .arg("init")
        .assert()
        .success();
}

fn pin_tagged(vault: &Path, content: &str, tag: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .arg("--tag")
        .arg(tag)
        .assert()
        .success();
}

fn pin(vault: &Path, content: &str) {
    common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .assert()
        .success();
}

fn first_needle_line<'a>(stdout: &'a str, needle: &str) -> &'a str {
    stdout.lines().find(|l| l.contains(needle)).unwrap_or("")
}

fn seed_review_dumps(vault: &Path, needle: &str) {
    for i in 0..15 {
        let repeats = format!("{needle} ").repeat(12);
        pin(
            vault,
            &format!("# Review of Track 285: dump {i}\n{repeats}review body"),
        );
    }
}

#[test]
fn recall_and_search__tagged_pin_vs_review_dumps__hit_one__ac12() {
    let dir = tempdir().expect("tempdir");
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T285-rank-needle-{}", uuid::Uuid::new_v4());
    seed_review_dumps(&vault, &needle);
    pin_tagged(
        &vault,
        &format!("DECISION: {needle} we chose rank v2"),
        "t285",
    );

    for cmd in ["recall", "search"] {
        let pretty = common::hermetic_cmd(&vault)
            .arg(cmd)
            .arg(&needle)
            .arg("--limit")
            .arg("5")
            .arg("--format")
            .arg("pretty")
            .arg("--no-bridge")
            .output()
            .unwrap_or_else(|_| panic!("{cmd} pretty"));
        assert_eq!(
            pretty.status.code(),
            Some(0),
            "AC12: {cmd} must exit 0; stderr={}",
            String::from_utf8_lossy(&pretty.stderr)
        );
        let pretty_out = String::from_utf8_lossy(&pretty.stdout);
        let first_hit = first_needle_line(&pretty_out, &needle);
        assert!(
            first_hit.contains("DECISION:"),
            "AC12: {cmd} pretty hit #1 must be the pin; first_hit={first_hit:?} stdout={pretty_out}"
        );
        assert!(
            !first_hit.contains("# AI-Brains Session Onboarding")
                && !first_hit.contains("# Review of Track")
                && !first_hit.contains("## Objective"),
            "AC12: {cmd} pretty hit #1 must not be session chrome; first_hit={first_hit:?}"
        );
    }
}
