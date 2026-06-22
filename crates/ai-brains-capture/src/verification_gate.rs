//! Verification gate: intercepts `ingest-final` (assistant capture) and calls
//! ChangeGuard's predictive CI engine before the event is committed.
//!
//! ## Fail-open
//! If ChangeGuard IPC is unreachable the gate **must** let capture proceed.
//! We never block capture due to a transient pipe failure.
//!
//! ## CQRS
//! This is a command-side interceptor — it blocks event append, not queries.

use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_core::privacy::Privacy;
use serde::{Deserialize, Serialize};
use std::fs;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Structured verification response parsed from ChangeGuard IPC output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Predicted CI failure probability (0.0 – 1.0).
    pub failure_probability: f64,
    /// Whether ChangeGuard detected uncommitted ledger drift.
    pub drift_detected: bool,
    /// Risk classification: "low", "medium", "high", or "critical".
    pub risk_level: String,
    /// Human-readable explanation of the verification result.
    pub explanation: String,
}

/// Decision returned by [`VerificationGate::check`].
#[derive(Debug, Clone)]
pub enum GateDecision {
    /// Capture is allowed to proceed.
    Proceed,
    /// Capture is blocked — the AI harness must self-remediate.
    Blocked {
        failure_probability: f64,
        drift_detected: bool,
        risk_level: String,
        explanation: String,
    },
}

// ---------------------------------------------------------------------------
// VerificationBackend trait (mockable for testing)
// ---------------------------------------------------------------------------

/// Abstraction over the ChangeGuard IPC call. Mock implementations are used in
/// unit tests so we can exercise both the blocking and fail-open paths without
/// a real ChangeGuard installation.
pub trait VerificationBackend: Send + Sync + std::fmt::Debug {
    /// Execute a verification check. Returns the parsed response on success,
    /// or a human-readable error string when IPC fails.
    fn run_verify(&self) -> Result<VerifyResponse, String>;
}

// ---------------------------------------------------------------------------
// Real ChangeGuard backend (via bridge export IPC)
// ---------------------------------------------------------------------------

/// Production backend that shells out to `changeguard bridge export`.
#[derive(Debug)]
pub struct ChangeGuardVerificationBackend;

impl VerificationBackend for ChangeGuardVerificationBackend {
    fn run_verify(&self) -> Result<VerifyResponse, String> {
        query_changeguard_verification()
    }
}

// ---------------------------------------------------------------------------
// VerificationGate
// ---------------------------------------------------------------------------

/// The verification gate that sits between ingest and event commit.
///
/// # Threshold
/// When `failure_probability >= threshold` the gate blocks. Default threshold
/// is 0.7 (70 %).
#[derive(Debug)]
pub struct VerificationGate {
    backend: Box<dyn VerificationBackend>,
    threshold: f64,
}

impl VerificationGate {
    /// Create a new gate with the given backend and failure threshold.
    pub fn new(backend: Box<dyn VerificationBackend>, threshold: f64) -> Self {
        Self { backend, threshold }
    }

    /// Default failure-probability threshold above which the gate blocks.
    pub const DEFAULT_THRESHOLD: f64 = 0.7;

    /// Run the verification check.
    ///
    /// Returns [`GateDecision::Proceed`] when:
    /// - ChangeGuard IPC is unreachable (**fail-open**), or
    /// - failure probability is below the threshold and no drift is detected.
    ///
    /// Returns [`GateDecision::Blocked`] when:
    /// - failure probability exceeds the threshold, or
    /// - drift is detected.
    pub fn check(&self) -> GateDecision {
        match self.backend.run_verify() {
            Ok(resp) => {
                let blocked = resp.failure_probability >= self.threshold || resp.drift_detected;
                if blocked {
                    tracing::warn!(
                        failure_prob = resp.failure_probability,
                        drift = resp.drift_detected,
                        risk = %resp.risk_level,
                        "Verification gate BLOCKED ingest"
                    );
                    GateDecision::Blocked {
                        failure_probability: resp.failure_probability,
                        drift_detected: resp.drift_detected,
                        risk_level: resp.risk_level,
                        explanation: resp.explanation,
                    }
                } else {
                    tracing::debug!("Verification gate: PROCEED");
                    GateDecision::Proceed
                }
            }
            Err(e) => {
                // **Fail-open**: never block capture on transient IPC failure.
                tracing::warn!(
                    "ChangeGuard IPC unreachable, failing open (proceeding with ingest): {e}"
                );
                GateDecision::Proceed
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bridge IPC query
// ---------------------------------------------------------------------------

/// Call `changeguard bridge export --hotspots --ledger`, parse the NDJSON
/// output, and synthesise a [`VerifyResponse`].
#[allow(clippy::disallowed_methods)]
fn query_changeguard_verification() -> Result<VerifyResponse, String> {
    // -- Build the request envelope -----------------------------------------
    let record = BridgeRecord {
        bridge_version: "0.3".to_string(),
        direction: BridgeDirection::Outbound,
        timestamp: chrono::Utc::now(),
        parent_hash: None,
        project_id: String::new(),
        session_id: None,
        tx_id: None,
        record_kind: "verification_request".to_string(),
        payload: ai_brains_contracts::bridge::BridgePayload::Unknown(
            serde_json::json!({"kind": "ingest_gate_check"}),
        ),
        privacy: Privacy::LocalOnly,
    };

    let temp_in = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {e}"))?;
    let temp_in_path = temp_in.path().to_path_buf();
    let ndjson = serde_json::to_string(&record).map_err(|e| format!("serialize error: {e}"))?;
    fs::write(&temp_in_path, ndjson.as_bytes()).map_err(|e| format!("write error: {e}"))?;

    let temp_out = tempfile::NamedTempFile::new().map_err(|e| format!("tempfile error: {e}"))?;
    let temp_out_path = temp_out.path().to_path_buf();

    // -- Invoke changeguard bridge export ----------------------------------
    let output = std::process::Command::new("ledgerful")
        .args([
            "bridge",
            "export",
            "--hotspots",
            "--ledger",
            "--out",
            temp_out_path
                .to_str()
                .ok_or_else(|| "invalid temp output path".to_string())?,
        ])
        .output()
        .map_err(|e| format!("ledgerful CLI not available: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ledgerful bridge export failed: {stderr}"));
    }

    // -- Parse NDJSON output -----------------------------------------------
    let content =
        fs::read_to_string(&temp_out_path).map_err(|e| format!("read output error: {e}"))?;

    parse_verification_from_ndjson(&content)
}

/// Walk every NDJSON line, look for record kinds we recognise, and build a
/// consolidated [`VerifyResponse`].
#[allow(clippy::disallowed_methods)]
fn parse_verification_from_ndjson(content: &str) -> Result<VerifyResponse, String> {
    let mut failure_prob: f64 = 0.0;
    let mut drift_detected = false;
    let mut risk_level = "low".to_string();
    let mut explanations: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let record: BridgeRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let payload = record.payload_value();

        match record.record_kind.as_str() {
            "verification_summary" => {
                if let Some(prob) = payload.get("failure_probability").and_then(|v| v.as_f64()) {
                    failure_prob = prob;
                }
                if let Some(drift) = payload.get("drift_detected").and_then(|v| v.as_bool()) {
                    drift_detected = drift_detected || drift;
                }
                if let Some(level) = payload.get("risk_level").and_then(|v| v.as_str()) {
                    if risk_level_ord(level) > risk_level_ord(&risk_level) {
                        risk_level = level.to_string();
                    }
                }
                if let Some(explain) = payload.get("explanation").and_then(|v| v.as_str()) {
                    explanations.push(explain.to_string());
                }
            }
            "hotspot" => {
                if let Some(severity) = payload.get("severity").and_then(|v| v.as_f64()) {
                    // Use the highest hotspot severity as a proxy for failure
                    // probability when no explicit verification_summary exists.
                    failure_prob = f64::max(failure_prob, severity);
                }
                if let Some(level) = payload.get("risk").and_then(|v| v.as_str()) {
                    if risk_level_ord(level) > risk_level_ord(&risk_level) {
                        risk_level = level.to_string();
                    }
                }
                if let Some(explain) = payload.get("explanation").and_then(|v| v.as_str()) {
                    explanations.push(explain.to_string());
                }
            }
            "drift_delta" => {
                drift_detected = true;
                if let Some(explain) = payload.get("explanation").and_then(|v| v.as_str()) {
                    explanations.push(format!("[DRIFT] {explain}"));
                }
            }
            _ => { /* ignore unknown record kinds */ }
        }
    }

    let explanation = if explanations.is_empty() {
        "No verification signals received from ChangeGuard".to_string()
    } else {
        explanations.join("; ")
    };

    Ok(VerifyResponse {
        failure_probability: failure_prob,
        drift_detected,
        risk_level,
        explanation,
    })
}

/// Ordinal mapping for risk levels so we can pick the maximum.
fn risk_level_ord(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // Mock backend for testing
    // ---------------------------------------------------------------

    #[derive(Debug)]
    struct MockBackend {
        response: Option<VerifyResponse>,
    }

    impl VerificationBackend for MockBackend {
        fn run_verify(&self) -> Result<VerifyResponse, String> {
            match &self.response {
                Some(resp) => Ok(resp.clone()),
                None => Err("mock IPC failure".to_string()),
            }
        }
    }

    fn low_risk_response() -> VerifyResponse {
        VerifyResponse {
            failure_probability: 0.1,
            drift_detected: false,
            risk_level: "low".to_string(),
            explanation: "All clear".to_string(),
        }
    }

    fn high_risk_response() -> VerifyResponse {
        VerifyResponse {
            failure_probability: 0.85,
            drift_detected: false,
            risk_level: "high".to_string(),
            explanation: "High failure probability predicted".to_string(),
        }
    }

    fn drift_response() -> VerifyResponse {
        VerifyResponse {
            failure_probability: 0.2,
            drift_detected: true,
            risk_level: "medium".to_string(),
            explanation: "Ledger drift detected".to_string(),
        }
    }

    // ---------------------------------------------------------------
    // Gate decision tests
    // ---------------------------------------------------------------

    #[test]
    fn gate_proceeds_when_low_risk() {
        let backend = Box::new(MockBackend {
            response: Some(low_risk_response()),
        });
        let gate = VerificationGate::new(backend, VerificationGate::DEFAULT_THRESHOLD);
        let decision = gate.check();
        assert!(matches!(decision, GateDecision::Proceed));
    }

    #[test]
    fn gate_blocks_when_high_failure_probability() {
        let backend = Box::new(MockBackend {
            response: Some(high_risk_response()),
        });
        let gate = VerificationGate::new(backend, VerificationGate::DEFAULT_THRESHOLD);
        let decision = gate.check();
        assert!(matches!(decision, GateDecision::Blocked { .. }));
        if let GateDecision::Blocked {
            failure_probability,
            drift_detected,
            risk_level,
            ..
        } = decision
        {
            assert!(
                failure_probability >= VerificationGate::DEFAULT_THRESHOLD,
                "failure_prob={failure_probability} should be >= threshold"
            );
            assert!(!drift_detected);
            assert_eq!(risk_level, "high");
        }
    }

    #[test]
    fn gate_blocks_when_drift_detected() {
        let backend = Box::new(MockBackend {
            response: Some(drift_response()),
        });
        let gate = VerificationGate::new(backend, VerificationGate::DEFAULT_THRESHOLD);
        let decision = gate.check();
        assert!(matches!(decision, GateDecision::Blocked { .. }));
        if let GateDecision::Blocked {
            drift_detected,
            risk_level,
            ..
        } = decision
        {
            assert!(drift_detected);
            assert_eq!(risk_level, "medium");
        }
    }

    #[test]
    fn gate_fails_open_when_ipc_unreachable() {
        // Backend that simulates a crash / missing CLI
        let backend = Box::new(MockBackend { response: None });
        let gate = VerificationGate::new(backend, VerificationGate::DEFAULT_THRESHOLD);
        let decision = gate.check();
        assert!(
            matches!(decision, GateDecision::Proceed),
            "Gate MUST fail-open when IPC is unreachable"
        );
    }

    #[test]
    fn gate_respects_custom_threshold() {
        // failure_prob = 0.85, custom threshold = 0.9 -> should proceed
        let backend = Box::new(MockBackend {
            response: Some(high_risk_response()),
        });
        let gate = VerificationGate::new(backend, 0.9);
        let decision = gate.check();
        assert!(
            matches!(decision, GateDecision::Proceed),
            "Gate should proceed when failure_prob < custom threshold"
        );
    }

    // ---------------------------------------------------------------
    // NDJSON parsing tests
    // ---------------------------------------------------------------

    #[test]
    fn parse_empty_ndjson_returns_zeros() {
        let result = parse_verification_from_ndjson("").unwrap();
        assert!((result.failure_probability - 0.0).abs() < f64::EPSILON);
        assert!(!result.drift_detected);
        assert_eq!(result.risk_level, "low");
    }

    #[test]
    fn parse_verification_summary_record() {
        let ndjson = serde_json::to_string(&BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "verification_summary".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "failure_probability": 0.92,
                "drift_detected": true,
                "risk_level": "critical",
                "explanation": "CI prediction failure rate 92%"
            })),
            privacy: Privacy::LocalOnly,
        })
        .unwrap();

        let result = parse_verification_from_ndjson(&ndjson).unwrap();
        assert!((result.failure_probability - 0.92).abs() < f64::EPSILON);
        assert!(result.drift_detected);
        assert_eq!(result.risk_level, "critical");
        assert!(result.explanation.contains("92%"));
    }

    #[test]
    fn parse_hotspot_record_as_failure_proxy() {
        let ndjson = serde_json::to_string(&BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "hotspot".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "severity": 0.88,
                "risk": "high",
                "explanation": "Hotspot in core module"
            })),
            privacy: Privacy::LocalOnly,
        })
        .unwrap();

        let result = parse_verification_from_ndjson(&ndjson).unwrap();
        assert!(result.failure_probability >= 0.88);
        assert_eq!(result.risk_level, "high");
    }

    #[test]
    fn parse_drift_delta_record() {
        let ndjson = serde_json::to_string(&BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "drift_delta".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "explanation": "Uncommitted changes in ledger"
            })),
            privacy: Privacy::LocalOnly,
        })
        .unwrap();

        let result = parse_verification_from_ndjson(&ndjson).unwrap();
        assert!(result.drift_detected);
        assert!(result.explanation.contains("DRIFT"));
    }

    #[test]
    fn parse_multiple_records_chooses_max_risk() {
        let hotspot = serde_json::to_string(&BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "hotspot".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "severity": 0.60,
                "risk": "medium",
                "explanation": "Medium hotspot"
            })),
            privacy: Privacy::LocalOnly,
        })
        .unwrap();

        let summary = serde_json::to_string(&BridgeRecord {
            bridge_version: "0.3".to_string(),
            direction: BridgeDirection::Outbound,
            timestamp: chrono::Utc::now(),
            parent_hash: None,
            project_id: "test".to_string(),
            session_id: None,
            tx_id: None,
            record_kind: "verification_summary".to_string(),
            payload: ai_brains_contracts::bridge::BridgePayload::Unknown(serde_json::json!({
                "failure_probability": 0.95,
                "drift_detected": false,
                "risk_level": "critical",
                "explanation": "Critical from CI prediction"
            })),
            privacy: Privacy::LocalOnly,
        })
        .unwrap();

        let ndjson = format!("{hotspot}\n{summary}");
        let result = parse_verification_from_ndjson(&ndjson).unwrap();
        assert!((result.failure_probability - 0.95).abs() < f64::EPSILON);
        assert_eq!(result.risk_level, "critical");
    }

    // ---------------------------------------------------------------
    // risk_level_ord tests
    // ---------------------------------------------------------------

    #[test]
    fn risk_level_ord_ordering() {
        assert!(risk_level_ord("low") < risk_level_ord("medium"));
        assert!(risk_level_ord("medium") < risk_level_ord("high"));
        assert!(risk_level_ord("high") < risk_level_ord("critical"));
        assert_eq!(risk_level_ord("unknown"), 0);
    }
}
