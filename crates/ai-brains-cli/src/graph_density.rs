//! Graph density honesty (T213) — pure assessor + SQL gather.
//!
//! Capture-independent: rusqlite COUNT only; no `ai-brains-graph`, models, or embeddings.
//! Shared by `graph update` (graph-on) and doctor `graph_density` soft check.

use rusqlite::Connection;
use std::env;

/// Default minimum pinned memories before empty_lag can fire (small vault skip below).
pub const MIN_PINNED: i64 = 100;
/// Default minimum node count before orphan/sparse ratio arms apply.
pub const MIN_NODES: i64 = 50;
/// Typed-lineage floor: mean edges per node. **Not** 1.0 (directed trees always E/N &lt; 1).
pub const MIN_EDGE_NODE_RATIO: f64 = 0.50;
/// Severe projection-coverage floor (`kind = 'memory'` nodes / pinned).
pub const MIN_MEMORY_COVERAGE: f64 = 0.10;

const ENV_MIN_PINNED: &str = "AI_BRAINS_GRAPH_MIN_PINNED";
const ENV_MIN_NODES: &str = "AI_BRAINS_GRAPH_MIN_NODES";
const ENV_MIN_EDGE_RATIO: &str = "AI_BRAINS_GRAPH_MIN_EDGE_RATIO";
const ENV_MIN_MEMORY_COVERAGE: &str = "AI_BRAINS_GRAPH_MIN_MEMORY_COVERAGE";

/// Counts gathered for density assessment (no I/O inside assessor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDensitySnapshot {
    pub nodes: i64,
    pub edges: i64,
    pub pinned_memories: i64,
    /// `None` omits the projection_lag arm (`kind = 'memory'` query failed or skipped).
    pub memory_nodes: Option<i64>,
}

/// Primary density verdict (single primary reason; F11 priority).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityVerdict {
    Ok,
    Skip,
    EmptyLag,
    OrphanNodes,
    Sparse,
    ProjectionLag,
}

/// Full assessment shared by graph update + doctor.
#[derive(Debug, Clone, PartialEq)]
pub struct Assessment {
    pub verdict: DensityVerdict,
    /// `ok` | `warn` | `skip` (never `fail` — query errors use error path).
    pub density: &'static str,
    /// `live` | `sparse` | `empty` for graph update status.
    pub status: &'static str,
    pub message: String,
    pub note: String,
    pub remediation: Option<String>,
    pub edge_node_ratio: f64,
}

/// Result of SQL gather on a held vault connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatherResult {
    /// `graph_node` and/or `graph_edge` absent from sqlite_master.
    TablesMissing,
    /// Node/edge counts succeeded; pinned COUNT failed (doctor must skip, not empty_lag).
    PinnedCountFailed {
        nodes: i64,
        edges: i64,
        memory_nodes: Option<i64>,
    },
    Ok(GraphDensitySnapshot),
}

/// Mean edges per node; `0.0` when `nodes == 0`.
pub fn edge_node_ratio(nodes: i64, edges: i64) -> f64 {
    if nodes > 0 {
        edges as f64 / nodes as f64
    } else {
        0.0
    }
}

fn parse_i64_env(key: &str, default: i64) -> i64 {
    match env::var(key) {
        Ok(raw) => match raw.trim().parse::<i64>() {
            Ok(v) if v >= 0 => v,
            _ => default,
        },
        Err(_) => default,
    }
}

fn parse_f64_env(key: &str, default: f64) -> f64 {
    match env::var(key) {
        Ok(raw) => match raw.trim().parse::<f64>() {
            Ok(v) if v.is_finite() && v >= 0.0 => v,
            _ => default,
        },
        Err(_) => default,
    }
}

/// Effective MIN_PINNED (env override; invalid → default).
pub fn threshold_min_pinned() -> i64 {
    parse_i64_env(ENV_MIN_PINNED, MIN_PINNED)
}

/// Effective MIN_NODES (env override; invalid → default).
pub fn threshold_min_nodes() -> i64 {
    parse_i64_env(ENV_MIN_NODES, MIN_NODES)
}

/// Effective MIN_EDGE_NODE_RATIO (env override; invalid → default).
pub fn threshold_min_edge_ratio() -> f64 {
    parse_f64_env(ENV_MIN_EDGE_RATIO, MIN_EDGE_NODE_RATIO)
}

/// Effective MIN_MEMORY_COVERAGE (env override; invalid → default).
pub fn threshold_min_memory_coverage() -> f64 {
    parse_f64_env(ENV_MIN_MEMORY_COVERAGE, MIN_MEMORY_COVERAGE)
}

fn format_ratio(ratio: f64) -> String {
    format!("{ratio:.3}")
}

fn counts_suffix(snap: &GraphDensitySnapshot, ratio: f64) -> String {
    let mem = match snap.memory_nodes {
        Some(m) => format!(" memory_nodes={m}"),
        None => String::new(),
    };
    format!(
        "nodes={} edges={} E/N={} pinned={}{mem}",
        snap.nodes,
        snap.edges,
        format_ratio(ratio),
        snap.pinned_memories
    )
}

const REMEDIATION_REBUILD: &str = "ai-brains graph rebuild";

/// Empty-lag remediation: rebuild primary + install SOOT substring (T222 F27; branching → T232).
fn remediation_empty_lag() -> String {
    format!(
        "ai-brains graph rebuild (if graph CLI unavailable: {})",
        crate::commands::governed_common::GRAPH_REINSTALL_SOOT
    )
}

/// Pure density assessment. Thresholds read from env with invalid→default (F17).
///
/// Priority (F11): empty_lag → orphan_nodes → sparse → projection_lag → small skip → Ok.
pub fn assess_graph_density(snap: &GraphDensitySnapshot) -> Assessment {
    let min_pinned = threshold_min_pinned();
    let min_nodes = threshold_min_nodes();
    let min_ratio = threshold_min_edge_ratio();
    let min_coverage = threshold_min_memory_coverage();
    let ratio = edge_node_ratio(snap.nodes, snap.edges);
    let suffix = counts_suffix(snap, ratio);

    // 1. empty_lag
    if snap.pinned_memories >= min_pinned && snap.nodes == 0 && snap.edges == 0 {
        let message = format!("empty_lag: vault has pinned memories but graph is empty ({suffix})");
        return Assessment {
            verdict: DensityVerdict::EmptyLag,
            density: "warn",
            status: "empty",
            note: format!("{message}; run graph rebuild (graph-on install if needed)"),
            remediation: Some(remediation_empty_lag()),
            message,
            edge_node_ratio: ratio,
        };
    }

    // 2. orphan_nodes
    if snap.nodes >= min_nodes && snap.edges == 0 {
        let message = format!("orphan_nodes: many nodes with zero edges ({suffix})");
        return Assessment {
            verdict: DensityVerdict::OrphanNodes,
            density: "warn",
            status: "sparse",
            note: format!("{message}; run graph rebuild"),
            remediation: Some(REMEDIATION_REBUILD.into()),
            message,
            edge_node_ratio: ratio,
        };
    }

    // 3. sparse (typed-lineage under-link floor)
    if snap.nodes >= min_nodes && snap.edges > 0 && ratio < min_ratio {
        let message =
            format!("sparse: edge/node ratio below typed-lineage floor {min_ratio} ({suffix})");
        return Assessment {
            verdict: DensityVerdict::Sparse,
            density: "warn",
            status: "sparse",
            note: format!("{message}; rebuild if projection lag suspected"),
            remediation: Some(REMEDIATION_REBUILD.into()),
            message,
            edge_node_ratio: ratio,
        };
    }

    // 4. projection_lag (kind=`memory` only; omit when memory_nodes unknown or 0)
    if let Some(memory_nodes) = snap.memory_nodes
        && snap.pinned_memories >= min_pinned
        && memory_nodes > 0
        && (memory_nodes as f64 / snap.pinned_memories as f64) < min_coverage
    {
        let coverage = memory_nodes as f64 / snap.pinned_memories as f64;
        let message = format!(
            "projection_lag: memory_nodes/pinned={:.3} below severe floor {min_coverage} ({suffix})",
            coverage
        );
        return Assessment {
            verdict: DensityVerdict::ProjectionLag,
            density: "warn",
            status: "sparse",
            note: format!("{message}; run graph rebuild"),
            remediation: Some(REMEDIATION_REBUILD.into()),
            message,
            edge_node_ratio: ratio,
        };
    }

    // 5. small / empty vault skip (no warn)
    if snap.pinned_memories < min_pinned && snap.nodes == 0 && snap.edges == 0 {
        let message = format!("small vault: empty graph below pin threshold ({suffix})");
        return Assessment {
            verdict: DensityVerdict::Skip,
            density: "skip",
            status: "live",
            note: message.clone(),
            remediation: None,
            message,
            edge_node_ratio: ratio,
        };
    }

    // 6. Ok
    let message = format!("graph density ok ({suffix})");
    Assessment {
        verdict: DensityVerdict::Ok,
        density: "ok",
        status: "live",
        note: format!(
            "{message}. Graph updates incrementally on event append; use 'graph rebuild' for full resync."
        ),
        remediation: None,
        message,
        edge_node_ratio: ratio,
    }
}

/// True when both `graph_node` and `graph_edge` exist (sqlite_master; same pattern as `has_core_tables`).
pub fn has_graph_tables(conn: &Connection) -> bool {
    let has_node = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name = 'graph_node' LIMIT 1",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    let has_edge = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name = 'graph_edge' LIMIT 1",
            [],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    has_node && has_edge
}

fn count_star(conn: &Connection, sql: &str) -> Result<i64, String> {
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|e| format!("count query failed: {e}"))
}

/// Gather density snapshot on an **already-held** connection (no double-open).
///
/// - Tables missing → [`GatherResult::TablesMissing`]
/// - Node/edge COUNT error → `Err` (fail-closed for graph update)
/// - Pinned COUNT error → [`GatherResult::PinnedCountFailed`] (doctor skip)
/// - `memory_nodes` query fail → `None` (omit coverage arm)
pub fn gather_density_snapshot(conn: &Connection) -> Result<GatherResult, String> {
    if !has_graph_tables(conn) {
        return Ok(GatherResult::TablesMissing);
    }

    let nodes = count_star(conn, "SELECT COUNT(*) FROM graph_node")?;
    let edges = count_star(conn, "SELECT COUNT(*) FROM graph_edge")?;

    let memory_nodes = conn
        .query_row(
            "SELECT COUNT(*) FROM graph_node WHERE kind = 'memory'",
            [],
            |row| row.get(0),
        )
        .ok();

    let pinned = match conn.query_row(
        "SELECT COUNT(*) FROM memory_projection WHERE status = 'pinned'",
        [],
        |row| row.get(0),
    ) {
        Ok(n) => n,
        Err(_) => {
            return Ok(GatherResult::PinnedCountFailed {
                nodes,
                edges,
                memory_nodes,
            });
        }
    };

    Ok(GatherResult::Ok(GraphDensitySnapshot {
        nodes,
        edges,
        pinned_memories: pinned,
        memory_nodes,
    }))
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, non_snake_case)]
mod tests {
    use super::*;
    use ai_brains_core::temp_env::TempEnv;

    fn snap(
        nodes: i64,
        edges: i64,
        pinned: i64,
        memory_nodes: Option<i64>,
    ) -> GraphDensitySnapshot {
        GraphDensitySnapshot {
            nodes,
            edges,
            pinned_memories: pinned,
            memory_nodes,
        }
    }

    #[test]
    fn assess_graph_density__empty_lag_pinned500_nodes0__warn_empty() {
        // AC1
        let a = assess_graph_density(&snap(0, 0, 500, Some(0)));
        assert_eq!(a.verdict, DensityVerdict::EmptyLag);
        assert_eq!(a.density, "warn");
        assert_eq!(a.status, "empty");
        assert!(a.message.contains("empty_lag"), "msg={}", a.message);
        assert!(
            a.remediation
                .as_deref()
                .is_some_and(|r| r.contains("graph rebuild")),
            "remediation={:?}",
            a.remediation
        );
        assert!(
            a.remediation.as_deref().is_some_and(|r| {
                r.contains(crate::commands::governed_common::GRAPH_REINSTALL_SOOT)
            }),
            "empty-lag remediation must include GRAPH_REINSTALL_SOOT; remediation={:?}",
            a.remediation
        );
        assert!(!a.message.contains("x'"), "no secrets: {}", a.message);
    }

    #[test]
    fn assess_graph_density__small_empty_pinned10__skip() {
        // AC2
        let a = assess_graph_density(&snap(0, 0, 10, None));
        assert_eq!(a.verdict, DensityVerdict::Skip);
        assert_eq!(a.density, "skip");
        assert_eq!(a.status, "live");
        assert!(a.remediation.is_none());
    }

    #[test]
    fn assess_graph_density__orphan_nodes200_edges0__warn_sparse() {
        // AC3
        let a = assess_graph_density(&snap(200, 0, 50, None));
        assert_eq!(a.verdict, DensityVerdict::OrphanNodes);
        assert_eq!(a.density, "warn");
        assert_eq!(a.status, "sparse");
        assert!(a.message.contains("orphan_nodes"), "msg={}", a.message);
        assert!(
            a.remediation
                .as_deref()
                .is_some_and(|r| r.contains("rebuild")),
            "orphan remediation must mention rebuild: {:?}",
            a.remediation
        );
    }

    #[test]
    fn assess_graph_density__live_like_1304_95__warn_sparse() {
        // AC4 — E/N ≈ 0.073 < 0.50; AC9 remediation mentions rebuild
        let a = assess_graph_density(&snap(1304, 95, 8398, Some(500)));
        assert_eq!(a.verdict, DensityVerdict::Sparse);
        assert_eq!(a.density, "warn");
        assert_eq!(a.status, "sparse");
        assert!(a.message.contains("sparse"), "msg={}", a.message);
        assert!((a.edge_node_ratio - (95.0 / 1304.0)).abs() < 1e-9);
        assert!(
            a.remediation
                .as_deref()
                .is_some_and(|r| r.contains("rebuild")),
            "sparse remediation must mention rebuild: {:?}",
            a.remediation
        );
    }

    #[test]
    fn assess_graph_density__small_ok_nodes10_edges5__ok_live() {
        // AC5 — below MIN_NODES
        let a = assess_graph_density(&snap(10, 5, 5, Some(5)));
        assert_eq!(a.verdict, DensityVerdict::Ok);
        assert_eq!(a.density, "ok");
        assert_eq!(a.status, "live");
        assert!(a.remediation.is_none());
        assert!(a.note.contains("Graph"), "note kept: {}", a.note);
    }

    #[test]
    fn assess_graph_density__ratio_0_8__ok() {
        // AC6 — tree-healthy canary (0.8 ≥ 0.50)
        let a = assess_graph_density(&snap(100, 80, 50, Some(40)));
        assert_eq!(a.verdict, DensityVerdict::Ok);
        assert_eq!(a.density, "ok");
        assert_eq!(a.status, "live");
    }

    #[test]
    fn assess_graph_density__ratio_0_4__warn_sparse() {
        // AC6b
        let a = assess_graph_density(&snap(100, 40, 50, None));
        assert_eq!(a.verdict, DensityVerdict::Sparse);
        assert_eq!(a.density, "warn");
        assert_eq!(a.status, "sparse");
    }

    #[test]
    fn assess_graph_density__projection_lag_memory_coverage__warn() {
        // AC7 — memory_nodes/pinned = 50/1000 = 0.05 < 0.10; dense enough otherwise
        let a = assess_graph_density(&snap(100, 80, 1000, Some(50)));
        assert_eq!(a.verdict, DensityVerdict::ProjectionLag);
        assert_eq!(a.density, "warn");
        assert_eq!(a.status, "sparse");
        assert!(a.message.contains("projection_lag"), "msg={}", a.message);
    }

    #[test]
    fn assess_graph_density__projection_lag_omitted_when_memory_nodes_none() {
        let a = assess_graph_density(&snap(100, 80, 1000, None));
        assert_eq!(
            a.verdict,
            DensityVerdict::Ok,
            "coverage arm omitted when memory_nodes unknown"
        );
    }

    #[test]
    fn assess_graph_density__projection_lag_skipped_when_memory_nodes_zero() {
        // kinds may be session/turn only
        let a = assess_graph_density(&snap(100, 80, 1000, Some(0)));
        assert_eq!(a.verdict, DensityVerdict::Ok);
    }

    #[test]
    fn assess_graph_density__priority_empty_lag_before_orphan() {
        // empty graph with large pins is empty_lag (nodes=0), not orphan
        let a = assess_graph_density(&snap(0, 0, 200, Some(0)));
        assert_eq!(a.verdict, DensityVerdict::EmptyLag);
    }

    #[test]
    fn assess_graph_density__priority_orphan_before_sparse() {
        // edges==0 → orphan, not sparse
        let a = assess_graph_density(&snap(100, 0, 10, None));
        assert_eq!(a.verdict, DensityVerdict::OrphanNodes);
    }

    #[test]
    fn assess_graph_density__priority_sparse_before_projection_lag() {
        // ratio 0.1 sparse wins even if coverage also bad
        let a = assess_graph_density(&snap(100, 10, 1000, Some(10)));
        assert_eq!(a.verdict, DensityVerdict::Sparse);
    }

    #[test]
    fn edge_node_ratio__nodes_zero__zero() {
        assert_eq!(edge_node_ratio(0, 5), 0.0);
        assert!((edge_node_ratio(10, 5) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn threshold_env__invalid_falls_back_to_defaults() {
        let _a = TempEnv::set(ENV_MIN_PINNED, "not-a-number");
        let _b = TempEnv::set(ENV_MIN_NODES, "-1");
        let _c = TempEnv::set(ENV_MIN_EDGE_RATIO, "nan");
        let _d = TempEnv::set(ENV_MIN_MEMORY_COVERAGE, "");
        assert_eq!(threshold_min_pinned(), MIN_PINNED);
        assert_eq!(threshold_min_nodes(), MIN_NODES);
        assert!((threshold_min_edge_ratio() - MIN_EDGE_NODE_RATIO).abs() < 1e-12);
        assert!((threshold_min_memory_coverage() - MIN_MEMORY_COVERAGE).abs() < 1e-12);
    }

    #[test]
    fn threshold_env__valid_override_changes_verdict() {
        // Force MIN_NODES very high so 100 nodes stays Ok
        let _g = TempEnv::set(ENV_MIN_NODES, "1000");
        let a = assess_graph_density(&snap(100, 10, 50, None));
        assert_eq!(
            a.verdict,
            DensityVerdict::Ok,
            "high MIN_NODES should suppress sparse"
        );
    }

    #[test]
    fn has_graph_tables__empty_db__false() {
        let conn = Connection::open_in_memory().expect("memdb");
        assert!(!has_graph_tables(&conn));
    }

    #[test]
    fn has_graph_tables__both_tables__true() {
        let conn = Connection::open_in_memory().expect("memdb");
        conn.execute_batch(
            "CREATE TABLE graph_node (id TEXT);
             CREATE TABLE graph_edge (id TEXT);",
        )
        .expect("create");
        assert!(has_graph_tables(&conn));
    }

    #[test]
    fn gather_density_snapshot__tables_missing__tables_missing_variant() {
        // AC11 shape
        let conn = Connection::open_in_memory().expect("memdb");
        let r = gather_density_snapshot(&conn).expect("ok result");
        assert_eq!(r, GatherResult::TablesMissing);
    }

    #[test]
    fn gather_density_snapshot__pinned_table_missing__pinned_count_failed() {
        // AC12 shape — graph tables exist; memory_projection absent → pinned fail skip path
        let conn = Connection::open_in_memory().expect("memdb");
        conn.execute_batch(
            "CREATE TABLE graph_node (id TEXT, kind TEXT);
             CREATE TABLE graph_edge (id TEXT);
             INSERT INTO graph_node (id, kind) VALUES ('n1', 'memory');
             INSERT INTO graph_edge (id) VALUES ('e1');",
        )
        .expect("seed");
        let r = gather_density_snapshot(&conn).expect("ok result");
        match r {
            GatherResult::PinnedCountFailed {
                nodes,
                edges,
                memory_nodes,
            } => {
                assert_eq!(nodes, 1);
                assert_eq!(edges, 1);
                assert_eq!(memory_nodes, Some(1));
            }
            other => panic!("expected PinnedCountFailed, got {other:?}"),
        }
    }

    #[test]
    fn gather_density_snapshot__full_tables__ok_snapshot() {
        let conn = Connection::open_in_memory().expect("memdb");
        conn.execute_batch(
            "CREATE TABLE graph_node (id TEXT, kind TEXT);
             CREATE TABLE graph_edge (id TEXT);
             CREATE TABLE memory_projection (status TEXT);
             INSERT INTO graph_node (id, kind) VALUES ('n1', 'memory'), ('n2', 'session');
             INSERT INTO graph_edge (id) VALUES ('e1');
             INSERT INTO memory_projection (status) VALUES ('pinned'), ('pinned'), ('archived');",
        )
        .expect("seed");
        let r = gather_density_snapshot(&conn).expect("ok result");
        match r {
            GatherResult::Ok(s) => {
                assert_eq!(s.nodes, 2);
                assert_eq!(s.edges, 1);
                assert_eq!(s.pinned_memories, 2);
                assert_eq!(s.memory_nodes, Some(1));
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn assess_graph_density__messages_have_no_secret_markers() {
        // AC17 string deny
        let cases = [
            snap(0, 0, 500, None),
            snap(200, 0, 10, None),
            snap(1304, 95, 100, Some(5)),
            snap(100, 80, 1000, Some(20)),
        ];
        for s in cases {
            let a = assess_graph_density(&s);
            for text in [
                a.message.as_str(),
                a.note.as_str(),
                a.remediation.as_deref().unwrap_or(""),
            ] {
                assert!(
                    !text.contains("x'")
                        && !text.contains("AI_BRAINS_KEY")
                        && !text.to_ascii_lowercase().contains("passphrase"),
                    "secret-like token in density text: {text}"
                );
            }
        }
    }
}
