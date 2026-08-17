# Track Completion Audit — T257-WarningJsonStdoutHygiene

## Verdict: PASS WITH DEFERRED P3

Process-only P2 at review time (closeout not yet written) is **not** a product defect. Product DoD is met. Orchestrator classifies CX1-P2 as **already expected** — closeout is Phase 5 after the full gate.

## Scope Reviewed

- Branch `track/T257-warning-json-stdout-hygiene` vs `main`
- Commits `4c5a718` (red) + `1c0eed0` (green)
- `identity_warn.rs`, `main.rs`, `scope.rs` both emit sites, `governed_common.rs`, F8 pretty printers, hermetic suite, CAPABILITIES / PROTOCOL-COMPAT / CHANGELOG

## Requirement and DoD Matrix

| Requirement | Met/Partial/Unmet | Evidence |
|-------------|-------------------|----------|
| F1 stdout purity | Met | Record then flush; hermetics assert no SOOT on stdout |
| F2 JSON-effective silent | Met | `print_json_stdout` / `note_machine_stdout`; AC3/AC5/AC7/AC9 |
| F3/F24/F25/AC17 scope token both arms | Met | `scope.rs` local + daemon inject before `emit_json` |
| F5 remediator skip | Met | whoami / adopt-path consecutive tokens |
| F6 once/delay | Met | `OnceLock` + flush in `handle_cli_result` |
| F7 T240 AC4 | Met | `project_list__identity_mismatch__warn_on_stderr` |
| F8 helper location | Met | `identity_warn.rs`, not `format_resolve.rs` |
| F9 no DTO field | Met | `warnings[]` additive string only |
| F10 no pin bumps | Met | clap 4.6.1, no new crates |
| F11 hotspot extract | Met | `project.rs` 1514 (was ~1549) |
| F15 nightly keys | Met | AC7 no `warnings` key |
| AC1–AC17 | Met | hermetics + units + AC16 live classify |
| No live `.env` / no `cargo install` | Met | classify-only |

## Findings

**P0** None.

**P1** None.

**P2**

```
[P2] Track closeout artifacts still In Progress
Confidence: High
Requirement: DoD / conductor Completed
Location: conductor.md:204, deferred.md:222, plan.md Phase 4 remaining
Problem: Governance still In Progress while review.md claimed AC/DoD met.
Evidence: Unchecked Phase 4 gate/closeout boxes at review time.
Failure scenario: Declaring Completed before full gate.
Correction: Finish Phase 5 (full gate) then mark Completed / close deferred.
Verification: conductor Completed + deferred T257 row closed after gate exit 0.
Deferrable: No as a process gate; not a product defect.
```

**Orchestrator disposition:** **Already expected / process-timing.** Implement-track forbids marking Completed until the full gate exits 0. Product implementation is complete. Closeout follows the gate.

**P3** None.

## Completeness Sweep

No TODO/FIXME/placeholder in `identity_warn.rs`. Both scope emit sites wired. Doctor stays early-route.

## Wiring and Regression Review

`AppContext::from_cli` → `record_identity_mismatch` → command (`print_json_stdout` may `note_machine_stdout`) → `handle_cli_result` → `flush_identity_mismatch_warn` only if pending && !machine && !skip.

T240 human `project list` still eprints SOOT after the table (F6 order change; AC4 asserts presence).

## Verification Evidence

- Observed by orchestrator: clippy `-D warnings` exit 0; T257 hermetics 9/9; T240/T249/T255 targeted green; AC16 source bin concat parses.
- Workspace nextest 3023/3026 then 3/3 restore tests green after stopping the live daemon (unrelated environmental guard).
- Codex sandbox could not re-run hermetics (`tempdir` denied).

## Deferred Candidates

PATH-behind F13 (operator `cargo install`). T223 env-override still separate (F17).

## Completion Decision

Product: **PASS**. Governance closeout after full gate. No product re-review required unless the gate finds an in-scope failure.
