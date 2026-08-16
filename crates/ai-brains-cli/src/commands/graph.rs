use crate::context::AppContext;
use crate::graph_density::{
    GatherResult, GraphDensitySnapshot, assess_graph_density, gather_density_snapshot,
};
use ai_brains_control_plane::clamp_list_limit;
use ai_brains_graph::{GraphRebuilder, GraphSearch, GraphVault, NeighborHit};
use ai_brains_store::SqliteEventStore;
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::io::IsTerminal;

const PRETTY_NEXT: &str = "next: ai-brains graph update";

#[derive(Serialize)]
struct NeighborsOutput<'a> {
    memory_id: &'a str,
    neighbors: Vec<NeighborHit>,
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

/// One pretty-neighbors row (kind/preview are pretty-only).
#[derive(Debug, Clone)]
pub(crate) struct PrettyNeighborRow {
    pub direction: String,
    pub label: String,
    pub external_id: String,
    pub kind: String,
    pub preview: String,
}

pub(crate) fn resolve_graph_format(explicit: &str, is_tty: bool) -> &'static str {
    match explicit {
        "pretty" | "human" | "text" => "pretty",
        "json" => "json",
        "auto" if is_tty => "pretty",
        "auto" => "json",
        // Clap value_parser rejects unknowns; fail-closed compact for scripts.
        _ => "json",
    }
}

pub(crate) fn pretty_no_graph_node(id: &str) -> String {
    format!("No graph node for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_no_memory_node(id: &str) -> String {
    format!("No memory node for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_no_session_node(id: &str) -> String {
    format!("No session node for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_no_neighbors(id: &str) -> String {
    format!("No neighbors for {id}.\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_hierarchy_leaf() -> String {
    format!("No SYNTHESIZED_FROM children (leaf).\n{PRETTY_NEXT}")
}

pub(crate) fn pretty_session_empty() -> String {
    format!("No memories in this session via graph edges.\n{PRETTY_NEXT}")
}

fn dir_display(direction: &str) -> &str {
    match direction {
        "incoming" => "in",
        "outgoing" => "out",
        other => other,
    }
}

pub(crate) fn sort_neighbor_hits(hits: &mut [NeighborHit]) {
    hits.sort_by(|a, b| {
        a.direction
            .cmp(&b.direction)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.external_id.cmp(&b.external_id))
    });
}

fn apply_limit(len: usize, limit: usize) -> (usize, Option<usize>) {
    if len > limit {
        (limit, Some(len - limit))
    } else {
        (len, None)
    }
}

pub(crate) fn format_neighbors_pretty(
    memory_id: &str,
    rows: &[PrettyNeighborRow],
    limit: usize,
) -> String {
    let total = rows.len();
    let (shown, more) = apply_limit(total, limit);
    let mut out = format!("Neighbors of {memory_id} ({total})\n");
    out.push_str(&format!(
        "{:<3} {:<16} {:<36} {:<14} {}\n",
        "DIR", "LABEL", "ID", "KIND", "PREVIEW"
    ));
    for row in rows.iter().take(shown) {
        out.push_str(&format!(
            "{:<3} {:<16} {:<36} {:<14} {}\n",
            dir_display(&row.direction),
            row.label,
            row.external_id,
            row.kind,
            row.preview
        ));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    out
}

pub(crate) fn format_neighbors_json(
    memory_id: &str,
    neighbors: &[NeighborHit],
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&NeighborsOutput {
        memory_id,
        neighbors: neighbors.to_vec(),
    })
}

pub(crate) fn format_hierarchy_pretty(root: &str, rows: &[(String, i64)], limit: usize) -> String {
    let total = rows.len();
    let (shown, more) = apply_limit(total, limit);
    let mut out = format!("Hierarchy of {root} ({total})\n");
    for (id, depth) in rows.iter().take(shown) {
        let indent = " ".repeat(2 * (*depth).max(0) as usize);
        out.push_str(&format!("{indent}{id}\n"));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    out
}

pub(crate) fn format_session_pretty(
    session_id: &str,
    rows: &[(String, String)],
    limit: usize,
) -> String {
    let total = rows.len();
    let (shown, more) = apply_limit(total, limit);
    let mut out = format!("Memories in session {session_id} ({total})\n");
    for (id, preview) in rows.iter().take(shown) {
        out.push_str(&format!("{id} {preview}\n"));
    }
    if let Some(n) = more {
        out.push_str(&format!("… and {n} more\n"));
    }
    out
}

fn json_take(limit: Option<usize>) -> usize {
    if limit.is_some() {
        clamp_list_limit(limit)
    } else {
        usize::MAX
    }
}

fn memory_preview(ctx: &AppContext, memory_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let conn = ctx
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock vault: {e}"))?;
    let content: Option<String> = conn
        .query_row(
            "SELECT content FROM memory_projection WHERE memory_id = ?1 LIMIT 1",
            [memory_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Failed to load memory preview: {e}"))?;
    Ok(content.map_or_else(String::new, |c| {
        crate::commands::memory::preview_line(&c, 80)
    }))
}

fn pretty_neighbor_rows(
    ctx: &AppContext,
    searcher: &GraphSearch<'_>,
    hits: &[NeighborHit],
) -> Result<Vec<PrettyNeighborRow>, Box<dyn std::error::Error>> {
    let mut rows = Vec::with_capacity(hits.len());
    for hit in hits {
        let kind = searcher.node_kind(&hit.external_id)?.unwrap_or_default();
        let preview = if kind == "memory" {
            memory_preview(ctx, &hit.external_id)?
        } else {
            String::new()
        };
        rows.push(PrettyNeighborRow {
            direction: hit.direction.clone(),
            label: hit.label.clone(),
            external_id: hit.external_id.clone(),
            kind,
            preview,
        });
    }
    Ok(rows)
}

fn emit_graph_health_human(report: &GraphHealthOutput) {
    println!("status: {}", report.status);
    println!("density: {}", report.density);
    println!("nodes: {}", report.nodes);
    println!("edges: {}", report.edges);
    println!("pinned_memories: {}", report.pinned_memories);
    println!("memory_nodes: {}", report.memory_nodes);
    println!("edge_node_ratio: {}", report.edge_node_ratio);
    println!("note: {}", report.note);
    if let Some(remediation) = &report.remediation {
        println!("remediation: {remediation}");
    }
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

pub fn neighbors(
    ctx: &AppContext,
    memory_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    let kind = searcher.node_kind(memory_id)?;

    if kind.is_none() {
        if resolved == "pretty" {
            println!("{}", pretty_no_graph_node(memory_id));
        } else {
            println!("{}", format_neighbors_json(memory_id, &[])?);
        }
        return Ok(());
    }

    let mut hits = searcher.get_neighbors(memory_id)?;
    sort_neighbor_hits(&mut hits);

    if resolved == "pretty" {
        if hits.is_empty() {
            println!("{}", pretty_no_neighbors(memory_id));
            return Ok(());
        }
        let rows = pretty_neighbor_rows(ctx, &searcher, &hits)?;
        print!(
            "{}",
            format_neighbors_pretty(memory_id, &rows, clamp_list_limit(limit))
        );
        return Ok(());
    }

    let take = json_take(limit);
    if hits.len() > take {
        hits.truncate(take);
    }
    println!("{}", format_neighbors_json(memory_id, &hits)?);
    Ok(())
}

pub fn hierarchy(
    ctx: &AppContext,
    memory_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    let kind = searcher.node_kind(memory_id)?;

    match kind.as_deref() {
        None => {
            if resolved == "pretty" {
                println!("{}", pretty_no_graph_node(memory_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&HierarchyOutput {
                        root: memory_id,
                        synthesized_from: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
        Some("memory") => {}
        Some(_) => {
            if resolved == "pretty" {
                println!("{}", pretty_no_memory_node(memory_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&HierarchyOutput {
                        root: memory_id,
                        synthesized_from: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
    }

    if resolved == "pretty" {
        let rows = searcher.get_synthesized_hierarchy_with_depth(memory_id)?;
        if rows.is_empty() {
            println!("{}", pretty_hierarchy_leaf());
            return Ok(());
        }
        print!(
            "{}",
            format_hierarchy_pretty(memory_id, &rows, clamp_list_limit(limit))
        );
        return Ok(());
    }

    let mut synthesized = searcher.get_synthesized_hierarchy(memory_id)?;
    synthesized.sort();
    let take = json_take(limit);
    if synthesized.len() > take {
        synthesized.truncate(take);
    }
    println!(
        "{}",
        serde_json::to_string(&HierarchyOutput {
            root: memory_id,
            synthesized_from: synthesized,
        })?
    );
    Ok(())
}

pub fn session(
    ctx: &AppContext,
    session_id: &str,
    format: &str,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_vault = GraphVault::new((*ctx.conn).clone());
    let searcher = GraphSearch::new(&graph_vault);
    let resolved = resolve_graph_format(format, std::io::stdout().is_terminal());
    let kind = searcher.node_kind(session_id)?;

    match kind.as_deref() {
        None => {
            if resolved == "pretty" {
                println!("{}", pretty_no_graph_node(session_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&SessionOutput {
                        session_id,
                        memories: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
        Some("session") => {}
        Some(_) => {
            if resolved == "pretty" {
                println!("{}", pretty_no_session_node(session_id));
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&SessionOutput {
                        session_id,
                        memories: Vec::new(),
                    })?
                );
            }
            return Ok(());
        }
    }

    let mut memories = searcher.get_session_memories(session_id)?;
    memories.sort();

    if resolved == "pretty" {
        if memories.is_empty() {
            println!("{}", pretty_session_empty());
            return Ok(());
        }
        let mut rows = Vec::with_capacity(memories.len());
        for id in &memories {
            let preview = memory_preview(ctx, id)?;
            rows.push((id.clone(), preview));
        }
        print!(
            "{}",
            format_session_pretty(session_id, &rows, clamp_list_limit(limit))
        );
        return Ok(());
    }

    let take = json_take(limit);
    if memories.len() > take {
        memories.truncate(take);
    }
    println!(
        "{}",
        serde_json::to_string(&SessionOutput {
            session_id,
            memories,
        })?
    );
    Ok(())
}

pub fn update(ctx: &AppContext, format: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    if format == "human" {
        emit_graph_health_human(&report);
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
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

    #[test]
    fn resolve_graph_format__auto_tty__pretty() {
        assert_eq!(resolve_graph_format("auto", true), "pretty");
    }

    #[test]
    fn resolve_graph_format__auto_pipe__json() {
        assert_eq!(resolve_graph_format("auto", false), "json");
    }

    #[test]
    fn resolve_graph_format__pretty_human_text__pretty_regardless_of_tty() {
        for token in ["pretty", "human", "text"] {
            assert_eq!(resolve_graph_format(token, true), "pretty", "{token} tty");
            assert_eq!(resolve_graph_format(token, false), "pretty", "{token} pipe");
        }
    }

    #[test]
    fn resolve_graph_format__json__json_regardless_of_tty() {
        assert_eq!(resolve_graph_format("json", true), "json");
        assert_eq!(resolve_graph_format("json", false), "json");
    }

    fn fixture_neighbor_rows() -> Vec<PrettyNeighborRow> {
        vec![
            PrettyNeighborRow {
                direction: "incoming".into(),
                label: "RECALLS".into(),
                external_id: "3b4e95b8-a011-48a8-b5ea-72e36c6a2458".into(),
                kind: "session".into(),
                preview: String::new(),
            },
            PrettyNeighborRow {
                direction: "outgoing".into(),
                label: "SYNTHESIZED_FROM".into(),
                external_id: "5a0e0a71-1ee7-445b-84a9-aa06fe499c2e".into(),
                kind: "memory".into(),
                preview: "child preview".into(),
            },
        ]
    }

    #[test]
    fn format_neighbors_pretty__incoming_and_outgoing__header_in_out_kinds() {
        let text = format_neighbors_pretty("root-id", &fixture_neighbor_rows(), 50);
        assert!(text.starts_with("Neighbors of root-id (2)"));
        assert!(text.contains("DIR"));
        assert!(text.contains("LABEL"));
        assert!(text.contains("KIND"));
        assert!(text.contains("PREVIEW"));
        assert!(text.contains("in "));
        assert!(text.contains("out"));
        assert!(text.contains("RECALLS"));
        assert!(text.contains("SYNTHESIZED_FROM"));
        assert!(text.contains("3b4e95b8-a011-48a8-b5ea-72e36c6a2458"));
        assert!(text.contains("5a0e0a71-1ee7-445b-84a9-aa06fe499c2e"));
        assert!(text.contains("session"));
        assert!(text.contains("memory"));
        assert!(!text.contains("incoming"));
        assert!(!text.contains("outgoing"));
    }

    #[test]
    fn format_neighbors_json__fixture__keeps_incoming_outgoing_external_id() {
        let hits = vec![
            NeighborHit {
                external_id: "3b4e95b8-a011-48a8-b5ea-72e36c6a2458".into(),
                label: "RECALLS".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "5a0e0a71-1ee7-445b-84a9-aa06fe499c2e".into(),
                label: "SYNTHESIZED_FROM".into(),
                direction: "outgoing".into(),
            },
        ];
        let raw = format_neighbors_json("root-id", &hits).expect("json");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(v["memory_id"], "root-id");
        let obj_keys: Vec<&str> = v
            .as_object()
            .expect("obj")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(obj_keys, vec!["memory_id", "neighbors"]);
        let n0 = &v["neighbors"][0];
        assert_eq!(n0["direction"], "incoming");
        assert_eq!(n0["external_id"], "3b4e95b8-a011-48a8-b5ea-72e36c6a2458");
        assert_eq!(n0["label"], "RECALLS");
        let n1 = &v["neighbors"][1];
        assert_eq!(n1["direction"], "outgoing");
        let hit_keys: std::collections::BTreeSet<&str> = n0
            .as_object()
            .expect("hit")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            hit_keys,
            std::collections::BTreeSet::from(["external_id", "label", "direction"])
        );
    }

    #[test]
    fn empty_pretty__missing_and_present__exact_f3_and_graph_update() {
        assert_eq!(
            pretty_no_graph_node("abc"),
            "No graph node for abc.\nnext: ai-brains graph update"
        );
        assert_eq!(
            pretty_no_neighbors("abc"),
            "No neighbors for abc.\nnext: ai-brains graph update"
        );
        assert_eq!(
            pretty_no_memory_node("abc"),
            "No memory node for abc.\nnext: ai-brains graph update"
        );
        assert_eq!(
            pretty_no_session_node("abc"),
            "No session node for abc.\nnext: ai-brains graph update"
        );
        assert_eq!(
            pretty_hierarchy_leaf(),
            "No SYNTHESIZED_FROM children (leaf).\nnext: ai-brains graph update"
        );
        assert_eq!(
            pretty_session_empty(),
            "No memories in this session via graph edges.\nnext: ai-brains graph update"
        );
        assert!(pretty_no_graph_node("abc").contains("graph update"));
        assert!(pretty_no_neighbors("abc").contains("graph update"));
    }

    #[test]
    fn format_hierarchy_pretty__depth_1_and_2__indent_2_and_4_no_box() {
        let rows = vec![("child-a".into(), 1_i64), ("child-b".into(), 2_i64)];
        let text = format_hierarchy_pretty("root", &rows, 50);
        assert!(text.contains("  child-a"));
        assert!(text.contains("    child-b"));
        assert!(!text.contains('└'));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[1], "  child-a");
        assert_eq!(lines[2], "    child-b");
    }

    #[test]
    fn format_neighbors_pretty__51_rows_default_limit__50_lines_and_1_more() {
        let rows: Vec<PrettyNeighborRow> = (0..51)
            .map(|i| PrettyNeighborRow {
                direction: "incoming".into(),
                label: "RECALLS".into(),
                external_id: format!("id-{i:02}"),
                kind: "session".into(),
                preview: String::new(),
            })
            .collect();
        let limit = clamp_list_limit(None);
        let text = format_neighbors_pretty("root", &rows, limit);
        let data_lines: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("in ") || l.starts_with("out"))
            .collect();
        assert_eq!(data_lines.len(), 50);
        assert!(text.contains("… and 1 more"));
    }

    #[test]
    fn sort_neighbor_hits__incoming_before_outgoing_then_label_then_id() {
        let mut hits = vec![
            NeighborHit {
                external_id: "z".into(),
                label: "B".into(),
                direction: "outgoing".into(),
            },
            NeighborHit {
                external_id: "b".into(),
                label: "A".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "a".into(),
                label: "B".into(),
                direction: "incoming".into(),
            },
            NeighborHit {
                external_id: "a".into(),
                label: "A".into(),
                direction: "incoming".into(),
            },
        ];
        sort_neighbor_hits(&mut hits);
        let keys: Vec<(&str, &str, &str)> = hits
            .iter()
            .map(|h| {
                (
                    h.direction.as_str(),
                    h.label.as_str(),
                    h.external_id.as_str(),
                )
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                ("incoming", "A", "a"),
                ("incoming", "A", "b"),
                ("incoming", "B", "a"),
                ("outgoing", "B", "z"),
            ]
        );
    }
}
