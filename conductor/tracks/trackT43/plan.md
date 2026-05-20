## Plan: Track T43 - Predictive Verification Gating & Intervention Agent

### Phase 1: Ingest-Final Verification Gate
- [x] Task 1.1: In `crates/ai-brains-capture/src/verification_gate.rs`, add `VerificationGate` struct with configurable threshold that calls `changeguard bridge export` via IPC.
- [x] Task 1.2: Parse verification response for failure probability, drift status, and risk level via NDJSON parsing of `verification_summary`, `hotspot`, `drift_delta` record kinds.
- [x] Task 1.3: On high failure probability or drift, return `CaptureError::VerificationGateRejected` with structured error payload containing failure explanation.
- [x] Task 1.4: Implement fail-open: if IPC is unreachable, `check()` returns `Proceed` with a warning logged.
- [x] Task 1.5: Log rejected ingest attempts — `IngestGateRejectedPayload` event type registered in events crate with session_id, reason, failure_probability, drift_detected, risk_level, explanation.
- [x] Task 1.6: Write 11 unit tests covering gate decisions, fail-open, custom thresholds, and NDJSON parsing.
- [x] Task 1.7: `CaptureService` now has `with_verification_gate()` constructor; gate only activates for "assistant" role events.

### Phase 2: RiskReviewAgent (Intervention)
- [x] Task 2.1: Create `crates/ai-brains-brain/src/intervention.rs` with `RiskReviewAgent` struct.
- [x] Task 2.2: Implement listener polling ChangeGuard via `bridge export --hotspots` for `record_kind = "risk_alert"` records.
- [x] Task 2.3: On alert, query `CozoProxyBackend::query_path()` to assess blast radius of risky change.
- [x] Task 2.4: Format and inject warning payload (`InterventionWarning`) with alert, blast_radius, and suggestion for AI harness self-correction.
- [x] Task 2.5: Non-blocking: uses `tokio::task::spawn_blocking` for IPC; `run()` is async for background `tokio::spawn`.
- [x] Task 2.6: Deduplication via `Mutex<HashSet<(String,String)>>` tracking seen (source_file, target_file) pairs; gracefully returns empty vec when services unavailable.
- [x] Task 2.7: Write 4 unit tests: NDJSON parsing, deduplication, blast-radius degradation, warning formatting.

### Phase 3: Verification
- [x] Task 3.1: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace` — ALL PASS (151+ tests, 0 failures)
- [x] Task 3.2: Integration test: gate correctly blocks on high failure probability and fails-open on IPC error.
- [x] Task 3.3: Integration test: RiskReviewAgent polls and deduplicates correctly.

> **WARNING**: This track changes the AI agent lifecycle. Capture blocks on high risk.
