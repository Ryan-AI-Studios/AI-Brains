# Track T22 — Nightly Summaries

## Owner
architecture-planner

## Status
In Progress

## Objective
Implement the nightly background process that generates summaries for completed sessions and daily activity.

## Scope
- Scaffold `ai-brains-brain` crate (brain orchestration).
- Implement `NightlyService` to find unsummarized sessions.
- Implement prompt templates for session summarization.
- Implement `SummaryProjector` to update `memory_projection`.
- Emit `NightlyJobStarted` and `SessionSummaryCreated` events.

## Out of Scope
- Scheduler/schtasks integration (Phase 11).
- Advanced RAPTOR clustering (Phase 10).

## Files Owned
- `crates/ai-brains-brain/**`

## Files Allowed To Touch
- `Cargo.toml`
- `Docs/conductor/conductor.md`
- `Docs/status.md`
- `crates/ai-brains-store/**` (if migrations needed)

## Files Forbidden To Touch
- `crates/ai-brains-core/**`
- `crates/ai-brains-events/**`

## Public Contracts Consumed
- `ai_brains_models::ModelProvider`
- `ai_brains_store::EventStore`

## Public Contracts Produced
- `ai_brains_brain::NightlyService`

## Required Tests First
- `tests/nightly_summarizes_session.rs`
- `tests/nightly_skips_summarized_sessions.rs`

## Implementation Steps
1. [ ] Scaffold `ai-brains-brain` crate and add to workspace.
2. [ ] Implement `NightlyService` loop.
3. [ ] Implement session summarization logic with `ModelProvider`.
4. [ ] Implement event emission for summaries.
5. [ ] Verification and CI gate.

## Failure Modes To Handle
- Model provider offline (graceful degradation, skip summary, record partial failure).
- Database locked.
- Malformed context (too many words).

## Security Requirements
- Privacy levels respected during summarization.
- No cloud calls for `local_only` sessions.

## Acceptance Criteria
- Nightly service correctly summarizes a session with 3 turns.
- Summaries are emitted as events and projected into `memory_projection`.
- CI pass with clippy and nextest.

## Handoff Notes
- Phase 9 final track.
