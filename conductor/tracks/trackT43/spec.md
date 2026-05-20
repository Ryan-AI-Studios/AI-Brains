# Specification: Track T43 - Predictive Verification Gating & Intervention Agent

## Objective
Inject a verification gate into AI-Brains that leverages ChangeGuard's predictive CI engine to block bad code from being finalized, and create a background RiskReviewAgent that proactively warns about dangerous temporal coupling.

## Architecture & Scope
1. **Ingest-Final Verification Gate (`ai-brains-capture`)**: Hook into the `ingest-final` workflow. Before committing a Final Response, execute an IPC call to `changeguard verify`. If verification fails or drift/risk is deemed too high, reject the `ingest-final` event and return the failure explanation to the AI harness so it can self-remediate.
2. **RiskReviewAgent (`ai-brains-brain`)**: A new background agent (`intervention.rs`) that listens for high-risk drift alerts from ChangeGuard's watcher. When triggered, queries the CozoDB reachability graph (via `CozoProxyBackend` from T42), evaluates the diff, and injects a warning into the AI harness before the user or agent proceeds further.

## Technical Constraints & Mandates
- **CQRS Integrity**: The verification gate is a command-side interceptor — it blocks event append, not queries.
- **Event Sourcing**: Rejected ingest attempts must be logged as compensating/audit events for provenance.
- **Fail-Open on IPC Failure**: If ChangeGuard IPC is unreachable during the gate check, the capture MUST proceed (fail-open). Never block capture due to a transient pipe failure.
- **Non-Blocking Intervention**: The RiskReviewAgent must run asynchronously — it must never block the capture pipeline.
- **Privacy**: Verification outcomes stored as events must inherit privacy from the associated session.

> **WARNING**: This track introduces a strict gating mechanism. If ChangeGuard predicts high failure probability, AI-Brains will block Final Response capture and instruct the AI to remediate. This changes the AI agent lifecycle.
