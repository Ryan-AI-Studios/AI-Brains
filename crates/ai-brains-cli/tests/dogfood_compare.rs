#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

//! T170 — `dogfood compare` CLI smoke + basic compare packet.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("ai-brains").expect("binary")
}

#[test]
fn dogfood_compare__help__lists_flags() {
    cmd()
        .arg("--no-project-context")
        .arg("dogfood")
        .arg("compare")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--governed"))
        .stdout(predicate::str::contains("--legacy"))
        .stdout(predicate::str::contains("--out"));
}

#[test]
fn dogfood_compare__basic_packet__writes_compare_hash() {
    let dir = tempdir().unwrap();
    let governed = dir.path().join("governed.json");
    let legacy = dir.path().join("legacy.json");
    let out = dir.path().join("compare.json");

    fs::write(
        &governed,
        r#"{
          "api_version": "1",
          "briefing_id": "b1",
          "kind": "Project",
          "denied": false,
          "decisions": [
            {"id": "d1", "kind": "Decision", "statement": "s", "state": "Approved", "evidence_handles": [{"evidence_id": "e1"}]}
          ],
          "conclusions": [],
          "warnings": [
            {"kind": "stale", "message": "m", "subject_id": "d1"},
            {"kind": "other", "message": "info"}
          ],
          "constraints": [],
          "freshness": {
            "total_sources": 0,
            "fresh_count": 0,
            "stale_count": 0,
            "unavailable_count": 0,
            "worst_state": "Unknown"
          },
          "evidence_handles": [],
          "budget": {
            "max_words": 100,
            "used_words": 1,
            "truncated_sections": [],
            "more_available": false
          },
          "scope": {
            "scope_key": "Repository:00000000-0000-0000-0000-000000000001",
            "confidence": "High",
            "warnings": [],
            "alternatives": [],
            "authoritative": true
          }
        }"#,
    )
    .unwrap();

    fs::write(
        &legacy,
        r#"{"text": "DECISION: one\nCONSTRAINT: two", "word_count": 4}"#,
    )
    .unwrap();

    cmd()
        .arg("--no-project-context")
        .arg("dogfood")
        .arg("compare")
        .arg("--governed")
        .arg(&governed)
        .arg("--legacy")
        .arg(&legacy)
        .arg("--out")
        .arg(&out)
        .arg("--stage")
        .arg("C")
        .arg("--t169-exit")
        .arg("0")
        .arg("--sha256-pre")
        .arg("aa")
        .arg("--sha256-post")
        .arg("aa")
        .assert()
        .success()
        .stdout(predicate::str::contains("compare_hash"))
        .stdout(predicate::str::contains("warning_refs_all"));

    let body = fs::read_to_string(&out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["legacy_preflight"]["decision_marker_count"], 1);
    assert_eq!(v["legacy_preflight"]["constraint_marker_count"], 1);
    assert_eq!(v["governed_briefing"]["decision_count"], 1);
    assert!(v["compare_hash"].as_str().unwrap().len() == 64);
    // risk only: other excluded
    let refs = v["human_review_seed"]["warning_refs_all"]
        .as_array()
        .unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0]["kind"], "stale");
}
