//! T300 — graph rebuild dry-run / clap / help hermetics (graph-on).
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

#[cfg(feature = "graph")]
use rstest::rstest;
#[cfg(feature = "graph")]
use tempfile::tempdir;

#[cfg(feature = "graph")]
fn parse_pin_id(stdout: &str) -> String {
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("Memory ")
            && let Some(id) = rest.split_whitespace().next()
        {
            return id.to_string();
        }
    }
    panic!("pin stdout missing memory id: {stdout}");
}

#[cfg(feature = "graph")]
fn pin_decision(vault: &std::path::Path) -> String {
    const PROJECT_ID: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    const SESSION_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    common::hermetic_vault(vault).arg("init").assert().success();

    let pin = common::hermetic_cmd_with_ids(vault, PROJECT_ID, SESSION_ID)
        .arg("pin")
        .arg("DECISION: T300 rebuild dry-run keeps pin node.")
        .output()
        .expect("pin");
    assert!(
        pin.status.success(),
        "pin failed: stdout={} stderr={}",
        String::from_utf8_lossy(&pin.stdout),
        String::from_utf8_lossy(&pin.stderr)
    );
    parse_pin_id(&String::from_utf8_lossy(&pin.stdout))
}

/// T300 AC1: dry-run prints density + [dry-run]; does not drop graph nodes (RECALLS stay).
#[cfg(feature = "graph")]
#[test]
fn graph_rebuild__dry_run__prints_density_no_mutation() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let memory_id = pin_decision(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("rebuild")
        .arg("--dry-run")
        .output()
        .expect("rebuild --dry-run");
    assert!(
        out.status.success(),
        "dry-run failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("status:") && stdout.contains("nodes:"),
        "must print density labels; got: {stdout}"
    );
    assert!(
        stdout.contains("[dry-run]") && stdout.contains("no mutation"),
        "must print dry-run line; got: {stdout}"
    );
    // When the host daemon is reachable, dry-run prints the T188 NOTICE (AC4).
    // CI typically has no daemon — NOTICE is optional there.
    if stdout.contains("NOTICE:") {
        assert!(
            stdout.contains("daemon")
                && stdout.contains("ai-brains daemon stop")
                && stdout.contains("sc stop"),
            "NOTICE must carry T188 stop guidance; got: {stdout}"
        );
    }

    let json_out = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(&memory_id)
        .arg("--format")
        .arg("json")
        .output()
        .expect("neighbors json");
    assert!(
        json_out.status.success(),
        "neighbors after dry-run failed: {}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json_out.stdout).trim())
            .expect("neighbors json parse");
    let neighbors = parsed["neighbors"].as_array().expect("neighbors array");
    assert!(
        neighbors
            .iter()
            .any(|hit| { hit["direction"] == "incoming" && hit["label"] == "RECALLS" }),
        "dry-run must keep RECALLS; got: {parsed}"
    );
}

/// T300 AC9: rebuild --format rejects auto/JSON/Pretty.
#[cfg(feature = "graph")]
#[rstest]
#[case::auto("auto")]
#[case::json_upper("JSON")]
#[case::pretty("Pretty")]
fn graph_rebuild__format_token__rejected_exit_2(#[case] bad: &str) {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    common::hermetic_vault(&vault)
        .arg("init")
        .assert()
        .success();

    let out = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("rebuild")
        .arg("--dry-run")
        .arg("--format")
        .arg(bad)
        .output()
        .expect("rebuild bad format");
    assert_eq!(
        out.status.code(),
        Some(2),
        "bad format {bad} must exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// T300 AC9: `--format json` accepted; health object only (no human `[dry-run]`).
#[cfg(feature = "graph")]
#[test]
fn graph_rebuild__format_json__accepted_health_only() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    common::hermetic_vault(&vault)
        .arg("init")
        .assert()
        .success();

    let ok = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("rebuild")
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .output()
        .expect("rebuild json dry-run");
    assert!(
        ok.status.success(),
        "json dry-run failed: {}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let stdout = String::from_utf8_lossy(&ok.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json object");
    assert!(v.get("status").is_some(), "health object required; got {v}");
    assert!(
        !stdout.contains("[dry-run]"),
        "JSON dry-run must omit human [dry-run] line; got {stdout}"
    );

    let update_help = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("graph")
        .arg("update")
        .arg("--help")
        .output()
        .expect("update help");
    let help = String::from_utf8_lossy(&update_help.stdout);
    assert!(
        help.contains("[default: json]")
            || (help.to_ascii_lowercase().contains("json") && help.contains("default")),
        "update --help must keep default json; got: {help}"
    );
}

/// T300 AC11: rebuild --help after_help names daemon / --dry-run / sparse floor / graph update.
#[cfg(feature = "graph")]
#[test]
fn graph_rebuild__help__after_help_names_daemon_dry_run_sparse() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("graph")
        .arg("rebuild")
        .arg("--help")
        .output()
        .expect("rebuild help");
    assert!(out.status.success(), "help failed");
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.to_ascii_lowercase().contains("daemon"),
        "after_help must name daemon; got: {help}"
    );
    assert!(
        help.contains("--dry-run"),
        "after_help must name --dry-run; got: {help}"
    );
    assert!(
        help.contains("0.50") || help.to_ascii_lowercase().contains("sparse"),
        "after_help must name floor/sparse; got: {help}"
    );
    assert!(
        help.contains("graph update"),
        "after_help must name graph update; got: {help}"
    );

    let enum_help = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("graph")
        .arg("--help")
        .output()
        .expect("graph help");
    let graph_help = String::from_utf8_lossy(&enum_help.stdout);
    assert!(
        graph_help.contains("PREVIEW") || graph_help.contains("prefer"),
        "enum neighbors after_help must stay; got: {graph_help}"
    );
}

/// T300 AC2 (CI / daemon-down): mutating rebuild prints density and keeps pin RECALLS.
///
/// When the host daemon is Running, Safety fail-closed exits 1 — that is expected on
/// operator machines; unit inject `rebuild_with_daemon_state(..., daemon_up=false)` is
/// the local SoT. CI has no daemon so this path is exit 0.
#[cfg(feature = "graph")]
#[test]
fn graph_rebuild__mutate__prints_density_and_keeps_pin_node() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let memory_id = pin_decision(&vault);

    let out = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("rebuild")
        .output()
        .expect("rebuild mutate");
    if out.status.code() == Some(1) {
        let err = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            err.contains("daemon is running") || err.contains("daemon"),
            "exit 1 must be daemon fail-closed; got: {err}"
        );
        return;
    }
    assert!(
        out.status.success(),
        "mutate failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("status:"), "got: {stdout}");
    assert!(stdout.contains("nodes:"), "got: {stdout}");
    assert!(stdout.contains("edges:"), "got: {stdout}");
    assert!(stdout.contains("edge_node_ratio:"), "got: {stdout}");

    let status_line = stdout
        .lines()
        .find(|l| l.starts_with("status:"))
        .expect("status line");
    let rebuild_status = status_line.trim_start_matches("status:").trim();

    let update = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("update")
        .arg("--format")
        .arg("human")
        .output()
        .expect("update human");
    assert!(update.status.success());
    let update_out = String::from_utf8_lossy(&update.stdout);
    let update_status = update_out
        .lines()
        .find(|l| l.starts_with("status:"))
        .expect("update status")
        .trim_start_matches("status:")
        .trim();
    assert_eq!(
        rebuild_status, update_status,
        "rebuild status must equal update status"
    );

    let json_out = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(&memory_id)
        .arg("--format")
        .arg("json")
        .output()
        .expect("neighbors");
    assert!(json_out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json_out.stdout).trim())
            .expect("neighbors json");
    let neighbors = parsed["neighbors"].as_array().expect("array");
    assert!(
        neighbors
            .iter()
            .any(|hit| hit["direction"] == "incoming" && hit["label"] == "RECALLS"),
        "RECALLS must remain; got {parsed}"
    );
}
