//! T246 — hermetic graph human CLI (pretty/json + update --format human).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use std::collections::BTreeSet;
use std::path::Path;
use tempfile::tempdir;

#[cfg(not(feature = "graph"))]
#[test]
fn graph_neighbors__format_pretty__feature_off_exit_2() {
    let out = common::hermetic_bin()
        .arg("--no-project-context")
        .arg("graph")
        .arg("neighbors")
        .arg("x")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("graph neighbors stub");
    assert_eq!(
        out.status.code(),
        Some(2),
        "feature-off must exit 2; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("FEATURE_UNAVAILABLE"),
        "must prefix FEATURE_UNAVAILABLE; got: {stdout}"
    );
}

#[cfg(feature = "graph")]
fn init_vault(vault: &Path) {
    common::hermetic_vault(vault).arg("init").assert().success();
}

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
fn seed_session_recall(
    vault: &Path,
    session_id: &str,
    memory_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::SqlCipherKey;
    use ai_brains_store::VaultConnection;

    let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::try_from_raw(common::ZERO_SQLCIPHER_KEY.to_string())?;
    let conn = VaultConnection::open(vault, &key)?;
    let guard = conn.lock()?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('session', ?1)",
        [session_id],
    )?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('memory', ?1)",
        [memory_id],
    )?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight)
         SELECT s.node_id, 'RECALLS', d.node_id, 1.0
         FROM graph_node s
         JOIN graph_node d
         WHERE s.external_id = ?1 AND d.external_id = ?2",
        [session_id, memory_id],
    )?;
    Ok(())
}

#[cfg(feature = "graph")]
fn pin_fixture(vault: &Path) -> (String, String) {
    const PROJECT_ID: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    const SESSION_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    init_vault(vault);

    let turn_json = r#"{
        "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "project_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "harness_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "turn_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
        "privacy": "LocalOnly",
        "role": "user",
        "content": "T246 session memory preview seed."
    }"#;
    common::hermetic_vault(vault)
        .arg("ingest")
        .write_stdin(turn_json)
        .assert()
        .success();

    let pin = common::hermetic_cmd_with_ids(vault, PROJECT_ID, SESSION_ID)
        .arg("pin")
        .arg("T246 session memory preview seed.")
        .output()
        .expect("pin");
    assert!(
        pin.status.success(),
        "pin failed: {}",
        String::from_utf8_lossy(&pin.stderr)
    );
    // Pin stdout is turn_id; memory_projection.memory_id is a separate MemoryId.
    // AC9 preview must use the projection id that `memory list` shows.
    let listed = common::hermetic_cmd_with_ids(vault, PROJECT_ID, SESSION_ID)
        .arg("memory")
        .arg("list")
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("20")
        .output()
        .expect("memory list");
    assert!(
        listed.status.success(),
        "memory list failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let listed_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&listed.stdout).trim())
            .unwrap_or_else(|e| panic!("memory list json: {e}"));
    let memory_id = listed_json["items"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|item| {
            let preview = item["preview"].as_str().unwrap_or("");
            if preview.contains("T246 session memory preview seed") {
                item["memory_id"].as_str().map(str::to_string)
            } else {
                None
            }
        })
        .unwrap_or_else(|| parse_pin_id(&String::from_utf8_lossy(&pin.stdout)));
    seed_session_recall(vault, SESSION_ID, &memory_id).expect("seed RECALLS");
    (memory_id, SESSION_ID.to_string())
}

#[cfg(feature = "graph")]
fn object_keys(v: &serde_json::Value) -> BTreeSet<String> {
    v.as_object().expect("object").keys().cloned().collect()
}

#[cfg(feature = "graph")]
#[test]
fn graph_neighbors__json_and_pretty__frozen_keys_and_dir() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let (memory_id, _) = pin_fixture(&vault);

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
    let stdout = String::from_utf8_lossy(&json_out.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("neighbors json parse");
    assert_eq!(
        object_keys(&parsed),
        BTreeSet::from(["memory_id".into(), "neighbors".into()])
    );
    assert!(parsed["neighbors"].is_array());
    for hit in parsed["neighbors"].as_array().expect("arr") {
        assert_eq!(
            object_keys(hit),
            BTreeSet::from(["external_id".into(), "label".into(), "direction".into()])
        );
    }

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
        pretty_out.contains("DIR") || pretty_out.contains("in") || pretty_out.contains("RECALLS"),
        "pretty neighbors should be scannable; got: {pretty_out}"
    );
}

#[cfg(feature = "graph")]
#[test]
fn graph_neighbors__unknown_id__pretty_no_node_json_empty_exit_0() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    let unknown = "00000000-0000-0000-0000-000000000000";

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(unknown)
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("pretty unknown");
    assert_eq!(pretty.status.code(), Some(0));
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains("No graph node"),
        "pretty unknown must mention No graph node; got: {pretty_out}"
    );

    let json = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg(unknown)
        .arg("--format")
        .arg("json")
        .output()
        .expect("json unknown");
    assert_eq!(json.status.code(), Some(0));
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json.stdout).trim()).expect("json");
    assert_eq!(parsed["neighbors"], serde_json::json!([]));
    assert_eq!(
        object_keys(&parsed),
        BTreeSet::from(["memory_id".into(), "neighbors".into()])
    );
}

#[cfg(feature = "graph")]
#[test]
fn graph_session__pretty_and_json__id_preview_and_compact_ids() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    let (memory_id, session_id) = pin_fixture(&vault);

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("session")
        .arg(&session_id)
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("session pretty");
    assert!(
        pretty.status.success(),
        "session pretty failed: {}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains(&memory_id),
        "pretty session must print memory id; got: {pretty_out}"
    );
    assert!(
        pretty_out.contains("T246 session memory preview seed"),
        "pretty session must print preview; got: {pretty_out}"
    );

    let json = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("session")
        .arg(&session_id)
        .arg("--format")
        .arg("json")
        .output()
        .expect("session json");
    assert!(json.status.success());
    let stdout = String::from_utf8_lossy(&json.stdout);
    assert!(
        !stdout.trim_start().starts_with("{\n"),
        "session json must be compact; got: {stdout}"
    );
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("json");
    assert_eq!(
        object_keys(&parsed),
        BTreeSet::from(["session_id".into(), "memories".into()])
    );
    let memories = parsed["memories"].as_array().expect("memories");
    assert!(
        memories
            .iter()
            .any(|v| v.as_str() == Some(memory_id.as_str())),
        "json memories must include pinned id; got: {parsed}"
    );
}

#[cfg(feature = "graph")]
#[test]
fn graph_update__default_json_and_human__t213_keys_and_labels() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let json = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("update")
        .output()
        .expect("graph update");
    assert!(
        json.status.success(),
        "graph update failed: {}",
        String::from_utf8_lossy(&json.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json.stdout).trim()).expect("pretty json");
    for key in [
        "nodes",
        "edges",
        "pinned_memories",
        "memory_nodes",
        "edge_node_ratio",
        "density",
        "status",
        "note",
    ] {
        assert!(
            parsed.get(key).is_some(),
            "missing T213 key {key} in {parsed}"
        );
    }

    let human = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("update")
        .arg("--format")
        .arg("human")
        .output()
        .expect("graph update human");
    assert!(human.status.success());
    let human_out = String::from_utf8_lossy(&human.stdout);
    assert!(
        human_out.contains("status:"),
        "human must contain status:; got: {human_out}"
    );
    assert!(
        human_out.contains("density:"),
        "human must contain density:; got: {human_out}"
    );
}
