//! CozoDB proxy backend: translates AI-Brains graph operations into Datalog
//! statements and routes them through BridgeRecord IPC to ChangeGuard's CozoDB.
//!
//! Feature-gated: activates only when `.changeguard/` directory is present.
//! Falls back gracefully to the SQLite graph backend otherwise.

use crate::errors::{GraphError, Result};
use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A node in the AI-Brains knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub category: String,
    pub metadata: serde_json::Value,
}

/// An edge connecting two nodes in the AI-Brains knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: f64,
}

/// A path result from a reachability traversal query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
}

/// Parsed CozoDB response carrying named rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CozoNamedRows {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// GraphBackend trait
// ---------------------------------------------------------------------------

/// Abstraction over graph storage backends. Implementations route mutations
/// and queries to either a local SQLite store or a remote CozoDB instance
/// via the ChangeGuard bridge.
pub trait GraphBackend {
    /// Insert or update a node in the graph.
    fn add_node(
        &self,
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> Result<()>;

    /// Insert or update an edge between two nodes.
    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()>;

    /// Query neighbors of a node, returning (target_id, relation) pairs.
    fn query_neighbors(&self, node_id: &str) -> Result<Vec<(String, String)>>;

    /// Find a path from `from` to `to`, returning the sequence of node IDs.
    fn query_path(&self, from: &str, to: &str) -> Result<Vec<String>>;

    /// Returns true when the backend is available and functional.
    fn is_available(&self) -> bool;
}

// ---------------------------------------------------------------------------
// CozoProxyBackend
// ---------------------------------------------------------------------------

/// Translates AI-Brains graph operations into CozoDB Datalog statements and
/// routes them through the ChangeGuard Bridge IPC (named pipe / CLI).
///
/// When `.changeguard/` is not present in the current working directory or
/// the `changeguard` CLI is unavailable, all operations return
/// `GraphError::DbError` (fail-closed for mutations) so callers can fall
/// back to the SQLite backend.
pub struct CozoProxyBackend {
    changeguard_available: bool,
}

impl CozoProxyBackend {
    /// Create a new CozoProxyBackend. Automatically detects whether
    /// ChangeGuard is available by checking for `.changeguard/` and the
    /// `changeguard` binary. When neither is present the backend marks
    /// itself unavailable so callers can fall back.
    pub fn new(working_dir: Option<PathBuf>) -> Self {
        let cwd = working_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let changeguard_dir = cwd.join(".changeguard");
        let dir_exists = changeguard_dir.exists() && changeguard_dir.is_dir();

        let cli_available = std::process::Command::new("changeguard")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        // Backend is available only when BOTH the directory marker AND the
        // CLI binary are present.
        let available = dir_exists && cli_available;

        tracing::info!(
            changeguard_dir=%changeguard_dir.display(),
            available,
            "CozoProxyBackend initialized"
        );

        Self {
            changeguard_available: available,
        }
    }

    // ------------------------------------------------------------------
    // Datalog translation helpers
    // ------------------------------------------------------------------

    /// Translate `add_node` into a CozoDB `:put` Datalog statement.
    fn datalog_put_node(
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> String {
        let meta_str = serde_json::to_string(metadata).unwrap_or_else(|_| "null".to_string());
        format!(
            "?[id, label, category, metadata] <- [[\"{}\", \"{}\", \"{}\", {}]] :put node",
            escape_datalog_str(id),
            escape_datalog_str(label),
            escape_datalog_str(category),
            meta_str
        )
    }

    /// Translate `add_edge` into a CozoDB `:put` Datalog statement.
    fn datalog_put_edge(source: &str, target: &str, relation: &str, confidence: f64) -> String {
        format!(
            "?[source, target, relation, confidence] <- [[\"{}\", \"{}\", \"{}\", {}]] :put edge",
            escape_datalog_str(source),
            escape_datalog_str(target),
            escape_datalog_str(relation),
            confidence
        )
    }

    /// Translate `query_neighbors` into a CozoDB Datalog query.
    fn datalog_query_neighbors(node_id: &str) -> String {
        format!(
            "?[target, relation] := *edge{{source: \"{}\", target, relation}}",
            escape_datalog_str(node_id)
        )
    }

    /// Translate `query_path` into a CozoDB reachability traversal.
    fn datalog_query_path(from: &str, to: &str) -> String {
        format!(
            "?[path] := *reachable{{source: \"{}\", target: \"{}\", path}}",
            escape_datalog_str(from),
            escape_datalog_str(to)
        )
    }

    // ------------------------------------------------------------------
    // Bridge IPC helpers
    // ------------------------------------------------------------------

    /// Send a Datalog mutation (put) to ChangeGuard via bridge import.
    /// Returns Ok(()) on success or an error describing the failure.
    #[allow(clippy::disallowed_methods)]
    fn send_datalog_mutation(&self, datalog: &str, record_kind: &str) -> Result<()> {
        if !self.changeguard_available {
            return Err(GraphError::DbError(
                "ChangeGuard is not available; CozoProxyBackend cannot route mutations."
                    .to_string(),
            ));
        }

        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let temp_path = temp_file.path().to_path_buf();

        // Build a BridgeRecord carrying the Datalog payload
        let timestamp = chrono::Utc::now();
        let record = BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp,
            parent_hash: None,
            project_id: "ChangeGuard".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: record_kind.to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "datalog": datalog,
            })),
            privacy: ai_brains_core::privacy::Privacy::LocalOnly,
        };

        // Serialize to NDJSON and write to temp file
        let ndjson = serde_json::to_string(&record)
            .map_err(|e| GraphError::DbError(format!("Failed to serialize BridgeRecord: {}", e)))?;
        std::fs::write(&temp_path, ndjson.as_bytes()).map_err(GraphError::IoError)?;

        // Call changeguard bridge import
        let output = std::process::Command::new("changeguard")
            .args([
                "bridge",
                "import",
                "--from",
                temp_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| {
                GraphError::DbError(format!("Failed to invoke changeguard bridge import: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("ChangeGuard bridge import failed: {}", stderr);
            return Err(GraphError::DbError(format!(
                "ChangeGuard bridge import rejected Datalog mutation: {}",
                stderr
            )));
        }

        // Clean up temp file (drop would also delete, but be explicit)
        // NamedTempFile is dropped here automatically.

        Ok(())
    }

    /// Run a Datalog query through the bridge and parse NamedRows response.
    #[allow(clippy::disallowed_methods)]
    fn run_datalog_query(&self, datalog: &str) -> Result<CozoNamedRows> {
        if !self.changeguard_available {
            return Err(GraphError::DbError(
                "ChangeGuard is not available; CozoProxyBackend cannot run queries.".to_string(),
            ));
        }

        // Write query to temp NDJSON file
        let temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let temp_path = temp_file.path().to_path_buf();

        let timestamp = chrono::Utc::now();
        let record = BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp,
            parent_hash: None,
            project_id: "ChangeGuard".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "datalog_query".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "datalog": datalog,
            })),
            privacy: ai_brains_core::privacy::Privacy::LocalOnly,
        };

        let ndjson = serde_json::to_string(&record).map_err(|e| {
            GraphError::DbError(format!("Failed to serialize query BridgeRecord: {}", e))
        })?;
        std::fs::write(&temp_path, ndjson.as_bytes()).map_err(GraphError::IoError)?;

        // Write query output to a separate temp file
        let out_file = tempfile::NamedTempFile::new()
            .map_err(|e| GraphError::IoError(std::io::Error::other(e)))?;
        let out_path = out_file.path().to_path_buf();

        let output = std::process::Command::new("changeguard")
            .args([
                "bridge",
                "export",
                "--out",
                out_path.to_str().unwrap_or(""),
                "--graph-query",
                temp_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| {
                GraphError::DbError(format!("Failed to invoke changeguard bridge export: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GraphError::DbError(format!(
                "ChangeGuard bridge export failed: {}",
                stderr
            )));
        }

        // Parse the first BridgeRecord from the output file to extract NamedRows
        let content = std::fs::read_to_string(&out_path).map_err(GraphError::IoError)?;

        // The output is NDJSON. Parse the first line containing a "named_rows" record.
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<BridgeRecord>(line) {
                if record.record_kind == "named_rows" {
                    if let Ok(rows) =
                        serde_json::from_value::<CozoNamedRows>(record.payload_value())
                    {
                        return Ok(rows);
                    }
                }
            }
        }

        // If no named_rows found, return empty result
        Ok(CozoNamedRows {
            headers: Vec::new(),
            rows: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// GraphBackend impl for CozoProxyBackend
// ---------------------------------------------------------------------------

impl GraphBackend for CozoProxyBackend {
    fn add_node(
        &self,
        id: &str,
        label: &str,
        category: &str,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        let datalog = Self::datalog_put_node(id, label, category, metadata);
        tracing::debug!(id, label, category, "CozoProxyBackend::add_node");
        self.send_datalog_mutation(&datalog, "datalog_put_node")
    }

    fn add_edge(&self, source: &str, target: &str, relation: &str, confidence: f64) -> Result<()> {
        let datalog = Self::datalog_put_edge(source, target, relation, confidence);
        tracing::debug!(
            source,
            target,
            relation,
            confidence,
            "CozoProxyBackend::add_edge"
        );
        self.send_datalog_mutation(&datalog, "datalog_put_edge")
    }

    fn query_neighbors(&self, node_id: &str) -> Result<Vec<(String, String)>> {
        let datalog = Self::datalog_query_neighbors(node_id);
        tracing::debug!(node_id, "CozoProxyBackend::query_neighbors");
        let rows = self.run_datalog_query(&datalog)?;

        let mut results = Vec::new();
        // Expected headers: [target, relation]
        for row in &rows.rows {
            if row.len() >= 2 {
                let target = row[0].as_str().unwrap_or("").to_string();
                let relation = row[1].as_str().unwrap_or("").to_string();
                if !target.is_empty() {
                    results.push((target, relation));
                }
            }
        }
        Ok(results)
    }

    fn query_path(&self, from: &str, to: &str) -> Result<Vec<String>> {
        let datalog = Self::datalog_query_path(from, to);
        tracing::debug!(from, to, "CozoProxyBackend::query_path");
        let rows = self.run_datalog_query(&datalog)?;

        let mut results = Vec::new();
        // Expected headers: [path]
        // The path column may contain a JSON array of node IDs
        for row in &rows.rows {
            if let Some(val) = row.first() {
                if let Some(arr) = val.as_array() {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            results.push(s.to_string());
                        }
                    }
                } else if let Some(s) = val.as_str() {
                    results.push(s.to_string());
                }
            }
        }
        Ok(results)
    }

    fn is_available(&self) -> bool {
        self.changeguard_available
    }
}

// ---------------------------------------------------------------------------
// Datalog string escaping
// ---------------------------------------------------------------------------

/// Escape a string literal for safe embedding inside a Datalog string constant.
/// Replaces backslashes and double-quotes.
fn escape_datalog_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Datalog translation tests
    // ---------------------------------------------------------------

    #[test]
    fn datalog_put_node_produces_valid_syntax() {
        let metadata = serde_json::json!({"key": "value"});
        let datalog = CozoProxyBackend::datalog_put_node("node-1", "TestNode", "memory", &metadata);

        assert!(datalog.starts_with("?[id, label, category, metadata] <- [["));
        assert!(datalog.ends_with("]] :put node"));
        assert!(datalog.contains("\"node-1\""));
        assert!(datalog.contains("\"TestNode\""));
        assert!(datalog.contains("\"memory\""));
        assert!(datalog.contains("\"key\""));
        assert!(datalog.contains("\"value\""));
    }

    #[test]
    fn datalog_put_edge_produces_valid_syntax() {
        let datalog = CozoProxyBackend::datalog_put_edge("src-1", "dst-1", "RELATES_TO", 0.95);

        assert!(datalog.starts_with("?[source, target, relation, confidence] <- [["));
        assert!(datalog.ends_with("]] :put edge"));
        assert!(datalog.contains("\"src-1\""));
        assert!(datalog.contains("\"dst-1\""));
        assert!(datalog.contains("\"RELATES_TO\""));
        assert!(datalog.contains("0.95"));
    }

    #[test]
    fn datalog_query_neighbors_produces_valid_syntax() {
        let datalog = CozoProxyBackend::datalog_query_neighbors("node-42");

        assert!(datalog.starts_with("?[target, relation] := *edge{source: \""));
        assert!(datalog.contains("\"node-42\""));
        assert!(datalog.ends_with(", target, relation}"));
    }

    #[test]
    fn datalog_query_path_produces_valid_syntax() {
        let datalog = CozoProxyBackend::datalog_query_path("A", "B");

        assert!(datalog.starts_with("?[path] := *reachable{source: \""));
        assert!(datalog.contains("\"A\""));
        assert!(datalog.contains("\"B\""));
        assert!(datalog.ends_with(", path}"));
    }

    #[test]
    fn escape_datalog_str_handles_special_chars() {
        let result = escape_datalog_str(r#"hello "world" \test"#);
        assert_eq!(result, r#"hello \"world\" \\test"#);
    }

    // ---------------------------------------------------------------
    // Backend availability tests
    // ---------------------------------------------------------------

    #[test]
    fn cozo_proxy_backend_unavailable_when_no_changeguard_dir() {
        let backend = CozoProxyBackend::new(Some(PathBuf::from("./nonexistent_dir_12345")));
        assert!(
            !backend.is_available(),
            "Backend should be unavailable without .changeguard/"
        );
    }

    #[test]
    fn cozo_proxy_backend_graphbackend_methods_return_error_when_unavailable() {
        let backend = CozoProxyBackend::new(Some(PathBuf::from("./nonexistent_dir_12345")));
        assert!(!backend.is_available());

        let result = backend.add_node("n1", "Test", "memory", &serde_json::json!({}));
        assert!(result.is_err());

        let result = backend.add_edge("a", "b", "REL", 1.0);
        assert!(result.is_err());

        let result = backend.query_neighbors("n1");
        assert!(result.is_err());

        let result = backend.query_path("a", "b");
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // NamedRows parsing tests
    // ---------------------------------------------------------------

    #[test]
    fn cozo_named_rows_deserialization() {
        let json = serde_json::json!({
            "headers": ["target", "relation"],
            "rows": [
                ["node-2", "RELATES_TO"],
                ["node-3", "DEPENDS_ON"]
            ]
        });
        let rows: CozoNamedRows = serde_json::from_value(json).unwrap();
        assert_eq!(rows.headers.len(), 2);
        assert_eq!(rows.rows.len(), 2);
        assert_eq!(rows.rows[0][0], "node-2");
        assert_eq!(rows.rows[1][1], "DEPENDS_ON");
    }

    // ---------------------------------------------------------------
    // GraphBackend default impl test (via CozoProxyBackend)
    // ---------------------------------------------------------------

    #[test]
    fn cozo_proxy_backend_smoke_test() {
        // Smoke test: ensure the type compiles and basic construction works
        let backend = CozoProxyBackend::new(None);
        // Backend may or may not be available depending on environment,
        // but construction must not panic.
        let _ = backend.is_available();
    }
}
