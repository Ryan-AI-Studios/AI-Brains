# Track T27: E2E Hardening

## Context
Phase 12 is the final push for production readiness. We need to verify that all components work together seamlessly under load and in failure scenarios.

## Goals
- Implement E2E smoke tests for the entire capture-to-recall loop.
- Verify daemon concurrency with multiple simultaneous sessions.
- Implement and verify failure drills (corrupt projections, missing models, etc.).
- Ensure all CI gates pass with a clean clippy and fmt.

## Implementation Plan

### Phase 1: E2E Test Suite
- [ ] Create `tests/e2e` directory.
- [ ] Implement `cli_capture_smoke.rs`.
- [ ] Implement `daemon_concurrency.rs`.
- [ ] Implement `recovery_restore.rs`.

### Phase 2: Failure Drills
- [ ] Create `Docs/conductor/failure-drills.md`.
- [ ] Implement `scripts/run-failure-drills.ps1`.
- [ ] Verify "Kill daemon mid-ingest" drill.
- [ ] Verify "Corrupt projection rebuild" drill.

### Phase 3: Final Polishing
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo nextest run --workspace`.

## Progress
- [ ] Phase 1
- [ ] Phase 2
- [ ] Phase 3
