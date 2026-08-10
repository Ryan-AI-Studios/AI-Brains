use crate::context::AppContext;
use crate::graph_density::{
    GatherResult, GraphDensitySnapshot, assess_graph_density, gather_density_snapshot,
};
use ai_brains_graph::{GraphRebuilder, GraphSearch, GraphVault};
use ai_brains_store::SqliteEventStore;
use serde::Serialize;

#[derive(Serialize)]
struct NeighborsOutput<'a> {
    memory_id: &'a str,
    neighbors: Vec<ai_brains_graph::queries::NeighborHit>,
}

#[derive(Serialize)]
struct HierarchyOutput<'a> {
    root: &'a str,
    synthesized_from: Vec<String>,
}

#[derive(Serialize)]
struct SessionOutput<'a> {
    session_id: &'a str,
    memories: Vec<String>,
}

/// Graph health report (`graph update`). T213 expands density fields; keeps `note` (M3).
#[derive(Serialize)]
struct GraphHealthOutput {
    nodes: i64,
    edges: i64,
    pinned_memories: i64,
    memory_nodes: i64,
    edge_node_ratio: f64,
    /// `ok` | `warn` | `skip` (never `fail` on this field).
    density: &'static str,
    /// `live` | `sparse` | `empty`
    status: &'static str,
    note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    remediation: Option<String>,
}

pub fn rebuild(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("[graph] Starting graph rebuild...");

    let event_store = SqliteEventStore::new((*ctx.conn).clone());
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let rebuilder = GraphRebuilder::new(&graph_vault, &event_store);

    rebuilder.rebuild()?;

    tracing::info!("[graph] Rebuild complete.");
    Ok(())
}

pub fn neighbors(ctx: &AppContext, memory_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let neighbors = searcher.get_neighbors(memory_id)?;

    let output = NeighborsOutput {
        memory_id,
        neighbors,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn hierarchy(ctx: &AppContext, memory_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let synthesized = searcher.get_synthesized_hierarchy(memory_id)?;

    let output = HierarchyOutput {
        root: memory_id,
        synthesized_from: synthesized,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn session(ctx: &AppContext, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let memories = searcher.get_session_memories(session_id)?;

    let output = SessionOutput {
        session_id,
        memories,
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

pub fn update(ctx: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {}", e))?;

    // Fail-closed on node/edge COUNT errors (audit2 F8); density via shared pure assessor (F32).
    let gather = gather_density_snapshot(&conn)
        .map_err(|e| format!("Failed to gather graph density: {e}"))?;

    let (snap, pinned_for_json, memory_for_json) = match gather {
        GatherResult::TablesMissing => {
            return Err(
                "Failed to count graph nodes/edges: graph tables missing (run migrate / init)"
                    .into(),
            );
        }
        GatherResult::PinnedCountFailed {
            nodes,
            edges,
            memory_nodes,
        } => {
            // Still report structural density; pinned unknown → 0 field, omit false empty_lag.
            let s = GraphDensitySnapshot {
                nodes,
                edges,
                pinned_memories: 0,
                memory_nodes,
            };
            let mem_json = memory_nodes.unwrap_or(0);
            (s, 0_i64, mem_json)
        }
        GatherResult::Ok(s) => {
            let mem_json = s.memory_nodes.unwrap_or(0);
            let pinned = s.pinned_memories;
            (s, pinned, mem_json)
        }
    };

    let assessment = assess_graph_density(&snap);

    let report = GraphHealthOutput {
        nodes: snap.nodes,
        edges: snap.edges,
        pinned_memories: pinned_for_json,
        memory_nodes: memory_for_json,
        edge_node_ratio: assessment.edge_node_ratio,
        density: assessment.density,
        status: assessment.status,
        note: assessment.note,
        remediation: assessment.remediation,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::GraphHealthOutput;
    use crate::graph_density::{
        GraphDensitySnapshot, REMEDIATION_REBUILD, assess_graph_density_with,
    };

    /// T213 AC8: success JSON shape includes expanded density fields + note.
    #[test]
    fn graph_health_output__serde_keys__include_density_fields() {
        let snap = GraphDensitySnapshot {
            nodes: 10,
            edges: 5,
            pinned_memories: 5,
            memory_nodes: Some(5),
        };
        let a = assess_graph_density_with(&snap, true);
        assert_eq!(a.status, "live");
        assert_eq!(a.density, "ok");

        let report = GraphHealthOutput {
            nodes: snap.nodes,
            edges: snap.edges,
            pinned_memories: snap.pinned_memories,
            memory_nodes: snap.memory_nodes.unwrap_or(0),
            edge_node_ratio: a.edge_node_ratio,
            density: a.density,
            status: a.status,
            note: a.note,
            remediation: a.remediation,
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
                .expect("json");
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
            assert!(v.get(key).is_some(), "missing key {key} in {v}");
        }
        assert!(v.get("remediation").is_none(), "ok path omits remediation");
        assert!(matches!(
            v["status"].as_str(),
            Some("live" | "sparse" | "empty")
        ));
        assert!(matches!(
            v["density"].as_str(),
            Some("ok" | "warn" | "skip")
        ));
    }

    /// T213 AC9 / T232 L3: sparse fixture maps status/density + exact rebuild remediation.
    #[test]
    fn graph_health_output__sparse_fixture__status_sparse_with_remediation() {
        let snap = GraphDensitySnapshot {
            nodes: 1304,
            edges: 95,
            pinned_memories: 8398,
            memory_nodes: Some(500),
        };
        let a = assess_graph_density_with(&snap, true);
        assert_eq!(a.remediation.as_deref(), Some(REMEDIATION_REBUILD));
        let report = GraphHealthOutput {
            nodes: snap.nodes,
            edges: snap.edges,
            pinned_memories: snap.pinned_memories,
            memory_nodes: snap.memory_nodes.unwrap_or(0),
            edge_node_ratio: a.edge_node_ratio,
            density: a.density,
            status: a.status,
            note: a.note,
            remediation: a.remediation,
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
                .expect("json");
        assert_eq!(v["status"], "sparse");
        assert_eq!(v["density"], "warn");
        assert_eq!(
            v["remediation"].as_str(),
            Some(REMEDIATION_REBUILD),
            "remediation: {}",
            v["remediation"]
        );
        assert!(!v["note"].as_str().unwrap_or("").is_empty());
    }
}
