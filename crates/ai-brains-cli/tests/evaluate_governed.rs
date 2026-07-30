#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T169 — `evaluate governed` CLI exit codes + path refuse.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/governed-memory/scenarios")
}

fn cmd() -> Command {
    Command::cargo_bin("ai-brains").expect("binary")
}

#[test]
fn evaluate_cli__all_pass__exit_0_json_stdout() {
    let fixtures = fixtures_dir();
    assert!(
        fixtures.is_dir(),
        "fixtures missing: {}",
        fixtures.display()
    );
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(&fixtures)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("hard_gates_passed"))
        .stdout(predicate::str::contains("report_hash"));
}

#[test]
fn evaluate_cli__missing_fixtures_dir__exit_nonzero() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no-such-scenarios");
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(&missing)
        .assert()
        .code(predicate::in_iter([1_i32, 6]));
}

#[test]
fn evaluate_cli__invalid_scenario_json__exit_6() {
    let dir = tempdir().unwrap();
    let fixtures = dir.path().join("scenarios");
    fs::create_dir_all(&fixtures).unwrap();
    fs::write(
        fixtures.join("bad.json"),
        r#"{ "schema_version": 99, "id": "x", "title": "t" }"#,
    )
    .unwrap();
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(&fixtures)
        .assert()
        .code(6)
        .stdout(predicate::str::contains("INVALID_PAYLOAD").or(predicate::str::contains("schema")));
}

#[test]
fn evaluate_cli__hard_fail_fixture__exit_7() {
    // path_alias with min_valid=1 is not our fixture; craft a zero-recall hard fail.
    let dir = tempdir().unwrap();
    let fixtures = dir.path().join("scenarios");
    fs::create_dir_all(&fixtures).unwrap();
    fs::write(
        fixtures.join("zero_recall.json"),
        r#"{
          "schema_version": 1,
          "id": "zero_recall_trap",
          "title": "zero recall",
          "status": "active",
          "seed": { "program": "path_alias_wsl", "params": {} },
          "min_valid_claims_count": 1,
          "actions": [{ "op": "resolve_scope" }],
          "asserts": {
            "hard": [
              { "metric": "scope_key_stable", "op": "eq", "value": true }
            ],
            "soft": []
          }
        }"#,
    )
    .unwrap();
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(&fixtures)
        .assert()
        .code(7);
}

#[test]
fn evaluate_cli__strict_soft__exit_7() {
    let dir = tempdir().unwrap();
    let fixtures = dir.path().join("scenarios");
    fs::create_dir_all(&fixtures).unwrap();
    // Valid seed but impossible soft citation threshold.
    fs::write(
        fixtures.join("soft_fail.json"),
        r#"{
          "schema_version": 1,
          "id": "soft_only_fail",
          "title": "soft",
          "status": "active",
          "seed": { "program": "project_briefing_minimal", "params": {} },
          "min_valid_claims_count": 1,
          "actions": [{ "op": "build_project_briefing", "dry_run": true }],
          "asserts": {
            "hard": [
              { "metric": "stale_as_current_count", "op": "eq", "value": 0 }
            ],
            "soft": [
              { "metric": "citation_coverage", "op": "gte", "value": 2.0 }
            ]
          }
        }"#,
    )
    .unwrap();
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(&fixtures)
        .arg("--strict-soft")
        .assert()
        .code(7);
}

#[test]
fn evaluate_cli__refuse_report_equals_vault_path() {
    let dir = tempdir().unwrap();
    let vaultish = dir.path().join("vault.db");
    // Report path with .db extension is refused before run.
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(fixtures_dir())
        .arg("--report")
        .arg(&vaultish)
        .arg("--allow-report-overwrite")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("PATH_REFUSED").or(predicate::str::contains("vault")));
}

#[test]
fn evaluate_cli__does_not_open_live_vault_for_write() {
    // Point --vault-path at a non-existing live path; evaluate must still succeed
    // without creating/writing that path (hermetic tempfile only).
    let dir = tempdir().unwrap();
    let live = dir.path().join("must-not-touch.db");
    assert!(!live.exists());
    cmd()
        .arg("--no-project-context")
        .arg("--vault-path")
        .arg(&live)
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(fixtures_dir())
        .arg("--scenario")
        .arg("windows_wsl_repo_alias")
        .assert()
        .code(0);
    assert!(
        !live.exists(),
        "evaluate must not create live vault path {}",
        live.display()
    );
}

#[test]
fn evaluate_cli__report_write__exit_0() {
    let dir = tempdir().unwrap();
    let report = dir.path().join("evaluate-report.json");
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(fixtures_dir())
        .arg("--scenario")
        .arg("cold_start_cited_project")
        .arg("--report")
        .arg(&report)
        .assert()
        .code(0);
    let body = fs::read_to_string(&report).expect("report written");
    assert!(body.contains("report_hash"));
    assert!(body.contains("hard_gates_passed"));
}

#[test]
fn evaluate_cli__report_dash__stdout_only_no_file() {
    // `--report -` must not create a literal file named "-" and must still print JSON.
    let dir = tempdir().unwrap();
    let cwd_dash = dir.path().join("-");
    assert!(!cwd_dash.exists());
    cmd()
        .current_dir(dir.path())
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(fixtures_dir())
        .arg("--scenario")
        .arg("cold_start_cited_project")
        .arg("--report")
        .arg("-")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("report_hash"));
    assert!(
        !cwd_dash.exists(),
        "must not write a file named '-' for --report -"
    );
}

#[test]
fn evaluate_cli__unknown_scenario_filter__exit_6() {
    cmd()
        .arg("--no-project-context")
        .arg("evaluate")
        .arg("governed")
        .arg("--fixtures")
        .arg(fixtures_dir())
        .arg("--scenario")
        .arg("definitely_not_a_real_scenario_id")
        .assert()
        .code(6)
        .stdout(
            predicate::str::contains("INVALID_PAYLOAD")
                .or(predicate::str::contains("scenario"))
                .or(predicate::str::contains("filter")),
        );
}
