//! Structured verification Evidence from gate outcomes (T149 Phase F / Task 2.6).
//!
//! Pure mapping: [`GateDecision`] → bounded evidence attributes.
//! Fail-open IPC errors become [`VerificationEvidenceStatus::Unavailable`], never Passed.
//! Secrets, raw env dumps, and full command stdout are never stored by default.

use crate::action_digest::{looks_like_secret_key, redact_sensitive_text};
use crate::verification_gate::{GateDecision, VerifyResponse};
use ai_brains_core::ids::{EvidenceId, SourceId};
use ai_brains_core::privacy::Privacy;
use ai_brains_events::constructors::EventBuilder;
use ai_brains_events::payload::{EvidenceRecordedPayload, Payload};
use ai_brains_events::{Actor, AggregateType, Envelope, EventKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Verification outcome status stored as evidence attributes (R6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum VerificationEvidenceStatus {
    Passed,
    Blocked,
    Unavailable,
}

/// Bounded structured verification evidence (no env dump, no raw stdout).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub status: VerificationEvidenceStatus,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl VerificationEvidence {
    /// Map a gate decision into storeable verification evidence.
    ///
    /// `ProceedUnavailable` is always `Unavailable` — never upgraded to Passed.
    pub fn from_gate_decision(decision: &GateDecision) -> Self {
        match decision {
            GateDecision::Proceed { verify } => Self::from_passed(verify),
            GateDecision::ProceedUnavailable { reason } => Self::from_unavailable(reason),
            GateDecision::Blocked {
                failure_probability,
                drift_detected,
                risk_level,
                explanation,
            } => Self::from_blocked(
                *failure_probability,
                *drift_detected,
                risk_level,
                explanation,
            ),
        }
    }

    pub fn from_passed(verify: &VerifyResponse) -> Self {
        Self {
            status: VerificationEvidenceStatus::Passed,
            kind: "verification_gate".to_string(),
            failure_probability: Some(verify.failure_probability),
            drift_detected: Some(verify.drift_detected),
            risk_level: Some(sanitize_risk_level(&verify.risk_level)),
            explanation: Some(redact_sensitive_text(&verify.explanation)),
        }
    }

    pub fn from_blocked(
        failure_probability: f64,
        drift_detected: bool,
        risk_level: &str,
        explanation: &str,
    ) -> Self {
        Self {
            status: VerificationEvidenceStatus::Blocked,
            kind: "verification_gate".to_string(),
            failure_probability: Some(failure_probability),
            drift_detected: Some(drift_detected),
            risk_level: Some(sanitize_risk_level(risk_level)),
            explanation: Some(redact_sensitive_text(explanation)),
        }
    }

    pub fn from_unavailable(reason: &str) -> Self {
        Self {
            status: VerificationEvidenceStatus::Unavailable,
            kind: "verification_gate".to_string(),
            failure_probability: None,
            drift_detected: None,
            risk_level: None,
            explanation: Some(redact_sensitive_text(reason)),
        }
    }

    /// JSON summary suitable for [`EvidenceRecordedPayload::summary`].
    pub fn to_summary(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Reject accidental inclusion of secret-looking env keys in the summary blob.
    pub fn contains_secret_looking_key(&self) -> bool {
        if let Some(ref level) = self.risk_level
            && looks_like_secret_key(level)
        {
            return true;
        }
        if let Some(ref expl) = self.explanation {
            for token in expl.split(|c: char| c.is_whitespace() || c == '=' || c == ':' || c == ',')
            {
                if looks_like_secret_key(token) {
                    return true;
                }
            }
        }
        false
    }
}

/// Stable SourceId for the Ledgerful verification-gate provenance source.
pub fn verification_gate_source_id() -> SourceId {
    SourceId::from_uuid(Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        b"ai-brains.capture.verification-gate",
    ))
}

/// Build governed envelopes for a verification outcome.
///
/// Emits only `EvidenceRecorded` keyed to the well-known verification-gate
/// [`SourceId`] (T149-F9). Registration of that source is a one-time/setup
/// concern — re-emitting `SourceRegistered` on every gated capture polluted the
/// event log without changing projection state.
pub fn build_verification_evidence_events(
    evidence: &VerificationEvidence,
    actor: Actor,
    privacy: Privacy,
) -> crate::Result<Vec<Envelope>> {
    let source_id = verification_gate_source_id();
    let evidence_id = EvidenceId::new();
    let summary = evidence.to_summary()?;

    let evidence_event = EventBuilder::new(
        AggregateType::Evidence,
        evidence_id.as_uuid(),
        EventKind::EvidenceRecorded,
        actor,
        privacy,
    )
    .build(Payload::EvidenceRecorded(EvidenceRecordedPayload {
        evidence_id,
        source_id,
        source_version_id: None,
        fingerprint: None,
        model_provenance: None,
        summary,
    }))?;

    Ok(vec![evidence_event])
}

fn sanitize_risk_level(level: &str) -> String {
    let trimmed = level.trim();
    if looks_like_secret_key(trimmed) {
        "unknown".to_string()
    } else {
        // Bound length; keep alphanumeric + common separators only.
        trimmed
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/'))
            .take(32)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::verification_gate::VerifyResponse;

    fn low_verify() -> VerifyResponse {
        VerifyResponse {
            failure_probability: 0.1,
            drift_detected: false,
            risk_level: "low".to_string(),
            explanation: "All clear".to_string(),
        }
    }

    #[test]
    fn from_gate_decision__proceed__passed_status() {
        let decision = GateDecision::Proceed {
            verify: low_verify(),
        };
        let ev = VerificationEvidence::from_gate_decision(&decision);
        assert_eq!(ev.status, VerificationEvidenceStatus::Passed);
        assert_eq!(ev.failure_probability, Some(0.1));
        assert_eq!(ev.risk_level.as_deref(), Some("low"));
        assert!(!ev.contains_secret_looking_key());
    }

    #[test]
    fn from_gate_decision__blocked__blocked_with_risk_fields() {
        let decision = GateDecision::Blocked {
            failure_probability: 0.9,
            drift_detected: true,
            risk_level: "critical".to_string(),
            explanation: "High risk predicted".to_string(),
        };
        let ev = VerificationEvidence::from_gate_decision(&decision);
        assert_eq!(ev.status, VerificationEvidenceStatus::Blocked);
        assert_eq!(ev.failure_probability, Some(0.9));
        assert_eq!(ev.drift_detected, Some(true));
        assert_eq!(ev.risk_level.as_deref(), Some("critical"));
        assert!(ev.explanation.as_deref().unwrap().contains("High risk"));
    }

    #[test]
    fn from_gate_decision__proceed_unavailable__never_passed() {
        let decision = GateDecision::ProceedUnavailable {
            reason: "mock IPC failure".to_string(),
        };
        let ev = VerificationEvidence::from_gate_decision(&decision);
        assert_eq!(ev.status, VerificationEvidenceStatus::Unavailable);
        assert_ne!(ev.status, VerificationEvidenceStatus::Passed);
        assert!(ev.failure_probability.is_none());
    }

    #[test]
    fn summary__redacts_secret_looking_tokens() {
        let ev = VerificationEvidence::from_unavailable(
            "backend down API_KEY=sk-super-secret TOKEN=abc stdout dump",
        );
        let summary = ev.to_summary().unwrap();
        assert!(
            !summary.contains("sk-super-secret"),
            "secret value must not appear in summary: {summary}"
        );
        assert!(
            !summary.contains("TOKEN=abc"),
            "token value must be redacted: {summary}"
        );
        assert!(
            summary.contains("[REDACTED]"),
            "expected redaction markers: {summary}"
        );
        // stdout / env must never be structured stored fields.
        assert!(!summary.contains("\"stdout\""));
        assert!(!summary.contains("\"env\""));
    }

    #[test]
    fn build_events__evidence_recorded_summary_carries_status() {
        let ev = VerificationEvidence::from_passed(&low_verify());
        let events =
            build_verification_evidence_events(&ev, Actor::System, Privacy::LocalOnly).unwrap();
        assert_eq!(events.len(), 1, "only EvidenceRecorded (no re-register)");
        match &events[0].payload {
            Payload::EvidenceRecorded(p) => {
                assert_eq!(p.source_id, verification_gate_source_id());
                assert!(p.summary.contains("Passed"));
                assert!(p.summary.contains("verification_gate"));
                assert!(!p.summary.contains("\"env\""));
            }
            other => panic!("expected EvidenceRecorded, got {other:?}"),
        }
    }
}
