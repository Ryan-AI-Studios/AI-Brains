# T278 Independent Completion Audit

**Scope:** `track/T278-graph-density-preview` vs `origin/main`  
**HEAD:** `134944d`  
**Base:** `origin/main` `400dd78`  
**Verdict:** Product implementation appears correct, but the track is **not completion-ready** because required closeout verification and publish steps remain pending.

## P0 Findings

None.

## P1 Findings

### P1-001 — Required closeout gates are incomplete

`plan.md` and `review.md` still show Phase 4, full verification, cross-model review, and publish steps as pending. `conductor.md` remains **In Progress**.

Required before completion:

- Run the full workspace gate.
- Run `ledgerful verify --scope full`.
- Record the independent review result in `review.md`.
- Complete implement-track publish/hygiene steps.

Evidence: [plan.md](C:/dev/AI-Brains/conductor/tracks/trackT278-graph-density-preview/plan.md:120), [review.md](C:/dev/AI-Brains/conductor/tracks/trackT278-graph-density-preview/review.md:15)

## P2 Findings

### P2-001 — Fail-open behavior lacks an executable regression test

The implementation correctly returns `String`, catches session lookup and preview errors, logs warnings, and avoids `?` propagation in the session arm ([graph.rs](C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/graph.rs:271)).

However, no test forces `get_session_memories`, `memory_preview`, or lock failure and verifies that `graph neighbors` still exits successfully with an honest fallback. A future regression could reintroduce `?` without the suite detecting it.

Add a hermetic failure-path test for AC5.

### P2-002 — AC3 does not precisely verify the PREVIEW column

The integration test checks that a line contains `session`, `memories`, and any non-whitespace character. The final character assertion is tautological for any nonempty output line; it does not parse or specifically validate the PREVIEW field.

Evidence: [graph_live_projection.rs](C:/dev/AI-Brains/crates/ai-brains-cli/tests/graph_live_projection.rs:138)

Strengthen the assertion to validate the session row’s final column, preferably including the expected count and preview text.

## P3 Findings

None.

## Requirement and DoD Audit

| Area | Result |
|---|---|
| Session PREVIEW caption | Implemented: `{n} memories · first line` |
| Empty/whitespace handling | Implemented and unit-tested |
| Unicode-safe 80-character cap | Implemented through `truncate_preview_chars` and tested |
| Sorted session memory selection | Implemented |
| Skip-empty first preview | Implemented through pure `pick_first_nonempty` and tested |
| Session-arm fail-open behavior | Implemented; failure-path test missing |
| JSON schema | Frozen; `memory_id`, `neighbors`, and three `NeighborHit` keys remain unchanged |
| Pretty-row wiring | Reachable through `pretty_neighbor_rows` |
| Clap help | Additive `after_help` implemented and tested; `Command::after_help` remains supported by current clap documentation ([docs.rs](https://docs.rs/clap/latest/clap/struct.Command.html)) |
| Density floors | Untouched |
| Projector/event/contracts | Untouched |
| Live rebuild/install/pin boundaries | No evidence of prohibited actions; manual evidence says none performed |
| Documentation | CAPABILITIES, OPERATIONS, PROTOCOL-COMPAT, CHANGELOG, and skill documentation updated |
| Red stubs | Replaced by green implementation; no relevant production placeholders remain |
| Targeted gates | Orchestrator evidence reports passing targeted nextest, clippy, and fmt |
| Full workspace gate | Not run/evidenced |
| Ledgerful full verification | Not run; local Ledgerful database could not be opened |
| Publish/hygiene | Not complete |

## Verification Notes

- `cargo fmt --check`: passed locally.
- Local nextest rerun was blocked by read-only filesystem access to `target\debug\.cargo-lock`.
- `ai-brains preflight --summary`: unavailable because `AI_BRAINS_KEY` is missing.
- `ledgerful doctor` / status: unavailable because Ledgerful could not open its database.
- Branch is clean and contains no forbidden density/projector/doctor/contract changes.
- No deferred P3 items are proposed.

## Final Assessment

The product code satisfies the substantive T278 behavior, with two test-proof gaps. The track must remain open until P1-001 is completed and the P2 test gaps are addressed or explicitly resolved by the owner.