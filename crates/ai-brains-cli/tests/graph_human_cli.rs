//! T246 — hermetic graph human CLI (pretty/json + update --format human).

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

#[cfg(feature = "graph")]
use std::collections::BTreeSet;
#[cfg(feature = "graph")]
use std::path::Path;
#[cfg(feature = "graph")]
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
fn open_zero_vault(
    vault: &Path,
) -> Result<ai_brains_store::VaultConnection, Box<dyn std::error::Error>> {
    use ai_brains_core::temp_env::TempEnv;
    use ai_brains_crypto::SqlCipherKey;
    use ai_brains_store::VaultConnection;

    let _allow = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    let key = SqlCipherKey::try_from_raw(common::ZERO_SQLCIPHER_KEY.to_string())?;
    Ok(VaultConnection::open(vault, &key)?)
}

#[cfg(feature = "graph")]
fn seed_node(
    vault: &Path,
    kind: &str,
    external_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_zero_vault(vault)?;
    let guard = conn.lock()?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES (?1, ?2)",
        [kind, external_id],
    )?;
    Ok(())
}

#[cfg(feature = "graph")]
fn seed_edge(
    vault: &Path,
    src: &str,
    label: &str,
    dst: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_zero_vault(vault)?;
    let guard = conn.lock()?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight)
         SELECT s.node_id, ?2, d.node_id, 1.0
         FROM graph_node s
         JOIN graph_node d
         WHERE s.external_id = ?1 AND d.external_id = ?3",
        [src, label, dst],
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

    let auto = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("update")
        .arg("--format")
        .arg("auto")
        .output()
        .expect("graph update auto");
    assert!(auto.status.success());
    let auto_parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&auto.stdout).trim())
            .expect("auto must stay pretty JSON");
    assert!(auto_parsed.get("status").is_some());
    assert!(auto_parsed.get("density").is_some());
}

#[cfg(feature = "graph")]
#[test]
fn graph_neighbors__present_empty__pretty_no_neighbors() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_node(&vault, "memory", "mem-leaf").expect("seed memory");

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg("mem-leaf")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("pretty");
    assert_eq!(pretty.status.code(), Some(0));
    let out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        out.contains("No neighbors for mem-leaf."),
        "present-empty must say No neighbors; got: {out}"
    );
    assert!(out.contains("next: ai-brains graph update"));
}

#[cfg(feature = "graph")]
#[test]
fn graph_hierarchy_session__missing_wrong_kind_empty__f3_copy() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_node(&vault, "session", "sess-only").expect("session node");
    seed_node(&vault, "memory", "mem-leaf").expect("memory node");

    let missing = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("hierarchy")
        .arg("00000000-0000-0000-0000-000000000000")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("hierarchy missing");
    let missing_out = String::from_utf8_lossy(&missing.stdout);
    assert!(missing_out.contains("No graph node"));

    let missing_session = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("session")
        .arg("00000000-0000-0000-0000-000000000000")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("session missing");
    let missing_session_out = String::from_utf8_lossy(&missing_session.stdout);
    assert!(
        missing_session_out.contains("No graph node"),
        "session missing must say No graph node; got: {missing_session_out}"
    );
    assert!(missing_session_out.contains("next: ai-brains graph update"));

    let wrong_h = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("hierarchy")
        .arg("sess-only")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("hierarchy wrong kind");
    let wrong_h_out = String::from_utf8_lossy(&wrong_h.stdout);
    assert!(
        wrong_h_out.contains("No memory node for sess-only."),
        "got: {wrong_h_out}"
    );

    let leaf = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("hierarchy")
        .arg("mem-leaf")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("hierarchy leaf");
    let leaf_out = String::from_utf8_lossy(&leaf.stdout);
    assert!(
        leaf_out.contains("No SYNTHESIZED_FROM children (leaf)."),
        "got: {leaf_out}"
    );

    let wrong_s = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("session")
        .arg("mem-leaf")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("session wrong kind");
    let wrong_s_out = String::from_utf8_lossy(&wrong_s.stdout);
    assert!(
        wrong_s_out.contains("No session node for mem-leaf."),
        "got: {wrong_s_out}"
    );

    let empty_s = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("session")
        .arg("sess-only")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("session empty");
    let empty_s_out = String::from_utf8_lossy(&empty_s.stdout);
    assert!(
        empty_s_out.contains("No memories in this session via graph edges."),
        "got: {empty_s_out}"
    );
}

#[cfg(feature = "graph")]
#[test]
fn graph_hierarchy__pretty_and_json__indent_and_sorted_ids() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    seed_node(&vault, "memory", "root").expect("root");
    seed_node(&vault, "memory", "child-b").expect("child-b");
    seed_node(&vault, "memory", "child-a").expect("child-a");
    seed_edge(&vault, "root", "SYNTHESIZED_FROM", "child-b").expect("edge b");
    seed_edge(&vault, "root", "SYNTHESIZED_FROM", "child-a").expect("edge a");

    let pretty = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("hierarchy")
        .arg("root")
        .arg("--format")
        .arg("pretty")
        .output()
        .expect("hierarchy pretty");
    assert!(pretty.status.success());
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(pretty_out.contains("  child-a"));
    assert!(pretty_out.contains("  child-b"));
    assert!(!pretty_out.contains('└'));

    let json = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("hierarchy")
        .arg("root")
        .arg("--format")
        .arg("json")
        .output()
        .expect("hierarchy json");
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&json.stdout).trim()).expect("json");
    assert_eq!(
        object_keys(&parsed),
        BTreeSet::from(["root".into(), "synthesized_from".into()])
    );
    let ids: Vec<&str> = parsed["synthesized_from"]
        .as_array()
        .expect("arr")
        .iter()
        .map(|v| v.as_str().expect("id"))
        .collect();
    assert_eq!(ids, vec!["child-a", "child-b"]);
}

#[cfg(feature = "graph")]
#[test]
fn graph_neighbors__json_limit_and_sort__unlimited_unless_flag() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);
    {
        let conn = open_zero_vault(&vault).expect("open");
        let guard = conn.lock().expect("lock");
        guard
            .execute(
                "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('memory', 'hub')",
                [],
            )
            .expect("hub");
        for i in 0..51 {
            let id = format!("n-{i:02}");
            guard
                .execute(
                    "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('memory', ?1)",
                    [&id],
                )
                .expect("node");
            guard
                .execute(
                    "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight)
                     SELECT s.node_id, 'SOURCE_FOR', d.node_id, 1.0
                     FROM graph_node s JOIN graph_node d
                     WHERE s.external_id = 'hub' AND d.external_id = ?1",
                    [&id],
                )
                .expect("edge");
        }
        guard
            .execute(
                "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('session', 'sess-z')",
                [],
            )
            .expect("sess");
        guard
            .execute(
                "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight)
                 SELECT s.node_id, 'RECALLS', d.node_id, 1.0
                 FROM graph_node s JOIN graph_node d
                 WHERE s.external_id = 'sess-z' AND d.external_id = 'hub'",
                [],
            )
            .expect("incoming");
    }

    let unlimited = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg("hub")
        .arg("--format")
        .arg("json")
        .output()
        .expect("json unlimited");
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&unlimited.stdout).trim()).expect("json");
    let hits = parsed["neighbors"].as_array().expect("arr");
    assert_eq!(hits.len(), 52, "JSON without --limit must not cap at 50");
    assert_eq!(hits[0]["direction"], "incoming");
    assert_eq!(hits[1]["direction"], "outgoing");
    let outgoing_ids: Vec<&str> = hits
        .iter()
        .filter(|h| h["direction"] == "outgoing")
        .map(|h| h["external_id"].as_str().expect("id"))
        .collect();
    let mut sorted = outgoing_ids.clone();
    sorted.sort_unstable();
    assert_eq!(outgoing_ids, sorted, "outgoing ids must be lexicographic");

    let limited = common::hermetic_vault(&vault)
        .arg("graph")
        .arg("neighbors")
        .arg("hub")
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("2")
        .output()
        .expect("json limit");
    let limited_parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&limited.stdout).trim()).expect("json");
    assert_eq!(
        limited_parsed["neighbors"].as_array().expect("arr").len(),
        2
    );
}
