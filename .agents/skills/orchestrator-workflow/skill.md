---
name: orchestrator-workflow
description: Defines the standard operating procedure for orchestrating sub-agents, managing conductor tracks, maintaining the CI gate, and tracking provenance via ChangeGuard for the AI-Brains project. Trigger this skill when an AI acts as the Orchestrator to ensure consistent project delivery.
---

# Orchestrator Workflow (AI-Brains)

You are the **Orchestrator**. Your primary responsibility is to maintain the high-level project state, enforce architectural invariants (Event Sourcing, CQRS, Privacy), and coordinate specialized sub-agents through the Track system.

## The Conductor / Track System

AI-Brains uses a structured delivery mechanism known as **Tracks** (T00 to T27). Each track is a bounded unit of work. Statuses are maintained in the implementation plan and individual track files.

## ChangeGuard Integration

ChangeGuard tracks architectural provenance. Use it at these points:

| Phase | ChangeGuard Command | Purpose |
|-------|-------------------|---------|
| Start of Session | `changeguard ledger status` | Detect untracked drift before starting |
| Before implementation | `changeguard ledger start` | Begin transaction for the specific track |
| After implementation | `changeguard impact` | Check blast radius (e.g., unintended crate coupling) |
| Before commit | `changeguard verify` | Run Rust CI gate commands |
| On commit | `changeguard ledger commit` | Close transaction with summary + reason |

### Ledger Categories for AI-Brains

- `ARCHITECTURE` — Event sourcing, CQRS boundaries, crate structure.
- `SECURITY` — SQLCipher, secret scanning, privacy gates.
- `FEATURE` — CLI commands, harness adapters, retrieval logic.
- `INFRA` — Windows Task Scheduler, service install, CI/CD.
- `REFACTOR` — Internal cleanup without behavior change.
- `DOCS` — Track documentation, ADRs, PRD updates.

## The Standard Operating Procedure

### 1. Planning Phase
1. **Identify Track:** Consult `Docs/Implementation-Plan.md` for the next uncompleted track.
2. **Historical Recall:** Run `ai-brains recall "<track topic>"` to retrieve past decisions or constraints relevant to this work.
3. **Analyze Couplings:** Run `changeguard hotspots` to identify if the target crate is brittle.
4. **Delegate Planning:** Invoke `architecture-planner`.
5. **Register:** Create `tracks/T<ID>-<name>.md` and update `Docs/Implementation-Plan.md` status to `[IN PROGRESS]`.
6. **Start Transaction:** `changeguard ledger start T<ID>-<name> --category <CAT>`

### 2. Implementation Phase
1. **Delegate Implementation:** Invoke `generalist`.
2. **TDD Loop:** Red (failing test) -> Green (implementation) -> Refactor.
3. **Impact Check:** `changeguard impact`. Ensure logic hasn't leaked across crate boundaries.

### 3. Verification Phase (The CI Gate)
Ensure the workspace passes the full gate:
```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; changeguard verify
```

### 4. Finalization Phase
1. **Durable Ingest:** Manually ingest any significant architectural decisions or newly discovered constraints:
   `powershell .agents/skills/ai-brains/scripts/ingest.ps1 -Content "DECISION: <...> | RATIONALE: <...>" -Role assistant`
2. **Close Track:** Mark as `[COMPLETED]` in `Docs/Implementation-Plan.md`.
3. **Commit with Ledger:** `changeguard ledger commit --tx-id <ID> --category <CAT> --summary "Implemented Track <ID>"`
4. **Audit:** Run `changeguard ledger status` to ensure a clean baseline for the next track.

## Orchestrator Rules of Engagement

*   **Capture Independence**: Ensure capture code never depends on `ai-brains-models` or `ai-brains-graph`.
*   **One Writer Rule**: Do not run parallel implementation agents on the same crate.
*   **Rust Idioms**: Enforce strict typing, zero-cost abstractions, and error handling.
*   **Privacy Propagation**: Verify that any new memory derivation inherits privacy tags.
