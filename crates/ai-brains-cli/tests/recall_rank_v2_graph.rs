//! T285 AC17 — chrome parents must not seed graph neighbors (CLI `test(graph)`).
#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

#[cfg(feature = "graph")]
use std::path::Path;
#[cfg(feature = "graph")]
use tempfile::tempdir;

#[cfg(feature = "graph")]
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
fn pin_content(vault: &Path, content: &str) -> String {
    let out = common::hermetic_cmd(vault)
        .arg("pin")
        .arg(content)
        .output()
        .expect("pin");
    assert!(
        out.status.success(),
        "pin failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_pin_id(&String::from_utf8_lossy(&out.stdout))
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
fn seed_source_for(vault: &Path, src: &str, dst: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = open_zero_vault(vault)?;
    let guard = conn.lock()?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('memory', ?1)",
        [src],
    )?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_node (kind, external_id) VALUES ('memory', ?1)",
        [dst],
    )?;
    guard.execute(
        "INSERT OR IGNORE INTO graph_edge (src_id, label, dst_id, weight)
         SELECT s.node_id, 'SOURCE_FOR', d.node_id, 1.0
         FROM graph_node s JOIN graph_node d
         WHERE s.external_id = ?1 AND d.external_id = ?2",
        [src, dst],
    )?;
    Ok(())
}

/// AC17: a chrome dump that MATCHES is a hit; a non-MATCH neighbor is absent
/// because chrome did not seed graph expansion.
#[cfg(feature = "graph")]
#[test]
fn recall__graph_on__chrome_parent_does_not_seed_nonmatch_neighbor__ac17() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("vault.db");
    init_vault(&vault);

    let needle = format!("T285-graph-needle-{}", uuid::Uuid::new_v4());
    let neighbor_unique = format!("T285-graph-neighbor-{}", uuid::Uuid::new_v4());
    let repeats = format!("{needle} ").repeat(12);
    let chrome_id = pin_content(
        &vault,
        &format!("# Review of Track 285: chrome seed\n{repeats}review body"),
    );
    let neighbor_id = pin_content(
        &vault,
        &format!("plain chat that never mentions the query {neighbor_unique}"),
    );
    seed_source_for(&vault, &chrome_id, &neighbor_id).expect("SOURCE_FOR edge");

    let pretty = common::hermetic_cmd(&vault)
        .arg("recall")
        .arg(&needle)
        .arg("--limit")
        .arg("5")
        .arg("--format")
        .arg("pretty")
        .arg("--no-bridge")
        .output()
        .expect("recall pretty");
    assert_eq!(
        pretty.status.code(),
        Some(0),
        "AC17: recall must exit 0; stderr={}",
        String::from_utf8_lossy(&pretty.stderr)
    );
    let pretty_out = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_out.contains(&needle),
        "AC17: chrome dump that MATCHES must remain a hit; stdout={pretty_out}"
    );
    assert!(
        !pretty_out.contains(&neighbor_unique),
        "AC17: non-MATCH neighbor must be absent (chrome did not seed); stdout={pretty_out}"
    );
}
