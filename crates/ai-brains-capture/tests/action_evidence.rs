//! T149 Phase F / R6 — verification Evidence from capture gate outcomes.
//!
//! RED cases (now green):
//! - success verify → Passed evidence attributes
//! - blocked → Failed/Blocked with risk fields from real response
//! - IPC error fail-open → Unavailable, never Passed
//! - no secret-looking env keys in stored blob

#![allow(clippy::disallowed_methods)]
#![allow(non_snake_case)]

mod common;

use ai_brains_capture::{
    CaptureError, CaptureService, GateDecision, MemorySink, VerificationBackend,
    VerificationEvidence, VerificationEvidenceStatus, VerificationGate, VerifyResponse,
};
use ai_brains_events::Payload;

#[derive(Debug)]
struct MockBackend {
    response: Option<VerifyResponse>,
    error: Option<String>,
}

impl VerificationBackend for MockBackend {
    fn run_verify(&self) -> Result<VerifyResponse, String> {
        if let Some(ref e) = self.error {
            return Err(e.clone());
        }
        match &self.response {
            Some(r) => Ok(r.clone()),
            None => Err("mock IPC failure".to_string()),
        }
    }
}

fn low_risk() -> VerifyResponse {
    VerifyResponse {
        failure_probability: 0.05,
        drift_detected: false,
        risk_level: "low".to_string(),
        explanation: "All clear".to_string(),
    }
}

fn high_risk() -> VerifyResponse {
    VerifyResponse {
        failure_probability: 0.92,
        drift_detected: true,
        risk_level: "critical".to_string(),
        explanation: "High failure probability predicted".to_string(),
    }
}

fn service_with(backend: MockBackend) -> CaptureService {
    CaptureService::with_verification_gate(VerificationGate::new(
        Box::new(backend),
        VerificationGate::DEFAULT_THRESHOLD,
    ))
}

fn evidence_summaries(sink: &MemorySink) -> Vec<String> {
    // MemorySink does not expose events; re-run path via CaptureOutcome in tests below.
    let _ = sink;
    Vec::new()
}

fn find_evidence_payloads(events: &[ai_brains_events::Envelope]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match &e.payload {
            Payload::EvidenceRecorded(p) => Some(p.summary.as_str()),
            _ => None,
        })
        .collect()
}

fn parse_status(summary: &str) -> VerificationEvidence {
    serde_json::from_str(summary).expect("verification evidence summary must be JSON")
}

#[test]
fn gate_success__produces_passed_verification_evidence() {
    let service = service_with(MockBackend {
        response: Some(low_risk()),
        error: None,
    });
    let mut sink = MemorySink::default();
    let request = common::ingest_request("assistant", "safe change");
    let outcome = service
        .ingest_request(request, common::context(), &mut sink)
        .expect("capture should proceed");

    let summaries = find_evidence_payloads(&outcome.events);
    assert_eq!(
        summaries.len(),
        1,
        "expected exactly one verification EvidenceRecorded"
    );
    let ev = parse_status(summaries[0]);
    assert_eq!(ev.status, VerificationEvidenceStatus::Passed);
    assert_eq!(ev.kind, "verification_gate");
    assert_eq!(ev.failure_probability, Some(0.05));
    assert_eq!(ev.risk_level.as_deref(), Some("low"));
    assert!(
        outcome
            .events
            .iter()
            .any(|e| matches!(e.payload, Payload::AssistantFinalRecorded(_))),
        "assistant final must still be recorded on Passed"
    );
}

#[test]
fn gate_blocked__records_blocked_evidence_with_risk_fields() {
    let service = service_with(MockBackend {
        response: Some(high_risk()),
        error: None,
    });
    let mut sink = MemorySink::default();
    let request = common::ingest_request("assistant", "risky change");
    let err = service
        .ingest_request(request, common::context(), &mut sink)
        .expect_err("capture must be blocked");

    match err {
        CaptureError::VerificationGateRejected(r) => {
            assert!((r.failure_probability - 0.92).abs() < f64::EPSILON);
            assert!(r.drift_detected);
            assert_eq!(r.risk_level, "critical");
        }
        other => panic!("expected VerificationGateRejected, got {other:?}"),
    }

    // Blocked path returns Err but still appends evidence to the sink.
    let events = sink.into_events();
    let summaries = find_evidence_payloads(&events);
    assert_eq!(summaries.len(), 1);
    let ev = parse_status(summaries[0]);
    assert_eq!(ev.status, VerificationEvidenceStatus::Blocked);
    assert_eq!(ev.failure_probability, Some(0.92));
    assert_eq!(ev.drift_detected, Some(true));
    assert_eq!(ev.risk_level.as_deref(), Some("critical"));
    assert!(
        ev.explanation
            .as_deref()
            .is_some_and(|e| e.contains("High failure") || e.contains("probability")),
        "blocked explanation must retain risk signal: {:?}",
        ev.explanation
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.payload, Payload::AssistantFinalRecorded(_))),
        "assistant final must NOT be recorded when blocked"
    );
}

#[test]
fn gate_ipc_error__fail_open_marks_unavailable_never_passed() {
    let service = service_with(MockBackend {
        response: None,
        error: Some("ledgerful CLI not available: pipe broken".to_string()),
    });
    let mut sink = MemorySink::default();
    let request = common::ingest_request("assistant", "proceed under fail-open");
    let outcome = service
        .ingest_request(request, common::context(), &mut sink)
        .expect("fail-open must allow capture");

    let summaries = find_evidence_payloads(&outcome.events);
    assert_eq!(summaries.len(), 1);
    let ev = parse_status(summaries[0]);
    assert_eq!(
        ev.status,
        VerificationEvidenceStatus::Unavailable,
        "IPC failure must produce Unavailable evidence"
    );
    assert_ne!(ev.status, VerificationEvidenceStatus::Passed);
    assert!(
        outcome
            .events
            .iter()
            .any(|e| matches!(e.payload, Payload::AssistantFinalRecorded(_))),
        "assistant final must be recorded on fail-open"
    );
}

#[test]
fn verification_evidence__no_secret_looking_env_keys_in_blob() {
    let decision = GateDecision::ProceedUnavailable {
        reason: "down API_KEY=sk-live-secret TOKEN=abc DATABASE_PASSWORD=hunter2".to_string(),
    };
    let ev = VerificationEvidence::from_gate_decision(&decision);
    let summary = ev.to_summary().expect("serialize");
    assert!(
        !summary.contains("sk-live-secret"),
        "secret value leaked: {summary}"
    );
    assert!(
        !summary.contains("hunter2"),
        "password value leaked: {summary}"
    );
    assert!(
        !summary.contains("\"env\""),
        "env object must not be stored: {summary}"
    );
    assert!(
        !summary.contains("\"stdout\""),
        "stdout must not be stored: {summary}"
    );
    assert_eq!(ev.status, VerificationEvidenceStatus::Unavailable);

    // Wire through capture path as well.
    let service = service_with(MockBackend {
        response: None,
        error: Some("fail API_KEY=sk-xyz".to_string()),
    });
    let mut sink = MemorySink::default();
    let outcome = service
        .ingest_request(
            common::ingest_request("assistant", "ok"),
            common::context(),
            &mut sink,
        )
        .expect("fail-open");
    for s in find_evidence_payloads(&outcome.events) {
        assert!(!s.contains("sk-xyz"), "secret leaked via capture path: {s}");
    }
}

#[test]
fn pure_mapping__three_outcomes_distinct() {
    let passed =
        VerificationEvidence::from_gate_decision(&GateDecision::Proceed { verify: low_risk() });
    let blocked = VerificationEvidence::from_gate_decision(&GateDecision::Blocked {
        failure_probability: 0.8,
        drift_detected: false,
        risk_level: "high".into(),
        explanation: "block".into(),
    });
    let unavailable = VerificationEvidence::from_gate_decision(&GateDecision::ProceedUnavailable {
        reason: "ipc down".into(),
    });

    assert_eq!(passed.status, VerificationEvidenceStatus::Passed);
    assert_eq!(blocked.status, VerificationEvidenceStatus::Blocked);
    assert_eq!(unavailable.status, VerificationEvidenceStatus::Unavailable);
    assert_ne!(unavailable.status, VerificationEvidenceStatus::Passed);
}

/// Production `CaptureService::new()` installs the real gate (T149 Codex P1).
/// Without Ledgerful IPC this fails open and still emits Unavailable evidence.
#[test]
fn production_new__has_gate_and_emits_verification_evidence() {
    let service = CaptureService::new();
    assert!(
        service.has_verification_gate(),
        "CaptureService::new() must install a verification gate for production"
    );

    let mut sink = MemorySink::default();
    let request = common::ingest_request("assistant", "production path final");
    let outcome = service
        .ingest_request(request, common::context(), &mut sink)
        .expect("production gate must fail-open when Ledgerful is unreachable");

    let summaries = find_evidence_payloads(&outcome.events);
    assert_eq!(
        summaries.len(),
        1,
        "production capture must emit verification Evidence"
    );
    let ev = parse_status(summaries[0]);
    // Real backend either Passed (if ledgerful works) or Unavailable (IPC fail-open).
    // Never silent / no-evidence.
    assert!(
        matches!(
            ev.status,
            VerificationEvidenceStatus::Passed
                | VerificationEvidenceStatus::Unavailable
                | VerificationEvidenceStatus::Blocked
        ),
        "unexpected status: {:?}",
        ev.status
    );
    // Fail-open path is the common CI case without ledgerful installed.
    if ev.status == VerificationEvidenceStatus::Unavailable {
        assert!(
            outcome
                .events
                .iter()
                .any(|e| matches!(e.payload, Payload::AssistantFinalRecorded(_))),
            "fail-open must still record assistant final"
        );
    }
}

#[test]
fn new_without_verification_gate__no_evidence_events() {
    let service = CaptureService::new_without_verification_gate();
    assert!(!service.has_verification_gate());
    let mut sink = MemorySink::default();
    let outcome = service
        .ingest_request(
            common::ingest_request("assistant", "no gate"),
            common::context(),
            &mut sink,
        )
        .expect("capture");
    assert!(
        find_evidence_payloads(&outcome.events).is_empty(),
        "without gate, no verification evidence"
    );
    assert_eq!(outcome.events.len(), 1);
}

/// When the verification gate emits EvidenceRecorded before AssistantFinal,
/// `primary_event()` must return the assistant-final envelope (not evidence).
#[test]
fn primary_event__with_verification_evidence__is_assistant_final() {
    let service = CaptureService::new();
    assert!(service.has_verification_gate());
    let mut sink = MemorySink::default();
    let outcome = service
        .ingest_request(
            common::ingest_request("assistant", "primary id check"),
            common::context(),
            &mut sink,
        )
        .expect("capture with gate (fail-open ok)");

    let evidence = find_evidence_payloads(&outcome.events);
    assert!(
        !evidence.is_empty(),
        "production gate must emit verification evidence"
    );
    assert!(
        outcome.events.len() >= 2,
        "evidence + assistant final expected"
    );

    let primary = outcome.primary_event().expect("primary event must exist");
    assert!(
        matches!(primary.payload, Payload::AssistantFinalRecorded(_)),
        "primary must be AssistantFinalRecorded, got {:?}",
        primary.event_type
    );
    // first() would be evidence when gate runs — primary must differ.
    assert_ne!(
        outcome.events[0].event_id, primary.event_id,
        "primary must not be the first (evidence) event when gate emits evidence"
    );
}

// Silence unused helper warning if compiler is pedantic on older paths.
#[allow(dead_code)]
fn _use_helpers() {
    let _ = evidence_summaries(&MemorySink::default());
}
