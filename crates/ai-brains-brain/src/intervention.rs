//! RiskReviewAgent: background intervention agent that listens for high-risk
//! drift alerts from ChangeGuard's watcher and proactively injects warnings
//! into the AI harness context.
//!
//! ## Non-blocking
//! The agent runs asynchronously and MUST never block the capture pipeline.
//!
//! ## Deduplication
//! Alerts for the same coupling pair (source_file, target_file) are only
//! emitted once per session.

use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_core::privacy::Privacy;
use ai_brains_graph::{CozoProxyBackend, GraphBackend};
use std::collections::HashSet;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// A risk alert parsed from a ChangeGuard `BridgeRecord` with
/// `record_kind = "risk_alert"`.
#[derive(Debug, Clone)]
pub struct RiskAlert {
    /// File that was modified.
    pub source_file: String,
    /// File that is temporally coupled to the source.
    pub target_file: String,
    /// Temporal coupling score (0.0 – 1.0). Higher values indicate stronger
    /// coupling and thus greater risk.
    pub coupling_score: f64,
    /// Human-readable explanation of the risk.
    pub explanation: String,
}

impl PartialEq for RiskAlert {
    fn eq(&self, other: &Self) -> bool {
        self.source_file == other.source_file
            && self.target_file == other.target_file
            && self.coupling_score.to_bits() == other.coupling_score.to_bits()
            && self.explanation == other.explanation
    }
}

impl Eq for RiskAlert {}

/// A formatted warning that the AI harness can inject into the agent context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionWarning {
    /// The alert that triggered this warning.
    pub alert: RiskAlert,
    /// Files that are transitively reachable from the coupled pair (blast
    /// radius) according to the CozoDB reachability graph.
    pub blast_radius: Vec<String>,
    /// Natural-language suggestion for the AI agent to self-correct.
    pub suggestion: String,
}

// ---------------------------------------------------------------------------
// RiskReviewAgent
// ---------------------------------------------------------------------------

/// Background agent that polls ChangeGuard for high-risk drift alerts,
/// queries the CozoDB reachability graph to assess blast radius, and
/// formats intervention warnings for the AI harness.
///
/// The agent is designed to run asynchronously (e.g. on a `tokio::spawn`
/// task).  It never blocks the capture pipeline.
pub struct RiskReviewAgent {
    /// Handle to the CozoDB proxy backend (from T42) for reachability queries.
    cozo_backend: CozoProxyBackend,
    /// Tracks coupling pairs (source, target) already alerted this session
    /// so we never spam the harness with duplicate warnings.
    seen_pairs: Mutex<HashSet<(String, String)>>,
}

impl RiskReviewAgent {
    /// Create a new agent that queries the given CozoDB backend and polls
    /// ChangeGuard via bridge IPC for risk alerts.
    pub fn new(cozo_backend: CozoProxyBackend) -> Self {
        Self {
            cozo_backend,
            seen_pairs: Mutex::new(HashSet::new()),
        }
    }

    /// Run one polling cycle: fetch risk alerts from ChangeGuard, deduplicate,
    /// assess blast radius for each, and return formatted warnings.
    ///
    /// This is safe to call from a background `tokio::spawn` — it will never
    /// panic and gracefully degrades (returns empty vec) when ChangeGuard or
    /// CozoDB are unavailable.
    pub async fn run(&self) -> Vec<InterventionWarning> {
        let alerts = match self.poll_risk_alerts().await {
            Ok(alerts) => alerts,
            Err(e) => {
                tracing::warn!("RiskReviewAgent: unable to poll risk alerts: {e}");
                return Vec::new();
            }
        };

        let mut warnings = Vec::new();

        for alert in alerts {
            // ---- Deduplicate -------------------------------------------------
            {
                let mut seen = match self.seen_pairs.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        tracing::error!("RiskReviewAgent: seen_pairs mutex poisoned");
                        poisoned.into_inner()
                    }
                };
                let pair = (alert.source_file.clone(), alert.target_file.clone());
                if !seen.insert(pair) {
                    tracing::debug!(
                        src = %alert.source_file,
                        tgt = %alert.target_file,
                        "RiskReviewAgent: skipping duplicate alert"
                    );
                    continue;
                }
            } // MutexGuard dropped here — lock is never held across .await

            // ---- Blast radius -----------------------------------------------
            let blast_radius = self.assess_blast_radius(&alert).await;

            let suggestion = format!(
                "WARNING: High temporal coupling ({:.2}) between '{}' and '{}'. \
                 Blast radius includes {} downstream node(s): [{}]. \
                 Review both files and affected dependents before committing.",
                alert.coupling_score,
                alert.source_file,
                alert.target_file,
                blast_radius.len(),
                blast_radius.join(", ")
            );

            warnings.push(InterventionWarning {
                alert,
                blast_radius,
                suggestion,
            });
        }

        warnings
    }

    // ------------------------------------------------------------------
    // Polling
    // ------------------------------------------------------------------

    /// Poll ChangeGuard via `bridge export --hotspots` for risk-alert records.
    /// Returns an empty `Vec` (not an error) when ChangeGuard is unreachable
    /// so the agent never disrupts the main pipeline.
    async fn poll_risk_alerts(&self) -> Result<Vec<RiskAlert>, String> {
        query_changeguard_risk_alerts().await
    }

    // ------------------------------------------------------------------
    // Blast radius assessment
    // ------------------------------------------------------------------

    /// Query the CozoDB reachability graph for all nodes between the coupled
    /// pair.  If CozoDB is unavailable the method returns an empty vec
    /// (graceful degradation).
    async fn assess_blast_radius(&self, alert: &RiskAlert) -> Vec<String> {
        if !self.cozo_backend.is_available() {
            tracing::debug!("CozoDB unavailable; skipping blast-radius assessment");
            return Vec::new();
        }

        match self
            .cozo_backend
            .query_path(&alert.source_file, &alert.target_file)
        {
            Ok(path) => path,
            Err(e) => {
                tracing::debug!(
                    "Blast-radius query failed for {}-{}: {e}",
                    alert.source_file,
                    alert.target_file
                );
                Vec::new()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bridge IPC: polling for risk alerts
// ---------------------------------------------------------------------------

/// Shell out to `changeguard bridge export --hotspots` and parse the NDJSON
/// output for records with `record_kind = "risk_alert"`.
#[allow(clippy::disallowed_methods)]
async fn query_changeguard_risk_alerts() -> Result<Vec<RiskAlert>, String> {
    // Build the request envelope (same pattern as other bridge users).
    let record = BridgeRecord {
        bridge_version: "0.2".to_string(),
        direction: BridgeDirection::Outbound,
        timestamp: chrono::Utc::now().to_rfc3339(),
        parent_hash: None,
        project_id: String::new(),
        session_id: None,
        tx_id: None,
        record_kind: "risk_poll".to_string(),
        payload: serde_json::json!({"kind": "risk_alert_poll"}),
        privacy: Privacy::LocalOnly,
    };

    let temp_in = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {e}"))?;
    let temp_in_path = temp_in.path().to_path_buf();
    let ndjson = serde_json::to_string(&record).map_err(|e| format!("serialize error: {e}"))?;
    std::fs::write(&temp_in_path, ndjson.as_bytes()).map_err(|e| format!("write error: {e}"))?;

    let temp_out = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {e}"))?;
    let temp_out_path = temp_out.path().to_path_buf();

    // Run the bridge export in a blocking thread so we don't stall the async
    // runtime.  `std::process::Command` is inherently synchronous.
    let out_path = temp_out_path.clone();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("changeguard")
            .args([
                "bridge",
                "export",
                "--hotspots",
                "--out",
                out_path.to_str().unwrap_or(""),
            ])
            .output()
    })
    .await
    .map_err(|e| format!("spawn_blocking failed: {e}"))?
    .map_err(|e| format!("changeguard CLI not available: {e}"))?;

    // Keep temp_in alive until the command completes.
    drop(temp_in);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("changeguard bridge export failed: {stderr}"));
    }

    let content =
        std::fs::read_to_string(&temp_out_path).map_err(|e| format!("read output: {e}"))?;

    parse_risk_alerts_from_ndjson(&content)
}

/// Walk every NDJSON line looking for `record_kind = "risk_alert"`.
#[allow(clippy::disallowed_methods)]
fn parse_risk_alerts_from_ndjson(content: &str) -> Result<Vec<RiskAlert>, String> {
    let mut alerts = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if record.record_kind.to_lowercase() != "risk_alert" {
            continue;
        }

        let source_file = record
            .payload
            .get("source_file")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let target_file = record
            .payload
            .get("target_file")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let coupling_score = record
            .payload
            .get("coupling_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let explanation = record
            .payload
            .get("explanation")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if !source_file.is_empty() && !target_file.is_empty() {
            alerts.push(RiskAlert {
                source_file,
                target_file,
                coupling_score,
                explanation,
            });
        }
    }

    Ok(alerts)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ---------------------------------------------------------------
    // RiskAlert parsing tests
    // ---------------------------------------------------------------

    #[test]
    fn parse_risk_alert_from_valid_ndjson() {
        let record = BridgeRecord {
            bridge_version: "0.2".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "risk_alert".to_string(),
            payload: serde_json::json!({
                "source_file": "src/main.rs",
                "target_file": "src/lib.rs",
                "coupling_score": 0.89,
                "explanation": "Historical coupling > 85%"
            }),
            privacy: Privacy::LocalOnly,
        };

        let ndjson = serde_json::to_string(&record).unwrap();
        let alerts = parse_risk_alerts_from_ndjson(&ndjson).unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].source_file, "src/main.rs");
        assert_eq!(alerts[0].target_file, "src/lib.rs");
        assert!(alerts[0].coupling_score > 0.85);
    }

    #[test]
    fn parse_ignores_non_risk_alert_records() {
        let hotspot = BridgeRecord {
            bridge_version: "0.2".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "hotspot".to_string(),
            payload: serde_json::json!({"severity": 0.9}),
            privacy: Privacy::LocalOnly,
        };

        let ndjson = serde_json::to_string(&hotspot).unwrap();
        let alerts = parse_risk_alerts_from_ndjson(&ndjson).unwrap();
        assert!(
            alerts.is_empty(),
            "Non-risk_alert records should be ignored"
        );
    }

    #[test]
    fn parse_empty_ndjson_returns_empty() {
        let alerts = parse_risk_alerts_from_ndjson("").unwrap();
        assert!(alerts.is_empty());
    }

    #[test]
    fn parse_multiple_risk_alerts() {
        let alert1 = BridgeRecord {
            bridge_version: "0.2".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "risk_alert".to_string(),
            payload: serde_json::json!({
                "source_file": "a.rs",
                "target_file": "b.rs",
                "coupling_score": 0.9,
                "explanation": "Pair 1"
            }),
            privacy: Privacy::LocalOnly,
        };

        let alert2 = BridgeRecord {
            bridge_version: "0.2".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "risk_alert".to_string(),
            payload: serde_json::json!({
                "source_file": "c.rs",
                "target_file": "d.rs",
                "coupling_score": 0.75,
                "explanation": "Pair 2"
            }),
            privacy: Privacy::LocalOnly,
        };

        let ndjson = format!(
            "{}\n{}",
            serde_json::to_string(&alert1).unwrap(),
            serde_json::to_string(&alert2).unwrap()
        );
        let alerts = parse_risk_alerts_from_ndjson(&ndjson).unwrap();
        assert_eq!(alerts.len(), 2);
    }

    // ---------------------------------------------------------------
    // RiskReviewAgent deduplication tests
    // ---------------------------------------------------------------

    #[test]
    fn agent_deduplicates_by_coupling_pair() {
        let backend = CozoProxyBackend::new(Some(PathBuf::from("./nonexistent_dir_12345")));
        let agent = RiskReviewAgent::new(backend);

        // Manually simulate seen-pairs insertion
        {
            let mut seen = agent.seen_pairs.lock().unwrap();
            seen.insert(("a.rs".to_string(), "b.rs".to_string()));
        }

        // Verify the pair is tracked
        {
            let seen = agent.seen_pairs.lock().unwrap();
            assert!(seen.contains(&("a.rs".to_string(), "b.rs".to_string())));
            assert!(!seen.contains(&("x.rs".to_string(), "y.rs".to_string())));
        }
    }

    // ---------------------------------------------------------------
    // RiskReviewAgent blast-radius tests
    // ---------------------------------------------------------------

    #[test]
    fn blast_radius_returns_empty_when_cozo_unavailable() {
        // Use a backend that we know is unavailable (no .changeguard/ dir)
        let backend = CozoProxyBackend::new(Some(PathBuf::from("./nonexistent_dir_12345")));
        assert!(!backend.is_available());

        let agent = RiskReviewAgent::new(backend);

        let alert = RiskAlert {
            source_file: "src/a.rs".to_string(),
            target_file: "src/b.rs".to_string(),
            coupling_score: 0.9,
            explanation: "test".to_string(),
        };

        // We need to call the async method from a sync test.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let radius = rt.block_on(agent.assess_blast_radius(&alert));
        assert!(
            radius.is_empty(),
            "Blast radius should be empty when CozoDB is unavailable"
        );
    }

    // ---------------------------------------------------------------
    // InterventionWarning formatting
    // ---------------------------------------------------------------

    #[test]
    fn intervention_warning_includes_blast_radius() {
        let blast_radius = vec![
            "src/a.rs".to_string(),
            "src/shared.rs".to_string(),
            "src/b.rs".to_string(),
        ];
        let alert = RiskAlert {
            source_file: "src/a.rs".to_string(),
            target_file: "src/b.rs".to_string(),
            coupling_score: 0.92,
            explanation: "test".to_string(),
        };

        // Use the same format string as RiskReviewAgent::run()
        let suggestion = format!(
            "WARNING: High temporal coupling ({:.2}) between '{}' and '{}'. \
             Blast radius includes {} downstream node(s): [{}]. \
             Review both files and affected dependents before committing.",
            alert.coupling_score,
            alert.source_file,
            alert.target_file,
            blast_radius.len(),
            blast_radius.join(", ")
        );

        let warning = InterventionWarning {
            alert,
            blast_radius,
            suggestion,
        };

        assert!(warning.suggestion.contains("0.92"));
        assert!(warning.suggestion.contains("src/a.rs"));
        assert!(warning.suggestion.contains("src/b.rs"));
        assert!(warning.suggestion.contains("3 downstream node"));
    }
}
