# Track Completion Audit — T284-RetentionWorkSamples

## Verdict: FAIL

The functional fix is sound for the two reported regressions. Track completion is not clear because final verification and governance are unfinished, and AC7’s claimed event-log proof is not actually present.

## P0 — Blockers

None.

## P1 — Completion blocker

### P1-01 — Required closeout, full gate, and durable review evidence are incomplete

Evidence:

- The full workspace gate and `ledgerful verify --scope full` have not run.
- [plan.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/plan.md:3) remains `Pending`, with every phase and DoD item unchecked.
- [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/spec.md:4) remains `Planned`.
- [conductor.md](/C:/dev/AI-Brains/conductor/conductor.md:231) says `In Progress`, while the series README and deferred register still say `Planned`.
- [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/review.md:4) explicitly says CX and full gate are pending. It is also ignored by `.gitignore` and absent from the branch/diff, so the review record is not durable.
- The working tree still has an uncommitted change in [retention.rs](/C:/dev/AI-Brains/crates/ai-brains-contracts/src/retention.rs:152).
- No tracked PR/CI/publish closeout is present.

Required fix: run and record the full gate, obtain clean Ledgerful status, track the final review log, reconcile every status/checklist surface, commit all intentional changes, then complete the approved PR/CI/squash-merge workflow.

## P2 — Major

### P2-01 — AC7’s event-log immutability regression is not actually tested

AC7 requires the event-log count to remain unchanged across `retention plan` in [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/spec.md:175).

The cited hermetic test instead compares `memory list --summary` output before and after plan at [retention_plan_human.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/retention_plan_human.rs:176) and [retention_plan_human.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/tests/retention_plan_human.rs:270). That test would still pass if plan appended an unrelated `RetentionApplied` event without changing the memory projection.

The production path currently appears read-only, but the required regression proof is missing and [review.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/review.md:35) overstates the evidence.

Required fix: directly count `events` or compare `read_all_events()` before and after a hermetic CLI plan invocation.

## P3 — Minors

### P3-01 — F27 visibility and the resulting public API do not agree

F27 requires `class_dispose_count` to be `pub(crate)` in [spec.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/spec.md:147). It is instead public and re-exported from the crate root in [retention.rs](/C:/dev/AI-Brains/crates/ai-brains-contracts/src/retention.rs:154) and [lib.rs](/C:/dev/AI-Brains/crates/ai-brains-contracts/src/lib.rs:24).

The cross-crate use explains why `pub(crate)` is not viable as written, but the frozen requirement and API surface still need reconciliation. Either keep the helper local to control-plane or amend F27 to authorize the public API.

### P3-02 — Scoped diff fails whitespace validation

`git diff --check origin/main` reports trailing whitespace in [agy-review.md](/C:/dev/AI-Brains/conductor/tracks/trackT284-retention-work-samples/agy-review.md:3) through line 6.

This is easy hygiene and should be fixed directly.

## Requirement and DoD audit

| Requirement | Result |
|---|---|
| F0 | PASS: red and green commits plus BUGFIX TX evidence exist |
| F1–F3 | PASS: Work uses per-class dispose counts; one bucket remains; dominant mechanism unchanged |
| F4–F5 | PASS: optional skip-if-zero fields; per-class counters; CE-first dispose samples |
| F6 | PASS: one Work row per non-zero mechanism with fallback samples |
| F7–F8 | PASS: disposal-only audit samples; class counts remain dominant/count |
| F9–F20 | PASS: overlay, apply gates, isolation, versions, and stated non-goals remain intact |
| F21 | PASS: red commit precedes green; required named regression cases exist |
| F22 | INCOMPLETE: this review ran, but durable review/closeout remains pending |
| F23–F26 | PASS: correct debt file, docs, existing behavior, and touch boundaries |
| F27 | DEVIATION: public `class_dispose_count` contradicts `pub(crate)` requirement |
| F28–F39 | PASS |
| F40 | INCOMPLETE: targeted gates reported green; full workspace gate pending |
| F41 | PASS: both same-file helper tests exist |

| Acceptance criterion | Result |
|---|---|
| AC1–AC6 | PASS |
| AC7 | PRODUCT CODE PASS; REQUIRED REGRESSION PROOF MISSING |
| AC8–AC12 | PASS |
| AC13 | Optional dogfood reported; independently blocked by missing vault key; no live apply performed |
| AC14 | Targeted clippy reported green; full-workspace clippy still pending |
| AC15 | PASS by source inspection |
| AC16–AC17 | PASS |

## Completeness sweep

No production placeholders, stubs, new ignored tests, fake values, no-op branches, plaintext-body exposure, dependency changes, migrations, or forbidden production `unwrap`/`expect`/`panic` were found.

Production wiring is complete:

1. `build_report` computes class disposal counters and samples.
2. Plan and both apply implementations consume that report.
3. Pretty output uses disposal counters rather than the dominant mechanism.
4. All `append_retention_applied` callers use `audit_sample_ids`.
5. Overlay-only reports retain pin samples and `Nothing to dispose.` behavior.
6. JSON extras remain additive and omitted at zero.

## Verification

Independent read-only results:

- Contracts retention units: **6 passed**
- Control-plane `audit_sample_ids`: **2 passed**
- CLI pretty formatter: **10 passed**
- CLI retention clap units: **2 passed**
- `cargo fmt --check`: **PASS**
- `retention plan --format xml`: exit **2**
- Dependency/lockfile diff: none
- `git diff --check`: **FAIL** on four trailing-whitespace lines
- Integration reruns were blocked by sandbox denial creating temporary database files; the recorded targeted run reports **39 passed**
- Full workspace gate: **not run**
- Ledgerful doctor/status: unavailable under read-only database access; the TX identifier is present in ledger storage, but authoritative clean status could not be obtained

## Deferred candidates

None. Both P3 items are straightforward and should be fixed directly.

## Completion decision

**FAIL.** The main retention behavior is correctly implemented, but T284 must not be marked complete until P1/P2 are resolved and the full closeout workflow is green.