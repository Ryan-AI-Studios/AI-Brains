//! T246 AC16 — hierarchy depth + diamond MIN(depth); node_kind missing vs present.

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_graph::{GraphSearch, GraphVault};
use std::collections::BTreeSet;

fn insert_memory_node(
    conn: &rusqlite::Connection,
    external_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO graph_node (kind, external_id) VALUES ('memory', ?1)",
        [external_id],
    )?;
    Ok(())
}

fn insert_synthesized_from(
    conn: &rusqlite::Connection,
    src: &str,
    dst: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO graph_edge (src_id, label, dst_id, weight)
         SELECT s.node_id, 'SYNTHESIZED_FROM', d.node_id, 1.0
         FROM graph_node s
         JOIN graph_node d
         WHERE s.external_id = ?1 AND d.external_id = ?2",
        [src, dst],
    )?;
    Ok(())
}

#[test]
fn get_synthesized_hierarchy_with_depth__diamond__min_depth_once()
-> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    {
        let conn = store.connection().lock()?;
        insert_memory_node(&conn, "root")?;
        insert_memory_node(&conn, "mid-a")?;
        insert_memory_node(&conn, "mid-b")?;
        insert_memory_node(&conn, "child")?;
        insert_synthesized_from(&conn, "root", "mid-a")?;
        insert_synthesized_from(&conn, "root", "mid-b")?;
        insert_synthesized_from(&conn, "mid-a", "child")?;
        insert_synthesized_from(&conn, "mid-b", "child")?;
    }

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    let with_depth = search.get_synthesized_hierarchy_with_depth("root")?;
    assert_eq!(
        with_depth,
        vec![
            ("mid-a".to_string(), 1),
            ("mid-b".to_string(), 1),
            ("child".to_string(), 2),
        ]
    );

    let ids = search.get_synthesized_hierarchy("root")?;
    let id_set: BTreeSet<String> = ids.into_iter().collect();
    let depth_set: BTreeSet<String> = with_depth.into_iter().map(|(id, _)| id).collect();
    assert_eq!(id_set, depth_set);
    assert_eq!(
        id_set,
        BTreeSet::from([
            "mid-a".to_string(),
            "mid-b".to_string(),
            "child".to_string(),
        ])
    );
    Ok(())
}

#[test]
fn node_kind__existing_and_missing__some_and_none() -> Result<(), Box<dyn std::error::Error>> {
    let store = common::setup_store()?;
    {
        let conn = store.connection().lock()?;
        insert_memory_node(&conn, "mem-1")?;
        conn.execute(
            "INSERT INTO graph_node (kind, external_id) VALUES ('session', ?1)",
            ["sess-1"],
        )?;
    }

    let vault = GraphVault::new(store.connection().clone());
    let search = GraphSearch::new(&vault);
    assert_eq!(search.node_kind("mem-1")?.as_deref(), Some("memory"));
    assert_eq!(search.node_kind("sess-1")?.as_deref(), Some("session"));
    assert_eq!(search.node_kind("missing-id")?, None);
    Ok(())
}
