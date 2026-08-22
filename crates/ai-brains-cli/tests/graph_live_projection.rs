//! T262 AC6/AC7 — hermetic pin printed id is a graph memory node (no rebuild).
#![allow(clippy::disallowed_methods, non_snake_case)]

mod common;

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
        .arg("DECISION: T262 pin-id is the graph memory id.")
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

#[cfg(feature = "graph")]
#[test]
fn pin__graph_on__printed_id_neighbors_json_without_rebuild__ac6() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let memory_id = pin_decision(&vault);

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
        "neighbors json failed: {}",
        String::from_utf8_lossy(&json_out.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json_out.stdout).trim())
            .expect("neighbors json parse");
    assert_eq!(parsed["memory_id"], memory_id);
    let neighbors = parsed["neighbors"].as_array().expect("neighbors array");
    assert!(
        neighbors.iter().any(|hit| {
            hit["direction"] == "incoming"
                && hit["label"] == "RECALLS"
                && hit["external_id"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
        }),
        "expected incoming RECALLS to session; got: {parsed}"
    );
}

#[cfg(feature = "graph")]
#[test]
fn pin__graph_on__printed_id_neighbors_pretty__ac7() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let memory_id = pin_decision(&vault);

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(&memory_id)
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("neighbors pretty");
    assert!(
        pretty.status.success(),
        "neighbors pretty failed: {}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains("in") && pretty_out.contains("RECALLS"),
        "pretty must show incoming RECALLS; got: {pretty_out}"
    );
    assert!(
        !pretty_out.contains("No graph node"),
        "printed pin id must be a graph node; got: {pretty_out}"
    );
}

/// T278 AC3: pin → neighbors pretty PREVIEW includes session memory caption (no rebuild).
#[cfg(feature = "graph")]
#[test]
fn pin__graph_on__neighbors_pretty__session_preview_nonblank() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let memory_id = pin_decision(&vault);

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(&memory_id)
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("neighbors pretty");
    assert!(
        pretty.status.success(),
        "neighbors pretty failed: {}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains("session"),
        "pretty must name session KIND; got: {pretty_out}"
    );
    assert!(
        pretty_out.contains("memories"),
        "session PREVIEW must include memories; got: {pretty_out}"
    );
    let preview_has_text = pretty_out.lines().any(|line| {
        line.contains("session")
            && line.contains("memories")
            && line.chars().any(|c| !c.is_whitespace())
    });
    assert!(
        preview_has_text,
        "session PREVIEW row must have a non-whitespace caption; got: {pretty_out}"
    );
}

/// CX1-P2: a vault session missing from graph_node must get rebuild, not F1b.
#[cfg(feature = "graph")]
#[test]
fn graph_session__vault_session_missing_graph_node__next_rebuild() {
    const PROJECT_ID: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    const SESSION_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let _memory_id = pin_decision(&vault);

    {
        use ai_brains_core::temp_env::TempEnv;
        use ai_brains_crypto::SqlCipherKey;
        use ai_brains_store::VaultConnection;
        let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
        let key =
            SqlCipherKey::try_from_raw(common::ZERO_SQLCIPHER_KEY.to_string()).expect("zero key");
        let conn = VaultConnection::open(&vault, &key).expect("open");
        let guard = conn.lock().expect("lock");
        guard
            .execute(
                "DELETE FROM graph_node WHERE external_id = ?1",
                [SESSION_ID],
            )
            .expect("delete session graph node");
    }

    let pretty = common::hermetic_cmd_with_ids(&vault, PROJECT_ID, SESSION_ID)
        .arg("graph")
        .arg("session")
        .arg(SESSION_ID)
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("session pretty");
    assert_eq!(pretty.status.code(), Some(0));
    let out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        out.contains("No graph node") && out.contains("graph rebuild"),
        "vault session missing graph node must next rebuild; got: {out}"
    );
    assert!(
        !out.contains("not a vault memory id") && !out.contains("graph update"),
        "must not use F1b unknown copy; got: {out}"
    );
}
